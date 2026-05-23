use std::sync::Arc;
use crate::{
    buffer::{buffer_pool_manager::BufferPoolManager, lru_k_replacer::AccessType},
    common::{LRUK_REPLACER_K, rid::RID},
    storage::{
        disk::disk_manager_memory::DiskManagerMemory,
        index::b_plus_tree::BPlusTree,
        page::b_plus_tree_page::{BPlusTreeLeafPage, PageMetaData}
    }
};
use super::{IntComparator, int_comparator};

#[test]
fn basic_insert_test() {
    let disk_manager = Arc::new(DiskManagerMemory::new());
    let bpm = BufferPoolManager::new(50, disk_manager, LRUK_REPLACER_K);

    let page_id = bpm.new_page();
    let mut tree = BPlusTree::<i64, RID, IntComparator>::new("foo_pk".to_owned(), bpm.clone(), page_id, 2, 3, int_comparator);

    let key = 42i64;
    let value = key & 0xFFFFFFFF;
    let rid = RID::from_parts(key as i32, value as u32);
    let index_key = key;

    tree.insert(index_key, rid);
    let root_page_id = tree.get_root_page_id();
    let root_page_guard = bpm.read_page(root_page_id, AccessType::Unknown);
    assert!(!root_page_guard.as_ptr().is_null());
    unsafe {
        let ptr = root_page_guard.as_ptr();
        assert!((*(ptr as *const PageMetaData)).is_leaf);
    }

    unsafe {
        let ptr = root_page_guard.as_ptr();
        let leaf = &*(ptr as *const BPlusTreeLeafPage<i64, RID>);
        assert_eq!(leaf.size, 1);
        assert_eq!(leaf.key_at(0), Some(&42));
    }
}

/// Insert sequential keys {1, 2, 3, 4, 5} and verify each can be found with GetValue.
#[test]
fn insert_test_1_no_iterator() {
    let disk_manager = Arc::new(DiskManagerMemory::new());
    let bpm = BufferPoolManager::new(50, disk_manager, LRUK_REPLACER_K);

    let page_id = bpm.new_page();
    let mut tree = BPlusTree::<i64, RID, IntComparator>::new("foo_pk".to_owned(), bpm.clone(), page_id, 2, 3, int_comparator);

    let keys: Vec<i64> = vec![1, 2, 3, 4, 5];
    for &key in &keys {
        let value = key & 0xFFFFFFFF;
        let rid = RID::from_parts((key >> 32) as i32, value as u32);
        let index_key = key;
        tree.insert(index_key, rid);
    }

    for &key in &keys {
        let index_key = key;
        let result = tree.get_value(&index_key);
        assert!(result.is_some());
        let found_rid = result.unwrap();
        assert_eq!(found_rid.page_id(), 0);
        let value = key & 0xFFFFFFFF;
        assert_eq!(found_rid.slot_num(), value as u32);
    }
}

/// Insert keys in reverse order {5, 4, 3, 2, 1}, verify with GetValue
/// and also verify iteration order.
#[test]
fn insert_test_2() {
    let disk_manager = Arc::new(DiskManagerMemory::new());
    let bpm = BufferPoolManager::new(50, disk_manager, LRUK_REPLACER_K);

    let page_id = bpm.new_page();
    let mut tree = BPlusTree::<i64, RID, IntComparator>::new("foo_pk".to_owned(), bpm.clone(), page_id, 2, 3, int_comparator);

    let keys: Vec<i64> = vec![5, 4, 3, 2, 1];
    for &key in &keys {
        let value = key & 0xFFFFFFFF;
        let rid = RID::from_parts((key >> 32) as i32, value as u32);
        let index_key = key;
        tree.insert(index_key, rid);
    }

    // Verify each key individually
    for &key in &keys {
        let result = tree.get_value(&key);
        assert!(result.is_some());
        let found_rid = result.unwrap();
        let value = key & 0xFFFFFFFF;
        assert_eq!(found_rid.slot_num(), value as u32);
    }

    // Iterate from beginning
    let mut current_key: i64 = 1;
    for (k, v) in tree.iter() {
        assert_eq!(v.page_id(), 0);
        assert_eq!(v.slot_num(), current_key as u32);
        let _ = k;
        current_key += 1;
    }
    assert_eq!(current_key, keys.len() as i64 + 1);

    // Iterate starting from key 3
    current_key = 3;
    for (k, v) in tree.iter_from(&3) {
        assert_eq!(v.page_id(), 0);
        assert_eq!(v.slot_num(), current_key as u32);
        let _ = k;
        current_key += 1;
    }
}
