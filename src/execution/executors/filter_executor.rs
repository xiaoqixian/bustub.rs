// Date:   Fri Jun 26 15:37:10 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// filter_executor.rs
//
// Identification: src/include/execution/executors/filter_executor.h
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
        plans::FilterPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * The FilterExecutor executor executes a filter.
 */
pub struct FilterExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a FilterPlanNode,
    child_executor: Box<dyn Executor + 'a>,
}

impl<'a> FilterExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a FilterPlanNode,
        child_executor: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            child_executor,
        }
    }
}

impl<'a> Iterator for FilterExecutor<'a> {
    type Item = (Tuple, RID);
    fn next(&mut self) -> Option<Self::Item> {
        todo!("")
    }
}

impl<'a> Executor for FilterExecutor<'a> {
    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }
}


