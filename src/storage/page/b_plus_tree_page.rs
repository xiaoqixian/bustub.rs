// Date:   Mon May 18 10:41:45 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian

use crate::common::{INVALID_PAGE_ID, PageId};

#[repr(C)]
pub(crate) struct PageMetaData {
    pub(crate) is_leaf: bool
}

pub enum BPlusTreePageRef<'a, I, L> {
    Internal(&'a I),
    Leaf(&'a L)
}
pub enum BPlusTreePageMutRef<'a, I, L> {
    Internal(&'a mut I),
    Leaf(&'a mut L)
}

#[allow(dead_code)]
pub struct BPlusTreeInternalPage<K, V> {
    meta: PageMetaData,
    pub(crate) size: usize,
    data: [(K, V); 0]
}

#[allow(dead_code)]
pub struct BPlusTreeLeafPage<K, V> {
    meta: PageMetaData,
    pub(crate) size: usize,
    pub(crate) next_page_id: PageId,
    data: [(K, V); 0],
}

impl<K, V> BPlusTreeLeafPage<K, V> {
    pub fn init(&mut self) {
        self.meta.is_leaf = true;
        self.size = 0;
        self.next_page_id = INVALID_PAGE_ID;
    }

    pub fn key_at(&self, index: usize) -> Option<&K> {
        if index >= self.size {
            return None;
        }
        unsafe {
            Some(&self.data.get_unchecked(index).0)
        }
    }
}

impl<K, V> BPlusTreeInternalPage<K, V> {
    pub fn init(&mut self) {
        self.meta.is_leaf = false;
        self.size = 0;
    }

    pub fn key_at(&self, index: usize) -> Option<&K> {
        if index >= self.size {
            return None;
        }
        unsafe {
            Some(&self.data.get_unchecked(index).0)
        }
    }
}
