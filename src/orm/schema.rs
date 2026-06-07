//! # schema
//!
//! Column-level metadata and the [`TableSchema`] trait that every persistable
//! struct must implement.
//!
//! ## Design
//!
//! [`Column`] is a plain data struct built with a fluent builder API.
//! It knows how to render itself as a DDL fragment (`to_ddl`), which keeps
//! all SQL generation co-located with the type system rather than scattered
//! across the client methods.
//!
//! [`TableSchema`] is the single seam between a Rust struct and the database.
//! Implementing it for a type gives the client full knowledge of:
//!
//! - the table name
//! - the ordered column list (drives CREATE TABLE and SELECT column positions)
//! - how to bind a struct instance into INSERT parameters (`to_row`)
//! - how to hydrate a struct from a query row (`from_row`)

use rusqlite::{Result, Row};
use super::types::Sqlite3Type;

// ─── Column descriptor ────────────────────────────────────────────────────────

/// Metadata for a single database column.
///
/// Constructed with [`Column::new`] and refined with builder methods.
///
/// ```rust
/// Column::new("email", Sqlite3Type::Text)
///     .not_null()
///     .unique()
/// ```
#[derive(Debug, Clone)]
pub struct Column {
    /// Column name as it appears in DDL and queries.
    pub name: &'static str,
    /// SQLite type affinity.
    pub kind: Sqlite3Type,
    /// Whether this column is part of the PRIMARY KEY.
    pub primary_key: bool,
    /// Whether a NOT NULL constraint is emitted.
    pub not_null: bool,
    /// Whether a UNIQUE constraint is emitted.
    pub unique: bool,
    /// Optional literal DEFAULT expression (e.g. `"0"`, `"'guest'"`).
    pub default: Option<String>,
}

impl Column {
    /// Create a column with no constraints. Use builder methods to add them.
    pub fn new(name: &'static str, kind: Sqlite3Type) -> Self {
        Self {
            name,
            kind,
            primary_key: false,
            not_null:    false,
            unique:      false,
            default:     None,
        }
    }

    /// Mark this column as PRIMARY KEY.
    pub fn primary_key(mut self) -> Self { self.primary_key = true; self }

    /// Add a NOT NULL constraint.
    pub fn not_null(mut self) -> Self { self.not_null = true; self }

    /// Add a UNIQUE constraint.
    pub fn unique(mut self) -> Self { self.unique = true; self }

    /// Set a DEFAULT expression. The value is inserted verbatim into DDL,
    /// so string literals must be quoted: `.default("'guest'")`.
    pub fn default(mut self, v: impl Into<String>) -> Self {
        self.default = Some(v.into());
        self
    }

    /// Render this column as a DDL fragment for use inside CREATE TABLE.
    ///
    /// Example output: `"email TEXT NOT NULL UNIQUE"`
    pub(crate) fn to_ddl(&self) -> String {
        let mut parts = vec![format!("{} {}", self.name, self.kind)];
        if self.primary_key          { parts.push("PRIMARY KEY".into()); }
        if self.not_null             { parts.push("NOT NULL".into());    }
        if self.unique               { parts.push("UNIQUE".into());      }
        if let Some(ref d) = self.default {
            parts.push(format!("DEFAULT {d}"));
        }
        parts.join(" ")
    }
}

// ─── TableSchema trait ────────────────────────────────────────────────────────

/// Contract between a Rust struct and its database representation.
///
/// Implement this once per table. The client uses the four required methods
/// to drive all DDL and DML operations without knowing the concrete type.
///
/// The `non_pk_columns` helper is provided automatically; it strips primary-key
/// columns from the list so INSERT statements never try to bind an auto-id.
pub trait TableSchema: Sized {
    /// The SQL table name (e.g. `"users"`).
    fn table_name() -> &'static str;

    /// Ordered list of all columns, including the primary key.
    /// The order must match the `SELECT *` column positions used in `from_row`.
    fn columns() -> Vec<Column>;

    /// Serialize `self` into ordered bind parameters for INSERT.
    /// Must match the order of `non_pk_columns()` — primary key excluded.
    fn to_row(&self) -> Vec<Box<dyn rusqlite::ToSql>>;

    /// Deserialize a query `Row` back into `Self`.
    /// Column indices must match the order declared in `columns()`.
    fn from_row(row: &Row<'_>) -> Result<Self>;

    /// All columns that are not part of the primary key.
    /// Derived automatically from `columns()`; override only if needed.
    fn non_pk_columns() -> Vec<Column> {
        Self::columns().into_iter().filter(|c| !c.primary_key).collect()
    }
}
