use std::{
    sync::Arc,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

#[cfg(all(not(target_os = "ios"), any(target_arch = "x86_64", target_arch = "aarch64")))]
use test_utils::TestNativeExecutor;
use test_utils::{TestPlatform, TestPlatformEvent};
use wie_backend::{Emulator, Options, extract_zip};
use wie_ktf::KtfEmulator;
use wie_util::Result;

#[test]
pub fn test_helloworld() -> Result<()> {
    #[cfg(all(not(target_os = "ios"), any(target_arch = "x86_64", target_arch = "aarch64")))]
    let executor = TestNativeExecutor::new();
    #[cfg(all(not(target_os = "ios"), any(target_arch = "x86_64", target_arch = "aarch64")))]
    let retired = executor.retired.clone();

    for aot in [
        None,
        #[cfg(all(not(target_os = "ios"), any(target_arch = "x86_64", target_arch = "aarch64")))]
        Some(Box::new(executor) as _),
    ] {
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

    #[cfg(all(not(target_os = "ios"), any(target_arch = "x86_64", target_arch = "aarch64")))]
    assert!(retired.load(Ordering::Relaxed) > 0, "hello-world must execute native instructions");

    Ok(())
}
