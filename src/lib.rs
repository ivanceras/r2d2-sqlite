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
//!     let manager = SqliteConnectionManager::file("file.db");
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


use rusqlite::{Connection, Error};
use std::path::{Path, PathBuf};



enum Source {
    File(PathBuf),
    Memory,
}

/// An `r2d2::ManageConnection` for `rusqlite::Connection`s.
pub struct SqliteConnectionManager(Source);

impl SqliteConnectionManager {
    /// Creates a new `SqliteConnectionManager` from file.
    ///
    /// See `rusqlite::Connection::open`
    pub fn file<P: AsRef<Path>>(path: P) -> Self {
       SqliteConnectionManager(Source::File(path.as_ref().to_path_buf()))
    }

    /// Creates a new `SqliteConnectionManager` from memory.
    pub fn memory() -> Self {
        SqliteConnectionManager(Source::Memory)
    }
}

impl r2d2::ManageConnection for SqliteConnectionManager {
    type Connection = Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Connection, Error> {
        match self.0 {
            Source::File(ref path) => Connection::open(path),
            Source::Memory => Connection::open_in_memory()
         }
            .map_err(Into::into)
    }

    fn is_valid(&self, conn: &mut Connection) -> Result<(), Error> {
        conn.execute_batch("").map_err(Into::into)
    }

    fn has_broken(&self, _: &mut Connection) -> bool {
        false
    }
}
