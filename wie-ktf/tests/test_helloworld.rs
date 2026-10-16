use std::{
    collections::BTreeMap,
    io::{Cursor, Write},
    sync::Arc,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use futures::FutureExt;
use zip::{ZipWriter, write::SimpleFileOptions};

use test_utils::{TestNativeExecutor, TestPlatform, TestPlatformEvent};
use wie_backend::{Emulator, Options, extract_zip};
use wie_ktf::{KtfEmulator, dump_image};
use wie_util::{Result, WieError};

#[test]
pub fn test_helloworld() -> Result<()> {
    let executor = TestNativeExecutor::new();
    let executed = executor.executed.clone();

    for aot in [None, Some(Box::new(executor) as _)] {
        let stdout = Arc::new(Mutex::new(Vec::new()));
        let exited = Arc::new(AtomicBool::new(false));

        let stdout_clone = stdout.clone();
        let exited_clone = exited.clone();
        let event_handler = move |event| match event {
            TestPlatformEvent::Stdout(buf) => {
                stdout_clone.lock().unwrap().extend(buf);
            }
            TestPlatformEvent::Exit => {
                exited_clone.store(true, Ordering::SeqCst);
            }
        };

        let platform = Box::new(TestPlatform::with_event_handler(event_handler));

        let archive = extract_zip(include_bytes!("data/helloworld_ktf.zip"))?;
        assert_eq!(KtfEmulator::archive_id(&archive).as_deref(), Some("PD000000"));
        let mut emulator = KtfEmulator::from_archive(
            platform,
            archive,
            Options {
                enable_gdbserver: false,
                aot,
                profile: None,
            },
        )?;

        while !exited.load(Ordering::SeqCst) {
            emulator.tick()?;
        }

        let stdout_str = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert_eq!(stdout_str, "Hello, world!");
    }

    assert!(executed.load(Ordering::Relaxed), "hello-world must execute native instructions");

    Ok(())
}

#[test]
fn test_unsupported_client_header() -> Result<()> {
    fn zip_files(files: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in files {
            zip.start_file(name, SimpleFileOptions::default()).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    for header in [
        &[0x28, 0, 0, 0, 0x1c, 0x1b, 0, 0, 0, 0, 0, 0][..],
        &[0x04, 0xe0, 0xc0, 0x46, 0x24, 0x02, 0x04, 0x20, 0x01, 0x00, 0x02],
        &[],
    ] {
        let mut archive = extract_zip(include_bytes!("data/helloworld_ktf.zip"))?;
        let mut jar = extract_zip(&archive["00000000.jar"])?;
        *jar.get_mut("client.bin12").unwrap() = header.to_vec();
        let jar = zip_files(&jar);
        assert!(KtfEmulator::loadable_jar(&jar));
        archive.insert("00000000.jar".into(), jar.clone());

        let result = KtfEmulator::from_jar(
            Box::new(TestPlatform::new()),
            "00000000.jar",
            jar,
            "",
            "",
            None,
            Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            },
        );
        assert!(matches!(result, Err(WieError::FatalError(message)) if message == "Unsupported KTF client.bin format"));

        let zip = zip_files(&archive);
        let result = KtfEmulator::from_archive(
            Box::new(TestPlatform::new()),
            archive,
            Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            },
        );
        assert!(matches!(result, Err(WieError::FatalError(message)) if message == "Unsupported KTF client.bin format"));

        let result = dump_image(&zip)
            .now_or_never()
            .expect("unsupported images must fail before native execution");
        assert!(matches!(result, Err(WieError::FatalError(message)) if message == "Unsupported KTF client.bin format"));
    }

    Ok(())
}
