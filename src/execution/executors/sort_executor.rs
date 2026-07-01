// Date:   Fri Jun 26 15:37:23 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// sort_executor.rs
//
// Identification: src/include/execution/executors/external_merge_sort_executor.h
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
        plans::SortPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * SortExecutor executes a sort operation using external merge sort.
 */
pub struct SortExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a SortPlanNode,
    #[allow(dead_code)]
    child_executor: Box<dyn Executor + 'a>,
}

impl<'a> SortExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a SortPlanNode,
        child_executor: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            child_executor,
        }
    }
}

impl<'a> Executor for SortExecutor<'a> {
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


