use crate::{binder::BaseTableRef, catalog::SchemaRef, common::errors::BustubError, execution::plans::{MockScanPlanNode, PlanNode, SeqScanPlanNode}};

pub fn plan_base_table_ref(table_ref: &BaseTableRef) -> Result<PlanNode, BustubError> {
    let table_name = table_ref.table.as_str();
    
    if table_name.starts_with("__") {
        return if table_name.starts_with("__mock") {
            Ok(PlanNode::MockScan(MockScanPlanNode {
                output_schema: SchemaRef::new(SeqScanPlanNode::infer_scan_schema(table_ref)),
                table: table_ref.table.clone(),
            }))
        } else {
            Err(BustubError::Message(format!("unsupported internal table: {}", table_name)))
        }
    }

    Ok(PlanNode::SeqScan(SeqScanPlanNode {
        output_schema: SchemaRef::new(SeqScanPlanNode::infer_scan_schema(table_ref)),
        table_name: table_ref.table.clone(),
        table_oid: table_ref.oid,
        filter_predicate: None,
    }))
}
