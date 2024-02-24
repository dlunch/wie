use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_core_arm::{Allocator, ArmCore, PEB_BASE};
use wie_util::{read_generic, read_null_terminated_table, write_generic};

use crate::runtime::KtfPeb;

use super::class_definition::JavaClassDefinition;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RawJavaContextData {
    pub ptr_vtables_base: u32,
    pub fn_get_class: u32,
}

pub struct JavaContextData {}

impl JavaContextData {
    pub fn init(core: &mut ArmCore, ptr_vtables_base: u32, fn_get_class: u32) -> anyhow::Result<u32> {
        let ptr_java_context_data = Allocator::alloc(core, size_of::<RawJavaContextData>() as _)?;
        write_generic(
            core,
            ptr_java_context_data,
            RawJavaContextData {
                ptr_vtables_base,
                fn_get_class,
            },
        )?;

        Ok(ptr_java_context_data)
    }

    pub fn get_vtable_index(core: &mut ArmCore, class: &JavaClassDefinition) -> anyhow::Result<u32> {
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

    pub fn fn_get_class(core: &ArmCore) -> anyhow::Result<u32> {
        let context_data = Self::read(core)?;

        Ok(context_data.fn_get_class)
    }

    fn read(core: &ArmCore) -> anyhow::Result<RawJavaContextData> {
        let peb: KtfPeb = read_generic(core, PEB_BASE)?;

        Ok(read_generic(core, peb.ptr_java_context_data)?)
    }
}
