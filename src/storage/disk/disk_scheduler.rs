//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// disk_scheduler.rs
//
// Identification: src/storage/disk/disk_scheduler.rs
//
// Copyright (c) 2015-2024, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::common::PageId;
use crate::common::BUSTUB_PAGE_SIZE;

// ---------------------------------------------------------------------------
// DiskManager trait
// ---------------------------------------------------------------------------

/// Trait that mirrors the C++ `DiskManager` interface required by
/// `DiskScheduler`. Any type implementing this trait can serve as a disk
/// backend (e.g., an in-memory test disk or a file-based disk manager).
pub trait DiskManager: Send + Sync {
    /// Write a page to the database file.
    fn write_page(&self, page_id: PageId, page_data: &[u8]);

    /// Read a page from the database file.
    fn read_page(&self, page_id: PageId, page_data: &mut [u8]);

    /// Increases the size of the database file to fit the specified number of
    /// pages. This works like a dynamic array, where the capacity is doubled
    /// until all pages can fit.
    fn increase_disk_space(&self, pages: usize);

    /// Deallocates a page on disk.
    ///
    /// Note: This is a no-op without a more complex data structure to track
    /// deallocated pages.
    fn delete_page(&self, page_id: PageId);
}

// ---------------------------------------------------------------------------
// DiskRequest
// ---------------------------------------------------------------------------

enum DataWithDirection {
    Read(*mut u8),
    Write(*const u8),
}

/// Represents a Write or Read request for the DiskManager to execute.
pub struct DiskRequest {
    /// Pointer to the start of the memory location where a page is either:
    ///   1. being read into from disk (on a read).
    ///   2. being written out to disk (on a write).
    ///
    /// # Safety
    /// The pointer must be valid for `BUSTUB_PAGE_SIZE` bytes for the entire
    /// lifetime of this request.
    data: DataWithDirection,

    /// ID of the page being read from / written to disk.
    page_id: PageId,

    /// Callback used to signal to the request issuer when the request has been
    /// completed. This is a oneshot sender: the worker thread sends `true`
    /// through it once the I/O is complete, and the caller blocks on the
    /// corresponding receiver.
    callback: mpsc::SyncSender<bool>,
}

// SAFETY: `DiskRequest` is moved across threads but the raw pointer `data` is
// only accessed by the worker thread while the caller waits on the receiver.
unsafe impl Send for DiskRequest {}

// ---------------------------------------------------------------------------
// DiskScheduler
// ---------------------------------------------------------------------------

/// The DiskScheduler schedules disk read and write operations.
///
/// A request is scheduled by calling `DiskScheduler::schedule()` with an
/// appropriate `DiskRequest` object. The scheduler maintains a background
/// worker thread that processes the scheduled requests using the disk manager.
/// The background thread is created in the `DiskScheduler::new()` constructor
/// and joined when the `DiskScheduler` is dropped.
pub struct DiskScheduler {
    /// Pointer to the disk manager (shared ownership between the caller and
    /// the background worker thread).
    disk_manager: Arc<dyn DiskManager>,

    /// Sending end of the MPSC channel. `schedule()` pushes requests here;
    /// the worker thread receives them on the other end. Wrapped in a `Mutex`
    /// so that `DiskScheduler` is `Sync`.
    ///
    /// When the `DiskScheduler` is dropped, a `None` sentinel is pushed into
    /// the queue to signal the background thread to stop.
    request_queue: mpsc::Sender<Option<DiskRequest>>,

    /// The background thread responsible for issuing scheduled requests to the
    /// disk manager.
    background_thread: Option<JoinHandle<()>>,
}

impl DiskScheduler {
    /// Creates a new `DiskScheduler` and spawns the background worker thread.
    pub fn new(disk_manager: Arc<dyn DiskManager>) -> Self {
        let (tx, rx) = mpsc::channel::<Option<DiskRequest>>();
        let dm_clone = Arc::clone(&disk_manager);

        let handle = thread::spawn(move || {
            Self::start_worker_thread(dm_clone, rx);
        });

        DiskScheduler {
            disk_manager,
            request_queue: tx,
            background_thread: Some(handle),
        }
    }

    /// Schedules a request for the DiskManager to execute.
    pub fn schedule_read(&self, page_id: PageId, data: *mut u8) -> mpsc::Receiver<bool> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let req = DiskRequest {
            data: DataWithDirection::Read(data),
            page_id,
            callback: sender,
        };
        self.request_queue
            .send(Some(req))
            .expect("DiskScheduler worker thread has terminated unexpectedly");
        receiver
    }

    pub fn schedule_write(&self, page_id: PageId, data: *const u8) -> mpsc::Receiver<bool> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let req = DiskRequest {
            data: DataWithDirection::Write(data),
            page_id,
            callback: sender,
        };
        self.request_queue
            .send(Some(req))
            .expect("DiskScheduler worker thread has terminated unexpectedly");
        receiver
    }

    /// Background worker thread function that processes scheduled requests.
    ///
    /// The background thread processes requests while the `DiskScheduler`
    /// exists. When `None` is received from the queue, the loop exits.
    fn start_worker_thread(dm: Arc<dyn DiskManager>, rx: mpsc::Receiver<Option<DiskRequest>>) {
        loop {
            match rx.recv() {
                Ok(None) => {
                    // Sentinel value – shut down the worker thread.
                    break;
                }
                Ok(Some(r)) => {
                    // SAFETY: The caller guarantees that `r.data` points to a
                    // valid buffer of at least `BUSTUB_PAGE_SIZE` bytes.
                    match r.data {
                        DataWithDirection::Read(data) => {
                            let data_slice =
                                unsafe { std::slice::from_raw_parts_mut(data, BUSTUB_PAGE_SIZE) };
                            dm.read_page(r.page_id, data_slice);
                        },
                        DataWithDirection::Write(data) => {
                            let data_slice =
                                unsafe { std::slice::from_raw_parts(data, BUSTUB_PAGE_SIZE) };
                            dm.write_page(r.page_id, data_slice);
                        }
                    }

                    // Signal completion to the caller.
                    r.callback.send(true).ok();
                }
                Err(_) => {
                    // Channel closed – shut down the worker thread.
                    break;
                }
            }
        }
    }

    /// Increases the size of the database file to fit the specified number of
    /// pages.
    ///
    /// This works like a dynamic array, where the capacity is doubled until
    /// all pages can fit.
    pub fn increase_disk_space(&self, pages: usize) {
        self.disk_manager.increase_disk_space(pages);
    }

    /// Deallocates a page on disk.
    ///
    /// Note: You should look at the documentation for `delete_page` in
    /// `BufferPoolManager` before using this method. Also note: This is a
    /// no-op without a more complex data structure to track deallocated pages.
    pub fn deallocate_page(&self, page_id: PageId) {
        self.disk_manager.delete_page(page_id);
    }
}

impl Drop for DiskScheduler {
    fn drop(&mut self) {
        // Put a `None` in the queue to signal to exit the loop.
        self.request_queue
            .send(None)
            .expect("DiskScheduler worker thread has terminated unexpectedly");

        if let Some(handle) = self.background_thread.take() {
            handle.join().expect("Failed to join background worker thread");
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod disk_scheduler {
    use super::*;
    use crate::storage::disk::disk_manager_memory::DiskManagerMemory;

    #[test]
    fn schedule_write_read_page_test() {
        let mut buf = vec![0u8; BUSTUB_PAGE_SIZE];
        let mut data = vec![0u8; BUSTUB_PAGE_SIZE];

        let dm = Arc::new(DiskManagerMemory::new());
        let disk_scheduler = DiskScheduler::new(dm);

        let test_str = "A test string.";
        let test_bytes = test_str.as_bytes();
        let len = test_bytes.len().min(BUSTUB_PAGE_SIZE);
        data[..len].copy_from_slice(&test_bytes[..len]);

        let recv1 = disk_scheduler.schedule_write(0, data.as_ptr());
        let recv2 = disk_scheduler.schedule_read(0, buf.as_mut_ptr());

        assert!(recv1.recv().unwrap());
        assert!(recv2.recv().unwrap());
        assert_eq!(&buf[..len], &data[..len]);

        // Drop the scheduler to join the background thread.
        drop(disk_scheduler);
    }
}

