use alloc::vec::Vec;
use core::mem::size_of;

use java_constants::MethodAccessFlags;
use jvm::Method;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::write_generic;

use super::{JavaClassDefinition, JavaMethod, Result};

pub struct JavaVtable {
    pub ptr_raw: u32,
    pub methods: Vec<JavaMethod>,
}

impl JavaVtable {
    pub fn new(core: &mut ArmCore, ptr_class: u32, parent_class: Option<&JavaClassDefinition>, declared_methods: &[JavaMethod]) -> Result<Self> {
        let mut methods = parent_class.map(|x| x.virtual_methods().to_vec()).unwrap_or_default();

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

        let ptr_raw = Allocator::alloc(core, ((methods.len() + 1) * size_of::<u32>()) as u32)?;
        write_generic(core, ptr_raw, ptr_class)?;
        for (index, method) in methods.iter().enumerate() {
            write_generic(core, ptr_raw + ((index + 1) * size_of::<u32>()) as u32, method.target()?)?;
        }

        Ok(Self { ptr_raw, methods })
    }
}
