use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, JavaError, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class net.wie.WieAudioClip
pub struct WieAudioClip;

impl WieAudioClip {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/WieAudioClip",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["com/skt/m/AudioClip"],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("open", "([BII)V", Self::open, MethodAccessFlags::PUBLIC | MethodAccessFlags::SYNCHRONIZED),
                JavaMethodProto::new("close", "()V", Self::close, MethodAccessFlags::PUBLIC | MethodAccessFlags::SYNCHRONIZED),
                JavaMethodProto::new("loop", "()V", Self::r#loop, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("pause", "()V", Self::pause, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("play", "()V", Self::play, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("resume", "()V", Self::resume, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("stop", "()V", Self::stop, MethodAccessFlags::PUBLIC | MethodAccessFlags::SYNCHRONIZED),
                JavaMethodProto::new(
                    "playback",
                    "(Z)V",
                    Self::playback,
                    MethodAccessFlags::PRIVATE | MethodAccessFlags::SYNCHRONIZED,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("audioHandle", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("playing", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("generation", "J", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "net/wie/WieAudioClip", "audioHandle", "I", -1).await
    }

    async fn open(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        buffer_size: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::open({this:?}, {data:?}, {offset}, {buffer_size})");

        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "data is null").await);
        }

        let array_length = jvm.array_length(&data).await?;
        if offset < 0 || buffer_size < 0 || (offset as usize).checked_add(buffer_size as usize).is_none_or(|end| end > array_length) {
            return Err(jvm
                .exception("java/lang/ArrayIndexOutOfBoundsException", "Invalid offset or length")
                .await);
        }

        let data: Vec<i8> = jvm.load_array(&data, offset as usize, buffer_size as usize).await?;
        let data: &[u8] = bytemuck::cast_slice(&data);
        if !data.starts_with(b"MMMD") {
            return Err(jvm.exception("com/skt/m/UnsupportedFormatException", "Expected SMAF audio data").await);
        }

        let _: () = jvm.invoke_virtual(&this, "net/wie/WieAudioClip", "stop", "()V", ()).await?;
        let _: () = jvm.invoke_virtual(&this, "net/wie/WieAudioClip", "close", "()V", ()).await?;
        let handle = context.system().audio().load_smaf(data);
        let handle = match handle {
            Ok(handle) => handle,
            Err(_) => {
                return Err(JavaError::JavaException(
                    jvm.new_class("com/skt/m/ResourceAllocException", "()V", ()).await?,
                ));
            }
        };
        jvm.put_field(&mut this, "net/wie/WieAudioClip", "audioHandle", "I", handle as i32).await
    }

    async fn play(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::play({this:?})");
        jvm.invoke_special(&this, "net/wie/WieAudioClip", "playback", "(Z)V", (false,)).await
    }

    async fn r#loop(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::loop({this:?})");
        jvm.invoke_special(&this, "net/wie/WieAudioClip", "playback", "(Z)V", (true,)).await
    }

    async fn playback(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, repeat: bool) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::playback({this:?}, {repeat})");

        let _: () = jvm.invoke_virtual(&this, "net/wie/WieAudioClip", "stop", "()V", ()).await?;
        let handle: i32 = jvm.get_field(&this, "net/wie/WieAudioClip", "audioHandle", "I").await?;
        let duration = context.system().audio().play(handle as u32, repeat);
        let duration = match duration {
            Ok(duration) => duration,
            Err(_) => return Err(jvm.exception("java/io/IOException", "Audio clip is not open").await),
        };
        let generation: i64 = jvm.get_field(&this, "net/wie/WieAudioClip", "generation", "J").await?;
        jvm.put_field(&mut this, "net/wie/WieAudioClip", "playing", "Z", true).await?;
        let deadline = context.system().platform().now() + duration;

        loop {
            if jvm.get_field::<i64>(&this, "net/wie/WieAudioClip", "generation", "J").await? != generation {
                return Err(JavaError::JavaException(jvm.new_class("com/skt/m/UserStopException", "()V", ()).await?));
            }
            let now = context.system().platform().now();
            if !repeat && now >= deadline {
                context.system().audio().stop(handle as u32);
                jvm.put_field(&mut this, "net/wie/WieAudioClip", "playing", "Z", false).await?;
                return Ok(());
            }
            let timeout = if repeat { 0 } else { (deadline - now) as i64 };
            let wait: JvmResult<()> = jvm.invoke_virtual(&this, "java/lang/Object", "wait", "(J)V", (timeout,)).await;
            if let Err(error) = wait {
                if jvm.get_field::<i64>(&this, "net/wie/WieAudioClip", "generation", "J").await? == generation {
                    context.system().audio().stop(handle as u32);
                    jvm.put_field(&mut this, "net/wie/WieAudioClip", "playing", "Z", false).await?;
                }
                return match error {
                    JavaError::JavaException(exception) if jvm.is_instance(&*exception, "java/lang/InterruptedException") => {
                        Err(jvm.exception("java/io/IOException", "Audio playback interrupted").await)
                    }
                    error => Err(error),
                };
            }
        }
    }

    async fn pause(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub net.wie.WieAudioClip::pause({this:?})");

        Ok(())
    }

    async fn resume(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub net.wie.WieAudioClip::resume({this:?})");

        Ok(())
    }

    async fn stop(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::stop({this:?})");

        if jvm.get_field::<bool>(&this, "net/wie/WieAudioClip", "playing", "Z").await? {
            let handle: i32 = jvm.get_field(&this, "net/wie/WieAudioClip", "audioHandle", "I").await?;
            context.system().audio().stop(handle as u32);
            let generation: i64 = jvm.get_field(&this, "net/wie/WieAudioClip", "generation", "J").await?;
            jvm.put_field(&mut this, "net/wie/WieAudioClip", "generation", "J", generation.wrapping_add(1))
                .await?;
            jvm.put_field(&mut this, "net/wie/WieAudioClip", "playing", "Z", false).await?;
            let _: () = jvm.invoke_virtual(&this, "java/lang/Object", "notifyAll", "()V", ()).await?;
        }
        Ok(())
    }

    async fn close(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::close({this:?})");

        if jvm.get_field::<bool>(&this, "net/wie/WieAudioClip", "playing", "Z").await? {
            return Err(jvm.exception("java/io/IOException", "Audio clip is playing").await);
        }
        let handle: i32 = jvm.get_field(&this, "net/wie/WieAudioClip", "audioHandle", "I").await?;
        if handle != -1 {
            let result = context.system().audio().close(handle as u32);
            if result.is_err() {
                return Err(jvm.exception("java/io/IOException", "Unable to close audio clip").await);
            }
            jvm.put_field(&mut this, "net/wie/WieAudioClip", "audioHandle", "I", -1).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    use jvm::{Array, ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::{String, Thread};
    use test_utils::{TestClock, TestPlatform, run_jvm_test_with_system};
    use wie_backend::{AudioCommand, AudioSink, DatabaseRepository, Filesystem, Font, Instant, Platform, Screen, System};

    use crate::{classes::com::skt::m::AudioClip, get_protos};

    struct AudioPlatform {
        inner: TestPlatform,
        commands: Arc<Mutex<Vec<AudioCommand>>>,
    }

    struct RecordingSink(Arc<Mutex<Vec<AudioCommand>>>);

    impl AudioSink for RecordingSink {
        fn send(&self, command: AudioCommand) {
            self.0.lock().unwrap().push(command);
        }
    }

    impl Platform for AudioPlatform {
        fn font(&self) -> &Font {
            self.inner.font()
        }
        fn screen(&self) -> &dyn Screen {
            self.inner.screen()
        }
        fn now(&self) -> Instant {
            self.inner.now()
        }
        fn database_repository(&self) -> &dyn DatabaseRepository {
            self.inner.database_repository()
        }
        fn filesystem(&self) -> &dyn Filesystem {
            self.inner.filesystem()
        }
        fn audio_sink(&self) -> Box<dyn AudioSink> {
            Box::new(RecordingSink(self.commands.clone()))
        }
        fn write_stdout(&self, bytes: &[u8]) {
            self.inner.write_stdout(bytes);
        }
        fn write_stderr(&self, bytes: &[u8]) {
            self.inner.write_stderr(bytes);
        }
        fn exit(&self) {
            self.inner.exit();
        }
        fn vibrate(&self, duration: u64, intensity: u8) {
            self.inner.vibrate(duration, intensity);
        }
    }

    fn start_playback(
        system: &System,
        jvm: Jvm,
        clip: ClassInstanceRef<AudioClip>,
        method: &'static str,
        result: Arc<AtomicUsize>,
    ) -> Arc<Mutex<Option<ClassInstanceRef<Thread>>>> {
        let thread = Arc::new(Mutex::new(None));
        let current_thread = thread.clone();
        system.spawn(async move || {
            jvm.attach_thread(None).await.unwrap();
            *current_thread.lock().unwrap() = Some(jvm.current_java_thread().into());
            let playback: JvmResult<()> = jvm.invoke_virtual(&clip, "com/skt/m/AudioClip", method, "()V", ()).await;
            let outcome = match playback {
                Ok(()) => 1,
                Err(JavaError::JavaException(exception)) if jvm.is_instance(&*exception, "com/skt/m/UserStopException") => 2,
                Err(JavaError::JavaException(exception)) if jvm.is_instance(&*exception, "java/io/IOException") => 3,
                Err(error) => panic!("unexpected playback result: {error:?}"),
            };
            result.store(outcome, Ordering::SeqCst);
            jvm.detach_thread().unwrap();
            Ok(())
        });
        thread
    }

    #[test]
    fn audio_clip_blocks_until_media_end_and_preserves_stopped_playback_across_replay() {
        let clock = TestClock::new();
        let commands = Arc::new(Mutex::new(Vec::new()));
        let platform = AudioPlatform {
            inner: TestPlatform::with_clock(clock.clone()),
            commands: commands.clone(),
        };
        let result = run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            move |jvm, system| async move {
                let formats: ClassInstanceRef<Array<String>> = jvm
                    .invoke_static("com/skt/m/AudioSystem", "getClipFormats", "()[Ljava/lang/String;", ())
                    .await?;
                let formats: Vec<ClassInstanceRef<String>> = jvm.load_array(&formats, 0, 1).await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &formats[0]).await?, "mmf");
                let clip: ClassInstanceRef<AudioClip> = jvm
                    .invoke_static(
                        "com/skt/m/AudioSystem",
                        "getAudioClip",
                        "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                        (formats[0].clone(),),
                    )
                    .await?;

                // A mobile-standard SMAF track with one note ending after 100 ms.
                let mut track = vec![2, 0, 0, 0];
                track.extend_from_slice(&[0; 16]);
                track.extend_from_slice(b"Mtsq\0\0\0\x09\0\x90\x3c\x40\x64\x64\xff\x2f\0");
                let mut smaf = b"MMMD".to_vec();
                smaf.extend_from_slice(&(track.len() as u32 + 10).to_be_bytes());
                smaf.extend_from_slice(b"MTR\0");
                smaf.extend_from_slice(&(track.len() as u32).to_be_bytes());
                smaf.extend(track);
                smaf.extend_from_slice(&[0, 0]);
                let length = smaf.len() as i32;
                let mut data = jvm.instantiate_array("B", smaf.len() + 2).await?;
                jvm.store_array(&mut data, 1, smaf.into_iter().map(|byte| byte as i8)).await?;
                let data: ClassInstanceRef<Array<i8>> = data.into();
                let _: () = jvm
                    .invoke_virtual(&clip, "com/skt/m/AudioClip", "open", "([BII)V", (data.clone(), 1, length))
                    .await?;

                for (offset, length) in [(-1, 1), (0, -1), (length + 2, 1), (length + 1, 2)] {
                    let invalid: JvmResult<()> = jvm
                        .invoke_virtual(&clip, "com/skt/m/AudioClip", "open", "([BII)V", (data.clone(), offset, length))
                        .await;
                    let Err(JavaError::JavaException(exception)) = invalid else {
                        panic!("accepted invalid audio slice")
                    };
                    assert!(jvm.is_instance(&*exception, "java/lang/ArrayIndexOutOfBoundsException"));
                }
                let null: JvmResult<()> = jvm.invoke_virtual(&clip, "com/skt/m/AudioClip", "open", "([BII)V", (None, 0, 0)).await;
                let Err(JavaError::JavaException(exception)) = null else {
                    panic!("accepted null audio")
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));
                let empty: JvmResult<()> = jvm
                    .invoke_virtual(&clip, "com/skt/m/AudioClip", "open", "([BII)V", (data.clone(), length + 2, 0))
                    .await;
                let Err(JavaError::JavaException(exception)) = empty else {
                    panic!("accepted unsupported audio data")
                };
                assert!(jvm.is_instance(&*exception, "com/skt/m/UnsupportedFormatException"));

                let completed = Arc::new(AtomicUsize::new(0));
                start_playback(&system, jvm.clone(), clip.clone(), "play", completed.clone());
                for _ in 0..10 {
                    system.yield_now().await;
                }
                assert_eq!(completed.load(Ordering::SeqCst), 0);
                {
                    let commands = commands.lock().unwrap();
                    let AudioCommand::Play { sequence, repeat: false, .. } = &commands[0] else {
                        panic!("expected finite playback")
                    };
                    assert_eq!(sequence.duration, 100);
                    assert!(!sequence.events.is_empty());
                }
                clock.set(99);
                for _ in 0..10 {
                    system.yield_now().await;
                }
                assert_eq!(completed.load(Ordering::SeqCst), 0);
                clock.set(100);
                for _ in 0..10 {
                    system.yield_now().await;
                }
                assert_eq!(completed.load(Ordering::SeqCst), 1);

                for (method, interrupt) in [("loop", false), ("play", false), ("play", true)] {
                    let stopped = Arc::new(AtomicUsize::new(0));
                    let thread = start_playback(&system, jvm.clone(), clip.clone(), method, stopped.clone());
                    for _ in 0..10 {
                        system.yield_now().await;
                    }
                    if method == "loop" {
                        clock.advance(10_000);
                        for _ in 0..10 {
                            system.yield_now().await;
                        }
                    }
                    assert_eq!(stopped.load(Ordering::SeqCst), 0);
                    let close: JvmResult<()> = jvm.invoke_virtual(&clip, "com/skt/m/AudioClip", "close", "()V", ()).await;
                    let Err(JavaError::JavaException(exception)) = close else {
                        panic!("closed active audio")
                    };
                    assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                    // Hold the monitor until the replay is queued ahead of the stopped waiter.
                    jvm.monitor_enter(&clip).await?;
                    let replayed = Arc::new(AtomicUsize::new(0));
                    start_playback(&system, jvm.clone(), clip.clone(), "play", replayed.clone());
                    for _ in 0..10 {
                        system.yield_now().await;
                    }
                    let _: () = jvm.invoke_virtual(&clip, "com/skt/m/AudioClip", "stop", "()V", ()).await?;
                    if interrupt {
                        let thread = thread.lock().unwrap().as_ref().unwrap().clone();
                        let _: () = jvm.invoke_virtual(&thread, "java/lang/Thread", "interrupt", "()V", ()).await?;
                    }
                    for _ in 0..10 {
                        system.yield_now().await;
                    }
                    jvm.monitor_exit(&clip).await?;
                    for _ in 0..10 {
                        system.yield_now().await;
                    }
                    assert_eq!(stopped.load(Ordering::SeqCst), if interrupt { 3 } else { 2 });
                    assert_eq!(replayed.load(Ordering::SeqCst), 0);
                    assert!(matches!(commands.lock().unwrap().last(), Some(AudioCommand::Play { repeat: false, .. })));
                    clock.advance(100);
                    for _ in 0..10 {
                        system.yield_now().await;
                    }
                    assert_eq!(replayed.load(Ordering::SeqCst), 1);
                }
                let interrupted = Arc::new(AtomicUsize::new(0));
                let thread = start_playback(&system, jvm.clone(), clip.clone(), "loop", interrupted.clone());
                for _ in 0..10 {
                    system.yield_now().await;
                }
                let thread = thread.lock().unwrap().as_ref().unwrap().clone();
                let _: () = jvm.invoke_virtual(&thread, "java/lang/Thread", "interrupt", "()V", ()).await?;
                for _ in 0..10 {
                    system.yield_now().await;
                }
                assert_eq!(interrupted.load(Ordering::SeqCst), 3);
                assert!(matches!(commands.lock().unwrap().last(), Some(AudioCommand::Stop { .. })));
                let _: () = jvm.invoke_virtual(&clip, "com/skt/m/AudioClip", "close", "()V", ()).await?;
                Ok(())
            },
        );
        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
