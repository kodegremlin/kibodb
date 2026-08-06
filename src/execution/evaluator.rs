use crate::{
    error::Error,
    relation::{schema::Schema, tuple::Tuple, types::Value},
    sql::{
        ast::{BinaryOperator, Expr},
        parser::AstLiteral,
    },
};

/// A stateless engine that computes the expressions of the SQL we've parsed.
/// It resolves the AST expressions into concrete `Value`s based on the data
/// present within the `Tuple`s.
///
/// # Note to me
/// I am writing the Evaluator mostly as the Interpreter book, however there
/// were some things that were different, after some of this is complete look
/// into the book's model and see if there is something I may have forgotten.
pub struct Evaluator;

impl Evaluator {
    /// Recursively evaluates an AST expression against a Tuple.
    pub fn evaluate(expr: &Expr, tuple: &Tuple, schema: &Schema) -> Result<Value, Error> {
        match expr {
            Expr::Column(col_ref) => {
                let val = tuple.get_value(schema, &col_ref.column_name)?;
                Ok(val.clone())
            }
            Expr::Literal(lit) => Ok(Self::eval_literal(lit)),
            Expr::BinaryOp { left, op, right } => {
                let left_val = Self::evaluate(left, tuple, schema)?;
                let right_val = Self::evaluate(right, tuple, schema)?;
                Self::eval_binary_op(&left_val, *op, &right_val)
            }
            Expr::Average(_) | Expr::Count(_) => Err(Error::SyntaxErr(
                "aggregate functions cannot be evaluated in linear context".into(),
            )),
        }
    }

    /// Temporary stub
    fn eval_literal(lit: &AstLiteral) -> Value {
        match lit {
            AstLiteral::String(val) => Value::Varchar(val.clone()),
            AstLiteral::Int(val) => Value::BigInt(*val),
            AstLiteral::Boolean(val) => Value::Boolean(*val),
            AstLiteral::Null => Value::Null,
        }
    }

    /// Evaluates a binary operation between two values. Handles the SQL's Three-Valued
    /// Logic (3VL) for NULLs.
    fn eval_binary_op(left: &Value, op: BinaryOperator, right: &Value) -> Result<Value, Error> {
        if op == BinaryOperator::And || op == BinaryOperator::Or {
            return Self::eval_logical_op(left, op, right);
        }
        // For all standards comparisons (=, >, <, etc), if either operand is Null,
        // the entire expression evaluates to Null.
        if matches!(left, Value::Null) || matches!(right, Value::Null) {
            return Ok(Value::Null);
        }
        let result = match op {
            BinaryOperator::Neq => left != right,
            BinaryOperator::Eq => left == right,
            BinaryOperator::Gt => left > right,
            BinaryOperator::Lt => left < right,
            BinaryOperator::Gte => left >= right,
            BinaryOperator::Lte => left <= right,
            _ => unreachable!("logical operators should've been hanlded above"),
        };
        Ok(Value::Boolean(result))
    }

    /// Evaluates logical And/Or operations, and the tricky Null handling called the 3VL.
    fn eval_logical_op(left: &Value, op: BinaryOperator, right: &Value) -> Result<Value, Error> {
        let left_bool = match left {
            Value::Boolean(val) => Some(*val),
            Value::Null => None,
            _ => {
                return Err(Error::SyntaxErr(
                    "left operand of logical operator must be boolean".into(),
                ));
            }
        };
        let right_bool = match right {
            Value::Boolean(val) => Some(*val),
            Value::Null => None,
            _ => {
                return Err(Error::SyntaxErr(
                    "right operand of logical operator must be boolean".into(),
                ));
            }
        };
        match op {
            BinaryOperator::And => {
                match (left_bool, right_bool) {
                    // True AND <anything> || <anything> AND True is True
                    (Some(false), _) | (_, Some(false)) => Ok(Value::Boolean(false)),
                    // Only True AND True is True
                    (Some(true), Some(true)) => Ok(Value::Boolean(true)),
                    // True AND Null || Null AND Null is Null
                    _ => Ok(Value::Null),
                }
            }
            BinaryOperator::Or => {
                match (left_bool, right_bool) {
                    // False AND <anything> || <anything> AND False is False
                    (Some(true), _) | (_, Some(true)) => Ok(Value::Boolean(true)),
                    // Only False AND False is False
                    (Some(false), Some(false)) => Ok(Value::Boolean(false)),
                    // False AND Null || Null AND Null is Null
                    _ => Ok(Value::Null),
                }
            }
            _ => unreachable!("only And/Or should be routed here"),
        }
    }
}
