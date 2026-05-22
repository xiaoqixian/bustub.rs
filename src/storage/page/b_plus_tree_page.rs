//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// b_plus_tree_page.rs
//
// Identification: src/storage/page/b_plus_tree_page.rs
//
// Copyright (c) 2015-2024, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use crate::common::{INVALID_PAGE_ID, PageId};

/// Metadata stored at the beginning of every B+Tree page, used to distinguish
/// between leaf and internal pages at runtime.
#[repr(C)]
pub(crate) struct PageMetaData {
    /// `true` if this is a leaf page, `false` if internal.
    pub(crate) is_leaf: bool
}

/// An immutable reference to either a leaf or an internal B+Tree page.
pub enum BPlusTreePageRef<'a, I, L> {
    Internal(&'a I),
    Leaf(&'a L)
}

/// A mutable reference to either a leaf or an internal B+Tree page.
pub enum BPlusTreePageMutRef<'a, I, L> {
    Internal(&'a mut I),
    Leaf(&'a mut L)
}

/// A B+Tree internal page stored inside a page frame.
///
/// Internal pages store routing keys and child page IDs. The actual
/// key-value pairs are stored at the end of the struct using a
/// zero-length array (`[(K, V); 0]`) so that the page data can be
/// interpreted directly from the raw frame bytes.
#[allow(dead_code)]
pub struct BPlusTreeInternalPage<K, V> {
    meta: PageMetaData,
    /// The number of key-value pairs currently stored in this page.
    pub(crate) size: usize,
    data: [(K, V); 0]
}

/// A B+Tree leaf page stored inside a page frame.
///
/// Leaf pages store key-value pairs and a pointer to the next leaf
/// page for range scans. The actual key-value pairs are stored at the
/// end of the struct using a zero-length array (`[(K, V); 0]`).
#[allow(dead_code)]
pub struct BPlusTreeLeafPage<K, V> {
    meta: PageMetaData,
    /// The number of key-value pairs currently stored in this page.
    pub(crate) size: usize,
    /// The page ID of the next leaf page, or `INVALID_PAGE_ID` if this
    /// is the last leaf.
    pub(crate) next_page_id: PageId,
    data: [(K, V); 0],
}

impl<K, V> BPlusTreeLeafPage<K, V> {
    /// Initializes a new leaf page with size 0 and no next page.
    pub fn init(&mut self) {
        self.meta.is_leaf = true;
        self.size = 0;
        self.next_page_id = INVALID_PAGE_ID;
    }

    /// Returns a reference to the key at the given index, or `None` if
    /// the index is out of bounds.
    pub fn key_at(&self, index: usize) -> Option<&K> {
        if index >= self.size {
            return None;
        }
        unsafe {
            Some(&self.data.get_unchecked(index).0)
        }
    }

    /// Returns a reference to the value at the given index, or `None` if
    /// the index is out of bounds.
    pub fn value_at(&self, index: usize) -> Option<&V> {
        if index >= self.size {
            return None;
        }
        unsafe {
            Some(&self.data.get_unchecked(index).1)
        }
    }

    /// Returns a mut reference to the value at the given index, or `None` if
    /// the index is out of bounds.
    pub fn mut_value_at(&mut self, index: usize) -> Option<&mut V> {
        if index >= self.size {
            return None;
        }
        unsafe {
            Some(&mut self.data.get_unchecked_mut(index).1)
        }
    }

    /// Returns a reference to the entry at the given index, or `None` if
    /// the index is out of bounds.
    pub fn entry_at(&self, index: usize) -> Option<(&K, &V)> {
        if index >= self.size {
            return None;
        }
        unsafe {
            let (k, v) = self.data.get_unchecked(index);
            Some((k, v))
        }
    }

    /// Returns a mut reference to the entry at the given index, or `None` if
    /// the index is out of bounds.
    pub fn mut_entry_at(&mut self, index: usize) -> Option<(&K, &mut V)> {
        if index >= self.size {
            return None;
        }
        unsafe {
            let (k, v) = self.data.get_unchecked_mut(index);
            Some((&*k, v))
        }
    }
}

impl<K, V> BPlusTreeInternalPage<K, V> {
    /// Initializes a new internal page with size 0.
    pub fn init(&mut self) {
        self.meta.is_leaf = false;
        self.size = 0;
    }

    /// Returns a reference to the key at the given index, or `None` if
    /// the index is out of bounds.
    pub fn key_at(&self, index: usize) -> Option<&K> {
        if index >= self.size {
            return None;
        }
        unsafe {
            Some(&self.data.get_unchecked(index).0)
        }
    }
}
