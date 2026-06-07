# aoi

A typed SQLite client for Rust built on [`rusqlite`](https://github.com/rusqlite/rusqlite).

Define your table schema as a Rust struct, implement one trait, and get a fully typed API for all CRUD and DDL operations — no raw SQL required for common queries.

```rust
use aoi::prelude::*;

let mut client = Sqlite3Client::new("app.db")?;
client.create_table::<User>()?;
client.insert_row(&User { id: None, username: "alice".into(), .. })?;
```

---

## Installation

```toml
[dependencies]
aoi = { path = "../aoi" }
rusqlite = "0.31"
```

> SQLite is bundled via the `bundled` feature on rusqlite — no system SQLite install required.

---

## Quick start

### 1. Define your schema

Implement `TableSchema` for your struct. This is the single source of truth for the table name, column definitions, serialization, and deserialization.

```rust
use aoi::prelude::*;
use rusqlite::{Result, Row};

#[derive(Debug)]
struct User {
    id:       Option<i64>,  // None → auto-assigned by SQLite
    username: String,
    email:    String,
    age:      i64,
    score:    f64,
    active:   bool,
}

impl TableSchema for User {
    fn table_name() -> &'static str { "users" }

    fn columns() -> Vec<Column> {
        vec![
            Column::new("id",       Sqlite3Type::Integer).primary_key(),
            Column::new("username", Sqlite3Type::Varchar(64)).not_null().unique(),
            Column::new("email",    Sqlite3Type::Text).not_null().unique(),
            Column::new("age",      Sqlite3Type::Integer).not_null(),
            Column::new("score",    Sqlite3Type::Real).default("0.0"),
            Column::new("active",   Sqlite3Type::Boolean).not_null().default("1"),
        ]
    }

    // Bind values in non-primary-key column order
    fn to_row(&self) -> Vec<Box<dyn rusqlite::ToSql>> {
        vec![
            Box::new(self.username.clone()),
            Box::new(self.email.clone()),
            Box::new(self.age),
            Box::new(self.score),
            Box::new(self.active as i64),
        ]
    }

    // Hydrate from a SELECT row; indices match columns() order
    fn from_row(row: &Row<'_>) -> Result<Self> {
        Ok(User {
            id:       row.get(0)?,
            username: row.get(1)?,
            email:    row.get(2)?,
            age:      row.get(3)?,
            score:    row.get(4)?,
            active:   row.get::<_, i64>(5)? != 0,
        })
    }
}
```

### 2. Open a connection

```rust
// Persistent file
let mut client = Sqlite3Client::new("data/app.db")?;

// In-memory (useful for tests — dropped when client is dropped)
let mut client = Sqlite3Client::new(":memory:")?;
```

### 3. Use the API

```rust
client.create_table::<User>()?;

// Insert
let id = client.insert_row(&User { id: None, username: "alice".into(), ... })?;

// Select all
let users: Vec<User> = client.select_all::<User>(None)?;

// Select with filter
let active: Vec<User> = client.select_all::<User>(
    Some(WhereClause::new("active = ?", vec![SqlValue::Int(1)]))
)?;

// Select one
let user: Option<User> = client.select_one::<User>(
    WhereClause::new("id = ?", vec![SqlValue::Int(id)])
)?;

// Update
client.update_rows::<User>(
    UpdateSet::new().set("score", SqlValue::Float(10.0)),
    WhereClause::new("username = ?", vec![SqlValue::Text("alice".into())]),
)?;

// Delete
client.delete_rows::<User>(
    WhereClause::new("active = ?", vec![SqlValue::Int(0)])
)?;

// Count
let total: i64 = client.count::<User>(None)?;
```

---

## Column types

| `Sqlite3Type`     | SQLite affinity | Notes                          |
|-------------------|-----------------|--------------------------------|
| `Integer`         | `INTEGER`       |                                |
| `Real`            | `REAL`          |                                |
| `Text`            | `TEXT`          |                                |
| `Blob`            | `BLOB`          |                                |
| `Varchar(n)`      | `VARCHAR(n)`    | Stored as TEXT; n is advisory  |
| `Boolean`         | `INTEGER`       | Stored as 0 / 1                |
| `Null`            | `NULL`          |                                |

## Column constraints

```rust
Column::new("email", Sqlite3Type::Text)
    .primary_key()   // PRIMARY KEY
    .not_null()      // NOT NULL
    .unique()        // UNIQUE
    .default("''")   // DEFAULT '' (literal SQL expression)
```

---

## ALTER TABLE

Apply schema mutations via `alter_table`, which accepts a slice of `AlterOp` values applied in order.

```rust
// Add a column (must be nullable or have a DEFAULT)
client.alter_table::<User>(&[
    AlterOp::AddColumn(Column::new("bio", Sqlite3Type::Text).default("''")),
])?;

// Rename a column
client.alter_table::<User>(&[
    AlterOp::RenameColumn { from: "bio".into(), to: "biography".into() },
])?;

// Drop a column (emulated — see note below)
client.alter_table::<User>(&[
    AlterOp::DropColumn("biography".into()),
])?;

// Rename the table
client.alter_table::<User>(&[
    AlterOp::RenameTable("members".into()),
])?;
```

> **`DropColumn` note:** SQLite only added native `DROP COLUMN` in v3.35.0 (2021). For compatibility, this crate emulates it by creating a new table with the surviving columns, copying all rows, dropping the original, and renaming the copy back. The entire sequence runs inside a transaction and is atomic.

---

## Transactions

```rust
client.transaction(|tx| {
    tx.execute("INSERT INTO users (username, email, age, score, active) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params!["carol", "carol@example.com", 28, 8.0, 1])?;
    tx.execute("INSERT INTO audit (event) VALUES (?)",
        rusqlite::params!["user_created"])?;
    Ok(())
})?;
```

The transaction commits if the closure returns `Ok`, and rolls back automatically on `Err`.

---

## Upsert

```rust
// INSERT OR REPLACE — replaces the existing row on primary key conflict
client.upsert_row(&user)?;
```

---

## Raw SQL escape hatch

For `PRAGMA` statements, index creation, or anything outside the typed API:

```rust
client.execute_raw("PRAGMA journal_mode=WAL")?;
client.execute_raw("CREATE INDEX idx_users_email ON users(email)")?;
```

---

## Module layout

```
src/
├── lib.rs              ← crate root; pub mod prelude
├── main.rs             ← example binary (cargo run --bin example)
└── db/
    ├── mod.rs
    ├── types.rs        — Sqlite3Type, SqlValue
    ├── schema.rs       — Column, TableSchema
    ├── query.rs        — WhereClause, UpdateSet
    ├── alter.rs        — AlterOp
    ├── client.rs       — Sqlite3Client
    ├── ddl.rs          — create_table, drop_table, alter_table
    └── dml.rs          — insert_row, upsert_row, select_all, select_one,
                          update_rows, delete_rows, count, transaction
```

---

## Docs

```bash
cargo doc --no-deps --open
```

---

## License

MIT