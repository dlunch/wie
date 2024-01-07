use alloc::boxed::Box;

use java_runtime_base::MethodBody;

use jvm::JvmResult;
use wie_backend::{AsyncCallable, SystemHandle};
use wie_core_arm::ArmCore;
use wie_impl_java::WieContextBase;

use crate::runtime::java::jvm::KtfJvm;

#[derive(Clone)]
pub struct KtfWieContext {
    core: ArmCore,
    system: SystemHandle,
}

impl KtfWieContext {
    pub fn new(core: &ArmCore, system: &SystemHandle) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl WieContextBase for KtfWieContext {
    fn system(&mut self) -> &mut SystemHandle {
        &mut self.system
    }

    fn spawn(&mut self, callback: Box<dyn MethodBody<anyhow::Error, dyn WieContextBase>>) -> JvmResult<()> {
        struct SpawnProxy {
            core: ArmCore,
            system: SystemHandle,
            callback: Box<dyn MethodBody<anyhow::Error, dyn WieContextBase>>,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, anyhow::Error> for SpawnProxy {
            async fn call(mut self) -> Result<u32, anyhow::Error> {
                let mut context = KtfWieContext::new(&self.core, &self.system);
                let mut jvm = KtfJvm::new(&self.core, &self.system).jvm();

                let _ = self.callback.call(&mut jvm, &mut context, Box::new([])).await?;

                Ok(0) // TODO resturn value
            }
        }

        let system = self.system.clone();

        self.core.spawn(SpawnProxy {
            core: self.core.clone(),
            system,
            callback,
        });

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;

    use wie_backend::{AudioSink, Platform, System, SystemHandle};
    use wie_base::util::write_generic;
    use wie_core_arm::{Allocator, ArmCore, PEB_BASE};

    use crate::runtime::{java::jvm::KtfJvm, KtfPeb};

    struct TestPlatform;

    impl Platform for TestPlatform {
        fn screen(&mut self) -> &mut dyn wie_backend::Screen {
            todo!()
        }

        fn now(&self) -> wie_backend::Instant {
            todo!()
        }

        fn database_repository(&self) -> &dyn wie_backend::DatabaseRepository {
            todo!()
        }

        fn audio_sink(&self) -> Box<dyn AudioSink> {
            Box::new(TestAudioSink)
        }
    }

    struct TestAudioSink;

    impl AudioSink for TestAudioSink {
        fn play_wave(&self, _channel: u8, _sampling_rate: u32, _wave_data: &[i16]) {
            todo!()
        }
    }

    fn test_core(system_handle: &SystemHandle) -> anyhow::Result<ArmCore> {
        let mut core = ArmCore::new(system_handle.clone())?;
        Allocator::init(&mut core)?;

        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x100)?;
        context.sp = stack + 0x100;
        core.restore_context(&context);

        let ptr_vtables_base = Allocator::alloc(&mut core, 0x100)?;
        let ptr_java_context_data = KtfJvm::init(&mut core, ptr_vtables_base, 0)?;

        core.map(PEB_BASE, 0x1000)?;
        write_generic(
            &mut core,
            PEB_BASE,
            KtfPeb {
                ptr_java_context_data,
                ptr_current_java_exception_handler: 0,
            },
        )?;

        Ok(core)
    }

    #[futures_test::test]
    async fn test_context() -> anyhow::Result<()> {
        let system_handle = System::new(Box::new(TestPlatform)).handle();
        let core = test_core(&system_handle)?;

        let mut jvm = KtfJvm::new(&core, &system_handle).jvm();

        let string1 = String::from_rust_string(&mut jvm, "test1").await?;
        let string2 = String::from_rust_string(&mut jvm, "test2").await?;

        let string3 = jvm
            .invoke_virtual(
                &string1,
                "java/lang/String",
                "concat",
                "(Ljava/lang/String;)Ljava/lang/String;",
                [string2.into()],
            )
            .await?;

        assert_eq!(String::to_rust_string(&mut jvm, &string3)?, "test1test2");

        Ok(())
    }
}
