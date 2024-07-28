mod array_class_definition;
mod array_class_instance;
mod class_definition;
mod class_instance;
mod classes;
mod field;
mod method;
mod name;
mod value;
mod vtable_builder;

use alloc::{boxed::Box, string::ToString};
use core::{iter, mem::size_of};

use bytemuck::{Pod, Zeroable};

use java_runtime::{
    classes::java::util::{jar::JarEntry, Enumeration},
    Runtime,
};
use jvm::{runtime::JavaLangString, ClassDefinition, ClassInstance, ClassInstanceRef, Jvm};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{read_generic, read_null_terminated_table, write_generic};

use crate::runtime::{java::runtime::KtfRuntime, KtfWIPIJavaContext};

use self::{
    array_class_instance::JavaArrayClassInstance,
    class_instance::JavaClassInstance,
    classes::wie::{ClassLoaderContextBase, KtfClassLoader},
    name::JavaFullName,
};

pub use array_class_definition::JavaArrayClassDefinition;
pub use class_definition::JavaClassDefinition;

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
    pub async fn init(core: &mut ArmCore, system: &mut System, jar_name: Option<&str>) -> JvmSupportResult<(Jvm, Box<dyn ClassInstance>)> {
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

        let runtime = KtfRuntime::new(core, system);
        let properties = [("file.encoding", "EUC-KR")].into_iter();

        let properties = if let Some(x) = jar_name {
            properties.chain(iter::once(("java.class.path", x))).collect()
        } else {
            properties.collect()
        };

        let jvm = Jvm::new(
            java_runtime::get_bootstrap_class_loader(Box::new(runtime.clone())),
            move || runtime.current_task_id(),
            properties,
        )
        .await?;

        let context = KtfWIPIJavaContext::new(core, system, &jvm);
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
            jvm: Jvm,
        }

        impl ClassLoaderContextBase for ClassLoaderContext {
            fn core(&mut self) -> &mut ArmCore {
                &mut self.core
            }

            fn system(&mut self) -> &mut System {
                &mut self.system
            }

            fn jvm(&self) -> Jvm {
                self.jvm.clone()
            }
        }

        let client_bin = if let Some(x) = jar_name {
            // find client.bin
            let jar_name_java = JavaLangString::from_rust_string(&jvm, x).await?;
            let jar_file = jvm.new_class("java/util/jar/JarFile", "(Ljava/lang/String;)V", (jar_name_java,)).await?;
            let entries: ClassInstanceRef<Enumeration> = jvm.invoke_virtual(&jar_file, "entries", "()Ljava/util/Enumeration;", []).await?;

            loop {
                let has_more_elements: bool = jvm.invoke_virtual(&entries, "hasMoreElements", "()Z", []).await?;
                if !has_more_elements {
                    break None;
                }

                let entry: ClassInstanceRef<JarEntry> = jvm.invoke_virtual(&entries, "nextElement", "()Ljava/lang/Object;", []).await?;
                let name = jvm.invoke_virtual(&entry, "getName", "()Ljava/lang/String;", []).await?;
                let name_rust = JavaLangString::to_rust_string(&jvm, &name).await?;

                if name_rust.contains("client.bin") {
                    break Some(name);
                }
            }
        } else {
            None
        };

        let class_loader_class = JavaClassDefinition::new(
            core,
            &jvm,
            "wie/KtfClassLoader",
            KtfClassLoader::as_proto(),
            Box::new(ClassLoaderContext {
                core: core.clone(),
                system: system.clone(),
                jvm: jvm.clone(),
            }) as Box<_>,
        )
        .await?;

        jvm.register_class(Box::new(class_loader_class), None).await?;

        let system_class_loader: Box<dyn ClassInstance> = jvm
            .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", [])
            .await?;

        let class_loader = jvm
            .new_class(
                "wie/KtfClassLoader",
                "(Ljava/lang/ClassLoader;Ljava/lang/String;II)V",
                (system_class_loader, client_bin, ptr_jvm_context as i32, ptr_jvm_exception_context as i32),
            )
            .await?;

        Ok((jvm, class_loader))
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
    use core::sync::atomic::{AtomicBool, Ordering};

    use jvm::{runtime::JavaLangString, Jvm};

    use wie_backend::System;
    use wie_core_arm::{Allocator, ArmCore};

    use crate::runtime::java::jvm_support::KtfJvmSupport;

    use test_utils::TestPlatform;

    async fn init_jvm(system: &mut System) -> anyhow::Result<Jvm> {
        let mut core = ArmCore::new()?;
        Allocator::init(&mut core)?;

        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x100)?;
        context.sp = stack + 0x100;
        core.restore_context(&context);

        let (jvm, _) = KtfJvmSupport::init(&mut core, system, None).await?;

        Ok(jvm)
    }

    #[test]
    fn test_jvm_support() -> anyhow::Result<()> {
        let mut system = System::new(Box::new(TestPlatform));

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(|| async move {
            let jvm = init_jvm(&mut system_clone).await?;

            let string1 = JavaLangString::from_rust_string(&jvm, "test1").await?;
            let string2 = JavaLangString::from_rust_string(&jvm, "test2").await?;

            let string3 = jvm
                .invoke_virtual(&string1, "concat", "(Ljava/lang/String;)Ljava/lang/String;", [string2.into()])
                .await?;

            assert_eq!(JavaLangString::to_rust_string(&jvm, &string3).await?, "test1test2");

            done_clone.store(true, Ordering::Relaxed);

            anyhow::Ok(())
        });

        loop {
            system.tick()?;
            if done.load(Ordering::Relaxed) {
                break;
            }
        }

        Ok(())
    }
}
