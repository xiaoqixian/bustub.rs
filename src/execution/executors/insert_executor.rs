// Date:   Fri Jun 26 15:36:39 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// insert_executor.rs
//
// Identification: src/include/execution/executors/insert_executor.h
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
        plans::InsertPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * InsertExecutor executes an insert on a table.
 * Inserted values are always pulled from a child executor.
 */
pub struct InsertExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a InsertPlanNode,
    #[allow(dead_code)]
    child_executor: Box<dyn Executor + 'a>,
}

impl<'a> InsertExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a InsertPlanNode,
        child_executor: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            child_executor,
        }
    }
}

impl<'a> Executor for InsertExecutor<'a> {
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


