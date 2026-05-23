//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// type_util.rs
//
// Identification: src/sql_type/type_util.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::cmp::Ordering;

/// Type utility functions.
pub struct TypeUtil;

impl TypeUtil {
    /// Use memcmp to evaluate two strings.
    /// This does not work with VARBINARY attributes.
    pub fn compare_strings(str1: &[u8], len1: usize, str2: &[u8], len2: usize) -> Ordering {
        let min_len = len1.min(len2);
        let a = &str1[..min_len];
        let b = &str2[..min_len];
        match a.cmp(b) {
            Ordering::Equal => {
                if len1 != len2 {
                    len1.cmp(&len2)
                } else {
                    Ordering::Equal
                }
            }
            other => other,
        }
    }
}


