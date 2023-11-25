use alloc::{boxed::Box, vec::Vec};
use core::mem::size_of;

use wie_base::util::{write_generic, write_null_terminated_string};
use wie_core_arm::Allocator;
use wie_impl_java::{get_class_proto, JavaClassProto, JavaFieldAccessFlag, JavaResult};

use super::{field::JavaField, method::JavaMethod, vtable_builder::JavaVtableBuilder, JavaClass, JavaClassDescriptor, KtfJavaContext};

pub struct ClassLoader {}

impl ClassLoader {
    #[async_recursion::async_recursion(?Send)]
    pub async fn get_or_load_ptr_class(context: &mut KtfJavaContext<'_>, name: &str) -> JavaResult<u32> {
        let ptr_class = Self::get_ptr_class(context, name)?;

        if let Some(ptr_class) = ptr_class {
            Ok(ptr_class)
        } else {
            // array class is created dynamically
            if name.as_bytes()[0] == b'[' {
                let ptr_class = Self::load_array_class_into_vm(context, name).await?;

                Ok(ptr_class)
            } else {
                let proto = get_class_proto(name);
                if let Some(x) = proto {
                    Self::load_class_into_vm(context, name, x).await
                } else {
                    // find from client.bin
                    let fn_get_class = context.read_context_data()?.fn_get_class;

                    let ptr_name = Allocator::alloc(context.core, 50)?; // TODO size fix
                    write_null_terminated_string(context.core, ptr_name, name)?;

                    let ptr_class = context.core.run_function(fn_get_class, &[ptr_name]).await?;
                    Allocator::free(context.core, ptr_name)?;

                    if ptr_class != 0 {
                        context.register_class(ptr_class).await?;

                        Ok(ptr_class)
                    } else {
                        anyhow::bail!("Cannot find class {}", name);
                    }
                }
            }
        }
    }

    pub fn get_ptr_class(context: &KtfJavaContext<'_>, name: &str) -> JavaResult<Option<u32>> {
        let context_data = context.read_context_data()?;
        let ptr_classes = context.read_null_terminated_table(context_data.classes_base)?;
        for ptr_class in ptr_classes {
            let (_, _, class_name) = context.read_ptr_class(ptr_class)?;

            if class_name == name {
                return Ok(Some(ptr_class));
            }
        }

        Ok(None)
    }

    async fn load_array_class_into_vm(context: &mut KtfJavaContext<'_>, name: &str) -> JavaResult<u32> {
        let ptr_parent_class = Self::get_or_load_ptr_class(context, "java/lang/Object").await?;
        let ptr_class = Allocator::alloc(context.core, size_of::<JavaClass>() as u32)?;

        let element_type_name = &name[1..];
        let element_type = if element_type_name.starts_with('L') {
            Self::get_or_load_ptr_class(context, &element_type_name[1..element_type_name.len() - 1]).await?
        } else {
            0
        };

        let ptr_name = Allocator::alloc(context.core, (name.len() + 1) as u32)?;
        write_null_terminated_string(context.core, ptr_name, name)?;

        let ptr_descriptor = Allocator::alloc(context.core, size_of::<JavaClassDescriptor>() as u32)?;
        write_generic(
            context.core,
            ptr_descriptor,
            JavaClassDescriptor {
                ptr_name,
                unk1: 0,
                ptr_parent_class,
                ptr_methods: 0,
                ptr_interfaces: 0,
                ptr_fields_or_element_type: element_type,
                method_count: 0,
                fields_size: 0,
                access_flag: 0x21, // ACC_PUBLIC | ACC_SUPER
                unk6: 0,
                unk7: 0,
                unk8: 0,
            },
        )?;

        write_generic(
            context.core,
            ptr_class,
            JavaClass {
                ptr_next: ptr_class + 4,
                unk1: 0,
                ptr_descriptor,
                ptr_vtable: 0,
                vtable_count: 0,
                unk_flag: 8,
            },
        )?;

        let context_data = context.read_context_data()?;
        let ptr_classes = context.read_null_terminated_table(context_data.classes_base)?;
        write_generic(
            context.core,
            context_data.classes_base + (ptr_classes.len() * size_of::<u32>()) as u32,
            ptr_class,
        )?;

        Ok(ptr_class)
    }

    async fn load_class_into_vm(context: &mut KtfJavaContext<'_>, name: &str, proto: JavaClassProto) -> JavaResult<u32> {
        let ptr_parent_class = if let Some(x) = proto.parent_class {
            Some(Self::get_or_load_ptr_class(context, x).await?)
        } else {
            None
        };

        let mut vtable_builder = JavaVtableBuilder::new(context, ptr_parent_class)?;

        let ptr_class = Allocator::alloc(context.core, size_of::<JavaClass>() as u32)?;

        let mut methods = Vec::new();
        for method in proto.methods.into_iter() {
            let method = JavaMethod::new(context, ptr_class, method, &mut vtable_builder)?;

            methods.push(method.ptr_raw);
        }
        let ptr_methods = context.write_null_terminated_table(&methods)?;

        let mut fields = Vec::new();
        for (index, field) in proto.fields.into_iter().enumerate() {
            let offset_or_value = if field.access_flag == JavaFieldAccessFlag::STATIC {
                0
            } else {
                (index as u32) * 4
            };

            let field = JavaField::new(context, ptr_class, field, offset_or_value)?;

            fields.push(field.ptr_raw);
        }
        let ptr_fields = context.write_null_terminated_table(&fields)?;

        let ptr_name = Allocator::alloc(context.core, (name.len() + 1) as u32)?;
        write_null_terminated_string(context.core, ptr_name, name)?;

        let ptr_descriptor = Allocator::alloc(context.core, size_of::<JavaClassDescriptor>() as u32)?;
        write_generic(
            context.core,
            ptr_descriptor,
            JavaClassDescriptor {
                ptr_name,
                unk1: 0,
                ptr_parent_class: ptr_parent_class.unwrap_or(0),
                ptr_methods,
                ptr_interfaces: 0,
                ptr_fields_or_element_type: ptr_fields,
                method_count: methods.len() as u16,
                fields_size: (fields.len() * 4) as u16,
                access_flag: 0x21, // ACC_PUBLIC | ACC_SUPER
                unk6: 0,
                unk7: 0,
                unk8: 0,
            },
        )?;

        let vtable = vtable_builder.serialize();
        let ptr_vtable = context.write_null_terminated_table(&vtable)?;

        write_generic(
            context.core,
            ptr_class,
            JavaClass {
                ptr_next: ptr_class + 4,
                unk1: 0,
                ptr_descriptor,
                ptr_vtable,
                vtable_count: vtable.len() as u16,
                unk_flag: 8,
            },
        )?;

        context.register_class(ptr_class).await?;

        Ok(ptr_class)
    }
}
