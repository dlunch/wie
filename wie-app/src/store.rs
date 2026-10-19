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
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS record_stores (
            pid TEXT NOT NULL,
            name TEXT NOT NULL,
            next_id INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY (pid, name)
        );
        CREATE TABLE IF NOT EXISTS records (
            pid TEXT NOT NULL,
            name TEXT NOT NULL,
            id INTEGER NOT NULL,
            data BLOB NOT NULL,
            PRIMARY KEY (pid, name, id),
            FOREIGN KEY (pid, name) REFERENCES record_stores(pid, name) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS files (
            pid TEXT NOT NULL,
            path TEXT NOT NULL,
            data BLOB NOT NULL,
            PRIMARY KEY (pid, path)
        );",
    )?;
    let has_aid: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM pragma_table_info('files') WHERE name = 'aid')", [], |row| {
        row.get(0)
    })?;
    if has_aid {
        transaction.execute_batch("ALTER TABLE files RENAME COLUMN aid TO pid")?;
    }
    transaction.commit()?;
    Ok(Arc::new(Mutex::new(connection)))
}
