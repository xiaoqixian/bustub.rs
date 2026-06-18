use crate::catalog::Schema;

pub enum PlanNode {
    SeqScan(SeqScanPlanNode),
    MockScan(MockScanPlanNode),
}

pub struct SeqScanPlanNode {}

pub struct MockScanPlanNode {
    pub table: String
}

impl PlanNode {
    pub fn output_schema_ref(&self) -> &Schema {
        todo!("")
    }
}

impl MockScanPlanNode {
    pub fn output_schema_ref(&self) -> &Schema {
        todo!("")
    }
}
