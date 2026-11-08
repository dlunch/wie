use alloc::{format, string::String, vec::Vec};
use core::mem::size_of;

use jvm::{ClassDefinition, Jvm, Method};
use jvm_types::MethodAccessFlags;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{Result, WieError, read_generic, write_generic};

use crate::runtime::{SVC_CATEGORY_MISSING_JAVA_VTABLE_ENTRY, java::abi::JAVA_ABI};

use super::{JavaClassDefinition, JavaMethod};

#[derive(Clone)]
pub struct JavaVtableEntry {
    pub target: u32,
    pub method: Option<JavaMethod>,
}

pub struct JavaVtable;

impl JavaVtable {
    pub fn allocate(core: &mut ArmCore, entry_count: usize) -> Result<u32> {
        let allocation = Allocator::alloc(core, ((entry_count + 2) * size_of::<u32>()) as u32)?;
        write_generic(core, allocation + ((entry_count + 1) * size_of::<u32>()) as u32, 0u32)?;
        Ok(allocation)
    }

    pub fn read(core: &ArmCore, ptr_vtable: u32, entry_count: usize, known_classes: &[(String, Vec<JavaMethod>)]) -> Result<Vec<JavaVtableEntry>> {
        (0..entry_count)
            .map(|index| {
                let target = read_generic(core, ptr_vtable + ((index + 1) * size_of::<u32>()) as u32)?;
                let method = known_classes
                    .iter()
                    .flat_map(|(_, methods)| methods)
                    .find(|method| method.target().is_ok_and(|method_target| method_target == target))
                    .cloned()
                    .or_else(|| {
                        known_classes.iter().find_map(|(class_name, methods)| {
                            let entry = JAVA_ABI.class(class_name)?.vtable.iter().find(|entry| entry.index == index)?;
                            methods
                                .iter()
                                .find(|method| method.name() == entry.name && method.descriptor() == entry.descriptor)
                                .cloned()
                        })
                    });
                Ok(JavaVtableEntry { target, method })
            })
            .collect()
    }

    pub fn write(core: &mut ArmCore, ptr_vtable: u32, ptr_class: u32, entries: &[JavaVtableEntry]) -> Result<()> {
        write_generic(core, ptr_vtable, ptr_class)?;
        for (index, entry) in entries.iter().enumerate() {
            let target = if entry.target == 0 {
                core.make_svc_stub(SVC_CATEGORY_MISSING_JAVA_VTABLE_ENTRY, index as u32)?
            } else {
                entry.target
            };
            write_generic(core, ptr_vtable + ((index + 1) * size_of::<u32>()) as u32, target)?;
        }

        Ok(())
    }

    pub fn build_from_compiler_vtable(
        core: &ArmCore,
        ptr_vtable: u32,
        entry_count: usize,
        parent_methods: &[JavaVtableEntry],
        declared_methods: &[JavaMethod],
    ) -> Result<Vec<JavaVtableEntry>> {
        (0..entry_count)
            .map(|index| {
                let target = read_generic(core, ptr_vtable + ((index + 1) * size_of::<u32>()) as u32)?;
                if target == 0 {
                    return Ok(parent_methods.get(index).cloned().unwrap_or(JavaVtableEntry { target: 0, method: None }));
                }

                let method = declared_methods
                    .iter()
                    .chain(parent_methods.iter().filter_map(|entry| entry.method.as_ref()))
                    .find(|method| method.target().is_ok_and(|method_target| method_target == target))
                    .cloned();
                Ok(JavaVtableEntry { target, method })
            })
            .collect()
    }

    pub fn build_interface_methods(declared_methods: &[JavaMethod]) -> Result<Vec<JavaVtableEntry>> {
        declared_methods
            .iter()
            .filter(|method| {
                !method.access_flags().intersects(MethodAccessFlags::STATIC | MethodAccessFlags::PRIVATE) && !method.name().starts_with('<')
            })
            .map(|method| {
                Ok(JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                })
            })
            .collect()
    }

    pub async fn build_methods(
        jvm: &Jvm,
        class_name: &str,
        parent_class: Option<&JavaClassDefinition>,
        declared_methods: &[JavaMethod],
    ) -> Result<Vec<JavaVtableEntry>> {
        let mut methods = if let Some(parent_class) = parent_class {
            parent_class.vtable_entries(jvm).await?
        } else {
            Vec::new()
        };
        let mut abi_classes = JAVA_ABI.class(class_name).into_iter().collect::<Vec<_>>();
        let mut ancestor = parent_class.cloned();
        while let Some(class) = ancestor {
            let name = ClassDefinition::name(&class);
            if let Some(class) = JAVA_ABI.class(&name) {
                abi_classes.push(class);
            }
            ancestor = ClassDefinition::super_class_name(&class).map(|name| {
                jvm.get_class(&name)
                    .unwrap()
                    .definition
                    .as_any()
                    .downcast_ref::<JavaClassDefinition>()
                    .unwrap()
                    .clone()
            });
        }
        let minimum_size = abi_classes
            .iter()
            .flat_map(|class| class.vtable_size.into_iter().chain(class.vtable.iter().map(|entry| entry.index + 1)))
            .max()
            .unwrap_or(0);
        if JAVA_ABI.class(class_name).is_some() {
            // Runtime superclasses can contain methods outside the native API layout.
            let inherited = core::mem::take(&mut methods);
            methods.resize(minimum_size, JavaVtableEntry { target: 0, method: None });
            for entry in inherited {
                if let Some(method) = &entry.method
                    && let Some(index) = abi_classes
                        .iter()
                        .find_map(|class| class.vtable_index(&method.name(), &method.descriptor()))
                {
                    methods[index] = entry;
                }
            }
        } else if methods.len() < minimum_size {
            methods.resize(minimum_size, JavaVtableEntry { target: 0, method: None });
        }

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
            let confirmed_index = abi_classes.iter().find_map(|class| class.vtable_index(&name, &descriptor));

            let index = if let Some(index) = confirmed_index {
                if inherited_index.is_some_and(|inherited_index| inherited_index != index) {
                    return Err(WieError::FatalError(format!(
                        "Inherited vtable index does not match LGT Java ABI for {class_name}.{name}{descriptor}"
                    )));
                }
                if methods[index]
                    .method
                    .as_ref()
                    .is_some_and(|method| method.name() != name || method.descriptor() != descriptor)
                {
                    return Err(WieError::FatalError(format!(
                        "Fixed vtable index collision for {class_name}.{name}{descriptor} at index {index}"
                    )));
                }
                index
            } else if let Some(index) = inherited_index {
                index
            } else {
                methods.push(JavaVtableEntry { target: 0, method: None });
                methods.len() - 1
            };

            methods[index] = JavaVtableEntry {
                target: method.target()?,
                method: Some(method.clone()),
            };
        }

        Ok(methods)
    }
}
