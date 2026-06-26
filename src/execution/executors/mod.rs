use crate::{
    catalog::Schema,
    common::{errors::BustubError, rid::RID},
    execution::{
        executor_context::ExecutorContext,
        executors::{
            aggregation_executor::AggregationExecutor,
            delete_executor::DeleteExecutor,
            filter_executor::FilterExecutor,
            hash_join_executor::HashJoinExecutor,
            index_scan_executor::IndexScanExecutor,
            insert_executor::InsertExecutor,
            limit_executor::LimitExecutor,
            mock_scan_executor::MockScanExecutor,
            nested_index_join_executor::NestedIndexJoinExecutor,
            projection_executor::ProjectionExecutor,
            seq_scan_executor::SeqScanExecutor,
            sort_executor::SortExecutor,
            topn_per_group_executor::TopNPerGroupExecutor,
            update_executor::UpdateExecutor,
            values_executor::ValuesExecutor,
            window_function_executor::WindowFunctionExecutor,
        },
        plans::PlanNode,
    },
    storage::table::tuple::Tuple,
};

pub mod aggregation_executor;
pub mod delete_executor;
pub mod filter_executor;
pub mod hash_join_executor;
pub mod index_scan_executor;
pub mod insert_executor;
pub mod limit_executor;
pub mod mock_scan_executor;
pub mod nested_index_join_executor;
pub mod projection_executor;
pub mod seq_scan_executor;
pub mod sort_executor;
pub mod topn_per_group_executor;
pub mod update_executor;
pub mod values_executor;
pub mod window_function_executor;

pub trait Executor {
    fn next(&mut self, batch_size: usize) -> Option<(Vec<Tuple>, Vec<RID>)>;
    fn output_schema_ref(&self) -> &Schema;
    fn executor_context(&self) -> &ExecutorContext;
}

pub struct ExecutorFactory;

impl ExecutorFactory {
    /**
     * Creates a new executor given the executor context and plan node.
     * @param exec_ctx The executor context for the created executor
     * @param plan The plan node that needs to be executed
     * @return An executor for the given plan in the provided context
     */
    pub fn create_executor<'a>(
        exec_ctx: &'a ExecutorContext,
        plan: &'a PlanNode,
    ) -> Result<Box<dyn Executor + 'a>, BustubError> {
        match plan {
            // Create a new sequential scan executor
            PlanNode::SeqScan(plan) => Ok(Box::new(SeqScanExecutor::new(exec_ctx, plan))),

            // Create a new index scan executor
            PlanNode::IndexScan(plan) => Ok(Box::new(IndexScanExecutor::new(exec_ctx, plan))),

            // Create a new insert executor
            PlanNode::Insert(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(InsertExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new update executor
            PlanNode::Update(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(UpdateExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new delete executor
            PlanNode::Delete(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(DeleteExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new limit executor
            PlanNode::Limit(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(LimitExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new aggregation executor
            PlanNode::Aggregation(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(AggregationExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new window function executor
            PlanNode::Window(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(WindowFunctionExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new nested-index join executor
            PlanNode::NestedIndexJoin(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(NestedIndexJoinExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new hash join executor
            PlanNode::HashJoin(plan) => {
                let left_child = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                let right_child = ExecutorFactory::create_executor(exec_ctx, &plan.children[1])?;
                Ok(Box::new(HashJoinExecutor::new(exec_ctx, plan, left_child, right_child)))
            }

            // Create a new mock scan executor
            PlanNode::MockScan(plan) => Ok(Box::new(MockScanExecutor::new(exec_ctx, plan))),

            // Create a new projection executor
            PlanNode::Projection(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(ProjectionExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new filter executor
            PlanNode::Filter(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(FilterExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new values executor
            PlanNode::Values(plan) => Ok(Box::new(ValuesExecutor::new(exec_ctx, plan))),

            // Create a new sort executor
            PlanNode::Sort(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(SortExecutor::new(exec_ctx, plan, child_executor)))
            }

            // Create a new topN per group executor
            PlanNode::TopNPerGroup(plan) => {
                let child_executor = ExecutorFactory::create_executor(exec_ctx, &plan.children[0])?;
                Ok(Box::new(TopNPerGroupExecutor::new(exec_ctx, plan, child_executor)))
            }

            // NestedLoopJoin is not yet supported
            PlanNode::NestedLoopJoin(_) => Err(BustubError::Message(format!(
                "{} plan executor is not supported yet.",
                plan
            ))),

            // TopN is not yet supported
            PlanNode::TopN(_) => Err(BustubError::Message(format!(
                "{} plan executor is not supported yet.",
                plan
            ))),
        }
    }
}
