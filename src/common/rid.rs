use crate::common::{INVALID_PAGE_ID, PageId};

/// Record identifier.
///
/// A RID consists of:
/// - page id
/// - slot number inside the page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RID {
    page_id: PageId,
    slot_num: u32,
}

impl RID {
    /// Create an invalid RID.
    pub fn new() -> Self {
        Self {
            page_id: INVALID_PAGE_ID,
            slot_num: 0,
        }
    }

    /// Create a RID from page id and slot number.
    pub fn from_parts(page_id: PageId, slot_num: u32) -> Self {
        Self {
            page_id,
            slot_num,
        }
    }

    /// Create a RID from packed i64 value.
    pub fn from_i64(rid: i64) -> Self {
        Self {
            page_id: (rid >> 32) as PageId,
            slot_num: rid as u32,
        }
    }

    /// Convert RID into packed i64 value.
    pub fn get(&self) -> i64 {
        ((self.page_id as i64) << 32) | (self.slot_num as i64)
    }

    /// Get page id.
    pub fn page_id(&self) -> PageId {
        self.page_id
    }

    /// Get slot number.
    pub fn slot_num(&self) -> u32 {
        self.slot_num
    }

    /// Update RID fields.
    pub fn set(&mut self, page_id: PageId, slot_num: u32) {
        self.page_id = page_id;
        self.slot_num = slot_num;
    }
}

impl Default for RID {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "page_id: {} slot_num: {}",
            self.page_id,
            self.slot_num
        )
    }
}
