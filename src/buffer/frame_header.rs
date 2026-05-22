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

use crate::common::{BUSTUB_PAGE_SIZE, INVALID_PAGE_ID, PageId};
use crate::common::FrameId;

/// A helper class for `BufferPoolManager` that manages a frame of memory and
/// related metadata.
///
/// This struct represents headers for frames of memory that the
/// `BufferPoolManager` stores pages of data into. The actual frame data is
/// stored directly inside `FrameHeader` as a `Vec<u8>` of `BUSTUB_PAGE_SIZE`
/// bytes.
///
/// ---
///
/// In a traditional production buffer pool manager, all memory that the
/// buffer pool manages would be allocated in one large contiguous array and
/// then divided into page-sized frames. In BusTub, each frame is instead
/// allocated separately (via its own `Vec<u8>`) so that buffer overflows can
/// be easily detected by address sanitizer. If frames were contiguous, it
/// would be very easy to cast a page's data pointer to a larger type and
/// accidentally overwrite adjacent pages.
#[allow(dead_code)]
pub struct FrameHeader {
    /// The frame ID / index of the frame this header represents.
    pub(crate) frame_id: FrameId,

    /// The number of pins on this frame keeping the page in memory.
    pub(crate) pin_count: usize,

    /// The dirty flag — set to `true` when the page has been modified and
    /// needs to be flushed to disk.
    pub(crate) is_dirty: bool,

    /// The ID of the page currently stored in this frame, or
    /// `INVALID_PAGE_ID` if the frame is empty.
    pub(crate) page_id: PageId,

    /// The actual page data. Allocated as a `Vec<u8>` of `BUSTUB_PAGE_SIZE`
    /// bytes so that ASan can detect out-of-bounds writes.
    pub(crate) data: Vec<u8>,
}

impl FrameHeader {
    /// Creates a new `FrameHeader` for the given frame ID.
    ///
    /// The frame is initialized to all zeroes, with `pin_count = 0`,
    /// `is_dirty = false`, and `page_id = INVALID_PAGE_ID`.
    pub fn new(frame_id: FrameId) -> Self {
        let mut this = FrameHeader {
            frame_id,
            pin_count: 0,
            is_dirty: false,
            page_id: INVALID_PAGE_ID,
            data: vec![0u8; BUSTUB_PAGE_SIZE],
        };
        this.reset();
        this
    }

    /// Resets the frame header to its default state: zeroes the page data,
    /// clears the pin count, marks the frame as clean, and sets the page ID
    /// to `INVALID_PAGE_ID`.
    pub(crate) fn reset(&mut self) {
        self.data.fill(0);
        self.pin_count = 0;
        self.is_dirty = false;
        self.page_id = INVALID_PAGE_ID;
    }
}

