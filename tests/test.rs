extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;
extern crate tempdir;

use std::sync::mpsc;
use std::thread;

use r2d2::ManageConnection;
use tempdir::TempDir;

use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

#[test]
fn test_basic() {
    let manager = SqliteConnectionManager::file("file.db");
    let pool = r2d2::Pool::builder().max_size(2).build(manager).unwrap();

    let (s1, r1) = mpsc::channel();
    let (s2, r2) = mpsc::channel();

    let pool1 = pool.clone();
    let t1 = thread::spawn(move || {
        let conn = pool1.get().unwrap();
        s1.send(()).unwrap();
        r2.recv().unwrap();
        drop(conn);
    });

    let pool2 = pool.clone();
    let t2 = thread::spawn(move || {
        let conn = pool2.get().unwrap();
        s2.send(()).unwrap();
        r1.recv().unwrap();
        drop(conn);
    });

    t1.join().unwrap();
    t2.join().unwrap();

    pool.get().unwrap();
}

#[test]
fn test_file() {
    let manager = SqliteConnectionManager::file("file.db");
    let pool = r2d2::Pool::builder().max_size(2).build(manager).unwrap();

    let (s1, r1) = mpsc::channel();
    let (s2, r2) = mpsc::channel();

    let pool1 = pool.clone();
    let t1 = thread::spawn(move || {
        let conn = pool1.get().unwrap();
        let conn1: &Connection = &*conn;
        s1.send(()).unwrap();
        r2.recv().unwrap();
        drop(conn1);
    });

    let pool2 = pool.clone();
    let t2 = thread::spawn(move || {
        let conn = pool2.get().unwrap();
        s2.send(()).unwrap();
        r1.recv().unwrap();
        drop(conn);
    });

    t1.join().unwrap();
    t2.join().unwrap();

    pool.get().unwrap();
}

#[test]
fn test_is_valid() {
    let manager = SqliteConnectionManager::file("file.db");
    let pool = r2d2::Pool::builder()
        .max_size(1)
        .test_on_check_out(true)
        .build(manager)
        .unwrap();

    pool.get().unwrap();
}

#[test]
fn test_error_handling() {
    //! We specify a directory as a database. This is bound to fail.
    let dir = TempDir::new("r2d2-sqlite").expect("Could not create temporary directory");
    let dirpath = dir.path().to_str().unwrap();
    let manager = SqliteConnectionManager::file(dirpath);
    assert!(manager.connect().is_err());
}

#[test]
fn test_with_flags() {
    // Open db as read only and try to modify it, it should fail
    let manager = SqliteConnectionManager::file("file.db")
        .with_flags(rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY);
    let pool = r2d2::Pool::builder().max_size(2).build(manager).unwrap();
    let conn = pool.get().unwrap();
    let result = conn.execute_batch("CREATE TABLE hello(world)");
    assert!(result.is_err());
}

#[test]
fn test_with_init() {
    // Set user_version in init, then read it back to check that it was set
    let manager = SqliteConnectionManager::file("file.db")
        .with_init(|c| c.execute_batch("PRAGMA user_version=123"));
    let pool = r2d2::Pool::builder().max_size(2).build(manager).unwrap();
    let conn = pool.get().unwrap();
    let db_version = conn
        .query_row(
            "PRAGMA user_version",
            &[] as &[&rusqlite::types::ToSql],
            |r| r.get::<_, i32>(0),
        ).unwrap();
    assert_eq!(db_version, 123);
}
