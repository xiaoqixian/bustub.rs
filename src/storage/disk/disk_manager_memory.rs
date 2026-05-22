use std::sync::Mutex;
use crate::common::PageId;
use crate::storage::disk::disk_scheduler::DiskManager;

/// A simple in-memory DiskManager for testing.
#[allow(dead_code)]
pub struct DiskManagerMemory {
    pages: Mutex<std::collections::HashMap<PageId, Vec<u8>>>,
}

#[allow(dead_code)]
impl DiskManagerMemory {
    pub fn new() -> Self {
        Self {
            pages: Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl DiskManager for DiskManagerMemory {
    fn write_page(&self, page_id: PageId, page_data: &[u8]) {
        let mut pages = self.pages.lock().unwrap();
        pages.insert(page_id, page_data.to_vec());
    }

    fn read_page(&self, page_id: PageId, page_data: &mut [u8]) {
        let pages = self.pages.lock().unwrap();
        if let Some(data) = pages.get(&page_id) {
            let len = data.len().min(page_data.len());
            page_data[..len].copy_from_slice(&data[..len]);
        }
    }

    fn increase_disk_space(&self, _pages: usize) {
        // no-op for in-memory manager
    }

    fn delete_page(&self, page_id: PageId) {
        let mut pages = self.pages.lock().unwrap();
        pages.remove(&page_id);
    }
}

