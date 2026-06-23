use crate::{
    binder::{BoundExpression, SelectStatement, TableRef},
    catalog::SchemaRef,
    common::errors::BustubError,
    execution::{
        expressions::{AbstractExpression, ColumnValueExpression},
        plans::{AggregationPlanNode, LimitPlanNode, PlanNode, ProjectionPlanNode, SortPlanNode},
    },
    planner::PlannerContextGuard,
    sql_type::TypeId,
};

use super::Planner;

pub fn plan_select(planner: &mut Planner, sel: &SelectStatement) -> Result<PlanNode, BustubError> {
    let _ctx_guard = PlannerContextGuard::new(&mut planner.ctx);
    if !sel.ctes.is_empty() {
        return Err(BustubError::Message(
            "select with cte is not supported yet.".to_string(),
        ));
    }
    let plan = match &sel.table {
        TableRef::Empty => {
            return Err(BustubError::Message(
                "select with empty table if not supported yet.".to_string(),
            ))
        }
        t => planner.plan_table_ref(t)?,
    };

    if sel.where_clause.is_some() {
        return Err(BustubError::Message(
            "select with where clause is not supported yet.".to_string(),
        ));
    }

    let (has_agg, has_window_agg) = sel
        .select_list
        .iter()
        .find_map(|item| {
            if item.has_aggregation() {
                Some((true, false))
            } else if item.has_window_function() {
                Some((false, true))
            } else {
                None
            }
        })
        .unwrap_or((false, false));

    let has_having = sel.having.is_some();
    let has_group_by = !sel.group_by.is_empty();
    let plan = match (has_window_agg, has_agg, has_having, has_group_by) {
        (true, ..) => {
            if has_having {
                return Err(BustubError::Message(
                    "HAVING on window function is not supported yet.".to_string(),
                ));
            }
            if has_group_by {
                return Err(BustubError::Message(
                    "GROUP BY is not allowed to use with window function".to_string(),
                ));
            }

            planner.plan_select_window(sel, plan)?
        }
        (false, false, false, false) => {
            let mut col_names = Vec::with_capacity(sel.select_list.len());
            let mut exprs = Vec::with_capacity(sel.select_list.len());
            let children = vec![plan];
            for item in sel.select_list.iter() {
                let (name, expr) = planner.plan_expression(item, &children)?;
                col_names.push(name);
                exprs.push(expr);
            }
            let schema = SchemaRef::new(ProjectionPlanNode::infer_projection_schema(&exprs));
            PlanNode::Projection(ProjectionPlanNode {
                output_schema: schema,
                children,
                expressions: exprs,
            })
        }
        _ => planner.plan_select_agg(sel, plan)?,
    };

    // Plan DISTINCT as group agg
    let plan = match sel.is_distinct {
        false => plan,
        true => {
            let group_bys = plan
                .output_schema_ref()
                .columns
                .iter()
                .enumerate()
                .map(|(col_idx, col)| {
                    AbstractExpression::ColumnValue(ColumnValueExpression {
                        tuple_idx: 0,
                        col_idx,
                        ret_type: col.clone(),
                    })
                })
                .collect();
            PlanNode::Aggregation(AggregationPlanNode {
                output_schema: plan.output_schema().clone(),
                children: vec![plan],
                group_bys,
                aggregates: vec![],
                agg_types: vec![],
            })
        }
    };

    let plan = match sel.sort.is_empty() {
        true => plan,
        false => {
            let mut order_bys = Vec::with_capacity(sel.sort.len());
            let output_schema = plan.output_schema().clone();
            let children = vec![plan];
            for ob in sel.sort.iter() {
                let (_, expr) = planner.plan_expression(&ob.expr, &children)?;
                order_bys.push((ob.order_by_type, expr));
            }
            PlanNode::Sort(SortPlanNode {
                output_schema,
                children,
                order_bys,
            })
        }
    };

    let plan = match (sel.limit_count.as_ref(), sel.limit_offset.as_ref()) {
        (None, None) => plan,
        (Some(count), None) => {
            let count = match count {
                BoundExpression::BoundConstant(constant)
                    if constant.val.sql_type_id == TypeId::Integer =>
                {
                    constant.val.get_as::<i32>()
                }
                _ => {
                    return Err(BustubError::Message(
                        "LIMIT clause must be an integer constant".to_string(),
                    ))
                }
            } as usize;

            PlanNode::Limit(LimitPlanNode {
                output_schema: plan.output_schema().clone(),
                children: vec![plan],
                limit: count,
            })
        }
        (_, Some(offset)) => match offset {
            BoundExpression::BoundConstant(constant)
                if constant.val.sql_type_id == TypeId::Integer =>
            {
                return Err(BustubError::Message(
                    "OFFSET clause is not supported yet.".to_string(),
                ));
            }
            _ => {
                return Err(BustubError::Message(
                    "OFFSET clause must be an integer constant".to_string(),
                ))
            }
        },
    };

    Ok(plan)
}
