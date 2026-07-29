use std::fmt::{self, Display};

use crate::error::DbError;

/// Represents the logical data type of a column defined in Schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DataType {
    BigInt = 0,
    Int = 1,
    Boolean = 2,
    Varchar = 3,
}

impl DataType {
    /// Returns the DataType corresponding to the given u8 `val`.
    pub fn from_u8(val: u8) -> Result<Self, DbError> {
        match val {
            0 => Ok(Self::BigInt),
            1 => Ok(Self::Int),
            2 => Ok(Self::Boolean),
            3 => Ok(Self::Varchar),
            _ => Err(DbError::CorruptPage(format!(
                "invalid DataType discriminant: {}",
                val
            ))),
        }
    }
}

/// A concrete, physical value or data stored inside a Tuple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    BigInt(i64),
    Int(i32),
    Null,
    Boolean(bool),
    Varchar(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::BigInt(val) => write!(f, "{}", val),
            Value::Int(val) => write!(f, "{}", val),
            Value::Null => write!(f, "NULL"),
            Value::Boolean(val) => write!(f, "{}", val),
            Value::Varchar(val) => write!(f, "{}", val),
        }
    }
}
