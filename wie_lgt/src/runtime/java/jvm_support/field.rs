use alloc::string::String;
use core::fmt::{self, Debug, Formatter};

use java_class_proto::JavaFieldProto;
use java_constants::FieldAccessFlags;
use jvm::Field;
use wipi_types::lgt::java::LgtJavaClassField as RawJavaField;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes};

use super::{JavaClassDefinition, Result};

#[derive(Clone)]
pub struct JavaField {
    pub ptr_raw: u32,
    core: ArmCore,
}

impl JavaField {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn new(core: &mut ArmCore, ptr_raw: u32, ptr_class: u32, proto: JavaFieldProto, slot: u32) -> Result<Self> {
        let ptr_name = Allocator::alloc(core, (proto.name.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_name, proto.name.as_bytes())?;

        let ptr_descriptor = Allocator::alloc(core, (proto.descriptor.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_descriptor, proto.descriptor.as_bytes())?;

        write_generic(
            core,
            ptr_raw,
            RawJavaField {
                ptr_class,
                ptr_name,
                ptr_descriptor,
                flags: proto.access_flags.bits(),
                unk2: 0,
                slot,
            },
        )?;

        Ok(Self::from_raw(ptr_raw, core))
    }

    fn raw(&self) -> Result<RawJavaField> {
        read_generic(&self.core, self.ptr_raw)
    }

    pub fn slot(&self) -> Result<u32> {
        Ok(self.raw()?.slot)
    }

    pub fn static_address(&self) -> Result<u32> {
        let raw = self.raw()?;
        let class = JavaClassDefinition::from_raw(raw.ptr_class, &self.core);

        Ok(class.ptr_static_fields()? + raw.slot * 4)
    }
}

impl Field for JavaField {
    fn name(&self) -> String {
        String::from_utf8(read_null_terminated_string_bytes(&self.core, self.raw().unwrap().ptr_name).unwrap()).unwrap()
    }

    fn descriptor(&self) -> String {
        String::from_utf8(read_null_terminated_string_bytes(&self.core, self.raw().unwrap().ptr_descriptor).unwrap()).unwrap()
    }

    fn access_flags(&self) -> FieldAccessFlags {
        FieldAccessFlags::from_bits_truncate(self.raw().unwrap().flags as _)
    }
}

impl Debug for JavaField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaField").field("ptr_raw", &self.ptr_raw).finish()
    }
}
