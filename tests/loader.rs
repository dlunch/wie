use std::{
    io::{Cursor, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use test_utils::{TestPlatform, TestPlatformEvent};
use wie::{extract_app_metadata, load_emulator};
use wie_backend::{Options, extract_zip};
use zip::{ZipWriter, write::SimpleFileOptions};

#[test]
fn imported_archives_run_through_the_shared_loader() -> anyhow::Result<()> {
    for (filename, bytes) in [
        ("hello.ZIP", include_bytes!("../wie-ktf/tests/data/helloworld_ktf.zip").as_slice()),
        ("hello.zip", include_bytes!("../wie-lgt/tests/data/helloworld_lgt.zip").as_slice()),
    ] {
        let mut archive = extract_zip(bytes)?;
        for (name, data) in &mut archive {
            if name == "__adf__" || name == "app_info" {
                data.extend_from_slice(b"\nName:Hello\n");
            }
        }
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in archive {
            writer.start_file(name, SimpleFileOptions::default())?;
            writer.write_all(&data)?;
        }
        let bytes = writer.finish()?.into_inner();
        let metadata = extract_app_metadata(filename, &bytes)?;
        assert_eq!(metadata.id, "PD000000");
        assert!(!metadata.title.is_empty());
        let stdout = Arc::new(Mutex::new(Vec::new()));
        let exited = Arc::new(AtomicBool::new(false));
        let output = stdout.clone();
        let exit = exited.clone();
        let platform = TestPlatform::with_event_handler(move |event| match event {
            TestPlatformEvent::Stdout(bytes) => output.lock().unwrap().extend(bytes),
            TestPlatformEvent::Exit => exit.store(true, Ordering::SeqCst),
        });
        let mut emulator = load_emulator(
            filename,
            bytes,
            Box::new(platform),
            Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            },
        )?;
        for _ in 0..10_000 {
            if exited.load(Ordering::SeqCst) {
                break;
            }
            emulator.tick()?;
        }
        assert!(exited.load(Ordering::SeqCst));
        assert_eq!(*stdout.lock().unwrap(), b"Hello, world!");
    }
    assert!(extract_app_metadata("bad.zip", b"not an archive").is_err());
    assert!(extract_app_metadata("bad.txt", b"not an archive").is_err());
    Ok(())
}
