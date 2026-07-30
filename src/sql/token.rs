/// The keywords and symbols the front-end of the database recognises consequently supports.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Select,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,
    From,
    Where,
    And,
    Or,
    Create,
    Table,
    Database,
    Use,
    Show,
    Index,
    Unique,

    // Data Type
    BigIntType,
    IntType,
    BooleanType,
    VarcharType,

    // Identifiers & Literals
    StringLit(String),
    Ident(String),
    IntLit(i64),
    BoolLit(bool),

    // Symbols & Operators
    Asterisk,
    Comma,
    LParen,
    RParen,
    Eq,
    Neq,
    Gt,
    Lt,
    Gte, // >=
    Lte, // <=
    Semicolon,

    // End of file/input-stream
    Eof,
}
