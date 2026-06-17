use crate::catalog::Schema;

pub enum PlanNode {
    SeqScan(SeqScanPlanNode)
}

pub struct SeqScanPlanNode {}

impl PlanNode {
    pub fn output_schema_ref(&self) -> &Schema {
        todo!("")
    }
}
