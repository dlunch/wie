use std::fs;

use futures::executor::block_on;
use tempfile::tempdir;

use wie::{DatabaseRepository, DiskFilesystem};
use wie_backend::{DatabaseRepository as BackendDatabaseRepository, Filesystem, extract_zip};
use wie_ktf::KtfEmulator;

#[test]
fn disk_storage_reopens_and_isolates_apps() {
    block_on(async {
        let root = tempdir().unwrap();
        let files = extract_zip(include_bytes!("../wie-ktf/tests/data/helloworld_ktf.zip")).unwrap();
        let first = KtfEmulator::archive_id(&files).unwrap();
        let second = format!("{first}-other");

        let repository = DatabaseRepository::new(root.path().to_owned());
        let filesystem = DiskFilesystem::new(root.path().to_owned());
        let mut database = repository.open("/save0.dat", &first).await;
        let record = database.add(b"first").await;
        assert_eq!(database.next_id().await, record + 1);
        assert!(database.set(record, b"updated").await);
        assert_eq!(database.get_record_ids().await, [record]);
        assert_eq!(repository.usage(&first).await, 7);
        assert_eq!(
            fs::read(root.path().join(&first).join("db/save0.dat").join(record.to_string())).unwrap(),
            b"updated"
        );

        let mut other = repository.open("/save0.dat", &second).await;
        let other_record = other.add(b"second").await;
        assert_eq!(filesystem.write(&first, "save/game.dat", 2, b"one").await, 3);
        assert_eq!(filesystem.write(&second, "save/game.dat", 0, b"two").await, 3);

        let repository = DatabaseRepository::new(root.path().to_owned());
        let filesystem = DiskFilesystem::new(root.path().to_owned());
        let mut database = repository.open("/save0.dat", &first).await;
        assert_eq!(database.get(record).await.as_deref(), Some(b"updated".as_slice()));
        assert!(filesystem.exists(&first, "save/game.dat").await);
        assert_eq!(filesystem.size(&first, "save/game.dat").await, Some(5));
        let mut bytes = [0; 5];
        assert_eq!(filesystem.read(&first, "save/game.dat", 0, bytes.len(), &mut bytes).await, Some(5));
        assert_eq!(&bytes, b"\0\0one");
        filesystem.truncate(&first, "save/game.dat", 3).await;
        assert_eq!(filesystem.size(&first, "save/game.dat").await, Some(3));
        assert_eq!(fs::read(root.path().join(&second).join("fs/save/game.dat")).unwrap(), b"two");

        assert!(database.delete(record).await);
        assert!(database.get(record).await.is_none());
        assert!(repository.delete("/save0.dat", &first).await);
        assert!(!repository.exists("/save0.dat", &first).await);
        assert_eq!(other.get(other_record).await.as_deref(), Some(b"second".as_slice()));
        assert_eq!(repository.usage(&first).await, 0);
        assert_eq!(repository.usage(&second).await, 6);
    });
}

#[test]
fn guest_paths_stay_under_the_injected_storage_root() {
    block_on(async {
        let root = tempdir().unwrap();
        let files = extract_zip(include_bytes!("../wie-ktf/tests/data/helloworld_ktf.zip")).unwrap();
        let app = KtfEmulator::archive_id(&files).unwrap();
        let filesystem = DiskFilesystem::new(root.path().to_owned());
        for path in ["../escape", "/escape", "save/../../escape", ""] {
            assert_eq!(filesystem.write(&app, path, 0, b"data").await, 0);
            assert!(!filesystem.exists(&app, path).await);
        }

        let repository = DatabaseRepository::new(root.path().to_owned());
        let mut database = repository.open("/../save0.dat", &app).await;
        let record = database.add(b"saved").await;
        assert_eq!(
            fs::read(root.path().join(&app).join("db/_/save0.dat").join(record.to_string())).unwrap(),
            b"saved"
        );

        let blocked_root = root.path().join("file");
        fs::write(&blocked_root, b"not a directory").unwrap();
        let filesystem = DiskFilesystem::new(blocked_root);
        assert_eq!(filesystem.write(&app, "save", 0, b"data").await, 0);
        assert_eq!(filesystem.size(&app, "save").await, None);
    });
}
