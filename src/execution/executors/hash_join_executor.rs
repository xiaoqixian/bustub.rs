// Date:   Fri Jun 26 15:37:07 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// hash_join_executor.rs
//
// Identification: src/include/execution/executors/hash_join_executor.h
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
        plans::HashJoinPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * HashJoinExecutor executes a hash JOIN on two tables.
 */
pub struct HashJoinExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a HashJoinPlanNode,
    left_child: Box<dyn Executor + 'a>,
    right_child: Box<dyn Executor + 'a>,
}

impl<'a> HashJoinExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a HashJoinPlanNode,
        left_child: Box<dyn Executor + 'a>,
        right_child: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            left_child,
            right_child,
        }
    }
}

impl<'a> Iterator for HashJoinExecutor<'a> {
    type Item = (Tuple, RID);
    fn next(&mut self) -> Option<Self::Item> {
        todo!("")
    }
}

impl<'a> Executor for HashJoinExecutor<'a> {
    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }
}


