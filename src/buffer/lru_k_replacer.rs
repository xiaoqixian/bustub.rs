//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// lru_k_replacer.rs
//
// Identification: src/buffer/lru_k_replacer.rs
//
// Copyright (c) 2015-2022, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::common::FrameId;

/// The type of access that was made to a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Unknown = 0,
    Lookup,
    Scan,
    Index,
}

struct RingBuffer<T> {
    data: Vec<T>,
    size: usize,
    head: usize,
    tail: usize,
}

impl<T> RingBuffer<T> {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
            size: 0,
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, val: T) {
        let v = &mut self.data;
        if v.len() < v.capacity() {
            v.push(val);
            self.tail = v.len();
            self.size = v.len();
            if self.tail == v.capacity() {
                self.tail = 0;
            }
            return;
        }

        v[self.tail] = val;
        self.tail += 1;
        if self.tail == v.capacity() { self.tail = 0; }
        if self.size == v.capacity() {
            self.head += 1;
            if self.head == v.capacity() {
                self.head = 0;
            }
        } else {
            self.size += 1;
        }
    }

    pub fn front(&self) -> &T {
        assert!(self.size > 0);
        &self.data[self.head]
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn full(&self) -> bool {
        self.size == self.data.capacity()
    }
}

/// LRUKNode stores the access history and eviction metadata for a single
/// frame tracked by the LRU-K replacer.
#[allow(dead_code)]
pub struct LRUKNode {
    /// History of last seen K timestamps of this frame. Least recent
    /// timestamp is stored in front (oldest first).
    history: RingBuffer<usize>,

    /// The K value for this node (the backward k-distance threshold).
    k: usize,

    /// The frame ID this node represents.
    fid: FrameId,

    /// Whether this frame is currently eligible for eviction.
    evictable: bool,
}

impl LRUKNode {
    /// Creates a new `LRUKNode` for the given frame.
    ///
    /// The node starts with an empty access history and is not evictable
    /// by default.
    pub fn new(k: usize, fid: FrameId, first_ts: usize) -> Self {
        let mut history = RingBuffer::with_capacity(k);
        history.push(first_ts);
        Self {
            history,
            k,
            fid,
            evictable: true,
        }
    }

    pub fn access_at(&mut self, ts: usize) {
        self.history.push(ts);
    }
}

impl PartialEq for LRUKNode {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for LRUKNode {}

impl PartialOrd for LRUKNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.history.full(), other.history.full()) {
            (false, false) => match (self.history.size(), other.history.size()) {
                (0, _) => Some(Ordering::Greater),
                (_, 0) => Some(Ordering::Less),
                _ => Some(other.history.front().cmp(self.history.front())),
            },
            (false, true) => Some(Ordering::Greater),
            (true, false) => Some(Ordering::Less),
            (true, true) => Some(other.history.front().cmp(self.history.front()))
        }
    }
}

impl Ord for LRUKNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// LRUKReplacer implements the LRU-k replacement policy.
///
/// The LRU-k algorithm evicts a frame whose backward k-distance is maximum
/// of all frames. Backward k-distance is computed as the difference in time
/// between the current timestamp and the timestamp of the kth previous
/// access.
///
/// A frame with less than k historical references is given +inf as its
/// backward k-distance. When multiple frames have +inf backward k-distance,
/// the classical LRU algorithm is used to choose the victim.
#[allow(dead_code)]
struct LRUKReplacerCore {
    /// Maps frame IDs to their corresponding LRU-K nodes.
    node_store: HashMap<FrameId, LRUKNode>,

    /// The current logical timestamp, incremented on each access.
    current_timestamp: usize,

    /// The current number of evictable frames tracked by the replacer.
    curr_size: usize,

    /// The maximum number of frames the replacer can track.
    replacer_size: usize,

    /// The backward k-distance threshold.
    k: usize,
}

/// A thread-safe, shareable handle to the LRU-K replacer.
///
/// Wraps the `LRUKReplacerCore` in `Arc<Mutex<...>>` so that it can be
/// shared between the buffer pool manager and page guards.
#[allow(dead_code)]
#[derive(Clone)]
pub struct LRUKReplacer {
    core: Arc<Mutex<LRUKReplacerCore>>,
}

impl LRUKReplacerCore {
    /// Creates a new LRU-K replacer.
    ///
    /// * `num_frames` - the maximum number of frames the replacer will be
    ///   required to store.
    /// * `k` - the backward k-distance threshold.
    ///
    /// TODO(P1): Add implementation.
    fn new(num_frames: usize, k: usize) -> Self {
        Self {
            node_store: HashMap::new(),
            current_timestamp: 0,
            curr_size: 0,
            replacer_size: num_frames,
            k,
        }
    }

    /// Finds the frame with the largest backward k-distance and evicts it.
    /// Only frames that are marked as 'evictable' are candidates for
    /// eviction.
    ///
    /// A frame with less than k historical references is given +inf as its
    /// backward k-distance. If multiple frames have infinite backward
    /// k-distance, then the frame with the earliest timestamp (based on LRU)
    /// is evicted.
    ///
    /// Successful eviction decrements the replacer's size and removes the
    /// frame's access history.
    ///
    /// Returns `Some(frame_id)` if a frame was evicted successfully, or
    /// `None` if no frames can be evicted.
    ///
    /// TODO(P1): Add implementation.
    fn evict(&mut self) -> Option<FrameId> {
        let evict_fid = self.node_store.values()
            .filter(|node| node.evictable)
            .max()
            .map(|node| node.fid);
        if let Some(fid) = &evict_fid {
            self.node_store.remove(fid);
            self.curr_size -= 1;
        }
        evict_fid
    }

    /// Records an access event for the given frame at the current timestamp.
    /// Creates a new entry in the access history if the frame has not been
    /// seen before.
    ///
    /// If the frame ID is invalid (i.e., larger than `replacer_size`), the
    /// process is aborted via an assertion.
    ///
    /// * `frame_id` - the ID of the frame that received a new access.
    /// * `access_type` - the type of access that was received. This
    ///   parameter is only needed for leaderboard tests.
    ///
    /// TODO(P1): Add implementation.
    fn record_access(&mut self, frame_id: FrameId, _access_type: AccessType) {
        let ts = self.current_timestamp;
        self.current_timestamp += 1;
        if let Some(node) = self.node_store.get_mut(&frame_id) {
            node.access_at(ts);
            return;
        }
        
        if self.node_store.len() == self.replacer_size {
            let evict_fid = self.evict().expect("failed to record access");
            self.node_store.remove(&evict_fid);
        }

        self.node_store.insert(frame_id, LRUKNode::new(self.k, frame_id, ts));
        self.curr_size += 1;
    }

    /// Toggles whether a frame is evictable or non-evictable. This function
    /// also controls the replacer's size, where size equals the number of
    /// evictable entries.
    ///
    /// - If a frame was previously evictable and is being set to
    ///   non-evictable, the size is decremented.
    /// - If a frame was previously non-evictable and is being set to
    ///   evictable, the size is incremented.
    ///
    /// If the frame ID is invalid, the process is aborted via an assertion.
    /// For all other unexpected scenarios, this function terminates without
    /// modifying anything.
    ///
    /// * `frame_id` - the ID of the frame whose evictable status will be
    ///   modified.
    /// * `evictable` - whether the given frame should be evictable or
    ///   not.
    ///
    /// TODO(P1): Add implementation.
    fn set_evictable(&mut self, frame_id: FrameId, evictable: bool) {
        if let Some(node) = self.node_store.get_mut(&frame_id) {
            let old_ev = std::mem::replace(&mut node.evictable, evictable);
            match (old_ev, evictable) {
                (true, false) => self.curr_size -= 1,
                (false, true) => self.curr_size += 1,
                _ => {}
            }
        }
    }

    /// Removes an evictable frame from the replacer, along with its access
    /// history. Also decrements the replacer's size if the removal was
    /// successful.
    ///
    /// Note: This is different from evicting a frame, which always removes
    /// the frame with the largest backward k-distance. This function removes
    /// the specified frame regardless of its backward k-distance.
    ///
    /// If `remove()` is called on a non-evictable frame, the process is
    /// aborted via an assertion. If the specified frame is not found, this
    /// function returns without modifying anything.
    ///
    /// * `frame_id` - the ID of the frame to be removed.
    ///
    /// TODO(P1): Add implementation.
    fn remove(&mut self, frame_id: FrameId) {
        if let Some(node) = self.node_store.remove(&frame_id) {
            assert!(node.evictable, "try to remove an unevictable frame");
        }
        assert!(self.curr_size > 0);
        self.curr_size -= 1;
    }

    /// Returns the replacer's size, which tracks the number of evictable
    /// frames.
    ///
    /// TODO(P1): Add implementation.
    fn size(&self) -> usize {
        self.curr_size
    }
}

impl LRUKReplacer {
    pub fn new(num_frames: usize, k: usize) -> Self {
        Self {
            core: Arc::new(Mutex::new(LRUKReplacerCore::new(num_frames, k)))
        }
    }

    /// Finds the frame with the largest backward k-distance and evicts it.
    /// Only frames that are marked as 'evictable' are candidates for
    /// eviction.
    ///
    /// A frame with less than k historical references is given +inf as its
    /// backward k-distance. If multiple frames have infinite backward
    /// k-distance, then the frame with the earliest timestamp (based on LRU)
    /// is evicted.
    ///
    /// Successful eviction decrements the replacer's size and removes the
    /// frame's access history.
    ///
    /// Returns `Some(frame_id)` if a frame was evicted successfully, or
    /// `None` if no frames can be evicted.
    ///
    /// TODO(P1): Add implementation.
    pub fn evict(&self) -> Option<FrameId> {
        let mut core = self.core.lock().unwrap();
        core.evict()
    }

    pub fn record_access(&self, frame_id: FrameId, access_type: AccessType) {
        let mut core = self.core.lock().unwrap();
        core.record_access(frame_id, access_type)
    }

    pub fn set_evictable(&self, frame_id: FrameId, set_evictable: bool) {
        let mut core = self.core.lock().unwrap();
        core.set_evictable(frame_id, set_evictable)
    }

    pub fn remove(&self, frame_id: FrameId) {
        let mut core = self.core.lock().unwrap();
        core.remove(frame_id)
    }

    #[allow(dead_code)]
    fn size(&self) -> usize {
        let core = self.core.lock().unwrap();
        core.size()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod lru_k {
    use super::*;

    #[test]
    fn sample_test() {
        // Note that comparison with None always results in checking whether
        // the optional type actually contains a value.

        // Initialize the replacer.
        let lru_replacer = LRUKReplacer::new(7, 2);

        // Add six frames to the replacer. We now have frames [1, 2, 3, 4, 5].
        // We set frame 6 as non-evictable.
        lru_replacer.record_access(1, AccessType::Unknown);
        lru_replacer.record_access(2, AccessType::Unknown);
        lru_replacer.record_access(3, AccessType::Unknown);
        lru_replacer.record_access(4, AccessType::Unknown);
        lru_replacer.record_access(5, AccessType::Unknown);
        lru_replacer.record_access(6, AccessType::Unknown);
        lru_replacer.set_evictable(1, true);
        lru_replacer.set_evictable(2, true);
        lru_replacer.set_evictable(3, true);
        lru_replacer.set_evictable(4, true);
        lru_replacer.set_evictable(5, true);
        lru_replacer.set_evictable(6, false);

        // The size of the replacer is the number of frames that can be evicted,
        // _not_ the total number of frames entered.
        assert_eq!(5, lru_replacer.size());

        // Record an access for frame 1. Now frame 1 has two accesses total.
        lru_replacer.record_access(1, AccessType::Unknown);
        // All other frames now share the maximum backward k-distance. Since we use
        // timestamps to break ties, where the first to be evicted is the frame with
        // the oldest timestamp, the order of eviction should be [2, 3, 4, 5, 1].

        // Evict three pages from the replacer.
        // To break ties, we use LRU with respect to the oldest timestamp,
        // or the least recently used frame.
        assert_eq!(Some(2), lru_replacer.evict());
        assert_eq!(Some(3), lru_replacer.evict());
        assert_eq!(Some(4), lru_replacer.evict());
        assert_eq!(2, lru_replacer.size());
        // Now the replacer has the frames [5, 1].

        // Insert new frames [3, 4], and update the access history for 5.
        // Now, the ordering is [3, 1, 5, 4].
        lru_replacer.record_access(3, AccessType::Unknown);
        lru_replacer.record_access(4, AccessType::Unknown);
        lru_replacer.record_access(5, AccessType::Unknown);
        lru_replacer.record_access(4, AccessType::Unknown);
        lru_replacer.set_evictable(3, true);
        lru_replacer.set_evictable(4, true);
        assert_eq!(4, lru_replacer.size());

        // Look for a frame to evict. We expect frame 3 to be evicted next.
        assert_eq!(Some(3), lru_replacer.evict());
        assert_eq!(3, lru_replacer.size());

        // Set 6 to be evictable. 6 Should be evicted next since it has the
        // maximum backward k-distance.
        lru_replacer.set_evictable(6, true);
        assert_eq!(4, lru_replacer.size());
        assert_eq!(Some(6), lru_replacer.evict());
        assert_eq!(3, lru_replacer.size());

        // Mark frame 1 as non-evictable. We now have [5, 4].
        lru_replacer.set_evictable(1, false);

        // We expect frame 5 to be evicted next.
        assert_eq!(2, lru_replacer.size());
        assert_eq!(Some(5), lru_replacer.evict());
        assert_eq!(1, lru_replacer.size());

        // Update the access history for frame 1 and make it evictable.
        // Now we have [4, 1].
        lru_replacer.record_access(1, AccessType::Unknown);
        lru_replacer.record_access(1, AccessType::Unknown);
        lru_replacer.set_evictable(1, true);
        assert_eq!(2, lru_replacer.size());

        // Evict the last two frames.
        assert_eq!(Some(4), lru_replacer.evict());
        assert_eq!(1, lru_replacer.size());
        assert_eq!(Some(1), lru_replacer.evict());
        assert_eq!(0, lru_replacer.size());

        // Insert frame 1 again and mark it as non-evictable.
        lru_replacer.record_access(1, AccessType::Unknown);
        lru_replacer.set_evictable(1, false);
        assert_eq!(0, lru_replacer.size());

        // A failed eviction should not change the size of the replacer.
        let frame = lru_replacer.evict();
        assert_eq!(false, frame.is_some());

        // Mark frame 1 as evictable again and evict it.
        lru_replacer.set_evictable(1, true);
        assert_eq!(1, lru_replacer.size());
        assert_eq!(Some(1), lru_replacer.evict());
        assert_eq!(0, lru_replacer.size());

        // There is nothing left in the replacer, so make sure this doesn't
        // do something strange.
        let frame = lru_replacer.evict();
        assert_eq!(false, frame.is_some());
        assert_eq!(0, lru_replacer.size());

        // Make sure that setting a non-existent frame as evictable or
        // non-evictable doesn't do something strange.
        lru_replacer.set_evictable(6, false);
        lru_replacer.set_evictable(6, true);
    }
}

