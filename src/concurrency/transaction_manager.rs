use std::{collections::HashMap, sync::Mutex};

use crate::{common::SlotOffset, concurrency::{Transaction, transaction::UndoLink}};

#[allow(dead_code)]
pub(crate) struct PageVersionInfo {
    prev_link: Mutex<HashMap<SlotOffset, UndoLink>>
}

pub struct TransactionManager {}

impl TransactionManager {
    pub fn new_txn(&self) -> Transaction {
        todo!("")
    }

    pub fn commit_txn(&self, _txn: &Transaction) {
        todo!("")
    }

    pub fn abort_txn(&self, _txn: &Transaction) {
        todo!("")
    }
}
