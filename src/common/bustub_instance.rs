use std::sync::{Arc, Mutex};

use crate::{
    buffer::buffer_pool_manager::BufferPoolManager,
    catalog::Catalog,
    concurrency::{LockManager, Transaction, TransactionManager},
    storage::disk::disk_scheduler::DiskManager,
};

pub struct BustubInstance {
    pub(crate) disk_maanger: Box<dyn DiskManager>,
    pub(crate) bpm: Arc<BufferPoolManager>,
    pub(crate) lock_manager: LockManager,
    pub(crate) txn_manager: TransactionManager,
    pub(crate) catalog: Mutex<Catalog>,
    curr_txn: Option<Arc<Transaction>>,
}

impl BustubInstance {
}
