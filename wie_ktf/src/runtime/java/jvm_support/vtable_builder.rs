use alloc::{borrow::ToOwned, string::String, vec::Vec};

use jvm::JvmResult;

use super::class_definition::JavaClassDefinition;

struct JavaVtableMethod {
    ptr_method: u32,
    name: String,
    descriptor: String,
}

pub struct JavaVtableBuilder {
    items: Vec<JavaVtableMethod>,
}

impl JavaVtableBuilder {
    pub fn new(parent_class: &Option<JavaClassDefinition>) -> JvmResult<Self> {
        let items = if let Some(x) = parent_class {
            Self::build_vtable(x)?
        } else {
            Vec::new()
        };

        Ok(Self { items })
    }

    pub fn add(&mut self, ptr_method: u32, name: &str, descriptor: &str) -> usize {
        if let Some(index) = self.items.iter().position(|x| x.name == name && x.descriptor == descriptor) {
            self.items[index] = JavaVtableMethod {
                ptr_method,
                name: name.to_owned(),
                descriptor: descriptor.to_owned(),
            };

            index
        } else {
            self.items.push(JavaVtableMethod {
                ptr_method,
                name: name.to_owned(),
                descriptor: descriptor.to_owned(),
            });

            self.items.len() - 1
        }
    }

    pub fn serialize(&self) -> Vec<u32> {
        self.items.iter().map(|x| x.ptr_method).collect()
    }

    fn build_vtable(class: &JavaClassDefinition) -> JvmResult<Vec<JavaVtableMethod>> {
        let class_hierarchy = class.read_class_hierarchy()?.into_iter().rev();

        let mut vtable: Vec<JavaVtableMethod> = Vec::new();

        for class in class_hierarchy {
            let methods = class.methods()?;

            let items = methods
                .into_iter()
                .map(|x| {
                    let name = x.name()?;

                    anyhow::Ok(JavaVtableMethod {
                        ptr_method: x.ptr_raw,
                        name: name.name,
                        descriptor: name.descriptor,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            for item in items {
                if let Some(index) = vtable.iter().position(|x| x.name == item.name && x.descriptor == item.descriptor) {
                    vtable[index] = item;
                } else {
                    vtable.push(item);
                }
            }
        }

        Ok(vtable)
    }
}
