use crate::{catalog::Catalog, execution::plans::PlanNode};

#[allow(dead_code)]
pub struct Optimizer<'cat> {
    catalog: &'cat Catalog,
    force_starter_rule: bool,
}

impl<'cat> Optimizer<'cat> {
    pub fn new(catalog: &'cat Catalog, force_starter_rule: bool) -> Self {
        Self {
            catalog,
            force_starter_rule
        }
    }

    pub fn optimize(&self, plan: PlanNode) -> PlanNode {
        plan
    }
}
