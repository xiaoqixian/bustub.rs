// Date:   Sun May 17 15:32:56 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// config.rs
//
// Identification: src/common/config.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

/// frame id type
pub type FrameId = i32;

/// page id type
pub type PageId = i32;

/// transaction id type
pub type TxnId = i64;

/// log sequence number type
pub type LSN = i32;

/// slot offset type
pub type SlotOffset = usize;

/// invalid frame id
pub const INVALID_FRAME_ID: FrameId = -1;

/// invalid page id
pub const INVALID_PAGE_ID: PageId = -1;

/// invalid transaction id
pub const INVALID_TXN_ID: TxnId = -1;

/// invalid log sequence number
pub const INVALID_LSN: LSN = -1;

/// size of a data page in byte
pub const BUSTUB_PAGE_SIZE: usize = 4096;

/// size of buffer pool
pub const BUFFER_POOL_SIZE: usize = 128;

/// backward k-distance for lru-k
pub const LRUK_REPLACER_K: usize = 10;


