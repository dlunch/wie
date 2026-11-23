use std::{
    io::{Cursor, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use zip::{ZipWriter, write::SimpleFileOptions};

use test_utils::{TestPlatform, TestPlatformEvent};
use wie::load_emulator;
use wie_backend::{Options, extract_zip};

#[test]
fn standalone_ktf_jar_starts_from_manifest_entry_point() -> anyhow::Result<()> {
    let archive = extract_zip(include_bytes!("../wie-ktf/tests/data/helloworld_ktf.zip"))?;
    let mut jar = extract_zip(&archive["00000000.jar"])?;
    jar.insert("META-INF/MANIFEST.MF".into(), b"MIDlet-1: Hello, , Clet\n".to_vec());
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in &jar {
        writer.start_file(name, SimpleFileOptions::default())?;
        writer.write_all(data)?;
    }
    let jar = writer.finish()?.into_inner();

    let stdout = Arc::new(Mutex::new(Vec::new()));
    let exited = Arc::new(AtomicBool::new(false));
    let stdout_clone = stdout.clone();
    let exited_clone = exited.clone();
    let platform = Box::new(TestPlatform::with_event_handler(move |event| match event {
        TestPlatformEvent::Stdout(buf) => stdout_clone.lock().unwrap().extend(buf),
        TestPlatformEvent::Exit => exited_clone.store(true, Ordering::SeqCst),
    }));

    let mut emulator = load_emulator(
        "00000000.jar",
        jar,
        platform,
        Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        },
    )?;
    while !exited.load(Ordering::SeqCst) {
        emulator.tick()?;
    }

    assert_eq!(String::from_utf8(stdout.lock().unwrap().clone())?, "Hello, world!");

    Ok(())
}
