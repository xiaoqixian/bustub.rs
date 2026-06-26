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
    // child tuple batch offset
    child_offset: usize,
    // reusable child tuple batch & rid batch
    child_tuples: Vec<Tuple>,
    child_rids: Vec<RID>,
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
            child_offset: 0,
            child_tuples: Vec::new(),
            child_rids: Vec::new(),
        }
    }
}

impl<'a> Executor for ProjectionExecutor<'a> {
    fn next(&mut self, batch_size: usize) -> Option<(Vec<Tuple>, Vec<RID>)> {
        let mut tuple_batch = Vec::with_capacity(batch_size);
        let mut rid_batch = Vec::with_capacity(batch_size);

        while tuple_batch.len() < batch_size {
            for (i, (child_tuple, child_rid)) in self.child_tuples.iter().zip(self.child_rids.iter()).enumerate().skip(self.child_offset) {
                let child_schema = self.child_executor.output_schema_ref();
                let values = self.plan.expressions.iter()
                    .map(|expr| expr.evaluate(child_tuple, child_schema))
                    .collect::<Vec<_>>();
                
                tuple_batch.push(Tuple::new_with_values(values, &self.plan.output_schema));
                rid_batch.push(*child_rid);

                if tuple_batch.len() == batch_size {
                    self.child_offset = i;
                    break;
                }
            }
            
            if tuple_batch.len() < batch_size {
                (self.child_tuples, self.child_rids) = match self.child_executor.next(batch_size) {
                    None => break,
                    Some(x) => x,
                };
                self.child_offset = 0;
            }
        }

        match tuple_batch.is_empty() {
            true => None,
            false => Some((tuple_batch, rid_batch))
        }
    }

    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }
}


