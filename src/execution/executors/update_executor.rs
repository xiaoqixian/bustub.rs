// Date:   Fri Jun 26 15:36:42 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// update_executor.rs
//
// Identification: src/include/execution/executors/update_executor.h
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
        plans::UpdatePlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * UpdateExecutor executes an update on a table.
 * Updated values are always pulled from a child.
 */
pub struct UpdateExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a UpdatePlanNode,
    child_executor: Box<dyn Executor + 'a>,
}

impl<'a> UpdateExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a UpdatePlanNode,
        child_executor: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            child_executor,
        }
    }
}

impl<'a> Iterator for UpdateExecutor<'a> {
    type Item = (Tuple, RID);
    fn next(&mut self) -> Option<Self::Item> {
        todo!("")
    }
}

impl<'a> Executor for UpdateExecutor<'a> {
    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }
}


