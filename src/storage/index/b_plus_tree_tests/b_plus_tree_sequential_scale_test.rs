//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// b_plus_tree_sequential_scale_test.rs
//
// Identification: src/storage/index/tests/b_plus_tree_sequential_scale_test.rs
//
// Copyright (c) 2024-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::Arc;
use rand::seq::SliceRandom;
use rand::thread_rng;
use crate::{
    buffer::buffer_pool_manager::BufferPoolManager,
    common::{LRUK_REPLACER_K, rid::RID},
    storage::{
        disk::disk_manager_memory::DiskManagerMemory,
        index::b_plus_tree::BPlusTree,
    }
};


// ---------------------------------------------------------------------------
// Scale tests
// ---------------------------------------------------------------------------

/// Insert 5000 keys in random order and verify each can be found.
#[test]
fn basic_scale_test() {
    let disk_manager = Arc::new(DiskManagerMemory::new());
    let bpm = BufferPoolManager::new(30, disk_manager, LRUK_REPLACER_K);

    let page_id = bpm.new_page();
    let mut tree = BPlusTree::<i64, RID>::new("foo_pk".to_owned(), bpm.clone(), page_id, 2, 3);

    let scale: i64 = 5000;
    let mut keys: Vec<i64> = (1..=scale).collect();

    // Randomize insertion order
    let mut rng = thread_rng();
    keys.shuffle(&mut rng);

    for &key in &keys {
        let value = key & 0xFFFFFFFF;
        let rid = RID::from_parts((key >> 32) as i32, value as u32);
        tree.insert(key, rid);
    }

    // Verify each key can be found
    for &key in &keys {
        let result = tree.get_value(&key);
        assert!(result.is_some(), "key {} should exist", key);
        let rid = result.unwrap();
        assert_eq!(rid.page_id(), 0);
        let value = key & 0xFFFFFFFF;
        assert_eq!(rid.slot_num(), value as u32);
    }
}


