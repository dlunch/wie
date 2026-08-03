mod array_class_definition;
mod array_class_instance;
mod class_definition;
mod class_instance;
mod field;
mod jvm_implementation;
mod method;
mod value;
mod vtable;

pub use jvm_implementation::LgtJvmImplementation;

use self::{
    array_class_definition::JavaArrayClassDefinition,
    array_class_instance::JavaArrayClassInstance,
    class_definition::{ClassRegistry, JavaClassDefinition},
    class_instance::JavaClassInstance,
    field::JavaField,
    method::JavaMethod,
};

pub(super) type LgtJvmWord = u32;
pub(super) type Result<T> = wie_util::Result<T>;

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use core::{
        mem::offset_of,
        sync::atomic::{AtomicBool, Ordering},
    };

    use java_class_proto::{JavaClassProto, JavaMethodProto};
    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstance, ClassInstanceRef, Jvm, Method, Result as JvmResult, runtime::JavaLangString};
    use wipi_types::lgt::java::LgtJavaClassInstance as RawJavaClassInstance;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_jvm_support::{JvmImplementation, JvmSupport};
    use wie_util::{Result, read_generic};

    use super::{JavaArrayClassInstance, JavaClassInstance, LgtJvmImplementation};

    struct Base;
    struct Child;

    async fn base_value(_jvm: &Jvm, _context: &mut (), _this: ClassInstanceRef<Base>) -> JvmResult<i32> {
        Ok(1)
    }

    async fn child_value(_jvm: &Jvm, _context: &mut (), _this: ClassInstanceRef<Child>) -> JvmResult<i32> {
        Ok(2)
    }

    async fn init_jvm(system: &System) -> Result<(Jvm, ArmCore, LgtJvmImplementation)> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x100)?;
        context.sp = stack + 0x100;
        core.restore_context(&context);

        let protos = [wie_midp::get_protos().into(), wie_wipi_java::get_protos().into()];
        let implementation = LgtJvmImplementation::new(&mut core)?;
        let jvm = JvmSupport::new_jvm(system, None, Box::new(protos), &[], implementation.clone()).await?;

        Ok((jvm, core, implementation))
    }

    #[test]
    fn test_native_jvm_runtime() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, core, implementation) = init_jvm(&system_clone).await?;

            let first = JavaLangString::from_rust_string(&jvm, "lgt").await.unwrap();
            let second = JavaLangString::from_rust_string(&jvm, "-native").await.unwrap();
            let combined = jvm
                .invoke_virtual(&first, "concat", "(Ljava/lang/String;)Ljava/lang/String;", [second.into()])
                .await
                .unwrap();
            assert_eq!(JavaLangString::to_rust_string(&jvm, &combined).await.unwrap(), "lgt-native");
            let invalid_char: jvm::Result<u16> = jvm.invoke_virtual(&first, "charAt", "(I)C", (i32::MAX,)).await;
            assert!(invalid_char.is_err());

            let date: Box<dyn ClassInstance> = jvm.new_class("java/util/Date", "(J)V", (0x12345678_abcdef01i64,)).await.unwrap();
            let time: i64 = jvm.invoke_virtual(&date, "getTime", "()J", ()).await.unwrap();
            assert_eq!(time, 0x12345678_abcdef01);

            let native_date = date.as_any().downcast_ref::<JavaClassInstance>().unwrap();
            let ptr_dispatch_table: u32 = read_generic(&core, native_date.ptr_raw + offset_of!(RawJavaClassInstance, ptr_dispatch_table) as u32)?;
            let ptr_class: u32 = read_generic(&core, ptr_dispatch_table)?;
            let ptr_fields: u32 = read_generic(&core, native_date.ptr_raw + offset_of!(RawJavaClassInstance, ptr_fields) as u32)?;
            assert_eq!(ptr_class, native_date.class().ptr_raw());
            assert_ne!(ptr_fields, 0);

            let mut shorts = jvm.instantiate_array("S", 10).await.unwrap();
            jvm.store_array(&mut shorts, 0, (0..10i16).collect::<Vec<_>>()).await.unwrap();
            assert_eq!(jvm.load_array::<i16>(&shorts, 5, 4).await.unwrap(), vec![5, 6, 7, 8]);

            let long_values = vec![i64::MIN, -1, 0x12345678_9abcdef0, i64::MAX];
            let mut longs = jvm.instantiate_array("J", long_values.len()).await.unwrap();
            jvm.store_array(&mut longs, 0, long_values.clone()).await.unwrap();
            assert_eq!(jvm.load_array::<i64>(&longs, 0, long_values.len()).await.unwrap(), long_values);
            let long_array = longs.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap();
            let mut raw_long = [0; 8];
            long_array.load_raw(16, &mut raw_long)?;
            assert_eq!(raw_long, 0x12345678_9abcdef0u64.to_le_bytes());

            let object = JavaLangString::from_rust_string(&jvm, "reference").await.unwrap();
            let expected_identity = object.identity();
            let mut references = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
            jvm.store_array(&mut references, 0, vec![object]).await.unwrap();
            let loaded: Vec<ClassInstanceRef<String>> = jvm.load_array(&references, 0, 1).await.unwrap();
            assert_eq!(loaded[0].identity(), expected_identity);

            let base_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/Base",
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec![],
                        methods: vec![JavaMethodProto::new("value", "()I", base_value, Default::default())],
                        fields: vec![],
                        access_flags: Default::default(),
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let base_definition = base_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap().clone();
            jvm.register_class(base_class, None).await.unwrap();

            let child_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/Child",
                        parent_class: Some("net/wie/test/Base"),
                        interfaces: vec![],
                        methods: vec![JavaMethodProto::new("value", "()I", child_value, Default::default())],
                        fields: vec![],
                        access_flags: Default::default(),
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let child_definition = child_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap().clone();
            jvm.register_class(child_class, None).await.unwrap();

            let base_slot = base_definition
                .virtual_methods()
                .iter()
                .position(|method| method.name() == "value" && method.descriptor() == "()I")
                .unwrap();
            let child_slot = child_definition
                .virtual_methods()
                .iter()
                .position(|method| method.name() == "value" && method.descriptor() == "()I")
                .unwrap();
            assert_eq!(child_slot, base_slot);
            let child_target = child_definition.virtual_methods()[child_slot].target()?;
            let dispatch_target: u32 = read_generic(&core, child_definition.ptr_dispatch_table() + ((child_slot + 1) * 4) as u32)?;
            assert_eq!(dispatch_target, child_target);

            let child = jvm.instantiate_class("net/wie/test/Child").await.unwrap();
            let value: i32 = jvm.invoke_virtual(&child, "value", "()I", ()).await.unwrap();
            assert_eq!(value, 2);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }
}
