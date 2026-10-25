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

use jvm::{ClassDefinition, ClassInstance, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use rustjava_runtime::classes::java::util::{Enumeration, jar::JarEntry};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_midp::classes::javax::microedition::midlet::MIDlet;
use wie_util::{Result, WieError, read_generic, read_null_terminated_table, write_generic};

use wipi_types::ktf::InitParam2;

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
pub struct KtfJvmSupportContext {
    ptr_vtables_base: u32,
    ptr_current_jvm_thread_context: u32,
    pub ptr_visit_gc_root: u32,
}

pub const SUPPORT_CONTEXT_BASE: u32 = 0x7fff0000;

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
        mem::{offset_of, size_of},
        sync::atomic::{AtomicBool, Ordering},
    };

    use bytemuck::Zeroable;
    use jvm::{ClassInstance, ClassInstanceRef, JavaValue, Jvm, runtime::JavaLangString};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::Class;
    use wipi_types::ktf::{
        ExeInterfaceFunctions,
        java::{
            JavaClass as RawJavaClass, JavaClassDescriptor as RawJavaClassDescriptor, JavaClassInstance as RawJavaClassInstance,
            JavaExceptionHandler as RawJavaExceptionHandler, JavaFieldDefinition as RawJavaField, JavaMethodDefinition as RawJavaMethod,
            JavaMethodExceptionTableEntry as RawJavaMethodExceptionTableEntry, WIPIJBInterface,
        },
    };

    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_jvm_support::native::encode_method_arguments;
    use wie_midp::classes::javax::microedition::{lcdui::Display as MidpDisplay, midlet::MIDlet};
    use wie_util::{ByteWrite, Result, WieError, read_generic, write_generic, write_null_terminated_string_bytes, write_null_terminated_table};

    use crate::runtime::java::{JavaSvcFunctions, handle_java_svc, interface::get_wipi_jb_interface};

    use super::{
        ClassLoaderContext, JavaArrayClassInstance, JavaClassDefinition, JavaClassInstance, JavaMethod, JavaMethodResult, KtfClassLoader,
        KtfJvmSupport, KtfJvmThreadContext, name::JavaFullName, value::JavaValueCodec,
    };

    use test_utils::{TestClock, TestPlatform};

    async fn init_jvm(system: &mut System) -> Result<(Jvm, ArmCore)> {
        let mut core = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
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
    fn test_native_gc_visitor_keeps_guest_references() -> Result<()> {
        use bytemuck::{bytes_of, cast_slice};
        use wipi_types::ktf::{ExeInterface, WipiExe};

        use crate::runtime::{SVC_CATEGORY_JAVA_INTERFACE, init::load_native, svc_ids::JavaSvcId};

        async fn collect_during_init(core: &mut ArmCore, jvm: &mut Jvm) -> Result<u32> {
            let context: super::KtfJvmSupportContext = read_generic(core, super::SUPPORT_CONTEXT_BASE)?;
            assert_eq!(
                context.ptr_visit_gc_root, 0,
                "the visitor must not run against partially initialized tables"
            );
            jvm.collect_garbage().unwrap();
            let string: u32 = read_generic(core, 0x100104)?;
            assert!(Allocator::is_allocated(core, string, size_of::<RawJavaClassInstance>() as u32)?);
            Ok(0)
        }

        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let completed = done.clone();
        let mut task_system = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut task_system).await?;
            core.register_svc_handler(5, collect_during_init, &jvm)?;
            let gc = core.make_svc_stub(5, 0u32)?;
            let register_string = core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::RegisterJavaString)?;
            let mut image = vec![0; 0x1c0];
            // Thumb entry returns WipiExe; init stores the fifth argument's root callback.
            image[..8].copy_from_slice(&[0x00, 0x48, 0x70, 0x47, 0x80, 0x00, 0x10, 0x00]);
            image[0x10..0x24].copy_from_slice(&[
                0x00, 0x98, 0xc0, 0x69, 0x02, 0x49, 0x08, 0x60, 0x48, 0x60, 0x88, 0x60, 0x00, 0x20, 0x70, 0x47, 0x10, 0x01, 0x10, 0x00,
            ]);
            // ARM visitor reads three guest slots and calls the supplied callback with each value.
            let visitor = [
                0xe92d4070u32,
                0xe59f401c,
                0xe59f501c,
                0xe3a06003,
                0xe4940004,
                0xe4953004,
                0xe12fff33,
                0xe2566001,
                0x1afffffa,
                0xe8bd8070,
                0x00100100,
                0x00100110,
            ];
            image[0x40..0x70].copy_from_slice(cast_slice(&visitor));
            let mut executable = WipiExe::zeroed();
            executable.ptr_exe_interface = 0x1000b0;
            executable.fn_init = 0x100120;
            executable.fn_visit_gc_roots = 0x100040;
            image[0x80..0x80 + size_of::<WipiExe>()].copy_from_slice(bytes_of(&executable));
            let mut interface = ExeInterface::zeroed();
            interface.ptr_functions = 0x1000e0;
            image[0xb0..0xb0 + size_of::<ExeInterface>()].copy_from_slice(bytes_of(&interface));
            let mut functions = ExeInterfaceFunctions::zeroed();
            functions.fn_init = 0x100011;
            image[0xe0..0xe0 + size_of::<ExeInterfaceFunctions>()].copy_from_slice(bytes_of(&functions));
            // Register a native literal, store it in the guest pool, then collect during initialization.
            let init = [
                0xe92d4010u32,
                0xe59f0020,
                0xe3a0100e,
                0xe59f301c,
                0xe12fff33,
                0xe59f1018,
                0xe5810000,
                0xe59f3014,
                0xe12fff33,
                0xe3a00000,
                0xe8bd8010,
                0x00100180,
                register_string,
                0x00100104,
                gc,
            ];
            image[0x120..0x15c].copy_from_slice(cast_slice(&init));
            let literal = "native literal".encode_utf16().collect::<Vec<_>>();
            image[0x180..0x19c].copy_from_slice(cast_slice(&literal));
            let thread_context = KtfJvmSupport::current_thread_context(&core)?;
            {
                let _frame = jvm.push_native_frame(&[]);
                load_native(&mut core, &mut task_system, &jvm, "client.bin0", &image, 0, thread_context).await?;
            }

            let callback: u32 = read_generic(&core, 0x100110)?;
            assert_ne!(callback, 0, "native initialization must receive a root callback");
            let string: u32 = read_generic(&core, 0x100104)?;
            let (array, child, garbage) = {
                let _frame = jvm.push_native_frame(&[]);
                let mut array = jvm.instantiate_array("Ljava/lang/Object;", 2).await.unwrap();
                let child = jvm.new_class("java/lang/Object", "()V", ()).await.unwrap();
                let references = [array.clone(), child.clone()];
                jvm.store_array(&mut array, 0, references).await.unwrap();
                let garbage = jvm.new_class("java/lang/Object", "()V", ()).await.unwrap();
                (
                    KtfJvmSupport::class_instance_raw(&array),
                    KtfJvmSupport::class_instance_raw(&child),
                    KtfJvmSupport::class_instance_raw(&garbage),
                )
            };
            write_generic(&mut core, 0x100100, array)?;
            let before = core.save_context();
            jvm.collect_garbage().unwrap();
            let after = core.save_context();
            assert_eq!((after.pc, after.lr, after.sp, after.cpsr), (before.pc, before.lr, before.sp, before.cpsr));
            let instance_size = size_of::<RawJavaClassInstance>() as u32;
            for root in [array, child, string] {
                assert!(Allocator::is_allocated(&core, root, instance_size)?);
            }
            assert!(!Allocator::is_allocated(&core, garbage, instance_size)?);

            // An unexpected SVC must abort collection before any object is freed.
            write_generic(&mut core, 0x100114, register_string)?;
            assert!(jvm.collect_garbage().is_err());
            for root in [array, child, string] {
                assert!(Allocator::is_allocated(&core, root, instance_size)?);
            }
            write_generic(&mut core, 0x100114, callback)?;
            write_generic(&mut core, 0x100100, [0u32; 3])?;
            jvm.collect_garbage().unwrap();
            for root in [array, child, string] {
                assert!(!Allocator::is_allocated(&core, root, instance_size)?);
            }
            completed.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_class_loader_before_native_initialization() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let completed = done.clone();
        let mut task_system = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut task_system).await?;
            let java_functions = JavaSvcFunctions::default();
            core.register_svc_handler(5, handle_java_svc, &java_functions)?;
            let loader_class = JavaClassDefinition::new(
                &mut core.clone(),
                &jvm,
                KtfClassLoader::as_proto(),
                Box::new(ClassLoaderContext {
                    core: core.clone(),
                    system: task_system.clone(),
                }),
                java_functions,
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
            write_generic(&mut core, ptr_functions, ExeInterfaceFunctions::zeroed())?;
            jvm.put_field(&mut loader, "net/wie/KtfClassLoader", "nativeFunctions", "I", ptr_functions as i32)
                .await
                .unwrap();
            let name = JavaLangString::from_rust_string(&jvm, "test/Missing").await.unwrap();
            let missing: ClassInstanceRef<Class> = jvm
                .invoke_virtual(
                    &loader,
                    "net/wie/KtfClassLoader",
                    "findClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    (name,),
                )
                .await
                .unwrap();
            assert!(missing.is_null());
            completed.store(true, Ordering::Relaxed);
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
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;

            let midlet: ClassInstanceRef<MIDlet> = jvm.new_class("net/wie/WIPIMIDlet", "()V", ()).await.unwrap().into();
            let display: ClassInstanceRef<MidpDisplay> = MIDlet::display(&jvm, &midlet).await.unwrap();
            let paint_disabled: bool = jvm
                .get_field(&display, "javax/microedition/lcdui/Display", "paintDisabled", "Z")
                .await
                .unwrap();
            assert!(!paint_disabled);

            KtfJvmSupport::disable_midp_paint(&jvm).await.unwrap();

            let paint_disabled: bool = jvm
                .get_field(&display, "javax/microedition/lcdui/Display", "paintDisabled", "Z")
                .await
                .unwrap();
            assert!(paint_disabled);

            let class = jvm.resolve_class("javax/microedition/lcdui/Display").await.unwrap();
            let class = class.definition.as_any().downcast_ref::<JavaClassDefinition>().unwrap();
            let field = class.field("paintDisabled", "Z", false)?.unwrap();
            assert!(class.field("paintDisabled", "Z", true)?.is_none());
            assert!(class.field("paintDisabled", "I", false)?.is_none());
            assert!(class.field("missing", "Z", false)?.is_none());
            let mut raw: RawJavaField = read_generic(&core, field.ptr_raw)?;
            let flags = raw.access_flags;
            raw.access_flags |= u32::from(FieldAccessFlags::STATIC.bits());
            write_generic(&mut core, field.ptr_raw, raw)?;
            assert!(class.field("paintDisabled", "Z", false)?.is_none());
            assert_eq!(class.field("paintDisabled", "Z", true)?.unwrap().ptr_raw, field.ptr_raw);
            raw.access_flags = flags;
            write_generic(&mut core, field.ptr_raw, raw)?;

            let string1 = JavaLangString::from_rust_string(&jvm, "test1").await.unwrap();
            let string2 = JavaLangString::from_rust_string(&jvm, "test2").await.unwrap();
            assert!(!string1.class_definition().fields().is_empty());

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

            let reference_array = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
            assert!(reference_array.class_definition().fields().is_empty());

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
    fn test_native_virtual_overrides() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let object = jvm.resolve_class("java/lang/Object").await.unwrap();
            let ptr_object = KtfJvmSupport::class_definition_raw(&*object.definition)?;
            let name = JavaFullName::new(0, "value", "()I").as_bytes();
            let ptr_name = Allocator::alloc(&mut core, name.len() as u32)?;
            core.write_bytes(ptr_name, &name)?;

            let mut classes: Vec<u32> = Vec::new();
            let mut methods = Vec::new();
            for (name, parent, flags, slots, value) in [
                ("test/Base", None, Some(MethodAccessFlags::PRIVATE), &[0][..], 1),
                (
                    "test/Abstract",
                    Some(0),
                    Some(MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                    &[0, 1][..],
                    0,
                ),
                ("other/Concrete", Some(1), Some(MethodAccessFlags::PUBLIC), &[0, 1, 2][..], 42),
                ("other/Inherited", Some(2), None, &[0, 1, 2][..], 0),
                ("other/Sibling", Some(1), Some(MethodAccessFlags::PUBLIC), &[0, 1, 3][..], 84),
                ("test/PackageBase", Some(0), Some(MethodAccessFlags::empty()), &[0, 4][..], 10),
                ("other/Unrelated", Some(5), Some(MethodAccessFlags::PUBLIC), &[0, 4, 5][..], 20),
                ("test/Bridge", Some(5), Some(MethodAccessFlags::PROTECTED), &[0, 4, 6][..], 30),
                ("other/BridgeLeaf", Some(7), Some(MethodAccessFlags::PUBLIC), &[0, 4, 6, 7][..], 40),
            ] {
                let ptr_class = Allocator::alloc(&mut core, size_of::<RawJavaClass>() as u32)?;
                if let Some(flags) = flags {
                    let ptr_method = Allocator::alloc(&mut core, size_of::<RawJavaMethod>() as u32)?;
                    let mut method = RawJavaMethod::zeroed();
                    if !flags.contains(MethodAccessFlags::ABSTRACT) {
                        method.fn_body = Allocator::alloc(&mut core, 4)? | 1;
                        core.write_bytes(method.fn_body & !1, &[value, 0x20, 0x70, 0x47])?; // movs r0, #value; bx lr
                    }
                    method.ptr_class = ptr_class;
                    method.ptr_name = ptr_name;
                    method.access_flags = flags.bits();
                    method.index_in_vtable = (slots.len() - 1) as u16;
                    write_generic(&mut core, ptr_method, method)?;
                    methods.push(ptr_method);
                }

                // Native descriptors include inherited declarations, and their vtables
                // have an explicit count rather than a terminating null pointer.
                let ptr_methods = Allocator::alloc(&mut core, (slots.len() as u32 + 1) * 4)?;
                let pointers = slots.iter().map(|&index| methods[index]).collect::<Vec<_>>();
                write_null_terminated_table(&mut core, ptr_methods, &pointers)?;
                let ptr_vtable = Allocator::alloc(&mut core, slots.len() as u32 * 4)?;
                for (index, pointer) in pointers.iter().enumerate() {
                    write_generic(&mut core, ptr_vtable + index as u32 * 4, *pointer)?;
                }
                let ptr_class_name = Allocator::alloc(&mut core, name.len() as u32 + 1)?;
                write_null_terminated_string_bytes(&mut core, ptr_class_name, name.as_bytes())?;
                let ptr_descriptor = Allocator::alloc(&mut core, size_of::<RawJavaClassDescriptor>() as u32)?;
                let mut descriptor = RawJavaClassDescriptor::zeroed();
                descriptor.ptr_name = ptr_class_name;
                descriptor.ptr_parent_class = parent.map(|index: usize| classes[index]).unwrap_or(ptr_object);
                descriptor.ptr_methods = ptr_methods;
                descriptor.method_count = slots.len() as u16;
                descriptor.access_flag = ClassAccessFlags::PUBLIC.bits();
                if flags.is_some_and(|flags| flags.contains(MethodAccessFlags::ABSTRACT)) {
                    descriptor.access_flag |= ClassAccessFlags::ABSTRACT.bits();
                }
                write_generic(&mut core, ptr_descriptor, descriptor)?;
                write_generic(
                    &mut core,
                    ptr_class,
                    RawJavaClass {
                        ptr_next: ptr_class + 4,
                        unk1: 0,
                        ptr_descriptor,
                        ptr_vtable,
                        vtable_count: slots.len() as u16,
                        unk_flag: 0x8080,
                    },
                )?;
                let class = JavaClassDefinition::from_raw(ptr_class, &core);
                jvm.register_class(Box::new(class), None).await.unwrap();
                classes.push(ptr_class);
            }

            for (index, declaration, expected) in [(2, 1, 42), (3, 1, 42), (4, 1, 84), (6, 4, 10), (7, 4, 30), (8, 4, 40)] {
                let class = JavaClassDefinition::from_raw(classes[index], &core);
                let instance = jvm.instantiate_class(&class.name()?).await.unwrap();
                let base_method: RawJavaMethod = read_generic(&core, methods[declaration])?;
                let vtable = class.ptr_vtable()?;
                let ptr_target: u32 = read_generic(&core, vtable + u32::from(base_method.index_in_vtable) * 4)?;
                let target: RawJavaMethod = read_generic(&core, ptr_target)?;
                assert_eq!(target.access_flags & MethodAccessFlags::ABSTRACT.bits(), 0);
                let value: u32 = core
                    .run_function(target.fn_body, &[0, KtfJvmSupport::class_instance_raw(&instance)])
                    .await?;
                assert_eq!(value, expected);
                assert_eq!(read_generic::<u32, _>(&core, vtable)?, methods[0]);
                assert_eq!(base_method.index_in_vtable, 1);
            }
            let abstract_class = JavaClassDefinition::from_raw(classes[1], &core);
            assert_eq!(read_generic::<u32, _>(&core, abstract_class.ptr_vtable()? + 4)?, methods[1]);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
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
            jvm.collect_garbage().unwrap();
            assert_eq!(JavaLangString::to_rust_string(&jvm, &result).await.unwrap(), "화장");

            let caller = core.save_context();
            let result = method.run(vec![chars.into(), (-1).into(), 2.into()].into_boxed_slice()).await;
            assert!(matches!(result, Err(WieError::JavaException(_))));
            let restored = core.save_context();
            assert_eq!(
                (restored.r0, restored.r1, restored.sp, restored.lr, restored.pc),
                (caller.r0, caller.r1, caller.sp, caller.lr, caller.pc)
            );

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

                if name == "arraycopy" {
                    core.load(&[0x55, 0x24, 0x00, 0xe8], 0x10000, 0x1000)?; // movs r4, #0x55; undefined
                    let mut faulting_method = raw;
                    faulting_method.fn_body_native_or_exception_table = 0x10001;
                    write_generic(&mut core, method.ptr_raw, faulting_method)?;
                    let result = method.run(args.into_boxed_slice()).await;
                    let Err(WieError::FatalError(message)) = result else {
                        panic!("Expected an ARM instruction fault");
                    };
                    assert!(message.contains("Undefined instruction at 0x10002"), "{message}");
                    assert!(message.contains("R4: 0x55"), "{message}");
                    assert_eq!(core.save_context().r4, 0x55);
                    write_generic(&mut core, method.ptr_raw, raw)?;
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
        async fn throw_from_nested_call(core: &mut ArmCore, (jvm, ptr_exception): &mut (Jvm, u32)) -> Result<JavaMethodResult> {
            let mut context = core.save_context();
            context.r4 = 0x99;
            core.restore_context(&context);
            let exception = Box::new(JavaClassInstance::from_raw(*ptr_exception, core));
            JavaMethod::handle_exception(core, jvm, exception).await
        }

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

            core.map(0x10000, 0x1000)?;
            write_generic(
                &mut core,
                0x10000,
                RawJavaMethod {
                    fn_body_native_or_exception_table: 0x10100,
                    exception_table_count: 1,
                    ..RawJavaMethod::zeroed()
                },
            )?;
            write_generic(&mut core, 0x10100, 0x10120u32)?;
            write_generic(
                &mut core,
                0x10120,
                RawJavaMethodExceptionTableEntry {
                    from_pc: 1,
                    to_pc: 2,
                    target: 3,
                    ptr_class: runtime_exception_class.ptr_raw,
                },
            )?;
            write_generic(
                &mut core,
                0x10200,
                RawJavaExceptionHandler {
                    ptr_method: 0x10000,
                    current_pc: 1,
                    ptr_exception_class_instance: 0xdeadbeef,
                    ptr_functions: 0x10300,
                    ..RawJavaExceptionHandler::zeroed()
                },
            )?;
            write_generic(&mut core, 0x10304, 0x10401u32)?;
            let thread_context = KtfJvmSupport::current_thread_context(&core)?;
            write_generic(
                &mut core,
                thread_context + offset_of!(KtfJvmThreadContext, current_java_exception_handler) as u32,
                0x10200u32,
            )?;
            let exception = Box::new(JavaClassInstance::from_raw(ptr_exception, &core));
            let result = JavaMethod::handle_exception(&mut core, &jvm, exception).await;
            assert!(matches!(
                result,
                Err(WieError::JavaExceptionUnwind {
                    context_base: 0x10218,
                    target: 3,
                    next_pc: 0x10401
                })
            ));
            let caught: u32 = read_generic(&core, 0x10210)?;
            assert_eq!(caught, ptr_exception);
            write_generic(&mut core, 0x10600, RawJavaMethod::zeroed())?;
            // A method without catches links only its header, with no saved context.
            write_generic(&mut core, 0x1fff4, [0x10600u32, 0, 0x10200])?;
            write_generic(
                &mut core,
                thread_context + offset_of!(KtfJvmThreadContext, current_java_exception_handler) as u32,
                0x1fff4u32,
            )?;
            let exception = Box::new(JavaClassInstance::from_raw(ptr_exception, &core));
            let result = JavaMethod::handle_exception(&mut core, &jvm, exception).await;
            assert!(matches!(result, Err(WieError::JavaException(ptr)) if ptr == ptr_exception));
            write_generic(
                &mut core,
                thread_context + offset_of!(KtfJvmThreadContext, current_java_exception_handler) as u32,
                0x10200u32,
            )?;
            let caught = Box::new(JavaClassInstance::from_raw(caught, &core)) as Box<dyn ClassInstance>;
            let _: () = jvm
                .invoke_virtual(&caught, "java/lang/Throwable", "printStackTrace", "()V", ())
                .await
                .unwrap();

            core.register_svc_handler(5, throw_from_nested_call, &(jvm, ptr_exception))?;
            let throwing = core.make_svc_stub(5, 0u32)?;
            let ptr_interface = get_wipi_jb_interface(&mut core)?;
            let interface: WIPIJBInterface = read_generic(&core, ptr_interface)?;
            let mut caller = core.save_context();
            caller.r4 = 0x55;
            core.restore_context(&caller);
            write_generic(&mut core, 0x10218 + 9 * 4, caller.sp)?;
            write_generic(&mut core, 0x10400, [0x4620u16, 0x4770])?; // mov r0, r4; bx lr
            for (entry, args) in [
                (interface.fn_java_jump_1, vec![0, throwing]),
                (interface.fn_java_jump_2, vec![0, 0, throwing]),
                (interface.fn_java_jump_3, vec![0, 0, 0, throwing]),
                (interface.fn_call_native, vec![throwing, 0x10500]),
            ] {
                let result: u32 = core.run_function(entry, &args).await?;
                assert_eq!(result, caller.r4);
            }

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
