use alloc::vec::Vec;
use core::mem::size_of;

use java_constants::MethodAccessFlags;
use jvm::Method;

use wie_core_arm::ArmCore;
use wie_util::write_generic;

use super::{JavaMethod, Result};

pub struct JavaVtable;

impl JavaVtable {
    pub fn write(core: &mut ArmCore, ptr_raw: u32, ptr_class: u32, methods: &[JavaMethod]) -> Result<()> {
        write_generic(core, ptr_raw, ptr_class)?;
        for (index, method) in methods.iter().enumerate() {
            write_generic(core, ptr_raw + ((index + 1) * size_of::<u32>()) as u32, method.target()?)?;
        }

        Ok(())
    }

    pub fn build_methods(parent_methods: &[JavaMethod], declared_methods: &[JavaMethod]) -> Vec<JavaMethod> {
        let mut methods = parent_methods.to_vec();

        for method in declared_methods {
            let flags = method.access_flags();
            if flags.intersects(MethodAccessFlags::STATIC | MethodAccessFlags::PRIVATE) || method.name().starts_with('<') {
                continue;
            }

            if let Some(index) = methods
                .iter()
                .position(|x| x.name() == method.name() && x.descriptor() == method.descriptor())
            {
                methods[index] = method.clone();
            } else {
                methods.push(method.clone());
            }
        }

        methods
    }
}
