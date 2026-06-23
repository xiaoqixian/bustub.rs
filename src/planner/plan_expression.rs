
use crate::{
    binder::{BoundColumnRef, BoundExpression},
    common::errors::BustubError,
    execution::{
        expressions::AbstractExpression,
        plans::PlanNode,
    },
};

use super::Planner;

pub fn plan_expression(
    planner: &Planner,
    expr: &BoundExpression,
    children: &[PlanNode],
) -> Result<(String, AbstractExpression), BustubError> {
    match expr {
        BoundExpression::BoundColumnRef(col_ref) => plan_column_ref(planner, col_ref, children),
        _ => Err(BustubError::Message(format!(
            "expr planning is not supported yet: {}",
            expr
        ))),
    }
}

pub fn plan_column_ref(
    _planner: &Planner,
    col_ref: &BoundColumnRef,
    children: &[PlanNode],
) -> Result<(String, AbstractExpression), BustubError> {
    if children.is_empty() {
        return Err(BustubError::Message(
            "column ref should have at least one child".to_string(),
        ));
    }

    let col_name = col_ref.to_string();

    if children.len() == 1 {
        // Projections, Filters, and other executors evaluating expressions with one single child
        // will use this branch.
        let child = &children[0];
        let schema = child.output_schema_ref();

        // Before we can call get_col_idx, we need to ensure there's no duplicated column.
        let mut found = false;
        for col in schema.get_columns() {
            if col_name == col.get_name() {
                if found {
                    return Err(BustubError::Message(
                        "duplicated column found in schema".to_string(),
                    ));
                }
                found = true;
            }
        }

        let col_idx = schema.get_col_idx(&col_name) as usize;
        let col_type = schema.get_column(col_idx).clone();

        return Ok((
            col_name,
            AbstractExpression::column_value(0, col_idx, col_type),
        ));
    }

    if children.len() == 2 {
        /*
         * Joins will use this branch to plan expressions.
         *
         * If an expression is for join condition, e.g.
         * SELECT * from test_1 inner join test_2 on test_1.colA = test_2.col2
         * The plan will be like:
         * ```
         * NestedLoopJoin condition={ ColumnRef 0.0=ColumnRef 1.1 }
         *   SeqScan colA, colB
         *   SeqScan col1, col2
         * ```
         * In `ColumnRef n.m`, when executor is using the expression, it picks from its
         * nth child's mth column to get the data.
         */
        let left = &children[0];
        let right = &children[1];
        let left_schema = left.output_schema_ref();
        let right_schema = right.output_schema_ref();

        let col_idx_left = left_schema.try_get_col_idx(&col_name);
        let col_idx_right = right_schema.try_get_col_idx(&col_name);

        if col_idx_left.is_some() && col_idx_right.is_some() {
            return Err(BustubError::Message(format!(
                "ambiguous column name {}",
                col_name
            )));
        }

        if let Some(col_idx) = col_idx_left {
            let col_type = left_schema.get_column(col_idx as usize).clone();
            return Ok((
                col_name,
                AbstractExpression::column_value(0, col_idx as usize, col_type),
            ));
        }

        if let Some(col_idx) = col_idx_right {
            let col_type = right_schema.get_column(col_idx as usize).clone();
            return Ok((
                col_name,
                AbstractExpression::column_value(1, col_idx as usize, col_type),
            ));
        }

        return Err(BustubError::Message(format!(
            "column name {} not found",
            col_name
        )));
    }

    Err(BustubError::Message(
        "no executor with expression has more than 2 children for now".to_string(),
    ))
}
