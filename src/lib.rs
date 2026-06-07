//! # aoi
//!
//! A typed SQLite client built on top of [`rusqlite`].
//!
//! ## Module layout
//!
//! ```text
//! wrapper/
//! ├── types.rs   — Sqlite3Type (column affinity) + SqlValue (bind params)
//! ├── schema.rs  — Column descriptor + TableSchema trait
//! ├── query.rs   — WhereClause + UpdateSet builders
//! ├── alter.rs   — AlterOp enum
//! ├── client.rs  — Sqlite3Client struct + constructor
//! ├── ddl.rs     — impl Sqlite3Client { create_table, drop_table, alter_table }
//! └── dml.rs     — impl Sqlite3Client { insert_row, select_all, update_rows, … }
//! ```
//!
//! ## Quick start
//!
//! ```rust
//! use aoi::prelude::*;
//!
//! struct User { id: Option<i64>, name: String }
//!
//! impl TableSchema for User {
//!     fn table_name() -> &'static str { "users" }
//!     fn columns() -> Vec<Column> {
//!         vec![
//!             Column::new("id",   Sqlite3Type::Integer).primary_key(),
//!             Column::new("name", Sqlite3Type::Varchar(64)).not_null(),
//!         ]
//!     }
//!     fn to_row(&self) -> Vec<Box<dyn rusqlite::ToSql>> {
//!         vec![Box::new(self.name.clone())]
//!     }
//!     fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
//!         Ok(User { id: row.get(0)?, name: row.get(1)? })
//!     }
//! }
//!
//! let client = Sqlite3Client::new(":memory:")?;
//! client.create_table::<User>()?;
//! client.insert_row(&User { id: None, name: "alice".into() })?;
//! ```

mod orm;

/// Re-exports every type and trait needed for day-to-day use.
/// Import with `use aoi::prelude::*`.
pub mod prelude {
    pub use super::orm::alter::AlterOp;
    pub use super::orm::client::Sqlite3Client;
    pub use super::orm::query::{UpdateSet, WhereClause};
    pub use super::orm::schema::{Column, TableSchema};
    pub use super::orm::types::{SqlValue, Sqlite3Type};
}