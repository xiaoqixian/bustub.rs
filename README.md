# cmu-15445-rustlab

A [Rust](https://www.rust-lang.org/) implementation of **BusTub** — the disk-oriented database management system from [CMU 15-445/645 (Intro to Database Systems, Fall 2024)](https://15445.courses.cs.cmu.edu/fall2024/). The goal is to build a simple SQL kernel incrementally through the course's programming projects, using Rust's ownership model and concurrency primitives in place of C++.

> **Status:** 🚧 Under active development. Not yet complete.

---

## Project #A — Buffer Pool Manager (Summary)

This section summarizes what [CMU 15-445 Project #1](https://15445.courses.cs.cmu.edu/fall2024/project1) requires students to build. It is the foundation of the DBMS's storage layer.

### Key Concepts

- **Page** — 4096 bytes (4 KB) of logical data. Can reside in memory, on disk, or both.
- **Frame** — A fixed-size 4 KB block of memory that stores exactly one page. The buffer pool manages a fixed number of frames.
- **Buffer Pool Manager (BPM)** — The cache layer that moves pages between disk and main memory, allowing the DBMS to work with databases larger than available RAM.
- **Thread safety** — All components must be safe under concurrent access using latches.

### Task #1 — LRU-K Replacement Policy

The `LRUKReplacer` tracks page access history and decides which frame to evict when the buffer pool is full.

- **Eviction rule:** Evict the frame with the largest backward _k-distance_ (the time difference between now and the k-th most recent access).
- Frames with fewer than _k_ accesses get an infinite backward k-distance, breaking ties via LRU.
- Only frames marked as **evictable** count toward the replacer's size.
- Must be thread-safe.

**Key methods:** `evict()`, `record_access(frame_id)`, `set_evictable(frame_id, bool)`, `remove(frame_id)`, `size()`.

### Task #2 — Disk Scheduler

The `DiskScheduler` maintains a background worker thread that processes read/write requests from a shared channel. Each request includes a promise/future callback so the caller can block until the I/O completes.

- The background thread dequeues `DiskRequest` objects and dispatches them to the `DiskManager`.
- On completion, it signals the request's callback to notify the caller.
- Must be thread-safe.

### Task #3 — Buffer Pool Manager

The main `BufferPoolManager` ties everything together:

- Uses the `LRUKReplacer` to choose eviction victims.
- Uses the `DiskScheduler` to perform disk I/O.
- Maintains a **page table** (HashMap) mapping page IDs to frames.
- Tracks **pin counts** on each frame — a page cannot be evicted while pinned.
- Provides RAII **page guards** (`ReadPageGuard` / `WritePageGuard`) that give threads safe access to page data and automatically release pins/latches on drop.

**Key methods:** `new_page()`, `delete_page()`, `checked_read_page()`, `checked_write_page()`, `flush_page()`, `flush_all_pages()`, `get_pin_count()`.

### Leaderboard (Optional)

An optional performance challenge benchmarks the buffer pool under concurrent scan and zipfian-distributed random access workloads with simulated disk latency. Optimizations include smarter replacement policies, parallel I/O, and lock-free inter-thread communication.

---

## Building & Testing

```bash
# Build
cargo build

# Run tests
cargo test --bin cmu-15445-rustlab

# Run a specific test (e.g., disk scheduler)
cargo test --bin cmu-15445-rustlab disk_scheduler
```

## References

- [CMU 15-445/645 Fall 2024 Course Page](https://15445.courses.cs.cmu.edu/fall2024/)
- [Project #1 Specification](https://15445.courses.cs.cmu.edu/fall2024/project1)
- [BusTub (C++ Reference Implementation)](https://github.com/cmu-db/bustub)
- [The LRU-K Page Replacement Algorithm (O'Neil et al.)](https://www.cs.cmu.edu/~natassa/courses/15-721/papers/p297-o_neil.pdf)

