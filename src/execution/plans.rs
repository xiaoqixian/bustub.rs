//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// plans.rs
//
// Identification: src/execution/plans.rs
//
// Copyright (c) 2015-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::collections::HashMap;
use std::fmt::Display;

use crate::binder::BaseTableRef;
use crate::binder::OrderByType;
use crate::catalog::Column;
use crate::catalog::IndexOid;
use crate::catalog::{Schema, SchemaRef};
use crate::catalog::TableOid;
use crate::execution::expressions::AbstractExpression;
use crate::sql_type::type_id::TypeId;
use crate::sql_type::value::Value;

//===----------------------------------------------------------------------===//
// Type aliases
//===----------------------------------------------------------------------===//

/// JoinType enumerates all possible join types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinType {
    Invalid,
    Left,
    Right,
    Inner,
    Outer,
}

/// AggregationType enumerates all possible aggregation functions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggregationType {
    CountStar,
    Count,
    Sum,
    Min,
    Max,
}

/// WindowFunctionType enumerates all possible window functions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowFunctionType {
    CountStar,
    Count,
    Sum,
    Min,
    Max,
    Rank,
}

//===----------------------------------------------------------------------===//
// Aggregate helper structs
//===----------------------------------------------------------------------===//

/// AggregateKey represents a key used for grouping in an aggregation operation.
#[derive(Clone)]
pub struct AggregateKey {
    /// The group-by values.
    pub group_bys: Vec<Value>,
}

/// AggregateValue represents the running aggregate values.
#[derive(Clone)]
pub struct AggregateValue {
    /// The aggregate values.
    pub aggregates: Vec<Value>,
}

//===----------------------------------------------------------------------===//
// Plan node structs (data holders for each PlanNode enum variant)
//===----------------------------------------------------------------------===//

/// SeqScanPlanNode represents a sequential table scan operation.
#[derive(Clone)]
pub struct SeqScanPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The identifier of the table to be scanned.
    pub table_oid: TableOid,
    /// The name of the table to be scanned.
    pub table_name: String,
    /// An optional filter predicate pushed down to the scan.
    pub filter_predicate: Option<AbstractExpression>,
}

/// IndexScanPlanNode identifies a table to be scanned via an index with an optional predicate.
#[derive(Clone)]
pub struct IndexScanPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The identifier of the table on which the index is created.
    pub table_oid: TableOid,
    /// The identifier of the index whose tuples should be scanned.
    pub index_oid: IndexOid,
    /// An optional filter predicate pushed down to the index scan.
    pub filter_predicate: Option<AbstractExpression>,
    /// The constant value keys for point lookup (e.g., `WHERE v = 1`).
    pub pred_keys: Vec<AbstractExpression>,
}

/// InsertPlanNode identifies a table into which tuples are inserted.
///
/// The values to be inserted come from the child plan node.
#[derive(Clone)]
pub struct InsertPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The identifier of the table to insert into.
    pub table_oid: TableOid,
}

/// UpdatePlanNode identifies a table that should be updated.
///
/// The tuples to be updated come from the child plan node.
#[derive(Clone)]
pub struct UpdatePlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The identifier of the table to be updated.
    pub table_oid: TableOid,
    /// The target expressions for new column values.
    pub target_expressions: Vec<AbstractExpression>,
}

/// DeletePlanNode identifies a table from which tuples should be deleted.
///
/// The tuples to be deleted come from the child plan node.
#[derive(Clone)]
pub struct DeletePlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The identifier of the table from which tuples are deleted.
    pub table_oid: TableOid,
}

/// AggregationPlanNode represents SQL aggregation functions (COUNT, SUM, MIN, MAX, etc.).
///
/// NOTE: AggregationPlanNode must always have exactly one child.
#[derive(Clone)]
pub struct AggregationPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The GROUP BY expressions.
    pub group_bys: Vec<AbstractExpression>,
    /// The aggregation expressions.
    pub aggregates: Vec<AbstractExpression>,
    /// The aggregation function types.
    pub agg_types: Vec<AggregationType>,
}

/// LimitPlanNode constrains the number of output tuples produced by its child executor.
#[derive(Clone)]
pub struct LimitPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The maximum number of tuples to output.
    pub limit: usize,
}

/// NestedLoopJoinPlanNode joins tuples from two child plan nodes.
#[derive(Clone)]
pub struct NestedLoopJoinPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node (left and right inputs).
    pub children: Vec<PlanNode>,
    /// The join predicate. Tuples are joined if predicate(tuple) is true.
    pub predicate: AbstractExpression,
    /// The join type (inner, left, right, outer).
    pub join_type: JoinType,
}

/// NestedIndexJoinPlanNode represents a nested index join between two tables.
///
/// The outer table tuples are propagated from the child, but the inner table tuples
/// are obtained using the outer table tuples and an index from the catalog.
#[derive(Clone)]
pub struct NestedIndexJoinPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The predicate to extract the join key from the outer (child) tuple.
    pub key_predicate: AbstractExpression,
    /// The table OID of the inner table.
    pub inner_table_oid: TableOid,
    /// The index OID for the index on the inner table.
    pub index_oid: IndexOid,
    /// The name of the index.
    pub index_name: String,
    /// The name of the inner table.
    pub index_table_name: String,
    /// The schema of the inner table.
    pub inner_table_schema: SchemaRef,
    /// The join type.
    pub join_type: JoinType,
}

/// HashJoinPlanNode performs a hash join between two child plan nodes.
#[derive(Clone)]
pub struct HashJoinPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node (left and right inputs).
    pub children: Vec<PlanNode>,
    /// The expressions for computing the left join key.
    pub left_key_expressions: Vec<AbstractExpression>,
    /// The expressions for computing the right join key.
    pub right_key_expressions: Vec<AbstractExpression>,
    /// The join type.
    pub join_type: JoinType,
}

/// FilterPlanNode represents a filter operation.
///
/// It retains any tuple from its child that satisfies the predicate.
#[derive(Clone)]
pub struct FilterPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The predicate to test tuples against.
    pub predicate: AbstractExpression,
}

/// ValuesPlanNode represents rows of literal values.
///
/// For example, `INSERT INTO table VALUES ((0, 1), (1, 2))` produces
/// `(0, 1)` and `(1, 2)` as the output of this executor.
#[derive(Clone)]
pub struct ValuesPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The literal values produced by this plan node. Each inner vector
    /// represents one row of values.
    pub values: Vec<Vec<AbstractExpression>>,
}

/// ProjectionPlanNode represents a projection operation.
///
/// It computes expressions based on the input from its child.
#[derive(Clone)]
pub struct ProjectionPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The projection expressions to evaluate.
    pub expressions: Vec<AbstractExpression>,
}

/// SortPlanNode represents a sort operation.
///
/// It sorts the input tuples according to the given order-by expressions.
#[derive(Clone)]
pub struct SortPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The order-by expressions and their sort directions.
    pub order_bys: Vec<(OrderByType, AbstractExpression)>,
}

/// TopNPlanNode represents a top-N operation.
///
/// It retains only the N extreme rows based on the order-by expressions.
#[derive(Clone)]
pub struct TopNPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The order-by expressions and their sort directions.
    pub order_bys: Vec<(OrderByType, AbstractExpression)>,
    /// The maximum number of rows to retain.
    pub n: usize,
}

/// TopNPerGroupPlanNode represents a top-N per group operation.
#[derive(Clone)]
pub struct TopNPerGroupPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// The group-by expressions.
    pub group_bys: Vec<AbstractExpression>,
    /// The order-by expressions and their sort directions.
    pub order_bys: Vec<(OrderByType, AbstractExpression)>,
    /// The maximum number of rows to retain per group.
    pub n: usize,
}

/// MockScanPlanNode represents a dummy sequential scan over a table.
///
/// Unlike SeqScanPlanNode, this does not require the table to exist.
/// NOTE: This class is used solely for testing.
#[derive(Clone)]
pub struct MockScanPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The table name used to determine generated content.
    pub table: String,
}

/// WindowFunction represents a single window function inside WindowFunctionPlanNode.
#[derive(Clone)]
pub struct WindowFunction {
    /// The expression being aggregated.
    pub function: AbstractExpression,
    /// The type of window function.
    pub func_type: WindowFunctionType,
    /// The PARTITION BY expressions.
    pub partition_by: Vec<AbstractExpression>,
    /// The ORDER BY expressions and directions.
    pub order_by: Vec<(OrderByType, AbstractExpression)>,
}

/// WindowFunctionPlanNode represents window function operations.
///
/// Window aggregation is different from normal aggregation as it outputs one row
/// for each input row, and can be combined with normal selected columns.
///
/// For example, for query:
/// ```sql
/// SELECT 0.1, 0.2, SUM(0.3) OVER (PARTITION BY 0.2 ORDER BY 0.3),
///        SUM(0.4) OVER (PARTITION BY 0.1 ORDER BY 0.2, 0.3)
/// FROM table;
/// ```
///
/// The struct will contain:
/// - `columns`: all column expressions (including placeholders for window functions)
/// - `window_functions`: a map from column index to the window function definition
#[derive(Clone)]
pub struct WindowFunctionPlanNode {
    /// The output schema of this plan node.
    pub output_schema: SchemaRef,
    /// The child plans of this plan node.
    pub children: Vec<PlanNode>,
    /// All column expressions including placeholders for window functions.
    pub columns: Vec<AbstractExpression>,
    /// A map from column index to the window function definition.
    pub window_functions: HashMap<usize, WindowFunction>,
}

//===----------------------------------------------------------------------===//
// PlanNode enum (replaces AbstractPlanNode base class)
//===----------------------------------------------------------------------===//

/// PlanNode represents all possible types of plan nodes in the system.
///
/// Plan nodes are modeled as trees, so each plan node can have a variable
/// number of children. Per the Volcano model, the plan node receives the
/// tuples of its children. The ordering of the children may matter.
#[derive(Clone)]
pub enum PlanNode {
    SeqScan(SeqScanPlanNode),
    IndexScan(IndexScanPlanNode),
    Insert(InsertPlanNode),
    Update(UpdatePlanNode),
    Delete(DeletePlanNode),
    Aggregation(AggregationPlanNode),
    Limit(LimitPlanNode),
    NestedLoopJoin(NestedLoopJoinPlanNode),
    NestedIndexJoin(NestedIndexJoinPlanNode),
    HashJoin(HashJoinPlanNode),
    Filter(FilterPlanNode),
    Values(ValuesPlanNode),
    Projection(ProjectionPlanNode),
    Sort(SortPlanNode),
    TopN(TopNPlanNode),
    TopNPerGroup(TopNPerGroupPlanNode),
    MockScan(MockScanPlanNode),
    Window(WindowFunctionPlanNode),
}

//===----------------------------------------------------------------------===//
// PlanNode struct method implementations
//===----------------------------------------------------------------------===//

impl SeqScanPlanNode {
    /// Infer the scan schema from a table reference by prefixing each column
    /// name with the bound table name.
    pub fn infer_scan_schema(table: &BaseTableRef) -> Schema {
        let table_name = table.table.as_str();
        let columns = table.schema.columns.iter()
            .map(|c| Column::new_with_name(format!("{}.{}", table_name, c.get_name()).as_str(), c))
            .collect();
        Schema::new(columns)
    }
}

impl NestedLoopJoinPlanNode {
    /// Infer the join schema by concatenating the output schemas of the left
    /// and right child plan nodes.
    pub fn infer_join_schema(left: &PlanNode, right: &PlanNode) -> Schema {
        let left_columns = left.output_schema_ref().get_columns();
        let right_columns = right.output_schema_ref().get_columns();
        let mut schema = Vec::with_capacity(left_columns.len() + right_columns.len());
        for column in left_columns {
            schema.push(column.clone());
        }
        for column in right_columns {
            schema.push(column.clone());
        }
        Schema::new(schema)
    }
}

impl ProjectionPlanNode {
    /// Infer the projection schema from a list of expressions.
    pub fn infer_projection_schema(expressions: &[AbstractExpression]) -> Schema {
        let mut output = Vec::new();
        for expr in expressions {
            let return_type = expr.get_return_type();
            output.push(return_type.with_column_name("<unnamed>".to_string()));
        }
        Schema::new(output)
    }

    /// Rename the columns of a schema according to the provided column names.
    /// Panics if the number of column names does not match the schema's column count.
    pub fn rename_schema(schema: &Schema, col_names: &[String]) -> Schema {
        if col_names.len() != schema.get_column_count() {
            panic!("mismatched number of columns");
        }
        let mut output = Vec::with_capacity(col_names.len());
        for (idx, column) in schema.get_columns().iter().enumerate() {
            output.push(Column::new_with_name(&col_names[idx], column));
        }
        Schema::new(output)
    }
}

impl AggregationPlanNode {
    /// Infer the aggregation schema from the group-by and aggregate expressions.
    ///
    /// The output schema consists of the group-by columns followed by the
    /// aggregate columns (which are currently inferred as INTEGER).
    pub fn infer_agg_schema(
        group_bys: &[AbstractExpression],
        aggregates: &[AbstractExpression],
        _agg_types: &[AggregationType],
    ) -> Schema {
        let mut output = Vec::with_capacity(group_bys.len() + aggregates.len());
        for column in group_bys {
            let return_type = column.get_return_type();
            output.push(return_type.with_column_name("<unnamed>".to_string()));
        }
        for _idx in 0..aggregates.len() {
            // TODO(chi): correctly infer agg call return type
            output.push(Column::new("<unnamed>", TypeId::Integer));
        }
        Schema::new(output)
    }
}

impl WindowFunctionPlanNode {
    /// Infer the window function schema from the list of column expressions.
    pub fn infer_window_schema(columns: &[AbstractExpression]) -> Schema {
        let mut output = Vec::with_capacity(columns.len());
        // TODO(avery): correctly infer window call return type
        for column in columns {
            let return_type = column.get_return_type();
            output.push(return_type.with_column_name("<unnamed>".to_string()));
        }
        Schema::new(output)
    }
}

//===----------------------------------------------------------------------===//
// PlanNode method implementations
//===----------------------------------------------------------------------===//

impl PlanNode {
    /// Returns a reference to the output schema of this plan node.
    pub fn output_schema(&self) -> &SchemaRef {
        match self {
            PlanNode::SeqScan(n) => &n.output_schema,
            PlanNode::IndexScan(n) => &n.output_schema,
            PlanNode::Insert(n) => &n.output_schema,
            PlanNode::Update(n) => &n.output_schema,
            PlanNode::Delete(n) => &n.output_schema,
            PlanNode::Aggregation(n) => &n.output_schema,
            PlanNode::Limit(n) => &n.output_schema,
            PlanNode::NestedLoopJoin(n) => &n.output_schema,
            PlanNode::NestedIndexJoin(n) => &n.output_schema,
            PlanNode::HashJoin(n) => &n.output_schema,
            PlanNode::Filter(n) => &n.output_schema,
            PlanNode::Values(n) => &n.output_schema,
            PlanNode::Projection(n) => &n.output_schema,
            PlanNode::Sort(n) => &n.output_schema,
            PlanNode::TopN(n) => &n.output_schema,
            PlanNode::TopNPerGroup(n) => &n.output_schema,
            PlanNode::MockScan(n) => &n.output_schema,
            PlanNode::Window(n) => &n.output_schema,
        }
    }

    /// Returns a reference to the output schema of this plan node.
    pub fn output_schema_ref(&self) -> &Schema {
        match self {
            PlanNode::SeqScan(n) => &n.output_schema,
            PlanNode::IndexScan(n) => &n.output_schema,
            PlanNode::Insert(n) => &n.output_schema,
            PlanNode::Update(n) => &n.output_schema,
            PlanNode::Delete(n) => &n.output_schema,
            PlanNode::Aggregation(n) => &n.output_schema,
            PlanNode::Limit(n) => &n.output_schema,
            PlanNode::NestedLoopJoin(n) => &n.output_schema,
            PlanNode::NestedIndexJoin(n) => &n.output_schema,
            PlanNode::HashJoin(n) => &n.output_schema,
            PlanNode::Filter(n) => &n.output_schema,
            PlanNode::Values(n) => &n.output_schema,
            PlanNode::Projection(n) => &n.output_schema,
            PlanNode::Sort(n) => &n.output_schema,
            PlanNode::TopN(n) => &n.output_schema,
            PlanNode::TopNPerGroup(n) => &n.output_schema,
            PlanNode::MockScan(n) => &n.output_schema,
            PlanNode::Window(n) => &n.output_schema,
        }
    }

    /// Returns the children of this plan node.
    pub fn get_children(&self) -> &[PlanNode] {
        match self {
            PlanNode::SeqScan(_)
            | PlanNode::IndexScan(_)
            | PlanNode::Values(_)
            | PlanNode::MockScan(_) => &[],
            PlanNode::Insert(n) => &n.children,
            PlanNode::Update(n) => &n.children,
            PlanNode::Delete(n) => &n.children,
            PlanNode::Aggregation(n) => &n.children,
            PlanNode::Limit(n) => &n.children,
            PlanNode::NestedLoopJoin(n) => &n.children,
            PlanNode::NestedIndexJoin(n) => &n.children,
            PlanNode::HashJoin(n) => &n.children,
            PlanNode::Filter(n) => &n.children,
            PlanNode::Projection(n) => &n.children,
            PlanNode::Sort(n) => &n.children,
            PlanNode::TopN(n) => &n.children,
            PlanNode::TopNPerGroup(n) => &n.children,
            PlanNode::Window(n) => &n.children,
        }
    }

    /// Returns the child at the given index. Panics if out of bounds.
    pub fn get_child_at(&self, child_idx: usize) -> &PlanNode {
        &self.get_children()[child_idx]
    }

    /// Return the number of children.
    pub fn get_child_count(&self) -> usize {
        self.get_children().len()
    }
}

impl Display for PlanNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanNode::SeqScan(_) => write!(f, "SeqScan"),
            PlanNode::IndexScan(_) => write!(f, "IndexScan"),
            PlanNode::Insert(_) => write!(f, "Insert"),
            PlanNode::Update(_) => write!(f, "Update"),
            PlanNode::Delete(_) => write!(f, "Delete"),
            PlanNode::Aggregation(_) => write!(f, "Aggregation"),
            PlanNode::Limit(_) => write!(f, "Limit"),
            PlanNode::NestedLoopJoin(_) => write!(f, "NestedLoopJoin"),
            PlanNode::NestedIndexJoin(_) => write!(f, "NestedIndexJoin"),
            PlanNode::HashJoin(_) => write!(f, "HashJoin"),
            PlanNode::Filter(_) => write!(f, "Filter"),
            PlanNode::Values(_) => write!(f, "Values"),
            PlanNode::Projection(_) => write!(f, "Projection"),
            PlanNode::Sort(_) => write!(f, "Sort"),
            PlanNode::TopN(_) => write!(f, "TopN"),
            PlanNode::TopNPerGroup(_) => write!(f, "TopNPerGroup"),
            PlanNode::MockScan(_) => write!(f, "MockScan"),
            PlanNode::Window(_) => write!(f, "Window"),
        
        }
    }
}

//===----------------------------------------------------------------------===//
// PlanNode struct method implementations
//===----------------------------------------------------------------------===//

impl MockScanPlanNode {
    /// Returns a reference to the output schema of this plan node.
    pub fn output_schema_ref(&self) -> &Schema {
        &self.output_schema
    }
}

//===----------------------------------------------------------------------===//
// Helper impls for AggregateKey
//===----------------------------------------------------------------------===//

impl AggregateKey {
    /// Compares two aggregate keys for equality by comparing each group-by value.
    pub fn equals(&self, other: &AggregateKey) -> bool {
        if self.group_bys.len() != other.group_bys.len() {
            return false;
        }
        for i in 0..self.group_bys.len() {
            if self.group_bys[i].compare_equals(&other.group_bys[i]) != CmpBool::CmpTrue {
                return false;
            }
        }
        true
    }
}

use crate::sql_type::sql_type::CmpBool;
