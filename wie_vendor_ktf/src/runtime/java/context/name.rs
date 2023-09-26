use alloc::{string::String, vec::Vec};
use core::fmt::Display;
use wie_core_arm::ArmCore;

use wie_base::util::{read_generic, read_null_terminated_string};
use wie_wipi_java::JavaResult;

#[derive(Clone)]
pub struct JavaFullName {
    pub tag: u8,
    pub name: String,
    pub signature: String,
}

impl JavaFullName {
    pub fn from_ptr(core: &ArmCore, ptr: u32) -> JavaResult<Self> {
        let tag = read_generic(core, ptr)?;

        let value = read_null_terminated_string(core, ptr + 1)?;
        let value = value.split('+').collect::<Vec<_>>();

        Ok(JavaFullName {
            tag,
            name: value[1].into(),
            signature: value[0].into(),
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.tag);
        bytes.extend_from_slice(self.signature.as_bytes());
        bytes.push(b'+');
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.push(0);

        bytes
    }
}

impl Display for JavaFullName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.name.fmt(f)?;
        self.signature.fmt(f)?;
        write!(f, "@{}", self.tag)?;

        Ok(())
    }
}

impl PartialEq for JavaFullName {
    fn eq(&self, other: &Self) -> bool {
        self.signature == other.signature && self.name == other.name
    }
}
