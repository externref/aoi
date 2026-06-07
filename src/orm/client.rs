//! # client
//!
//! The [`Sqlite3Client`] struct owns the `rusqlite::Connection` and is the
//! single entry point for all database operations.
//!
//! DDL methods live in [`super::ddl`]; DML methods live in [`super::dml`].
//! Both modules extend `Sqlite3Client` via separate `impl` blocks, keeping
//! each concern in its own file while sharing the same struct.

use rusqlite::{Connection, Result};

/// A typed SQLite client wrapping a `rusqlite::Connection`.
///
/// Obtain one with [`Sqlite3Client::new`].  Pass `":memory:"` as the address
/// for an in-process ephemeral database (useful for tests).
///
/// Methods that mutate the schema (`alter_table`, `transaction`) take
/// `&mut self` because they call `conn.transaction()`, which requires
/// exclusive access to the connection.  All read and non-transactional write
/// methods take `&self`.
pub struct Sqlite3Client {
    /// The underlying rusqlite connection. Kept private so all access is
    /// funnelled through the typed API; use `execute_raw` for escape hatches.
    pub(crate) conn: Connection,
}

impl Sqlite3Client {
    /// Open (or create) the database file at `address`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Persistent file
    /// let client = Sqlite3Client::new("data/app.db")?;
    ///
    /// // In-memory (lost when the client is dropped)
    /// let client = Sqlite3Client::new(":memory:")?;
    /// ```
    pub fn new(address: &str) -> Result<Self> {
        Ok(Self { conn: Connection::open(address)? })
    }
}
