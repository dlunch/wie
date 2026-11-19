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
                pid TEXT NOT NULL,
                path TEXT NOT NULL,
                data BLOB NOT NULL,
                PRIMARY KEY (pid, path)
            );
            PRAGMA user_version = 2;",
        )?;
    }
    let version: u32 = transaction.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version < 1 {
        transaction.execute_batch(
            "ALTER TABLE files RENAME COLUMN aid TO pid;
            UPDATE files SET pid = 'migrated_aid_' || pid;
            PRAGMA user_version = 1;",
        )?;
    }
    if version < 2 {
        // AID-keyed saves are unreachable from PID namespaces.
        transaction.execute_batch("DELETE FROM files WHERE pid GLOB 'migrated_aid_*'; PRAGMA user_version = 2;")?;
    }
    transaction.commit()?;
    Ok(Arc::new(Mutex::new(connection)))
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::tempdir;

    use super::open;

    fn schema(connection: &Connection) -> (u32, Vec<Option<String>>) {
        let version = connection.query_row("PRAGMA user_version", [], |row| row.get(0)).unwrap();
        let mut statement = connection.prepare("SELECT sql FROM sqlite_schema ORDER BY name").unwrap();
        let sql = statement.query_map([], |row| row.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
        (version, sql)
    }

    #[test]
    fn upgrades_initial_schema_to_latest() {
        let latest = tempdir().unwrap();
        let latest = schema(&open(latest.path()).unwrap().lock().unwrap());

        let root = tempdir().unwrap();
        Connection::open(root.path().join("state.sqlite"))
            .unwrap()
            .execute_batch(
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
            );
            INSERT INTO files VALUES ('00000000', 'save', X'00');",
            )
            .unwrap();
        let store = open(root.path()).unwrap();
        let connection = store.lock().unwrap();
        assert_eq!(schema(&connection), latest);
        let files: u32 = connection.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0)).unwrap();
        assert_eq!(files, 0);
    }
}
