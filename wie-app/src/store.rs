use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use rusqlite::Connection;

mod migration;

// ponytail: one connection serializes guest storage; split connections if contention is measured.
pub(crate) type Store = Arc<Mutex<Connection>>;

pub(crate) fn open(root: &Path) -> Result<Store> {
    fs::create_dir_all(root)?;
    let mut connection = Connection::open(root.join("state.sqlite"))?;
    let transaction = connection.transaction()?;
    migration::run(&transaction)?;
    transaction.commit()?;
    Ok(Arc::new(Mutex::new(connection)))
}

pub fn delete_app_storage(store: &Store, pid: &str) -> Result<()> {
    let mut connection = store.lock().unwrap();
    let transaction = connection.transaction()?;
    transaction.execute("DELETE FROM record_stores WHERE pid = ?1", [pid])?;
    transaction.execute("DELETE FROM files WHERE pid = ?1", [pid])?;
    transaction.commit()?;
    Ok(())
}
