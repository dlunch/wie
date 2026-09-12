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
mod vtable;

use alloc::boxed::Box;
use core::mem::{offset_of, size_of};
use jvm_implementation::KtfJvmImplementation;

use bytemuck::{Pod, Zeroable};
use futures::TryFutureExt;

use jvm::{ClassDefinition, ClassInstance, ClassInstanceRef, Field, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_types::FieldAccessFlags;
use rustjava_runtime::classes::java::util::{Enumeration, jar::JarEntry};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_midp::classes::javax::microedition::midlet::MIDlet;
use wie_util::{Result, WieError, read_generic, read_null_terminated_table, write_generic};

use wipi_types::ktf::{ExeInterfaceFunctions, InitParam2, java::JavaClass as RawJavaClass};

use self::{
    array_class_instance::JavaArrayClassInstance,
    classes::net::wie::{ClassLoaderContext, KtfClassLoader},
    name::JavaFullName,
};
use super::interface::register_java_interface_svc_handler;

pub use self::{
    array_class_definition::JavaArrayClassDefinition,
    class_definition::JavaClassDefinition,
    class_instance::JavaClassInstance,
    method::{JavaMethod, JavaMethodResult},
    vtable::JavaVtable,
};

pub type KtfJvmWord = u32;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct KtfJvmThreadContext {
    unk: [u32; 8],
    current_java_exception_handler: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct KtfJvmSupportContext {
    ptr_vtables_base: u32,
    ptr_current_jvm_thread_context: u32,
}

const SUPPORT_CONTEXT_BASE: u32 = 0x7fff0000;

pub struct KtfJvmSupport;

impl KtfJvmSupport {
    pub async fn init(core: &mut ArmCore, system: &mut System, jar_name: Option<&str>) -> Result<(Jvm, Box<dyn ClassInstance>)> {
        let jvm_context = InitParam2 {
            unk1: 0,
            unk2: 0,
            unk3: 0,
            ptr_java_vtables: [0; 128],
        };
        let ptr_jvm_context = Allocator::alloc(core, size_of::<InitParam2>() as u32)?;
        write_generic(core, ptr_jvm_context, jvm_context)?;

        write_generic(
            core,
            SUPPORT_CONTEXT_BASE + offset_of!(KtfJvmSupportContext, ptr_vtables_base) as u32,
            ptr_jvm_context + 12,
        )?;

        let protos = [wie_wipi_java::get_protos().into(), wie_midp::get_protos().into()];
        let jvm_implementation = KtfJvmImplementation::new(core);
        let jvm = JvmSupport::new_jvm(system, jar_name, Box::new(protos), &[], jvm_implementation.clone()).await?;
        register_java_interface_svc_handler(core, &jvm)?;

        let system_class_loader: Box<dyn ClassInstance> = jvm
            .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", [])
            .await
            .unwrap();

        // used in tests
        if jar_name.is_none() {
            return Ok((jvm, system_class_loader));
        }

        // find client.bin
        let jar_name_java = JavaLangString::from_rust_string(&jvm, jar_name.unwrap()).await.unwrap();
        let jar_file = jvm
            .new_class("java/util/jar/JarFile", "(Ljava/lang/String;)V", (jar_name_java,))
            .await
            .unwrap();
        let entries: ClassInstanceRef<Enumeration> = jvm
            .invoke_virtual(&jar_file, "java/util/jar/JarFile", "entries", "()Ljava/util/Enumeration;", [])
            .await
            .unwrap();

        let binary_name = loop {
            let has_more_elements: bool = jvm
                .invoke_virtual(&entries, "java/util/Enumeration", "hasMoreElements", "()Z", [])
                .await
                .unwrap();
            if !has_more_elements {
                return Err(WieError::FatalError("client.bin not found".into()));
            }

            let entry: ClassInstanceRef<JarEntry> = jvm
                .invoke_virtual(&entries, "java/util/Enumeration", "nextElement", "()Ljava/lang/Object;", [])
                .await
                .unwrap();
            let name = jvm
                .invoke_virtual(&entry, "java/util/jar/JarEntry", "getName", "()Ljava/lang/String;", [])
                .await
                .unwrap();
            let name_rust = JavaLangString::to_rust_string(&jvm, &name).await.unwrap();

            if name_rust.starts_with("client.bin") {
                break name;
            }
        };

        let class_loader_class = JavaClassDefinition::new(
            core,
            &jvm,
            KtfClassLoader::as_proto(),
            Box::new(ClassLoaderContext {
                core: core.clone(),
                system: system.clone(),
            }) as Box<_>,
            jvm_implementation.java_functions(),
        )
        .await?;

        jvm.register_class(Box::new(class_loader_class), None).await.unwrap();

        let class_loader = jvm
            .new_class(
                "net/wie/KtfClassLoader",
                "(Ljava/lang/ClassLoader;Ljava/lang/String;II)V",
                (
                    system_class_loader,
                    binary_name,
                    ptr_jvm_context as i32,
                    (SUPPORT_CONTEXT_BASE + offset_of!(KtfJvmSupportContext, ptr_current_jvm_thread_context) as u32) as i32,
                ),
            )
            .await
            .unwrap();

        Ok((jvm, class_loader))
    }

    // Native initialization must have linked the class catalog before registration.
    pub(crate) async fn register_static_classes(
        core: &mut ArmCore,
        jvm: &Jvm,
        class_loader: Box<dyn ClassInstance>,
        main_class_name: &str,
    ) -> Result<()> {
        let ptr_functions: i32 = jvm
            .get_field(&class_loader, "nativeFunctions", "I")
            .or_else(async |error| Err(JvmSupport::to_wie_err(jvm, error).await))
            .await?;
        let functions: ExeInterfaceFunctions = read_generic(core, ptr_functions as u32)?;
        let predicate = functions.fn_is_native_class_address;
        if predicate == 0 {
            return Ok(());
        }

        let main_class = jvm
            .resolve_class(main_class_name)
            .or_else(async |error| Err(JvmSupport::to_wie_err(jvm, error).await))
            .await?;
        let mut ptr_class = Self::class_definition_raw(&*main_class.definition)?;
        if core.run_function::<u32>(predicate, &[ptr_class]).await? == 0 {
            return Ok(());
        }

        let class_size = size_of::<RawJavaClass>() as u32;
        while core.run_function::<u32>(predicate, &[ptr_class - class_size]).await? != 0 {
            ptr_class -= class_size;
        }
        while core.run_function::<u32>(predicate, &[ptr_class]).await? != 0 {
            let class = JavaClassDefinition::from_raw(ptr_class, core);
            let name = class.name()?;
            if !jvm.has_class(&name)
                && class
                    .fields()?
                    .iter()
                    .any(|field| field.access_flags().contains(FieldAccessFlags::STATIC))
            {
                jvm.register_class(Box::new(class), Some(class_loader.clone()))
                    .or_else(async |error| Err(JvmSupport::to_wie_err(jvm, error).await))
                    .await?;
            }
            ptr_class += class_size;
        }

        Ok(())
    }

    pub(crate) async fn disable_midp_paint(jvm: &Jvm) -> JvmResult<()> {
        let midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;
        let display = MIDlet::display(jvm, &midlet).await?;

        jvm.invoke_virtual(&display, "javax/microedition/lcdui/Display", "disablePaint", "()V", ())
            .await
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
        let ptr_thread_context = Self::current_thread_context(core)?;
        let thread_context: KtfJvmThreadContext = read_generic(core, ptr_thread_context)?;

        Ok(thread_context.current_java_exception_handler)
    }

    pub fn set_current_thread_context(core: &mut ArmCore, ptr_thread_context: u32) -> Result<()> {
        write_generic(
            core,
            SUPPORT_CONTEXT_BASE + offset_of!(KtfJvmSupportContext, ptr_current_jvm_thread_context) as u32,
            ptr_thread_context,
        )
    }

    pub fn current_thread_context(core: &ArmCore) -> Result<u32> {
        let context_data: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;

        Ok(context_data.ptr_current_jvm_thread_context)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use core::{
        mem::size_of,
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    };

    use bytemuck::Zeroable;
    use jvm::{
        ClassInstanceRef, JavaValue, Jvm,
        runtime::{JavaLangClass, JavaLangString},
    };
    use jvm_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::Class;
    use wipi_types::ktf::{
        ExeInterfaceFunctions,
        java::{
            JavaClass as RawJavaClass, JavaClassInstance as RawJavaClassInstance, JavaFieldDefinition as RawJavaField,
            JavaMethodDefinition as RawJavaMethod,
        },
    };

    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_jvm_support::native::encode_method_arguments;
    use wie_midp::classes::javax::microedition::{lcdui::Display as MidpDisplay, midlet::MIDlet};
    use wie_util::{Result, WieError, read_generic, write_generic};

    use crate::runtime::java::{JavaSvcFunctions, handle_java_svc};

    use super::{
        ClassLoaderContext, JavaArrayClassInstance, JavaClassDefinition, JavaClassInstance, JavaMethod, KtfClassLoader, KtfJvmSupport,
        KtfJvmThreadContext, value::JavaValueCodec,
    };

    use test_utils::{TestClock, TestPlatform};

    async fn init_jvm(system: &mut System) -> Result<(Jvm, ArmCore)> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x100)?;
        context.sp = stack + 0x100;
        core.restore_context(&context);

        let ptr_thread_context = Allocator::alloc(&mut core, size_of::<KtfJvmThreadContext>() as u32)?;
        write_generic(&mut core, ptr_thread_context, KtfJvmThreadContext::zeroed())?;
        KtfJvmSupport::set_current_thread_context(&mut core, ptr_thread_context)?;

        let (jvm, _) = KtfJvmSupport::init(&mut core, system, None).await?;

        Ok((jvm, core))
    }

    #[test]
    fn test_register_static_classes_keeps_live_guest_roots_without_initializing() -> Result<()> {
        async fn is_native_class(_core: &mut ArmCore, range: &mut core::ops::Range<u32>, address: u32) -> Result<u32> {
            Ok(u32::from(range.contains(&address)))
        }

        async fn count_initialization(_jvm: &Jvm, count: &mut Arc<AtomicUsize>) -> jvm::Result<()> {
            count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let java_functions = JavaSvcFunctions::default();
            core.register_svc_handler(5, handle_java_svc, &java_functions)?;
            let loader_class = JavaClassDefinition::new(
                &mut core.clone(),
                &jvm,
                KtfClassLoader::as_proto(),
                Box::new(ClassLoaderContext {
                    core: core.clone(),
                    system: system_clone.clone(),
                }),
                java_functions.clone(),
            )
            .await?;
            for method in loader_class.methods()? {
                let mut raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
                raw.fn_body = core.make_svc_stub(5, method.ptr_raw)?;
                write_generic(&mut core, method.ptr_raw, raw)?;
            }
            jvm.register_class(Box::new(loader_class), None).await.unwrap();
            let mut loader = jvm.instantiate_class("net/wie/KtfClassLoader").await.unwrap();
            let ptr_functions = Allocator::alloc(&mut core, size_of::<ExeInterfaceFunctions>() as u32)?;
            let mut functions = ExeInterfaceFunctions::zeroed();
            write_generic(&mut core, ptr_functions, functions)?;
            jvm.put_field(&mut loader, "nativeFunctions", "I", ptr_functions as i32).await.unwrap();

            let missing_name = JavaLangString::from_rust_string(&jvm, "test/Missing").await.unwrap();
            let missing: ClassInstanceRef<Class> = jvm
                .invoke_virtual(
                    &loader,
                    "net/wie/KtfClassLoader",
                    "findClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    (missing_name,),
                )
                .await
                .unwrap();
            assert!(missing.is_null());

            let class_size = size_of::<RawJavaClass>() as u32;
            let begin = Allocator::alloc(&mut core, class_size * 4)?;
            let end = begin + class_size * 4;
            core.register_svc_handler(6, is_native_class, &(begin..end))?;
            let predicate = core.make_svc_stub(6, 0u32)?;
            let initialization_count = Arc::new(AtomicUsize::new(0));
            let mut classes = Vec::new();
            for (index, name) in ["test/Before", "test/Unused", "test/Main", "test/After"].into_iter().enumerate() {
                let class = JavaClassDefinition::new(
                    &mut core,
                    &jvm,
                    JavaClassProto {
                        name,
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec![],
                        methods: if index == 0 {
                            vec![JavaMethodProto::new("<clinit>", "()V", count_initialization, MethodAccessFlags::STATIC)]
                        } else {
                            vec![]
                        },
                        fields: vec![JavaFieldProto::new(
                            "root",
                            "Ljava/lang/Object;",
                            if index == 1 {
                                FieldAccessFlags::PUBLIC
                            } else {
                                FieldAccessFlags::STATIC
                            },
                        )],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(initialization_count.clone()),
                    java_functions.clone(),
                )
                .await?;

                // Relocate only the class headers into the native ABI's contiguous table.
                let ptr_class = begin + index as u32 * class_size;
                let mut raw: RawJavaClass = read_generic(&core, class.ptr_raw)?;
                raw.ptr_next = ptr_class + 4;
                write_generic(&mut core, ptr_class, raw)?;
                for field in class.fields()? {
                    let mut raw: RawJavaField = read_generic(&core, field.ptr_raw)?;
                    raw.ptr_class = ptr_class;
                    write_generic(&mut core, field.ptr_raw, raw)?;
                }
                for method in class.methods()? {
                    let mut raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
                    raw.ptr_class = ptr_class;
                    raw.fn_body = core.make_svc_stub(5, method.ptr_raw)?;
                    write_generic(&mut core, method.ptr_raw, raw)?;
                }
                classes.push(JavaClassDefinition::from_raw(ptr_class, &core));
            }
            let seed = jvm
                .register_class(Box::new(classes[2].clone()), Some(loader.clone()))
                .await
                .unwrap()
                .unwrap();
            let seed_identity = seed.identity();

            KtfJvmSupport::register_static_classes(&mut core, &jvm, loader.clone(), "test/Main").await?;
            assert!(!jvm.has_class("test/Before"));
            assert!(!jvm.has_class("test/After"));
            functions.fn_is_native_class_address = predicate;
            write_generic(&mut core, ptr_functions, functions)?;
            KtfJvmSupport::register_static_classes(&mut core, &jvm, loader.clone(), "java/lang/Object").await?;
            assert!(!jvm.has_class("test/Before"));
            assert!(!jvm.has_class("test/After"));

            let mut identities = Vec::new();
            for pass in 0..2 {
                KtfJvmSupport::register_static_classes(&mut core, &jvm, loader.clone(), "test/Main").await?;
                assert!(!jvm.has_class("test/Unused"));
                for (index, name) in ["test/Before", "test/Main", "test/After"].into_iter().enumerate() {
                    assert!(jvm.has_class(name));
                    let class = jvm.resolve_class(name).await.unwrap().java_class();
                    let registered_loader = JavaLangClass::class_loader(&jvm, &class).await.unwrap().unwrap();
                    assert_eq!(registered_loader.identity(), loader.identity());
                    if pass == 0 {
                        identities.push(class.identity());
                    } else {
                        assert_eq!(class.identity(), identities[index]);
                    }
                }
                assert_eq!(identities[1], seed_identity);
                assert_eq!(initialization_count.load(Ordering::Relaxed), 0);
            }
            for class in &classes {
                let raw: RawJavaClass = read_generic(&core, class.ptr_raw)?;
                assert_eq!(raw.unk_flag, 8);
            }

            let before_root = classes[0].field("root", "Ljava/lang/Object;", true)?.unwrap();
            let after_root = classes[3].field("root", "Ljava/lang/Object;", true)?.unwrap();
            jvm.pop_frame();
            jvm.collect_garbage().unwrap();
            jvm.push_native_frame();
            let mut objects = Vec::new();
            for _ in 0..3 {
                let object = jvm.new_class("java/lang/Object", "()V", ()).await.unwrap();
                objects.push(KtfJvmSupport::class_instance_raw(&object));
            }
            classes[0].write_static_field(&before_root, objects[0])?;
            classes[3].write_static_field(&after_root, objects[1])?;
            jvm.pop_frame();
            jvm.collect_garbage().unwrap();
            let instance_size = size_of::<RawJavaClassInstance>() as u32;
            assert!(Allocator::is_allocated(&core, objects[0], instance_size)?);
            assert!(Allocator::is_allocated(&core, objects[1], instance_size)?);
            assert!(!Allocator::is_allocated(&core, objects[2], instance_size)?);

            jvm.push_native_frame();
            let replacement = jvm.new_class("java/lang/Object", "()V", ()).await.unwrap();
            let replacement = KtfJvmSupport::class_instance_raw(&replacement);
            classes[0].write_static_field(&before_root, replacement)?;
            jvm.pop_frame();
            jvm.collect_garbage().unwrap();
            assert!(!Allocator::is_allocated(&core, objects[0], instance_size)?);
            assert!(Allocator::is_allocated(&core, objects[1], instance_size)?);
            assert!(Allocator::is_allocated(&core, replacement, instance_size)?);

            classes[0].write_static_field(&before_root, 0)?;
            classes[3].write_static_field(&after_root, 0)?;
            jvm.collect_garbage().unwrap();
            assert!(!Allocator::is_allocated(&core, objects[1], instance_size)?);
            assert!(!Allocator::is_allocated(&core, replacement, instance_size)?);
            assert_eq!(initialization_count.load(Ordering::Relaxed), 0);
            jvm.push_native_frame();
            jvm.ensure_initialized(&jvm.resolve_class("test/Before").await.unwrap()).await.unwrap();
            assert_eq!(initialization_count.load(Ordering::Relaxed), 1);
            jvm.pop_frame();

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_jvm_support() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, _core) = init_jvm(&mut system_clone).await?;

            let midlet: ClassInstanceRef<MIDlet> = jvm.new_class("net/wie/WIPIMIDlet", "()V", ()).await.unwrap().into();
            let display: ClassInstanceRef<MidpDisplay> = MIDlet::display(&jvm, &midlet).await.unwrap();
            let paint_disabled: bool = jvm.get_field(&display, "paintDisabled", "Z").await.unwrap();
            assert!(!paint_disabled);

            KtfJvmSupport::disable_midp_paint(&jvm).await.unwrap();

            let paint_disabled: bool = jvm.get_field(&display, "paintDisabled", "Z").await.unwrap();
            assert!(paint_disabled);

            let string1 = JavaLangString::from_rust_string(&jvm, "test1").await.unwrap();
            let string2 = JavaLangString::from_rust_string(&jvm, "test2").await.unwrap();

            let string3 = jvm
                .invoke_virtual(
                    &string1,
                    "java/lang/String",
                    "concat",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    [string2.into()],
                )
                .await
                .unwrap();

            assert_eq!(JavaLangString::to_rust_string(&jvm, &string3).await.unwrap(), "test1test2");

            let mut array = jvm.instantiate_array("S", 10).await.unwrap();
            jvm.store_array(&mut array, 0, (0..10i16).collect::<Vec<_>>()).await.unwrap();
            let temp: Vec<i16> = jvm.load_array(&array, 5, 4).await.unwrap();

            assert_eq!(temp, vec![5, 6, 7, 8]);

            // test 64bit parameter passing
            let date = jvm.new_class("java/util/Date", "(J)V", (0x12345678_abcdef01i64,)).await.unwrap();
            let time: i64 = jvm.invoke_virtual(&date, "java/util/Date", "getTime", "()J", ()).await.unwrap();

            assert_eq!(time, 0x12345678_abcdef01);

            let calendar = jvm.new_class("java/util/GregorianCalendar", "()V", ()).await.unwrap();
            assert!(jvm.is_instance(&*calendar, "java/util/Calendar"));
            let cloneable = jvm.resolve_class("java/lang/Cloneable").await.unwrap();
            assert!(cloneable.definition.access_flags().contains(ClassAccessFlags::INTERFACE));

            done_clone.store(true, Ordering::Relaxed);

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

    #[test]
    fn test_non_native_method_through_native_entry() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let mut chars = jvm.instantiate_array("C", 4).await.unwrap();
            jvm.store_array(&mut chars, 0, vec![0x41u16, 0xd654, 0xc7a5, 0x42]).await.unwrap();
            let class = jvm.resolve_class("java/lang/String").await.unwrap();
            let class = class.definition.as_any().downcast_ref::<JavaClassDefinition>().unwrap();
            let method = class.method("valueOf", "([CII)Ljava/lang/String;", true)?.unwrap();
            let raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
            assert!(!MethodAccessFlags::from_bits_truncate(raw.access_flags).contains(MethodAccessFlags::NATIVE));
            assert_eq!(raw.exception_table_count, 0);
            assert_ne!(raw.fn_body_native_or_exception_table, 0);

            let args = vec![chars.clone().into(), 1.into(), 2.into()];
            let java_result = method.run(args.clone().into_boxed_slice()).await?;
            let java_result = Box::<dyn jvm::ClassInstance>::from(java_result);
            assert_eq!(JavaLangString::to_rust_string(&jvm, &java_result).await.unwrap(), "화장");

            let codec = JavaValueCodec::new(&core);
            let words = encode_method_arguments(&codec, &args);
            let ptr_args = Allocator::alloc(&mut core, words.len() as u32 * 4)?;
            for (index, word) in words.iter().enumerate() {
                write_generic(&mut core, ptr_args + index as u32 * 4, *word)?;
            }
            // KTF AOT callers can use the native argument-buffer ABI even when
            // the host implementation's Java prototype is not marked native.
            let result: u32 = core.run_function(raw.fn_body_native_or_exception_table, &[0, ptr_args]).await?;
            let result: Box<dyn jvm::ClassInstance> = Box::new(JavaClassInstance::from_raw(result, &core));
            assert_eq!(JavaLangString::to_rust_string(&jvm, &result).await.unwrap(), "화장");

            write_generic(&mut core, ptr_args + 4, (-1i32) as u32)?;
            let result = core.run_function::<u32>(raw.fn_body_native_or_exception_table, &[0, ptr_args]).await;
            assert!(matches!(result, Err(WieError::JavaException(_))));
            Allocator::free(&mut core, ptr_args, words.len() as u32 * 4)?;

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_native_method_entry_points() -> Result<()> {
        struct ReturnWords([u32; 2]);

        impl wie_core_arm::RunFunctionResult<ReturnWords> for ReturnWords {
            fn get(core: &ArmCore) -> Self {
                Self([core.read_param(0).unwrap(), core.read_param(1).unwrap()])
            }
        }

        let clock = TestClock::new();
        clock.set(0x12345678_9abcdef0);
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let runtime = jvm.new_class("java/lang/Runtime", "()V", ()).await.unwrap();
            let mut source = jvm.instantiate_array("I", 4).await.unwrap();
            let mut destination = jvm.instantiate_array("I", 4).await.unwrap();
            jvm.store_array(&mut source, 0, vec![11i32, 22, 33, 44]).await.unwrap();

            for (class_name, name, descriptor, is_static, args, expected) in [
                (
                    "java/lang/Runtime",
                    "totalMemory",
                    "()J",
                    false,
                    vec![JavaValue::from(runtime)],
                    vec![0x100000, 0],
                ),
                ("java/lang/System", "currentTimeMillis", "()J", true, vec![], vec![0x9abcdef0, 0x12345678]),
                (
                    "java/lang/System",
                    "arraycopy",
                    "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                    true,
                    vec![source.into(), 1.into(), destination.clone().into(), 0.into(), 2.into()],
                    vec![0],
                ),
            ] {
                let class = jvm.resolve_class(class_name).await.unwrap();
                let class = class.definition.as_any().downcast_ref::<JavaClassDefinition>().unwrap();
                let method = class.method(name, descriptor, is_static)?.unwrap();
                let raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
                assert!(MethodAccessFlags::from_bits_truncate(raw.access_flags).contains(MethodAccessFlags::NATIVE));
                assert_ne!(raw.fn_body, 0);
                assert_ne!(raw.fn_body_native_or_exception_table, 0);
                assert_ne!(raw.fn_body, raw.fn_body_native_or_exception_table);

                for native_entry in [false, true] {
                    jvm.store_array(&mut destination, 0, vec![0i32; 4]).await.unwrap();
                    let codec = JavaValueCodec::new(&core);
                    let actual = if native_entry {
                        let result = method.run(args.clone().into_boxed_slice()).await?;
                        encode_method_arguments(&codec, &[result])
                    } else {
                        let mut params = vec![0];
                        params.extend(encode_method_arguments(&codec, &args));
                        let result = core.run_function::<ReturnWords>(raw.fn_body, &params).await?;
                        result.0[..expected.len()].to_vec()
                    };
                    assert_eq!(actual, expected, "{name}, native_entry={native_entry}");
                    if name == "arraycopy" {
                        assert_eq!(jvm.load_array::<i32>(&destination, 0, 4).await.unwrap(), vec![22, 33, 0, 0]);
                    }
                }
            }

            done_clone.store(true, Ordering::Relaxed);
            clock.advance(16);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_exception_class_matches_raw_class_and_vtable() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;

            let exception = jvm.new_class("java/lang/NullPointerException", "()V", ()).await.unwrap();
            let null_pointer_class = jvm
                .resolve_class("java/lang/NullPointerException")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<JavaClassDefinition>()
                .unwrap()
                .clone();
            let runtime_exception_class = jvm
                .resolve_class("java/lang/RuntimeException")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<JavaClassDefinition>()
                .unwrap()
                .clone();
            let illegal_argument_class = jvm
                .resolve_class("java/lang/IllegalArgumentException")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<JavaClassDefinition>()
                .unwrap()
                .clone();

            assert!(JavaMethod::exception_class_matches(&core, &jvm, &*exception, 0)?);
            assert!(JavaMethod::exception_class_matches(&core, &jvm, &*exception, null_pointer_class.ptr_raw)?);
            assert!(JavaMethod::exception_class_matches(
                &core,
                &jvm,
                &*exception,
                null_pointer_class.ptr_vtable()?
            )?);
            assert!(JavaMethod::exception_class_matches(
                &core,
                &jvm,
                &*exception,
                runtime_exception_class.ptr_vtable()?
            )?);
            assert!(!JavaMethod::exception_class_matches(
                &core,
                &jvm,
                &*exception,
                illegal_argument_class.ptr_vtable()?
            )?);

            let ptr_exception = KtfJvmSupport::class_instance_raw(&exception);
            let result = JavaMethod::handle_exception(&mut core, &jvm, exception).await;
            assert!(matches!(result, Err(WieError::JavaException(ptr)) if ptr == ptr_exception));

            done_clone.store(true, Ordering::Relaxed);

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

    #[test]
    fn test_long_array_store_load() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, core) = init_jvm(&mut system_clone).await?;

            let values = vec![i64::MIN, -1, 0x12345678_9abcdef0, i64::MAX];

            let mut array = jvm.instantiate_array("J", 4).await.unwrap();
            jvm.store_array(&mut array, 0, values.clone()).await.unwrap();
            let loaded: Vec<i64> = jvm.load_array(&array, 0, 4).await.unwrap();

            assert_eq!(loaded, values);

            // guard against store/load flipping words symmetrically: check raw guest memory layout
            let array_instance = JavaArrayClassInstance::from_raw(KtfJvmSupport::class_instance_raw(&array), &core);
            let mut raw = [0u8; 8];
            array_instance.load_raw(16, &mut raw)?;
            assert_eq!(raw, 0x12345678_9abcdef0u64.to_le_bytes());

            done_clone.store(true, Ordering::Relaxed);

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

    #[test]
    fn test_double_array_store_load() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, _core) = init_jvm(&mut system_clone).await?;

            let values = vec![f64::MIN_POSITIVE, -1.5, f64::MAX];

            let mut array = jvm.instantiate_array("D", 3).await.unwrap();
            jvm.store_array(&mut array, 0, values.clone()).await.unwrap();
            let loaded: Vec<f64> = jvm.load_array(&array, 0, 3).await.unwrap();

            let to_bits = |x: &Vec<f64>| x.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
            assert_eq!(to_bits(&loaded), to_bits(&values));

            done_clone.store(true, Ordering::Relaxed);

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
