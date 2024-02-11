mod array_class_definition;
mod array_class_instance;
mod class_definition;
mod class_instance;
mod classes;
mod context_data;
mod detail;
mod field;
mod method;
mod name;
mod value;
mod vtable_builder;

use alloc::{boxed::Box, rc::Rc, string::ToString};
use bytemuck::{Pod, Zeroable};

use wie_backend::System;
use wie_common::util::write_generic;
use wie_core_arm::{ArmCore, PEB_BASE};

use jvm::{ClassDefinition, ClassInstance, Jvm, JvmResult};

use crate::{
    context::KtfContextExt,
    runtime::{java::runtime::KtfRuntime, KtfPeb, KtfWIPIJavaContext},
};

use self::{
    array_class_definition::JavaArrayClassDefinition,
    array_class_instance::JavaArrayClassInstance,
    class_definition::JavaClassDefinition,
    class_instance::JavaClassInstance,
    classes::wie::{ClassLoaderContextBase, KtfClassLoader},
    name::JavaFullName,
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

pub struct KtfJvmSupport;

impl KtfJvmSupport {
    pub async fn init(
        core: &mut ArmCore,
        system: &mut System,
        ptr_vtables_base: u32,
        fn_get_class: u32,
        ptr_current_java_exception_handler: u32,
    ) -> JvmResult<Rc<Jvm>> {
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
        system.set_jvm(Jvm::new(detail::KtfJvmDetail::new(core)).await?);

        let jvm = system.jvm();

        let runtime = KtfRuntime::new(core, system, jvm.clone());
        let core_clone = core.clone();
        let jvm_clone = jvm.clone();
        java_runtime::initialize(&jvm, move |name, proto| {
            let name = name.to_string();
            let mut core_clone = core_clone.clone();
            let jvm_clone = jvm_clone.clone();
            let runtime = runtime.clone();

            async move {
                Box::new(
                    JavaClassDefinition::new(&mut core_clone, &jvm_clone, &name, proto, Box::new(runtime) as Box<_>)
                        .await
                        .unwrap(),
                ) as Box<_>
            }
        })
        .await?;

        let context = KtfWIPIJavaContext::new(core, system, jvm.clone());
        let core_clone = core.clone();
        let jvm_clone = jvm.clone();
        wie_wipi_java::register(&jvm, move |name, proto| {
            let name = name.to_string();
            let mut core_clone = core_clone.clone();
            let jvm_clone = jvm_clone.clone();
            let context = context.clone();

            async move {
                Box::new(
                    JavaClassDefinition::new(&mut core_clone, &jvm_clone, &name, proto, Box::new(context) as Box<_>)
                        .await
                        .unwrap(),
                ) as Box<_>
            }
        })
        .await?;

        #[derive(Clone)]
        struct ClassLoaderContext {
            core: ArmCore,
            system: System,
        }

        impl ClassLoaderContextBase for ClassLoaderContext {
            fn core(&mut self) -> &mut ArmCore {
                &mut self.core
            }

            fn system(&self) -> &System {
                &self.system
            }
        }

        let class_loader_class = JavaClassDefinition::new(
            core,
            &jvm,
            "wie/KtfClassLoader",
            KtfClassLoader::as_proto(),
            Box::new(ClassLoaderContext {
                core: core.clone(),
                system: system.clone(),
            }) as Box<_>,
        )
        .await?;

        jvm.register_class(Box::new(class_loader_class), None).await?;

        let old_class_loader = jvm.get_system_class_loader().await?;
        let class_loader = jvm
            .new_class("wie/KtfClassLoader", "(Ljava/lang/ClassLoader;)V", (old_class_loader,))
            .await?;

        jvm.set_system_class_loader(class_loader);

        Ok(jvm)
    }

    pub fn class_definition_raw(definition: &dyn ClassDefinition) -> JvmResult<u32> {
        Ok(if let Some(x) = definition.as_any().downcast_ref::<JavaClassDefinition>() {
            x.ptr_raw
        } else {
            let class = definition.as_any().downcast_ref::<JavaArrayClassDefinition>().unwrap();

            class.class.ptr_raw
        })
    }

    pub fn class_from_raw(core: &ArmCore, ptr_class: u32) -> JavaClassDefinition {
        JavaClassDefinition::from_raw(ptr_class, core)
    }

    pub fn read_name(core: &ArmCore, ptr_name: u32) -> JvmResult<JavaFullName> {
        JavaFullName::from_ptr(core, ptr_name)
    }

    #[allow(clippy::borrowed_box)]
    pub fn class_instance_raw(instance: &Box<dyn ClassInstance>) -> u32 {
        if let Some(x) = instance.as_any().downcast_ref::<JavaClassInstance>() {
            x.ptr_raw
        } else {
            let instance = instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap();

            instance.class_instance.ptr_raw
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, rc::Rc};

    use jvm::{runtime::JavaLangString, Jvm};

    use wie_backend::System;
    use wie_core_arm::{Allocator, ArmCore};

    use crate::{context::KtfContext, runtime::java::jvm_support::KtfJvmSupport};

    use test_utils::TestPlatform;

    async fn init_jvm(system: &mut System) -> anyhow::Result<Rc<Jvm>> {
        let mut core = ArmCore::new(system.clone())?;
        Allocator::init(&mut core)?;

        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x100)?;
        context.sp = stack + 0x100;
        core.restore_context(&context);

        let ptr_vtables_base = Allocator::alloc(&mut core, 0x100)?;
        let jvm = KtfJvmSupport::init(&mut core, system, ptr_vtables_base, 0, 0).await?;

        Ok(jvm)
    }

    #[futures_test::test]
    async fn test_jvm_support() -> anyhow::Result<()> {
        let mut system = System::new(Box::new(TestPlatform), Box::new(KtfContext::new()));
        let jvm = init_jvm(&mut system).await?;

        let string1 = JavaLangString::from_rust_string(&jvm, "test1").await?;
        let string2 = JavaLangString::from_rust_string(&jvm, "test2").await?;

        let string3 = jvm
            .invoke_virtual(&string1, "concat", "(Ljava/lang/String;)Ljava/lang/String;", [string2.into()])
            .await?;

        assert_eq!(JavaLangString::to_rust_string(&jvm, string3)?, "test1test2");

        Ok(())
    }
}
