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

use std::sync::{RwLockReadGuard, RwLockWriteGuard};

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
pub struct ReadPageGuard<'a> {
    /// The page ID of the page we are guarding.
    pub(crate) page_id: PageId,

    /// The frame that holds the page this guard is protecting.
    pub(crate) frame: RwLockReadGuard<'a, FrameHeader>,

    /// A shared pointer to the buffer pool's replacer.
    pub(crate) replacer: LRUKReplacer,
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
pub struct WritePageGuard<'a> {
    /// The page ID of the page we are guarding.
    pub(crate) page_id: PageId,

    /// The frame that holds the page this guard is protecting.
    pub(crate) frame: RwLockWriteGuard<'a, FrameHeader>,

    /// A shared pointer to the buffer pool's replacer.
    pub(crate) replacer: LRUKReplacer,
}

impl<'a> ReadPageGuard<'a> {
    pub fn new(page_id: PageId, frame: RwLockReadGuard<'a, FrameHeader>, replacer: LRUKReplacer) -> Self {
        Self {
            page_id,
            frame,
            replacer
        }
    }

    /// Gets the page ID of the page this guard is protecting.
    pub fn get_page_id(&self) -> PageId {
        self.page_id
    }

    /// Gets a `const` pointer to the page of data this guard is protecting.
    ///
    /// TODO(P1): Add implementation.
    pub fn as_ptr(&self) -> *const u8 {
        todo!("TODO(P1): Add implementation.")
    }

    pub fn as_slice(&self) -> &[u8] {
        self.frame.data.as_slice()
    }

    /// Returns whether the page is dirty (modified but not flushed to disk).
    ///
    /// TODO(P1): Add implementation.
    pub fn is_dirty(&self) -> bool {
        todo!("TODO(P1): Add implementation.")
    }
}

impl<'a> WritePageGuard<'a> {
    pub fn new(page_id: PageId, frame: RwLockWriteGuard<'a, FrameHeader>, replacer: LRUKReplacer) -> Self {
        Self {
            page_id,
            frame,
            replacer
        }
    }

    /// Gets the page ID of the page this guard is protecting.
    pub fn get_page_id(&self) -> PageId {
        self.page_id
    }

    /// Gets a `const` pointer to the page of data this guard is protecting.
    ///
    /// TODO(P1): Add implementation.
    pub fn as_ptr(&self) -> *const u8 {
        todo!("TODO(P1): Add implementation.")
    }

    /// Gets a mutable pointer to the page of data this guard is protecting.
    ///
    /// TODO(P1): Add implementation.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        todo!("TODO(P1): Add implementation.")
    }

    pub fn as_slice(&self) -> &[u8] {
        self.frame.data.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.frame.data.as_mut_slice()
    }

    /// Returns whether the page is dirty (modified but not flushed to disk).
    ///
    /// TODO(P1): Add implementation.
    pub fn is_dirty(&self) -> bool {
        todo!("TODO(P1): Add implementation.")
    }

}
