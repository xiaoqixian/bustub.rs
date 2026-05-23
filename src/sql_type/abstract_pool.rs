//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// abstract_pool.rs
//
// Identification: src/sql_type/abstract_pool.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

/// Interface of a memory pool that can quickly allocate chunks of memory.
pub trait AbstractPool {
    /// Allocate a contiguous block of memory of the given size.
    /// Returns a non-null pointer if allocation is successful, or a null pointer
    /// if allocation fails.
    fn allocate(&mut self, size: usize) -> *mut u8;

    /// Returns the provided chunk of memory back into the pool.
    fn free(&mut self, ptr: *mut u8);
}


