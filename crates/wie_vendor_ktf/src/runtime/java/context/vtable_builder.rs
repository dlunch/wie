use alloc::{borrow::ToOwned, vec::Vec};

use wie_base::util::read_generic;
use wie_wipi_java::{JavaError, JavaResult};

use super::{name::JavaFullName, JavaMethod, KtfJavaContext};

struct JavaVtableMethod {
    ptr_method: u32,
    name: JavaFullName,
}

pub struct JavaVtableBuilder {
    items: Vec<JavaVtableMethod>,
}

impl JavaVtableBuilder {
    pub fn new(context: &KtfJavaContext<'_>, ptr_parent_class: Option<u32>) -> JavaResult<Self> {
        let items = if let Some(x) = ptr_parent_class {
            Self::build_vtable(context, x)?
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

    fn build_vtable(context: &KtfJavaContext<'_>, ptr_class: u32) -> JavaResult<Vec<JavaVtableMethod>> {
        let class_hierarchy = context.read_class_hierarchy(ptr_class)?.into_iter().rev();

        let mut vtable: Vec<JavaVtableMethod> = Vec::new();

        for class_descriptor in class_hierarchy {
            let ptr_methods = context.read_null_terminated_table(class_descriptor.ptr_methods)?;

            let items = ptr_methods
                .into_iter()
                .map(|x| {
                    let method: JavaMethod = read_generic(context.core, x)?;
                    let name = JavaFullName::from_ptr(context.core, method.ptr_name)?;

                    Ok::<_, JavaError>(JavaVtableMethod { ptr_method: x, name })
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
