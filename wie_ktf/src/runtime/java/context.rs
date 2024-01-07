mod array_class;
mod array_class_instance;
mod class;
mod class_instance;
mod class_loader;
mod context_data;
mod field;
mod method;
mod name;
mod platform;
mod value;
mod vtable_builder;

use alloc::{boxed::Box, format};

use bytemuck::{Pod, Zeroable};

use java_runtime_base::MethodBody;
use jvm::{ArrayClass, Class, ClassInstance, Jvm, JvmDetail, JvmResult, ThreadContext, ThreadId};

use wie_backend::{AsyncCallable, SystemHandle};
use wie_core_arm::ArmCore;
use wie_impl_java::JavaContext;

pub use self::name::JavaFullName;
use self::{
    array_class::JavaArrayClass, array_class_instance::JavaArrayClassInstance, class::JavaClass, class_instance::JavaClassInstance,
    class_loader::ClassLoader, context_data::JavaContextData,
};

pub type KtfJvmWord = u32;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaExceptionHandler {
    ptr_method: u32,
    ptr_this: u32,
    ptr_old_handler: u32,
    current_state: u32, // state is returned on restore context
    unk3: u32,
    ptr_functions: u32, // function table to restore context and unk
    context: [u32; 11], // r4-lr
}

struct KtfJvmDetail {
    core: ArmCore,
    system: SystemHandle,
}

impl KtfJvmDetail {
    pub fn new(core: &ArmCore, system: SystemHandle) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl JvmDetail for KtfJvmDetail {
    async fn load_class(&mut self, class_name: &str) -> JvmResult<Option<Box<dyn Class>>> {
        let class = ClassLoader::get_or_load_class(&mut self.core, &mut self.system, class_name).await?;

        Ok(class.map(|x| Box::new(x) as _))
    }

    async fn load_array_class(&mut self, element_type_name: &str) -> JvmResult<Option<Box<dyn ArrayClass>>> {
        let class_name = format!("[{}", element_type_name);
        if let Some(x) = JavaContextData::find_class(&self.core, &class_name)? {
            let class = JavaArrayClass::from_raw(x.ptr_raw, &self.core);
            return Ok(Some(Box::new(class)));
        } else {
            let class = JavaArrayClass::new(&mut self.core, &mut self.system, &class_name).await?;

            Ok(Some(Box::new(class)))
        }
    }

    fn get_class(&self, class_name: &str) -> JvmResult<Option<Box<dyn Class>>> {
        let class = JavaContextData::find_class(&self.core, class_name)?;

        Ok(class.map(|x| Box::new(x) as _))
    }

    fn thread_context(&mut self, _thread_id: ThreadId) -> &mut dyn ThreadContext {
        todo!()
    }
}

#[derive(Clone)]
pub struct KtfJavaContext {
    core: ArmCore,
    system: SystemHandle,
}

impl KtfJavaContext {
    pub fn init(core: &mut ArmCore, ptr_vtables_base: u32, fn_get_class: u32) -> JvmResult<u32> {
        JavaContextData::init(core, ptr_vtables_base, fn_get_class)
    }

    pub fn new(core: &mut ArmCore, system: &mut SystemHandle) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
        }
    }

    pub fn jvm(&self) -> Jvm {
        Jvm::new(KtfJvmDetail::new(&self.core, self.system.clone()))
    }

    pub async fn load_class(&mut self, name: &str) -> JvmResult<Option<JavaClass>> {
        ClassLoader::get_or_load_class(&mut self.core, &mut self.system, name).await
    }

    pub async fn load_array_class(&mut self, name: &str) -> JvmResult<Option<JavaArrayClass>> {
        ClassLoader::load_array_class(&mut self.core, &mut self.system, name).await
    }

    #[async_recursion::async_recursion(?Send)]
    pub async fn register_class(core: &mut ArmCore, system: &mut SystemHandle, class: &JavaClass) -> JvmResult<()> {
        if JavaContextData::has_class(core, class)? {
            return Ok(());
        }

        JavaContextData::register_class(core, class)?;

        let clinit = class.method("<clinit>", "()V")?;

        if let Some(x) = clinit {
            tracing::trace!("Call <clinit>");

            x.run(Box::new([])).await?;
        }

        if let Some(x) = class.super_class() {
            let super_class = ClassLoader::get_or_load_class(core, system, &x.name()).await?.unwrap(); // TODO we can use superclass as-is
            Self::register_class(core, system, &super_class).await?;
        }

        Ok(())
    }

    pub fn class_from_raw(&self, ptr_class: u32) -> JavaClass {
        JavaClass::from_raw(ptr_class, &self.core)
    }

    #[allow(clippy::borrowed_box)]
    pub fn class_raw(&self, instance: &Box<dyn ClassInstance>) -> u32 {
        if let Some(x) = instance.as_any().downcast_ref::<JavaClassInstance>() {
            x.ptr_raw
        } else {
            let instance = instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap();

            instance.class_instance.ptr_raw
        }
    }
}

#[async_trait::async_trait(?Send)]
impl JavaContext for KtfJavaContext {
    fn system(&mut self) -> &mut SystemHandle {
        &mut self.system
    }

    fn spawn(&mut self, callback: Box<dyn MethodBody<anyhow::Error, dyn JavaContext>>) -> JvmResult<()> {
        struct SpawnProxy {
            core: ArmCore,
            system: SystemHandle,
            callback: Box<dyn MethodBody<anyhow::Error, dyn JavaContext>>,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, anyhow::Error> for SpawnProxy {
            async fn call(mut self) -> Result<u32, anyhow::Error> {
                let mut context = KtfJavaContext::new(&mut self.core, &mut self.system);
                let mut jvm = context.jvm();

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

    use crate::runtime::KtfPeb;

    use super::KtfJavaContext;

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
        let ptr_java_context_data = KtfJavaContext::init(&mut core, ptr_vtables_base, 0)?;

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
        let mut system_handle = System::new(Box::new(TestPlatform)).handle();
        let mut core = test_core(&system_handle)?;

        let context = KtfJavaContext::new(&mut core, &mut system_handle);
        let mut jvm = context.jvm();

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
