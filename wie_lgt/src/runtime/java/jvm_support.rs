mod array_class_definition;
mod array_class_instance;
mod class_definition;
mod class_instance;
mod field;
mod jvm_implementation;
mod method;
mod value;
mod vtable;

use alloc::{boxed::Box, format, string::String};

use jvm::{ClassDefinition, ClassInstance, JavaError, Jvm, Method};

use wie_backend::System;
use wie_core_arm::ArmCore;
use wie_jvm_support::{JvmImplementation, JvmSupport, native::NativeJavaValueCodec};
use wie_midp::get_protos as get_midp_protos;
use wie_util::{Result, WieError};
use wie_wipi_java::get_protos as get_wipi_java_protos;

use super::classes::net::wie::{CletWrapper, CletWrapperCard, CletWrapperContext, LgtClassLoader};

use jvm_implementation::LgtJvmImplementation;

use self::{
    array_class_definition::JavaArrayClassDefinition,
    array_class_instance::JavaArrayClassInstance,
    class_definition::JavaClassDefinition,
    class_instance::JavaClassInstance,
    field::{JavaField, JavaReferenceField, JavaStaticReferenceField},
    method::JavaMethod,
    value::JavaValueCodec,
    vtable::JavaVtableEntry,
};

type LgtJvmWord = u32;

pub struct LgtJvmSupport;

impl LgtJvmSupport {
    pub async fn init(core: &mut ArmCore, system: &System, jar_name: Option<&str>) -> Result<Jvm> {
        let protos = [get_midp_protos().into(), get_wipi_java_protos().into()];
        let implementation = LgtJvmImplementation::new(core)?;
        let jvm = JvmSupport::new_jvm(system, jar_name, Box::new(protos), &[], implementation.clone()).await?;

        let class = match implementation
            .define_class_rust(&jvm, LgtClassLoader::as_proto(), Box::new(core.clone()))
            .await
        {
            Ok(class) => class,
            Err(error) => return Err(JvmSupport::to_wie_err(&jvm, error).await),
        };
        if let Err(error) = jvm.register_class(class, None).await {
            return Err(JvmSupport::to_wie_err(&jvm, error).await);
        }

        let context = CletWrapperContext { core: core.clone() };
        for proto in [CletWrapper::as_proto(), CletWrapperCard::as_proto()] {
            let class = match implementation.define_class_rust(&jvm, proto, Box::new(context.clone()) as Box<_>).await {
                Ok(class) => class,
                Err(error) => return Err(JvmSupport::to_wie_err(&jvm, error).await),
            };
            jvm.register_class(class, None).await.unwrap();
        }

        Ok(jvm)
    }

    pub fn class_from_raw(core: &ArmCore, ptr_class: u32) -> JavaClassDefinition {
        JavaClassDefinition::from_raw(ptr_class, core)
    }

    pub fn class_instance_from_raw(core: &ArmCore, ptr_instance: u32) -> Box<dyn ClassInstance> {
        JavaValueCodec::new(core).object_from_raw(ptr_instance)
    }

    pub fn class_instance_raw(instance: &dyn ClassInstance) -> u32 {
        if let Some(instance) = instance.as_any().downcast_ref::<JavaClassInstance>() {
            instance.ptr_raw
        } else {
            instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap().class_instance.ptr_raw
        }
    }

    pub fn class_definition_raw(definition: &dyn ClassDefinition) -> u32 {
        if let Some(definition) = definition.as_any().downcast_ref::<JavaClassDefinition>() {
            definition.ptr_raw
        } else {
            definition.as_any().downcast_ref::<JavaArrayClassDefinition>().unwrap().class.ptr_raw
        }
    }

    pub fn field_word_index(jvm: &Jvm, class_name: &str, name: &str, descriptor: &str, is_static: bool) -> Result<u16> {
        let mut current_name = String::from(class_name);
        loop {
            let class = jvm
                .get_class(&current_name)
                .ok_or_else(|| WieError::FatalError(format!("Class not loaded while linking field: {current_name}")))?;
            if let Some(field) = class.definition.field(name, descriptor, is_static) {
                let field = field
                    .as_any()
                    .downcast_ref::<JavaField>()
                    .ok_or_else(|| WieError::FatalError(format!("Unsupported field implementation for {current_name}.{name}{descriptor}")))?;
                return u16::try_from(field.word_index()?)
                    .map_err(|_| WieError::FatalError(format!("Field word index does not fit LGT ABI for {current_name}.{name}{descriptor}")));
            }

            let Some(parent_name) = class.definition.super_class_name() else {
                return Err(WieError::FatalError(format!(
                    "Unable to resolve {} field {class_name}.{name}{descriptor}",
                    if is_static { "static" } else { "instance" }
                )));
            };
            current_name = parent_name;
        }
    }

    pub async fn virtual_method_index(jvm: &Jvm, class_name: &str, name: &str, descriptor: &str) -> Result<u16> {
        let class = jvm
            .get_class(class_name)
            .ok_or_else(|| WieError::FatalError(format!("Class not loaded while linking virtual method: {class_name}")))?;
        let definition = class
            .definition
            .as_any()
            .downcast_ref::<JavaClassDefinition>()
            .ok_or_else(|| WieError::FatalError(format!("Unsupported class implementation while linking virtual method: {class_name}")))?
            .clone();
        let mut methods = definition.vtable_entries(jvm).await?;
        if let Some(index) = methods.iter().position(|entry| {
            entry
                .method
                .as_ref()
                .is_some_and(|method| method.name() == name && method.descriptor() == descriptor)
        }) {
            return u16::try_from(index)
                .map_err(|_| WieError::FatalError(format!("Virtual method index does not fit LGT ABI for {class_name}.{name}{descriptor}")));
        }

        // TODO Remove this fallback once data/lgt_java_abi.toml covers every linked virtual method.
        let mut current_name = String::from(class_name);
        let method = loop {
            let class = jvm
                .get_class(&current_name)
                .ok_or_else(|| WieError::FatalError(format!("Class not loaded while linking virtual method: {current_name}")))?;
            if let Some(method) = class.definition.method(name, descriptor, false) {
                break method
                    .as_any()
                    .downcast_ref::<JavaMethod>()
                    .ok_or_else(|| WieError::FatalError(format!("Unsupported method implementation for {current_name}.{name}{descriptor}")))?
                    .clone();
            }

            let Some(parent_name) = class.definition.super_class_name() else {
                return Err(WieError::FatalError(format!(
                    "Unable to resolve virtual method {class_name}.{name}{descriptor}"
                )));
            };
            current_name = parent_name;
        };

        let index = methods.len();
        methods.push(JavaVtableEntry {
            target: method.target()?,
            method: Some(method),
        });
        definition.set_vtable_entries(&methods)?;

        u16::try_from(index)
            .map_err(|_| WieError::FatalError(format!("Virtual method index does not fit LGT ABI for {class_name}.{name}{descriptor}")))
    }

    pub async fn interface_dispatch_table(jvm: &mut Jvm, class_name: &str) -> Result<u32> {
        let class = jvm
            .resolve_class(class_name)
            .await
            .map_err(|JavaError::JavaException(instance)| WieError::JavaException(Self::class_instance_raw(&*instance)))?;
        let definition = class
            .definition
            .as_any()
            .downcast_ref::<JavaClassDefinition>()
            .ok_or_else(|| WieError::FatalError(format!("Unsupported interface class implementation: {class_name}")))?;

        definition.ptr_vtable()
    }

    pub fn non_virtual_method_target(jvm: &Jvm, class_name: &str, name: &str, descriptor: &str) -> Result<u32> {
        let mut current_name = String::from(class_name);
        loop {
            let class = jvm
                .get_class(&current_name)
                .ok_or_else(|| WieError::FatalError(format!("Class not loaded while linking direct method: {current_name}")))?;
            if let Some(method) = class
                .definition
                .method(name, descriptor, true)
                .or_else(|| class.definition.method(name, descriptor, false))
            {
                let method = method
                    .as_any()
                    .downcast_ref::<JavaMethod>()
                    .ok_or_else(|| WieError::FatalError(format!("Unsupported method implementation for {current_name}.{name}{descriptor}")))?;
                return method.target();
            }

            let Some(parent_name) = class.definition.super_class_name() else {
                return Err(WieError::FatalError(format!(
                    "Unable to resolve non-virtual method {class_name}.{name}{descriptor}"
                )));
            };
            current_name = parent_name;
        }
    }

    pub fn class_getter_targets(jvm: &Jvm, class_name: &str) -> Result<(u32, u32)> {
        let class = jvm
            .get_class(class_name)
            .ok_or_else(|| WieError::FatalError(format!("Class not loaded while linking class getters: {class_name}")))?;
        let definition = class
            .definition
            .as_any()
            .downcast_ref::<JavaClassDefinition>()
            .ok_or_else(|| WieError::FatalError(format!("Unsupported class implementation while linking class getters: {class_name}")))?;
        let descriptor = definition.descriptor()?;
        Ok((descriptor.fn_get_initialized_class, descriptor.fn_get_class))
    }

    pub async fn register_generated_class(
        core: &mut ArmCore,
        jvm: &Jvm,
        ptr_class: u32,
        generated_classes: u32,
        loader: Box<dyn ClassInstance>,
    ) -> Result<Box<dyn ClassInstance>> {
        let mut definition = JavaClassDefinition::from_raw(ptr_class, core);
        let class_name = ClassDefinition::name(&definition);
        if let Some(existing) = jvm.get_class(&class_name) {
            if existing
                .definition
                .as_any()
                .downcast_ref::<JavaClassDefinition>()
                .is_some_and(|definition| definition.ptr_raw == ptr_class)
                && definition.descriptor()?.link_state == 3
            {
                return Ok(existing.java_class());
            }
            return Err(wie_util::WieError::FatalError(format!(
                "Class {class_name} is already registered from a different definition"
            )));
        }

        let previous_link_state = definition.descriptor()?.link_state;
        definition.prepare_generated(core, jvm, generated_classes).await?;
        let registered_definition = definition.clone();
        let mut java_class = match jvm.register_class(Box::new(definition), Some(loader)).await {
            Ok(Some(java_class)) => java_class,
            Ok(None) => unreachable!(),
            Err(JavaError::JavaException(instance)) => {
                return Err(wie_util::WieError::JavaException(JavaValueCodec::new(core).object_to_raw(&*instance)));
            }
        };
        registered_definition.bind_class_object_storage(jvm, &mut java_class).await?;
        registered_definition.set_link_state(3)?;

        let fn_link_members = registered_definition.descriptor()?.fn_link_members;
        if fn_link_members != 0
            && let Err(error) = core.run_function::<()>(fn_link_members, &[Self::class_instance_raw(&*java_class)]).await
        {
            registered_definition.set_link_state(previous_link_state)?;
            return Err(error);
        }

        Ok(java_class)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, string::String as RustString, sync::Arc, vec, vec::Vec};
    use core::{
        mem::{offset_of, size_of},
        sync::atomic::{AtomicBool, Ordering},
    };

    use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
    use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use java_runtime::classes::java::lang::String;
    use jvm::{Array, ClassDefinition, ClassInstance, ClassInstanceRef, JavaValue, Jvm, Method, Result as JvmResult, runtime::JavaLangString};
    use wipi_types::lgt::java::{
        LGT_JAVA_CLASS_SUPER_CLASS_IS_NAME, LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor,
        LgtJavaClassField as RawJavaField, LgtJavaClassInstance as RawJavaClassInstance, LgtJavaClassMethod as RawJavaMethod,
        LgtJavaInterfaceReference as RawJavaInterfaceReference, LgtJavaInterfaceReferences as RawJavaInterfaceReferences,
    };

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_jvm_support::{JvmImplementation, JvmSupport};
    use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic, write_generic, write_null_terminated_string_bytes};

    use crate::runtime::java::abi::{CLASS_INITIALIZATION_STATE_FIELD, CLASS_NATIVE_NAME_FIELD, WORD_FIELD_DESCRIPTOR};

    use super::{JavaClassInstance, LgtClassLoader, LgtJvmImplementation, LgtJvmSupport, get_midp_protos, get_wipi_java_protos};

    struct Base;
    struct Child;
    struct DirectJlet;
    struct DeepJlet;

    async fn base_value(_jvm: &Jvm, _context: &mut (), _this: ClassInstanceRef<Base>) -> JvmResult<i32> {
        Ok(1)
    }

    async fn child_value(_jvm: &Jvm, _context: &mut (), _this: ClassInstanceRef<Child>) -> JvmResult<i32> {
        Ok(2)
    }

    async fn jlet_start(_jvm: &Jvm, _context: &mut (), _this: ClassInstanceRef<DirectJlet>, _args: ClassInstanceRef<Array<String>>) -> JvmResult<()> {
        Ok(())
    }

    async fn jlet_pause(_jvm: &Jvm, _context: &mut (), _this: ClassInstanceRef<DirectJlet>) -> JvmResult<()> {
        Ok(())
    }

    async fn jlet_resume(_jvm: &Jvm, _context: &mut (), _this: ClassInstanceRef<DirectJlet>) -> JvmResult<()> {
        Ok(())
    }

    async fn jlet_destroy(_jvm: &Jvm, _context: &mut (), _this: ClassInstanceRef<DirectJlet>, _unconditional: bool) -> JvmResult<()> {
        Ok(())
    }

    async fn deep_jlet_pause(_jvm: &Jvm, _context: &mut (), _this: ClassInstanceRef<DeepJlet>) -> JvmResult<()> {
        Ok(())
    }

    async fn init_jvm(system: &System) -> Result<(Jvm, ArmCore, LgtJvmImplementation)> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x100)?;
        context.sp = stack + 0x100;
        core.restore_context(&context);

        let protos = [get_midp_protos().into(), get_wipi_java_protos().into()];
        let implementation = LgtJvmImplementation::new(&mut core)?;
        let jvm = JvmSupport::new_jvm(system, None, Box::new(protos), &[], implementation.clone()).await?;
        let loader_class = implementation
            .define_class_rust(&jvm, LgtClassLoader::as_proto(), Box::new(core.clone()))
            .await
            .unwrap();
        jvm.register_class(loader_class, None).await.unwrap();
        Ok((jvm, core, implementation))
    }

    #[test]
    fn test_native_jvm_runtime() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, implementation) = init_jvm(&system_clone).await?;

            let first = JavaLangString::from_rust_string(&jvm, "lgt").await.unwrap();
            let second = JavaLangString::from_rust_string(&jvm, "-native").await.unwrap();
            let combined = jvm
                .invoke_virtual(
                    &first,
                    "java/lang/String",
                    "concat",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    [second.into()],
                )
                .await
                .unwrap();
            assert_eq!(JavaLangString::to_rust_string(&jvm, &combined).await.unwrap(), "lgt-native");
            let invalid_char: JvmResult<u16> = jvm.invoke_virtual(&first, "java/lang/String", "charAt", "(I)C", (i32::MAX,)).await;
            assert!(invalid_char.is_err());

            let date: Box<dyn ClassInstance> = jvm.new_class("java/util/Date", "(J)V", (0x12345678_abcdef01i64,)).await.unwrap();
            let time: i64 = jvm.invoke_virtual(&date, "java/util/Date", "getTime", "()J", ()).await.unwrap();
            assert_eq!(time, 0x12345678_abcdef01);

            let native_date = date.as_any().downcast_ref::<JavaClassInstance>().unwrap();
            let raw_instance: RawJavaClassInstance = read_generic(&core, native_date.ptr_raw)?;
            let ptr_class: u32 = read_generic(&core, raw_instance.ptr_dispatch_table)?;
            assert_eq!(ptr_class, native_date.class()?.ptr_raw);
            assert_eq!(raw_instance.unk1, 0);
            let raw_class: RawJavaClass = read_generic(&core, ptr_class)?;
            assert_eq!(raw_class.unk1, raw_instance.ptr_dispatch_table);
            assert_eq!(raw_class.unk2, 0);
            assert_ne!(raw_instance.ptr_fields, 0);

            let mut shorts = jvm.instantiate_array("S", 10).await.unwrap();
            jvm.store_array(&mut shorts, 0, (0..10i16).collect::<Vec<_>>()).await.unwrap();
            assert_eq!(jvm.load_array::<i16>(&shorts, 5, 4).await.unwrap(), vec![5, 6, 7, 8]);
            let short_array_definition = shorts
                .class_definition()
                .as_any()
                .downcast_ref::<super::JavaArrayClassDefinition>()
                .unwrap()
                .clone();
            let short_descriptor = short_array_definition.class.descriptor()?;
            assert_eq!(read_generic::<u32, _>(&core, short_descriptor.ptr_interface_names)?, 2);
            assert_eq!(
                ClassDefinition::interface_names(&short_array_definition.class),
                vec![RustString::from("java/lang/Cloneable"), RustString::from("java/io/Serializable")]
            );
            let short_raw = short_array_definition.class.raw()?;
            assert_eq!(short_raw.unk1, short_array_definition.class.ptr_vtable()?);
            assert_eq!(short_raw.unk2, 0);

            let long_values = vec![i64::MIN, -1, 0x12345678_9abcdef0, i64::MAX];
            let mut longs = jvm.instantiate_array("J", long_values.len()).await.unwrap();
            jvm.store_array(&mut longs, 0, long_values.clone()).await.unwrap();
            assert_eq!(jvm.load_array::<i64>(&longs, 0, long_values.len()).await.unwrap(), long_values);
            let mut raw_long = [0; 8];
            longs.as_array_instance().unwrap().raw_buffer().unwrap().read(2, &mut raw_long).unwrap();
            assert_eq!(raw_long, 0x12345678_9abcdef0u64.to_le_bytes());

            let object = JavaLangString::from_rust_string(&jvm, "reference").await.unwrap();
            let expected_identity = object.identity();
            let mut references = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
            jvm.store_array(&mut references, 0, vec![object]).await.unwrap();
            let loaded: Vec<ClassInstanceRef<String>> = jvm.load_array(&references, 0, 1).await.unwrap();
            assert_eq!(loaded[0].identity(), expected_identity);

            let object_definition = jvm
                .resolve_class("java/lang/Object")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<super::JavaClassDefinition>()
                .unwrap()
                .clone();
            let object_methods = object_definition.vtable_entries(&jvm).await?;
            assert_eq!(object_methods[1].method.as_ref().unwrap().name(), "getClass");
            assert_eq!(object_methods[4].method.as_ref().unwrap().name(), "toString");
            let object: Box<dyn ClassInstance> = jvm.new_class("java/lang/Object", "()V", ()).await.unwrap();
            let missing_target: u32 = read_generic(&core, object_definition.ptr_vtable()? + 4)?;
            let error = core
                .run_function::<u32>(missing_target, &[LgtJvmSupport::class_instance_raw(&*object)])
                .await
                .unwrap_err();
            match error {
                WieError::Unimplemented(message) => assert_eq!(message, "java/lang/Object vtable index 0"),
                error => panic!("unexpected missing vtable error: {error}"),
            }

            let string_definition = jvm
                .resolve_class("java/lang/String")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<super::JavaClassDefinition>()
                .unwrap()
                .clone();
            let string_methods = string_definition.vtable_entries(&jvm).await?;
            assert_eq!(string_methods[10].method.as_ref().unwrap().name(), "length");
            assert_eq!(string_methods[11].method.as_ref().unwrap().name(), "charAt");
            assert_eq!(string_methods[14].method.as_ref().unwrap().name(), "getBytes");
            assert_eq!(string_methods[28].method.as_ref().unwrap().name(), "substring");
            for index in [10usize, 11, 14, 28] {
                let target: u32 = read_generic(&core, string_definition.ptr_vtable()? + ((index + 1) * 4) as u32)?;
                assert_eq!(target, string_methods[index].method.as_ref().unwrap().target()?);
            }

            let vector_definition = jvm
                .resolve_class("java/util/Vector")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<super::JavaClassDefinition>()
                .unwrap()
                .clone();
            let vector_methods = vector_definition.vtable_entries(&jvm).await?;
            for (index, name, descriptor) in [
                (15usize, "size", "()I"),
                (23, "elementAt", "(I)Ljava/lang/Object;"),
                (27, "removeElementAt", "(I)V"),
                (28, "insertElementAt", "(Ljava/lang/Object;I)V"),
            ] {
                let method = vector_methods[index].method.as_ref().unwrap();
                assert_eq!((method.name().as_str(), method.descriptor().as_str()), (name, descriptor));
                let target: u32 = read_generic(&core, vector_definition.ptr_vtable()? + ((index + 1) * 4) as u32)?;
                assert_eq!(target, method.target()?);
            }

            let reader_definition = jvm
                .resolve_class("java/io/Reader")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<super::JavaClassDefinition>()
                .unwrap()
                .clone();
            let reader_methods = reader_definition.vtable_entries(&jvm).await?;
            for (index, name, descriptor) in [(11usize, "read", "([C)I"), (18, "close", "()V")] {
                let method = reader_methods[index].method.as_ref().unwrap();
                assert_eq!((method.name().as_str(), method.descriptor().as_str()), (name, descriptor));
                let target: u32 = read_generic(&core, reader_definition.ptr_vtable()? + ((index + 1) * 4) as u32)?;
                assert_eq!(target, method.target()?);
            }

            let input_stream_reader_definition = jvm
                .resolve_class("java/io/InputStreamReader")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<super::JavaClassDefinition>()
                .unwrap()
                .clone();
            let input_stream_reader_methods = input_stream_reader_definition.vtable_entries(&jvm).await?;
            for (index, name, descriptor) in [(11usize, "read", "([C)I"), (18, "close", "()V")] {
                let method = input_stream_reader_methods[index].method.as_ref().unwrap();
                assert_eq!((method.name().as_str(), method.descriptor().as_str()), (name, descriptor));
                let target: u32 = read_generic(&core, input_stream_reader_definition.ptr_vtable()? + ((index + 1) * 4) as u32)?;
                assert_eq!(target, method.target()?);
            }

            let input = b"lgt-reader-data".iter().map(|value| *value as i8).collect::<Vec<_>>();
            let mut input_array = jvm.instantiate_array("B", input.len()).await.unwrap();
            jvm.store_array(&mut input_array, 0, input.clone()).await.unwrap();
            let input_stream: Box<dyn ClassInstance> = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (input_array,)).await.unwrap();
            let charset = JavaLangString::from_rust_string(&jvm, "UTF-8").await.unwrap();
            let reader: Box<dyn ClassInstance> = jvm
                .new_class(
                    "java/io/InputStreamReader",
                    "(Ljava/io/InputStream;Ljava/lang/String;)V",
                    (input_stream, charset),
                )
                .await
                .unwrap();
            let chars = jvm.instantiate_array("C", input.len()).await.unwrap();
            let read: i32 = jvm
                .invoke_virtual(&reader, "java/io/Reader", "read", "([C)I", (chars.clone(),))
                .await
                .unwrap();
            assert!(read > 0 && read as usize <= input.len());
            assert_eq!(
                jvm.load_array::<u16>(&chars, 0, read as usize).await.unwrap(),
                input[..read as usize].iter().map(|value| *value as u16).collect::<Vec<_>>()
            );

            let base_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/Base",
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec![],
                        methods: vec![JavaMethodProto::new("value", "()I", base_value, MethodAccessFlags::PUBLIC)],
                        fields: vec![],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let base_definition = base_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap().clone();
            let base_descriptor = base_definition.descriptor()?;
            assert_eq!(read_generic::<u32, _>(&core, base_descriptor.ptr_interface_names)?, 0);
            assert!(ClassDefinition::interface_names(&base_definition).is_empty());
            jvm.register_class(base_class, None).await.unwrap();

            let child_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/Child",
                        parent_class: Some("net/wie/test/Base"),
                        interfaces: vec![],
                        methods: vec![JavaMethodProto::new("value", "()I", child_value, MethodAccessFlags::PUBLIC)],
                        fields: vec![],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            jvm.register_class(child_class, None).await.unwrap();

            let child = jvm.instantiate_class("net/wie/test/Child").await.unwrap();
            let value: i32 = jvm.invoke_virtual(&child, "net/wie/test/Base", "value", "()I", ()).await.unwrap();
            assert_eq!(value, 2);

            let interface_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/Interfaces",
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec!["java/lang/Runnable", "java/lang/Cloneable"],
                        methods: vec![],
                        fields: vec![],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let interface_definition = interface_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap().clone();
            let interface_descriptor = interface_definition.descriptor()?;
            assert_eq!(read_generic::<u32, _>(&core, interface_descriptor.ptr_interface_names)?, 2);
            assert_eq!(
                ClassDefinition::interface_names(&interface_definition),
                vec![RustString::from("java/lang/Runnable"), RustString::from("java/lang/Cloneable")]
            );
            let interface_raw = interface_definition.raw()?;
            assert_eq!(interface_raw.unk1, interface_definition.ptr_vtable()?);
            assert_eq!(interface_raw.unk2, 0);

            let direct_jlet_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/DirectJlet",
                        parent_class: Some("org/kwis/msp/lcdui/Jlet"),
                        interfaces: vec![],
                        methods: vec![
                            JavaMethodProto::new("startApp", "([Ljava/lang/String;)V", jlet_start, MethodAccessFlags::PROTECTED),
                            JavaMethodProto::new("pauseApp", "()V", jlet_pause, MethodAccessFlags::PROTECTED),
                            JavaMethodProto::new("resumeApp", "()V", jlet_resume, MethodAccessFlags::PROTECTED),
                            JavaMethodProto::new("destroyApp", "(Z)V", jlet_destroy, MethodAccessFlags::PROTECTED),
                        ],
                        fields: vec![],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let direct_jlet_definition = direct_jlet_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap().clone();
            let direct_jlet_methods = direct_jlet_definition.vtable_entries(&jvm).await?;
            for (index, name, descriptor) in [
                (15usize, "startApp", "([Ljava/lang/String;)V"),
                (16, "pauseApp", "()V"),
                (17, "resumeApp", "()V"),
                (18, "destroyApp", "(Z)V"),
            ] {
                let method = direct_jlet_methods[index].method.as_ref().unwrap();
                assert_eq!((method.name().as_str(), method.descriptor().as_str()), (name, descriptor));
                let target: u32 = read_generic(&core, direct_jlet_definition.ptr_vtable()? + ((index + 1) * 4) as u32)?;
                assert_eq!(target, method.target()?);
            }
            jvm.register_class(direct_jlet_class, None).await.unwrap();

            let deep_jlet_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/DeepJlet",
                        parent_class: Some("net/wie/test/DirectJlet"),
                        interfaces: vec![],
                        methods: vec![JavaMethodProto::new("pauseApp", "()V", deep_jlet_pause, MethodAccessFlags::PROTECTED)],
                        fields: vec![],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let deep_jlet_definition = deep_jlet_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap().clone();
            let deep_jlet_methods = deep_jlet_definition.vtable_entries(&jvm).await?;
            assert_eq!(deep_jlet_methods[16].method.as_ref().unwrap().name(), "pauseApp");
            assert_ne!(
                deep_jlet_methods[16].method.as_ref().unwrap().target()?,
                direct_jlet_methods[16].method.as_ref().unwrap().target()?
            );
            for index in [15usize, 17, 18] {
                assert_eq!(
                    deep_jlet_methods[index].method.as_ref().unwrap().target()?,
                    direct_jlet_methods[index].method.as_ref().unwrap().target()?
                );
            }
            for (offset, method) in deep_jlet_methods[15..=18].iter().enumerate() {
                let index = offset + 15;
                let target: u32 = read_generic(&core, deep_jlet_definition.ptr_vtable()? + ((index + 1) * 4) as u32)?;
                assert_eq!(target, method.method.as_ref().unwrap().target()?);
            }

            let static_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/StaticStorage",
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec![],
                        methods: vec![],
                        fields: vec![
                            JavaFieldProto::new("word", "I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                            JavaFieldProto::new("wide", "J", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                            JavaFieldProto::new("reference", "Ljava/lang/String;", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                        ],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let mut static_definition = static_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap().clone();
            let word_field = ClassDefinition::field(&static_definition, "word", "I", true).unwrap();
            let wide_field = ClassDefinition::field(&static_definition, "wide", "J", true).unwrap();
            let reference_field = ClassDefinition::field(&static_definition, "reference", "Ljava/lang/String;", true).unwrap();
            let reference_raw = first.as_any().downcast_ref::<JavaClassInstance>().unwrap().ptr_raw;
            static_definition.put_static_field(&*word_field, JavaValue::Int(0x1234_5678)).unwrap();
            static_definition
                .put_static_field(&*wide_field, JavaValue::Long(0x1234_5678_9abc_def0))
                .unwrap();
            static_definition
                .put_static_field(&*reference_field, JavaValue::Object(Some(first.clone())))
                .unwrap();

            let descriptor = static_definition.descriptor()?;
            assert_ne!(descriptor.fn_get_class, 0);
            assert_ne!(descriptor.fn_get_initialized_class, 0);
            let java_class = jvm.register_class(static_class, None).await.unwrap().unwrap();
            let java_class_instance = java_class.as_any().downcast_ref::<JavaClassInstance>().unwrap();
            let java_class_raw = java_class_instance.ptr_raw;
            let class_fields = java_class_instance.ptr_fields()?;
            let static_fields = static_definition.ptr_static_fields()?;

            let class_object: u32 = core.run_function(descriptor.fn_get_class, &[]).await?;
            assert_eq!(class_object, java_class_raw);
            assert_eq!(static_fields, java_class_instance.ptr_fields()? + 0x14);
            assert_eq!(read_generic::<u32, _>(&core, static_fields)?, 0x1234_5678);
            assert_eq!(read_generic::<u32, _>(&core, static_fields + 4)?, 0x9abc_def0);
            assert_eq!(read_generic::<u32, _>(&core, static_fields + 8)?, 0x1234_5678);
            assert_eq!(read_generic::<u32, _>(&core, static_fields + 12)?, reference_raw);

            static_definition.put_static_field(&*word_field, JavaValue::Int(0x7654_3210)).unwrap();
            assert_eq!(read_generic::<u32, _>(&core, static_fields)?, 0x7654_3210);
            write_generic(&mut core, static_fields + 4, 0x89ab_cdefu32)?;
            write_generic(&mut core, static_fields + 8, 0x0123_4567u32)?;
            assert_eq!(
                i64::from(static_definition.get_static_field(&*wide_field).unwrap()),
                0x0123_4567_89ab_cdef
            );
            let guest_reference = JavaLangString::from_rust_string(&jvm, "guest-reference").await.unwrap();
            let guest_reference_raw = guest_reference.as_any().downcast_ref::<JavaClassInstance>().unwrap().ptr_raw;
            write_generic(&mut core, static_fields + 12, guest_reference_raw)?;
            let JavaValue::Object(Some(loaded_reference)) = static_definition.get_static_field(&*reference_field).unwrap() else {
                panic!("expected static reference field")
            };
            assert_eq!(loaded_reference.identity(), guest_reference.identity());

            let initialized_class_object: u32 = core.run_function(descriptor.fn_get_initialized_class, &[]).await?;
            assert_eq!(initialized_class_object, class_object);
            assert_ne!(java_class_instance.ptr_fields()?, class_fields);
            let native_name: i32 = jvm.get_field(&java_class, CLASS_NATIVE_NAME_FIELD, WORD_FIELD_DESCRIPTOR).await.unwrap();
            let initialization_state: i32 = jvm
                .get_field(&java_class, CLASS_INITIALIZATION_STATE_FIELD, WORD_FIELD_DESCRIPTOR)
                .await
                .unwrap();
            assert_eq!(native_name as u32, descriptor.ptr_name);
            assert_eq!(initialization_state, 5);
            let class_definition = java_class.class_definition();
            let native_name_field = ClassDefinition::field(&*class_definition, CLASS_NATIVE_NAME_FIELD, WORD_FIELD_DESCRIPTOR, false).unwrap();
            let initialization_state_field =
                ClassDefinition::field(&*class_definition, CLASS_INITIALIZATION_STATE_FIELD, WORD_FIELD_DESCRIPTOR, false).unwrap();
            assert_eq!(native_name_field.as_any().downcast_ref::<super::JavaField>().unwrap().word_index()?, 2);
            assert_eq!(
                initialization_state_field
                    .as_any()
                    .downcast_ref::<super::JavaField>()
                    .unwrap()
                    .word_index()?,
                4
            );
            let class_object_again: u32 = core.run_function(descriptor.fn_get_class, &[]).await?;
            assert_eq!(class_object_again, class_object);
            let initialization_state: i32 = jvm
                .get_field(&java_class, CLASS_INITIALIZATION_STATE_FIELD, WORD_FIELD_DESCRIPTOR)
                .await
                .unwrap();
            assert_eq!(initialization_state, 5);

            let static_child_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/StaticStorageChild",
                        parent_class: Some("net/wie/test/StaticStorage"),
                        interfaces: vec![],
                        methods: vec![],
                        fields: vec![JavaFieldProto::new("word", "I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC)],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let mut static_child_definition = static_child_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap().clone();
            let child_word_field = ClassDefinition::field(&static_child_definition, "word", "I", true).unwrap();
            static_child_definition
                .put_static_field(&*child_word_field, JavaValue::Int(0x1071))
                .unwrap();
            let JavaValue::Object(Some(inherited_reference)) = static_child_definition.get_static_field(&*reference_field).unwrap() else {
                panic!("expected inherited static reference field")
            };
            assert_eq!(inherited_reference.identity(), guest_reference.identity());

            write_generic(&mut core, static_fields, guest_reference_raw)?;
            static_child_definition.put_static_field(&*child_word_field, JavaValue::Int(4)).unwrap();
            let static_reference_field = ClassDefinition::fields(&static_definition)
                .into_iter()
                .find(|field| {
                    field
                        .as_any()
                        .downcast_ref::<super::JavaStaticReferenceField>()
                        .is_some_and(|field| field.word_index == 0)
                })
                .unwrap();
            let JavaValue::Object(Some(inherited_reference)) = static_child_definition.get_static_field(&*static_reference_field).unwrap() else {
                panic!("expected inherited untyped static reference")
            };
            assert_eq!(inherited_reference.identity(), guest_reference.identity());

            let short_class = jvm.get_class(&ClassDefinition::name(&short_array_definition.class)).unwrap();
            let short_java_class = short_class.java_class();
            let short_java_class_raw = LgtJvmSupport::class_instance_raw(&*short_java_class);
            let short_class_object: u32 = core.run_function(short_descriptor.fn_get_class, &[]).await?;
            assert_eq!(short_class_object, short_java_class_raw);
            let short_initialized_class: u32 = core.run_function(short_descriptor.fn_get_initialized_class, &[]).await?;
            assert_eq!(short_initialized_class, short_class_object);
            let initialization_state: i32 = jvm
                .get_field(&short_java_class, CLASS_INITIALIZATION_STATE_FIELD, WORD_FIELD_DESCRIPTOR)
                .await
                .unwrap();
            assert_eq!(initialization_state, 5);

            let field_parent = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/FieldParent",
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec![],
                        methods: vec![],
                        fields: vec![
                            JavaFieldProto::new("f0", "I", FieldAccessFlags::PRIVATE),
                            JavaFieldProto::new("f1", "I", FieldAccessFlags::PRIVATE),
                            JavaFieldProto::new("f2", "I", FieldAccessFlags::PRIVATE),
                            JavaFieldProto::new("f3", "I", FieldAccessFlags::PRIVATE),
                            JavaFieldProto::new("f4", "I", FieldAccessFlags::PRIVATE),
                            JavaFieldProto::new("f5", "I", FieldAccessFlags::PRIVATE),
                        ],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            jvm.register_class(field_parent, None).await.unwrap();
            let field_child = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/FieldChild",
                        parent_class: Some("net/wie/test/FieldParent"),
                        interfaces: vec![],
                        methods: vec![],
                        fields: vec![
                            JavaFieldProto::new("own0", "I", FieldAccessFlags::PRIVATE),
                            JavaFieldProto::new("own1", "I", FieldAccessFlags::PRIVATE),
                            JavaFieldProto::new("static0", "I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                        ],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let field_child_definition = field_child.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap().clone();
            assert_eq!(field_child_definition.instance_field_word_count()?, 8);
            let own0 = ClassDefinition::field(&field_child_definition, "own0", "I", false).unwrap();
            let own1 = ClassDefinition::field(&field_child_definition, "own1", "I", false).unwrap();
            let static0 = ClassDefinition::field(&field_child_definition, "static0", "I", true).unwrap();
            let own0 = own0.as_any().downcast_ref::<super::JavaField>().unwrap();
            let own1 = own1.as_any().downcast_ref::<super::JavaField>().unwrap();
            let static0 = static0.as_any().downcast_ref::<super::JavaField>().unwrap();
            assert_eq!(own0.word_index()?, 6);
            assert_eq!(own1.word_index()?, 7);
            write_generic(&mut core, own0.ptr_raw + offset_of!(RawJavaField, word_index) as u32, 0u32)?;
            write_generic(&mut core, own1.ptr_raw + offset_of!(RawJavaField, word_index) as u32, 1u32)?;
            field_child_definition.patch_declared_instance_field_word_indices()?;
            field_child_definition.patch_declared_instance_field_word_indices()?;
            assert_eq!(own0.word_index()?, 6);
            assert_eq!(own1.word_index()?, 7);
            assert_eq!(static0.word_index()?, 0);

            let mut field_child_instance: Box<dyn ClassInstance> = Box::new(JavaClassInstance::new(&mut core, &field_child_definition)?);
            let ptr_fields = field_child_instance.as_any().downcast_ref::<JavaClassInstance>().unwrap().ptr_fields()?;
            jvm.put_field(&mut field_child_instance, "own0", "I", 0x1234_5678i32).await.unwrap();
            assert_eq!(read_generic::<u32, _>(&core, ptr_fields + 6 * size_of::<u32>() as u32)?, 0x1234_5678);
            write_generic(&mut core, ptr_fields + 7 * size_of::<u32>() as u32, 0x7654_3210u32)?;
            let own1: i32 = jvm.get_field(&field_child_instance, "own1", "I").await.unwrap();
            assert_eq!(own1, 0x7654_3210);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    #[test]
    fn test_generated_jlet_wrapper_overrides_use_confirmed_indices() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, implementation) = init_jvm(&system_clone).await?;
            let generated_entry = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/GeneratedJletEntry",
                        parent_class: Some("org/kwis/msp/lcdui/JletWrapper"),
                        interfaces: vec![],
                        methods: vec![
                            JavaMethodProto::new("startApp", "([Ljava/lang/String;)V", jlet_start, MethodAccessFlags::PROTECTED),
                            JavaMethodProto::new("pauseApp", "()V", jlet_pause, MethodAccessFlags::PROTECTED),
                            JavaMethodProto::new("resumeApp", "()V", jlet_resume, MethodAccessFlags::PROTECTED),
                            JavaMethodProto::new("destroyApp", "(Z)V", jlet_destroy, MethodAccessFlags::PROTECTED),
                        ],
                        fields: vec![],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let generated_definition = generated_entry.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap();
            let ptr_entry = generated_definition.ptr_raw;
            let ptr_parent_name = Allocator::alloc(&mut core, "org/kwis/msp/lcdui/JletWrapper".len() as u32 + 1)?;
            write_null_terminated_string_bytes(&mut core, ptr_parent_name, b"org/kwis/msp/lcdui/JletWrapper")?;
            let raw_class: RawJavaClass = read_generic(&core, ptr_entry)?;
            let mut descriptor: RawJavaClassDescriptor = read_generic(&core, raw_class.ptr_descriptor)?;
            descriptor.ptr_super_class = ptr_parent_name;
            descriptor.flags |= LGT_JAVA_CLASS_SUPER_CLASS_IS_NAME;
            write_generic(&mut core, raw_class.ptr_descriptor, descriptor)?;

            let loader: Box<dyn ClassInstance> = jvm
                .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
                .await
                .unwrap();
            let generated_classes = Allocator::alloc(&mut core, 2 * size_of::<u32>() as u32)?;
            write_generic(&mut core, generated_classes, 0u32)?;
            write_generic(&mut core, generated_classes + size_of::<u32>() as u32, ptr_entry)?;
            LgtJvmSupport::register_generated_class(&mut core, &jvm, ptr_entry, generated_classes, loader).await?;

            let definition = jvm
                .get_class("net/wie/test/GeneratedJletEntry")
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<super::JavaClassDefinition>()
                .unwrap()
                .clone();
            let descriptor = definition.descriptor()?;
            let parent_class = jvm.get_class("org/kwis/msp/lcdui/JletWrapper").unwrap();
            let parent = parent_class.definition.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap();
            assert_eq!(descriptor.ptr_super_class, parent.ptr_raw);
            assert_eq!(descriptor.flags & LGT_JAVA_CLASS_SUPER_CLASS_IS_NAME, 0);

            let methods = definition.vtable_entries(&jvm).await?;
            for (index, name, descriptor) in [
                (15usize, "startApp", "([Ljava/lang/String;)V"),
                (16, "pauseApp", "()V"),
                (17, "resumeApp", "()V"),
                (18, "destroyApp", "(Z)V"),
            ] {
                let method = methods[index].method.as_ref().unwrap();
                assert_eq!((method.name().as_str(), method.descriptor().as_str()), (name, descriptor));
                assert_eq!(
                    read_generic::<u32, _>(&core, definition.ptr_vtable()? + ((index + 1) * size_of::<u32>()) as u32)?,
                    method.target()?
                );
            }

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    #[test]
    fn generated_class_exposes_compiler_vtable_methods_to_jvm() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, implementation) = init_jvm(&system_clone).await?;
            let generated_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/GeneratedInputStream",
                        parent_class: Some("java/io/InputStream"),
                        interfaces: vec![],
                        methods: vec![JavaMethodProto::new("read", "()I", base_value, MethodAccessFlags::PUBLIC)],
                        fields: vec![],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let generated_definition = generated_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap();
            let raw_class: RawJavaClass = read_generic(&core, generated_definition.ptr_raw)?;
            let mut descriptor: RawJavaClassDescriptor = read_generic(&core, raw_class.ptr_descriptor)?;
            let parent_name = "java/io/InputStream";
            let ptr_parent_name = Allocator::alloc(&mut core, parent_name.len() as u32 + 1)?;
            write_null_terminated_string_bytes(&mut core, ptr_parent_name, parent_name.as_bytes())?;
            descriptor.ptr_super_class = ptr_parent_name;
            descriptor.flags |= LGT_JAVA_CLASS_SUPER_CLASS_IS_NAME;
            descriptor.ptr_vtable = raw_class.unk1;
            descriptor.ptr_methods = 0;
            write_generic(&mut core, raw_class.ptr_descriptor, descriptor)?;

            let loader: Box<dyn ClassInstance> = jvm
                .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
                .await
                .unwrap();
            let generated_classes = Allocator::alloc(&mut core, 2 * size_of::<u32>() as u32)?;
            write_generic(&mut core, generated_classes, 0u32)?;
            write_generic(&mut core, generated_classes + size_of::<u32>() as u32, generated_definition.ptr_raw)?;
            LgtJvmSupport::register_generated_class(&mut core, &jvm, generated_definition.ptr_raw, generated_classes, loader).await?;

            let linked_class = jvm.get_class("net/wie/test/GeneratedInputStream").unwrap();
            let linked_definition = linked_class.definition.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap();
            assert_ne!(linked_definition.descriptor()?.ptr_methods, 0);
            let instance = jvm.instantiate_class("net/wie/test/GeneratedInputStream").await.unwrap();
            let value: i32 = jvm.invoke_virtual(&instance, "java/io/InputStream", "read", "()I", ()).await.unwrap();
            assert_eq!(value, 1);

            let generated_class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/GeneratedRunnable",
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec!["java/lang/Runnable"],
                        methods: vec![JavaMethodProto::new("run", "()V", jlet_pause, MethodAccessFlags::PUBLIC)],
                        fields: vec![],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                )
                .await
                .unwrap();
            let generated_definition = generated_class.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap();
            let raw_class: RawJavaClass = read_generic(&core, generated_definition.ptr_raw)?;
            let mut descriptor: RawJavaClassDescriptor = read_generic(&core, raw_class.ptr_descriptor)?;
            let method: RawJavaMethod = read_generic(&core, descriptor.ptr_methods + size_of::<u32>() as u32)?;
            let ptr_vtable = Allocator::alloc(&mut core, 12 * size_of::<u32>() as u32)?;
            let mut inherited_vtable = vec![0; 11 * size_of::<u32>()];
            core.read_bytes(raw_class.unk1, &mut inherited_vtable)?;
            core.write_bytes(ptr_vtable, &inherited_vtable)?;
            write_generic(&mut core, ptr_vtable + 11 * size_of::<u32>() as u32, method.ptr_method)?;

            let interface_name = "java/lang/Runnable";
            let ptr_interface_name = Allocator::alloc(&mut core, interface_name.len() as u32 + 1)?;
            write_null_terminated_string_bytes(&mut core, ptr_interface_name, interface_name.as_bytes())?;
            let ptr_interface_reference = Allocator::alloc(&mut core, size_of::<RawJavaInterfaceReference>() as u32)?;
            write_generic(
                &mut core,
                ptr_interface_reference,
                RawJavaInterfaceReference {
                    ptr_class_or_name: ptr_interface_name,
                },
            )?;
            let ptr_interface_references = Allocator::alloc(&mut core, (size_of::<RawJavaInterfaceReferences>() + size_of::<u32>()) as u32)?;
            write_generic(
                &mut core,
                ptr_interface_references,
                RawJavaInterfaceReferences { count: 1, references: [] },
            )?;
            write_generic(
                &mut core,
                ptr_interface_references + size_of::<RawJavaInterfaceReferences>() as u32,
                ptr_interface_reference,
            )?;

            descriptor.ptr_vtable = ptr_vtable;
            descriptor.vtable_count = 11;
            descriptor.ptr_interface_references = ptr_interface_references;
            descriptor.ptr_interface_names = 0;
            descriptor.ptr_methods = 0;
            write_generic(&mut core, raw_class.ptr_descriptor, descriptor)?;

            let loader: Box<dyn ClassInstance> = jvm
                .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
                .await
                .unwrap();
            write_generic(&mut core, generated_classes + size_of::<u32>() as u32, generated_definition.ptr_raw)?;
            LgtJvmSupport::register_generated_class(&mut core, &jvm, generated_definition.ptr_raw, generated_classes, loader).await?;

            let linked_class = jvm.get_class("net/wie/test/GeneratedRunnable").unwrap();
            let linked_definition = linked_class.definition.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap();
            assert_eq!(
                ClassDefinition::interface_names(linked_definition),
                vec![RustString::from(interface_name)]
            );
            let reference: RawJavaInterfaceReference = read_generic(&core, ptr_interface_reference)?;
            let runnable = jvm.get_class(interface_name).unwrap();
            let runnable_definition = runnable.definition.as_any().downcast_ref::<super::JavaClassDefinition>().unwrap();
            assert_eq!(reference.ptr_class_or_name, runnable_definition.ptr_raw);
            let instance = jvm.instantiate_class("net/wie/test/GeneratedRunnable").await.unwrap();
            jvm.invoke_virtual::<_, ()>(&instance, "java/lang/Runnable", "run", "()V", ())
                .await
                .unwrap();

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }
}
