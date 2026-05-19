// Date:   Mon May 18 16:25:53 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian

use crate::{
    buffer::buffer_pool_manager::BufferPoolManager, 
    common::{BUSTUB_PAGE_SIZE, PageId}, 
    storage::page::{
        b_plus_tree_page::{BPlusTreeInternalPage, BPlusTreeLeafPage, BPlusTreePageMutRef, BPlusTreePageRef, PageMetaData}, 
        page_guard::{ReadPageGuard, WritePageGuard}
    }
};
use std::{cmp::Ord, marker::PhantomData, iter::Iterator};

#[allow(dead_code)]
pub struct BPlusTree<K, V> {
    index_name: String,
    bpm: BufferPoolManager,
    header_page_id: PageId,
    leaf_max_size: usize,
    internal_max_size: usize,
    _kv_marker: PhantomData<(K, V)>,
}

pub struct Iter<'a, K, V> {
    _kv_marker: PhantomData<(&'a (), K, V)>,
}
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

    pub fn is_empty(&self) -> bool {
        todo!("");
    }

    pub fn get_value(&self, _key: &K) -> Option<&V> {
        todo!("");
    }

    pub fn insert(&mut self, _key: K, _value: V) -> Option<V> {
        todo!("");
    }

    pub fn remove(&mut self, _key: &K) -> Option<V> {
        todo!("");
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        todo!("");
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        todo!("");
    }

    pub fn iter_from(&self, _key: &K) -> Iter<'_, K, V> {
        todo!("");
    }

    pub fn iter_from_mut(&mut self, _key: &K) -> IterMut<'_, K, V> {
        todo!("");
    }

    /// helper functions
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
        todo!("");
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> 
where
    K: Sized + Ord + 'a,
    V: Sized + 'a
{
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        todo!("");
    }
}
