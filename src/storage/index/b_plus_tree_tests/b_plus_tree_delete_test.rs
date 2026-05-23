//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// b_plus_tree_delete_test.rs
//
// Identification: src/storage/index/tests/b_plus_tree_delete_test.rs
//
// Copyright (c) 2015-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::Arc;
use crate::{
    buffer::buffer_pool_manager::BufferPoolManager,
    common::{LRUK_REPLACER_K, INVALID_PAGE_ID, rid::RID},
    storage::{
        disk::disk_manager_memory::DiskManagerMemory,
        index::b_plus_tree::BPlusTree,
    }
};
use super::{IntComparator, int_comparator};

/// Insert sequential keys {1, 2, 3, 4, 5}, then remove {1, 5, 3, 4} and
/// verify only key 2 remains. Finally remove key 2 and verify root page is
/// invalid.
#[test]
fn delete_test_no_iterator() {
    let disk_manager = Arc::new(DiskManagerMemory::new());
    let bpm = BufferPoolManager::new(50, disk_manager, LRUK_REPLACER_K);

    let page_id = bpm.new_page();
    let mut tree = BPlusTree::<i64, RID, IntComparator>::new("foo_pk".to_owned(), bpm.clone(), page_id, 2, 3, int_comparator);

    let keys: Vec<i64> = vec![1, 2, 3, 4, 5];
    for &key in &keys {
        let value = key & 0xFFFFFFFF;
        let rid = RID::from_parts((key >> 32) as i32, value as u32);
        tree.insert(key, rid);
    }

    // Verify all keys were inserted
    for &key in &keys {
        let result = tree.get_value(&key);
        assert!(result.is_some());
        let found_rid = result.unwrap();
        let value = key & 0xFFFFFFFF;
        assert_eq!(found_rid.slot_num(), value as u32);
    }

    let remove_keys: Vec<i64> = vec![1, 5, 3, 4];
    for &key in &remove_keys {
        tree.remove(&key);
    }

    let mut size: i64 = 0;
    for &key in &keys {
        let result = tree.get_value(&key);
        match result {
            None => {
                // Key should be in the remove_keys list
                assert!(remove_keys.contains(&key));
            }
            Some(found_rid) => {
                assert_eq!(found_rid.page_id(), 0);
                assert_eq!(found_rid.slot_num(), key as u32);
                size += 1;
            }
        }
    }
    assert_eq!(size, 1);

    // Remove the remaining key
    tree.remove(&2);
    let root_page_id = tree.get_root_page_id();
    assert_eq!(root_page_id, INVALID_PAGE_ID);
}

/// Insert and remove keys in mixed order while checking tree consistency
/// for various leaf_max_size values (2..=5).
///
/// This corresponds to the C++ SequentialEdgeMixTest, which uses
/// `TreeValuesMatch` to verify tree consistency after each operation.
/// Since `TreeValuesMatch` is not available in Rust yet, we simply verify
/// that each key can be found or not found as expected after the full
/// sequence.
#[test]
fn sequential_edge_mix_test() {
    for leaf_max_size in 2..=5 {
        let disk_manager = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(50, disk_manager, LRUK_REPLACER_K);

        let page_id = bpm.new_page();
        let mut tree = BPlusTree::<i64, RID, IntComparator>::new(
            "foo_pk".to_owned(),
            bpm.clone(),
            page_id,
            leaf_max_size,
            3,
            int_comparator,
        );

        // Insert some keys
        let keys_to_insert: Vec<i64> = vec![1, 5, 15, 20, 25, 2, -1, -2, 6, 14, 4];
        let mut inserted: Vec<i64> = Vec::new();
        for &key in &keys_to_insert {
            let value = key & 0xFFFFFFFF;
            let rid = RID::from_parts((key >> 32) as i32, value as u32);
            tree.insert(key, rid);
            inserted.push(key);
            // After each insert, verify all inserted keys exist
            for &k in &inserted {
                assert!(tree.get_value(&k).is_some(), "key {} should exist after insert", k);
            }
        }

        // Remove key 1, then verify
        tree.remove(&1);
        inserted.retain(|&k| k != 1);
        for &k in &inserted {
            assert!(tree.get_value(&k).is_some(), "key {} should exist after remove(1)", k);
        }
        assert!(tree.get_value(&1).is_none(), "key 1 should be removed");

        // Insert key 3
        tree.insert(3, RID::from_parts(3, 3));
        inserted.push(3);
        for &k in &inserted {
            assert!(tree.get_value(&k).is_some(), "key {} should exist after insert(3)", k);
        }

        // Remove all keys in specific order
        let remove_order: Vec<i64> = vec![4, 14, 6, 2, 15, -2, -1, 3, 5, 25, 20];
        for &key in &remove_order {
            tree.remove(&key);
            inserted.retain(|&k| k != key);
            for &k in &inserted {
                assert!(tree.get_value(&k).is_some(), "key {} should exist after remove({})", k, key);
            }
            assert!(tree.get_value(&key).is_none(), "key {} should be removed", key);
        }

        // All keys should be removed
        assert!(inserted.is_empty(), "all keys should be deleted");
    }
}


