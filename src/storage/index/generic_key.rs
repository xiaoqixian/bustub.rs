//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// generic_key.rs
//
// Identification: src/storage/index/generic_key.rs
//
// Copyright (c) 2015-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::{cmp::Ordering, slice, sync::Arc};

use crate::{catalog::{Column, Schema}, sql_type::{CmpBool, Value}, storage::table::tuple::Tuple};

/// A fixed-size key wrapper for use with B+Tree indices.
///
/// `GenericKey<N>` stores a `N`-byte buffer of encoded key data. It provides
/// `Ord`-based comparison (unsigned byte‑by‑byte) so that two keys can be
/// ordered by their encoded representation.  This mirrors the original C++
/// `GenericKey<KeySize>` + `GenericComparator<KeySize>` pair.
///
/// # Type parameter
///
/// * `N` – the size (in bytes) of the fixed‑length key buffer.
///         Must be large enough to hold the encoded key columns.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct GenericKey<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> GenericKey<N> {
    /// Create a new zero-initialized key.
    #[inline]
    pub fn new() -> Self {
        Self { data: [0u8; N] }
    }

    pub fn from_tuple_key(tuple: &Tuple) -> Self {
        let mut data = [0u8; N];
        let src_data = tuple.get_data();
        let copy_len = N.min(data.len());
        data[..copy_len].copy_from_slice(&src_data[..copy_len]);
        Self { data }
    }

    /// Return a raw pointer to the internal data.
    #[inline]
    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Return a mutable raw pointer to the internal data.
    #[inline]
    pub fn data_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Return a reference to the underlying byte array.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Return a mutable reference to the underlying byte array.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Return the size of the fixed‑length key in bytes.
    #[inline]
    pub fn size(&self) -> usize {
        N
    }

    /// Set the key from an `i64` value.
    ///
    /// The value is encoded in big‑endian with the sign bit flipped, which
    /// ensures that negative integers sort before positive ones under
    /// unsigned byte‑wise comparison.
    #[inline]
    pub fn set_from_integer(&mut self, key: i64) {
        let swapped = (key as u64) ^ 0x8000_0000_0000_0000;
        let bytes = swapped.to_be_bytes();
        // Copy as many bytes as N allows (at most 8).
        let copy_len = N.min(8);
        self.data[..copy_len].copy_from_slice(&bytes[..copy_len]);
    }

    /// Set the key from a `Tuple` using the provided key size.
    ///
    /// This copies the raw tuple data into the fixed‑size buffer.  The caller
    /// is responsible for ensuring that the tuple data has been encoded in a
    /// way that preserves sort order.
    ///
    /// **Note**: The original C++ version uses `Tuple::KeyFromTuple` to
    /// produce the serialized key first; this method is kept as a low‑level
    /// building block.  Prefer using the encoding functions in
    /// `BPlusTreeIndex` instead.
    #[inline]
    pub fn set_from_tuple_key(&mut self, tuple: &Tuple, key_size: usize) {
        let data = tuple.get_data();
        let copy_len = N.min(key_size).min(data.len());
        self.data[..copy_len].copy_from_slice(&data[..copy_len]);
    }

    /// Serialize the key into a byte slice.
    ///
    /// # Panics
    ///
    /// Panics if `storage.len() < N`.
    #[inline]
    pub fn serialize_to(&self, storage: &mut [u8]) {
        assert!(storage.len() >= N, "storage buffer too small");
        storage[..N].copy_from_slice(&self.data);
    }

    /// Deserialize the key from a byte slice.
    ///
    /// # Panics
    ///
    /// Panics if `storage.len() < N`.
    #[inline]
    pub fn deserialize_from(&mut self, storage: &[u8]) {
        assert!(storage.len() >= N, "storage buffer too small");
        self.data.copy_from_slice(&storage[..N]);
    }

    pub fn to_value(&self, col: &Column) -> Value {
        let slice = unsafe {
            let data_ptr = if col.is_inlined() {
                self.data_ptr().add(col.get_offset())
            } else {
                let data_ptr = self.data_ptr();
                let offset = *((data_ptr.add(col.get_offset())) as *const usize);
                data_ptr.add(offset)
            };
            slice::from_raw_parts(data_ptr, col.length)
        };
        Value::deserialize_from(slice, col.get_type())
    }
}

// --- Trait implementations ------------------------------------------------

impl<const N: usize> Default for GenericKey<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Comparison uses unsigned byte‑by‑byte order (same as `memcmp` on `uint8_t*`
/// in the original C++ `GenericComparator`).
impl<const N: usize> Ord for GenericKey<N> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl<const N: usize> PartialOrd for GenericKey<N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> PartialEq for GenericKey<N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<const N: usize> Eq for GenericKey<N> {}

// --- From implementations --------------------------------------------------

impl<const N: usize> From<i64> for GenericKey<N> {
    fn from(value: i64) -> Self {
        let mut key = GenericKey::new();
        key.set_from_integer(value);
        key
    }
}

impl<const N: usize> From<i32> for GenericKey<N> {
    fn from(value: i32) -> Self {
        GenericKey::from(value as i64)
    }
}

impl<const N: usize> From<[u8; N]> for GenericKey<N> {
    fn from(data: [u8; N]) -> Self {
        GenericKey { data }
    }
}

// --- Debug -----------------------------------------------------------------

impl<const N: usize> std::fmt::Debug for GenericKey<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GenericKey<{}>({:?})", N, &self.data[..])
    }
}

// ---------------------------------------------------------------------------
// GenericComparator
// ---------------------------------------------------------------------------

/// A comparator function for `GenericKey<N>`, suitable for passing as the `C`
/// type parameter of `BPlusTree<K, V, C>`.
///
pub fn gen_generic_key_cmp_with_schema<const N: usize>(schema: Arc<Schema>) -> impl Fn(&GenericKey<N>, &GenericKey<N>) -> Ordering {
    move |x, y| {
        for col in schema.columns.iter() {
            let lhs = x.to_value(col);
            let rhs = y.to_value(col);

            match lhs.compare_less_than(&rhs) {
                CmpBool::CmpTrue => return Ordering::Less,
                CmpBool::CmpFalse => return Ordering::Greater,
                CmpBool::CmpNull => {}
            }
        }
        Ordering::Equal
    }
}
