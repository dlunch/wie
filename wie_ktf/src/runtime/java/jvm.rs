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

use alloc::{boxed::Box, rc::Rc, string::ToString};
use bytemuck::{Pod, Zeroable};

use wie_backend::SystemHandle;
use wie_common::util::write_generic;
use wie_core_arm::{ArmCore, PEB_BASE};

use jvm::{Class, ClassInstance, Jvm, JvmResult};

use crate::{
    context::KtfContext,
    runtime::{
        java::{jvm::class_loader::ClassLoaderContextBase, runtime::KtfRuntime},
        KtfPeb, KtfWIPIJavaContext,
    },
};

use self::{
    array_class::JavaArrayClass, array_class_instance::JavaArrayClassInstance, class::JavaClass, class_instance::JavaClassInstance,
    class_loader::KtfClassLoader, name::JavaFullName,
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
    pub async fn init(
        core: &mut ArmCore,
        system: &mut SystemHandle,
        ptr_vtables_base: u32,
        fn_get_class: u32,
        ptr_current_java_exception_handler: u32,
    ) -> JvmResult<()> {
        let ptr_java_context_data = context_data::JavaContextData::init(core, ptr_vtables_base, fn_get_class)?;

        core.map(PEB_BASE, 0x1000)?;
        write_generic(
            core,
            PEB_BASE,
            KtfPeb {
                ptr_java_context_data,
                ptr_current_java_exception_handler,
            },
        )?;

        let jvm = Jvm::new(detail::KtfJvmDetail::new(core, system)).await?;
        KtfContext::set_jvm(system, jvm);

        let jvm = KtfContext::jvm(system);

        let runtime = KtfRuntime::new(core, system);
        let core_clone = core.clone();
        let system_clone = system.clone();
        java_runtime::initialize(&jvm, move |name, proto| {
            let name = name.to_string();
            let mut core_clone = core_clone.clone();
            let mut system_clone = system_clone.clone();
            let runtime = runtime.clone();

            async move {
                Box::new(
                    JavaClass::new(&mut core_clone, &mut system_clone, &name, proto, Box::new(runtime) as Box<_>)
                        .await
                        .unwrap(),
                ) as Box<_>
            }
        })
        .await?;

        let context = KtfWIPIJavaContext::new(core, system);
        let core_clone = core.clone();
        let system_clone = system.clone();
        wie_wipi_java::register(&jvm, move |name, proto| {
            let name = name.to_string();
            let mut core_clone = core_clone.clone();
            let mut system_clone = system_clone.clone();
            let context = context.clone();

            async move {
                Box::new(
                    JavaClass::new(&mut core_clone, &mut system_clone, &name, proto, Box::new(context) as Box<_>)
                        .await
                        .unwrap(),
                ) as Box<_>
            }
        })
        .await?;

        #[derive(Clone)]
        struct ClassLoaderContext {
            core: ArmCore,
        }

        impl ClassLoaderContextBase for ClassLoaderContext {
            fn core(&mut self) -> &mut ArmCore {
                &mut self.core
            }
        }

        jvm.register_class(Box::new(
            JavaClass::new(
                core,
                system,
                "wie/KtfClassLoader",
                KtfClassLoader::as_proto(),
                Box::new(ClassLoaderContext { core: core.clone() }) as Box<_>,
            )
            .await?,
        ))
        .await?;

        let old_class_loader = jvm.get_system_class_loader().await?;
        let class_loader = jvm
            .new_class("wie/KtfClassLoader", "(Ljava/lang/ClassLoader;)V", (old_class_loader,))
            .await?;

        jvm.set_system_class_loader(class_loader);

        Ok(())
    }

    pub fn new(core: &ArmCore, system: &SystemHandle) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
        }
    }

    pub fn class_raw(&self, class: &dyn Class) -> u32 {
        if let Some(x) = class.as_any().downcast_ref::<JavaClass>() {
            x.ptr_raw
        } else {
            let class = class.as_any().downcast_ref::<JavaArrayClass>().unwrap();

            class.class.ptr_raw
        }
    }

    pub fn class_from_raw(&self, ptr_class: u32) -> JavaClass {
        JavaClass::from_raw(ptr_class, &self.core)
    }

    pub fn read_name(&self, ptr_name: u32) -> JvmResult<JavaFullName> {
        JavaFullName::from_ptr(&self.core, ptr_name)
    }

    #[allow(clippy::borrowed_box)]
    pub fn class_instance_raw(&self, instance: &Box<dyn ClassInstance>) -> u32 {
        if let Some(x) = instance.as_any().downcast_ref::<JavaClassInstance>() {
            x.ptr_raw
        } else {
            let instance = instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap();

            instance.class_instance.ptr_raw
        }
    }

    pub fn jvm(&mut self) -> Rc<Jvm> {
        KtfContext::jvm(&mut self.system)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;

    use wie_backend::{System, SystemHandle};
    use wie_core_arm::{Allocator, ArmCore};

    use crate::{context::KtfContext, runtime::java::jvm::KtfJvm};

    use test_utils::TestPlatform;

    async fn test_core(system: &mut SystemHandle) -> anyhow::Result<ArmCore> {
        let mut core = ArmCore::new(system.clone())?;
        Allocator::init(&mut core)?;

        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x100)?;
        context.sp = stack + 0x100;
        core.restore_context(&context);

        let ptr_vtables_base = Allocator::alloc(&mut core, 0x100)?;
        KtfJvm::init(&mut core, system, ptr_vtables_base, 0, 0).await?;

        Ok(core)
    }

    #[futures_test::test]
    async fn test_jvm() -> anyhow::Result<()> {
        let mut system = System::new(Box::new(TestPlatform), Box::new(KtfContext::new())).handle();
        let core = test_core(&mut system).await?;

        let jvm = KtfJvm::new(&core, &system).jvm();

        let string1 = String::from_rust_string(&jvm, "test1").await?;
        let string2 = String::from_rust_string(&jvm, "test2").await?;

        let string3 = jvm
            .invoke_virtual(
                &string1,
                "java/lang/String",
                "concat",
                "(Ljava/lang/String;)Ljava/lang/String;",
                [string2.into()],
            )
            .await?;

        assert_eq!(String::to_rust_string(&jvm, &string3)?, "test1test2");

        Ok(())
    }
}
