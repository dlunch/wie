use core::{mem::size_of, pin::Pin, task::Poll};

use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec, vec::Vec};

use bytemuck::Zeroable;
use futures::future::poll_fn;
use jvm::{ClassInstance, Result as JvmResult, runtime::JavaLangString};

use wie_backend::{Emulator, Event, Options, Platform, System, TaskRunner};
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, write_generic};

use crate::{
    adf::{KtfAdf, find_client_bin},
    runtime::{KtfJvmSupport, KtfJvmThreadContext},
};

pub const IMAGE_BASE: u32 = 0x100000;

struct KtfTaskRunner {
    core: ArmCore,
}

#[async_trait::async_trait]
impl TaskRunner for KtfTaskRunner {
    async fn run(&self, mut future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()> {
        let mut core = self.core.clone();
        let ptr_thread_context = Allocator::alloc(&mut core, size_of::<KtfJvmThreadContext>() as u32)?;
        write_generic(&mut core, ptr_thread_context, KtfJvmThreadContext::zeroed())?;

        let mut poll_core = self.core.clone();
        let result = self
            .core
            .run_in_thread(move || {
                poll_fn(move |context| {
                    // KTF's first native init argument points at this cell and dereferences it for each stack check and try block.
                    if let Err(error) = KtfJvmSupport::set_current_thread_context(&mut poll_core, ptr_thread_context) {
                        return Poll::Ready(Err(error));
                    }

                    future.as_mut().poll(context)
                })
            })?
            .await;

        Allocator::free(&mut core, ptr_thread_context, size_of::<KtfJvmThreadContext>() as u32)?;

        result
    }
}

pub struct KtfEmulator {
    core: ArmCore,
    system: System,
}

impl KtfEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>, options: Options) -> Result<Self> {
        let adf = files
            .get("__adf__")
            .ok_or_else(|| WieError::FatalError("Missing __adf__ in KTF archive".into()))?;
        let adf = KtfAdf::parse(adf);

        tracing::info!("Loading app {}, pid {}, mclass {}", adf.aid, adf.pid, adf.mclass);
        if let Some((width, height)) = adf.display_size
            && let Err(error) = platform.screen().resize(width, height)
        {
            tracing::warn!("Ignoring unsupported display size {width}x{height}: {error}");
        }

        let jar_filename = format!("{}.jar", adf.aid);

        Self::load(platform, &jar_filename, &adf.pid, &adf.aid, Some(adf.mclass), &files, options)
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
        files.contains_key("__adf__")
    }

    pub fn loadable_jar(jar: &[u8]) -> bool {
        find_client_bin(jar).is_ok()
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        pid: &str,
        aid: &str,
        main_class_name: Option<String>,
        files: &BTreeMap<String, Vec<u8>>,
        mut options: Options,
    ) -> Result<Self> {
        let mut core = ArmCore::new(options.enable_gdbserver, options.profile.take())?;
        let system = System::new(platform, pid, aid, KtfTaskRunner { core: core.clone() });

        for (path, data) in files {
            let path = path.trim_start_matches("P/");
            system.filesystem().add_virtual(path, data.clone());
        }

        Allocator::init(&mut core)?;

        let mut core_clone = core.clone();
        let mut system_clone = system.clone();
        let jar_filename_clone = jar_filename.to_owned();

        system.spawn(async move || Self::start(&mut core_clone, &mut system_clone, jar_filename_clone, main_class_name).await);

        Ok(Self { core, system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn start(core: &mut ArmCore, system: &mut System, jar_filename: String, main_class_name: Option<String>) -> Result<()> {
        let (jvm, class_loader) = KtfJvmSupport::init(core, system, Some(&jar_filename)).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else {
            return Err(WieError::FatalError("Main class not found".into()));
        };

        let main_class_name = main_class_name.replace('.', "/");

        let main_class_name_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await.unwrap();
        let _main_class: Box<dyn ClassInstance> = jvm
            .invoke_virtual(
                &class_loader,
                "net/wie/KtfClassLoader",
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                (main_class_name_java.clone(),),
            )
            .await
            .unwrap();

        let mut args_array = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
        jvm.store_array(&mut args_array, 0, vec![main_class_name_java]).await.unwrap();
        let result: JvmResult<()> = jvm
            .invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", (args_array,))
            .await;

        if let Err(x) = result {
            return Err(JvmSupport::to_wie_err(&jvm, x).await);
        }

        Ok(())
    }
}

impl Emulator for KtfEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> Result<()> {
        self.system.tick().map_err(|x| {
            let reg_stack = self.core.dump_reg_stack(IMAGE_BASE);
            match x {
                WieError::FatalError(msg) => WieError::FatalError(format!("{msg}\n{reg_stack}")),
                _ => WieError::FatalError(format!("{x}\n{reg_stack}")),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

    use test_utils::TestPlatform;
    use wie_backend::{System, YieldFuture};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{Result, WieError};

    use super::{KtfJvmSupport, KtfTaskRunner};

    #[test]
    fn switches_jvm_thread_context_between_tasks() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let mut system = System::new(Box::new(TestPlatform::new()), "", "", KtfTaskRunner { core: core.clone() });
        let contexts = Arc::new([AtomicU32::new(0), AtomicU32::new(0)]);
        let completed = Arc::new(AtomicUsize::new(0));

        for index in 0..2 {
            let core = core.clone();
            let contexts = contexts.clone();
            let completed = completed.clone();
            system.spawn(async move || {
                let before = KtfJvmSupport::current_thread_context(&core)?;
                YieldFuture::new().await;
                let after = KtfJvmSupport::current_thread_context(&core)?;
                if before != after {
                    return Err(WieError::FatalError("KTF JVM thread context changed while the task was suspended".into()));
                }

                contexts[index].store(before, Ordering::Relaxed);
                completed.fetch_add(1, Ordering::Relaxed);

                Ok(())
            });
        }

        while completed.load(Ordering::Relaxed) != 2 {
            system.tick()?;
        }

        let first = contexts[0].load(Ordering::Relaxed);
        let second = contexts[1].load(Ordering::Relaxed);
        assert_ne!(first, 0);
        assert_ne!(second, 0);
        assert_ne!(first, second);

        Ok(())
    }
}
