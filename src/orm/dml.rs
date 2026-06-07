//! # dml
//!
//! Data Manipulation Language operations: INSERT, SELECT, UPDATE, DELETE,
//! COUNT, transactions, and a raw-SQL escape hatch.
//!
//! Every public method is parameterised and uses `?` placeholders — no
//! user-supplied string is ever interpolated directly into a query.

use rusqlite::Result;
use super::{
    client::Sqlite3Client,
    query::{UpdateSet, WhereClause},
    schema::TableSchema,
};

impl Sqlite3Client {
    // ── INSERT ────────────────────────────────────────────────────────────────

    /// Insert a single row and return the `rowid` of the new record.
    ///
    /// Primary-key columns are excluded from the INSERT (via `non_pk_columns`)
    /// so auto-increment IDs are assigned by the engine.
    pub fn insert_row<T: TableSchema>(&self, row: &T) -> Result<i64> {
        let cols = T::non_pk_columns();
        let names: Vec<&str> = cols.iter().map(|c| c.name).collect();
        let placeholders = vec!["?"; names.len()];

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            T::table_name(),
            names.join(", "),
            placeholders.join(", ")
        );

        let params = row.to_row();
        let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        self.conn.execute(&sql, refs.as_slice())?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert or replace a row when the primary key already exists (upsert).
    ///
    /// Uses `INSERT OR REPLACE`, which deletes the conflicting row and inserts
    /// the new one — triggers and foreign-key cascades fire on the delete side.
    /// Use a `WHERE`-guarded UPDATE if you need in-place mutation semantics.
    pub fn upsert_row<T: TableSchema>(&self, row: &T) -> Result<i64> {
        let cols = T::non_pk_columns();
        let names: Vec<&str> = cols.iter().map(|c| c.name).collect();
        let placeholders = vec!["?"; names.len()];

        let sql = format!(
            "INSERT OR REPLACE INTO {} ({}) VALUES ({})",
            T::table_name(),
            names.join(", "),
            placeholders.join(", ")
        );

        let params = row.to_row();
        let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        self.conn.execute(&sql, refs.as_slice())?;
        Ok(self.conn.last_insert_rowid())
    }

    // ── SELECT ────────────────────────────────────────────────────────────────

    /// Return all rows, optionally filtered by a WHERE clause.
    ///
    /// Pass `None` to fetch every row in the table.
    pub fn select_all<T: TableSchema>(
        &self,
        where_clause: Option<WhereClause>,
    ) -> Result<Vec<T>> {
        // Decompose the optional where clause into SQL fragment + bind params.
        let (condition, bind_params) = match where_clause {
            Some(w) => (format!(" WHERE {}", w.expr), w.params),
            None    => (String::new(), vec![]),
        };

        let sql = format!("SELECT * FROM {}{}", T::table_name(), condition);
        let refs: Vec<&dyn rusqlite::ToSql> = bind_params
            .iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(refs.as_slice(), |row| T::from_row(row))?;
        rows.collect()
    }

    /// Return the first row matching `where_clause`, or `None` if not found.
    ///
    /// Appends `LIMIT 1` so the engine stops scanning after the first hit.
    pub fn select_one<T: TableSchema>(
        &self,
        where_clause: WhereClause,
    ) -> Result<Option<T>> {
        let sql = format!(
            "SELECT * FROM {} WHERE {} LIMIT 1",
            T::table_name(),
            where_clause.expr
        );
        let refs: Vec<&dyn rusqlite::ToSql> = where_clause
            .params
            .iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query_map(refs.as_slice(), |row| T::from_row(row))?;
        rows.next().transpose()
    }

    // ── UPDATE ────────────────────────────────────────────────────────────────

    /// Update matching rows and return the number of rows affected.
    ///
    /// Bind order: SET values come first (in `.set()` call order), then the
    /// WHERE clause values.  This matches the SQL parameter ordering:
    /// `UPDATE t SET a = ?, b = ? WHERE c = ?`
    pub fn update_rows<T: TableSchema>(
        &self,
        set: UpdateSet,
        where_clause: WhereClause,
    ) -> Result<usize> {
        let assignments: Vec<String> = set
            .assignments
            .iter()
            .map(|(col, _)| format!("{col} = ?"))
            .collect();

        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            T::table_name(),
            assignments.join(", "),
            where_clause.expr
        );

        // Concatenate SET binds followed by WHERE binds in one slice.
        let mut params: Vec<&dyn rusqlite::ToSql> = set
            .assignments
            .iter()
            .map(|(_, v)| v as &dyn rusqlite::ToSql)
            .collect();
        let where_refs: Vec<&dyn rusqlite::ToSql> = where_clause
            .params
            .iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();
        params.extend(where_refs);

        Ok(self.conn.execute(&sql, params.as_slice())?)
    }

    // ── DELETE ────────────────────────────────────────────────────────────────

    /// Delete rows matching `where_clause`. Returns the number of rows removed.
    ///
    /// A WHERE clause is required — use `execute_raw` if you intentionally
    /// need to delete all rows (consider TRUNCATE semantics).
    pub fn delete_rows<T: TableSchema>(
        &self,
        where_clause: WhereClause,
    ) -> Result<usize> {
        let sql = format!(
            "DELETE FROM {} WHERE {}",
            T::table_name(),
            where_clause.expr
        );
        let refs: Vec<&dyn rusqlite::ToSql> = where_clause
            .params
            .iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();
        Ok(self.conn.execute(&sql, refs.as_slice())?)
    }

    // ── COUNT ─────────────────────────────────────────────────────────────────

    /// Return the number of rows, optionally scoped to a WHERE clause.
    pub fn count<T: TableSchema>(
        &self,
        where_clause: Option<WhereClause>,
    ) -> Result<i64> {
        let (condition, bind_params) = match where_clause {
            Some(w) => (format!(" WHERE {}", w.expr), w.params),
            None    => (String::new(), vec![]),
        };

        let sql = format!("SELECT COUNT(*) FROM {}{}", T::table_name(), condition);
        let refs: Vec<&dyn rusqlite::ToSql> = bind_params
            .iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();

        self.conn.query_row(&sql, refs.as_slice(), |row| row.get(0))
    }

    // ── TRANSACTIONS ──────────────────────────────────────────────────────────

    /// Run `f` inside a database transaction.
    ///
    /// The transaction is committed if `f` returns `Ok`, and rolled back
    /// automatically if `f` returns `Err` or panics.
    ///
    /// ```rust
    /// client.transaction(|tx| {
    ///     tx.execute("INSERT INTO users ...", [])?;
    ///     tx.execute("INSERT INTO audit ...", [])?;
    ///     Ok(())
    /// })?;
    /// ```
    pub fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> Result<R>,
    {
        let tx = self.conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }

    // ── RAW ESCAPE HATCH ──────────────────────────────────────────────────────

    /// Execute arbitrary SQL with no bind parameters.
    ///
    /// Intended for PRAGMA statements, index creation, or one-off migrations
    /// that fall outside the typed API.  Prefer the typed methods for all
    /// routine DML — raw SQL bypasses schema validation entirely.
    pub fn execute_raw(&self, sql: &str) -> Result<usize> {
        self.conn.execute(sql, [])
    }
}
