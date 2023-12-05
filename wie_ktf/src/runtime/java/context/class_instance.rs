use alloc::vec::Vec;
use core::{iter, mem::size_of};

use bytemuck::{Pod, Zeroable};

use wie_base::util::{read_generic, write_generic, ByteWrite};
use wie_core_arm::Allocator;
use wie_impl_java::{JavaResult, JavaWord};

use crate::runtime::java::context::context_data::JavaContextData;

use super::{class::JavaClass, field::JavaField, KtfJavaContext};

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
}

impl JavaClassInstance {
    pub fn from_raw(ptr_raw: u32) -> Self {
        Self { ptr_raw }
    }

    pub async fn new(context: &mut KtfJavaContext<'_>, class: &JavaClass) -> JavaResult<Self> {
        let field_size = class.field_size(context)?;

        let instance = Self::instantiate(context, class, field_size).await?;

        tracing::trace!("Instantiated {} at {:#x}", class.name(context)?, instance.ptr_raw);

        Ok(instance)
    }

    pub fn destroy(&self, context: &mut KtfJavaContext<'_>) -> JavaResult<()> {
        let raw = self.read_raw(context)?;

        Allocator::free(context.core, raw.ptr_fields)?;
        Allocator::free(context.core, self.ptr_raw)?;

        Ok(())
    }

    pub fn class(&self, context: &KtfJavaContext<'_>) -> JavaResult<JavaClass> {
        let raw = self.read_raw(context)?;

        Ok(JavaClass::from_raw(raw.ptr_class))
    }

    pub fn read_field(&self, context: &KtfJavaContext<'_>, field: &JavaField) -> JavaResult<JavaWord> {
        let offset = field.offset(context)?;

        let address = self.field_address(context, offset)?;

        let value: u32 = read_generic(context.core, address)?;

        Ok(value as _)
    }

    pub fn write_field(&self, context: &mut KtfJavaContext<'_>, field: &JavaField, value: JavaWord) -> JavaResult<()> {
        let offset = field.offset(context)?;

        let address = self.field_address(context, offset)?;

        write_generic(context.core, address, value as u32)
    }

    pub(super) fn field_address(&self, context: &KtfJavaContext<'_>, offset: u32) -> JavaResult<u32> {
        let raw = self.read_raw(context)?;

        Ok(raw.ptr_fields + offset + 4)
    }

    pub(super) async fn instantiate(context: &mut KtfJavaContext<'_>, class: &JavaClass, field_size: JavaWord) -> JavaResult<Self> {
        let ptr_raw = Allocator::alloc(context.core, size_of::<RawJavaClassInstance>() as _)?;
        let ptr_fields = Allocator::alloc(context.core, (field_size + 4) as _)?;

        let zero = iter::repeat(0).take((field_size + 4) as _).collect::<Vec<_>>();
        context.core.write_bytes(ptr_fields, &zero)?;

        let vtable_index = JavaContextData::get_vtable_index(context, class)?;

        write_generic(
            context.core,
            ptr_raw,
            RawJavaClassInstance {
                ptr_fields,
                ptr_class: class.ptr_raw,
            },
        )?;
        write_generic(context.core, ptr_fields, (vtable_index * 4) << 5)?;

        tracing::trace!("Instantiate {}, vtable_index {:#x}", class.name(context)?, vtable_index);

        Ok(Self { ptr_raw })
    }

    fn read_raw(&self, context: &KtfJavaContext<'_>) -> JavaResult<RawJavaClassInstance> {
        let instance: RawJavaClassInstance = read_generic(context.core, self.ptr_raw as _)?;

        Ok(instance)
    }
}
