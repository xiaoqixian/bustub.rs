//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// expressions.rs
//
// Identification: src/execution/expressions.rs
//
// Copyright (c) 2015-2023, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::fmt;

use crate::catalog::Column;
use crate::catalog::Schema;
use crate::sql_type::limits::BUSTUB_BOOLEAN_NULL;
use crate::sql_type::sql_type::CmpBool;
use crate::sql_type::type_id::TypeId;
use crate::sql_type::value::Value;
use crate::sql_type::value_factory::ValueFactory;
use crate::storage::table::tuple::Tuple;

//===----------------------------------------------------------------------===//
// Expression sub-type enums
//===----------------------------------------------------------------------===//

/// Arithmetic operation type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArithmeticType {
    Plus,
    Minus,
}

/// Comparison operation type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComparisonType {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

/// Logic operation type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicType {
    And,
    Or,
}

/// String expression operation type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StringExpressionType {
    Lower,
    Upper,
}

//===----------------------------------------------------------------------===//
// Concrete expression structs (data holders for each enum variant)
//===----------------------------------------------------------------------===//

/// A constant value expression wrapping a single Value.
#[derive(Clone)]
pub struct ConstantValueExpression {
    pub val: Value,
}

/// A column value expression referencing a column by tuple index and column index.
#[derive(Clone)]
pub struct ColumnValueExpression {
    /// Tuple index (0 = left side of join, 1 = right side of join).
    pub tuple_idx: usize,
    /// Column index within the schema.
    pub col_idx: usize,
    /// The return type of this expression.
    pub ret_type: Column,
}

/// A comparison expression comparing two child expressions.
pub struct ComparisonExpression {
    pub comp_type: ComparisonType,
    pub left: Box<AbstractExpression>,
    pub right: Box<AbstractExpression>,
}

/// An arithmetic expression performing arithmetic on two child expressions.
pub struct ArithmeticExpression {
    pub compute_type: ArithmeticType,
    pub left: Box<AbstractExpression>,
    pub right: Box<AbstractExpression>,
}

/// A logic expression performing logical operations on two child expressions.
pub struct LogicExpression {
    pub logic_type: LogicType,
    pub left: Box<AbstractExpression>,
    pub right: Box<AbstractExpression>,
}

/// A string expression applying a string operation (lower/upper) on a child expression.
pub struct StringExpression {
    pub expr_type: StringExpressionType,
    pub arg: Box<AbstractExpression>,
}

/// An array expression representing an array of values from its children.
pub struct ArrayExpression {
    pub children: Vec<AbstractExpression>,
}

//===----------------------------------------------------------------------===//
// AbstractExpression enum
//===----------------------------------------------------------------------===//

/// AbstractExpression is the base representation of all expressions in the system.
/// Expressions are modeled as trees, i.e., every expression may have a variable
/// number of children.
pub enum AbstractExpression {
    ConstantValue(ConstantValueExpression),
    ColumnValue(ColumnValueExpression),
    Comparison(ComparisonExpression),
    Arithmetic(ArithmeticExpression),
    Logic(LogicExpression),
    String(StringExpression),
    Array(ArrayExpression),
}

//===----------------------------------------------------------------------===//
// Manual Clone implementations for expression structs containing AbstractExpression
//===----------------------------------------------------------------------===//

impl Clone for ComparisonExpression {
    fn clone(&self) -> Self {
        ComparisonExpression {
            comp_type: self.comp_type,
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

impl Clone for ArithmeticExpression {
    fn clone(&self) -> Self {
        ArithmeticExpression {
            compute_type: self.compute_type,
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

impl Clone for LogicExpression {
    fn clone(&self) -> Self {
        LogicExpression {
            logic_type: self.logic_type,
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

impl Clone for StringExpression {
    fn clone(&self) -> Self {
        StringExpression {
            expr_type: self.expr_type,
            arg: self.arg.clone(),
        }
    }
}

impl Clone for ArrayExpression {
    fn clone(&self) -> Self {
        ArrayExpression {
            children: self.children.clone(),
        }
    }
}

impl Clone for AbstractExpression {
    fn clone(&self) -> Self {
        match self {
            AbstractExpression::ConstantValue(inner) => {
                AbstractExpression::ConstantValue(inner.clone())
            }
            AbstractExpression::ColumnValue(inner) => {
                AbstractExpression::ColumnValue(inner.clone())
            }
            AbstractExpression::Comparison(inner) => {
                AbstractExpression::Comparison(inner.clone())
            }
            AbstractExpression::Arithmetic(inner) => {
                AbstractExpression::Arithmetic(inner.clone())
            }
            AbstractExpression::Logic(inner) => {
                AbstractExpression::Logic(inner.clone())
            }
            AbstractExpression::String(inner) => {
                AbstractExpression::String(inner.clone())
            }
            AbstractExpression::Array(inner) => {
                AbstractExpression::Array(inner.clone())
            }
        }
    }
}

//===----------------------------------------------------------------------===//
// Helper: convert a CmpBool to a boolean Value
//===----------------------------------------------------------------------===//

/// Convert a three-valued comparison result into a boolean Value.
fn cmp_bool_to_value(result: CmpBool) -> Value {
    match result {
        CmpBool::CmpTrue => Value::from_i8(TypeId::Boolean, 1),
        CmpBool::CmpFalse => Value::from_i8(TypeId::Boolean, 0),
        CmpBool::CmpNull => Value::from_i8(TypeId::Boolean, BUSTUB_BOOLEAN_NULL),
    }
}

//===----------------------------------------------------------------------===//
// AbstractExpression method implementations
//===----------------------------------------------------------------------===//

impl AbstractExpression {
    /// Evaluate this expression against a single tuple.
    pub fn evaluate(&self, tuple: &Tuple, schema: &Schema) -> Value {
        match self {
            AbstractExpression::ConstantValue(inner) => inner.val.clone(),
            AbstractExpression::ColumnValue(inner) => tuple.get_value(schema, inner.col_idx),
            AbstractExpression::Comparison(inner) => {
                let lhs = inner.left.evaluate(tuple, schema);
                let rhs = inner.right.evaluate(tuple, schema);
                let result = match inner.comp_type {
                    ComparisonType::Equal => lhs.compare_equals(&rhs),
                    ComparisonType::NotEqual => lhs.compare_not_equals(&rhs),
                    ComparisonType::LessThan => lhs.compare_less_than(&rhs),
                    ComparisonType::LessThanOrEqual => lhs.compare_less_than_equals(&rhs),
                    ComparisonType::GreaterThan => lhs.compare_greater_than(&rhs),
                    ComparisonType::GreaterThanOrEqual => lhs.compare_greater_than_equals(&rhs),
                };
                cmp_bool_to_value(result)
            }
            AbstractExpression::Arithmetic(inner) => {
                let lhs = inner.left.evaluate(tuple, schema);
                let rhs = inner.right.evaluate(tuple, schema);
                if lhs.is_null() || rhs.is_null() {
                    return ValueFactory::get_null_value_by_type(TypeId::Integer);
                }
                let l = lhs.get_as::<i32>();
                let r = rhs.get_as::<i32>();
                match inner.compute_type {
                    ArithmeticType::Plus => ValueFactory::get_integer_value(l + r),
                    ArithmeticType::Minus => ValueFactory::get_integer_value(l - r),
                }
            }
            AbstractExpression::Logic(inner) => {
                let lhs = inner.left.evaluate(tuple, schema);
                let rhs = inner.right.evaluate(tuple, schema);
                let result = perform_logic(lhs, rhs, inner.logic_type);
                cmp_bool_to_value(result)
            }
            AbstractExpression::String(inner) => {
                let val = inner.arg.evaluate(tuple, schema);
                let str_val = val.to_string_val();
                let result = compute_string(&str_val, inner.expr_type);
                ValueFactory::get_varchar_value(&result)
            }
            AbstractExpression::Array(inner) => {
                let mut values = Vec::with_capacity(inner.children.len());
                for child in &inner.children {
                    let val = child.evaluate(tuple, schema);
                    // The C++ code checks for DECIMAL type; here we get f64 directly.
                    values.push(val.get_as::<f64>());
                }
                // Construct a Value from the collected f64 values.
                // Since the Rust project does not have a dedicated Vector type,
                // we store the f64 data as raw bytes in a Varchar value.
                let packed = pack_f64_values(&values);
                Value::from_bytes(TypeId::Varchar, &packed, packed.len() as u32, true)
            }
        }
    }

    /// Evaluate this expression in the context of a join between two tuples.
    pub fn evaluate_join(
        &self,
        left_tuple: &Tuple,
        left_schema: &Schema,
        right_tuple: &Tuple,
        right_schema: &Schema,
    ) -> Value {
        match self {
            AbstractExpression::ConstantValue(inner) => inner.val.clone(),
            AbstractExpression::ColumnValue(inner) => {
                if inner.tuple_idx == 0 {
                    left_tuple.get_value(left_schema, inner.col_idx)
                } else {
                    right_tuple.get_value(right_schema, inner.col_idx)
                }
            }
            AbstractExpression::Comparison(inner) => {
                let lhs = inner
                    .left
                    .evaluate_join(left_tuple, left_schema, right_tuple, right_schema);
                let rhs = inner
                    .right
                    .evaluate_join(left_tuple, left_schema, right_tuple, right_schema);
                let result = match inner.comp_type {
                    ComparisonType::Equal => lhs.compare_equals(&rhs),
                    ComparisonType::NotEqual => lhs.compare_not_equals(&rhs),
                    ComparisonType::LessThan => lhs.compare_less_than(&rhs),
                    ComparisonType::LessThanOrEqual => lhs.compare_less_than_equals(&rhs),
                    ComparisonType::GreaterThan => lhs.compare_greater_than(&rhs),
                    ComparisonType::GreaterThanOrEqual => lhs.compare_greater_than_equals(&rhs),
                };
                cmp_bool_to_value(result)
            }
            AbstractExpression::Arithmetic(inner) => {
                let lhs = inner
                    .left
                    .evaluate_join(left_tuple, left_schema, right_tuple, right_schema);
                let rhs = inner
                    .right
                    .evaluate_join(left_tuple, left_schema, right_tuple, right_schema);
                if lhs.is_null() || rhs.is_null() {
                    return ValueFactory::get_null_value_by_type(TypeId::Integer);
                }
                let l = lhs.get_as::<i32>();
                let r = rhs.get_as::<i32>();
                match inner.compute_type {
                    ArithmeticType::Plus => ValueFactory::get_integer_value(l + r),
                    ArithmeticType::Minus => ValueFactory::get_integer_value(l - r),
                }
            }
            AbstractExpression::Logic(inner) => {
                let lhs = inner
                    .left
                    .evaluate_join(left_tuple, left_schema, right_tuple, right_schema);
                let rhs = inner
                    .right
                    .evaluate_join(left_tuple, left_schema, right_tuple, right_schema);
                let result = perform_logic(lhs, rhs, inner.logic_type);
                cmp_bool_to_value(result)
            }
            AbstractExpression::String(inner) => {
                let val = inner
                    .arg
                    .evaluate_join(left_tuple, left_schema, right_tuple, right_schema);
                let str_val = val.to_string_val();
                let result = compute_string(&str_val, inner.expr_type);
                ValueFactory::get_varchar_value(&result)
            }
            AbstractExpression::Array(inner) => {
                let mut values = Vec::with_capacity(inner.children.len());
                for child in &inner.children {
                    let val =
                        child.evaluate_join(left_tuple, left_schema, right_tuple, right_schema);
                    values.push(val.get_as::<f64>());
                }
                let packed = pack_f64_values(&values);
                Value::from_bytes(TypeId::Varchar, &packed, packed.len() as u32, true)
            }
        }
    }

    /// Get the return type of this expression.
    pub fn get_return_type(&self) -> Column {
        match self {
            AbstractExpression::ConstantValue(inner) => value_to_column(&inner.val),
            AbstractExpression::ColumnValue(inner) => inner.ret_type.clone(),
            AbstractExpression::Comparison(_) => {
                Column::new("<val>", TypeId::Boolean)
            }
            AbstractExpression::Arithmetic(_) => {
                Column::new("<val>", TypeId::Integer)
            }
            AbstractExpression::Logic(_) => {
                Column::new("<val>", TypeId::Boolean)
            }
            AbstractExpression::String(_) => {
                Column::new_with_length("<val>", TypeId::Varchar, 256)
            }
            AbstractExpression::Array(inner) => {
                // The C++ code creates a VECTOR type with the number of children as length.
                // Since the Rust project does not have a Vector type in TypeId yet,
                // we use Varchar as a placeholder.
                Column::new_with_length("<val>", TypeId::Varchar, inner.children.len())
            }
        }
    }

    /// Get the number of children of this expression.
    pub fn get_child_count(&self) -> usize {
        match self {
            AbstractExpression::ConstantValue(_) => 0,
            AbstractExpression::ColumnValue(_) => 0,
            AbstractExpression::Comparison(_) => 2,
            AbstractExpression::Arithmetic(_) => 2,
            AbstractExpression::Logic(_) => 2,
            AbstractExpression::String(_) => 1,
            AbstractExpression::Array(inner) => inner.children.len(),
        }
    }

    /// Get the child at the given index. Panics if the index is out of bounds.
    pub fn get_child_at(&self, child_idx: usize) -> &AbstractExpression {
        match self {
            AbstractExpression::Comparison(inner) => match child_idx {
                0 => &inner.left,
                1 => &inner.right,
                _ => panic!("child index out of bounds"),
            },
            AbstractExpression::Arithmetic(inner) => match child_idx {
                0 => &inner.left,
                1 => &inner.right,
                _ => panic!("child index out of bounds"),
            },
            AbstractExpression::Logic(inner) => match child_idx {
                0 => &inner.left,
                1 => &inner.right,
                _ => panic!("child index out of bounds"),
            },
            AbstractExpression::String(inner) => match child_idx {
                0 => &inner.arg,
                _ => panic!("child index out of bounds"),
            },
            AbstractExpression::Array(inner) => &inner.children[child_idx],
            _ => panic!("this expression has no children"),
        }
    }

    /// Get all children of this expression.
    pub fn get_children(&self) -> Vec<&AbstractExpression> {
        match self {
            AbstractExpression::Comparison(inner) => {
                vec![inner.left.as_ref(), inner.right.as_ref()]
            }
            AbstractExpression::Arithmetic(inner) => {
                vec![inner.left.as_ref(), inner.right.as_ref()]
            }
            AbstractExpression::Logic(inner) => {
                vec![inner.left.as_ref(), inner.right.as_ref()]
            }
            AbstractExpression::String(inner) => {
                vec![inner.arg.as_ref()]
            }
            AbstractExpression::Array(inner) => {
                inner.children.iter().collect()
            }
            _ => vec![],
        }
    }

    /// Create a new expression with the same configuration but different children.
    pub fn clone_with_children(&self, children: Vec<AbstractExpression>) -> AbstractExpression {
        match self {
            AbstractExpression::ConstantValue(_inner) => {
                AbstractExpression::ConstantValue(ConstantValueExpression {
                    val: _inner.val.clone(),
                })
            }
            AbstractExpression::ColumnValue(_inner) => {
                AbstractExpression::ColumnValue(ColumnValueExpression {
                    tuple_idx: _inner.tuple_idx,
                    col_idx: _inner.col_idx,
                    ret_type: _inner.ret_type.clone(),
                })
            }
            AbstractExpression::Comparison(inner) => {
                let mut it = children.into_iter();
                AbstractExpression::Comparison(ComparisonExpression {
                    comp_type: inner.comp_type,
                    left: Box::new(it.next().expect("ComparisonExpression requires 2 children")),
                    right: Box::new(it.next().expect("ComparisonExpression requires 2 children")),
                })
            }
            AbstractExpression::Arithmetic(inner) => {
                let mut it = children.into_iter();
                AbstractExpression::Arithmetic(ArithmeticExpression {
                    compute_type: inner.compute_type,
                    left: Box::new(it.next().expect("ArithmeticExpression requires 2 children")),
                    right: Box::new(it.next().expect("ArithmeticExpression requires 2 children")),
                })
            }
            AbstractExpression::Logic(inner) => {
                let mut it = children.into_iter();
                AbstractExpression::Logic(LogicExpression {
                    logic_type: inner.logic_type,
                    left: Box::new(it.next().expect("LogicExpression requires 2 children")),
                    right: Box::new(it.next().expect("LogicExpression requires 2 children")),
                })
            }
            AbstractExpression::String(inner) => {
                let mut it = children.into_iter();
                AbstractExpression::String(StringExpression {
                    expr_type: inner.expr_type,
                    arg: Box::new(it.next().expect("StringExpression requires 1 child")),
                })
            }
            AbstractExpression::Array(_inner) => {
                AbstractExpression::Array(ArrayExpression {
                    children,
                })
            }
        }
    }
}

//===----------------------------------------------------------------------===//
// Helper functions
//===----------------------------------------------------------------------===//

/// Pack a slice of f64 values into a byte vector for storage in a Value.
fn pack_f64_values(values: &[f64]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 8);
    for v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

/// Perform a logic computation (And/Or) with three-valued logic.
fn perform_logic(lhs: Value, rhs: Value, logic_type: LogicType) -> CmpBool {
    let l = value_to_cmp_bool(&lhs);
    let r = value_to_cmp_bool(&rhs);
    match logic_type {
        LogicType::And => {
            if l == CmpBool::CmpFalse || r == CmpBool::CmpFalse {
                return CmpBool::CmpFalse;
            }
            if l == CmpBool::CmpTrue && r == CmpBool::CmpTrue {
                return CmpBool::CmpTrue;
            }
            CmpBool::CmpNull
        }
        LogicType::Or => {
            if l == CmpBool::CmpFalse && r == CmpBool::CmpFalse {
                return CmpBool::CmpFalse;
            }
            if l == CmpBool::CmpTrue || r == CmpBool::CmpTrue {
                return CmpBool::CmpTrue;
            }
            CmpBool::CmpNull
        }
    }
}

/// Convert a Value to a CmpBool, handling null as CmpNull.
fn value_to_cmp_bool(val: &Value) -> CmpBool {
    if val.is_null() {
        return CmpBool::CmpNull;
    }
    let b = val.get_as::<i8>();
    if b != 0 {
        CmpBool::CmpTrue
    } else {
        CmpBool::CmpFalse
    }
}

/// Compute a string operation (lower or upper) on the input string.
/// Note: The C++ Compute method is a TODO stub that returns an empty string.
fn compute_string(input: &str, expr_type: StringExpressionType) -> String {
    match expr_type {
        StringExpressionType::Lower => input.to_lowercase(),
        StringExpressionType::Upper => input.to_uppercase(),
    }
}

/// Convert a Value to a Column by extracting its type information.
fn value_to_column(val: &Value) -> Column {
    let type_id = val.get_type_id();
    if type_id == TypeId::Varchar {
        Column::new_with_length("<val>", TypeId::Varchar, 256)
    } else {
        Column::new("<val>", type_id)
    }
}

//===----------------------------------------------------------------------===//
// Constructor helpers
//===----------------------------------------------------------------------===//

impl AbstractExpression {
    /// Create a new constant value expression.
    pub fn constant_value(val: Value) -> AbstractExpression {
        AbstractExpression::ConstantValue(ConstantValueExpression { val })
    }

    /// Create a new column value expression.
    pub fn column_value(tuple_idx: usize, col_idx: usize, ret_type: Column) -> AbstractExpression {
        AbstractExpression::ColumnValue(ColumnValueExpression {
            tuple_idx,
            col_idx,
            ret_type,
        })
    }

    /// Create a new comparison expression.
    pub fn comparison(comp_type: ComparisonType, left: AbstractExpression, right: AbstractExpression) -> AbstractExpression {
        AbstractExpression::Comparison(ComparisonExpression {
            comp_type,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    /// Create a new arithmetic expression.
    /// Panics if either child's return type is not INTEGER.
    pub fn arithmetic(compute_type: ArithmeticType, left: AbstractExpression, right: AbstractExpression) -> AbstractExpression {
        assert_eq!(
            left.get_return_type().get_type(),
            TypeId::Integer,
            "ArithmeticExpression only supports INTEGER operands"
        );
        assert_eq!(
            right.get_return_type().get_type(),
            TypeId::Integer,
            "ArithmeticExpression only supports INTEGER operands"
        );
        AbstractExpression::Arithmetic(ArithmeticExpression {
            compute_type,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    /// Create a new logic expression.
    /// Panics if either child's return type is not BOOLEAN.
    pub fn logic(logic_type: LogicType, left: AbstractExpression, right: AbstractExpression) -> AbstractExpression {
        assert_eq!(
            left.get_return_type().get_type(),
            TypeId::Boolean,
            "LogicExpression expects BOOLEAN operands"
        );
        assert_eq!(
            right.get_return_type().get_type(),
            TypeId::Boolean,
            "LogicExpression expects BOOLEAN operands"
        );
        AbstractExpression::Logic(LogicExpression {
            logic_type,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    /// Create a new string expression.
    /// Panics if the child's return type is not VARCHAR.
    pub fn string(expr_type: StringExpressionType, arg: AbstractExpression) -> AbstractExpression {
        assert_eq!(
            arg.get_return_type().get_type(),
            TypeId::Varchar,
            "StringExpression expects a VARCHAR operand"
        );
        AbstractExpression::String(StringExpression {
            expr_type,
            arg: Box::new(arg),
        })
    }

    /// Create a new array expression from a list of children.
    pub fn array(children: Vec<AbstractExpression>) -> AbstractExpression {
        AbstractExpression::Array(ArrayExpression {
            children,
        })
    }
}

//===----------------------------------------------------------------------===//
// Display implementations
//===----------------------------------------------------------------------===//

impl fmt::Display for ArithmeticType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticType::Plus => write!(f, "+"),
            ArithmeticType::Minus => write!(f, "-"),
        }
    }
}

impl fmt::Display for ComparisonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonType::Equal => write!(f, "="),
            ComparisonType::NotEqual => write!(f, "!="),
            ComparisonType::LessThan => write!(f, "<"),
            ComparisonType::LessThanOrEqual => write!(f, "<="),
            ComparisonType::GreaterThan => write!(f, ">"),
            ComparisonType::GreaterThanOrEqual => write!(f, ">="),
        }
    }
}

impl fmt::Display for LogicType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicType::And => write!(f, "and"),
            LogicType::Or => write!(f, "or"),
        }
    }
}

impl fmt::Display for StringExpressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringExpressionType::Lower => write!(f, "lower"),
            StringExpressionType::Upper => write!(f, "upper"),
        }
    }
}

impl fmt::Display for AbstractExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AbstractExpression::ConstantValue(inner) => {
                write!(f, "{}", inner.val)
            }
            AbstractExpression::ColumnValue(inner) => {
                write!(f, "#{}.{}", inner.tuple_idx, inner.col_idx)
            }
            AbstractExpression::Comparison(inner) => {
                write!(f, "({}{}{})", inner.left, inner.comp_type, inner.right)
            }
            AbstractExpression::Arithmetic(inner) => {
                write!(f, "({}{}{})", inner.left, inner.compute_type, inner.right)
            }
            AbstractExpression::Logic(inner) => {
                write!(f, "({}{}{})", inner.left, inner.logic_type, inner.right)
            }
            AbstractExpression::String(inner) => {
                write!(f, "{}({})", inner.expr_type, inner.arg)
            }
            AbstractExpression::Array(inner) => {
                let parts: Vec<String> = inner.children.iter().map(|c| c.to_string()).collect();
                write!(f, "[{}]", parts.join(","))
            }
        }
    }
}

impl fmt::Debug for AbstractExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Debug for ConstantValueExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConstantValue({})", self.val)
    }
}

impl fmt::Debug for ColumnValueExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ColumnValue(#{}..{})", self.tuple_idx, self.col_idx)
    }
}

impl fmt::Debug for ComparisonExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Comparison({} {} {})", self.left, self.comp_type, self.right)
    }
}

impl fmt::Debug for ArithmeticExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Arithmetic({} {} {})", self.left, self.compute_type, self.right)
    }
}

impl fmt::Debug for LogicExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Logic({} {} {})", self.left, self.logic_type, self.right)
    }
}

impl fmt::Debug for StringExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "String({}({}))", self.expr_type, self.arg)
    }
}

impl fmt::Debug for ArrayExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.children.iter().map(|c| format!("{:?}", c)).collect();
        write!(f, "Array([{}])", parts.join(", "))
    }
}
