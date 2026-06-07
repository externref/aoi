//! # ddl
//!
//! Data Definition Language operations: CREATE TABLE, DROP TABLE, ALTER TABLE.
//!
//! All methods are implemented on [`Sqlite3Client`] via an `impl` block defined
//! here and re-exported from the crate root.  Keeping DDL separate from DML
//! makes it easier to audit schema-mutating code paths in isolation.

use rusqlite::Result;
use super::{
    alter::AlterOp,
    client::Sqlite3Client,
    schema::TableSchema,
};

impl Sqlite3Client {
    // ── CREATE ────────────────────────────────────────────────────────────────

    /// Create the table for schema type `T` if it does not already exist.
    ///
    /// Column DDL is derived entirely from `T::columns()`, so the database
    /// schema stays in sync with the Rust type definition.
    pub fn create_table<T: TableSchema>(&self) -> Result<()> {
        let col_defs: Vec<String> = T::columns()
            .iter()
            .map(|c| c.to_ddl())
            .collect();

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            T::table_name(),
            col_defs.join(", ")
        );

        self.conn.execute(&sql, [])?;
        Ok(())
    }

    // ── DROP ─────────────────────────────────────────────────────────────────

    /// Drop the table for schema type `T` if it exists. All data is lost.
    pub fn drop_table<T: TableSchema>(&self) -> Result<()> {
        let sql = format!("DROP TABLE IF EXISTS {}", T::table_name());
        self.conn.execute(&sql, [])?;
        Ok(())
    }

    // ── ALTER ─────────────────────────────────────────────────────────────────

    /// Apply one or more [`AlterOp`] alterations to the table for `T`.
    ///
    /// Ops are executed sequentially. If any op fails, previously applied ops
    /// in the same call are **not** rolled back (each native ALTER runs in its
    /// own implicit transaction). The exception is `DropColumn`, which wraps
    /// its entire copy-and-rename sequence in an explicit transaction.
    ///
    /// Pass multiple ops in a single call to group related migrations:
    ///
    /// ```rust
    /// client.alter_table::<User>(&[
    ///     AlterOp::AddColumn(Column::new("bio", Sqlite3Type::Text).default("''")),
    ///     AlterOp::RenameColumn { from: "bio".into(), to: "biography".into() },
    /// ])?;
    /// ```
    pub fn alter_table<T: TableSchema>(&mut self, ops: &[AlterOp]) -> Result<()> {
        let table = T::table_name();

        for op in ops {
            match op {
                // Native: ALTER TABLE … ADD COLUMN …
                AlterOp::AddColumn(col) => {
                    let sql = format!(
                        "ALTER TABLE {} ADD COLUMN {}",
                        table, col.to_ddl()
                    );
                    self.conn.execute(&sql, [])?;
                }

                // Native: ALTER TABLE … RENAME COLUMN … TO …
                AlterOp::RenameColumn { from, to } => {
                    let sql = format!(
                        "ALTER TABLE {} RENAME COLUMN {} TO {}",
                        table, from, to
                    );
                    self.conn.execute(&sql, [])?;
                }

                // Native: ALTER TABLE … RENAME TO …
                AlterOp::RenameTable(new_name) => {
                    let sql = format!(
                        "ALTER TABLE {} RENAME TO {}",
                        table, new_name
                    );
                    self.conn.execute(&sql, [])?;
                }

                // Emulated: SQLite didn't support DROP COLUMN until 3.35.0
                // (March 2021) and many bundled builds predate that.
                //
                // Strategy (official SQLite recommendation):
                //   1. Create __temp_<table> with the surviving columns.
                //   2. Copy all rows, omitting the dropped column.
                //   3. Drop the original table.
                //   4. Rename __temp_<table> back to <table>.
                //
                // Steps 1-4 run inside a single transaction so the operation
                // is atomic — either all four succeed or none of them persist.
                AlterOp::DropColumn(col_name) => {
                    let temp = format!("__temp_{table}");

                    // Filter the schema to the columns that survive.
                    let surviving = T::columns()
                        .into_iter()
                        .filter(|c| c.name != col_name.as_str())
                        .collect::<Vec<_>>();

                    // Guard: make sure the column actually exists in the schema.
                    if surviving.len() == T::columns().len() {
                        return Err(rusqlite::Error::InvalidParameterName(
                            format!("column '{col_name}' not found in schema for table '{table}'"),
                        ));
                    }

                    let col_ddl: Vec<String> = surviving.iter().map(|c| c.to_ddl()).collect();
                    let col_names: Vec<&str>  = surviving.iter().map(|c| c.name).collect();
                    let cols_csv = col_names.join(", ");

                    let tx = self.conn.transaction()?;

                    // Step 1 — new table with surviving schema
                    tx.execute(
                        &format!("CREATE TABLE {} ({})", temp, col_ddl.join(", ")),
                        [],
                    )?;

                    // Step 2 — copy rows (dropped column automatically excluded)
                    tx.execute(
                        &format!(
                            "INSERT INTO {temp} ({cols_csv}) SELECT {cols_csv} FROM {table}"
                        ),
                        [],
                    )?;

                    // Step 3 — remove original
                    tx.execute(&format!("DROP TABLE {table}"), [])?;

                    // Step 4 — rename temp back to the canonical name
                    tx.execute(
                        &format!("ALTER TABLE {temp} RENAME TO {table}"),
                        [],
                    )?;

                    tx.commit()?;
                }
            }
        }

        Ok(())
    }
}
