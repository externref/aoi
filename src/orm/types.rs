//! # types
//!
//! Primitive type vocabulary for the sqlite3 client.
//!
//! - [`Sqlite3Type`] — column affinity enum used when declaring schema columns.
//!   Renders directly into DDL fragments via `Display`.
//!
//! - [`SqlValue`] — a runtime-typed wrapper around a single SQL bind parameter.
//!   Implements `rusqlite::ToSql` so it can be passed to any execute / query call.

use std::fmt;

// ─── Column affinity ─────────────────────────────────────────────────────────

/// SQLite column type affinities, plus `Varchar(n)` as a common convenience.
///
/// `Boolean` is stored as `INTEGER` (0 / 1) because SQLite has no native bool.
/// `Varchar(n)` is stored as `TEXT`; SQLite ignores the length constraint at
/// the engine level, but it documents intent and is respected by most tooling.
#[derive(Debug, Clone)]
pub enum Sqlite3Type {
    Integer,
    Real,
    Text,
    Blob,
    /// Stored as TEXT; `n` is advisory and appears verbatim in DDL.
    Varchar(usize),
    /// Stored as INTEGER (0 = false, 1 = true).
    Boolean,
    Null,
}

impl fmt::Display for Sqlite3Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sqlite3Type::Integer    => write!(f, "INTEGER"),
            Sqlite3Type::Real       => write!(f, "REAL"),
            Sqlite3Type::Text       => write!(f, "TEXT"),
            Sqlite3Type::Blob       => write!(f, "BLOB"),
            Sqlite3Type::Varchar(n) => write!(f, "VARCHAR({n})"),
            Sqlite3Type::Boolean    => write!(f, "INTEGER"),
            Sqlite3Type::Null       => write!(f, "NULL"),
        }
    }
}

// ─── Bind-parameter value ─────────────────────────────────────────────────────

/// A heterogeneous SQL bind-parameter value.
///
/// Used in [`WhereClause`](super::query::WhereClause) and
/// [`UpdateSet`](super::query::UpdateSet) so callers never have to spell out
/// `Box<dyn ToSql>` at the use site.
pub enum SqlValue {
    Int(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
    Null,
}

impl rusqlite::ToSql for SqlValue {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use rusqlite::types::{ToSqlOutput, Value};
        Ok(match self {
            SqlValue::Int(v)   => ToSqlOutput::Owned(Value::Integer(*v)),
            SqlValue::Float(v) => ToSqlOutput::Owned(Value::Real(*v)),
            SqlValue::Text(v)  => ToSqlOutput::Owned(Value::Text(v.clone())),
            SqlValue::Blob(v)  => ToSqlOutput::Owned(Value::Blob(v.clone())),
            SqlValue::Null     => ToSqlOutput::Owned(Value::Null),
        })
    }
}
