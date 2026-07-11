use crate::{binder::InsertStatement, catalog::{Column, Schema, SchemaRef}, common::errors::BustubError, execution::plans::{InsertPlanNode, PlanNode}, planner::Planner, sql_type::TypeId};

pub fn plan_insert(planner: &mut Planner, ins: &InsertStatement) -> Result<PlanNode, BustubError> {
    let child = planner.plan_select(&ins.select)?;
    {
        let child_schema = child.output_schema_ref();
        let table_schema = &ins.table.schema;
        let column_matched = child_schema.columns.iter()
            .zip(table_schema.columns.iter())
            .all(|(a, b)| a.get_type() == b.get_type());
        if !column_matched {
            return Err(BustubError::Message("insert schema mismatch".to_string()));
        }
    }

    let output_schema = SchemaRef::new(Schema::new(
            vec![Column::new("__bustub_internal.insert_rows", TypeId::Integer)]));
    
    Ok(PlanNode::Insert(InsertPlanNode {
        children: vec![child],
        output_schema,
        table_oid: ins.table.oid,
    }))
}
