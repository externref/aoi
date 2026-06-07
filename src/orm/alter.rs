//! # alter
//!
//! The [`AlterOp`] enum describes a single schema mutation.
//! A slice of `AlterOp` values is passed to
//! [`Sqlite3Client::alter_table`](super::client::Sqlite3Client::alter_table),
//! which applies them in order.
//!
//! ## SQLite limitations
//!
//! SQLite natively supports only three ALTER TABLE forms:
//!
//! | Operation      | Native? | Notes                                      |
//! |----------------|---------|--------------------------------------------|
//! | ADD COLUMN     | ✓       | New column must have DEFAULT or be nullable |
//! | RENAME COLUMN  | ✓       | Available since SQLite 3.25.0 (2018)       |
//! | RENAME TABLE   | ✓       |                                             |
//! | DROP COLUMN    | ✗       | Emulated via copy-and-rename transaction   |
//!
//! The `DropColumn` variant triggers the emulation strategy: create a new
//! table with the surviving columns, copy all rows, drop the original, and
//! rename the copy. The whole sequence runs inside a single transaction so
//! it is atomic and rolls back cleanly on any error.

use super::schema::Column;

/// A single schema alteration applied by `alter_table`.
#[derive(Debug)]
pub enum AlterOp {
    /// `ALTER TABLE … ADD COLUMN …`
    ///
    /// The column must either be nullable or carry a DEFAULT value;
    /// otherwise SQLite will reject the statement.
    AddColumn(Column),

    /// `ALTER TABLE … RENAME COLUMN <from> TO <to>`
    RenameColumn {
        /// Existing column name.
        from: String,
        /// New column name.
        to: String,
    },

    /// `ALTER TABLE … RENAME TO <new_name>`
    ///
    /// Note: after renaming, the generic `T::table_name()` still returns the
    /// old name. Subsequent operations on the same `T` will fail unless you
    /// use `execute_raw` or redefine your schema type.
    RenameTable(String),

    /// Remove a column by name (emulated — see module docs).
    ///
    /// Returns an error if `col_name` does not appear in `T::columns()`.
    DropColumn(String),
}
