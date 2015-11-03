//! Sqlite support for the `r2d2` connection pool.
extern crate r2d2;
extern crate rusqlite;

use std::error;
use std::error::Error as _StdError;
use std::fmt;

use rusqlite::SqliteError;
use rusqlite::SqliteConnection;

use std::path::Path;

#[derive(Debug)]
pub enum Error {
    Connect(SqliteError),
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
    
    pub fn new(database: &str)-> Result<SqliteConnectionManager, SqliteError>{
        match database{
            ":memory:" => {
                Ok(SqliteConnectionManager {in_memory: true, path: None})
            },
            _ => {
                Ok(SqliteConnectionManager {in_memory: false, path: Some(database.to_string())})
           }
        }
    }
}

impl r2d2::ManageConnection for SqliteConnectionManager {
    type Connection = SqliteConnection;
    type Error = Error;

    fn connect(&self) -> Result<SqliteConnection, Error> {
        if self.in_memory{
            Ok(SqliteConnection::open_in_memory().unwrap())
        }
        else if self.path.is_some(){
            println!("path: {:?}", self.path);
            Ok(SqliteConnection::open(&Path::new(self.path.as_ref().unwrap())).unwrap())
        }
        else{
            unreachable!()
        }
    }

    fn is_valid(&self, conn: &mut SqliteConnection) -> Result<(), Error> {
        conn.execute_batch("").map_err(Error::Connect)
    }

    fn has_broken(&self, conn: &mut SqliteConnection) -> bool {
        false
    }
}
