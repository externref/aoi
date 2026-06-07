//! # query
//!
//! Composable builders for the parameterised parts of SQL statements.
//!
//! - [`WhereClause`] — pairs a `?`-placeholder expression with its bind values.
//!   Used by `select_all`, `select_one`, `update_rows`, `delete_rows`, and `count`.
//!
//! - [`UpdateSet`] — accumulates `col = ?` assignments for UPDATE statements.
//!
//! Both types keep every value as a typed [`SqlValue`] so the underlying
//! rusqlite calls always use parameterised queries — never string interpolation.

use super::types::SqlValue;

// ─── WHERE clause ─────────────────────────────────────────────────────────────

/// A parameterised WHERE expression with its bound values.
///
/// The `expr` string uses `?` placeholders; `params` must appear in the same
/// order as the placeholders.
///
/// ```rust
/// WhereClause::new("age > ? AND active = ?", vec![
///     SqlValue::Int(18),
///     SqlValue::Int(1),
/// ])
/// ```
pub struct WhereClause {
    /// SQL expression using `?` placeholders (e.g. `"id = ?"`)
    pub expr: String,
    /// Bind parameters in placeholder order.
    pub params: Vec<SqlValue>,
}

impl WhereClause {
    pub fn new(expr: impl Into<String>, params: Vec<SqlValue>) -> Self {
        Self { expr: expr.into(), params }
    }
}

// ─── UPDATE SET builder ───────────────────────────────────────────────────────

/// A fluent builder for the SET clause of an UPDATE statement.
///
/// Each call to `.set()` appends a `col = ?` assignment. The final
/// `assignments` vec is consumed by `update_rows` to build the SQL
/// and bind the values in order.
///
/// ```rust
/// UpdateSet::new()
///     .set("score", SqlValue::Float(10.0))
///     .set("active", SqlValue::Int(0))
/// ```
pub struct UpdateSet {
    /// Ordered list of (column name, new value) pairs.
    pub assignments: Vec<(String, SqlValue)>,
}

impl UpdateSet {
    pub fn new() -> Self {
        Self { assignments: vec![] }
    }

    /// Append a `column = value` assignment.
    pub fn set(mut self, col: impl Into<String>, val: SqlValue) -> Self {
        self.assignments.push((col.into(), val));
        self
    }
}
