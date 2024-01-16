use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use core::{
    fmt::{self, Debug, Formatter},
    mem::size_of,
};
use wie_backend::SystemHandle;

use jvm::{ArrayClass, ClassInstance, JavaType, JvmResult};

use wie_common::util::{write_generic, write_null_terminated_string};
use wie_core_arm::{Allocator, ArmCore};

use super::{
    array_class_instance::JavaArrayClassInstance,
    class::JavaClass,
    class::{RawJavaClass, RawJavaClassDescriptor},
    KtfJvm,
};

#[derive(Clone)]
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

    pub async fn new(core: &mut ArmCore, system: &mut SystemHandle, name: &str) -> JvmResult<Self> {
        let mut jvm = KtfJvm::new(core, system);
        let java_lang_object = jvm.jvm().resolve_class("java/lang/Object").await?;
        let java_lang_object_raw = jvm.class_raw(&*java_lang_object.unwrap());

        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;

        let element_type_name = &name[1..];
        let element_type_raw = if element_type_name.starts_with('L') {
            let java_class = jvm
                .jvm()
                .resolve_class(&element_type_name[1..element_type_name.len() - 1])
                .await?
                .unwrap();

            Some(jvm.class_raw(&*java_class))
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
                ptr_parent_class: java_lang_object_raw,
                ptr_methods: 0,
                ptr_interfaces: 0,
                ptr_fields_or_element_type: element_type_raw.unwrap_or(0),
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

        Ok(class)
    }

    pub fn element_type_descriptor(&self) -> JvmResult<String> {
        let class_name = self.class.name()?;

        Ok(class_name[1..].to_string())
    }

    pub fn element_size(&self) -> JvmResult<usize> {
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

impl ArrayClass for JavaArrayClass {
    fn element_type_name(&self) -> String {
        let class_name = self.class.name().unwrap();

        class_name[1..].into()
    }

    fn instantiate_array(&self, length: usize) -> Box<dyn ClassInstance> {
        Box::new(JavaArrayClassInstance::new(&mut self.core.clone(), self, length).unwrap())
    }
}

impl Debug for JavaArrayClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaArrayClass").field("class", &self.class).finish()
    }
}
