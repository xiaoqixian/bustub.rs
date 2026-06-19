//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// transaction_manager.rs
//
// Identification: src/concurrency/transaction_manager.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, RwLock};

use crate::common::errors::BustubError;
use crate::common::rid::RID;
use crate::common::{PageId, SlotOffset, TxnId};
use crate::concurrency::transaction::{IsolationLevel, TransactionState, UndoLink, UndoLog};
use crate::concurrency::watermark::Watermark;
use crate::concurrency::Transaction;
use crate::storage::table::tuple::TimeStamp;

/// Transaction identifier start value (first txn id).
const TXN_START_ID: TxnId = 1i64 << 62;

/// PageVersionInfo stores the previous version link of each tuple slot on a page.
/// Each page in the table heap has its own PageVersionInfo with a per-page lock.
#[derive(Default)]
pub(crate) struct PageVersionInfo {
    /// Protects the prev_link map (per-page lock).
    /// DO NOT use `[x]` to access it because it will create new elements.
    /// Use `get` / `find` instead.
    prev_link: RwLock<HashMap<SlotOffset, UndoLink>>,
}

/// TransactionManager keeps track of all the transactions running in the system.
pub struct TransactionManager {
    /// Protects txn_map and running_txns (watermark). These are protected together
    /// because the watermark tracks the read timestamps of running transactions.
    /// All transactions (running or committed) are stored in the map.
    txn_map: RwLock<(HashMap<TxnId, Arc<Transaction>>, Watermark)>,

    /// The last committed timestamp.
    #[allow(dead_code)]
    last_commit_ts: AtomicI64,

    /// The next transaction ID to assign.
    next_txn_id: AtomicI64,

    /// Stores the previous version of each tuple in the table heap.
    /// Each page entry is wrapped in Arc to allow releasing the global version_info lock
    /// while keeping the page-level PageVersionInfo alive.
    /// Do not directly access this field. Use the helper functions in this module.
    version_info: RwLock<HashMap<PageId, Arc<PageVersionInfo>>>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            txn_map: RwLock::new((HashMap::new(), Watermark::new(0))),
            last_commit_ts: AtomicI64::new(0),
            next_txn_id: AtomicI64::new(TXN_START_ID),
            version_info: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new transaction with the default isolation level (Snapshot Isolation).
    pub fn new_txn(&self) -> Result<Arc<Transaction>, BustubError> {
        self.new_txn_with_iso_level(IsolationLevel::SnapshotIsolation)
    }

    /// Begin a new transaction with the given isolation level.
    ///
    /// Allocates a new transaction ID, creates the transaction object,
    /// adds it to the transaction map, and registers it in the watermark.
    ///
    /// This corresponds to the C++ `Begin()` method.
    pub fn new_txn_with_iso_level(
        &self,
        iso_level: IsolationLevel,
    ) -> Result<Arc<Transaction>, BustubError> {
        let txn_id = self.next_txn_id.fetch_add(1, Ordering::Relaxed);
        let txn = Arc::new(Transaction::new(txn_id, iso_level));

        // Acquire exclusive lock on txn_map and watermark (protected together).
        let mut guard = self.txn_map.write().expect("txn_map write lock");
        guard.0.insert(txn_id, Arc::clone(&txn));
        // TODO(fall2023): set the timestamps here. Watermark updated below.
        guard.1.add_txn(txn.get_read_ts());

        Ok(txn)
    }

    /// Commit a transaction.
    ///
    /// Verifies that the transaction is in the RUNNING state. For SERIALIZABLE isolation,
    /// performs serial verification. On success, sets the transaction state to COMMITTED,
    /// updates the watermark's commit timestamp and removes this transaction from the watermark.
    ///
    /// This corresponds to the C++ `Commit()` method, but without the separate commit_mutex_,
    /// since the exclusive lock on txn_map serves as the commit serialization point.
    pub fn commit_txn(&self, txn: &Transaction) -> Result<(), BustubError> {
        // TODO(fall2023): acquire commit ts!

        // Check that the transaction is in the RUNNING state.
        if txn.get_state() != TransactionState::Running {
            return Err(BustubError::Message("txn not in running state".to_string()));
        }

        // For serializable isolation, verify the transaction.
        if txn.get_isolation_level() == IsolationLevel::Serializable {
            if !self.verify_txn(txn) {
                // Verification failed, abort the transaction.
                self.abort_txn(txn)?;
                return Err(BustubError::Message(
                    "txn verification failed".to_string(),
                ));
            }
        }

        // TODO(fall2023): Implement the commit logic!

        // Acquire exclusive lock on txn_map (also serves as commit serialization).
        let mut guard = self.txn_map.write().expect("txn_map write lock");

        // TODO(fall2023): set commit timestamp + update last committed timestamp here.

        txn.set_state(TransactionState::Committed);

        // Update watermark: advance the commit timestamp and remove this txn's read_ts.
        guard.1.update_commit_ts(txn.get_commit_ts());
        guard.1.remove_txn(txn.get_read_ts());

        Ok(())
    }

    /// Abort a transaction.
    ///
    /// Verifies that the transaction is in the RUNNING or TAINTED state.
    /// Sets the transaction state to ABORTED and removes it from the watermark.
    ///
    /// This corresponds to the C++ `Abort()` method.
    pub fn abort_txn(&self, txn: &Transaction) -> Result<(), BustubError> {
        let state = txn.get_state();
        if state != TransactionState::Running && state != TransactionState::Tainted {
            return Err(BustubError::Message(
                "txn not in running / tainted state".to_string(),
            ));
        }

        // TODO(fall2023): Implement the abort logic!

        // Acquire exclusive lock on txn_map and watermark.
        let mut guard = self.txn_map.write().expect("txn_map write lock");

        txn.set_state(TransactionState::Aborted);
        guard.1.remove_txn(txn.get_read_ts());

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Version Info (Undo Link) Management
    //
    // These methods correspond to the helper functions in the C++
    // transaction_manager_impl.cpp.
    // -----------------------------------------------------------------------

    /// Update an undo link that links a table heap tuple to the first undo log.
    ///
    /// Before updating, the `check` function will be called with the current undo link
    /// (or `None` if no link exists) to ensure validity. Returns `false` if the check fails.
    ///
    /// This corresponds to the C++ `UpdateUndoLink()` method.
    pub fn update_undo_link<F: Fn(Option<UndoLink>) -> bool>(
        &self,
        rid: RID,
        prev_link: Option<UndoLink>,
        check: Option<F>,
    ) -> bool {
        let page_id = rid.page_id();
        let slot_num = rid.slot_num() as SlotOffset;

        let pg_ver_info = {
            // Acquire exclusive lock on the global version_info to find or create the page entry.
            let mut ver_guard = self.version_info.write().expect("version_info write lock");
            ver_guard
                .entry(page_id)
                .or_insert_with(|| Arc::new(PageVersionInfo::default()))
                .clone()
        };

        // Acquire the per-page lock on the prev_link map.
        let mut link_guard = pg_ver_info.prev_link.write().expect("prev_link write lock");

        // Run the check function if provided.
        let existing_link = link_guard.get(&slot_num).copied();
        if let Some(ref check_fn) = check {
            if !check_fn(existing_link) {
                return false;
            }
        }

        // Update or remove the undo link.
        if let Some(link) = prev_link {
            link_guard.insert(slot_num, link);
        } else {
            link_guard.remove(&slot_num);
        }

        true
    }

    /// Get the first undo link of a table heap tuple.
    /// Returns `None` if the RID has no undo link.
    ///
    /// This corresponds to the C++ `GetUndoLink()` method.
    pub fn get_undo_link(&self, rid: RID) -> Option<UndoLink> {
        let page_id = rid.page_id();
        let slot_num = rid.slot_num() as SlotOffset;

        // Clone the Arc so we can release the global lock.
        let pg_ver_info = {
            let ver_guard = self.version_info.read().expect("version_info read lock");
            ver_guard.get(&page_id)?.clone()
        };

        // Acquire exclusive lock on the per-page prev_link map.
        let link_guard = pg_ver_info.prev_link.write().expect("prev_link write lock");

        link_guard.get(&slot_num).copied()
    }

    /// Access the transaction undo log buffer and get the undo log.
    /// Returns `None` if the transaction does not exist.
    ///
    /// This corresponds to the C++ `GetUndoLogOptional()` method.
    pub fn get_undo_log_optional(&self, link: UndoLink) -> Option<UndoLog> {
        let txn = {
            let guard = self.txn_map.read().expect("txn_map read lock");
            guard.0.get(&link.prev_txn)?.clone()
        };

        Some(txn.get_undo_log(link.prev_log_idx))
    }

    /// Access the transaction undo log buffer and get the undo log.
    /// Returns an error if the undo log does not exist.
    ///
    /// This corresponds to the C++ `GetUndoLog()` method.
    pub fn get_undo_log(&self, link: UndoLink) -> Result<UndoLog, BustubError> {
        self.get_undo_log_optional(link)
            .ok_or_else(|| BustubError::Message("undo log not exist".to_string()))
    }

    /// Get the lowest read timestamp in the system (the watermark).
    ///
    /// This corresponds to the C++ `GetWatermark()` method.
    pub fn get_watermark(&self) -> TimeStamp {
        let guard = self.txn_map.read().expect("txn_map read lock");
        guard.1.get_watermark()
    }

    /// Stop-the-world garbage collection.
    /// Will be called only when all transactions are not accessing the table heap.
    ///
    /// This corresponds to the C++ `GarbageCollection()` method.
    pub fn garbage_collection(&self) {
        // Not implemented yet.
        unimplemented!("GarbageCollection not implemented");
    }

    /// Verify if a transaction satisfies serializability.
    /// This is a placeholder that currently always returns true.
    ///
    /// This corresponds to the C++ `VerifyTxn()` method.
    fn verify_txn(&self, _txn: &Transaction) -> bool {
        true
    }
}
