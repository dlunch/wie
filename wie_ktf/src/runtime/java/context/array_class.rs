use core::mem::size_of;

use wie_base::util::{write_generic, write_null_terminated_string};
use wie_core_arm::{Allocator, ArmCore};
use wie_impl_java::JavaResult;

use super::{
    class::JavaClass,
    class::{RawJavaClass, RawJavaClassDescriptor},
    class_loader::ClassLoader,
    context_data::JavaContextData,
};

pub struct JavaArrayClass {
    pub(crate) class: JavaClass,
}

impl JavaArrayClass {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self {
            class: JavaClass::from_raw(ptr_raw, core),
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
}
