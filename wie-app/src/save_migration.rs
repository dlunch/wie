// Temporary save migration added 2026-10-19; remove with the startup call once obsolete.
use std::path::Path;

use anyhow::Result;
use rusqlite::params;
use wie_backend::{extract_zip, normalize_guest_path};

use crate::store::Store;

pub fn run(store: &Store, pid: &str, filename: &str, bytes: &[u8]) -> Result<()> {
    let mut connection = store.lock().unwrap();
    let transaction = connection.transaction()?;
    transaction.execute_batch("CREATE TABLE IF NOT EXISTS save_migrations (pid TEXT PRIMARY KEY NOT NULL)")?;
    let migrated: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM save_migrations WHERE pid = ?1)", [pid], |row| row.get(0))?;
    if migrated {
        return Ok(());
    }

    let mut aid = pid.to_owned();
    let mut clet = false;
    if Path::new(filename)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        let files = extract_zip(bytes)?;
        let descriptor = files
            .get("__adf__")
            .map(|data| (data, true))
            .or_else(|| files.get("app_info").map(|data| (data, false)));
        if let Some((descriptor, trim)) = descriptor {
            aid.clear();
            for line in descriptor.split(|byte| *byte == b'\n') {
                if let Some(value) = line.strip_prefix(b"AID:") {
                    let value = String::from_utf8_lossy(value);
                    aid = if trim { value.trim().to_owned() } else { value.into_owned() };
                } else if let Some(value) = line.strip_prefix(b"MClass:") {
                    let value = String::from_utf8_lossy(value);
                    clet = if trim { value.trim() } else { &value } == "Clet";
                }
            }
        }
    }
    transaction.execute(
        "UPDATE OR IGNORE files SET pid = ?1 WHERE pid = ?2",
        params![pid, format!("migrated_aid_{aid}")],
    )?;

    if clet {
        let records = transaction
            .prepare(
                "SELECT name, data FROM records AS source WHERE pid = ?1 AND id = 1
                AND NOT EXISTS (SELECT 1 FROM records WHERE pid = source.pid AND name = source.name AND id <> 1)
                ORDER BY name",
            )?
            .query_map([pid], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for (name, data) in records {
            let Some(path) = normalize_guest_path(&name) else {
                continue;
            };
            transaction.execute(
                "INSERT INTO files (pid, path, data) VALUES (?1, ?2, ?3) ON CONFLICT(pid, path) DO UPDATE SET data = excluded.data",
                params![pid, path, data],
            )?;
            transaction.execute("DELETE FROM records WHERE pid = ?1 AND name = ?2", params![pid, name])?;
            transaction.execute("DELETE FROM record_stores WHERE pid = ?1 AND name = ?2", params![pid, name])?;
        }
    }
    transaction.execute("INSERT INTO save_migrations (pid) VALUES (?1)", [pid])?;
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use rusqlite::params;
    use tempfile::tempdir;
    use zip::{ZipWriter, write::SimpleFileOptions};

    use crate::store;

    use super::run;

    #[test]
    fn startup_migrates_saves_once_for_supported_apps() {
        for (filename, descriptor, contents, clet) in [
            ("app.zip", "__adf__", "AID: aid\r\nMClass: Clet\r\n", true),
            ("app.zip", "app_info", "AID:aid\nMClass:Clet\n", true),
            ("app.zip", "__adf__", "AID:aid\nMClass:Main\n", false),
            ("app.zip", "app_info", "AID:aid\nMClass:Main\n", false),
            ("app.zip", "app.msd", "", false),
            ("app.jar", "META-INF/MANIFEST.MF", "", false),
        ] {
            let root = tempdir().unwrap();
            let store = store::open(root.path()).unwrap();
            let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
            archive.start_file(descriptor, SimpleFileOptions::default()).unwrap();
            archive.write_all(contents.as_bytes()).unwrap();
            let archive = archive.finish().unwrap().into_inner();
            {
                let connection = store.lock().unwrap();
                let aid = if contents.is_empty() { "pid" } else { "aid" };
                for (namespace, path, data) in [
                    (format!("migrated_aid_{aid}"), "old", b"old".as_slice()),
                    (format!("migrated_aid_{aid}"), "save", b"legacy".as_slice()),
                    ("pid".to_owned(), "save", b"current".as_slice()),
                ] {
                    connection
                        .execute("INSERT INTO files VALUES (?1, ?2, ?3)", params![namespace, path, data])
                        .unwrap();
                }
                for (name, ids) in [("/save", vec![1]), ("multi", vec![1, 2]), ("second", vec![2]), ("../invalid", vec![1])] {
                    connection.execute("INSERT INTO record_stores VALUES ('pid', ?1, 3)", [name]).unwrap();
                    for id in ids {
                        connection
                            .execute("INSERT INTO records VALUES ('pid', ?1, ?2, X'00FF')", params![name, id])
                            .unwrap();
                    }
                }
            }
            run(&store, "pid", filename, &archive).unwrap();
            {
                let connection = store.lock().unwrap();
                let saved: Vec<u8> = connection
                    .query_row("SELECT data FROM files WHERE pid = 'pid' AND path = 'save'", [], |row| row.get(0))
                    .unwrap();
                assert_eq!(saved, if clet { vec![0, 255] } else { b"current".to_vec() });
                let old: Vec<u8> = connection
                    .query_row("SELECT data FROM files WHERE pid = 'pid' AND path = 'old'", [], |row| row.get(0))
                    .unwrap();
                assert_eq!(old, b"old");
                let remaining: u32 = connection.query_row("SELECT COUNT(*) FROM record_stores", [], |row| row.get(0)).unwrap();
                assert_eq!(remaining, if clet { 3 } else { 4 });
                let records: u32 = connection.query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0)).unwrap();
                assert_eq!(records, if clet { 4 } else { 5 });
                connection.execute("DELETE FROM files WHERE pid = 'pid'", []).unwrap();
            }
            run(&store, "pid", filename, &archive).unwrap();
            let count: u32 = store
                .lock()
                .unwrap()
                .query_row("SELECT COUNT(*) FROM files WHERE pid = 'pid'", [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 0);
        }
    }

    #[test]
    fn failed_migration_rolls_back_saves_and_can_retry() {
        let root = tempdir().unwrap();
        let store = store::open(root.path()).unwrap();
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        archive.start_file("__adf__", SimpleFileOptions::default()).unwrap();
        archive.write_all(b"AID:aid\nMClass:Clet\n").unwrap();
        let archive = archive.finish().unwrap().into_inner();
        store
            .lock()
            .unwrap()
            .execute_batch(
                "INSERT INTO files VALUES ('migrated_aid_aid', 'old', X'01');
            INSERT INTO record_stores VALUES ('pid', 'save', 2);
            INSERT INTO records VALUES ('pid', 'save', 1, X'02');
            CREATE TRIGGER fail_delete BEFORE DELETE ON record_stores BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
            )
            .unwrap();
        assert!(run(&store, "pid", "app.zip", &archive).is_err());
        {
            let connection = store.lock().unwrap();
            let namespace: String = connection.query_row("SELECT pid FROM files", [], |row| row.get(0)).unwrap();
            assert_eq!(namespace, "migrated_aid_aid");
            let records: u32 = connection.query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0)).unwrap();
            assert_eq!(records, 1);
            connection.execute_batch("DROP TRIGGER fail_delete").unwrap();
        }
        run(&store, "pid", "app.zip", &archive).unwrap();
        let data: Vec<u8> = store
            .lock()
            .unwrap()
            .query_row("SELECT data FROM files WHERE pid = 'pid' AND path = 'save'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(data, [2]);
    }
}
