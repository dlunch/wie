mod array_class;
mod array_class_instance;
mod class;
mod class_instance;
mod class_loader;
mod context_data;
mod field;
mod method;
mod name;
mod value;
mod vtable_builder;

use alloc::{boxed::Box, format, rc::Rc};
use core::cell::RefCell;

use bytemuck::{Pod, Zeroable};

use jvm::{ArrayClass, Class, ClassInstanceRef, ClassRef, Jvm, JvmDetail, JvmResult, ThreadContext, ThreadId};

use wie_backend::{
    task::{self, SleepFuture},
    AsyncCallable, System,
};
use wie_core_arm::ArmCore;
use wie_impl_java::{JavaContext, JavaError, JavaMethodBody, JavaResult};

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
}

impl KtfJvmDetail {
    pub fn new(core: &ArmCore) -> Self {
        Self { core: core.clone() }
    }
}

#[async_trait::async_trait(?Send)]
impl JvmDetail for KtfJvmDetail {
    async fn load_class(&mut self, class_name: &str) -> JvmResult<Option<ClassRef>> {
        let class = ClassLoader::get_or_load_class(&mut self.core, class_name).await?;

        Ok(class.map(|x| Rc::new(RefCell::new(Box::new(x) as Box<dyn Class>))))
    }

    async fn load_array_class(&mut self, element_type_name: &str) -> JvmResult<Option<Box<dyn ArrayClass>>> {
        let class_name = format!("[{}", element_type_name);
        if let Some(x) = JavaContextData::find_class(&self.core, &class_name)? {
            let class = JavaArrayClass::from_raw(x.ptr_raw, &self.core);
            return Ok(Some(Box::new(class)));
        } else {
            let class = JavaArrayClass::new(&mut self.core, &class_name).await?;

            Ok(Some(Box::new(class)))
        }
    }

    fn get_class(&self, class_name: &str) -> JvmResult<Option<ClassRef>> {
        let class = JavaContextData::find_class(&self.core, class_name)?;

        Ok(class.map(|x| Rc::new(RefCell::new(Box::new(x) as Box<dyn Class>))))
    }

    fn thread_context(&mut self, _thread_id: ThreadId) -> &mut dyn ThreadContext {
        todo!()
    }
}

pub struct KtfJavaContext<'a> {
    core: &'a mut ArmCore,
    system: &'a mut System,
    jvm: Jvm,
}

impl<'a> KtfJavaContext<'a> {
    pub fn init(core: &mut ArmCore, ptr_vtables_base: u32, fn_get_class: u32) -> JavaResult<u32> {
        JavaContextData::init(core, ptr_vtables_base, fn_get_class)
    }

    pub fn new(core: &'a mut ArmCore, system: &'a mut System) -> Self {
        let jvm = Jvm::new(KtfJvmDetail::new(core));

        Self { core, system, jvm }
    }

    pub async fn load_class(&mut self, name: &str) -> JavaResult<Option<JavaClass>> {
        ClassLoader::get_or_load_class(self.core, name).await
    }

    pub async fn load_array_class(&mut self, name: &str) -> JavaResult<Option<JavaArrayClass>> {
        ClassLoader::load_array_class(self.core, name).await
    }

    pub async fn register_class(core: &mut ArmCore, class: &JavaClass) -> JavaResult<()> {
        if JavaContextData::has_class(core, class)? {
            return Ok(());
        }

        JavaContextData::register_class(core, class)?;

        let clinit = class.method("<clinit>", "()V")?;

        if let Some(x) = clinit {
            tracing::trace!("Call <clinit>");

            x.run(Box::new([])).await?;
        }

        Ok(())
    }

    pub fn class_from_raw(&self, ptr_class: u32) -> JavaClass {
        JavaClass::from_raw(ptr_class, self.core)
    }

    pub fn class_raw(&self, instance: &ClassInstanceRef) -> u32 {
        let instance = instance.borrow();
        if let Some(x) = instance.as_any().downcast_ref::<JavaClassInstance>() {
            x.ptr_raw
        } else {
            let instance = instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap();

            instance.class_instance.ptr_raw
        }
    }
}

#[async_trait::async_trait(?Send)]
impl JavaContext for KtfJavaContext<'_> {
    fn jvm(&mut self) -> &mut Jvm {
        &mut self.jvm
    }

    fn system(&mut self) -> &mut System {
        self.system
    }

    fn spawn(&mut self, callback: JavaMethodBody) -> JavaResult<()> {
        struct SpawnProxy {
            core: ArmCore,
            system: System,
            callback: JavaMethodBody,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, JavaError> for SpawnProxy {
            #[allow(clippy::await_holding_refcell_ref)] // We manually drop RefMut https://github.com/rust-lang/rust-clippy/issues/6353
            async fn call(mut self) -> Result<u32, JavaError> {
                let mut context = KtfJavaContext::new(&mut self.core, &mut self.system);
                let _ = self.callback.call(&mut context, Box::new([])).await?;

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

    fn sleep(&mut self, duration: u64) -> SleepFuture {
        let until = self.system.time().now() + duration;

        task::sleep(until)
    }
}
