// Date:   Fri Jun 26 15:36:34 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// index_scan_executor.rs
//
// Identification: src/include/execution/executors/index_scan_executor.h
//
// Copyright (c) 2015-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use crate::{
    catalog::Schema,
    common::rid::RID,
    execution::{
        executor_context::ExecutorContext,
        executors::Executor,
        plans::IndexScanPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * IndexScanExecutor executes an index scan over a table.
 */
pub struct IndexScanExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a IndexScanPlanNode,
}

impl<'a> IndexScanExecutor<'a> {
    pub fn new(exec_ctx: &'a ExecutorContext, plan: &'a IndexScanPlanNode) -> Self {
        Self { exec_ctx, plan }
    }
}

impl<'a> Iterator for IndexScanExecutor<'a> {
    type Item = (Tuple, RID);
    fn next(&mut self) -> Option<Self::Item> {
        todo!("")
    }
}

impl<'a> Executor for IndexScanExecutor<'a> {
    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }
}


