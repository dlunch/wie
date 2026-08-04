use alloc::{format, vec::Vec};
use core::mem::size_of;

use java_constants::MethodAccessFlags;
use jvm::Method;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{WieError, read_generic, write_generic};

use crate::runtime::{SVC_CATEGORY_JAVA_VTABLE, java::abi::JavaAbi};

use super::{JavaMethod, Result};

#[derive(Clone)]
pub struct JavaVtableEntry {
    pub target: u32,
    pub method: Option<JavaMethod>,
}

pub struct JavaVtable;

impl JavaVtable {
    const FIXED_ENTRY_COUNT: usize = 35;

    fn fixed_index(abi: &JavaAbi, class_name: &str, super_class_name: Option<&str>, name: &str, descriptor: &str) -> Option<usize> {
        if (matches!(class_name, "org/kwis/msp/lcdui/Jlet" | "org/kwis/msp/lcdui/JletWrapper")
            || matches!(super_class_name, Some("org/kwis/msp/lcdui/Jlet" | "org/kwis/msp/lcdui/JletWrapper")))
            && let Some(index) = abi.vtable_index("org/kwis/msp/lcdui/Jlet", name, descriptor)
        {
            return Some(index);
        }

        abi.vtable_index(class_name, name, descriptor)
    }

    pub fn allocate(core: &mut ArmCore, ptr_class: u32, entries: &[JavaVtableEntry]) -> Result<u32> {
        let ptr_allocation = Allocator::alloc(core, ((entries.len() + 2) * size_of::<u32>()) as u32)?;
        let ptr_vtable = ptr_allocation + size_of::<u32>() as u32;
        Self::write(core, ptr_vtable, ptr_class, entries)?;
        Ok(ptr_vtable)
    }

    pub fn read(core: &ArmCore, ptr_vtable: u32, known_methods: &[JavaMethod]) -> Result<Vec<JavaVtableEntry>> {
        let entry_count: u32 = read_generic(core, ptr_vtable - size_of::<u32>() as u32)?;
        (0..entry_count as usize)
            .map(|index| {
                let target = read_generic(core, ptr_vtable + ((index + 1) * size_of::<u32>()) as u32)?;
                let method = known_methods
                    .iter()
                    .find(|method| method.target().is_ok_and(|method_target| method_target == target))
                    .cloned();
                Ok(JavaVtableEntry { target, method })
            })
            .collect()
    }

    pub fn write(core: &mut ArmCore, ptr_vtable: u32, ptr_class: u32, entries: &[JavaVtableEntry]) -> Result<()> {
        write_generic(core, ptr_vtable - size_of::<u32>() as u32, entries.len() as u32)?;
        write_generic(core, ptr_vtable, ptr_class)?;
        for (index, entry) in entries.iter().enumerate() {
            let target = if entry.target == 0 {
                core.make_svc_stub(SVC_CATEGORY_JAVA_VTABLE, index as u32)?
            } else {
                entry.target
            };
            write_generic(core, ptr_vtable + ((index + 1) * size_of::<u32>()) as u32, target)?;
        }

        Ok(())
    }

    pub fn build_runtime_methods(
        class_name: &str,
        super_class_name: Option<&str>,
        parent_methods: &[JavaVtableEntry],
        declared_methods: &[JavaMethod],
    ) -> Result<Vec<JavaVtableEntry>> {
        let abi = JavaAbi::parse();
        let mut methods = parent_methods.to_vec();
        if class_name == "java/lang/Object" {
            methods.resize(Self::FIXED_ENTRY_COUNT, JavaVtableEntry { target: 0, method: None });
        }

        for method in declared_methods {
            let flags = method.access_flags();
            if flags.intersects(MethodAccessFlags::STATIC | MethodAccessFlags::PRIVATE) || method.name().starts_with('<') {
                continue;
            }

            let index = Self::fixed_index(&abi, class_name, super_class_name, &method.name(), &method.descriptor());
            if let Some(index) = index {
                if methods.len() <= index {
                    methods.resize(index + 1, JavaVtableEntry { target: 0, method: None });
                }
                let inherited = &methods[index];
                if inherited
                    .method
                    .as_ref()
                    .is_some_and(|inherited| inherited.name() != method.name() || inherited.descriptor() != method.descriptor())
                {
                    return Err(WieError::FatalError(format!(
                        "Fixed vtable index collision for {class_name}.{}{} at index {index}",
                        method.name(),
                        method.descriptor()
                    )));
                }
                methods[index] = JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                };
            }
        }

        for method in declared_methods {
            let flags = method.access_flags();
            if flags.intersects(MethodAccessFlags::STATIC | MethodAccessFlags::PRIVATE) || method.name().starts_with('<') {
                continue;
            }

            if Self::fixed_index(&abi, class_name, super_class_name, &method.name(), &method.descriptor()).is_some() {
                continue;
            }

            if let Some(index) = methods.iter().position(|candidate| {
                candidate
                    .method
                    .as_ref()
                    .is_some_and(|candidate| candidate.name() == method.name() && candidate.descriptor() == method.descriptor())
            }) {
                methods[index] = JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                };
            } else {
                methods.push(JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                });
            }
        }

        Ok(methods)
    }

    pub fn build_generated_methods(
        class_name: &str,
        super_class_name: Option<&str>,
        is_jlet_subclass: bool,
        is_card_subclass: bool,
        parent_methods: &[JavaVtableEntry],
        declared_methods: &[JavaMethod],
    ) -> Result<Vec<JavaVtableEntry>> {
        let abi = JavaAbi::parse();
        let mut methods = parent_methods.to_vec();

        for method in declared_methods {
            let flags = method.access_flags();
            if flags.intersects(MethodAccessFlags::STATIC | MethodAccessFlags::PRIVATE) || method.name().starts_with('<') {
                continue;
            }

            let name = method.name();
            let descriptor = method.descriptor();
            let inherited_index = methods.iter().position(|candidate| {
                candidate
                    .method
                    .as_ref()
                    .is_some_and(|candidate| candidate.name() == name && candidate.descriptor() == descriptor)
            });
            let abi_class = if is_jlet_subclass {
                Some("org/kwis/msp/lcdui/Jlet")
            } else if is_card_subclass {
                Some("org/kwis/msp/lcdui/Card")
            } else {
                None
            };
            let confirmed_index = abi_class.and_then(|class_name| abi.vtable_index(class_name, &name, &descriptor));

            if let Some(index) = inherited_index {
                if confirmed_index != Some(index) {
                    return Err(WieError::FatalError(format!(
                        "Unknown generated override {class_name}.{name}{descriptor}"
                    )));
                }
                methods[index] = JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                };
            } else if let Some(index) = confirmed_index
                && super_class_name == Some("org/kwis/msp/lcdui/Jlet")
            {
                if methods.len() <= index {
                    methods.resize(index + 1, JavaVtableEntry { target: 0, method: None });
                }
                if methods[index].method.is_some() {
                    return Err(WieError::FatalError(format!(
                        "Fixed vtable index collision for {class_name}.{name}{descriptor} at index {index}"
                    )));
                }
                methods[index] = JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                };
            } else if confirmed_index.is_some() {
                return Err(WieError::FatalError(format!(
                    "Unknown generated override {class_name}.{name}{descriptor}"
                )));
            }
        }

        Ok(methods)
    }
}
