use alloc::{vec, vec::Vec};
use core::{iter, mem::size_of};

use bytemuck::{Pod, Zeroable};

use wie_base::util::{read_generic, write_generic, ByteWrite};
use wie_core_arm::{Allocator, ArmCore};
use wie_impl_java::{JavaResult, JavaWord};

use crate::runtime::java::context::context_data::JavaContextData;

use super::{class::JavaClass, field::JavaField, method::JavaMethod};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RawJavaClassInstance {
    ptr_fields: u32,
    ptr_class: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RawJavaClassInstanceFields {
    vtable_index: u32, // left shifted by 5
    fields: [u32; 1],
}

pub struct JavaClassInstance {
    pub(crate) ptr_raw: u32,
    core: ArmCore,
}

impl JavaClassInstance {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn new(core: &mut ArmCore, class: &JavaClass) -> JavaResult<Self> {
        let field_size = class.field_size()?;

        let instance = Self::instantiate(core, class, field_size)?;

        tracing::trace!("Instantiated {} at {:#x}", class.name()?, instance.ptr_raw);

        Ok(instance)
    }

    pub fn destroy(mut self) -> JavaResult<()> {
        let raw = self.read_raw()?;

        Allocator::free(&mut self.core, raw.ptr_fields)?;
        Allocator::free(&mut self.core, self.ptr_raw)?;

        Ok(())
    }

    pub fn class(&self) -> JavaResult<JavaClass> {
        let raw = self.read_raw()?;

        Ok(JavaClass::from_raw(raw.ptr_class, &self.core))
    }

    pub fn read_field(&self, field: &JavaField) -> JavaResult<JavaWord> {
        let offset = field.offset()?;

        let address = self.field_address(offset)?;

        let value: u32 = read_generic(&self.core, address)?;

        Ok(value as _)
    }

    pub fn write_field(&mut self, field: &JavaField, value: JavaWord) -> JavaResult<()> {
        let offset = field.offset()?;

        let address = self.field_address(offset)?;

        write_generic(&mut self.core, address, value as u32)
    }

    pub async fn invoke_method(&self, method: &mut JavaMethod, args: &[JavaWord]) -> JavaResult<u32> {
        let mut params = vec![self.ptr_raw as _];
        params.extend(args);

        method.run(&params).await
    }

    pub(super) fn field_address(&self, offset: u32) -> JavaResult<u32> {
        let raw = self.read_raw()?;

        Ok(raw.ptr_fields + offset + 4)
    }

    pub(super) fn instantiate(core: &mut ArmCore, class: &JavaClass, field_size: JavaWord) -> JavaResult<Self> {
        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClassInstance>() as _)?;
        let ptr_fields = Allocator::alloc(core, (field_size + 4) as _)?;

        let zero = iter::repeat(0).take((field_size + 4) as _).collect::<Vec<_>>();
        core.write_bytes(ptr_fields, &zero)?;

        let vtable_index = JavaContextData::get_vtable_index(core, class)?;

        write_generic(
            core,
            ptr_raw,
            RawJavaClassInstance {
                ptr_fields,
                ptr_class: class.ptr_raw,
            },
        )?;
        write_generic(core, ptr_fields, (vtable_index * 4) << 5)?;

        tracing::trace!("Instantiate {}, vtable_index {:#x}", class.name()?, vtable_index);

        Ok(Self::from_raw(ptr_raw, core))
    }

    fn read_raw(&self) -> JavaResult<RawJavaClassInstance> {
        let instance: RawJavaClassInstance = read_generic(&self.core, self.ptr_raw as _)?;

        Ok(instance)
    }
}
