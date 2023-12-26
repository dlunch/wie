use alloc::{boxed::Box, string::String};
use core::mem::size_of;

use jvm::{ArrayClass, Class, ClassInstance, Field, JavaValue, JvmResult, Method};

use wie_base::util::{write_generic, write_null_terminated_string};
use wie_core_arm::{Allocator, ArmCore};
use wie_impl_java::{JavaResult, JavaWord};

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

    pub fn element_size(&self) -> JavaResult<JavaWord> {
        let class_name = self.class.name()?;

        assert!(class_name.starts_with('['), "Not an array class {}", class_name);

        if class_name.starts_with("[L") || class_name.starts_with("[[") {
            Ok(4)
        } else {
            let element = class_name.as_bytes()[1];
            Ok(match element {
                b'B' => 1,
                b'C' => 2,
                b'I' => 4,
                b'Z' => 1,
                b'S' => 2,
                b'J' => 8,
                _ => unimplemented!("get_array_element_size {}", class_name),
            })
        }
    }
}

impl Class for JavaArrayClass {
    fn name(&self) -> String {
        self.class.name().unwrap()
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
