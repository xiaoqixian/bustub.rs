// Date:   Tue May 26 22:37:10 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// table_page.rs
//
// Identification: src/storage/page/table_page.rs
//
// Copyright (c) 2015-2024, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::ptr;

use crate::common::rid::RID;
use crate::common::{BUSTUB_PAGE_SIZE, INVALID_PAGE_ID, PageId};
use crate::storage::table::tuple::{Tuple, TupleMeta};

/// The size of the table page header in bytes.
/// Contains: next_page_id (4) + num_tuples (2) + num_deleted_tuples (2) = 8.
pub(crate) const TABLE_PAGE_HEADER_SIZE: usize = 8;

/// The size of a single tuple info entry in bytes.
/// Contains: offset (2) + size (2) + TupleMeta (16, with padding) = 24.
pub(crate) const TUPLE_INFO_SIZE: usize = 24;

// ---------------------------------------------------------------------------
// RawTupleMeta
// ---------------------------------------------------------------------------

/// A raw tuple metadata entry with a guaranteed C-compatible layout,
/// ensuring a consistent on-disk format regardless of how the Rust
/// compiler lays out `TupleMeta`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RawTupleMeta {
    ts: i64,
    is_deleted: bool,
}

impl From<RawTupleMeta> for TupleMeta {
    fn from(raw: RawTupleMeta) -> Self {
        TupleMeta {
            ts: raw.ts,
            is_deleted: raw.is_deleted,
        }
    }
}

impl From<TupleMeta> for RawTupleMeta {
    fn from(meta: TupleMeta) -> Self {
        RawTupleMeta {
            ts: meta.ts,
            is_deleted: meta.is_deleted,
        }
    }
}

// ---------------------------------------------------------------------------
// RawTupleInfo
// ---------------------------------------------------------------------------

/// A single slot-directory entry stored in a `TablePage`.
///
/// Layout (24 bytes total):
///   [offset: u16, size: u16, padding: 4, ts: i64, is_deleted: bool, padding: 7]
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RawTupleInfo {
    offset: u16,
    size: u16,
    meta: RawTupleMeta,
}

// ---------------------------------------------------------------------------
// TablePage
// ---------------------------------------------------------------------------

/// `TablePage` represents a single physical page of a slotted-page
/// table heap stored inside a buffer pool frame.
///
/// Slotted page format:
/// ```text
///  ---------------------------------------------------------
///  | HEADER | ... FREE SPACE ... | ... INSERTED TUPLES ... |
///  ---------------------------------------------------------
///                                ^
///                                free space pointer
///
///  Header format (size in bytes):
///  --------------------------------------------------------------------
///  | NextPageId (4) | NumTuples (2) | NumDeletedTuples (2) |
///  --------------------------------------------------------------------
///  ----------------------------------------------------------------
///  | Tuple_1 offset+size (4) | Tuple_2 offset+size (4) | ... |
///  ----------------------------------------------------------------
///
/// Tuple format:
/// | meta | data |
/// ```
#[repr(C)]
pub(crate) struct TablePage {
    pub(crate) next_page_id: PageId,
    pub(crate) num_tuples: u16,
    pub(crate) num_deleted_tuples: u16,
}

impl TablePage {
    /// Returns a raw pointer to the start of the page data (the first byte
    /// of the header).
    fn data_ptr(&self) -> *const u8 {
        self as *const Self as *const u8
    }

    /// Returns a raw mutable pointer to the start of the page data.
    fn data_mut_ptr(&mut self) -> *mut u8 {
        self as *mut Self as *mut u8
    }

    /// Returns a pointer to the `index`-th `RawTupleInfo` entry in the slot
    /// directory.
    fn tuple_info_ref(&self, index: u16) -> &RawTupleInfo {
        let offset = TABLE_PAGE_HEADER_SIZE + index as usize * TUPLE_INFO_SIZE;
        unsafe { &*(self.data_ptr().add(offset) as *const RawTupleInfo) }
    }

    /// Returns a mutable pointer to the `index`-th `RawTupleInfo` entry.
    fn tuple_info_mut_ref(&mut self, index: u16) -> &mut RawTupleInfo {
        let offset = TABLE_PAGE_HEADER_SIZE + index as usize * TUPLE_INFO_SIZE;
        unsafe { &mut *(self.data_mut_ptr().add(offset) as *mut RawTupleInfo) }
    }

    /// Initializes the table page header.
    pub(crate) fn init(&mut self) {
        self.next_page_id = INVALID_PAGE_ID;
        self.num_tuples = 0;
        self.num_deleted_tuples = 0;
    }

    /// Returns the number of tuples currently stored in this page.
    pub(crate) fn get_num_tuples(&self) -> u16 {
        self.num_tuples
    }

    /// Returns the page ID of the next page in the table.
    #[allow(dead_code)]
    pub(crate) fn get_next_page_id(&self) -> PageId {
        self.next_page_id
    }

    /// Sets the page ID of the next page in the table.
    pub(crate) fn set_next_page_id(&mut self, next_page_id: PageId) {
        self.next_page_id = next_page_id;
    }

    /// Computes the byte offset at which a new tuple of the given `tuple`
    /// would be placed. Returns `None` if the tuple is too large to fit in
    /// the remaining free space.
    pub(crate) fn get_next_tuple_offset(&self, _meta: &TupleMeta, tuple: &Tuple) -> Option<u16> {
        let slot_end_offset: usize = if self.num_tuples > 0 {
            let info = self.tuple_info_ref(self.num_tuples - 1);
            info.offset as usize
        } else {
            BUSTUB_PAGE_SIZE
        };

        let tuple_len = tuple.get_length() as usize;
        let tuple_offset = slot_end_offset.checked_sub(tuple_len)?;
        let offset_size = TABLE_PAGE_HEADER_SIZE + TUPLE_INFO_SIZE * (self.num_tuples as usize + 1);

        if tuple_offset < offset_size {
            return None;
        }

        Some(tuple_offset as u16)
    }

    /// Inserts a tuple into this page. Returns the slot ID (i.e., the
    /// index in the slot directory) on success, or `None` if there is
    /// not enough free space.
    pub(crate) fn insert_tuple(&mut self, meta: &TupleMeta, tuple: &Tuple) -> Option<u16> {
        let tuple_offset = self.get_next_tuple_offset(meta, tuple)?;
        let tuple_id = self.num_tuples;

        // Write the slot-directory entry.
        *self.tuple_info_mut_ref(tuple_id) = RawTupleInfo {
            offset: tuple_offset,
            size: tuple.get_length() as u16,
            meta: RawTupleMeta::from(*meta),
        };

        // Copy the tuple data into the page.
        let dest = unsafe { self.data_mut_ptr().add(tuple_offset as usize) };
        unsafe {
            ptr::copy_nonoverlapping(tuple.get_data().as_ptr(), dest, tuple.get_length() as usize);
        }

        self.num_tuples += 1;
        Some(tuple_id)
    }

    /// Reads a tuple (tuple data + metadata) from the page at the given
    /// `rid`.
    pub(crate) fn get_tuple(&self, rid: RID) -> (TupleMeta, Tuple) {
        let tuple_id = rid.slot_num();
        assert!(
            (tuple_id as u16) < self.num_tuples,
            "Tuple ID out of range"
        );

        let info = self.tuple_info_ref(tuple_id as u16);
        let meta = TupleMeta::from(info.meta);
        let offset = info.offset as usize;
        let size = info.size as usize;

        let mut data = vec![0u8; size];
        let src = unsafe { self.data_ptr().add(offset) };
        unsafe {
            ptr::copy_nonoverlapping(src, data.as_mut_ptr(), size);
        }

        let tuple = Tuple::new_with_data(rid, &data, size as u32);
        (meta, tuple)
    }

    /// Reads only the metadata of a tuple from the page.
    pub(crate) fn get_tuple_meta(&self, rid: RID) -> TupleMeta {
        let tuple_id = rid.slot_num();
        assert!(
            (tuple_id as u16) < self.num_tuples,
            "Tuple ID out of range"
        );

        let info = self.tuple_info_ref(tuple_id as u16);
        TupleMeta::from(info.meta)
    }

    /// Updates the metadata of an existing tuple in the slot directory.
    pub(crate) fn update_tuple_meta(&mut self, meta: &TupleMeta, rid: RID) {
        let tuple_id = rid.slot_num();
        assert!(
            (tuple_id as u16) < self.num_tuples,
            "Tuple ID out of range"
        );

        let old_deleted = {
            let info = self.tuple_info_ref(tuple_id as u16);
            info.meta.is_deleted
        };

        if !old_deleted && meta.is_deleted {
            self.num_deleted_tuples += 1;
        }
        let info = self.tuple_info_mut_ref(tuple_id as u16);
        info.meta = RawTupleMeta::from(*meta);
    }

    /// Unsafely updates a tuple in place. The new tuple must have the same
    /// byte size as the existing tuple, otherwise this method will panic.
    pub(crate) fn update_tuple_in_place_unsafe(
        &mut self,
        meta: &TupleMeta,
        tuple: &Tuple,
        rid: RID,
    ) {
        let tuple_id = rid.slot_num();
        assert!(
            (tuple_id as u16) < self.num_tuples,
            "Tuple ID out of range"
        );

        let old_deleted = {
            let info = self.tuple_info_ref(tuple_id as u16);
            assert!(
                info.size == tuple.get_length() as u16,
                "Tuple size mismatch"
            );
            info.meta.is_deleted
        };

        if !old_deleted && meta.is_deleted {
            self.num_deleted_tuples += 1;
        }

        let offset = {
            let info = self.tuple_info_mut_ref(tuple_id as u16);
            info.meta = RawTupleMeta::from(*meta);
            info.offset as usize
        };

        let dest = unsafe { self.data_mut_ptr().add(offset) };
        unsafe {
            ptr::copy_nonoverlapping(tuple.get_data().as_ptr(), dest, tuple.get_length() as usize);
        }
    }
}


