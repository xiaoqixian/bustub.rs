mod b_plus_tree_insert_test;
mod b_plus_tree_delete_test;
mod b_plus_tree_concurrent_test;
mod b_plus_tree_contention_test;
mod b_plus_tree_sequential_scale_test;

use std::cmp::Ordering;

/// Comparator type for `i64` keys, compatible with `BPlusTree<i64, RID, IntComparator>`.
pub type IntComparator = fn(&i64, &i64) -> Ordering;

/// Natural-order comparator for `i64` values.
pub fn int_comparator(a: &i64, b: &i64) -> Ordering {
    a.cmp(b)
}
