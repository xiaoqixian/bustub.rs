// Date:   Fri Jun 26 15:36:51 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// limit_executor.rs
//
// Identification: src/include/execution/executors/limit_executor.h
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
        plans::LimitPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * LimitExecutor limits the number of output tuples produced by a child operator.
 */
pub struct LimitExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a LimitPlanNode,
    child_executor: Box<dyn Executor + 'a>,
}

impl<'a> LimitExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a LimitPlanNode,
        child_executor: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            child_executor,
        }
    }
}

impl<'a> Executor for LimitExecutor<'a> {
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


