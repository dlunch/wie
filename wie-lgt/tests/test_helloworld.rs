use std::{
    sync::Arc,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use test_utils::{TestPlatform, TestPlatformEvent};
use wie_backend::{Emulator, Options, extract_zip};
use wie_lgt::LgtEmulator;
use wie_util::Result;

#[test]
pub fn test_helloworld() -> Result<()> {
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

    let mut archive = extract_zip(include_bytes!("data/helloworld_lgt.zip"))?;
    assert_eq!(LgtEmulator::archive_id(&archive).as_deref(), Some("PD000000"));
    let jar = archive.remove("00000000.jar").unwrap();
    archive.insert("application.jar".into(), jar);
    let mut emulator = LgtEmulator::from_archive(
        platform,
        archive,
        Options {
            enable_gdbserver: false,
            profile: None,
        },
    )?;

    while !exited.load(Ordering::SeqCst) {
        emulator.tick()?;
    }

    let stdout_str = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
    assert_eq!(stdout_str, "Hello, world!");

    Ok(())
}

#[test]
pub fn test_helloworld_jar_under_p_prefix() -> Result<()> {
    // Archives can store the entrypoint jar under a `P/` directory. `load()` strips that prefix
    // from every filesystem key, so the classpath name must be stripped the same way, or the
    // entrypoint fails to load with "Missing binary.mod in P/...".
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

    let mut archive = extract_zip(include_bytes!("data/helloworld_lgt.zip"))?;
    let jar = archive.remove("00000000.jar").unwrap();
    archive.insert("P/00000000.jar".into(), jar);
    let mut emulator = LgtEmulator::from_archive(
        platform,
        archive,
        Options {
            enable_gdbserver: false,
            profile: None,
        },
    )?;

    while !exited.load(Ordering::SeqCst) {
        emulator.tick()?;
    }

    let stdout_str = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
    assert_eq!(stdout_str, "Hello, world!");

    Ok(())
}
