// Date:   Fri Jun 26 15:37:19 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// projection_executor.rs
//
// Identification: src/include/execution/executors/projection_executor.h
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
        plans::ProjectionPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * The ProjectionExecutor executor executes a projection.
 */
pub struct ProjectionExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a ProjectionPlanNode,
    child_executor: Box<dyn Executor + 'a>,
}

impl<'a> ProjectionExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a ProjectionPlanNode,
        child_executor: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            child_executor,
        }
    }
}

impl<'a> Executor for ProjectionExecutor<'a> {
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


