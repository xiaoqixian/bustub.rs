//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// b_plus_tree.rs
//
// Identification: src/storage/index/b_plus_tree.rs
//
// Copyright (c) 2015-2024, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use crate::{
    buffer::buffer_pool_manager::BufferPoolManager,
    common::{BUSTUB_PAGE_SIZE, PageId},
    storage::page::{
        b_plus_tree_page::{BPlusTreeInternalPage, BPlusTreeLeafPage, BPlusTreePageMutRef, BPlusTreePageRef, PageMetaData},
        page_guard::{ReadPageGuard, WritePageGuard}
    }
};
use std::{cmp::Ord, marker::PhantomData, iter::Iterator};

/// A B+Tree index that stores key-value pairs.
///
/// The tree is backed by the buffer pool manager and stores its pages on disk.
/// Internal pages store routing keys and child page IDs, while leaf pages store
/// the actual key-value pairs and linked-list pointers for range scans.
#[allow(dead_code)]
pub struct BPlusTree<K, V> {
    index_name: String,
    bpm: BufferPoolManager,
    header_page_id: PageId,
    leaf_max_size: usize,
    internal_max_size: usize,
    _kv_marker: PhantomData<(K, V)>,
}

/// An iterator over key-value pairs in a B+Tree.
pub struct Iter<'a, K, V> {
    _kv_marker: PhantomData<(&'a (), K, V)>,
}

/// A mutable iterator over key-value pairs in a B+Tree.
pub struct IterMut<'a, K, V> {
    _kv_marker: PhantomData<(&'a (), K, V)>,
}

impl<K, V> BPlusTree<K, V>
where
    K: Sized + Ord,
    V: Sized
{
    #[allow(dead_code)]
    const LEAF_SLOT_CNT: usize = (BUSTUB_PAGE_SIZE - std::mem::size_of::<BPlusTreeLeafPage<K, V>>()) /
        (std::mem::size_of::<K>() + std::mem::size_of::<V>());
    #[allow(dead_code)]
    const INTERNAL_SLOT_CNT: usize = (BUSTUB_PAGE_SIZE - std::mem::size_of::<BPlusTreeInternalPage<K, PageId>>()) /
        (std::mem::size_of::<K>() + std::mem::size_of::<PageId>());

    /// Creates a new B+Tree.
    pub fn new(
        index_name: String,
        bpm: BufferPoolManager,
        header_page_id: PageId,
        leaf_max_size: usize,
        internal_max_size: usize,
    ) -> Self {
        Self {
            index_name,
            bpm,
            header_page_id,
            leaf_max_size,
            internal_max_size,
            _kv_marker: PhantomData::default(),
        }
    }

    /// Returns `true` if the tree contains no entries.
    pub fn is_empty(&self) -> bool {
        todo!("TODO(P2): Add implementation.")
    }

    /// Looks up a value by key. Returns `None` if the key is not found.
    pub fn get_value(&self, _key: &K) -> Option<&V> {
        todo!("TODO(P2): Add implementation.")
    }

    /// Inserts a key-value pair into the tree. Returns the old value if the
    /// key already existed.
    pub fn insert(&mut self, _key: K, _value: V) -> Option<V> {
        todo!("TODO(P2): Add implementation.")
    }

    /// Removes a key and its associated value from the tree. Returns the
    /// removed value if the key existed.
    pub fn remove(&mut self, _key: &K) -> Option<V> {
        todo!("TODO(P2): Add implementation.")
    }

    /// Returns an iterator over all key-value pairs in the tree.
    pub fn iter(&self) -> Iter<'_, K, V> {
        todo!("TODO(P2): Add implementation.")
    }

    /// Returns a mutable iterator over all key-value pairs in the tree.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        todo!("TODO(P2): Add implementation.")
    }

    /// Returns an iterator starting from the given key.
    pub fn iter_from(&self, _key: &K) -> Iter<'_, K, V> {
        todo!("TODO(P2): Add implementation.")
    }

    /// Returns a mutable iterator starting from the given key.
    pub fn iter_from_mut(&mut self, _key: &K) -> IterMut<'_, K, V> {
        todo!("TODO(P2): Add implementation.")
    }

    pub fn get_root_page_id(&self) -> PageId {
        todo!("TODO(P2): Add implementation.")
    }

    /// Helper function: reinterprets a `ReadPageGuard` as a B+Tree page
    /// reference. Uses the `PageMetaData` at the start of the page data to
    /// determine whether the page is a leaf or internal node.
    #[allow(dead_code)]
    fn read_guard_as_page_ref<'a>(read_guard: &ReadPageGuard<'a>)
        -> BPlusTreePageRef<'a, BPlusTreeInternalPage<K, PageId>, BPlusTreeLeafPage<K, V>>
    {
        let ptr = read_guard.frame.data.as_ptr();
        unsafe {
            let meta = &*(ptr as *const PageMetaData);
            if meta.is_leaf {
                BPlusTreePageRef::Leaf(&*(ptr as *const BPlusTreeLeafPage<K, V>))
            } else {
                BPlusTreePageRef::Internal(&*(ptr as *const BPlusTreeInternalPage<K, PageId>))
            }
        }
    }

    /// Helper function: reinterprets a `WritePageGuard` as a read-only
    /// B+Tree page reference.
    #[allow(dead_code)]
    fn write_guard_as_page_ref<'a>(write_guard: &WritePageGuard<'a>)
        -> BPlusTreePageRef<'a, BPlusTreeInternalPage<K, PageId>, BPlusTreeLeafPage<K, V>>
    {
        let ptr = write_guard.frame.data.as_ptr();
        unsafe {
            let meta = &*(ptr as *const PageMetaData);
            if meta.is_leaf {
                BPlusTreePageRef::Leaf(&*(ptr as *const BPlusTreeLeafPage<K, V>))
            } else {
                BPlusTreePageRef::Internal(&*(ptr as *const BPlusTreeInternalPage<K, PageId>))
            }
        }
    }

    /// Helper function: reinterprets a `WritePageGuard` as a mutable
    /// B+Tree page reference.
    #[allow(dead_code)]
    fn write_guard_as_mut_page_ref<'a>(write_guard: &mut WritePageGuard<'a>)
        -> BPlusTreePageMutRef<'a, BPlusTreeInternalPage<K, PageId>, BPlusTreeLeafPage<K, V>>
    {
        let ptr = write_guard.frame.data.as_mut_ptr();
        unsafe {
            let meta = &*(ptr as *const PageMetaData);
            if meta.is_leaf {
                BPlusTreePageMutRef::Leaf(&mut *(ptr as *mut BPlusTreeLeafPage<K, V>))
            } else {
                BPlusTreePageMutRef::Internal(&mut *(ptr as *mut BPlusTreeInternalPage<K, PageId>))
            }
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Sized + Ord + 'a,
    V: Sized + 'a
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        todo!("TODO(P2): Add implementation.")
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    K: Sized + Ord + 'a,
    V: Sized + 'a
{
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        todo!("TODO(P2): Add implementation.")
    }
}
