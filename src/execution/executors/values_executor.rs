// Date:   Fri Jun 26 15:37:15 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// values_executor.rs
//
// Identification: src/include/execution/executors/values_executor.h
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
        plans::ValuesPlanNode,
    },
    storage::table::tuple::Tuple,
};

/**
 * The ValuesExecutor executor produces rows of values.
 */
pub struct ValuesExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a ValuesPlanNode,
}

impl<'a> ValuesExecutor<'a> {
    pub fn new(exec_ctx: &'a ExecutorContext, plan: &'a ValuesPlanNode) -> Self {
        Self { exec_ctx, plan }
    }
}

impl<'a> Iterator for ValuesExecutor<'a> {
    type Item = (Tuple, RID);
    fn next(&mut self) -> Option<Self::Item> {
        todo!("")
    }
}

impl<'a> Executor for ValuesExecutor<'a> {
    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }
}


