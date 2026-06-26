// Date:   Fri Jun 26 15:36:55 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// aggregation_executor.rs
//
// Identification: src/include/execution/executors/aggregation_executor.h
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
        plans::AggregationPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * AggregationExecutor executes an aggregation operation (e.g. COUNT, SUM, MIN, MAX)
 * over the tuples produced by a child executor.
 */
pub struct AggregationExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a AggregationPlanNode,
    child_executor: Box<dyn Executor + 'a>,
}

impl<'a> AggregationExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a AggregationPlanNode,
        child_executor: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            child_executor,
        }
    }
}

impl<'a> Iterator for AggregationExecutor<'a> {
    type Item = (Tuple, RID);
    fn next(&mut self) -> Option<Self::Item> {
        todo!("")
    }
}

impl<'a> Executor for AggregationExecutor<'a> {
    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }
}


