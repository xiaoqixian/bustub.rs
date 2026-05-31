//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// table_heap.rs
//
// Identification: src/storage/table/table_heap.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use crate::buffer::buffer_pool_manager::BufferPoolManager;
use crate::buffer::lru_k_replacer::AccessType;
use crate::common::rid::RID;
use crate::common::{INVALID_PAGE_ID, PageId};
use crate::concurrency::LockManager;
use crate::concurrency::Transaction;
use crate::storage::disk::disk_scheduler::DiskManager;
use crate::storage::page::table_page::TablePage;
use crate::storage::table::tuple::{Tuple, TupleMeta};

// ---------------------------------------------------------------------------
// TableIterator
// ---------------------------------------------------------------------------

/// `TableIterator` enables the sequential scan of a `TableHeap`.
///
/// When created via `MakeIterator`, it records the last tuple position at
/// creation time and will stop at that point (avoiding the Halloween
/// problem). When created via `MakeEagerIterator`, there is no stop point
/// and the iterator will scan until the end of the table.
pub struct TableIterator<'a> {
    table_heap: &'a TableHeap,
    rid: RID,
    stop_at_rid: RID,
}

impl<'a> TableIterator<'a> {
    /// Creates a new `TableIterator` over the given `table_heap`, starting
    /// at `rid` and stopping at `stop_at_rid` (exclusive).
    ///
    /// If `stop_at_rid` has an invalid page ID, the iterator will scan
    /// until the end of the table.
    fn new(table_heap: &'a TableHeap, rid: RID, stop_at_rid: RID) -> Self {
        let mut this = TableIterator {
            table_heap,
            rid,
            stop_at_rid,
        };

        // If the starting RID does not correspond to a valid tuple
        // (e.g., the table has just been initialized), set rid to invalid.
        if this.rid.page_id() != INVALID_PAGE_ID {
            let page_guard = this
                .table_heap
                .bpm
                .read_page(this.rid.page_id(), AccessType::Unknown);
            let page = page_guard.as_ref::<TablePage>();
            if this.rid.slot_num() >= page.get_num_tuples() as u32 {
                this.rid = RID::new();
            }
        }

        this
    }

    /// Returns the tuple and its metadata at the current iterator position.
    pub fn get_tuple(&self) -> (TupleMeta, Tuple) {
        self.table_heap.get_tuple(self.rid)
    }

    /// Returns the RID of the current iterator position.
    pub fn get_rid(&self) -> RID {
        self.rid
    }

    /// Returns `true` if the iterator has reached the end of the scan.
    pub fn is_end(&self) -> bool {
        self.rid.page_id() == INVALID_PAGE_ID
    }

    /// Advances the iterator to the next tuple.
    pub fn next(&mut self) -> &mut Self {
        let page_guard = self
            .table_heap
            .bpm
            .read_page(self.rid.page_id(), AccessType::Unknown);
        let page = page_guard.as_ref::<TablePage>();

        let next_tuple_id = self.rid.slot_num() + 1;

        // Safety check: ensure we do not iterate past the stop-at RID.
        if self.stop_at_rid.page_id() != INVALID_PAGE_ID {
            assert!(
                self.rid.page_id() < self.stop_at_rid.page_id()
                    || (self.rid.page_id() == self.stop_at_rid.page_id()
                        && next_tuple_id <= self.stop_at_rid.slot_num()),
                "iterate out of bound"
            );
        }

        self.rid = RID::from_parts(self.rid.page_id(), next_tuple_id);

        if self.rid == self.stop_at_rid {
            self.rid = RID::new();
        } else if next_tuple_id < page.get_num_tuples() as u32 {
            // Still on the same page with a valid next tuple.
        } else {
            let next_page_id = page.get_next_page_id();
            self.rid = RID::from_parts(next_page_id, 0);
        }

        self
    }
}

// ---------------------------------------------------------------------------
// TableHeap
// ---------------------------------------------------------------------------

/// `TableHeap` represents a physical table on disk.
/// This is just a doubly-linked list of pages.
///
/// The internal `latch` is a table-level lock used for synchronizing
/// operations that modify the page chain (e.g., inserting a tuple and
/// updating `last_page_id`). It is **not** used for protecting individual
/// fields — `bpm`, `first_page_id`, and `last_page_id` are stored directly
/// on the struct and accessed under the latch only when their consistency
/// with each other matters.
pub struct TableHeap {
    /// The buffer pool manager used for reading / writing pages.
    bpm: Arc<BufferPoolManager>,
    /// The page ID of the first page in this table (set once at construction
    /// and never changed).
    first_page_id: PageId,
    /// The page ID of the last page in this table (updated during insert,
    /// under the protection of `latch`).  Exposed as an `AtomicI32` so that
    /// a shared `&self` reference can mutate it (the latch provides critical-
    /// section semantics).
    last_page_id: AtomicI32,
    /// Table-level latch that serializes operations modifying the page
    /// chain (e.g. insert which may update `last_page_id`).
    latch: Mutex<()>,
}

impl TableHeap {
    /// Creates a new `TableHeap` backed by the given buffer pool manager.
    /// A new first page is allocated and initialized.
    pub fn new(bpm: Arc<BufferPoolManager>) -> Self {
        let first_page_id = bpm.new_page();

        let mut guard = bpm.write_page(first_page_id, AccessType::Unknown);
        let first_page = guard.as_mut_ref::<TablePage>();
        first_page.init();
        drop(guard);

        TableHeap {
            bpm,
            first_page_id,
            last_page_id: AtomicI32::new(first_page_id),
            latch: Mutex::new(()),
        }
    }

    /// Private constructor used for creating an empty table heap (for binder
    /// tests). The resulting heap has a minimal buffer pool manager and no
    /// pages.
    fn create_empty() -> Self {
        /// A dummy disk manager that performs no I/O, sufficient for binder
        /// tests that never actually read or write table data.
        struct DummyDiskManager;

        impl DiskManager for DummyDiskManager {
            fn write_page(&self, _page_id: PageId, _page_data: &[u8]) {}
            fn read_page(&self, _page_id: PageId, _page_data: &mut [u8]) {}
            fn increase_disk_space(&self, _pages: usize) {}
            fn delete_page(&self, _page_id: PageId) {}
        }

        let bpm = Arc::new(BufferPoolManager::new(1, Arc::new(DummyDiskManager), 1));
        TableHeap {
            bpm,
            first_page_id: INVALID_PAGE_ID,
            last_page_id: AtomicI32::new(INVALID_PAGE_ID),
            latch: Mutex::new(()),
        }
    }

    /// Factory method for creating an empty table heap (used in binder
    /// tests).
    pub fn create_empty_heap() -> Self {
        TableHeap::create_empty()
    }

    /// Inserts a tuple into the table.
    ///
    /// If the tuple is too large (does not fit in any page), returns `None`.
    ///
    /// The table latch is acquired at the start and held for the duration of
    /// the insert, preventing concurrent inserts from racing on
    /// `last_page_id`.
    ///
    /// * `meta` - metadata of the tuple to insert.
    /// * `tuple` - the tuple data to insert.
    /// * `lock_mgr` - optional lock manager for row-level locking.
    /// * `txn` - the transaction performing the insertion.
    /// * `oid` - the table OID (used for locking).
    pub fn insert_tuple(
        &self,
        meta: &TupleMeta,
        tuple: &Tuple,
        _lock_mgr: Option<&LockManager>,
        _txn: Option<&Transaction>,
        _oid: u32,
    ) -> Option<RID> {
        // Acquire the table latch to serialise modifications of the page
        // chain (especially `last_page_id`).
        let _latch_guard = self.latch.lock().unwrap();
        let mut last_page_id = self.last_page_id.load(Ordering::Relaxed);

        let mut page_guard = self.bpm.write_page(last_page_id, AccessType::Unknown);

        // Walk the page chain, allocating new pages as needed, until we find
        // a page that can accommodate the tuple.
        loop {
            let page = page_guard.as_ref::<TablePage>();
            if page.get_next_tuple_offset(meta, tuple).is_some() {
                break;
            }

            // If the page is empty and still cannot fit the tuple, the tuple
            // is too large.
            assert!(
                page.get_num_tuples() != 0,
                "tuple is too large, cannot insert"
            );

            let next_page_id = self.bpm.new_page();

            // Update the current page's next pointer.
            {
                let page_mut = page_guard.as_mut_ref::<TablePage>();
                page_mut.set_next_page_id(next_page_id);
            }

            let mut next_page_guard = self.bpm.write_page(next_page_id, AccessType::Unknown);
            {
                let next_page = next_page_guard.as_mut_ref::<TablePage>();
                next_page.init();
            }

            // Update `last_page_id` on the heap (the latch ensures no other
            // thread reads a stale value).
            // SAFETY: We hold the latch, so no other thread can read or write
            // `last_page_id` concurrently. The `bpm` field is safe to access
            // without the latch because `BufferPoolManager` has its own
            // internal synchronisation.
            self.last_page_id.store(next_page_id, Ordering::Relaxed);
            last_page_id = next_page_id;

            drop(page_guard);
            page_guard = next_page_guard;
        }

        // Insert the tuple.
        let slot_id = {
            let page = page_guard.as_mut_ref::<TablePage>();
            page.insert_tuple(meta, tuple)?
        };

        // Row-level locking is disabled by default (see `DISABLE_LOCK_MANAGER`
        // in the C++ config.h). The `LockManager` implementation in the Rust
        // project is a stub, so we skip the actual lock call.

        // Drop the page guard before returning.
        drop(page_guard);

        // The latch guard is dropped here, releasing the table lock.
        Some(RID::from_parts(last_page_id, slot_id as u32))
    }

    /// Updates the metadata of an existing tuple (without modifying its
    /// data content).
    pub fn update_tuple_meta(&self, meta: &TupleMeta, rid: RID) {
        let mut page_guard = self.bpm.write_page(rid.page_id(), AccessType::Unknown);
        let page = page_guard.as_mut_ref::<TablePage>();
        page.update_tuple_meta(meta, rid);
    }

    /// Reads a tuple (metadata + data) from the table at the given `rid`.
    ///
    /// If you only need the metadata, consider using `get_tuple_meta`
    /// instead.
    pub fn get_tuple(&self, rid: RID) -> (TupleMeta, Tuple) {
        let page_guard = self.bpm.read_page(rid.page_id(), AccessType::Unknown);
        let page = page_guard.as_ref::<TablePage>();
        let (meta, mut tuple) = page.get_tuple(rid);
        tuple.set_rid(rid);
        (meta, tuple)
    }

    /// Reads only the metadata of a tuple from the table at the given `rid`.
    /// If you need both metadata and data, use `get_tuple` instead to ensure
    /// atomicity.
    pub fn get_tuple_meta(&self, rid: RID) -> TupleMeta {
        let page_guard = self.bpm.read_page(rid.page_id(), AccessType::Unknown);
        let page = page_guard.as_ref::<TablePage>();
        page.get_tuple_meta(rid)
    }

    /// Creates a `TableIterator` that records the last tuple at creation
    /// time and stops at that point. This avoids the Halloween problem when
    /// updating while scanning.
    ///
    /// You will typically use this method in project 3. In project 4, if the
    /// update executor is implemented as a pipeline breaker, `MakeIterator`
    /// and `MakeEagerIterator` should produce identical results.
    pub fn make_iterator(&self) -> TableIterator<'_> {
        // Acquire the latch to read `last_page_id` consistently.
        let _latch_guard = self.latch.lock().unwrap();
        let last_page_id = self.last_page_id.load(Ordering::Relaxed);

        // The latch is held while reading the page metadata so that a
        // concurrent insert cannot change the page we are examining.
        let page_guard = self.bpm.read_page(last_page_id, AccessType::Unknown);
        let page = page_guard.as_ref::<TablePage>();
        let num_tuples = page.get_num_tuples();
        drop(page_guard);

        // Release the latch before creating the iterator.
        drop(_latch_guard);

        TableIterator::new(
            self,
            RID::from_parts(self.first_page_id, 0),
            RID::from_parts(last_page_id, num_tuples as u32),
        )
    }

    /// Creates a `TableIterator` that scans until the end of the table (no
    /// artificial stop point).
    pub fn make_eager_iterator(&self) -> TableIterator<'_> {
        TableIterator::new(self, RID::from_parts(self.first_page_id, 0), RID::new())
    }

    /// Returns the page ID of the first page of this table.
    pub fn get_first_page_id(&self) -> PageId {
        self.first_page_id
    }

    /// Updates a tuple in place.
    ///
    /// Should **NOT** be used in project 3. Implement project 3's update
    /// executor as delete + insert instead. You will need this method in
    /// project 4.
    ///
    /// Returns `true` if the update was performed, or `false` if the optional
    /// `check` callback returned `false`.
    ///
    /// * `meta` - new metadata for the tuple.
    /// * `tuple` - new data for the tuple.
    /// * `rid` - the RID of the tuple to update.
    /// * `check` - an optional callback invoked before the update. The update
    ///   is performed only if this callback returns `true`.
    pub fn update_tuple_in_place<F>(
        &self,
        meta: &TupleMeta,
        tuple: &Tuple,
        rid: RID,
        check: Option<F>,
    ) -> bool 
        where F: FnOnce(&TupleMeta, &Tuple, RID) -> bool
    {
        let mut page_guard = self.bpm.write_page(rid.page_id(), AccessType::Unknown);
        let page = page_guard.as_mut_ref::<TablePage>();
        let (old_meta, old_tuple) = page.get_tuple(rid);

        if check.is_none() || check.unwrap()(&old_meta, &old_tuple, rid) {
            page.update_tuple_in_place_unsafe(meta, tuple, rid);
            true
        } else {
            false
        }
    }

    /// Updates a tuple in place on a page that has already been write-locked.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `page` is a valid mutable pointer to the
    /// `TablePage` data within the currently held write guard. This method
    /// is intended to be used together with
    /// `acquire_table_page_write_lock`.
    #[allow(dead_code)]
    pub(crate) fn update_tuple_in_place_with_lock_acquired(
        &self,
        meta: &TupleMeta,
        tuple: &Tuple,
        rid: RID,
        page: &mut TablePage,
    ) {
        page.update_tuple_in_place_unsafe(meta, tuple, rid);
    }

    /// Reads a tuple from a page that has already been read-locked.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `page` is a valid pointer to the
    /// `TablePage` data within the currently held read guard.
    #[allow(dead_code)]
    pub(crate) fn get_tuple_with_lock_acquired(
        &self,
        rid: RID,
        page: &TablePage,
    ) -> (TupleMeta, Tuple) {
        let (meta, mut tuple) = page.get_tuple(rid);
        tuple.set_rid(rid);
        (meta, tuple)
    }

    /// Reads tuple metadata from a page that has already been read-locked.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `page` is a valid pointer to the
    /// `TablePage` data within the currently held read guard.
    #[allow(dead_code)]
    pub(crate) fn get_tuple_meta_with_lock_acquired(
        &self,
        rid: RID,
        page: &TablePage,
    ) -> TupleMeta {
        page.get_tuple_meta(rid)
    }
}

