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

        if let Some(x) = class.super_class() {
            let super_class = ClassLoader::get_or_load_class(core, system, &x.name()).await?.unwrap(); // TODO we can use superclass as-is
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
