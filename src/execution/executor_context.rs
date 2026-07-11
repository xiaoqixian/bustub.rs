use std::sync::Arc;

use crate::{buffer::buffer_pool_manager::BufferPoolManager, catalog::CatalogRef, concurrency::{LockManager, Transaction}};

pub struct ExecutorContext {
    pub txn: Option<Arc<Transaction>>,
    pub bpm: Arc<BufferPoolManager>,
    pub catalog: CatalogRef,
    pub lock_mgr: Arc<LockManager>,
    pub is_delete: bool,
}


