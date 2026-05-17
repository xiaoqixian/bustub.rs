// Date:   Sun May 17 20:29:01 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
// Date:   Sun May 17 16:12:00 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// frame_header.rs
//
// Identification: src/buffer/frame_header.rs
//
// Copyright (c) 2015-2024, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::{Mutex, RwLock};

use crate::common::{BUSTUB_PAGE_SIZE, INVALID_PAGE_ID};
use crate::common::FrameId;

/// A helper class for `BufferPoolManager` that manages a frame of memory and
/// related metadata.
///
/// This struct represents headers for frames of memory that the
/// `BufferPoolManager` stores pages of data into. The actual frame data is
/// stored directly inside `FrameHeader` as a `Vec<u8>` of `BUSTUB_PAGE_SIZE`
/// bytes.
#[allow(dead_code)]
pub struct FrameHeader {
    /// The frame ID / index of the frame this header represents.
    pub(crate) frame_id: FrameId,

    /// The readers / writer latch for this frame.
    pub(crate) rwlatch: RwLock<()>,

    /// The number of pins on this frame keeping the page in memory.
    pub(crate) pin_count: AtomicUsize,

    /// The dirty flag.
    pub(crate) is_dirty: AtomicBool,

    /// The ID of the page currently stored in this frame, or
    /// `INVALID_PAGE_ID` if the frame is empty.
    pub(crate) page_id: AtomicI32,

    /// The actual page data. Protected by `Mutex` for interior mutability.
    pub(crate) data: Mutex<Vec<u8>>,
}

impl FrameHeader {
    /// Creates a new `FrameHeader` for the given frame ID.
    pub fn new(frame_id: FrameId) -> Self {
        let mut this = FrameHeader {
            frame_id,
            rwlatch: RwLock::new(()),
            pin_count: AtomicUsize::new(0),
            is_dirty: AtomicBool::new(false),
            page_id: AtomicI32::new(INVALID_PAGE_ID),
            data: Mutex::new(vec![0u8; BUSTUB_PAGE_SIZE]),
        };
        this.reset();
        this
    }

    /// Resets the frame header to its default state (zeroes data, clears
    /// pins, marks clean).
    pub(crate) fn reset(&mut self) {
        self.data.lock().unwrap().fill(0);
        self.pin_count.store(0, Ordering::Release);
        self.is_dirty.store(false, Ordering::Release);
        self.page_id.store(INVALID_PAGE_ID, Ordering::Release);
    }
}


