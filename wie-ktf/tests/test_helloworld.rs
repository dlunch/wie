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

fn zip_files(files: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in files {
        zip.start_file(name, SimpleFileOptions::default()).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

#[test]
pub fn test_helloworld() -> Result<()> {
    let executor = TestNativeExecutor::new();
    let executed = executor.executed.clone();

    for (aot, standalone_jar) in [(None, false), (Some(Box::new(executor) as _), false), (None, true)] {
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
        let options = Options { aot, ..Default::default() };
        let mut emulator = if standalone_jar {
            let mut jar = extract_zip(&archive["00000000.jar"])?;
            jar.insert("META-INF/MANIFEST.MF".into(), b"MIDlet-1: Hello, /icon.png,\r\n Clet\r\n".to_vec());
            KtfEmulator::from_jar(platform, "00000000.jar", zip_files(&jar), "", "", options)?
        } else {
            KtfEmulator::from_archive(platform, archive, options)?
        };

        while !exited.load(Ordering::SeqCst) {
            emulator.tick()?;
        }

        let stdout_str = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert_eq!(stdout_str, "Hello, world! 2024\n");
    }

    assert!(executed.load(Ordering::Relaxed), "hello-world must execute native instructions");

    Ok(())
}

#[test]
fn test_unsupported_client_header() -> Result<()> {
    let header = [0x28, 0, 0, 0, 0x1c, 0x1b, 0, 0, 0, 0, 0, 0];
    let mut archive = extract_zip(include_bytes!("data/helloworld_ktf.zip"))?;
    let mut jar = extract_zip(&archive["00000000.jar"])?;
    *jar.get_mut("client.bin36").unwrap() = header.to_vec();
    let jar = zip_files(&jar);
    assert!(KtfEmulator::loadable_jar(&jar));
    archive.insert("00000000.jar".into(), jar.clone());

    let result = KtfEmulator::from_jar(Box::new(TestPlatform::new()), "00000000.jar", jar, "", "", Options::default());
    assert!(matches!(result, Err(WieError::FatalError(message)) if message == "Unsupported KTF client.bin format"));

    let zip = zip_files(&archive);
    let result = KtfEmulator::from_archive(Box::new(TestPlatform::new()), archive, Options::default());
    assert!(matches!(result, Err(WieError::FatalError(message)) if message == "Unsupported KTF client.bin format"));

    let result = dump_image(&zip)
        .now_or_never()
        .expect("unsupported images must fail before native execution");
    assert!(matches!(result, Err(WieError::FatalError(message)) if message == "Unsupported KTF client.bin format"));

    Ok(())
}
