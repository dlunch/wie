use alloc::{string::String, vec::Vec};

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{read_null_terminated_table, write_null_terminated_table};

use super::{JavaMethod, Result, class_definition::JavaClassDefinition};

struct JavaVtableMethod {
    method: JavaMethod,
    name: String,
    descriptor: String,
}

pub struct JavaVtable {
    pub(crate) ptr_raw: u32,
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

    pub fn len(&self) -> Result<usize> {
        let items = read_null_terminated_table(&self.core, self.ptr_raw)?;

        Ok(items.len())
    }

    fn build_vtable(class: &JavaClassDefinition) -> Result<Vec<JavaVtableMethod>> {
        let class_hierarchy = class.read_class_hierarchy()?.into_iter().rev();

        let mut vtable: Vec<JavaVtableMethod> = Vec::new();

        for class in class_hierarchy {
            let methods = class.methods()?;

            let items = methods
                .into_iter()
                .map(|x| {
                    let name = x.name()?;

                    Ok(JavaVtableMethod {
                        method: x,
                        name: name.name,
                        descriptor: name.descriptor,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            for item in items {
                let index = if let Some(index) = vtable.iter().position(|x| x.name == item.name && x.descriptor == item.descriptor) {
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
