//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// mod.rs
//
// Identification: src/common/mod.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

/// Frame identifier type.
pub type FrameId = i32;

/// Page identifier type.
pub type PageId = i32;

/// Transaction identifier type.
pub type TxnId = i64;

/// Log sequence number type.
pub type LSN = i32;

/// Slot offset type.
pub type SlotOffset = usize;

/// Sentinel value representing an invalid frame ID.
pub const INVALID_FRAME_ID: FrameId = -1;

/// Sentinel value representing an invalid page ID.
pub const INVALID_PAGE_ID: PageId = -1;

/// Sentinel value representing an invalid transaction ID.
pub const INVALID_TXN_ID: TxnId = -1;

/// Sentinel value representing an invalid log sequence number.
pub const INVALID_LSN: LSN = -1;

/// The size of a data page in bytes (4 KB).
pub const BUSTUB_PAGE_SIZE: usize = 4096;

/// The default number of frames in the buffer pool.
pub const BUFFER_POOL_SIZE: usize = 128;

/// The default backward k-distance for the LRU-K replacer.
pub const LRUK_REPLACER_K: usize = 10;

pub mod rid;
