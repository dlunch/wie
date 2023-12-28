use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_base::util::{read_generic, read_null_terminated_table, write_generic};
use wie_core_arm::{Allocator, ArmCore, PEB_BASE};
use wie_impl_java::JavaResult;

use crate::runtime::KtfPeb;

use super::class::JavaClass;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RawJavaContextData {
    pub classes_base: u32,
    pub ptr_vtables_base: u32,
    pub fn_get_class: u32,
}

pub struct JavaContextData {}

impl JavaContextData {
    pub fn init(core: &mut ArmCore, ptr_vtables_base: u32, fn_get_class: u32) -> JavaResult<u32> {
        let classes_base = Allocator::alloc(core, 0x1000)?;
        write_generic(core, classes_base, 0u32)?;

        let ptr_java_context_data = Allocator::alloc(core, size_of::<RawJavaContextData>() as _)?;
        write_generic(
            core,
            ptr_java_context_data,
            RawJavaContextData {
                classes_base,
                ptr_vtables_base,
                fn_get_class,
            },
        )?;

        Ok(ptr_java_context_data)
    }

    pub fn get_vtable_index(core: &mut ArmCore, class: &JavaClass) -> JavaResult<u32> {
        let context_data = Self::read(core)?;
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

    pub fn register_class(core: &mut ArmCore, class: &JavaClass) -> JavaResult<()> {
        let context_data = Self::read(core)?;
        let ptr_classes = read_null_terminated_table(core, context_data.classes_base)?;
        if ptr_classes.contains(&class.ptr_raw) {
            return Ok(());
        }

        write_generic(
            core,
            context_data.classes_base + (ptr_classes.len() * size_of::<u32>()) as u32,
            class.ptr_raw,
        )
    }

    pub fn has_class(core: &ArmCore, class: &JavaClass) -> JavaResult<bool> {
        let context_data = Self::read(core)?;
        let ptr_classes = read_null_terminated_table(core, context_data.classes_base)?;

        Ok(ptr_classes.contains(&class.ptr_raw))
    }

    pub fn find_class(core: &ArmCore, name: &str) -> JavaResult<Option<JavaClass>> {
        let context_data = Self::read(core)?;
        let classes = read_null_terminated_table(core, context_data.classes_base)?;
        for ptr_raw in classes {
            let class = JavaClass::from_raw(ptr_raw, core);

            if class.name()? == name {
                return Ok(Some(JavaClass::from_raw(ptr_raw, core)));
            }
        }

        Ok(None)
    }

    pub fn fn_get_class(core: &ArmCore) -> JavaResult<u32> {
        let context_data = Self::read(core)?;

        Ok(context_data.fn_get_class)
    }

    fn read(core: &ArmCore) -> JavaResult<RawJavaContextData> {
        let peb: KtfPeb = read_generic(core, PEB_BASE)?;

        read_generic(core, peb.ptr_java_context_data)
    }
}
