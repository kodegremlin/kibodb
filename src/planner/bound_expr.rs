use crate::{
    relation::types::{DataType, Value},
    sql::ast::BinaryOperator,
};

/// A BoundExpression represents a Sql Expression that has been fully validated against
/// the database catalog.
///
/// Unlike raw Ast Expression, a `BoundExpr` has resolved all identifiers (like column
/// names) into exact physical offsets, and has verified that all data types are compatible.
#[derive(Debug, Clone, PartialEq)]
pub enum BoundExpr {
    /// A column reference to an index offset within a `Tuple`.
    ColumnRef {
        /// The physical index of the column in the tuple's
        /// value array.
        col_idx: usize,
        /// The verified data type of the column.
        data_type: DataType,
    },

    /// A literal value directly parsed from the query and saved as an enum Tuple.
    Constant(Value),

    /// The representation of a binary operation stored after parsing the query &
    /// validating it against the schema.
    BinaryOp {
        left: Box<BoundExpr>,
        op: BinaryOperator,
        right: Box<BoundExpr>,
    },
}
