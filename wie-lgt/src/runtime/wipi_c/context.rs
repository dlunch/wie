use alloc::{boxed::Box, format, string::ToString, vec, vec::Vec};

use jvm::{
    Jvm,
    runtime::{JavaIoInputStream, JavaLangClassLoader},
};
use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use wie_backend::{AsyncCallable, Event, Instant, System};
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic, write_generic};
use wie_wipi_c::{WIPICContext, WIPICMethodBody};

use crate::runtime::with_arm_context;

// mostly same as ktf's one, can we merge those?
#[derive(Clone)]
pub struct LgtWIPICContext {
    core: ArmCore,
    system: System,
    jvm: Jvm,
}

impl LgtWIPICContext {
    pub fn new(core: ArmCore, system: System, jvm: Jvm) -> Self {
        Self { core, system, jvm }
    }
}

#[async_trait::async_trait]
impl WIPICContext for LgtWIPICContext {
    fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord> {
        Allocator::alloc(&mut self.core, size)
    }

    fn alloc(&mut self, size: WIPICWord) -> Result<WIPICIndirectPtr> {
        let address = Allocator::alloc(&mut self.core, size + size_of::<WIPICWord>() as WIPICWord)?;
        write_generic(&mut self.core, address, size)?;

        Ok(WIPICIndirectPtr(address + size_of::<WIPICWord>() as WIPICWord))
    }

    fn free(&mut self, memory: WIPICIndirectPtr) -> Result<()> {
        let base_address = memory.0 - size_of::<WIPICWord>() as WIPICWord;

        let size: WIPICWord = read_generic(&self.core, base_address)?;
        Allocator::free(&mut self.core, base_address, size + size_of::<WIPICWord>() as WIPICWord)
    }

    fn free_raw(&mut self, address: WIPICWord, size: WIPICWord) -> Result<()> {
        Allocator::free(&mut self.core, address, size)?;

        Ok(())
    }

    fn data_ptr(&self, memory: WIPICIndirectPtr) -> Result<WIPICWord> {
        Ok(memory.0)
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
            context: LgtWIPICContext,
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
        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm).await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await.unwrap();

        if let Some(stream) = stream {
            let available: i32 = self
                .jvm
                .invoke_virtual(&stream, "java/io/InputStream", "available", "()I", ())
                .await
                .unwrap();
            return Ok(Some(available as _));
        }
        self.jvm.collect_garbage().map_err(|error| WieError::FatalError(error.to_string()))?;

        Ok(self.system.filesystem().size(name).await)
    }

    async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm).await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await.unwrap();

        if let Some(stream) = stream {
            return Ok(JavaIoInputStream::read_until_end(&self.jvm, &stream).await.unwrap());
        }

        let Some(size) = self.system.filesystem().size(name).await else {
            return Err(WieError::FatalError(format!("Missing resource: {name}")));
        };
        let mut data = vec![0; size];
        let read = self.system.filesystem().read(name, 0, size, &mut data).await.unwrap_or(0);
        data.truncate(read);

        self.jvm.collect_garbage().map_err(|error| WieError::FatalError(error.to_string()))?;

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

impl ByteRead for LgtWIPICContext {
    fn read_bytes(&self, address: WIPICWord, result: &mut [u8]) -> wie_util::Result<usize> {
        self.core.read_bytes(address, result)
    }
}

impl ByteWrite for LgtWIPICContext {
    fn write_bytes(&mut self, address: WIPICWord, data: &[u8]) -> wie_util::Result<()> {
        self.core.write_bytes(address, data)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, format, string::ToString, sync::Arc};
    use core::sync::atomic::{AtomicBool, Ordering};

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_jvm_support::JvmSupport;
    use wie_util::{Result, write_generic};
    use wie_wipi_c::api::kernel;

    use super::LgtWIPICContext;
    use crate::runtime::LgtJvmSupport;

    #[test]
    fn callback_fault_diagnostics_survive_java_exception_restoration() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_task = done.clone();
        let system_task = system.clone();

        system.spawn(async move || {
            let mut core = ArmCore::new(wie_backend::Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })?;
            Allocator::init(&mut core)?;
            let _thread = core.run_in_thread(core::future::pending)?;
            let _entered = core.enter_thread_context(1);
            let caller = core.save_context();
            let jvm = LgtJvmSupport::init(&mut core, &system_task, None).await?;
            let queue = jvm.new_class("net/wie/EventQueue", "()V", ()).await.unwrap();
            let timer = Allocator::alloc(&mut core, size_of::<kernel::WIPICTimer>() as u32)?;
            let mut context = LgtWIPICContext::new(core.clone(), system_task.clone(), jvm.clone());

            for instruction in [0x6800u16, 0xe800] {
                let callback = Allocator::alloc(&mut core, 6)?;
                write_generic(&mut core, callback, 0x2455u16)?; // movs r4, #0x55
                write_generic(&mut core, callback + 2, 0x2000u16)?; // movs r0, #0
                write_generic(&mut core, callback + 4, instruction)?; // ldr r0, [r0] or undefined instruction
                let expected = if instruction == 0xe800 {
                    format!("Undefined instruction at {:#x}", callback + 4)
                } else {
                    "Invalid memory access; address: 0".into()
                };
                for timer_callback in [true, false] {
                    let error = if timer_callback {
                        kernel::def_timer(&mut context, timer, callback + 1).await?;
                        kernel::set_timer(&mut context, timer, 0, 0, 0).await?;
                        let event = jvm.instantiate_array("I", 4).await.unwrap();
                        jvm.invoke_virtual::<_, ()>(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event,))
                            .await
                            .unwrap_err()
                    } else {
                        let mut card = jvm.instantiate_class("net/wie/CletWrapperCard").await.unwrap();
                        jvm.put_field(&mut card, "net/wie/CletWrapperCard", "handleCletEvent", "I", (callback + 1) as i32)
                            .await
                            .unwrap();
                        jvm.invoke_virtual::<_, ()>(&card, "net/wie/CletWrapperCard", "notifyEvent", "(III)V", (0, 0, 0))
                            .await
                            .unwrap_err()
                    };
                    let message = JvmSupport::to_wie_err(&jvm, error).await.to_string();
                    assert!(message.contains(&expected), "Expected {expected}: {message}");
                    assert!(message.contains("R4: 0x55"), "{message}");
                    assert_eq!(message.matches("Possible call stack:").count(), 1, "{message}");
                    let restored = core.save_context();
                    assert_eq!(
                        (restored.r4, restored.sp, restored.lr, restored.pc),
                        (caller.r4, caller.sp, caller.lr, caller.pc)
                    );
                }
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
