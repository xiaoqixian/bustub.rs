//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// b_plus_tree_contention_test.rs
//
// Identification: src/storage/index/tests/b_plus_tree_contention_test.rs
//
// Copyright (c) 2015-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use crate::{
    buffer::buffer_pool_manager::BufferPoolManager,
    common::{LRUK_REPLACER_K, rid::RID},
    storage::{
        disk::disk_manager_memory::DiskManagerMemory,
        index::b_plus_tree::BPlusTree,
    }
};

// ---------------------------------------------------------------------------
// Benchmark helper
// ---------------------------------------------------------------------------

/// Runs a benchmark where `num_threads` threads each insert
/// `keys_per_thread` keys into the tree. If `with_global_mutex` is true,
/// a global mutex serializes all inserts (emulating a non-crabbing
/// implementation).
fn b_plus_tree_lock_benchmark(
    num_threads: usize,
    leaf_node_size: usize,
    with_global_mutex: bool,
) -> bool {
    let disk_manager = Arc::new(DiskManagerMemory::new());
    let bpm = BufferPoolManager::new(64, disk_manager, LRUK_REPLACER_K);

    let page_id = bpm.new_page();
    let tree = BPlusTree::<i64, RID>::new(
        "foo_pk".to_owned(),
        bpm.clone(),
        page_id,
        leaf_node_size,
        10,
    );
    let shared_tree = Arc::new(Mutex::new(tree));

    let keys_per_thread: i64 = 20000 / num_threads as i64;
    let keys_stride: i64 = 100000;
    let global_mutex = Arc::new(Mutex::new(()));

    let mut handles = Vec::new();

    for i in 0..num_threads {
        let tree = Arc::clone(&shared_tree);
        let global_mutex = Arc::clone(&global_mutex);
        let start_key = i as i64 * keys_stride;
        let end_key = start_key + keys_per_thread;

        handles.push(thread::spawn(move || {
            for key in start_key..end_key {
                let value = key & 0xFFFFFFFF;
                let rid = RID::from_parts((key >> 32) as i32, value as u32);

                if with_global_mutex {
                    let _lock = global_mutex.lock().unwrap();
                    tree.lock().unwrap().insert(key, rid);
                } else {
                    tree.lock().unwrap().insert(key, rid);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    true
}

// ---------------------------------------------------------------------------
// Contention benchmark tests
// ---------------------------------------------------------------------------

/// Benchmark with leaf_node_size = 2, comparing normal vs serialized access.
#[test]
fn contention_benchmark() {
    println!("This test will see how your B+ tree performance differs with and without contention.");
    println!("If your submission timeout, segfault, or didn't implement lock crabbing, we will manually deduct all concurrent test points (maximum 25).");
    println!("leaf_node_size = 2");

    let mut time_ms_with_mutex: Vec<u128> = Vec::new();
    let mut time_ms_wo_mutex: Vec<u128> = Vec::new();

    for iter in 0..20 {
        let enable_mutex = iter % 2 == 0;
        let clock_start = Instant::now();
        assert!(b_plus_tree_lock_benchmark(32, 2, enable_mutex));
        let dur = clock_start.elapsed().as_millis();
        if enable_mutex {
            time_ms_with_mutex.push(dur);
        } else {
            time_ms_wo_mutex.push(dur);
        }
    }

    println!("<<< BEGIN");
    print!("Normal Access Time: ");
    let mut ratio_1: f64 = 0.0;
    for &x in &time_ms_wo_mutex {
        print!("{} ", x);
        ratio_1 += x as f64;
    }
    println!();

    print!("Serialized Access Time: ");
    let mut ratio_2: f64 = 0.0;
    for &x in &time_ms_with_mutex {
        print!("{} ", x);
        ratio_2 += x as f64;
    }
    println!();

    if ratio_2 > 0.0 {
        println!("Ratio: {}", ratio_1 / ratio_2);
    }
    println!(">>> END");
    println!("If your above data is an outlier in all submissions (based on statistics and probably some machine-learning), TAs will manually inspect your code to ensure you are implementing lock crabbing correctly.");
}

/// Benchmark with leaf_node_size = 10, comparing normal vs serialized access.
#[test]
fn contention_benchmark_2() {
    println!("This test will see how your B+ tree performance differs with and without contention.");
    println!("If your submission timeout, segfault, or didn't implement lock crabbing, we will manually deduct all concurrent test points (maximum 25).");
    println!("leaf_node_size = 10");

    let mut time_ms_with_mutex: Vec<u128> = Vec::new();
    let mut time_ms_wo_mutex: Vec<u128> = Vec::new();

    for iter in 0..20 {
        let enable_mutex = iter % 2 == 0;
        let clock_start = Instant::now();
        assert!(b_plus_tree_lock_benchmark(32, 10, enable_mutex));
        let dur = clock_start.elapsed().as_millis();
        if enable_mutex {
            time_ms_with_mutex.push(dur);
        } else {
            time_ms_wo_mutex.push(dur);
        }
    }

    println!("<<< BEGIN2");
    print!("Normal Access Time: ");
    let mut ratio_1: f64 = 0.0;
    for &x in &time_ms_wo_mutex {
        print!("{} ", x);
        ratio_1 += x as f64;
    }
    println!();

    print!("Serialized Access Time: ");
    let mut ratio_2: f64 = 0.0;
    for &x in &time_ms_with_mutex {
        print!("{} ", x);
        ratio_2 += x as f64;
    }
    println!();

    if ratio_2 > 0.0 {
        println!("Ratio: {}", ratio_1 / ratio_2);
    }
    println!(">>> END2");
    println!("If your above data is an outlier in all submissions (based on statistics and probably some machine-learning), TAs will manually inspect your code to ensure you are implementing lock crabbing correctly.");
}

