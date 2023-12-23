use alloc::{borrow::ToOwned, vec::Vec};

use wie_impl_java::JavaResult;

use super::{class::JavaClass, name::JavaFullName};

struct JavaVtableMethod {
    ptr_method: u32,
    name: JavaFullName,
}

pub struct JavaVtableBuilder {
    items: Vec<JavaVtableMethod>,
}

impl JavaVtableBuilder {
    pub fn new(parent_class: &Option<JavaClass>) -> JavaResult<Self> {
        let items = if let Some(x) = parent_class {
            Self::build_vtable(x)?
        } else {
            Vec::new()
        };

        Ok(Self { items })
    }

    pub fn add(&mut self, ptr_method: u32, name: &JavaFullName) -> usize {
        if let Some(index) = self.items.iter().position(|x| x.name == *name) {
            self.items[index] = JavaVtableMethod {
                ptr_method,
                name: name.to_owned(),
            };

            index
        } else {
            self.items.push(JavaVtableMethod {
                ptr_method,
                name: name.to_owned(),
            });

            self.items.len() - 1
        }
    }

    pub fn serialize(&self) -> Vec<u32> {
        self.items.iter().map(|x| x.ptr_method).collect()
    }

    fn build_vtable(class: &JavaClass) -> JavaResult<Vec<JavaVtableMethod>> {
        let class_hierarchy = class.read_class_hierarchy()?.into_iter().rev();

        let mut vtable: Vec<JavaVtableMethod> = Vec::new();

        for class in class_hierarchy {
            let methods = class.methods()?;

            let items = methods
                .into_iter()
                .map(|x| {
                    anyhow::Ok(JavaVtableMethod {
                        ptr_method: x.ptr_raw,
                        name: x.name()?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            for item in items {
                if let Some(index) = vtable.iter().position(|x| x.name == item.name) {
                    vtable[index] = item;
                } else {
                    vtable.push(item);
                }
            }
        }

        Ok(vtable)
    }
}
