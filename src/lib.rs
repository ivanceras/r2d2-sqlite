//! Sqlite support for the `r2d2` connection pool.
extern crate r2d2;
extern crate rusqlite;

use std::error::{self, Error as StdError};
use std::fmt;
use std::convert;

use rusqlite::{Connection, Error as RusqliteError};

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


pub struct SqliteConnectionManager {
    in_memory: bool,
    path: Option<String>,
}

impl SqliteConnectionManager {
    pub fn new(database: &str) -> Result<SqliteConnectionManager, RusqliteError> {
        match database {
            ":memory:" => {
                Ok(SqliteConnectionManager {
                    in_memory: true,
                    path: None,
                })
            }
            _ => {
                Ok(SqliteConnectionManager {
                    in_memory: false,
                    path: Some(database.to_string()),
                })
            }
        }
    }
}

impl r2d2::ManageConnection for SqliteConnectionManager {
    type Connection = Connection;
    type Error = Error;

    fn connect(&self) -> Result<Connection, Error> {
        if self.in_memory {
            Connection::open_in_memory().map_err(Into::into)
        } else {
            match self.path {
                Some(ref path) => Connection::open(path).map_err(Into::into),
                None => unreachable!(),
            }
        }
    }

    fn is_valid(&self, conn: &mut Connection) -> Result<(), Error> {
        conn.execute_batch("").map_err(Into::into)
    }

    fn has_broken(&self, _: &mut Connection) -> bool {
        false
    }
}
