use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use rusqlite::Connection;

// ponytail: one connection serializes guest storage; split connections if contention is measured.
pub(crate) type Store = Arc<Mutex<Connection>>;

pub(crate) fn open(root: &Path) -> Result<Store> {
    fs::create_dir_all(root)?;
    let mut connection = Connection::open(root.join("state.sqlite"))?;
    let transaction = connection.transaction()?;
    let initialized: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type = 'table')", [], |row| row.get(0))?;
    if !initialized {
        transaction.execute_batch(
            "CREATE TABLE record_stores (
                pid TEXT NOT NULL,
                name TEXT NOT NULL,
                next_id INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (pid, name)
            );
            CREATE TABLE records (
                pid TEXT NOT NULL,
                name TEXT NOT NULL,
                id INTEGER NOT NULL,
                data BLOB NOT NULL,
                PRIMARY KEY (pid, name, id),
                FOREIGN KEY (pid, name) REFERENCES record_stores(pid, name) ON DELETE CASCADE
            );
            CREATE TABLE files (
                aid TEXT NOT NULL,
                path TEXT NOT NULL,
                data BLOB NOT NULL,
                PRIMARY KEY (aid, path)
            );",
        )?;
    }
    let version: u32 = transaction.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version == 0 {
        transaction.execute_batch(
            "DROP TABLE files;
            CREATE TABLE files (
                pid TEXT NOT NULL,
                path TEXT NOT NULL,
                data BLOB NOT NULL,
                PRIMARY KEY (pid, path)
            );
            PRAGMA user_version = 1;",
        )?;
    }
    transaction.commit()?;
    Ok(Arc::new(Mutex::new(connection)))
}
