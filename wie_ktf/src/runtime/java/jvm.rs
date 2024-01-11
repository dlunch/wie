mod array_class;
mod array_class_instance;
mod class;
mod class_instance;
mod class_loader;
mod context_data;
mod detail;
mod field;
mod method;
mod name;
mod value;
mod vtable_builder;

use alloc::boxed::Box;
use bytemuck::{Pod, Zeroable};

use wie_backend::SystemHandle;
use wie_core_arm::ArmCore;

use jvm::{Class, ClassInstance, Jvm, JvmResult};

use self::{
    array_class::JavaArrayClass, array_class_instance::JavaArrayClassInstance, class::JavaClass, class_instance::JavaClassInstance,
    class_loader::ClassLoader, context_data::JavaContextData, name::JavaFullName,
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

pub struct KtfJvm {
    core: ArmCore,
    system: SystemHandle,
}

impl KtfJvm {
    pub fn init(core: &mut ArmCore, ptr_vtables_base: u32, fn_get_class: u32) -> JvmResult<u32> {
        context_data::JavaContextData::init(core, ptr_vtables_base, fn_get_class)
    }

    pub fn new(core: &ArmCore, system: &SystemHandle) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
        }
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

        if let Some(x) = class.super_class_name() {
            let super_class = ClassLoader::get_or_load_class(core, system, &x).await?.unwrap();
            Self::register_class(core, system, &super_class).await?;
        }

        Ok(())
    }

    pub fn class_from_raw(&self, ptr_class: u32) -> JavaClass {
        JavaClass::from_raw(ptr_class, &self.core)
    }

    pub fn read_name(&self, ptr_name: u32) -> JvmResult<JavaFullName> {
        JavaFullName::from_ptr(&self.core, ptr_name)
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

    pub fn jvm(&self) -> Jvm {
        Jvm::new(detail::KtfJvmDetail::new(&self.core, &self.system))
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;

    use wie_backend::{System, SystemHandle};
    use wie_common::util::write_generic;
    use wie_core_arm::{Allocator, ArmCore, PEB_BASE};

    use crate::runtime::{java::jvm::KtfJvm, KtfPeb};

    use test_utils::TestPlatform;

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
