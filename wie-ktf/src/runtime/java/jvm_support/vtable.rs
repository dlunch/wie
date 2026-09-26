use alloc::vec::Vec;

use jvm::Method;
use jvm_types::MethodAccessFlags;
use wipi_types::ktf::java::JavaClass as RawJavaClass;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{read_generic, read_null_terminated_table, write_generic, write_null_terminated_table};

use super::{JavaMethod, Result, class_definition::JavaClassDefinition, name::JavaFullName};

struct JavaVtableMethod {
    method: JavaMethod,
    name: JavaFullName,
}

pub struct JavaVtable {
    pub ptr_raw: u32,
    core: ArmCore,
}

impl JavaVtable {
    pub fn new(core: &mut ArmCore, class: &JavaClassDefinition) -> Result<Self> {
        let items = Self::build_vtable(class)?;

        let ptr_raw = Allocator::alloc(core, ((items.len() + 1) * size_of::<u32>()) as _)?;
        let ptr_methods = items.iter().map(|x| x.method.ptr_raw).collect::<Vec<_>>();
        write_null_terminated_table(core, ptr_raw, &ptr_methods)?;

        Ok(Self { ptr_raw, core: core.clone() })
    }

    pub fn from_raw(core: &ArmCore, ptr_raw: u32) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn find_method(&self, name: &str, descriptor: &str) -> Result<Option<JavaMethod>> {
        let items = read_null_terminated_table(&self.core, self.ptr_raw)?;

        for &ptr_method in &items {
            let method = JavaMethod::from_raw(ptr_method, &self.core);
            let method_name = method.name()?;

            if method_name.name() == name && method_name.descriptor() == descriptor {
                return Ok(Some(method));
            }
        }

        Ok(None)
    }

    pub fn len(&self) -> Result<usize> {
        let items = read_null_terminated_table(&self.core, self.ptr_raw)?;

        Ok(items.len())
    }

    pub(super) fn resolve_overrides(core: &mut ArmCore, class: &JavaClassDefinition) -> Result<()> {
        let raw: RawJavaClass = read_generic(core, class.ptr_raw)?;
        let hierarchy = class.read_class_hierarchy()?;

        // Native linking can leave an override in a separate slot. Keep the ABI's
        // indices and resolve each inherited virtual slot to its implementation.
        for index in 0..raw.vtable_count {
            let address = raw.ptr_vtable + u32::from(index) * size_of::<u32>() as u32;
            let ptr_method = read_generic(core, address)?;
            let mut method = JavaMethod::from_raw(ptr_method, core);
            let name = method.name()?;
            if method.access_flags().intersects(MethodAccessFlags::PRIVATE | MethodAccessFlags::STATIC) || name.name().starts_with('<') {
                continue;
            }

            let declaring_class = method.ptr_class();
            for subclass in hierarchy.iter().rev().skip_while(|class| class.ptr_raw != declaring_class).skip(1) {
                let Some(candidate) = subclass.method(name.name(), name.descriptor(), false)? else {
                    continue;
                };
                if candidate.access_flags().contains(MethodAccessFlags::PRIVATE) {
                    continue;
                }

                if !method.access_flags().intersects(MethodAccessFlags::PUBLIC | MethodAccessFlags::PROTECTED) {
                    let owner_name = JavaClassDefinition::from_raw(method.ptr_class(), core).name()?;
                    let subclass_name = subclass.name()?;
                    if owner_name.rsplit_once('/').map(|(package, _)| package) != subclass_name.rsplit_once('/').map(|(package, _)| package) {
                        continue;
                    }
                }
                method = candidate;
            }

            if method.ptr_raw != ptr_method {
                write_generic(core, address, method.ptr_raw)?;
            }
        }

        Ok(())
    }

    fn build_vtable(class: &JavaClassDefinition) -> Result<Vec<JavaVtableMethod>> {
        let class_hierarchy = class.read_class_hierarchy()?.into_iter().rev();

        let mut vtable: Vec<JavaVtableMethod> = Vec::new();

        for class in class_hierarchy {
            let methods = class.methods()?;

            let items = methods
                .map(|x| {
                    let name = x.name()?;

                    Ok(JavaVtableMethod { method: x, name })
                })
                .collect::<Result<Vec<_>>>()?;

            for item in items {
                let index = if let Some(index) = vtable.iter().position(|x| x.name == item.name) {
                    vtable[index] = item;

                    index
                } else {
                    vtable.push(item);

                    vtable.len() - 1
                };

                vtable[index].method.write_vtable_index(index as _)?;
            }
        }

        Ok(vtable)
    }
}
