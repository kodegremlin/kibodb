use std::{
    cmp::Ordering,
    fmt::{self, Display},
};

use crate::error::Error;

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
    pub fn from_u8(val: u8) -> Result<Self, Error> {
        match val {
            0 => Ok(Self::BigInt),
            1 => Ok(Self::Int),
            2 => Ok(Self::Boolean),
            3 => Ok(Self::Varchar),
            _ => Err(Error::CorruptPage(format!(
                "invalid DataType discriminant: {}",
                val
            ))),
        }
    }
}

/// A concrete, physical value or data stored inside a Tuple.
#[derive(Debug, Clone, Eq)]
pub enum Value {
    BigInt(i64),
    Int(i32),
    Null,
    Boolean(bool),
    Varchar(String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (BigInt(a), Int(b)) => *a == *b as i64,
            (Int(a), BigInt(b)) => (*a as i64) == *b,
            (BigInt(a), BigInt(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Null, Null) => true,
            (Boolean(a), Boolean(b)) => a == b,
            (Varchar(a), Varchar(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Value::*;
        match (self, other) {
            (BigInt(a), BigInt(b)) => a.partial_cmp(b),
            (Int(a), Int(b)) => a.partial_cmp(b),
            (Null, Null) => Some(Ordering::Equal),
            (Null, _) | (_, Null) => None,
            (BigInt(a), Int(b)) => a.partial_cmp(&(*b as i64)),
            (Int(a), BigInt(b)) => (*a as i64).partial_cmp(b),
            (Boolean(a), Boolean(b)) => a.partial_cmp(b),
            (Varchar(a), Varchar(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
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

impl Value {
    /// Returns the string from the `Varchar` type.
    pub fn varchar_to_str(&self) -> Option<&str> {
        if let Value::Varchar(val) = self {
            Some(val)
        } else {
            None
        }
    }

    /// Returns the value from the `BigInt` type.
    pub fn bigint_to_i64(&self) -> Option<i64> {
        if let Value::BigInt(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    /// Returns the value from the `Int` type.
    pub fn int_to_i32(&self) -> Option<i32> {
        if let Value::Int(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    /// Returns the value from the `Boolean` type.
    pub fn boolean_to_bool(&self) -> Option<bool> {
        if let Value::Boolean(val) = self {
            Some(*val)
        } else {
            None
        }
    }
}
