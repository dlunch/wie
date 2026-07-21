use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use spin::Mutex;

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

#[derive(Clone)]
pub struct KtfWIPICContext {
    core: ArmCore,
    system: System,
    jvm: Jvm, // We need jvm to access resource in jvm. TODO is there better way to do this?
    state: Arc<Mutex<WIPICRuntimeState>>,
}

#[derive(Default)]
pub(super) struct WIPICRuntimeState {
    resource_cache: BTreeMap<String, Vec<u8>>,
}

impl KtfWIPICContext {
    pub fn new(core: ArmCore, system: System, jvm: Jvm, state: Arc<Mutex<WIPICRuntimeState>>) -> Self {
        Self { core, system, jvm, state }
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
        self.core.run_function(address, args).await
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
        if let Some(size) = self.state.lock().resource_cache.get(name).map(Vec::len) {
            tracing::debug!("WIPI-C resource cache hit for size: name={name:?}, size={size}");
            return Ok(Some(size));
        }

        let class_loader = self
            .jvm
            .current_class_loader()
            .await
            .map_err(|err| WieError::FatalError(alloc::format!("Failed to get class loader for resource {name:?}: {err:?}")))?;
        let stream = match JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await {
            Ok(stream) => stream,
            Err(err) => {
                tracing::error!("Java exception while opening resource for size query: name={name:?}, error={err:?}");
                return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
            }
        };

        if stream.is_none() {
            return Ok(None);
        }

        let stream = stream.unwrap();
        let available: i32 = match self.jvm.invoke_virtual(&stream, "available", "()I", ()).await {
            Ok(available) => available,
            Err(err) => {
                tracing::error!("Java exception while querying resource size: name={name:?}, error={err:?}");
                return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
            }
        };
        drop(stream);
        let garbage_count = match self.jvm.collect_garbage() {
            Ok(count) => count,
            Err(err) => return Err(JvmSupport::to_wie_err(&self.jvm, err).await),
        };
        tracing::debug!("WIPI-C resource size GC: name={name:?}, collected={garbage_count}");

        Ok(Some(available as _))
    }

    async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
        if let Some(data) = self.state.lock().resource_cache.get(name).cloned() {
            tracing::debug!("WIPI-C resource cache hit for read: name={name:?}, size={}", data.len());
            return Ok(data);
        }

        let class_loader = self
            .jvm
            .current_class_loader()
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
        self.state.lock().resource_cache.insert(name.to_string(), data.clone());
        let garbage_count = match self.jvm.collect_garbage() {
            Ok(count) => count,
            Err(err) => return Err(JvmSupport::to_wie_err(&self.jvm, err).await),
        };
        tracing::info!("WIPI-C resource cached: name={name:?}, size={}, collected={garbage_count}", data.len());
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
