use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use core::mem::size_of;

use jvm::{ArrayClass, Class, ClassInstance, Field, JavaType, JavaValue, JvmResult, Method};

use wie_base::util::{write_generic, write_null_terminated_string};
use wie_core_arm::{Allocator, ArmCore};
use wie_impl_java::JavaResult;

use super::{
    array_class_instance::JavaArrayClassInstance,
    class::JavaClass,
    class::{RawJavaClass, RawJavaClassDescriptor},
    class_loader::ClassLoader,
    context_data::JavaContextData,
};

pub struct JavaArrayClass {
    pub(crate) class: JavaClass,
    core: ArmCore,
}

impl JavaArrayClass {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self {
            class: JavaClass::from_raw(ptr_raw, core),
            core: core.clone(),
        }
    }

    pub async fn new(core: &mut ArmCore, name: &str) -> JavaResult<Self> {
        let ptr_parent_class = ClassLoader::get_or_load_class(core, "java/lang/Object").await?.unwrap();
        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;

        let element_type_name = &name[1..];
        let element_type = if element_type_name.starts_with('L') {
            Some(
                ClassLoader::get_or_load_class(core, &element_type_name[1..element_type_name.len() - 1])
                    .await?
                    .unwrap(),
            )
        } else {
            None
        };

        let ptr_name = Allocator::alloc(core, (name.len() + 1) as u32)?;
        write_null_terminated_string(core, ptr_name, name)?;

        let ptr_descriptor = Allocator::alloc(core, size_of::<RawJavaClassDescriptor>() as u32)?;
        write_generic(
            core,
            ptr_descriptor,
            RawJavaClassDescriptor {
                ptr_name,
                unk1: 0,
                ptr_parent_class: ptr_parent_class.ptr_raw,
                ptr_methods: 0,
                ptr_interfaces: 0,
                ptr_fields_or_element_type: element_type.map(|x| x.ptr_raw).unwrap_or(0),
                method_count: 0,
                fields_size: 0,
                access_flag: 0x21, // ACC_PUBLIC | ACC_SUPER
                unk6: 0,
                unk7: 0,
                unk8: 0,
            },
        )?;

        write_generic(
            core,
            ptr_raw,
            RawJavaClass {
                ptr_next: ptr_raw + 4,
                unk1: 0,
                ptr_descriptor,
                ptr_vtable: 0,
                vtable_count: 0,
                unk_flag: 8,
            },
        )?;

        let class = JavaArrayClass::from_raw(ptr_raw, core);

        JavaContextData::register_class(core, &class.class)?;

        Ok(class)
    }

    pub fn element_type_descriptor(&self) -> JavaResult<String> {
        let class_name = self.class.name()?;

        Ok(class_name[1..].to_string())
    }

    pub fn element_size(&self) -> JavaResult<usize> {
        let r#type = JavaType::parse(&self.element_type_descriptor()?);
        Ok(match r#type {
            JavaType::Boolean => 1,
            JavaType::Byte => 1,
            JavaType::Char => 2,
            JavaType::Short => 2,
            JavaType::Int => 4,
            JavaType::Long => 8,
            JavaType::Float => 4,
            JavaType::Double => 8,
            JavaType::Class(_) => 4, // TODO do we need to extract pointer size to constant?
            JavaType::Array(_) => 4,
            _ => panic!("Should not reach here"),
        })
    }
}

impl Class for JavaArrayClass {
    fn name(&self) -> String {
        self.class.name().unwrap()
    }

    fn super_class_name(&self) -> Option<String> {
        Some("java/lang/Object".into())
    }

    fn instantiate(&self) -> Box<dyn ClassInstance> {
        panic!("Array class should not be instantiated here")
    }

    fn method(&self, _name: &str, _descriptor: &str) -> Option<Box<dyn Method>> {
        panic!("Array class does not have methods")
    }

    fn field(&self, _name: &str, _descriptor: &str, _is_static: bool) -> Option<Box<dyn Field>> {
        panic!("Array class does not have fields")
    }

    fn get_static_field(&self, _field: &dyn Field) -> JvmResult<JavaValue> {
        panic!("Array class does not have fields")
    }

    fn put_static_field(&mut self, _field: &dyn Field, _value: JavaValue) -> JvmResult<()> {
        panic!("Array class does not have fields")
    }
}

impl ArrayClass for JavaArrayClass {
    fn element_type_name(&self) -> String {
        let class_name = self.class.name().unwrap();

        class_name[1..].into()
    }

    fn instantiate_array(&self, length: usize) -> Box<dyn ClassInstance> {
        Box::new(JavaArrayClassInstance::new(&mut self.core.clone(), self, length).unwrap())
    }
}
