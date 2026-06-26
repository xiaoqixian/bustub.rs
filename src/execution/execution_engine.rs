use crate::{common::errors::BustubError, execution::{executor_context::ExecutorContext, executors::ExecutorFactory, plans::PlanNode}, storage::table::tuple::Tuple};

pub struct ExecutionEngine {}

impl ExecutionEngine {
    pub fn execute(&self, plan: &PlanNode, exec_ctx: &ExecutorContext) -> Result<Vec<Tuple>, BustubError> {
        let mut executor = ExecutorFactory::create_executor(exec_ctx, plan)?;
        let mut result_set = Vec::new();
        while let Some((tuple, _)) = executor.next() {
            result_set.push(tuple);
        }
        Ok(result_set)
    }
}
