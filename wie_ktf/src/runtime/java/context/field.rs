use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_base::util::{read_generic, write_generic, ByteWrite};
use wie_core_arm::Allocator;
use wie_impl_java::{JavaFieldAccessFlag, JavaFieldProto, JavaResult, JavaWord};

use super::{class_instance::JavaClassInstance, JavaFullName, KtfJavaContext};

bitflags::bitflags! {
    struct JavaFieldAccessFlagBit: u32 {
        const NONE = 0;
        const STATIC = 8;
    }
}

impl JavaFieldAccessFlagBit {
    fn from_access_flag(access_flag: JavaFieldAccessFlag) -> JavaFieldAccessFlagBit {
        match access_flag {
            JavaFieldAccessFlag::NONE => JavaFieldAccessFlagBit::NONE,
            JavaFieldAccessFlag::STATIC => JavaFieldAccessFlagBit::STATIC,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RawJavaField {
    access_flag: u32,
    ptr_class: u32,
    ptr_name: u32,
    offset_or_value: u32,
}

pub struct JavaField {
    pub(crate) ptr_raw: u32,
}

impl JavaField {
    pub fn from_raw(ptr_raw: u32) -> Self {
        Self { ptr_raw }
    }

    pub fn new(context: &mut KtfJavaContext<'_>, ptr_class: u32, proto: JavaFieldProto, offset_or_value: u32) -> JavaResult<Self> {
        let full_name = (JavaFullName {
            tag: 0,
            name: proto.name,
            signature: proto.signature,
        })
        .as_bytes();

        let ptr_name = Allocator::alloc(context.core, full_name.len() as u32)?;
        context.core.write_bytes(ptr_name, &full_name)?;

        let ptr_raw = Allocator::alloc(context.core, size_of::<RawJavaField>() as u32)?;

        write_generic(
            context.core,
            ptr_raw,
            RawJavaField {
                access_flag: JavaFieldAccessFlagBit::from_access_flag(proto.access_flag).bits(),
                ptr_class,
                ptr_name,
                offset_or_value,
            },
        )?;

        Ok(Self::from_raw(ptr_raw))
    }

    pub fn name(&self, context: &KtfJavaContext<'_>) -> JavaResult<JavaFullName> {
        let raw: RawJavaField = read_generic(context.core, self.ptr_raw)?;

        JavaFullName::from_ptr(context.core, raw.ptr_name)
    }

    pub fn read_value(&self, context: &KtfJavaContext<'_>, instance: JavaClassInstance) -> JavaResult<JavaWord> {
        let raw: RawJavaField = read_generic(context.core, self.ptr_raw)?;

        anyhow::ensure!(raw.access_flag & 0x0008 == 0, "Field is static");

        let offset = raw.offset_or_value;
        let value = instance.read_field(context, offset)?;

        Ok(value as _)
    }

    pub fn write_value(&self, context: &mut KtfJavaContext<'_>, instance: JavaClassInstance, value: JavaWord) -> JavaResult<()> {
        let raw: RawJavaField = read_generic(context.core, self.ptr_raw)?;

        anyhow::ensure!(raw.access_flag & 0x0008 == 0, "Field is static");

        let offset = raw.offset_or_value;
        instance.write_field(context, offset, value as _)
    }

    pub fn read_static_value(&self, context: &KtfJavaContext<'_>) -> JavaResult<JavaWord> {
        let raw: RawJavaField = read_generic(context.core, self.ptr_raw)?;

        anyhow::ensure!(raw.access_flag & 0x0008 != 0, "Field is not static");

        Ok(raw.offset_or_value as _)
    }

    pub fn write_static_value(&self, context: &mut KtfJavaContext<'_>, value: JavaWord) -> JavaResult<()> {
        let mut raw: RawJavaField = read_generic(context.core, self.ptr_raw)?;

        anyhow::ensure!(raw.access_flag & 0x0008 != 0, "Field is not static");

        raw.offset_or_value = value as _;

        write_generic(context.core, self.ptr_raw, raw)
    }
}
