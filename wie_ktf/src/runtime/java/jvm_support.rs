mod array_class_definition;
mod array_class_instance;
mod class_definition;
mod class_instance;
mod classes;
mod detail;
mod field;
mod method;
mod name;
mod value;
mod vtable_builder;

use alloc::{boxed::Box, string::ToString, sync::Arc};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{read_generic, read_null_terminated_table, write_generic};

use jvm::{runtime::JavaLangString, ClassDefinition, ClassInstance, Jvm};

use crate::{
    context::KtfContextExt,
    runtime::{java::runtime::KtfRuntime, KtfWIPIJavaContext},
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

type JvmSupportResult<T> = anyhow::Result<T>;

#[allow(dead_code)]
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct KtfJvmExceptionContext {
    unk: [u32; 8],
    current_java_exception_handler: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct KtfJvmContext {
    unk1: u32,
    unk2: u32,
    unk3: u32,
    ptr_vtables: [u32; 64],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct KtfJvmSupportContext {
    ptr_vtables_base: u32,
}

const SUPPORT_CONTEXT_BASE: u32 = 0x7fff0000;

pub struct KtfJvmSupport;

impl KtfJvmSupport {
    pub async fn init(core: &mut ArmCore, system: &mut System) -> JvmSupportResult<Arc<Jvm>> {
        let jvm_context = KtfJvmContext {
            unk1: 0,
            unk2: 0,
            unk3: 0,
            ptr_vtables: [0; 64],
        };
        let ptr_jvm_context = Allocator::alloc(core, size_of::<KtfJvmContext>() as u32)?;
        write_generic(core, ptr_jvm_context, jvm_context)?;

        let jvm_exception_context = KtfJvmExceptionContext {
            unk: [0; 8],
            current_java_exception_handler: 0,
        };
        let ptr_jvm_exception_context = Allocator::alloc(core, size_of::<KtfJvmExceptionContext>() as u32)?;
        write_generic(core, ptr_jvm_exception_context, jvm_exception_context)?;

        let context_data = KtfJvmSupportContext {
            ptr_vtables_base: ptr_jvm_context + 12,
        };
        core.map(SUPPORT_CONTEXT_BASE, 0x1000)?;
        write_generic(core, SUPPORT_CONTEXT_BASE, context_data)?;

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

        let client_bin = system
            .filesystem()
            .files()
            .find(|(x, _)| x.starts_with("client.bin"))
            .map(|x| x.0.to_string());
        if let Some(client_bin) = client_bin {
            let client_bin = JavaLangString::from_rust_string(&jvm, &client_bin).await?;

            let old_class_loader = jvm.get_system_class_loader().await?;
            let class_loader = jvm
                .new_class(
                    "wie/KtfClassLoader",
                    "(Ljava/lang/ClassLoader;Ljava/lang/String;II)V",
                    (old_class_loader, client_bin, ptr_jvm_context as i32, ptr_jvm_exception_context as i32),
                )
                .await?;

            jvm.set_system_class_loader(class_loader).await;
        }

        // set initial properties... TODO can we merge this with wie_core_jvm's one?
        let file_encoding_name = JavaLangString::from_rust_string(&jvm, "file.encoding").await?;
        let encoding_str = JavaLangString::from_rust_string(&jvm, "EUC-KR").await?; // TODO hardcoded
        let _: Option<Box<dyn ClassInstance>> = jvm
            .invoke_static(
                "java/lang/System",
                "setProperty",
                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;",
                (file_encoding_name, encoding_str),
            )
            .await?;

        Ok(jvm)
    }

    pub fn class_definition_raw(definition: &dyn ClassDefinition) -> JvmSupportResult<u32> {
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

    pub fn read_name(core: &ArmCore, ptr_name: u32) -> JvmSupportResult<JavaFullName> {
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

    pub fn get_vtable_index(core: &mut ArmCore, class: &JavaClassDefinition) -> JvmSupportResult<u32> {
        // TODO remove context
        let context_data: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
        let ptr_vtables = read_null_terminated_table(core, context_data.ptr_vtables_base)?;

        let ptr_vtable = class.ptr_vtable()?;

        for (index, &current_ptr_vtable) in ptr_vtables.iter().enumerate() {
            if ptr_vtable == current_ptr_vtable {
                return Ok(index as _);
            }
        }

        let index = ptr_vtables.len();
        write_generic(core, context_data.ptr_vtables_base + (index * size_of::<u32>()) as u32, ptr_vtable)?;

        Ok(index as _)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, sync::Arc};

    use jvm::{runtime::JavaLangString, Jvm};

    use wie_backend::System;
    use wie_core_arm::{Allocator, ArmCore};

    use crate::{context::KtfContext, runtime::java::jvm_support::KtfJvmSupport};

    use test_utils::TestPlatform;

    async fn init_jvm(system: &mut System) -> anyhow::Result<Arc<Jvm>> {
        let mut core = ArmCore::new(system.clone())?;
        Allocator::init(&mut core)?;

        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x100)?;
        context.sp = stack + 0x100;
        core.restore_context(&context);

        let jvm = KtfJvmSupport::init(&mut core, system).await?;

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

        assert_eq!(JavaLangString::to_rust_string(&jvm, &string3).await?, "test1test2");

        Ok(())
    }
}
