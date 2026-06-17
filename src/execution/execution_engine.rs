use crate::{execution::{executor_context::ExecutorContext, plans::PlanNode}, storage::table::tuple::Tuple};

pub struct ExecutionEngine {}

impl ExecutionEngine {
    pub fn execute(&self, _plan: &PlanNode, _exec_ctx: &ExecutorContext) -> Result<Vec<Tuple>, String> {
        todo!("")
    }
}
