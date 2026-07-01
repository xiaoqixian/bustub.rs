// Date:   Fri Jun 26 15:37:27 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// topn_per_group_executor.rs
//
// Identification: src/include/execution/executors/topn_per_group_executor.h
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
        plans::TopNPerGroupPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * The TopNPerGroupExecutor executor executes a topN per group.
 */
pub struct TopNPerGroupExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a TopNPerGroupPlanNode,
    #[allow(dead_code)]
    child_executor: Box<dyn Executor + 'a>,
}

impl<'a> TopNPerGroupExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a TopNPerGroupPlanNode,
        child_executor: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            child_executor,
        }
    }
}

impl<'a> Executor for TopNPerGroupExecutor<'a> {
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


