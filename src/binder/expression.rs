use crate::{binder::BindError, sql_type::Value};
use std::fmt;

//===----------------------------------------------------------------------===//
// WindowBoundary
//===----------------------------------------------------------------------===//

/// Window boundary types for window functions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowBoundary {
    Invalid,
    UnboundedPreceding,
    UnboundedFollowing,
    CurrentRowRange,
    CurrentRowRows,
    ExprPrecedingRows,
    ExprFollowingRows,
    ExprPrecedingRange,
    ExprFollowingRange,
}

impl fmt::Display for WindowBoundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowBoundary::Invalid => write!(f, "INVALID"),
            WindowBoundary::UnboundedPreceding => write!(f, "UNBOUNDED PRECEDING"),
            WindowBoundary::UnboundedFollowing => write!(f, "UNBOUNDED FOLLOWING"),
            WindowBoundary::CurrentRowRange => write!(f, "CURRENT ROW RANGE"),
            WindowBoundary::CurrentRowRows => write!(f, "CURRENT ROW ROWS"),
            WindowBoundary::ExprPrecedingRows => write!(f, "EXPR PRECEDING ROWS"),
            WindowBoundary::ExprFollowingRows => write!(f, "EXPR FOLLOWING ROWS"),
            WindowBoundary::ExprPrecedingRange => write!(f, "EXPR PRECEDING RANGE"),
            WindowBoundary::ExprFollowingRange => write!(f, "EXPR FOLLOWING RANGE"),
        }
    }
}

//===----------------------------------------------------------------------===//
// OrderByType & BoundOrderBy
//===----------------------------------------------------------------------===//

/// All types of order-bys in binder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderByType {
    Invalid,
    Default,
    Asc,
    Desc,
}

impl fmt::Display for OrderByType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderByType::Invalid => write!(f, "Invalid"),
            OrderByType::Default => write!(f, "Default"),
            OrderByType::Asc => write!(f, "Ascending"),
            OrderByType::Desc => write!(f, "Descending"),
        }
    }
}

/// BoundOrderBy is an item in the ORDER BY clause.
#[derive(Debug, Clone)]
pub struct BoundOrderBy {
    pub order_by_type: OrderByType,
    pub expr: Box<BoundExpression>,
}

impl fmt::Display for BoundOrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoundOrderBy {{ type={}, expr={} }}", self.order_by_type, self.expr)
    }
}

//===----------------------------------------------------------------------===//
// BoundExpression (enum)
//===----------------------------------------------------------------------===//

/// All types of bound expressions.
#[derive(Clone)]
pub enum BoundExpression {
    BoundColumnRef(BoundColumnRef),
    BoundConstant(BoundConstant),
    BoundStar,
    BoundAlias(BoundAlias),
    BoundBinaryOp(BoundBinaryOp),
    BoundUnaryOp(BoundUnaryOp),
    BoundFuncCall(BoundFuncCall),
    BoundAggCall(BoundAggCall),
    BoundWindow(BoundWindow),
}

impl BoundExpression {
    /// Returns true if this expression contains an aggregation function.
    pub fn has_aggregation(&self) -> bool {
        match self {
            BoundExpression::BoundAggCall(_) => true,
            BoundExpression::BoundAlias(alias) => alias.child.has_aggregation(),
            BoundExpression::BoundBinaryOp(op) => op.larg.has_aggregation() || op.rarg.has_aggregation(),
            BoundExpression::BoundUnaryOp(op) => op.arg.has_aggregation(),
            _ => false,
        }
    }

    /// Returns true if this expression contains a window function.
    pub fn has_window_function(&self) -> bool {
        match self {
            BoundExpression::BoundWindow(_) => true,
            BoundExpression::BoundAlias(alias) => alias.child.has_window_function(),
            _ => false,
        }
    }
}

impl fmt::Display for BoundExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoundExpression::BoundColumnRef(inner) => fmt::Display::fmt(inner, f),
            BoundExpression::BoundConstant(inner) => fmt::Display::fmt(inner, f),
            BoundExpression::BoundStar => write!(f, "*"),
            BoundExpression::BoundAlias(inner) => fmt::Display::fmt(inner, f),
            BoundExpression::BoundBinaryOp(inner) => fmt::Display::fmt(inner, f),
            BoundExpression::BoundUnaryOp(inner) => fmt::Display::fmt(inner, f),
            BoundExpression::BoundFuncCall(inner) => fmt::Display::fmt(inner, f),
            BoundExpression::BoundAggCall(inner) => fmt::Display::fmt(inner, f),
            BoundExpression::BoundWindow(inner) => fmt::Display::fmt(inner, f),
        }
    }
}

// Manual Debug impl since `Value` does not implement Debug
impl fmt::Debug for BoundExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoundExpression::BoundColumnRef(v) => f.debug_tuple("BoundColumnRef").field(v).finish(),
            BoundExpression::BoundConstant(v) => f.debug_tuple("BoundConstant").field(v).finish(),
            BoundExpression::BoundStar => write!(f, "BoundStar"),
            BoundExpression::BoundAlias(v) => f.debug_tuple("BoundAlias").field(v).finish(),
            BoundExpression::BoundBinaryOp(v) => f.debug_tuple("BoundBinaryOp").field(v).finish(),
            BoundExpression::BoundUnaryOp(v) => f.debug_tuple("BoundUnaryOp").field(v).finish(),
            BoundExpression::BoundFuncCall(v) => f.debug_tuple("BoundFuncCall").field(v).finish(),
            BoundExpression::BoundAggCall(v) => f.debug_tuple("BoundAggCall").field(v).finish(),
            BoundExpression::BoundWindow(v) => f.debug_tuple("BoundWindow").field(v).finish(),
        }
    }
}

//===----------------------------------------------------------------------===//
// BoundColumnRef
//===----------------------------------------------------------------------===//

/// A bound column reference, e.g., `y.x` in the SELECT list.
#[derive(Debug, Clone)]
pub struct BoundColumnRef {
    pub col_names: Vec<String>,
}

impl BoundColumnRef {
    /// Prepend a prefix to the column name.
    pub fn prepend(self: Box<Self>, prefix: String) -> Box<BoundColumnRef> {
        let mut col_name = vec![prefix];
        col_name.extend(self.col_names);
        Box::new(BoundColumnRef { col_names: col_name })
    }
}

impl fmt::Display for BoundColumnRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.col_names.join("."))
    }
}

//===----------------------------------------------------------------------===//
// BoundConstant
//===----------------------------------------------------------------------===//

/// A bound constant, e.g., `1`.
pub struct BoundConstant {
    pub val: Value,
}

// Manual Debug impl since `Value` does not implement Debug
impl fmt::Debug for BoundConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoundConstant")
            .field("val", &self.val.to_string())
            .finish()
    }
}

impl Clone for BoundConstant {
    fn clone(&self) -> Self {
        BoundConstant { val: self.val.clone() }
    }
}

impl fmt::Display for BoundConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.val, f)
    }
}

//===----------------------------------------------------------------------===//
// BoundAlias
//===----------------------------------------------------------------------===//

/// The alias in SELECT list, e.g. `SELECT count(x) AS y`, the `y` is an alias.
#[derive(Debug, Clone)]
pub struct BoundAlias {
    pub alias: String,
    pub child: Box<BoundExpression>,
}

impl fmt::Display for BoundAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} as {})", self.child, self.alias)
    }
}

//===----------------------------------------------------------------------===//
// BoundBinaryOp
//===----------------------------------------------------------------------===//

#[derive(Debug, Clone, Copy)]
pub enum BoundBinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    StringConcat,
    Gt,
    Lt,
    GtEq,
    LtEq,
    Spaceship,
    Eq,
    NotEq,
    And,
    Or,
}

/// A bound binary operator, e.g., `a+b`.
#[derive(Debug, Clone)]
pub struct BoundBinaryOp {
    pub op: BoundBinaryOperator,
    pub larg: Box<BoundExpression>,
    pub rarg: Box<BoundExpression>,
}

impl fmt::Display for BoundBinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}{:?}{})", self.larg, self.op, self.rarg)
    }
}

impl BoundBinaryOperator {
    pub fn from(op: &sqlparser::ast::BinaryOperator) -> Result<Self, BindError> {
        type BiOp = sqlparser::ast::BinaryOperator;
        match op {
            BiOp::Plus => Ok(Self::Plus),
            BiOp::Minus => Ok(Self::Minus),
            BiOp::Multiply => Ok(Self::Multiply),
            BiOp::Divide => Ok(Self::Divide),
            BiOp::Modulo => Ok(Self::Modulo),
            BiOp::StringConcat => Ok(Self::StringConcat),
            BiOp::Gt => Ok(Self::Gt),
            BiOp::Lt => Ok(Self::Lt),
            BiOp::GtEq => Ok(Self::GtEq),
            BiOp::LtEq => Ok(Self::LtEq),
            BiOp::Spaceship => Ok(Self::Spaceship),
            BiOp::Eq => Ok(Self::Eq),
            BiOp::NotEq => Ok(Self::NotEq),
            BiOp::And => Ok(Self::And),
            BiOp::Or => Ok(Self::Or),
            _ => Err(BindError::UnsupportedBinaryOperator(format!("{:?}", op)))
        }
    }
}

//===----------------------------------------------------------------------===//
// BoundUnaryOp
//===----------------------------------------------------------------------===//

/// A bound unary operation, e.g., `-x`.
#[derive(Debug, Clone)]
pub struct BoundUnaryOp {
    pub op_name: String,
    pub arg: Box<BoundExpression>,
}

impl fmt::Display for BoundUnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}{})", self.op_name, self.arg)
    }
}

//===----------------------------------------------------------------------===//
// BoundFuncCall
//===----------------------------------------------------------------------===//

/// A bound func call, e.g., `lower(x)`.
#[derive(Debug, Clone)]
pub struct BoundFuncCall {
    pub func_name: String,
    pub args: Vec<BoundExpression>,
}

impl fmt::Display for BoundFuncCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args_str: Vec<String> = self.args.iter().map(|a| a.to_string()).collect();
        write!(f, "{}({})", self.func_name, args_str.join(", "))
    }
}

//===----------------------------------------------------------------------===//
// BoundAggCall
//===----------------------------------------------------------------------===//

/// A bound aggregate call, e.g., `sum(x)`.
#[derive(Debug, Clone)]
pub struct BoundAggCall {
    pub func_name: String,
    pub is_distinct: bool,
    pub args: Vec<BoundExpression>,
}

impl fmt::Display for BoundAggCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args_str: Vec<String> = self.args.iter().map(|a| a.to_string()).collect();
        if self.is_distinct {
            write!(f, "{}_distinct({})", self.func_name, args_str.join(", "))
        } else {
            write!(f, "{}({})", self.func_name, args_str.join(", "))
        }
    }
}

//===----------------------------------------------------------------------===//
// BoundWindow
//===----------------------------------------------------------------------===//

/// A bound window aggregation, e.g., `sum(x) OVER (...)`.
#[derive(Debug, Clone)]
pub struct BoundWindow {
    pub func_name: String,
    pub args: Vec<BoundExpression>,
    pub partition_by: Vec<BoundExpression>,
    pub order_bys: Vec<BoundOrderBy>,
    pub start_offset: Option<Box<BoundExpression>>,
    pub end_offset: Option<Box<BoundExpression>>,
    pub start: WindowBoundary,
    pub end: WindowBoundary,
}

impl BoundWindow {
    pub fn set_start(&mut self, start: WindowBoundary) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: WindowBoundary) {
        self.end = end;
    }
}

impl fmt::Display for BoundWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args_str: Vec<String> = self.args.iter().map(|a| a.to_string()).collect();
        let partition_by_str: Vec<String> = self.partition_by.iter().map(|e| e.to_string()).collect();
        let order_bys_str: Vec<String> = self.order_bys.iter().map(|e| e.to_string()).collect();

        write!(
            f,
            "{}({}) Over {{\n  partition_by=[{}],\n  order_by=[{}]\n}}",
            self.func_name,
            args_str.join(", "),
            partition_by_str.join(", "),
            order_bys_str.join(", "),
        )
    }
}
