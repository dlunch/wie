use rusqlite::{Result, Transaction};

// Index `n` upgrades a database from `user_version` n to n + 1.
const UPGRADES: [fn(&Transaction) -> Result<()>; 2] = [rename_files_namespace, delete_aid_files];

pub fn run(transaction: &Transaction) -> Result<()> {
    let initialized: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type = 'table')", [], |row| row.get(0))?;
    if !initialized {
        create_initial(transaction)?;
    }
    let version: usize = transaction.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    for upgrade in &UPGRADES[version..] {
        upgrade(transaction)?;
    }
    transaction.pragma_update(None, "user_version", UPGRADES.len())
}

fn create_initial(transaction: &Transaction) -> Result<()> {
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
    )
}

fn rename_files_namespace(transaction: &Transaction) -> Result<()> {
    transaction.execute_batch(
        "ALTER TABLE files RENAME COLUMN aid TO pid;
        UPDATE files SET pid = 'migrated_aid_' || pid;",
    )
}

// AID-keyed saves are unreachable from PID namespaces.
fn delete_aid_files(transaction: &Transaction) -> Result<()> {
    transaction.execute_batch("DELETE FROM files WHERE pid GLOB 'migrated_aid_*';")
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::tempdir;

    use crate::store::open;

    use super::create_initial;

    #[test]
    fn upgrades_initial_schema_and_deletes_aid_files() {
        let root = tempdir().unwrap();
        let mut connection = Connection::open(root.path().join("state.sqlite")).unwrap();
        let transaction = connection.transaction().unwrap();
        create_initial(&transaction).unwrap();
        transaction.execute("INSERT INTO files VALUES ('00000000', 'save', X'00')", []).unwrap();
        transaction.commit().unwrap();

        let store = open(root.path()).unwrap();
        let connection = store.lock().unwrap();
        let version: u32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0)).unwrap();
        assert_eq!(version, 2);
        let files: u32 = connection.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0)).unwrap();
        assert_eq!(files, 0);
    }
}
