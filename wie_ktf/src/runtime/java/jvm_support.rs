mod array_class_definition;
mod array_class_instance;
mod class_definition;
mod class_instance;
mod classes;
mod field;
mod jvm_implementation;
mod method;
mod name;
mod value;
mod vtable_builder;

use alloc::boxed::Box;
use core::mem::size_of;
use jvm_implementation::KtfJvmImplementation;

use bytemuck::{Pod, Zeroable};

use java_runtime::classes::java::util::{jar::JarEntry, Enumeration};
use jvm::{runtime::JavaLangString, ClassDefinition, ClassInstance, ClassInstanceRef, Jvm};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_util::{read_generic, read_null_terminated_table, write_generic, Result};

use self::{
    array_class_instance::JavaArrayClassInstance,
    class_instance::JavaClassInstance,
    classes::net::wie::{ClassLoaderContext, KtfClassLoader},
    name::JavaFullName,
};

pub use array_class_definition::JavaArrayClassDefinition;
pub use class_definition::JavaClassDefinition;
pub use method::{JavaMethod, JavaMethodResult};

pub type KtfJvmWord = u32;

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
    ptr_vtables: [u32; 128],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct KtfJvmSupportContext {
    ptr_vtables_base: u32,
    class_loader: u32,
    ptr_jvm_exception_context: u32,
}

const SUPPORT_CONTEXT_BASE: u32 = 0x7fff0000;

pub struct KtfJvmSupport;

impl KtfJvmSupport {
    pub async fn init(core: &mut ArmCore, system: &mut System, jar_name: Option<&str>) -> Result<(Jvm, Box<dyn ClassInstance>)> {
        let jvm_context = KtfJvmContext {
            unk1: 0,
            unk2: 0,
            unk3: 0,
            ptr_vtables: [0; 128],
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
            class_loader: 0,
            ptr_jvm_exception_context,
        };
        core.map(SUPPORT_CONTEXT_BASE, 0x1000)?;
        write_generic(core, SUPPORT_CONTEXT_BASE, context_data)?;

        let protos = [wie_wipi_java::get_protos().into(), wie_midp::get_protos().into()];
        let jvm = JvmSupport::new_jvm(system, jar_name, Box::new(protos), &[], KtfJvmImplementation::new(core.clone())).await?;

        let client_bin = if let Some(x) = jar_name {
            // find client.bin
            let jar_name_java = JavaLangString::from_rust_string(&jvm, x).await.unwrap();
            let jar_file = jvm
                .new_class("java/util/jar/JarFile", "(Ljava/lang/String;)V", (jar_name_java,))
                .await
                .unwrap();
            let entries: ClassInstanceRef<Enumeration> = jvm.invoke_virtual(&jar_file, "entries", "()Ljava/util/Enumeration;", []).await.unwrap();

            loop {
                let has_more_elements: bool = jvm.invoke_virtual(&entries, "hasMoreElements", "()Z", []).await.unwrap();
                if !has_more_elements {
                    break None;
                }

                let entry: ClassInstanceRef<JarEntry> = jvm.invoke_virtual(&entries, "nextElement", "()Ljava/lang/Object;", []).await.unwrap();
                let name = jvm.invoke_virtual(&entry, "getName", "()Ljava/lang/String;", []).await.unwrap();
                let name_rust = JavaLangString::to_rust_string(&jvm, &name).await.unwrap();

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
            KtfClassLoader::as_proto(),
            Box::new(ClassLoaderContext {
                core: core.clone(),
                system: system.clone(),
            }) as Box<_>,
        )
        .await?;

        jvm.register_class(Box::new(class_loader_class), None).await.unwrap();

        let system_class_loader: Box<dyn ClassInstance> = jvm
            .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", [])
            .await
            .unwrap();

        let class_loader = jvm
            .new_class(
                "net/wie/KtfClassLoader",
                "(Ljava/lang/ClassLoader;Ljava/lang/String;II)V",
                (system_class_loader, client_bin, ptr_jvm_context as i32, ptr_jvm_exception_context as i32),
            )
            .await
            .unwrap();

        let context_data = KtfJvmSupportContext {
            ptr_vtables_base: ptr_jvm_context + 12,
            class_loader: KtfJvmSupport::class_instance_raw(&class_loader),
            ptr_jvm_exception_context,
        };
        write_generic(core, SUPPORT_CONTEXT_BASE, context_data)?;

        Ok((jvm, class_loader))
    }

    pub fn class_loader(core: &ArmCore) -> Result<Box<dyn ClassInstance>> {
        let context_data: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;

        Ok(Box::new(JavaClassInstance::from_raw(context_data.class_loader, core)))
    }

    pub fn class_definition_raw(definition: &dyn ClassDefinition) -> Result<u32> {
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

    pub fn read_name(core: &ArmCore, ptr_name: u32) -> Result<JavaFullName> {
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

    pub fn get_vtable_index(core: &mut ArmCore, class: &JavaClassDefinition) -> Result<u32> {
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

    pub fn current_java_exception_handler(core: &mut ArmCore) -> Result<u32> {
        let context_data: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
        let exception_context: KtfJvmExceptionContext = read_generic(core, context_data.ptr_jvm_exception_context)?;

        Ok(exception_context.current_java_exception_handler)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use core::sync::atomic::{AtomicBool, Ordering};

    use jvm::{runtime::JavaLangString, Jvm};

    use wie_backend::System;
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::Result;

    use crate::runtime::java::jvm_support::KtfJvmSupport;

    use test_utils::TestPlatform;

    async fn init_jvm(system: &mut System) -> Result<Jvm> {
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
    fn test_jvm_support() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform), "");

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(|| async move {
            let jvm = init_jvm(&mut system_clone).await?;

            let string1 = JavaLangString::from_rust_string(&jvm, "test1").await.unwrap();
            let string2 = JavaLangString::from_rust_string(&jvm, "test2").await.unwrap();

            let string3 = jvm
                .invoke_virtual(&string1, "concat", "(Ljava/lang/String;)Ljava/lang/String;", [string2.into()])
                .await
                .unwrap();

            assert_eq!(JavaLangString::to_rust_string(&jvm, &string3).await.unwrap(), "test1test2");

            let mut array = jvm.instantiate_array("S", 10).await.unwrap();
            jvm.store_array(&mut array, 0, (0..10i16).collect::<Vec<_>>()).await.unwrap();
            let temp: Vec<i16> = jvm.load_array(&array, 5, 4).await.unwrap();

            assert_eq!(temp, vec![5, 6, 7, 8]);

            done_clone.store(true, Ordering::Relaxed);

            // test 64bit parameter passing
            let date = jvm.new_class("java/util/Date", "(J)V", (0x12345678_abcdef01i64,)).await.unwrap();
            let time: i64 = jvm.invoke_virtual(&date, "getTime", "()J", ()).await.unwrap();

            assert_eq!(time, 0x12345678_abcdef01);

            Ok(())
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
