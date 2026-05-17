// Date:   Sun May 17 20:28:13 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
// Date:   Sun May 17 16:12:00 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// page_guard.rs
//
// Identification: src/storage/page/page_guard.rs
//
// Copyright (c) 2015-2024, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::{Arc, Mutex};

use crate::buffer::frame_header::FrameHeader;
use crate::buffer::lru_k_replacer::LRUKReplacer;
use crate::common::PageId;

// ---------------------------------------------------------------------------
// ReadPageGuard
// ---------------------------------------------------------------------------

/// An RAII object that grants thread-safe read access to a page of data.
///
/// The _only_ way the system should interact with the buffer pool's page data
/// is via page guards. With `ReadPageGuard`s, there can be multiple threads
/// that share read access to a page's data. However, the existence of any
/// `ReadPageGuard` on a page implies that no thread can be mutating the page's
/// data.
#[allow(dead_code)]
pub struct ReadPageGuard {
    /// The page ID of the page we are guarding.
    page_id: PageId,

    /// The frame that holds the page this guard is protecting.
    frame: Option<Arc<FrameHeader>>,

    /// A shared pointer to the buffer pool's replacer.
    replacer: Option<Arc<Mutex<LRUKReplacer>>>,

    /// A shared pointer to the buffer pool's latch.
    bpm_latch: Option<Arc<Mutex<()>>>,

    /// The validity flag. Only valid if constructed by the `BufferPoolManager`.
    is_valid: bool,
}

impl ReadPageGuard {
    /// Creates a default, **invalid** `ReadPageGuard`.
    ///
    /// Use of an invalid guard is undefined behavior. This constructor exists
    /// only to enable move-assignment patterns (placing an uninitialized guard
    /// on the stack, then assigning to it later).
    pub fn new() -> Self {
        ReadPageGuard {
            page_id: 0,
            frame: None,
            replacer: None,
            bpm_latch: None,
            is_valid: false,
        }
    }

    /// Creates a valid `ReadPageGuard`. Only the `BufferPoolManager` should
    /// call this constructor.
    ///
    /// TODO(P1): Add implementation.
    #[allow(dead_code)]
    pub(crate) fn create(
        _page_id: PageId,
        _frame: Arc<FrameHeader>,
        _replacer: Arc<Mutex<LRUKReplacer>>,
        _bpm_latch: Arc<Mutex<()>>,
    ) -> Self {
        todo!("TODO(P1): Add implementation.")
    }

    /// Gets the page ID of the page this guard is protecting.
    pub fn get_page_id(&self) -> PageId {
        debug_assert!(self.is_valid, "tried to use an invalid read guard");
        self.page_id
    }

    /// Gets a `const` pointer to the page of data this guard is protecting.
    ///
    /// TODO(P1): Add implementation.
    pub fn get_data(&self) -> &[u8] {
        todo!("TODO(P1): Add implementation.")
    }

    /// Returns whether the page is dirty (modified but not flushed to disk).
    ///
    /// TODO(P1): Add implementation.
    pub fn is_dirty(&self) -> bool {
        todo!("TODO(P1): Add implementation.")
    }

    /// Manually drops a valid `ReadPageGuard`'s data. If this guard is
    /// invalid, this function does nothing.
    ///
    /// TODO(P1): Add implementation.
    pub fn drop_guard(&mut self) {
        if !self.is_valid {
            return;
        }
        todo!("TODO(P1): Add implementation.");
    }
}

impl Default for ReadPageGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ReadPageGuard {
    fn drop(&mut self) {
        self.drop_guard();
    }
}

// ---------------------------------------------------------------------------
// WritePageGuard
// ---------------------------------------------------------------------------

/// An RAII object that grants thread-safe write access to a page of data.
///
/// The _only_ way the system should interact with the buffer pool's page data
/// is via page guards. With a `WritePageGuard`, there can only be 1 thread
/// that has exclusive ownership over the page's data. The owner can mutate the
/// page's data as much as they want.
#[allow(dead_code)]
pub struct WritePageGuard {
    /// The page ID of the page we are guarding.
    page_id: PageId,

    /// The frame that holds the page this guard is protecting.
    frame: Option<Arc<FrameHeader>>,

    /// A shared pointer to the buffer pool's replacer.
    replacer: Option<Arc<Mutex<LRUKReplacer>>>,

    /// A shared pointer to the buffer pool's latch.
    bpm_latch: Option<Arc<Mutex<()>>>,

    /// The validity flag. Only valid if constructed by the `BufferPoolManager`.
    is_valid: bool,
}

impl WritePageGuard {
    /// Creates a default, **invalid** `WritePageGuard`.
    ///
    /// Use of an invalid guard is undefined behavior. This constructor exists
    /// only to enable move-assignment patterns (placing an uninitialized guard
    /// on the stack, then assigning to it later).
    pub fn new() -> Self {
        WritePageGuard {
            page_id: 0,
            frame: None,
            replacer: None,
            bpm_latch: None,
            is_valid: false,
        }
    }

    /// Creates a valid `WritePageGuard`. Only the `BufferPoolManager` should
    /// call this constructor.
    ///
    /// TODO(P1): Add implementation.
    #[allow(dead_code)]
    pub(crate) fn create(
        _page_id: PageId,
        _frame: Arc<FrameHeader>,
        _replacer: Arc<Mutex<LRUKReplacer>>,
        _bpm_latch: Arc<Mutex<()>>,
    ) -> Self {
        todo!("TODO(P1): Add implementation.")
    }

    /// Gets the page ID of the page this guard is protecting.
    pub fn get_page_id(&self) -> PageId {
        debug_assert!(self.is_valid, "tried to use an invalid write guard");
        self.page_id
    }

    /// Gets a `const` pointer to the page of data this guard is protecting.
    ///
    /// TODO(P1): Add implementation.
    pub fn get_data(&self) -> &[u8] {
        todo!("TODO(P1): Add implementation.")
    }

    /// Gets a mutable pointer to the page of data this guard is protecting.
    ///
    /// TODO(P1): Add implementation.
    pub fn get_data_mut(&mut self) -> &mut [u8] {
        todo!("TODO(P1): Add implementation.")
    }

    /// Returns whether the page is dirty (modified but not flushed to disk).
    ///
    /// TODO(P1): Add implementation.
    pub fn is_dirty(&self) -> bool {
        todo!("TODO(P1): Add implementation.")
    }

    /// Manually drops a valid `WritePageGuard`'s data. If this guard is
    /// invalid, this function does nothing.
    ///
    /// TODO(P1): Add implementation.
    pub fn drop_guard(&mut self) {
        if !self.is_valid {
            return;
        }
        todo!("TODO(P1): Add implementation.");
    }
}

impl Default for WritePageGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WritePageGuard {
    fn drop(&mut self) {
        self.drop_guard();
    }
}


