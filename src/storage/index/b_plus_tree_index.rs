//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// b_plus_tree_index.rs
//
// Identification: src/storage/index/b_plus_tree_index.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

use crate::buffer::buffer_pool_manager::BufferPoolManager;
use crate::common::BUSTUB_PAGE_SIZE;
use crate::common::rid::RID;
use crate::concurrency::Transaction;
use crate::storage::index::b_plus_tree::BPlusTree;
use crate::storage::index::generic_key::{GenericKey, gen_generic_key_cmp_with_schema};
use crate::storage::index::index::{Index, IndexMetadata};
use crate::storage::page::b_plus_tree_page::{BPlusTreeInternalPage, BPlusTreeLeafPage};
use crate::storage::table::tuple::Tuple;

/**
 * BPlusTreeIndex - A B+Tree based index that implements the `Index` trait.
 *
 * The index stores key-value pairs where the key is a byte-encoded
 * representation of the indexed columns (encoded to preserve sort order)
 * and the value is the RID of the corresponding tuple.
 */
pub struct BPlusTreeIndex<K, V, C> {
    /// Index metadata (name, schema, key attrs, etc.).
    metadata: IndexMetadata,
    /// The underlying B+Tree container, protected by a mutex for interior mutability.
    container: Mutex<BPlusTree<K, V, C>>,
}

type GK<const N: usize> = GenericKey<N>;

pub fn new_gk_b_plus_tree_index<const N: usize, V>(metadata: IndexMetadata, bpm: Arc<BufferPoolManager>)
    -> BPlusTreeIndex<GenericKey<N>, V, impl Fn(&GenericKey<N>, &GenericKey<N>) -> Ordering> 
{
    // Default leaf node max size.
    let page_leaf_max_size: usize = (BUSTUB_PAGE_SIZE - std::mem::size_of::<BPlusTreeLeafPage<(), ()>>()) /
        (std::mem::size_of::<GK<N>>() + std::mem::size_of::<V>());
    // Default internal node max size.
    let page_interal_max_size: usize = (BUSTUB_PAGE_SIZE - std::mem::size_of::<BPlusTreeInternalPage<(), ()>>()) /
        (std::mem::size_of::<GK<N>>() + std::mem::size_of::<V>());

    let header_page_id = bpm.new_page();
    let comp = gen_generic_key_cmp_with_schema(metadata.clone_key_schema());
    let container = BPlusTree::new(
        metadata.get_name().to_string(),
        bpm,
        header_page_id,
        page_leaf_max_size,
        page_interal_max_size,
        comp
    );
    BPlusTreeIndex {
        metadata,
        container: Mutex::new(container),
    }
}

pub fn new_gk_b_plus_tree_index_with_sizes<const N: usize, V>(
    metadata: IndexMetadata, 
    bpm: Arc<BufferPoolManager>,
    mut leaf_max_size: usize,
    mut internal_max_size: usize,
) -> BPlusTreeIndex<GenericKey<N>, V, impl Fn(&GenericKey<N>, &GenericKey<N>) -> Ordering> 
{
    // Default leaf node max size.
    let page_leaf_max_size: usize = (BUSTUB_PAGE_SIZE - std::mem::size_of::<BPlusTreeLeafPage<(), ()>>()) /
        (std::mem::size_of::<GK<N>>() + std::mem::size_of::<V>());
    // Default internal node max size.
    let page_interal_max_size: usize = (BUSTUB_PAGE_SIZE - std::mem::size_of::<BPlusTreeInternalPage<(), ()>>()) /
        (std::mem::size_of::<GK<N>>() + std::mem::size_of::<V>());

    let header_page_id = bpm.new_page();
    leaf_max_size = leaf_max_size.min(page_leaf_max_size);
    internal_max_size = internal_max_size.min(page_interal_max_size);
    let container = BPlusTree::new(
        metadata.get_name().to_string(),
        bpm,
        header_page_id,
        leaf_max_size,
        internal_max_size,
        gen_generic_key_cmp_with_schema(metadata.clone_key_schema())
    );
    BPlusTreeIndex {
        metadata,
        container: Mutex::new(container),
    }
}

impl<const N: usize, C> Index for BPlusTreeIndex<GK<N>, RID, C> 
where
    C: Fn(&GK<N>, &GK<N>) -> Ordering
{
    fn get_metadata(&self) -> &IndexMetadata {
        &self.metadata
    }

    fn insert_entry(&self, key: &Tuple, rid: RID, _transaction: Option<&Transaction>) -> bool {
        let index_key = GK::<N>::from_tuple_key(key);
        self.container.lock().unwrap().insert(index_key, rid);
        true
    }

    fn delete_entry(&self, key: &Tuple, _rid: RID, _transaction: Option<&Transaction>) {
        let index_key = GK::<N>::from_tuple_key(key);
        self.container.lock().unwrap().remove(&index_key);
    }

    fn scan_key(&self, key: &Tuple, result: &mut Vec<RID>, _transaction: Option<&Transaction>) {
        let index_key = GK::<N>::from_tuple_key(key);
        if let Some(&rid) = self.container.lock().unwrap().get_value(&index_key) {
            result.push(rid);
        }
    }
}
