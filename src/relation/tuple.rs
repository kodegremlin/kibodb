use std::io::{Read, Write};

use crate::{
    error::DbError,
    relation::{
        schema::Schema,
        types::{DataType, Value},
    },
};

/// The in-memory representation of a single database row.
#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub values: Vec<Value>,
}

impl Tuple {
    /// Constructs an initialized Tuple with the provided `values`.
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// Encodes the tuple in little-endian binary format and writes them into
    /// the provided writer.
    pub fn encode<W: Write>(&self, schema: &Schema, writer: &mut W) -> Result<(), DbError> {
        // Why they must match is because schema is the high level view and
        // tuple is the low level view of the table data.
        // name | age | address
        // var..| int | varchar
        if self.values.len() != schema.columns.len() {
            return Err(DbError::CorruptPage(
                "tuple value count does not match schema column count".into(),
            ));
        }
        for (i, col) in schema.columns.iter().enumerate() {
            let value = &self.values[i];
            let is_null = matches!(value, Value::Null);

            writer
                .write_all(&[is_null as u8])
                .map_err(DbError::Io)?;

            if !is_null {
                match (col.data_type, value) {
                    (DataType::BigInt, Value::BigInt(v)) => {
                        writer
                            .write_all(&v.to_le_bytes())
                            .map_err(DbError::Io)?;
                    }
                    (DataType::Int, Value::Int(v)) => {
                        writer
                            .write_all(&v.to_le_bytes())
                            .map_err(DbError::Io)?;
                    }
                    (DataType::Boolean, Value::Boolean(v)) => {
                        writer
                            .write_all(&[*v as u8])
                            .map_err(DbError::Io)?;
                    }
                    (DataType::Varchar, Value::Varchar(v)) => {
                        let length = v.len() as u32;
                        writer
                            .write_all(&length.to_le_bytes())
                            .map_err(DbError::Io)?;
                        writer
                            .write_all(v.as_bytes())
                            .map_err(DbError::Io)?;
                    }
                    _ => {
                        return Err(DbError::CorruptPage(
                            "value type does not match schema type definition".into(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Decodes a `Tuple` from little-endian bytes using the provided schema
    /// blueprint.
    pub fn decode<R: Read>(schema: &Schema, reader: &mut R) -> Result<Self, DbError> {
        let mut values = Vec::with_capacity(schema.columns.len());
        let mut is_null = [0u8; 1];

        for col in &schema.columns {
            reader
                .read_exact(&mut is_null)
                .map_err(DbError::Io)?;

            if is_null[0] == 1 {
                values.push(Value::Null);
                continue;
            }
            let value = match col.data_type {
                DataType::BigInt => {
                    let mut buffer = [0u8; 8];

                    reader
                        .read_exact(&mut buffer)
                        .map_err(DbError::Io)?;
                    Value::BigInt(i64::from_le_bytes(buffer))
                }
                DataType::Int => {
                    let mut buffer = [0u8; 4];

                    reader
                        .read_exact(&mut buffer)
                        .map_err(DbError::Io)?;
                    Value::Int(i32::from_le_bytes(buffer))
                }
                DataType::Boolean => {
                    let mut buffer = [0u8; 1];

                    reader
                        .read_exact(&mut buffer)
                        .map_err(DbError::Io)?;

                    Value::Boolean(buffer[0] == 1)
                }
                DataType::Varchar => {
                    let mut len_buf = [0u8; 4];
                    reader
                        .read_exact(&mut len_buf)
                        .map_err(DbError::Io)?;

                    let str_len = u32::from_le_bytes(len_buf) as usize;
                    let mut buffer = vec![0u8; str_len];

                    reader
                        .read_exact(&mut buffer)
                        .map_err(DbError::Io)?;

                    let parsed_str = String::from_utf8(buffer)
                        .map_err(|err| DbError::CorruptPage(format!("invalid utf-8: {}", err)))?;
                    Value::Varchar(parsed_str)
                }
            };
            values.push(value);
        }
        Ok(Self { values })
    }
}
