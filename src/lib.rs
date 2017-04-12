#![deny(warnings)]
//! # Sqlite support for the `r2d2` connection pool.
//!
//! Library crate: [r2d2-sqlite](https://crates.io/crates/r2d2-sqlite/)
//!
//! Integrated with: [r2d2](https://crates.io/crates/r2d2)
//! and [rusqlite](https://crates.io/crates/rusqlite)
//!
//! ## Example
//!
//! ```rust,no_run
//! extern crate r2d2;
//! extern crate r2d2_sqlite;
//! extern crate rusqlite;
//!
//! use std::thread;
//! use r2d2_sqlite::SqliteConnectionManager;
//!
//! fn main() {
//!     let config = r2d2::Config::default();
//!     let manager = SqliteConnectionManager::new("file.db");
//!     let pool = r2d2::Pool::new(config, manager).unwrap();
//!
//!     for i in 0..10i32 {
//!         let pool = pool.clone();
//!         thread::spawn(move || {
//!             let conn = pool.get().unwrap();
//!             conn.execute("INSERT INTO foo (bar) VALUES (?)", &[&i]).unwrap();
//!         });
//!     }
//! }
//! ```
extern crate r2d2;
extern crate rusqlite;


use rusqlite::{Connection, Error as RusqliteError, OpenFlags};



enum ConnectionConfig {
    File(String, OpenFlags),
}

/// An `r2d2::ManageConnection` for `rusqlite::Connection`s.
pub struct SqliteConnectionManager {
    config: ConnectionConfig,
}

impl SqliteConnectionManager {
    /// Creates a new `SqliteConnectionManager` from file.
    ///
    /// See `rusqlite::Connection::open`
    pub fn new(database: &str) -> Self {
        Self::new_with_flags(database, OpenFlags::default())
    }

    /// Creates a new `SqliteConnectionManager` from file with open flags.
    ///
    /// See `rusqlite::Connection::open_with_flags`
    pub fn new_with_flags(database: &str, flags: OpenFlags) -> Self {
        SqliteConnectionManager { config: ConnectionConfig::File(database.to_string(), flags) }
    }
}

impl r2d2::ManageConnection for SqliteConnectionManager {
    type Connection = Connection;
    type Error = RusqliteError;

    fn connect(&self) -> Result<Connection, RusqliteError> {
        match self.config {
                ConnectionConfig::File(ref path, flags) => Connection::open_with_flags(path, flags),
            }
            .map_err(Into::into)
    }

    fn is_valid(&self, conn: &mut Connection) -> Result<(), RusqliteError> {
        conn.execute_batch("").map_err(Into::into)
    }

    fn has_broken(&self, _: &mut Connection) -> bool {
        false
    }
}
