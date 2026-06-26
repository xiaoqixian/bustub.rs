use crate::{catalog::Schema, common::rid::RID, execution::{executor_context::ExecutorContext, executors::Executor, plans::{SeqScanPlanNode}}, storage::table::tuple::Tuple};

pub struct SeqScanExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a SeqScanPlanNode,
}

impl<'a> SeqScanExecutor<'a> {
    pub fn new(exec_ctx: &'a ExecutorContext, plan: &'a SeqScanPlanNode) -> Self {
        Self { exec_ctx, plan }
    }
}

impl<'a> Executor for SeqScanExecutor<'a> {
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
