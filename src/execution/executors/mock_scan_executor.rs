// Date:   Fri Jun 26 15:37:32 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// mock_scan_executor.rs
//
// Identification: src/include/execution/executors/mock_scan_executor.h
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
        plans::MockScanPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * The MockScanExecutor executor executes a sequential table scan for tests.
 */
pub struct MockScanExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a MockScanPlanNode,
}

impl<'a> MockScanExecutor<'a> {
    pub fn new(exec_ctx: &'a ExecutorContext, plan: &'a MockScanPlanNode) -> Self {
        Self { exec_ctx, plan }
    }
}

impl<'a> Executor for MockScanExecutor<'a> {
    fn next(&mut self, _batch_size: usize) -> Option<(Vec<Tuple>, Vec<RID>)> {
        todo!("")
    }

    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }
}


