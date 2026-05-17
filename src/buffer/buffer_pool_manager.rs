// Date:   Sun May 17 20:33:00 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
// Date:   Sun May 17 16:12:00 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// buffer_pool_manager.rs
//
// Identification: src/buffer/buffer_pool_manager.rs
//
// Copyright (c) 2015-2024, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicI32;
use std::sync::{Arc, Mutex};

use crate::buffer::frame_header::FrameHeader;
use crate::buffer::lru_k_replacer::LRUKReplacer;
use crate::buffer::lru_k_replacer::AccessType;
use crate::common::{FrameId, PageId};
use crate::storage::disk::disk_scheduler::{DiskManager, DiskScheduler};
use crate::storage::page::page_guard::{ReadPageGuard, WritePageGuard};

/// The `BufferPoolManager` is responsible for moving physical pages of data
/// back and forth from buffers in main memory to persistent storage. It also
/// behaves as a cache, keeping frequently used pages in memory for faster
/// access, and evicting unused or cold pages back out to storage.
#[allow(dead_code)]
pub struct BufferPoolManager<D: DiskManager + 'static> {
    /// The number of frames in the buffer pool.
    num_frames: usize,

    /// The next page ID to be allocated.
    next_page_id: AtomicI32,

    /// The latch protecting the buffer pool's inner data structures.
    bpm_latch: Arc<Mutex<()>>,

    /// The frame headers of the frames that this buffer pool manages.
    frames: Vec<Arc<FrameHeader>>,

    /// The page table that keeps track of the mapping between pages and
    /// buffer pool frames.
    page_table: HashMap<PageId, FrameId>,

    /// A list of free frames that do not hold any page's data.
    free_frames: VecDeque<FrameId>,

    /// The replacer to find unpinned / candidate pages for eviction.
    replacer: Arc<Mutex<LRUKReplacer>>,

    /// The disk scheduler.
    disk_scheduler: DiskScheduler<D>,
}

impl<D: DiskManager + 'static> BufferPoolManager<D> {
    /// Creates a new `BufferPoolManager`.
    ///
    /// * `num_frames` - the size of the buffer pool.
    /// * `disk_manager` - the disk manager.
    /// * `k_dist` - the backward k-distance for the LRU-K replacer.
    pub fn new(num_frames: usize, disk_manager: D, k_dist: usize) -> Self {
        let bpm_latch = Arc::new(Mutex::new(()));
        let replacer = Arc::new(Mutex::new(LRUKReplacer::new(num_frames, k_dist)));
        let disk_scheduler = DiskScheduler::new(disk_manager);

        let mut frames = Vec::with_capacity(num_frames);
        let mut free_frames = VecDeque::with_capacity(num_frames);

        {
            let _latch = bpm_latch.lock().unwrap();

            // Allocate all in-memory frames up front and initialize the free
            // frame list with all possible frame IDs.
            for i in 0..num_frames {
                frames.push(Arc::new(FrameHeader::new(i as FrameId)));
                free_frames.push_back(i as FrameId);
            }
        }

        BufferPoolManager {
            num_frames,
            next_page_id: AtomicI32::new(0),
            bpm_latch,
            frames,
            page_table: HashMap::with_capacity(num_frames),
            free_frames,
            replacer,
            disk_scheduler,
        }
    }

    /// Returns the number of frames that this buffer pool manages.
    pub fn size(&self) -> usize {
        self.num_frames
    }

    /// Returns a reference to the inner `DiskScheduler`.
    pub fn disk_scheduler(&self) -> &DiskScheduler<D> {
        &self.disk_scheduler
    }

    /// Allocates a new page on disk.
    ///
    /// TODO(P1): Add implementation.
    pub fn new_page(&self) -> PageId {
        todo!("TODO(P1): Add implementation.")
    }

    /// Removes a page from the database, both on disk and in memory.
    ///
    /// If the page is pinned in the buffer pool, this function does nothing
    /// and returns `false`. Otherwise, removes the page from both disk and
    /// memory (if it is still in the buffer pool), returning `true`.
    ///
    /// TODO(P1): Add implementation.
    pub fn delete_page(&self, _page_id: PageId) -> bool {
        todo!("TODO(P1): Add implementation.")
    }

    /// Acquires an optional write-locked guard over a page of data.
    ///
    /// If it is not possible to bring the page into memory, returns `None`.
    ///
    /// TODO(P1): Add implementation.
    pub fn checked_write_page(
        &self,
        page_id: PageId,
        _access_type: AccessType,
    ) -> Option<WritePageGuard> {
        let _ = page_id;
        todo!("TODO(P1): Add implementation.")
    }

    /// Acquires an optional read-locked guard over a page of data.
    ///
    /// If it is not possible to bring the page into memory, returns `None`.
    ///
    /// TODO(P1): Add implementation.
    pub fn checked_read_page(
        &self,
        page_id: PageId,
        _access_type: AccessType,
    ) -> Option<ReadPageGuard> {
        let _ = page_id;
        todo!("TODO(P1): Add implementation.")
    }

    /// A wrapper around `checked_write_page` that aborts if the page could
    /// not be brought into memory.
    ///
    /// This should only be used for testing and ergonomic's sake.
    pub fn write_page(
        &self,
        page_id: PageId,
        access_type: AccessType,
    ) -> WritePageGuard {
        self.checked_write_page(page_id, access_type)
            .expect("CheckedWritePage failed to bring in page")
    }

    /// A wrapper around `checked_read_page` that aborts if the page could not
    /// be brought into memory.
    ///
    /// This should only be used for testing and ergonomic's sake.
    pub fn read_page(
        &self,
        page_id: PageId,
        access_type: AccessType,
    ) -> ReadPageGuard {
        self.checked_read_page(page_id, access_type)
            .expect("CheckedReadPage failed to bring in page")
    }

    /// Flushes a page's data out to disk.
    ///
    /// Returns `false` if the page could not be found in the page table.
    ///
    /// TODO(P1): Add implementation.
    pub fn flush_page(&self, page_id: PageId) -> bool {
        let _ = page_id;
        todo!("TODO(P1): Add implementation.")
    }

    /// Flushes all page data that is in memory to disk.
    ///
    /// TODO(P1): Add implementation.
    pub fn flush_all_pages(&self) {
        todo!("TODO(P1): Add implementation.")
    }

    /// Retrieves the pin count of a page.
    ///
    /// Returns `None` if the page does not exist in memory.
    ///
    /// TODO(P1): Add implementation.
    pub fn get_pin_count(&self, page_id: PageId) -> Option<usize> {
        let _ = page_id;
        todo!("TODO(P1): Add implementation.")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod buffer_pool_manager {
    use super::*;
    use std::sync::Mutex as StdMutex;
    use std::thread;
    use std::time::Duration;
    use std::sync::atomic::Ordering;
    use crate::common::BUSTUB_PAGE_SIZE;

    /// A simple in-memory DiskManager for testing.
    struct DiskManagerMemory {
        pages: StdMutex<std::collections::HashMap<PageId, Vec<u8>>>,
    }

    impl DiskManagerMemory {
        fn new() -> Self {
            Self {
                pages: StdMutex::new(std::collections::HashMap::new()),
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

        fn increase_disk_space(&self, _pages: usize) {}

        fn delete_page(&self, page_id: PageId) {
            let mut pages = self.pages.lock().unwrap();
            pages.remove(&page_id);
        }
    }

    const FRAMES: usize = 10;
    const K_DIST: usize = 5;

    /// Very basic test.
    #[test]
    #[ignore = "TODO(P1): BufferPoolManager not yet implemented"]
    fn very_basic_test() {
        let dm = DiskManagerMemory::new();
        let bpm = BufferPoolManager::new(FRAMES, dm, K_DIST);

        let pid = bpm.new_page();

        let test_bytes = b"Hello, world!";
        let len = test_bytes.len();

        // Check `WritePageGuard` basic functionality.
        {
            let mut guard = bpm.write_page(pid, AccessType::Unknown);
            let data = guard.get_data_mut();
            data[..len].copy_from_slice(test_bytes);
            assert_eq!(&data[..len], test_bytes);
        }

        // Check `ReadPageGuard` basic functionality.
        {
            let guard = bpm.read_page(pid, AccessType::Unknown);
            let data = guard.get_data();
            assert_eq!(&data[..len], test_bytes);
        }

        // Check `ReadPageGuard` basic functionality (again).
        {
            let guard = bpm.read_page(pid, AccessType::Unknown);
            let data = guard.get_data();
            assert_eq!(&data[..len], test_bytes);
        }

        assert!(bpm.delete_page(pid));
    }

    /// Page pin easy test.
    #[test]
    #[ignore = "TODO(P1): BufferPoolManager not yet implemented"]
    fn page_pin_easy_test() {
        let dm = DiskManagerMemory::new();
        let bpm = BufferPoolManager::new(2, dm, 5);

        let page_id0;
        let page_id1;

        // Scope: write to both pages, fill the buffer pool.
        {
            page_id0 = bpm.new_page();
            let mut page0_write = bpm
                .checked_write_page(page_id0, AccessType::Unknown)
                .expect("should get page0");
            {
                let data = page0_write.get_data_mut();
                let msg = b"page0";
                data[..msg.len()].copy_from_slice(msg);
            }

            page_id1 = bpm.new_page();
            let mut page1_write = bpm
                .checked_write_page(page_id1, AccessType::Unknown)
                .expect("should get page1");
            {
                let data = page1_write.get_data_mut();
                let msg = b"page1";
                data[..msg.len()].copy_from_slice(msg);
            }

            assert_eq!(Some(1), bpm.get_pin_count(page_id0));
            assert_eq!(Some(1), bpm.get_pin_count(page_id1));

            // Buffer pool is full, should not be able to create new pages.
            let temp_id1 = bpm.new_page();
            assert!(bpm.checked_read_page(temp_id1, AccessType::Unknown).is_none());

            let temp_id2 = bpm.new_page();
            assert!(bpm
                .checked_write_page(temp_id2, AccessType::Unknown)
                .is_none());

            assert_eq!(Some(1), bpm.get_pin_count(page_id0));
            page0_write.drop_guard();
            assert_eq!(Some(0), bpm.get_pin_count(page_id0));

            assert_eq!(Some(1), bpm.get_pin_count(page_id1));
            page1_write.drop_guard();
            assert_eq!(Some(0), bpm.get_pin_count(page_id1));
        }

        // Now pages are evicted. We should be able to bring in new pages.
        {
            let temp_id1 = bpm.new_page();
            assert!(bpm
                .checked_read_page(temp_id1, AccessType::Unknown)
                .is_some());

            let temp_id2 = bpm.new_page();
            assert!(bpm
                .checked_write_page(temp_id2, AccessType::Unknown)
                .is_some());

            assert!(bpm.get_pin_count(page_id0).is_none());
            assert!(bpm.get_pin_count(page_id1).is_none());
        }

        // Bring back the original pages and verify data.
        {
            let mut page0_write = bpm
                .checked_write_page(page_id0, AccessType::Unknown)
                .expect("page0 should be available");
            assert_eq!(&page0_write.get_data()[..5], b"page0");
            {
                let data = page0_write.get_data_mut();
                let msg = b"page0updated";
                data[..msg.len()].copy_from_slice(msg);
            }

            let mut page1_write = bpm
                .checked_write_page(page_id1, AccessType::Unknown)
                .expect("page1 should be available");
            assert_eq!(&page1_write.get_data()[..5], b"page1");
            {
                let data = page1_write.get_data_mut();
                let msg = b"page1updated";
                data[..msg.len()].copy_from_slice(msg);
            }

            assert_eq!(Some(1), bpm.get_pin_count(page_id0));
            assert_eq!(Some(1), bpm.get_pin_count(page_id1));
        }

        assert_eq!(Some(0), bpm.get_pin_count(page_id0));
        assert_eq!(Some(0), bpm.get_pin_count(page_id1));

        // Read back and verify updated data.
        {
            let page0_read = bpm
                .checked_read_page(page_id0, AccessType::Unknown)
                .expect("page0 should be readable");
            assert_eq!(&page0_read.get_data()[..13], b"page0updated");

            let page1_read = bpm
                .checked_read_page(page_id1, AccessType::Unknown)
                .expect("page1 should be readable");
            assert_eq!(&page1_read.get_data()[..13], b"page1updated");

            assert_eq!(Some(1), bpm.get_pin_count(page_id0));
            assert_eq!(Some(1), bpm.get_pin_count(page_id1));
        }

        assert_eq!(Some(0), bpm.get_pin_count(page_id0));
        assert_eq!(Some(0), bpm.get_pin_count(page_id1));
    }

    /// Page pin medium test.
    #[test]
    #[ignore = "TODO(P1): BufferPoolManager not yet implemented"]
    fn page_pin_medium_test() {
        let dm = DiskManagerMemory::new();
        let bpm = BufferPoolManager::new(FRAMES, dm, K_DIST);

        // Scenario: The buffer pool is empty. Create a new page.
        let pid0 = bpm.new_page();
        {
            let mut page0 = bpm.write_page(pid0, AccessType::Unknown);
            let msg = b"Hello";
            page0.get_data_mut()[..msg.len()].copy_from_slice(msg);
            assert_eq!(&page0.get_data()[..msg.len()], msg);
        }

        let mut pages: Vec<WritePageGuard> = Vec::new();

        // Scenario: Create new pages until the buffer pool is full.
        for _ in 0..FRAMES {
            let pid = bpm.new_page();
            let page = bpm.write_page(pid, AccessType::Unknown);
            pages.push(page);
        }

        // All pin counts should be 1.
        for page in &pages {
            let pid = page.get_page_id();
            assert_eq!(Some(1), bpm.get_pin_count(pid));
        }

        // Once full, cannot create new pages.
        for _ in 0..FRAMES {
            let pid = bpm.new_page();
            assert!(bpm
                .checked_write_page(pid, AccessType::Unknown)
                .is_none());
        }

        // Drop the first 5 pages to unpin them.
        for _ in 0..FRAMES / 2 {
            let pid = pages[0].get_page_id();
            assert_eq!(Some(1), bpm.get_pin_count(pid));
            pages.remove(0);
            assert_eq!(Some(0), bpm.get_pin_count(pid));
        }

        // Remaining pages still have pin count 1.
        for page in &pages {
            assert_eq!(Some(1), bpm.get_pin_count(page.get_page_id()));
        }

        // Create 4 new pages, which should evict the oldest ones.
        for _ in 0..(FRAMES / 2) - 1 {
            let pid = bpm.new_page();
            let page = bpm.write_page(pid, AccessType::Unknown);
            pages.push(page);
        }

        // Fetch page 0 and verify data.
        {
            let original_page = bpm.read_page(pid0, AccessType::Unknown);
            assert_eq!(&original_page.get_data()[..5], b"Hello");
        }

        // Pin the last page, then try to fetch page 0 again (should fail).
        let last_pid = bpm.new_page();
        let _last_page = bpm.read_page(last_pid, AccessType::Unknown);

        assert!(bpm.checked_read_page(pid0, AccessType::Unknown).is_none());
    }

    /// Page access test: concurrent read/write access.
    #[test]
    #[ignore = "TODO(P1): BufferPoolManager not yet implemented"]
    fn page_access_test() {
        let dm = DiskManagerMemory::new();
        let bpm = Arc::new(BufferPoolManager::new(1, dm, K_DIST));
        let rounds = 50;

        let pid = bpm.new_page();

        let bpm_writer = Arc::clone(&bpm);
        let writer = thread::spawn(move || {
            for i in 0..rounds {
                thread::sleep(Duration::from_millis(5));
                let mut guard = bpm_writer.write_page(pid, AccessType::Unknown);
                let s = i.to_string();
                let bytes = s.as_bytes();
                let data = guard.get_data_mut();
                data[..bytes.len()].copy_from_slice(bytes);
            }
        });

        let bpm_reader = Arc::clone(&bpm);
        for _ in 0..rounds {
            thread::sleep(Duration::from_millis(10));

            let guard = bpm_reader.read_page(pid, AccessType::Unknown);
            let mut buf = vec![0u8; BUSTUB_PAGE_SIZE];
            buf[..BUSTUB_PAGE_SIZE].copy_from_slice(guard.get_data());

            thread::sleep(Duration::from_millis(10));

            // Data should be unmodified while holding the read guard.
            assert_eq!(guard.get_data(), buf.as_slice());
        }

        writer.join().unwrap();
    }

    /// Contention test: multiple writers.
    #[test]
    #[ignore = "TODO(P1): BufferPoolManager not yet implemented"]
    fn contention_test() {
        let dm = DiskManagerMemory::new();
        let bpm = Arc::new(BufferPoolManager::new(FRAMES, dm, K_DIST));
        let rounds = 100_000;

        let pid = bpm.new_page();

        let mut handles = vec![];
        for _ in 0..4 {
            let bpm_clone = Arc::clone(&bpm);
            let handle = thread::spawn(move || {
                for i in 0..rounds {
                    let mut guard = bpm_clone.write_page(pid, AccessType::Unknown);
                    let s = i.to_string();
                    let bytes = s.as_bytes();
                    let data = guard.get_data_mut();
                    data[..bytes.len()].copy_from_slice(bytes);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    /// Deadlock test: verify latch ordering prevents deadlocks.
    #[test]
    #[ignore = "TODO(P1): BufferPoolManager not yet implemented"]
    fn deadlock_test() {
        let dm = DiskManagerMemory::new();
        let bpm = Arc::new(BufferPoolManager::new(FRAMES, dm, K_DIST));

        let pid0 = bpm.new_page();
        let pid1 = bpm.new_page();

        let mut guard0 = bpm.write_page(pid0, AccessType::Unknown);

        let start = Arc::new(AtomicI32::new(0));
        let start_clone = Arc::clone(&start);
        let bpm_child = Arc::clone(&bpm);

        let child = thread::spawn(move || {
            start_clone.store(1, Ordering::Release);
            let _guard0 = bpm_child.write_page(pid0, AccessType::Unknown);
        });

        while start.load(Ordering::Acquire) == 0 {}

        thread::sleep(Duration::from_millis(1000));

        let guard1 = bpm.write_page(pid1, AccessType::Unknown);

        guard0.drop_guard();
        drop(guard1);

        child.join().unwrap();
    }

    /// Evictable test: ensure evictable status is always correct.
    #[test]
    #[ignore = "TODO(P1): BufferPoolManager not yet implemented"]
    fn evictable_test() {
        let dm = DiskManagerMemory::new();
        let bpm = Arc::new(BufferPoolManager::new(1, dm, K_DIST));
        let rounds = 1000;
        let num_readers = 8;

        for i in 0..rounds {
            let winner_pid = bpm.new_page();
            let loser_pid = bpm.new_page();

            let signal = Arc::new(Mutex::new(false));
            let cv = Arc::new(std::sync::Condvar::new());

            let mut readers = vec![];
            for _ in 0..num_readers {
                let bpm_clone = Arc::clone(&bpm);
                let signal_clone = Arc::clone(&signal);
                let cv_clone = Arc::clone(&cv);

                let handle = thread::spawn(move || {
                    let mut signaled = signal_clone.lock().unwrap();
                    while !*signaled {
                        signaled = cv_clone.wait(signaled).unwrap();
                    }
                    drop(signaled);

                    let _read_guard = bpm_clone.read_page(winner_pid, AccessType::Unknown);
                    assert!(bpm_clone
                        .checked_read_page(loser_pid, AccessType::Unknown)
                        .is_none());
                });
                readers.push(handle);
            }

            {
                let mut signal_lock = signal.lock().unwrap();
                if i % 2 == 0 {
                    let _read_guard = bpm.read_page(winner_pid, AccessType::Unknown);
                    *signal_lock = true;
                    cv.notify_all();
                    drop(signal_lock);
                } else {
                    let _write_guard = bpm.write_page(winner_pid, AccessType::Unknown);
                    *signal_lock = true;
                    cv.notify_all();
                    drop(signal_lock);
                }
            }

            for reader in readers {
                reader.join().unwrap();
            }
        }
    }
}


