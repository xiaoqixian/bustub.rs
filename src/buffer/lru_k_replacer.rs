// Date:   Sun May 17 15:34:04 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
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

use std::collections::HashMap;
use std::sync::Mutex;

use crate::common::FrameId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Unknown = 0,
    Lookup,
    Scan,
    Index,
}

#[allow(dead_code)]
pub struct LRUKNode {
    /// History of last seen K timestamps of this page.
    /// Least recent timestamp stored in front.
    /// Remove maybe_unused if you start using them.
    /// Feel free to change the member variables as you want.
    history_: Vec<usize>,
    k_: usize,
    fid_: FrameId,
    is_evictable_: bool,
}

impl LRUKNode {
    pub fn new(k: usize, fid: FrameId) -> Self {
        Self {
            history_: Vec::new(),
            k_: k,
            fid_: fid,
            is_evictable_: false,
        }
    }
}

/// LRUKReplacer implements the LRU-k replacement policy.
///
/// The LRU-k algorithm evicts a frame whose backward k-distance is maximum
/// of all frames. Backward k-distance is computed as the difference in time between
/// current timestamp and the timestamp of kth previous access.
///
/// A frame with less than k historical references is given
/// +inf as its backward k-distance. When multiple frames have +inf backward k-distance,
/// classical LRU algorithm is used to choose victim.
#[allow(dead_code)]
pub struct LRUKReplacer {
    /// TODO(student): implement me! You can replace these member variables as you like.
    /// Remove maybe_unused if you start using them.
    node_store_: HashMap<FrameId, LRUKNode>,
    current_timestamp_: usize,
    curr_size_: usize,
    replacer_size_: usize,
    k_: usize,
    latch_: Mutex<()>,
}

impl LRUKReplacer {
    /// TODO(P1): Add implementation
    ///
    /// Create a new LRUKReplacer.
    ///
    /// * `num_frames` - the maximum number of frames the LRUReplacer will be required to store
    pub fn new(num_frames: usize, k: usize) -> Self {
        Self {
            node_store_: HashMap::new(),
            current_timestamp_: 0,
            curr_size_: 0,
            replacer_size_: num_frames,
            k_: k,
            latch_: Mutex::new(()),
        }
    }

    /// TODO(P1): Add implementation
    ///
    /// Find the frame with largest backward k-distance and evict that frame. Only frames
    /// that are marked as 'evictable' are candidates for eviction.
    ///
    /// A frame with less than k historical references is given +inf as its backward k-distance.
    /// If multiple frames have inf backward k-distance, then evict frame with earliest timestamp
    /// based on LRU.
    ///
    /// Successful eviction of a frame should decrement the size of replacer and remove the frame's
    /// access history.
    ///
    /// Returns `Some(frame_id)` if a frame is evicted successfully, `None` if no frames can be evicted.
    pub fn evict(&mut self) -> Option<FrameId> {
        todo!("TODO(P1): Add implementation")
    }

    /// TODO(P1): Add implementation
    ///
    /// Record the event that the given frame id is accessed at current timestamp.
    /// Create a new entry for access history if frame id has not been seen before.
    ///
    /// If frame id is invalid (ie. larger than replacer_size_), abort the process.
    ///
    /// * `frame_id` - id of frame that received a new access.
    /// * `access_type` - type of access that was received. This parameter is only needed for
    ///   leaderboard tests.
    pub fn record_access(&mut self, frame_id: FrameId, access_type: AccessType) {
        let _ = (frame_id, access_type);
        todo!("TODO(P1): Add implementation")
    }

    /// TODO(P1): Add implementation
    ///
    /// Toggle whether a frame is evictable or non-evictable. This function also
    /// controls replacer's size. Note that size is equal to number of evictable entries.
    ///
    /// If a frame was previously evictable and is to be set to non-evictable, then size should
    /// decrement. If a frame was previously non-evictable and is to be set to evictable,
    /// then size should increment.
    ///
    /// If frame id is invalid, abort the process.
    ///
    /// For other scenarios, this function should terminate without modifying anything.
    ///
    /// * `frame_id` - id of frame whose 'evictable' status will be modified
    /// * `set_evictable` - whether the given frame is evictable or not
    pub fn set_evictable(&mut self, frame_id: FrameId, set_evictable: bool) {
        let _ = (frame_id, set_evictable);
        todo!("TODO(P1): Add implementation")
    }

    /// TODO(P1): Add implementation
    ///
    /// Remove an evictable frame from replacer, along with its access history.
    /// This function should also decrement replacer's size if removal is successful.
    ///
    /// Note that this is different from evicting a frame, which always remove the frame
    /// with largest backward k-distance. This function removes specified frame id,
    /// no matter what its backward k-distance is.
    ///
    /// If Remove is called on a non-evictable frame, abort the process.
    ///
    /// If specified frame is not found, directly return from this function.
    ///
    /// * `frame_id` - id of frame to be removed
    pub fn remove(&mut self, frame_id: FrameId) {
        let _ = frame_id;
        todo!("TODO(P1): Add implementation")
    }

    /// TODO(P1): Add implementation
    ///
    /// Return replacer's size, which tracks the number of evictable frames.
    pub fn size(&self) -> usize {
        todo!("TODO(P1): Add implementation")
    }
}


#[cfg(test)]
mod lru_k {
    use super::*;

    #[test]
    fn sample_test() {
        // Note that comparison with None always results in checking whether
        // the optional type actually contains a value.

        // Initialize the replacer.
        let mut lru_replacer = LRUKReplacer::new(7, 2);

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


