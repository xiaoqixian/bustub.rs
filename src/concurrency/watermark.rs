use std::collections::HashMap;

use crate::storage::table::tuple::TimeStamp;

#[derive(Clone, Default, Debug)]
pub struct Watermark {
    commit_ts: TimeStamp,
    watermark: TimeStamp,
    current_reads: HashMap<TimeStamp, i32>,
}

impl Watermark {
    pub fn new(commit_ts: TimeStamp) -> Self {
        Self {
            commit_ts,
            watermark: commit_ts,
            current_reads: HashMap::new(),
        }
    }

    pub fn add_txn(&mut self, read_ts: TimeStamp) {
        let count = self.current_reads.entry(read_ts).or_insert(0);
        *count += 1;
        
        if read_ts < self.watermark {
            self.watermark = read_ts;
        }
    }

    pub fn remove_txn(&mut self, read_ts: TimeStamp) {
        if let Some(count) = self.current_reads.get_mut(&read_ts) {
            *count -= 1;
            if *count == 0 {
                self.current_reads.remove(&read_ts);
            }
        }
    }

    /// The caller should update commit ts before removing the txn from the watermark so that we can track watermark
    /// correctly.
    pub fn update_commit_ts(&mut self, commit_ts: TimeStamp) {
        self.commit_ts = commit_ts;
    }

    /// tracks all the read timestamps.
    pub fn get_watermark(&self) -> TimeStamp {
        if self.current_reads.is_empty() {
            return self.commit_ts;
        }
        self.watermark
    }
}
