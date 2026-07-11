use crate::{binder::{BoundExpression, BoundStatement, InsertStatement, SelectStatement, TableRef}, catalog::CatalogRef, common::errors::BustubError, execution::{expressions::AbstractExpression, plans::PlanNode}};

mod plan_expression;
mod plan_select;
mod plan_table;
mod plan_insert;

struct PlannerContext {}

struct PlannerContextGuard {
    ctx_ptr: *mut PlannerContext,
    old_ctx: Option<PlannerContext>
}

#[allow(dead_code)]
pub struct Planner {
    catalog: CatalogRef,
    ctx: PlannerContext,
}

impl PlannerContextGuard {
    pub fn new(ctx: &mut PlannerContext) -> Self {
        let old_ctx = std::mem::replace(ctx, PlannerContext {});
        Self {
            ctx_ptr: ctx as *mut _,
            old_ctx: Some(old_ctx),
        }
    }
}

impl Drop for PlannerContextGuard {
    fn drop(&mut self) {
        if let Some(old_ctx) = self.old_ctx.take() {
            unsafe { *self.ctx_ptr = old_ctx; }
        }
    }
}

impl Planner {
    pub fn new(catalog: CatalogRef) -> Self {
        Self {
            catalog,
            ctx: PlannerContext {}
        }
    }

    pub fn plan_query(&mut self, stmt: &BoundStatement) -> Result<PlanNode, BustubError> {
        match stmt {
            BoundStatement::Select(sel) => self.plan_select(sel),
            BoundStatement::Insert(ins) => self.plan_insert(ins),
            _ => Err(BustubError::Message(format!("Planner: unsupported statement type {}", stmt)))
        }
    }

    fn plan_select(&mut self, sel: &SelectStatement) -> Result<PlanNode, BustubError> {
        plan_select::plan_select(self, sel)
    }

    fn plan_insert(&mut self, ins: &InsertStatement) -> Result<PlanNode, BustubError> {
        plan_insert::plan_insert(self, ins)
    }

    fn plan_expression(&mut self, expr: &BoundExpression, children: &[PlanNode]) -> Result<(String, AbstractExpression), BustubError> {
        plan_expression::plan_expression(self, expr, children)
    }

    fn plan_table_ref(&self, table_ref: &TableRef) -> Result<PlanNode, BustubError> {
        match table_ref {
            TableRef::BaseTableRef(t) => plan_table::plan_base_table_ref(t),
            _ => Err(BustubError::Message(format!("table ref planning not supported yet: {:?}", table_ref)))
        }
    }

    fn plan_select_window(&self, _sel: &SelectStatement, _child: PlanNode) -> Result<PlanNode, BustubError> {
        todo!("")
    }

    fn plan_select_agg(&self, _sel: &SelectStatement, _child: PlanNode) -> Result<PlanNode, BustubError> {
        todo!("")
    }
}
