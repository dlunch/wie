use alloc::{boxed::Box, vec::Vec};

use jvm::{
    Jvm,
    runtime::{JavaIoInputStream, JavaLangClassLoader},
};
use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use wie_backend::{AsyncCallable, Event, Instant, System};
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic, write_generic};
use wie_wipi_c::{WIPICContext, WIPICMethodBody};

use crate::runtime::with_arm_context;

#[derive(Clone)]
pub struct KtfWIPICContext {
    core: ArmCore,
    system: System,
    jvm: Jvm, // We need jvm to access resource in jvm. TODO is there better way to do this?
}

impl KtfWIPICContext {
    pub fn new(core: ArmCore, system: System, jvm: Jvm) -> Self {
        Self { core, system, jvm }
    }
}

#[async_trait::async_trait]
impl WIPICContext for KtfWIPICContext {
    fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord> {
        Allocator::alloc(&mut self.core, size)
    }

    fn alloc(&mut self, size: WIPICWord) -> Result<WIPICIndirectPtr> {
        let ptr = Allocator::alloc(&mut self.core, size + 12)?; // all allocation has indirect pointer
        write_generic(&mut self.core, ptr, ptr + 4)?;
        write_generic(&mut self.core, ptr + 4, size)?;

        Ok(WIPICIndirectPtr(ptr))
    }

    fn free(&mut self, memory: WIPICIndirectPtr) -> Result<()> {
        let size: u32 = read_generic(&self.core, memory.0 + 4)?;
        Allocator::free(&mut self.core, memory.0, size + 12)?;

        Ok(())
    }

    fn free_raw(&mut self, address: WIPICWord, size: WIPICWord) -> Result<()> {
        Allocator::free(&mut self.core, address, size)?;

        Ok(())
    }

    fn data_ptr(&self, memory: WIPICIndirectPtr) -> Result<WIPICWord> {
        let base: WIPICWord = read_generic(&self.core, memory.0)?;

        Ok(base + 8) // all data has offset of 8 bytes
    }

    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> Result<WIPICWord> {
        self.core
            .run_function(address, args)
            .await
            .map_err(|error| with_arm_context(&self.core, error))
    }

    fn spawn(&mut self, callback: WIPICMethodBody) -> Result<()> {
        struct SpawnProxy {
            context: KtfWIPICContext,
            callback: WIPICMethodBody,
        }

        impl AsyncCallable<Result<()>> for SpawnProxy {
            async fn call(mut self) -> Result<()> {
                self.context.jvm.attach_thread(None).await.unwrap();
                self.callback.call(&mut self.context, Box::new([])).await?;
                self.context.jvm.detach_thread().unwrap();

                Ok(())
            }
        }

        self.system.spawn(SpawnProxy {
            context: self.clone(),
            callback,
        });

        Ok(())
    }

    async fn get_resource_size(&self, name: &str) -> Result<Option<usize>> {
        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm)
            .await
            .map_err(|err| WieError::FatalError(alloc::format!("Failed to get class loader for resource {name:?}: {err:?}")))?;
        let stream = match JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await {
            Ok(stream) => stream,
            Err(err) => {
                tracing::error!("Java exception while opening resource for size query: name={name:?}, error={err:?}");
                return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
            }
        };

        let result = match stream {
            Some(stream) => {
                let available: i32 = match self.jvm.invoke_virtual(&stream, "java/io/InputStream", "available", "()I", ()).await {
                    Ok(available) => available,
                    Err(err) => return Err(JvmSupport::to_wie_err(&self.jvm, err).await),
                };
                drop(stream);
                Some(available as usize)
            }
            None => None,
        };
        self.jvm
            .collect_garbage()
            .map_err(|error| WieError::FatalError(alloc::format!("Garbage collection failed: {error}")))?;

        Ok(result)
    }

    async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm)
            .await
            .map_err(|err| WieError::FatalError(alloc::format!("Failed to get class loader for resource {name:?}: {err:?}")))?;
        let stream = match JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await {
            Ok(Some(stream)) => stream,
            Ok(None) => return Err(WieError::FatalError(alloc::format!("Resource disappeared before read: {name:?}"))),
            Err(err) => {
                tracing::error!("Java exception while opening resource for read: name={name:?}, error={err:?}");
                return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
            }
        };

        let data = match JavaIoInputStream::read_until_end(&self.jvm, &stream).await {
            Ok(data) => data,
            Err(err) => {
                tracing::error!("Java exception while reading resource: name={name:?}, error={err:?}");
                return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
            }
        };
        drop(stream);
        self.jvm
            .collect_garbage()
            .map_err(|error| WieError::FatalError(alloc::format!("Garbage collection failed: {error}")))?;
        Ok(data)
    }

    fn set_timer(&mut self, due: Instant, callback: WIPICMethodBody) {
        let context = self.clone();

        self.system().event_queue().push(Event::timer(due, move || {
            let mut context = context.clone();

            async move {
                callback.call(&mut context, Box::new([])).await?;
                Ok(())
            }
        }))
    }
}

impl ByteRead for KtfWIPICContext {
    fn read_bytes(&self, address: WIPICWord, result: &mut [u8]) -> wie_util::Result<usize> {
        self.core.read_bytes(address, result)
    }
}

impl ByteWrite for KtfWIPICContext {
    fn write_bytes(&mut self, address: WIPICWord, data: &[u8]) -> wie_util::Result<()> {
        self.core.write_bytes(address, data)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, format, string::ToString, sync::Arc};
    use core::sync::atomic::{AtomicBool, Ordering};

    use bytemuck::Zeroable;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_jvm_support::JvmSupport;
    use wie_util::{Result, write_generic};
    use wie_wipi_c::api::kernel;

    use super::KtfWIPICContext;
    use crate::runtime::{KtfJvmSupport, KtfJvmThreadContext};

    #[test]
    fn timer_fault_diagnostics_survive_java_exception_restoration() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_task = done.clone();
        let mut system_task = system.clone();

        system.spawn(async move || {
            let mut core = ArmCore::new(wie_backend::Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })?;
            Allocator::init(&mut core)?;
            let mut caller = core.save_context();
            caller.sp = Allocator::alloc(&mut core, 0x100)? + 0x100;
            caller.cpsr = 0x10;
            core.restore_context(&caller);
            let thread = Allocator::alloc(&mut core, size_of::<KtfJvmThreadContext>() as u32)?;
            write_generic(&mut core, thread, KtfJvmThreadContext::zeroed())?;
            KtfJvmSupport::set_current_thread_context(&mut core, thread)?;
            let (jvm, _) = KtfJvmSupport::init(&mut core, &mut system_task, None).await?;
            let queue = jvm.new_class("net/wie/EventQueue", "()V", ()).await.unwrap();
            let timer = Allocator::alloc(&mut core, size_of::<kernel::WIPICTimer>() as u32)?;
            let mut context = KtfWIPICContext::new(core.clone(), system_task.clone(), jvm.clone());

            for instruction in [0x6800u16, 0xe800] {
                let callback = Allocator::alloc(&mut core, 6)?;
                write_generic(&mut core, callback, 0x2455u16)?; // movs r4, #0x55
                write_generic(&mut core, callback + 2, 0x2000u16)?; // movs r0, #0
                write_generic(&mut core, callback + 4, instruction)?; // ldr r0, [r0] or undefined instruction
                kernel::def_timer(&mut context, timer, callback + 1).await?;
                kernel::set_timer(&mut context, timer, 0, 0, 0).await?;
                let event = jvm.instantiate_array("I", 4).await.unwrap();
                let error = jvm
                    .invoke_virtual::<_, ()>(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event,))
                    .await
                    .unwrap_err();
                let message = JvmSupport::to_wie_err(&jvm, error).await.to_string();
                let expected = if instruction == 0xe800 {
                    format!("Undefined instruction at {:#x}", callback + 4)
                } else {
                    "Invalid memory access; address: 0".into()
                };
                assert!(message.contains(&expected), "Expected {expected}: {message}");
                assert!(message.contains("R4: 0x55"), "{message}");
                assert_eq!(message.matches("Possible call stack:").count(), 1, "{message}");
                let restored = core.save_context();
                assert_eq!(
                    (restored.r4, restored.sp, restored.lr, restored.pc),
                    (caller.r4, caller.sp, caller.lr, caller.pc)
                );
            }
            done_task.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }
}
