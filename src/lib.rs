//! Sqlite support for the `r2d2` connection pool.
extern crate r2d2;
extern crate rusqlite;

use std::error::{self, Error as StdError};
use std::fmt;
use std::convert;

use rusqlite::{Connection, Error as RusqliteError, OpenFlags};

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

pub struct SqliteConnectionManager {
    config: ConnectionConfig,
}

impl SqliteConnectionManager {
    pub fn new(database: &str) -> Self {
        Self::new_with_flags(database, OpenFlags::default())
    }

    pub fn new_with_flags(database: &str, flags: OpenFlags) -> Self {
        SqliteConnectionManager { config: ConnectionConfig::File(database.to_string(), flags) }
    }

    pub fn new_in_memory() -> Self {
        Self::new_in_memory_with_flags(OpenFlags::default())
    }

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
