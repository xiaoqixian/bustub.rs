//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// transaction.rs
//
// Identification: src/concurrency/transaction.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;
use std::thread::ThreadId;

use crate::catalog::TableOid;
use crate::common::rid::RID;
use crate::common::{TxnId, INVALID_TXN_ID};
use crate::storage::table::tuple::{TimeStamp, Tuple, INVALID_TS};

/// Transaction identifier start value (first txn id).
const TXN_START_ID: TxnId = 1i64 << 62;

/// Placeholder type for abstract expression reference.
pub type AbstractExpressionRef = String;

/// Transaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Running,
    Tainted,
    Committed,
    Aborted,
}

/// Transaction isolation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,
    SnapshotIsolation,
    Serializable,
}

/// Represents a link to a previous version of this tuple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UndoLink {
    /// Previous version can be found in which txn.
    pub prev_txn: TxnId,
    /// The log index of the previous version in `prev_txn`.
    pub prev_log_idx: i32,
}

impl UndoLink {
    /// Create a new UndoLink pointing to nothing.
    pub fn new() -> Self {
        UndoLink {
            prev_txn: INVALID_TXN_ID,
            prev_log_idx: 0,
        }
    }

    /// Checks if the undo link points to something.
    pub fn is_valid(&self) -> bool {
        self.prev_txn != INVALID_TXN_ID
    }
}

impl Default for UndoLink {
    fn default() -> Self {
        Self::new()
    }
}

/// Undo log entry for a transaction.
#[derive(Clone)]
pub struct UndoLog {
    /// Whether this log is a deletion marker.
    pub is_deleted: bool,
    /// The fields modified by this undo log.
    pub modified_fields: Vec<bool>,
    /// The modified fields.
    pub tuple: Tuple,
    /// Timestamp of this undo log.
    pub ts: TimeStamp,
    /// Undo log previous version.
    pub prev_version: UndoLink,
}

/// Internal fields of a transaction protected by a mutex.
struct TransactionInner {
    /// Store undo logs. Other undo logs / table heap will store (txn_id, index) pairs,
    /// and therefore you should only append to this vector or update things in-place
    /// without removing anything.
    undo_logs: Vec<UndoLog>,
    /// Stores the RIDs of write tuples.
    write_set: HashMap<TableOid, HashSet<RID>>,
    /// Stores all scan predicates.
    scan_predicates: HashMap<TableOid, Vec<AbstractExpressionRef>>,
}

/// Transaction tracks information related to a transaction.
pub struct Transaction {
    /// The state of this transaction (protected by mutex).
    state: Mutex<TransactionState>,
    /// The read timestamp (lock-free atomic access).
    read_ts: AtomicI64,
    /// The commit timestamp (lock-free atomic access).
    commit_ts: AtomicI64,
    /// The inner fields protected by a mutex (undo_logs, write_set, scan_predicates).
    inner: Mutex<TransactionInner>,
    /// The isolation level of the transaction. Set at creation and never changed.
    isolation_level: IsolationLevel,
    /// The thread ID from which the txn starts. Set at creation and never changed.
    thread_id: ThreadId,
    /// The ID of this transaction. Set at creation and never changed.
    txn_id: TxnId,
}

#[allow(dead_code)]
impl Transaction {
    /// Create a new transaction with the given `txn_id` and `isolation_level`.
    pub fn new(txn_id: TxnId, isolation_level: IsolationLevel) -> Self {
        Transaction {
            state: Mutex::new(TransactionState::Running),
            read_ts: AtomicI64::new(0),
            commit_ts: AtomicI64::new(INVALID_TS),
            inner: Mutex::new(TransactionInner {
                undo_logs: Vec::new(),
                write_set: HashMap::new(),
                scan_predicates: HashMap::new(),
            }),
            isolation_level,
            thread_id: std::thread::current().id(),
            txn_id,
        }
    }

    /// Get the ID of the thread running the transaction.
    pub fn get_thread_id(&self) -> ThreadId {
        self.thread_id
    }

    /// Get the ID of this transaction.
    pub fn get_transaction_id(&self) -> TxnId {
        self.txn_id
    }

    /// Get the ID of this transaction, stripping the highest bit.
    /// NEVER use/store this value unless for debugging.
    pub fn get_transaction_id_human_readable(&self) -> TxnId {
        self.txn_id ^ TXN_START_ID
    }

    /// Get the temporary timestamp of this transaction.
    pub fn get_transaction_temp_ts(&self) -> TimeStamp {
        self.txn_id
    }

    /// Get the isolation level of this transaction.
    pub fn get_isolation_level(&self) -> IsolationLevel {
        self.isolation_level
    }

    /// Get the transaction state.
    pub fn get_state(&self) -> TransactionState {
        *self.state.lock().expect("txn state lock")
    }

    /// Get the read timestamp.
    pub fn get_read_ts(&self) -> TimeStamp {
        self.read_ts.load(Ordering::Acquire)
    }

    /// Get the commit timestamp.
    pub fn get_commit_ts(&self) -> TimeStamp {
        self.commit_ts.load(Ordering::Acquire)
    }

    /// Modify an existing undo log at the given index.
    pub fn modify_undo_log(&self, log_idx: usize, new_log: UndoLog) {
        let mut inner = self.inner.lock().expect("txn inner lock");
        inner.undo_logs[log_idx] = new_log;
    }

    /// Append an undo log and return the UndoLink pointing to it.
    pub fn append_undo_log(&self, log: UndoLog) -> UndoLink {
        let mut inner = self.inner.lock().expect("txn inner lock");
        inner.undo_logs.push(log);
        UndoLink {
            prev_txn: self.txn_id,
            prev_log_idx: (inner.undo_logs.len() - 1) as i32,
        }
    }

    /// Append a RID to the write set for a given table.
    pub fn append_write_set(&self, t: TableOid, rid: RID) {
        let mut inner = self.inner.lock().expect("txn inner lock");
        inner.write_set.entry(t).or_default().insert(rid);
    }

    /// Get a clone of the write sets.
    pub fn get_write_sets(&self) -> HashMap<TableOid, HashSet<RID>> {
        let inner = self.inner.lock().expect("txn inner lock");
        inner.write_set.clone()
    }

    /// Append a scan predicate for a given table.
    pub fn append_scan_predicate(&self, t: TableOid, predicate: AbstractExpressionRef) {
        let mut inner = self.inner.lock().expect("txn inner lock");
        inner.scan_predicates.entry(t).or_default().push(predicate);
    }

    /// Get a clone of the scan predicates.
    pub fn get_scan_predicates(&self) -> HashMap<TableOid, Vec<AbstractExpressionRef>> {
        let inner = self.inner.lock().expect("txn inner lock");
        inner.scan_predicates.clone()
    }

    /// Get a clone of the undo log at the given index.
    pub fn get_undo_log(&self, log_id: usize) -> UndoLog {
        let inner = self.inner.lock().expect("txn inner lock");
        inner.undo_logs[log_id].clone()
    }

    /// Get the number of undo logs.
    pub fn get_undo_log_num(&self) -> usize {
        let inner = self.inner.lock().expect("txn inner lock");
        inner.undo_logs.len()
    }

    /// Clear the undo logs and return the number of logs before clearing.
    ///
    /// Use this function in leaderboard benchmarks for online garbage collection.
    /// For stop-the-world GC, simply remove the txn from the txn_map.
    pub fn clear_undo_log(&self) -> usize {
        let mut inner = self.inner.lock().expect("txn inner lock");
        let len = inner.undo_logs.len();
        inner.undo_logs.clear();
        len
    }

    /// Set the transaction state to Tainted.
    pub fn set_tainted(&self) {
        *self.state.lock().expect("txn state lock") = TransactionState::Tainted;
    }

    // -----------------------------------------------------------------------
    // Methods for TransactionManager (pub(crate) visibility)
    // -----------------------------------------------------------------------

    /// Set the transaction state. Used by TransactionManager.
    pub(crate) fn set_state(&self, state: TransactionState) {
        *self.state.lock().expect("txn state lock") = state;
    }

    /// Set the read timestamp. Used by TransactionManager.
    pub(crate) fn set_read_ts(&self, ts: TimeStamp) {
        self.read_ts.store(ts, Ordering::Release);
    }

    /// Set the commit timestamp. Used by TransactionManager.
    pub(crate) fn set_commit_ts(&self, ts: TimeStamp) {
        self.commit_ts.store(ts, Ordering::Release);
    }
}
