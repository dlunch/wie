use alloc::{format, string::String, vec::Vec};
use core::mem::size_of;

use java_constants::MethodAccessFlags;
use jvm::{ClassDefinition, Jvm, Method};

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
        Allocator::alloc(core, ((entry_count + 1) * size_of::<u32>()) as u32)
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
        let minimum_size = abi_classes.iter().filter_map(|class| class.vtable_size).max().unwrap_or(0);
        if methods.len() < minimum_size {
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
                if methods.len() <= index {
                    methods.resize(index + 1, JavaVtableEntry { target: 0, method: None });
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
                continue;
            };

            methods[index] = JavaVtableEntry {
                target: method.target()?,
                method: Some(method.clone()),
            };
        }

        Ok(methods)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};
    use core::mem::size_of;

    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{Result, write_generic};

    use super::{JavaVtable, JavaVtableEntry};

    #[test]
    fn compiler_vtable_uses_parent_targets_for_empty_entries() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let compiler_targets = [0, 0x99, 0, 0x88];
        let ptr_vtable = Allocator::alloc(&mut core, ((compiler_targets.len() + 1) * size_of::<u32>()) as u32)?;
        write_generic(&mut core, ptr_vtable, 0x1234u32)?;
        for (index, target) in compiler_targets.iter().enumerate() {
            write_generic(&mut core, ptr_vtable + ((index + 1) * size_of::<u32>()) as u32, *target)?;
        }

        let parent_methods = [0x11, 0x22, 0x33, 0x44]
            .into_iter()
            .map(|target| JavaVtableEntry { target, method: None })
            .collect::<Vec<_>>();
        let entries = JavaVtable::build_from_compiler_vtable(&core, ptr_vtable, compiler_targets.len(), &parent_methods, &[])?;

        assert_eq!(entries.iter().map(|entry| entry.target).collect::<Vec<_>>(), vec![0x11, 0x99, 0x33, 0x88]);
        Ok(())
    }
}
