//! Internal `db` module tree.
//! Each sub-module owns one logical concern; `ddl` and `dml` extend
//! `Sqlite3Client` via separate `impl` blocks rather than one monolithic file.

pub mod alter;
pub mod client;
pub mod ddl;
pub mod dml;
pub mod query;
pub mod schema;
pub mod types;
