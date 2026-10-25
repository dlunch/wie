use core::{pin::Pin, task::Poll};

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

use encoding_rs::EUC_KR;
use futures::future::poll_fn;
use jvm::{
    JavaError,
    runtime::{JavaIoInputStream, JavaLangClassLoader},
};

use wie_backend::{Emulator, Event, Options, Platform, System, TaskRunner, extract_zip};
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, write_generic};

use crate::runtime::{LgtJvmSupport, SUPPORT_CONTEXT_BASE, exception, init::load_native};

struct LgtTaskRunner {
    core: ArmCore,
}

struct JavaSupportContextGuard {
    core: ArmCore,
    address: Option<u32>,
}

impl Drop for JavaSupportContextGuard {
    fn drop(&mut self) {
        if let Some(address) = self.address
            && let Err(error) = exception::destroy(&mut self.core, address)
        {
            tracing::error!("Failed to free LGT support context: {error}");
        }
    }
}

#[async_trait::async_trait]
impl TaskRunner for LgtTaskRunner {
    async fn run(&self, mut future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()> {
        let mut core = self.core.clone();
        let ptr_context = exception::init(&mut core)?;
        // Declared before the wrapper so its guest context outlives thread teardown.
        let mut support_context = JavaSupportContextGuard {
            core: core.clone(),
            address: Some(ptr_context),
        };
        let mut poll_core = core.clone();
        let result = core
            .run_in_thread(move || {
                poll_fn(move |context| {
                    if let Err(error) = poll_core
                        .set_thread_runtime_context(ptr_context)
                        .and_then(|()| write_generic(&mut poll_core, SUPPORT_CONTEXT_BASE, ptr_context))
                    {
                        return Poll::Ready(Err(error));
                    }
                    future.as_mut().poll(context)
                })
            })?
            .await;
        exception::destroy(&mut core, support_context.address.take().unwrap())?;
        result
    }
}

pub struct LgtEmulator {
    core: ArmCore,
    system: System,
}

impl LgtEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>, options: Options) -> Result<Self> {
        let app_info = files
            .get("app_info")
            .ok_or_else(|| WieError::FatalError("Missing app_info in LGT archive".into()))?;
        let app_info = LgtAppInfo::parse(app_info);

        tracing::info!("Loading app {}, pid {}, mclass {}", app_info.aid, app_info.pid, app_info.mclass);

        let jar_filename = files
            .iter()
            .find_map(|(filename, data)| (filename.ends_with(".jar") && Self::loadable_jar(data)).then_some(filename))
            .ok_or_else(|| WieError::FatalError("Missing LGT application JAR containing binary.mod".into()))?;

        Self::load(
            platform,
            jar_filename,
            &app_info.pid,
            &app_info.aid,
            Some(app_info.mclass),
            &files,
            options,
        )
    }

    pub fn from_jar(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        jar: Vec<u8>,
        pid: &str,
        aid: &str,
        main_class_name: Option<String>,
        options: Options,
    ) -> Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, pid, aid, main_class_name, &files, options)
    }

    pub fn loadable_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.contains_key("app_info")
    }

    pub fn archive_title(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let title = LgtAppInfo::parse(files.get("app_info")?).name;
        (!title.is_empty()).then_some(title)
    }

    pub fn archive_id(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let id = LgtAppInfo::parse(files.get("app_info")?).pid;
        (!id.is_empty()).then_some(id)
    }

    pub fn archive_icon(files: &BTreeMap<String, Vec<u8>>) -> Option<Vec<u8>> {
        files.get("big.png").cloned()
    }

    pub fn loadable_jar(jar: &[u8]) -> bool {
        let Ok(files) = extract_zip(jar) else {
            return false;
        };

        files.contains_key("binary.mod")
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        pid: &str,
        aid: &str,
        main_class_name: Option<String>,
        files: &BTreeMap<String, Vec<u8>>,
        options: Options,
    ) -> Result<Self> {
        let mut core = ArmCore::new(options)?;
        let system = System::new(platform, pid, aid, LgtTaskRunner { core: core.clone() });

        for (filename, data) in files {
            let filename = filename.trim_start_matches("P/");
            system.filesystem().add_virtual(filename, data.clone())
        }

        Allocator::init(&mut core)?;

        let main_class_name = main_class_name.map(|x| x.replace('.', "/"));

        let mut core_clone = core.clone();
        let mut system_clone = system.clone();
        let main_class_name_clone = main_class_name.clone();
        // The filesystem keys above drop the `P/` prefix, so the classpath name must drop it
        // the same way, or an entrypoint stored as `P/<name>.jar` names an entry that does not exist.
        let jar_filename = jar_filename.trim_start_matches("P/").to_owned();

        system.spawn(async move || Self::do_start(&mut core_clone, &mut system_clone, jar_filename, main_class_name_clone).await);

        Ok(Self { core, system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(core: &mut ArmCore, system: &mut System, jar_filename: String, _main_class_name: Option<String>) -> Result<()> {
        let jvm = LgtJvmSupport::init(core, system, Some(&jar_filename)).await?;

        let class_loader = match JavaLangClassLoader::get_system_class_loader(&jvm).await {
            Ok(class_loader) => class_loader,
            Err(error) => return Err(JvmSupport::to_wie_err(&jvm, error).await),
        };
        let stream = match JavaLangClassLoader::get_resource_as_stream(&jvm, &class_loader, "binary.mod").await {
            Ok(Some(stream)) => stream,
            Ok(None) => return Err(WieError::FatalError(format!("Missing binary.mod in {jar_filename}"))),
            Err(error) => return Err(JvmSupport::to_wie_err(&jvm, error).await),
        };

        let binary_mod = match JavaIoInputStream::read_until_end(&jvm, &stream).await {
            Ok(binary_mod) => binary_mod,
            Err(error) => return Err(JvmSupport::to_wie_err(&jvm, error).await),
        };

        if let Err(error) = load_native(core, system, &jvm, &jar_filename, &binary_mod).await {
            return Err(match error {
                WieError::JavaException(ptr_exception) => {
                    let exception = LgtJvmSupport::class_instance_from_raw(core, ptr_exception);
                    JvmSupport::to_wie_err(&jvm, JavaError::JavaException(exception)).await
                }
                error => error,
            });
        }

        Ok(())
    }
}

impl Emulator for LgtEmulator {
    fn is_preparing(&self) -> bool {
        self.core.is_preparing()
    }

    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> Result<()> {
        self.system.tick()
    }
}

// almost similar to KtfAdf.. can we merge these?
struct LgtAppInfo {
    name: String,
    aid: String,
    pid: String,
    mclass: String,
}

impl LgtAppInfo {
    pub fn parse(data: &[u8]) -> Self {
        let mut name = String::new();
        let mut aid = String::new();
        let mut pid = String::new();
        let mut mclass = String::new();

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"Name:") {
                name = EUC_KR.decode(&line[5..]).0.trim().to_string();
            } else if line.starts_with(b"AID:") {
                aid = String::from_utf8_lossy(&line[4..]).into();
            } else if line.starts_with(b"PID:") {
                pid = String::from_utf8_lossy(&line[4..]).into();
            } else if line.starts_with(b"MClass:") {
                mclass = String::from_utf8_lossy(&line[7..]).into();
            }
        }

        Self { name, aid, pid, mclass }
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use core::task::{Context, Poll, Waker};

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System, TaskRunner, YieldFuture};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{Result, write_generic};

    use crate::runtime::exception::{pending, pop, push, unwind};

    use super::{LgtAppInfo, LgtTaskRunner};

    #[test]
    fn native_roots_preserve_and_release_guest_allocations() -> Result<()> {
        use crate::runtime::LgtJvmSupport;
        use alloc::sync::Arc;
        use core::sync::atomic::{AtomicBool, Ordering};
        use jvm::runtime::JavaLangString;

        let mut core = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        Allocator::init(&mut core)?;
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let system_task = system.clone();
        let done = Arc::new(AtomicBool::new(false));
        let done_task = done.clone();
        system.spawn(async move || {
            let runner = LgtTaskRunner { core: core.clone() };
            runner
                .run(Box::pin(async move {
                    let jvm = LgtJvmSupport::init(&mut core, &system_task, None).await?;
                    for element in ["B", "J", "D"] {
                        jvm.resolve_class(&alloc::format!("[{element}")).await.unwrap();
                    }
                    jvm.collect_garbage().unwrap();
                    let empty = core.save_context();
                    for (element, length, offset) in [("B", 3, 0), ("B", 3, 5), ("B", 3, 11), ("J", 2, 9), ("D", 2, 24), ("J", 0, 8)] {
                        let (header, storage) = {
                            let _frame = jvm.push_native_frame(&[]);
                            let array = jvm.instantiate_array(element, length).await.unwrap();
                            let header = array.identity() as u32;
                            let storage = wie_util::read_generic::<u32, _>(&core, header + 8)?;
                            (header, storage)
                        };
                        let mut live = empty;
                        live.r4 = if offset == 0 { header } else { storage + offset };
                        core.restore_context(&live);
                        assert_eq!(jvm.collect_garbage().unwrap(), 0, "{element}/{length}/{offset}");
                        core.restore_context(&empty);
                        assert_eq!(jvm.collect_garbage().unwrap(), 1);
                    }

                    // The String's character array must be traced from a guest-only root.
                    let reference = {
                        let _frame = jvm.push_native_frame(&[]);
                        JavaLangString::from_rust_string(&jvm, "native-only").await.unwrap().identity() as u32
                    };
                    let mut live = empty;
                    live.r4 = reference;
                    core.restore_context(&live);
                    let outer = core.save_resumable_context()?;
                    core.restore_context(&empty);
                    let inner = core.save_resumable_context()?;
                    jvm.collect_garbage().unwrap();
                    {
                        let _frame = jvm.push_native_frame(&[]);
                        let string = LgtJvmSupport::class_instance_from_raw(&core, reference);
                        assert_eq!(JavaLangString::to_rust_string(&jvm, &string).await.unwrap(), "native-only");
                    }
                    drop(outer);
                    drop(inner);
                    assert!(jvm.collect_garbage().unwrap() >= 2);

                    for one_past in [false, true] {
                        let (reference, fields, size) = {
                            let _frame = jvm.push_native_frame(&[]);
                            let string = JavaLangString::from_rust_string(&jvm, "field-storage").await.unwrap();
                            let reference = string.identity() as u32;
                            let header: wipi_types::lgt::java::LgtJavaClassInstance = wie_util::read_generic(&core, reference)?;
                            let class = LgtJvmSupport::class_from_raw(&core, wie_util::read_generic(&core, header.ptr_dispatch_table)?);
                            (reference, header.ptr_fields, class.instance_field_word_count()? as u32 * 4)
                        };
                        live = empty;
                        live.r4 = fields + if one_past { size } else { 1 };
                        core.restore_context(&live);
                        jvm.collect_garbage().unwrap();
                        {
                            let _frame = jvm.push_native_frame(&[]);
                            let string = LgtJvmSupport::class_instance_from_raw(&core, reference);
                            assert_eq!(JavaLangString::to_rust_string(&jvm, &string).await.unwrap(), "field-storage");
                        }
                        core.restore_context(&empty);
                        assert_eq!(jvm.collect_garbage().unwrap(), 2);
                    }

                    let array = {
                        let _frame = jvm.push_native_frame(&[]);
                        jvm.instantiate_array("B", 7).await.unwrap().identity() as u32
                    };
                    live = empty;
                    live.sp -= 4;
                    write_generic(&mut core, live.sp, array)?;
                    core.restore_context(&live);
                    assert_eq!(jvm.collect_garbage().unwrap(), 0);
                    let dead = {
                        let _frame = jvm.push_native_frame(&[]);
                        jvm.instantiate_array("B", 1).await.unwrap().identity() as u32
                    };
                    let mut invalid = live;
                    invalid.sp = 0;
                    core.restore_context(&invalid);
                    assert!(jvm.collect_garbage().is_err());
                    assert!(Allocator::is_allocated(&core, dead, 12)?);
                    core.restore_context(&live);
                    assert_eq!(jvm.collect_garbage().unwrap(), 1);
                    core.restore_context(&empty);
                    assert_eq!(jvm.collect_garbage().unwrap(), 1);

                    for source in ["register", "stack", "catch register", "catch stack", "pending exception", "saved caller"] {
                        let reference = {
                            let _frame = jvm.push_native_frame(&[]);
                            jvm.instantiate_array("B", 3).await.unwrap().identity() as u32
                        };
                        let mut owner_core = core.clone();
                        let owner_runner = LgtTaskRunner { core: core.clone() };
                        let mut owner = owner_runner.run(Box::pin(async move {
                            let empty = owner_core.save_context();
                            let mut state = empty;
                            state.lr = 0x4001;
                            match source {
                                "register" | "catch register" | "saved caller" => state.r5 = reference,
                                "stack" | "catch stack" => {
                                    state.sp -= 4;
                                    write_generic(&mut owner_core, state.sp, reference)?;
                                }
                                _ => (),
                            }
                            owner_core.restore_context(&state);
                            let _saved = if source == "saved caller" {
                                let saved = owner_core.save_resumable_context()?;
                                owner_core.restore_context(&empty);
                                Some(saved)
                            } else {
                                if source.starts_with("catch") || source == "pending exception" {
                                    push(&mut owner_core)?;
                                    if source == "pending exception" {
                                        assert_eq!(unwind(&mut owner_core, reference)?, Some(0x4001));
                                    }
                                    owner_core.restore_context(&empty);
                                }
                                None
                            };
                            YieldFuture::new().await;
                            Ok(())
                        }));
                        let mut context = Context::from_waker(Waker::noop());
                        assert!(owner.as_mut().poll(&mut context).is_pending());
                        core.restore_context(&empty);
                        let runtime_contexts = core.snapshot_roots()?.runtime_contexts;
                        let support = *runtime_contexts.last().unwrap();
                        assert_eq!(jvm.collect_garbage().unwrap(), 0, "{source}");
                        drop(owner);
                        assert!(!Allocator::is_allocated(&core, support, 8)?);
                        core.restore_context(&empty);
                        assert_eq!(jvm.collect_garbage().unwrap(), 1, "{source}");
                    }
                    done_task.store(true, Ordering::Relaxed);
                    Ok(())
                }))
                .await
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn exception_frames_are_restored_per_thread() -> Result<()> {
        let mut core = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        Allocator::init(&mut core)?;
        let runner = LgtTaskRunner { core: core.clone() };

        let mut threads = [0x4001, 0x5001].map(|resume| {
            let mut thread_core = core.clone();
            runner.run(Box::pin(async move {
                let mut context = thread_core.save_context();
                context.r4 = resume;
                context.lr = resume;
                thread_core.restore_context(&context);
                push(&mut thread_core)?;
                YieldFuture::new().await;

                let stack = context.sp;
                context.r4 = 0;
                context.lr = 0;
                thread_core.restore_context(&context);
                assert_eq!(unwind(&mut thread_core, resume)?, Some(resume));
                let restored = thread_core.save_context();
                assert_eq!(restored.r4, resume);
                assert_eq!(restored.sp, stack);
                assert_eq!(restored.pc, resume & !1);
                YieldFuture::new().await;
                assert_eq!(pending(&thread_core)?, resume);

                pop(&mut thread_core)?;
                assert_eq!(pending(&thread_core)?, 0);
                assert_eq!(unwind(&mut thread_core, 0x5678)?, None);
                Ok(())
            }))
        });
        let mut context = Context::from_waker(Waker::noop());
        for thread in &mut threads {
            assert!(thread.as_mut().poll(&mut context).is_pending());
        }
        for thread in &mut threads {
            assert!(thread.as_mut().poll(&mut context).is_pending());
        }
        for thread in &mut threads {
            assert!(matches!(thread.as_mut().poll(&mut context), Poll::Ready(Ok(()))));
        }

        let mut damaged_core = core.clone();
        let mut damaged_task = runner.run(Box::pin(async move {
            let support: u32 = wie_util::read_generic(&damaged_core, crate::runtime::SUPPORT_CONTEXT_BASE)?;
            write_generic(&mut damaged_core, support, 0xffff_ffffu32)?;
            Ok(())
        }));
        assert!(
            matches!(damaged_task.as_mut().poll(&mut context), Poll::Ready(Err(_))),
            "Normal context destruction errors must reach the task runner"
        );

        Ok(())
    }

    #[test]
    fn parse_app_info_name() {
        let app_info =
            LgtAppInfo::parse(b"PID:pid\nAID:aid\nName:\xbf\xb5\xbf\xf5\xbc\xad\xb1\xe24-\xc8\xaf\xbf\xb5\xc0\xc7\xb0\xa1\xb8\xe9\nMClass:Clet\n");

        assert_eq!(app_info.name, "영웅서기4-환영의가면");
        assert_eq!(app_info.aid, "aid");
        assert_eq!(app_info.pid, "pid");
        assert_eq!(app_info.mclass, "Clet");
    }
}
