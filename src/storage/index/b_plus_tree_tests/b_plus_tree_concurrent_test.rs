//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// b_plus_tree_concurrent_test.rs
//
// Identification: src/storage/index/tests/b_plus_tree_concurrent_test.rs
//
// Copyright (c) 2015-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::{Arc, Mutex};
use std::thread;
use crate::{
    buffer::buffer_pool_manager::BufferPoolManager,
    common::{LRUK_REPLACER_K, rid::RID},
    storage::{
        disk::disk_manager_memory::DiskManagerMemory,
        index::b_plus_tree::BPlusTree,
    }
};
/// Number of iterations for Insert/Delete tests.
const NUM_ITERS: usize = 50;
/// Number of iterations for Mix tests.
const MIXTEST_NUM_ITERS: usize = 20;
/// Buffer pool size for concurrent tests.
const BPM_SIZE: usize = 50;

// ---------------------------------------------------------------------------
// Helper functions: insert / delete / lookup
// ---------------------------------------------------------------------------

type Tree = BPlusTree<i64, RID>;
type SharedTree = Arc<Mutex<Tree>>;

/// Insert all keys into the tree (each thread inserts the full set).
fn insert_helper(tree: &SharedTree, keys: &[i64]) {
    for &key in keys {
        let value = key & 0xFFFFFFFF;
        let rid = RID::from_parts((key >> 32) as i32, value as u32);
        tree.lock().unwrap().insert(key, rid);
    }
}

/// Insert keys split among threads by modulo.
fn insert_helper_split(tree: &SharedTree, keys: &[i64], total_threads: usize, thread_itr: usize) {
    for &key in keys {
        if key as usize % total_threads == thread_itr {
            let value = key & 0xFFFFFFFF;
            let rid = RID::from_parts((key >> 32) as i32, value as u32);
            tree.lock().unwrap().insert(key, rid);
        }
    }
}

/// Delete all keys from the tree (each thread deletes the full set).
fn delete_helper(tree: &SharedTree, keys: &[i64]) {
    for &key in keys {
        tree.lock().unwrap().remove(&key);
    }
}

/// Delete keys split among threads by modulo.
fn delete_helper_split(tree: &SharedTree, keys: &[i64], total_threads: usize, thread_itr: usize) {
    for &key in keys {
        if key as usize % total_threads == thread_itr {
            tree.lock().unwrap().remove(&key);
        }
    }
}

/// Look up each key and assert it exists with the expected value.
fn lookup_helper(tree: &SharedTree, keys: &[i64]) {
    for &key in keys {
        let guard = tree.lock().unwrap();
        let result = guard.get_value(&key);
        // NOTE: get_value returns Option<&V>, but the reference lifetime is
        // tied to the mutex guard. We must resolve it before the guard drops,
        // or we can just check is_some() and copy the values.
        assert!(result.is_some(), "key {} should exist", key);
    }
}

// ---------------------------------------------------------------------------
// Concurrent Insert Tests
// ---------------------------------------------------------------------------

/// Two threads concurrently insert all keys {1..100}, then verify.
#[test]
fn concurrent_insert_test_1() {
    for _ in 0..NUM_ITERS {
        let disk_manager = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(BPM_SIZE, disk_manager, LRUK_REPLACER_K);

        let page_id = bpm.new_page();
        let tree = BPlusTree::<i64, RID>::new("foo_pk".to_owned(), bpm.clone(), page_id, 3, 5);
        let shared_tree = Arc::new(Mutex::new(tree));

        let scale_factor: i64 = 100;
        let keys: Vec<i64> = (1..scale_factor).collect();

        let num_threads = 2;
        let mut handles = Vec::new();

        for _ in 0..num_threads {
            let tree = Arc::clone(&shared_tree);
            let keys = keys.clone();
            handles.push(thread::spawn(move || {
                insert_helper(&tree, &keys);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify each key exists
        let tree = shared_tree.lock().unwrap();
        for &key in &keys {
            let result = tree.get_value(&key);
            assert!(result.is_some());
            let rid = result.unwrap();
            assert_eq!(rid.page_id(), 0);
            let value = key & 0xFFFFFFFF;
            assert_eq!(rid.slot_num(), value as u32);
        }

        // Verify iteration order
        let mut current_key: i64 = 1;
        for (_k, v) in tree.iter() {
            assert_eq!(v.page_id(), 0);
            assert_eq!(v.slot_num(), current_key as u32);
            current_key += 1;
        }
        assert_eq!(current_key, keys.len() as i64 + 1);
    }
}

/// Two threads concurrently insert keys {1..1000} split by modulo, then verify.
#[test]
fn concurrent_insert_test_2() {
    for _ in 0..NUM_ITERS {
        let disk_manager = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(BPM_SIZE, disk_manager, LRUK_REPLACER_K);

        let page_id = bpm.new_page();
        let tree = BPlusTree::<i64, RID>::new("foo_pk".to_owned(), bpm.clone(), page_id, 3, 5);
        let shared_tree = Arc::new(Mutex::new(tree));

        let scale_factor: i64 = 1000;
        let keys: Vec<i64> = (1..scale_factor).collect();

        let num_threads = 2;
        let mut handles = Vec::new();

        for thread_itr in 0..num_threads {
            let tree = Arc::clone(&shared_tree);
            let keys = keys.clone();
            handles.push(thread::spawn(move || {
                insert_helper_split(&tree, &keys, num_threads, thread_itr);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify each key exists
        let tree = shared_tree.lock().unwrap();
        for &key in &keys {
            let result = tree.get_value(&key);
            assert!(result.is_some());
            let rid = result.unwrap();
            assert_eq!(rid.page_id(), 0);
            let value = key & 0xFFFFFFFF;
            assert_eq!(rid.slot_num(), value as u32);
        }

        // Verify iteration order
        let mut current_key: i64 = 1;
        for (_k, v) in tree.iter() {
            assert_eq!(v.page_id(), 0);
            assert_eq!(v.slot_num(), current_key as u32);
            current_key += 1;
        }
        assert_eq!(current_key, keys.len() as i64 + 1);
    }
}

// ---------------------------------------------------------------------------
// Concurrent Delete Tests
// ---------------------------------------------------------------------------

/// Insert {1..5}, then two threads concurrently delete {1, 5, 3, 4}.
/// Verify only key 2 remains.
#[test]
fn concurrent_delete_test_1() {
    for _ in 0..NUM_ITERS {
        let disk_manager = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(BPM_SIZE, disk_manager, LRUK_REPLACER_K);

        let page_id = bpm.new_page();
        let mut tree = BPlusTree::<i64, RID>::new("foo_pk".to_owned(), bpm.clone(), page_id, 3, 5);

        // Sequential insert
        let keys: Vec<i64> = vec![1, 2, 3, 4, 5];
        for &key in &keys {
            let value = key & 0xFFFFFFFF;
            let rid = RID::from_parts((key >> 32) as i32, value as u32);
            tree.insert(key, rid);
        }
        let shared_tree = Arc::new(Mutex::new(tree));

        let remove_keys: Vec<i64> = vec![1, 5, 3, 4];

        // Two threads delete
        let num_threads = 2;
        let mut handles = Vec::new();

        for _ in 0..num_threads {
            let tree = Arc::clone(&shared_tree);
            let remove_keys = remove_keys.clone();
            handles.push(thread::spawn(move || {
                delete_helper(&tree, &remove_keys);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Only key 2 should remain
        let tree = shared_tree.lock().unwrap();
        let mut current_key: i64 = 2;
        let mut size: i64 = 0;
        for (_k, v) in tree.iter() {
            assert_eq!(v.page_id(), 0);
            assert_eq!(v.slot_num(), current_key as u32);
            current_key += 1;
            size += 1;
        }
        assert_eq!(size, 1);
    }
}

/// Insert {1..10}, then two threads concurrently delete {1, 4, 3, 2, 5, 6}
/// split by modulo. Verify keys {7, 8, 9, 10} remain.
#[test]
fn concurrent_delete_test_2() {
    for _ in 0..NUM_ITERS {
        let disk_manager = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(BPM_SIZE, disk_manager, LRUK_REPLACER_K);

        let page_id = bpm.new_page();
        let mut tree = BPlusTree::<i64, RID>::new("foo_pk".to_owned(), bpm.clone(), page_id, 3, 5);

        // Sequential insert
        let keys: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        for &key in &keys {
            let value = key & 0xFFFFFFFF;
            let rid = RID::from_parts((key >> 32) as i32, value as u32);
            tree.insert(key, rid);
        }
        let shared_tree = Arc::new(Mutex::new(tree));

        let remove_keys: Vec<i64> = vec![1, 4, 3, 2, 5, 6];

        // Two threads delete split by modulo
        let num_threads = 2;
        let mut handles = Vec::new();

        for thread_itr in 0..num_threads {
            let tree = Arc::clone(&shared_tree);
            let remove_keys = remove_keys.clone();
            handles.push(thread::spawn(move || {
                delete_helper_split(&tree, &remove_keys, num_threads, thread_itr);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Keys 7, 8, 9, 10 should remain
        let tree = shared_tree.lock().unwrap();
        let mut current_key: i64 = 7;
        let mut size: i64 = 0;
        for (_k, v) in tree.iter() {
            assert_eq!(v.page_id(), 0);
            assert_eq!(v.slot_num(), current_key as u32);
            current_key += 1;
            size += 1;
        }
        assert_eq!(size, 4);
    }
}

// ---------------------------------------------------------------------------
// Concurrent Mix Tests
// ---------------------------------------------------------------------------

/// Mixed insert/delete: 10 threads interleave inserting even keys and
/// deleting odd keys. After all threads finish, only even keys remain.
#[test]
fn concurrent_mix_test_1() {
    for _ in 0..MIXTEST_NUM_ITERS {
        let disk_manager = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(BPM_SIZE, disk_manager, LRUK_REPLACER_K);

        let page_id = bpm.new_page();
        let mut tree = BPlusTree::<i64, RID>::new("foo_pk".to_owned(), bpm.clone(), page_id, 3, 5);

        // First, populate with odd keys (these will be deleted)
        let total_keys: i64 = 1000;
        let sieve: i64 = 2;

        let mut for_insert: Vec<i64> = Vec::new();
        let mut for_delete: Vec<i64> = Vec::new();
        for i in 1..=total_keys {
            if i % sieve == 0 {
                for_insert.push(i);
            } else {
                for_delete.push(i);
            }
        }

        // Insert all keys to delete first (so they exist to be deleted)
        for &key in &for_delete {
            let value = key & 0xFFFFFFFF;
            let rid = RID::from_parts((key >> 32) as i32, value as u32);
            tree.insert(key, rid);
        }
        let shared_tree = Arc::new(Mutex::new(tree));

        // Launch threads: half insert even keys, half delete odd keys
        let num_threads = 10;
        let mut handles = Vec::new();

        for thread_itr in 0..num_threads {
            if thread_itr % 2 == 0 {
                let tree = Arc::clone(&shared_tree);
                let for_insert = for_insert.clone();
                handles.push(thread::spawn(move || {
                    insert_helper(&tree, &for_insert);
                }));
            } else {
                let tree = Arc::clone(&shared_tree);
                let for_delete = for_delete.clone();
                handles.push(thread::spawn(move || {
                    delete_helper(&tree, &for_delete);
                }));
            }
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Only even keys should remain
        let tree = shared_tree.lock().unwrap();
        let mut size: i64 = 0;
        for (k, _v) in tree.iter() {
            assert_eq!(*k % sieve, 0, "key {} should be divisible by {}", k, sieve);
            size += 1;
        }
        assert_eq!(size, for_insert.len() as i64);
    }
}

/// Mixed insert/delete/lookup: 6 threads interleave inserting dynamic keys,
/// deleting dynamic keys, and looking up preserved keys. After all threads
/// finish, preserved keys must still exist.
#[test]
fn concurrent_mix_test_2() {
    for _ in 0..MIXTEST_NUM_ITERS {
        let disk_manager = Arc::new(DiskManagerMemory::new());
        let bpm = BufferPoolManager::new(BPM_SIZE, disk_manager, LRUK_REPLACER_K);

        let page_id = bpm.new_page();
        let mut tree = BPlusTree::<i64, RID>::new("foo_pk".to_owned(), bpm.clone(), page_id, 3, 5);

        let total_keys: i64 = 1000;
        let sieve: i64 = 5;

        let mut perserved_keys: Vec<i64> = Vec::new();
        let mut dynamic_keys: Vec<i64> = Vec::new();

        for i in 1..=total_keys {
            if i % sieve == 0 {
                perserved_keys.push(i);
            } else {
                dynamic_keys.push(i);
            }
        }

        // Insert preserved keys first
        for &key in &perserved_keys {
            let value = key & 0xFFFFFFFF;
            let rid = RID::from_parts((key >> 32) as i32, value as u32);
            tree.insert(key, rid);
        }
        let shared_tree = Arc::new(Mutex::new(tree));

        // Launch threads
        let num_threads = 6;
        let mut handles = Vec::new();

        for thread_itr in 0..num_threads {
            let task = thread_itr % 3;

            if task == 0 {
                // Insert dynamic keys
                let tree = Arc::clone(&shared_tree);
                let dynamic_keys = dynamic_keys.clone();
                handles.push(thread::spawn(move || {
                    insert_helper(&tree, &dynamic_keys);
                }));
            } else if task == 1 {
                // Delete dynamic keys
                let tree = Arc::clone(&shared_tree);
                let dynamic_keys = dynamic_keys.clone();
                handles.push(thread::spawn(move || {
                    delete_helper(&tree, &dynamic_keys);
                }));
            } else {
                // Look up preserved keys
                let tree = Arc::clone(&shared_tree);
                let perserved_keys = perserved_keys.clone();
                handles.push(thread::spawn(move || {
                    lookup_helper(&tree, &perserved_keys);
                }));
            }
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Check preserved keys still exist
        let tree = shared_tree.lock().unwrap();
        let mut size: i64 = 0;
        for (k, _v) in tree.iter() {
            if *k % sieve == 0 {
                size += 1;
            }
        }
        assert_eq!(size, perserved_keys.len() as i64);
    }
}

