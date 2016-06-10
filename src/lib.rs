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
//!     let manager = SqliteConnectionManager::new_in_memory();
//!     let pool = r2d2::Pool::new(config, manager).unwrap();
//!
//!     for i in 0..10i32 {
//!         let pool = pool.clone();
//!         thread::spawn(move || {
//!             let conn = pool.get().unwrap();
//!             conn.execute("INSERT INTO foo (bar) VALUES ($1)", &[&i]).unwrap();
//!         });
//!     }
//! }
//! ```
extern crate r2d2;
extern crate rusqlite;

use std::error::{self, Error as StdError};
use std::fmt;
use std::convert;

use rusqlite::{Connection, Error as RusqliteError, OpenFlags};

/// A unified enum of errors returned by `rusqlite::Connection`
#[derive(Debug)]
pub enum Error {
    Connect(RusqliteError),
}

impl convert::From<RusqliteError> for Error {
    fn from(e: RusqliteError) -> Error {
        Error::Connect(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: {}", self.description(), self.cause().unwrap())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Connect(_) => "Error opening a connection",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Connect(ref err) => Some(err as &error::Error),
        }
    }
}


enum ConnectionConfig {
    InMemory(OpenFlags),
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

    /// Creates a new `SqliteConnectionManager` in memory.
    ///
    /// See `rusqlite::Connection::open_in_memory`
    pub fn new_in_memory() -> Self {
        Self::new_in_memory_with_flags(OpenFlags::default())
    }

    /// Creates a new `SqliteConnectionManager` in memory with open flags.
    ///
    /// See `rusqlite::Connection::open_in_memory_with_flags`
    pub fn new_in_memory_with_flags(flags: OpenFlags) -> Self {
        SqliteConnectionManager { config: ConnectionConfig::InMemory(flags) }
    }
}

impl r2d2::ManageConnection for SqliteConnectionManager {
    type Connection = Connection;
    type Error = Error;

    fn connect(&self) -> Result<Connection, Error> {
        match self.config {
                ConnectionConfig::InMemory(flags) => Connection::open_in_memory_with_flags(flags),
                ConnectionConfig::File(ref path, flags) => Connection::open_with_flags(path, flags),
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
