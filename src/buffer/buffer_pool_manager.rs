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
///
/// The buffer pool manager owns the set of frames it manages, the page table
/// that maps page IDs to frame IDs, a free frame list, and the LRU-K replacer
/// for eviction decisions. All internal state is protected by an `Arc<Mutex<>>`.
#[allow(dead_code)]
struct BufferPoolManagerCore {
    /// The number of frames in the buffer pool.
    num_frames: usize,

    /// The next page ID to be allocated. Initialized to 0 and incremented
    /// atomically on each `new_page()` call.
    next_page_id: AtomicI32,

    /// The frame headers of the frames that this buffer pool manages.
    /// Each frame is wrapped in `Arc<Mutex<FrameHeader>>` so it can be
    /// shared with page guards for safe concurrent access.
    frames: Vec<Arc<Mutex<FrameHeader>>>,

    /// The page table that keeps track of the mapping between pages and
    /// buffer pool frames.
    page_table: HashMap<PageId, FrameId>,

    /// A list of free frames that do not hold any page's data.
    free_frames: VecDeque<FrameId>,

    /// The replacer to find unpinned / candidate pages for eviction.
    replacer: LRUKReplacer,

    /// The disk scheduler for reading and writing pages to persistent storage.
    disk_scheduler: DiskScheduler,
}

/// A thread-safe, shareable handle to the buffer pool manager.
///
/// Wraps `BufferPoolManagerCore` in `Mutex<...>` so that it can be
/// shared between threads. The mutex guards all internal state
/// (page table, free list, replacer, etc.).
pub struct BufferPoolManager {
    core: Mutex<BufferPoolManagerCore>
}

impl BufferPoolManager {
    /// Creates a new `BufferPoolManager`.
    ///
    /// Allocates `num_frames` in-memory frames up front and initializes the
    /// free frame list with all possible frame IDs. The replacer is created
    /// with the given `k_dist` value.
    ///
    /// * `num_frames` - the size of the buffer pool (number of frames).
    /// * `disk_manager` - the disk manager for persistent storage I/O.
    /// * `k_dist` - the backward k-distance for the LRU-K replacer.
    pub fn new(num_frames: usize, disk_manager: Arc<dyn DiskManager>, k_dist: usize) -> Self {
        let replacer = LRUKReplacer::new(num_frames, k_dist);
        let disk_scheduler = DiskScheduler::new(disk_manager);

        let mut frames = Vec::with_capacity(num_frames);
        let mut free_frames = VecDeque::with_capacity(num_frames);

        // Allocate all in-memory frames up front and initialize the free
        // frame list with all possible frame IDs.
        for i in 0..num_frames {
            frames.push(Arc::new(Mutex::new(FrameHeader::new(i as FrameId))));
            free_frames.push_back(i as FrameId);
        }

        BufferPoolManager {
            core: Mutex::new(BufferPoolManagerCore {
                num_frames,
                next_page_id: AtomicI32::new(0),
                frames,
                page_table: HashMap::with_capacity(num_frames),
                free_frames,
                replacer,
                disk_scheduler,
            })
        }
    }

    /// Returns the number of frames that this buffer pool manages.
    pub fn size(&self) -> usize {
        let guard = self.core.lock()
            .expect("Unexpected error of mutex locking");
        guard.num_frames
    }

    /// Allocates a new page on disk.
    ///
    /// Returns the page ID of the newly allocated page.
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
    /// * `page_id` - the ID of the page to delete.
    ///
    /// TODO(P1): Add implementation.
    pub fn delete_page(&self, _page_id: PageId) -> bool {
        todo!("TODO(P1): Add implementation.")
    }

    /// Acquires an optional write-locked guard over a page of data.
    ///
    /// If the page is not in the buffer pool, it is fetched from disk. If
    /// there is no available frame and no evictable pages, returns `None`.
    ///
    /// * `page_id` - the ID of the page to access.
    /// * `_access_type` - the type of access (used for replacer tracking).
    ///
    /// TODO(P1): Add implementation.
    pub fn checked_write_page(
        &self,
        page_id: PageId,
        _access_type: AccessType,
    ) -> Option<WritePageGuard<'_>> {
        let _ = page_id;
        todo!("TODO(P1): Add implementation.")
    }

    /// Acquires an optional read-locked guard over a page of data.
    ///
    /// If the page is not in the buffer pool, it is fetched from disk. If
    /// there is no available frame and no evictable pages, returns `None`.
    ///
    /// * `page_id` - the ID of the page to access.
    /// * `_access_type` - the type of access (used for replacer tracking).
    ///
    /// TODO(P1): Add implementation.
    pub fn checked_read_page(
        &self,
        page_id: PageId,
        _access_type: AccessType,
    ) -> Option<ReadPageGuard<'_>> {
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
    ) -> WritePageGuard<'_> {
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
    ) -> ReadPageGuard<'_> {
        self.checked_read_page(page_id, access_type)
            .expect("CheckedReadPage failed to bring in page")
    }

    /// Flushes a page's data out to disk.
    ///
    /// Returns `false` if the page could not be found in the page table.
    ///
    /// * `page_id` - the ID of the page to flush.
    ///
    /// TODO(P1): Add implementation.
    pub fn flush_page(&self, page_id: PageId) -> bool {
        let _ = page_id;
        todo!("TODO(P1): Add implementation.")
    }

    /// Flushes all page data that is in memory to disk.
    ///
    /// Iterates over every frame in the buffer pool and flushes dirty pages
    /// to disk via the disk scheduler.
    ///
    /// TODO(P1): Add implementation.
    pub fn flush_all_pages(&self) {
        todo!("TODO(P1): Add implementation.")
    }

    /// Retrieves the pin count of a page.
    ///
    /// Returns `None` if the page does not exist in memory.
    ///
    /// * `page_id` - the ID of the page to query.
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
mod buffer_pool_manager_tests {
    use super::*;
    use std::sync::Barrier;
    use std::thread;
    use std::time::Duration;
    use std::collections::HashMap;
    use std::sync::Mutex as StdMutex;
    use crate::common::BUSTUB_PAGE_SIZE;

    /// A simple in-memory DiskManager for testing.
    struct DiskManagerMemory {
        pages: StdMutex<HashMap<PageId, Vec<u8>>>,
    }

    impl DiskManagerMemory {
        fn new() -> Self {
            Self {
                pages: StdMutex::new(HashMap::new()),
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

    #[test]
    fn very_basic_test() {
        let dm = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(FRAMES, dm, K_DIST);

        let pid = bpm.new_page();

        let test_bytes = b"Hello, world!";
        let len = test_bytes.len();

        {
            let mut guard = bpm.write_page(pid, AccessType::Unknown);
            let data = guard.as_mut_slice();
            data[..len].copy_from_slice(test_bytes);
            assert_eq!(&data[..len], test_bytes);
        }

        {
            let guard = bpm.read_page(pid, AccessType::Unknown);
            let data = guard.as_slice();
            assert_eq!(&data[..len], test_bytes);
        }

        {
            let guard = bpm.read_page(pid, AccessType::Unknown);
            let data = guard.as_slice();
            assert_eq!(&data[..len], test_bytes);
        }

        assert!(bpm.delete_page(pid));
    }

    #[test]
    fn page_pin_easy_test() {
        let dm = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(2, dm, 5);

        let page_id0;
        let page_id1;

        {
            page_id0 = bpm.new_page();
            let mut page0_write = bpm
                .checked_write_page(page_id0, AccessType::Unknown)
                .expect("should get page0");
            {
                let data = page0_write.as_mut_slice();
                let msg = b"page0";
                data[..msg.len()].copy_from_slice(msg);
            }

            page_id1 = bpm.new_page();
            let mut page1_write = bpm
                .checked_write_page(page_id1, AccessType::Unknown)
                .expect("should get page1");
            {
                let data = page1_write.as_mut_slice();
                let msg = b"page1";
                data[..msg.len()].copy_from_slice(msg);
            }

            assert_eq!(Some(1), bpm.get_pin_count(page_id0));
            assert_eq!(Some(1), bpm.get_pin_count(page_id1));

            let temp_id1 = bpm.new_page();
            assert!(bpm.checked_read_page(temp_id1, AccessType::Unknown).is_none());

            let temp_id2 = bpm.new_page();
            assert!(bpm
                .checked_write_page(temp_id2, AccessType::Unknown)
                .is_none());

            assert_eq!(Some(1), bpm.get_pin_count(page_id0));
            drop(page0_write);
            assert_eq!(Some(0), bpm.get_pin_count(page_id0));

            assert_eq!(Some(1), bpm.get_pin_count(page_id1));
            drop(page1_write);
            assert_eq!(Some(0), bpm.get_pin_count(page_id1));
        }

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

        {
            let mut page0_write = bpm
                .checked_write_page(page_id0, AccessType::Unknown)
                .expect("page0 should be available");
            assert_eq!(&page0_write.as_slice()[..5], b"page0");
            {
                let data = page0_write.as_mut_slice();
                let msg = b"page0updated";
                data[..msg.len()].copy_from_slice(msg);
            }

            let mut page1_write = bpm
                .checked_write_page(page_id1, AccessType::Unknown)
                .expect("page1 should be available");
            assert_eq!(&page1_write.as_slice()[..5], b"page1");
            {
                let data = page1_write.as_mut_slice();
                let msg = b"page1updated";
                data[..msg.len()].copy_from_slice(msg);
            }

            assert_eq!(Some(1), bpm.get_pin_count(page_id0));
            assert_eq!(Some(1), bpm.get_pin_count(page_id1));
        }

        assert_eq!(Some(0), bpm.get_pin_count(page_id0));
        assert_eq!(Some(0), bpm.get_pin_count(page_id1));

        {
            let page0_read = bpm
                .checked_read_page(page_id0, AccessType::Unknown)
                .expect("page0 should be readable");
            assert_eq!(&page0_read.as_slice()[..13], b"page0updated");

            let page1_read = bpm
                .checked_read_page(page_id1, AccessType::Unknown)
                .expect("page1 should be readable");
            assert_eq!(&page1_read.as_slice()[..13], b"page1updated");

            assert_eq!(Some(1), bpm.get_pin_count(page_id0));
            assert_eq!(Some(1), bpm.get_pin_count(page_id1));
        }

        assert_eq!(Some(0), bpm.get_pin_count(page_id0));
        assert_eq!(Some(0), bpm.get_pin_count(page_id1));
    }

    #[test]
    fn page_pin_medium_test() {
        let dm = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(FRAMES, dm, K_DIST);

        let pid0 = bpm.new_page();
        {
            let mut page0 = bpm.write_page(pid0, AccessType::Unknown);
            let msg = b"Hello";
            page0.as_mut_slice()[..msg.len()].copy_from_slice(msg);
            assert_eq!(&page0.as_slice()[..msg.len()], msg);
        }

        let mut pages: Vec<WritePageGuard> = Vec::new();

        for _ in 0..FRAMES {
            let pid = bpm.new_page();
            let page = bpm.write_page(pid, AccessType::Unknown);
            pages.push(page);
        }

        for page in &pages {
            let pid = page.get_page_id();
            assert_eq!(Some(1), bpm.get_pin_count(pid));
        }

        for _ in 0..FRAMES {
            let pid = bpm.new_page();
            assert!(bpm
                .checked_write_page(pid, AccessType::Unknown)
                .is_none());
        }

        for _ in 0..FRAMES / 2 {
            let pid = pages[0].get_page_id();
            assert_eq!(Some(1), bpm.get_pin_count(pid));
            pages.remove(0);
            assert_eq!(Some(0), bpm.get_pin_count(pid));
        }

        for page in &pages {
            assert_eq!(Some(1), bpm.get_pin_count(page.get_page_id()));
        }

        for _ in 0..(FRAMES / 2) - 1 {
            let pid = bpm.new_page();
            let page = bpm.write_page(pid, AccessType::Unknown);
            pages.push(page);
        }

        {
            let original_page = bpm.read_page(pid0, AccessType::Unknown);
            assert_eq!(&original_page.as_slice()[..5], b"Hello");
        }

        let last_pid = bpm.new_page();
        let _last_page = bpm.read_page(last_pid, AccessType::Unknown);

        assert!(bpm.checked_read_page(pid0, AccessType::Unknown).is_none());
    }

    #[test]
    fn page_access_test() {
        let dm = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(1, dm, K_DIST);
        let rounds = 50;

        let pid = bpm.new_page();

        thread::scope(|s| {
            s.spawn(|| {
                for i in 0..rounds {
                    thread::sleep(Duration::from_millis(5));
                    let mut guard = bpm.write_page(pid, AccessType::Unknown);
                    let str_val = i.to_string();
                    let bytes = str_val.as_bytes();
                    let data = guard.as_mut_slice();
                    data[..bytes.len()].copy_from_slice(bytes);
                }
            });

            for _ in 0..rounds {
                thread::sleep(Duration::from_millis(10));

                let guard = bpm.read_page(pid, AccessType::Unknown);
                let mut buf = vec![0u8; BUSTUB_PAGE_SIZE];
                buf[..BUSTUB_PAGE_SIZE].copy_from_slice(guard.as_slice());

                thread::sleep(Duration::from_millis(10));

                assert_eq!(guard.as_slice(), buf.as_slice());
            }
        });
    }

    #[test]
    fn contention_test() {
        let dm = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(FRAMES, dm, K_DIST);
        let rounds = 100_000;

        let pid = bpm.new_page();

        thread::scope(|s| {
            for _ in 0..4 {
                s.spawn(|| {
                    for i in 0..rounds {
                        let mut guard = bpm.write_page(pid, AccessType::Unknown);
                        let str_val = i.to_string();
                        let bytes = str_val.as_bytes();
                        let data = guard.as_mut_slice();
                        data[..bytes.len()].copy_from_slice(bytes);
                    }
                });
            }
        });
    }

    #[test]
    fn deadlock_test() {
        let dm = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(FRAMES, dm, K_DIST);

        let pid0 = bpm.new_page();
        let pid1 = bpm.new_page();

        let guard0 = bpm.write_page(pid0, AccessType::Unknown);
        let barrier = Barrier::new(2);

        thread::scope(|s| {
            s.spawn(|| {
                barrier.wait();
                let _guard0 = bpm.write_page(pid0, AccessType::Unknown);
            });

            barrier.wait();
            thread::sleep(Duration::from_millis(1000));

            let guard1 = bpm.write_page(pid1, AccessType::Unknown);

            drop(guard0);
            drop(guard1);
        });
    }

    /// Evictable test: ensure evictable status is always correct.
    #[test]
    fn evictable_test() {
        let dm = Arc::new(DiskManagerMemory::new());
        let bpm = Arc::new(BufferPoolManager::new(1, dm, K_DIST));
        let rounds = 1000;
        let num_readers = 8;

        for i in 0..rounds {
            let winner_pid = bpm.new_page();
            let loser_pid = bpm.new_page();

            let barrier = Barrier::new(num_readers + 1);

            thread::scope(|s| {
                for _ in 0..num_readers {
                    s.spawn(|| {
                        barrier.wait();

                        let _read_guard = bpm.read_page(winner_pid, AccessType::Unknown);
                        assert!(bpm
                            .checked_read_page(loser_pid, AccessType::Unknown)
                            .is_none());
                    });
                }

                if i % 2 == 0 {
                    let _read_guard = bpm.read_page(winner_pid, AccessType::Unknown);
                    barrier.wait();
                } else {
                    let _write_guard = bpm.write_page(winner_pid, AccessType::Unknown);
                    barrier.wait();
                }

            });
        }
    }
}

