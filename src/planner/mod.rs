use crate::{binder::BoundStatement, catalog::Catalog, execution::plans::PlanNode};

#[allow(dead_code)]
pub struct Planner<'cat> {
    catalog: &'cat Catalog,
}

impl<'cat> Planner<'cat> {
    pub fn new(catalog: &'cat Catalog) -> Self {
        Self {
            catalog
        }
    }

    pub fn plan_query(&self, _stmt: &BoundStatement) -> Result<PlanNode, ()> {
        todo!("")
    }
}
