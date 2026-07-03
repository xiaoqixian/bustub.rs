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

use std::sync::{Mutex, RwLockReadGuard, RwLockWriteGuard};

use crate::buffer::frame_header::FrameHeader;
use crate::buffer::lru_k_replacer::LRUKReplacer;
use crate::common::PageId;

// ---------------------------------------------------------------------------
// ReadPageGuard
// ---------------------------------------------------------------------------

/// An RAII guard that grants thread-safe read access to a page of data.
///
/// The _only_ way the system should interact with the buffer pool's page data
/// is via page guards. With `ReadPageGuard`s, there can be multiple threads
/// that share read access to a page's data. However, the existence of any
/// `ReadPageGuard` on a page implies that no thread can be mutating the page's
/// data.
///
/// When the guard is dropped, it releases the read lock on the frame's
/// `RwLock` and decrements the pin count. If the pin count reaches zero,
/// the frame is marked as evictable in the replacer.
#[allow(dead_code)]
pub struct ReadPageGuard<'a> {
    /// The page ID of the page we are guarding.
    pub(crate) page_id: PageId,

    /// The read-locked frame that holds the page this guard is protecting.
    /// All operations on this page guard should be done via this
    /// `RwLockReadGuard`.
    pub(crate) frame: RwLockReadGuard<'a, FrameHeader>,

    pub(crate) pin_count: &'a Mutex<usize>,

    /// A shared pointer to the buffer pool's replacer.
    ///
    /// Since the buffer pool cannot know when this guard is destructed, we
    /// maintain a handle to the replacer in order to set the frame as
    /// evictable on destruction.
    pub(crate) replacer: LRUKReplacer,
}

// ---------------------------------------------------------------------------
// WritePageGuard
// ---------------------------------------------------------------------------

/// An RAII guard that grants thread-safe write access to a page of data.
///
/// The _only_ way the system should interact with the buffer pool's page data
/// is via page guards. With a `WritePageGuard`, only one thread can have
/// exclusive ownership over the page's data. The owner can mutate the page's
/// data as much as they want. The existence of a `WritePageGuard` implies that
/// no other `WritePageGuard` or any `ReadPageGuard`s for the same page can
/// exist at the same time.
///
/// When the guard is dropped, it releases the write lock on the frame's
/// `RwLock`, marks the page as dirty, and decrements the pin count. If the
/// pin count reaches zero, the frame is marked as evictable in the replacer.
#[allow(dead_code)]
pub struct WritePageGuard<'a> {
    /// The page ID of the page we are guarding.
    pub(crate) page_id: PageId,

    /// The write-locked frame that holds the page this guard is protecting.
    /// All operations on this page guard should be done via this
    /// `RwLockWriteGuard`.
    pub(crate) frame: RwLockWriteGuard<'a, FrameHeader>,

    pub(crate) pin_count: &'a Mutex<usize>,

    /// A shared pointer to the buffer pool's replacer.
    ///
    /// Since the buffer pool cannot know when this guard is destructed, we
    /// maintain a handle to the replacer in order to set the frame as
    /// evictable on destruction.
    pub(crate) replacer: LRUKReplacer,
}

impl<'a> ReadPageGuard<'a> {
    /// Only the buffer pool manager is allowed to construct a valid
    /// `ReadPageGuard`.
    pub fn new(page_id: PageId, frame: RwLockReadGuard<'a, FrameHeader>, pin_count: &'a Mutex<usize>, replacer: LRUKReplacer) -> Self {
        Self {
            page_id,
            frame,
            pin_count,
            replacer
        }
    }

    /// Gets the page ID of the page this guard is protecting.
    pub fn get_page_id(&self) -> PageId {
        self.page_id
    }

    /// Gets a raw `const` pointer to the page of data this guard is
    /// protecting.
    ///
    /// TODO(P1): Add implementation.
    pub fn as_ptr(&self) -> *const u8 {
        self.frame.data.as_ptr()
    }

    /// Returns an immutable reference to the page data as a `&[u8]` slice.
    pub fn as_slice(&self) -> &[u8] {
        self.frame.data.as_slice()
    }

    pub fn as_ref<T>(&self) -> &T {
        unsafe { &*(self.as_ptr() as *const T) }
    }
}

impl<'a> WritePageGuard<'a> {
    /// Only the buffer pool manager is allowed to construct a valid
    /// `WritePageGuard`.
    pub fn new(page_id: PageId, frame: RwLockWriteGuard<'a, FrameHeader>, pin_count: &'a Mutex<usize>, replacer: LRUKReplacer) -> Self {
        Self {
            page_id,
            frame,
            pin_count,
            replacer
        }
    }

    /// Gets the page ID of the page this guard is protecting.
    pub fn get_page_id(&self) -> PageId {
        self.page_id
    }

    /// Gets a raw `const` pointer to the page of data this guard is
    /// protecting.
    ///
    /// TODO(P1): Add implementation.
    pub fn as_ptr(&self) -> *const u8 {
        self.frame.data.as_ptr()
    }

    /// Gets a raw mutable pointer to the page of data this guard is
    /// protecting.
    ///
    /// TODO(P1): Add implementation.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.frame.data.as_mut_ptr()
    }

    /// Returns an immutable reference to the page data as a `&[u8]` slice.
    pub fn as_slice(&self) -> &[u8] {
        self.frame.data.as_slice()
    }

    /// Returns a mutable reference to the page data as a `&mut [u8]` slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.frame.data.as_mut_slice()
    }

    pub fn as_ref<T>(&self) -> &T {
        unsafe { &*(self.as_ptr() as *const T) }
    }

    pub fn as_mut_ref<T>(&mut self) -> &mut T {
        unsafe { &mut *(self.as_mut_ptr() as *mut T) }
    }
}

impl<'a> Drop for ReadPageGuard<'a> {
    fn drop(&mut self) {
        let mut pc_guard = self.pin_count.lock().unwrap();
        *pc_guard -= 1;
        if *pc_guard == 0 {
            self.replacer.set_evictable(self.frame.frame_id, true);
        }
    }
}

impl<'a> Drop for WritePageGuard<'a> {
    fn drop(&mut self) {
        let mut pc_guard = self.pin_count.lock().unwrap();
        *pc_guard -= 1;
        if *pc_guard == 0 {
            self.replacer.set_evictable(self.frame.frame_id, true);
        }
    }
}
