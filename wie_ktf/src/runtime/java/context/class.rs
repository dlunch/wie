use alloc::{string::String, vec, vec::Vec};
use core::mem::size_of;
use wie_base::util::{
    read_generic, read_null_terminated_string, read_null_terminated_table, write_generic, write_null_terminated_string, write_null_terminated_table,
};

use bytemuck::{Pod, Zeroable};

use wie_core_arm::Allocator;
use wie_impl_java::{JavaClassProto, JavaFieldAccessFlag, JavaResult, JavaWord};

use super::{context_data::JavaContextData, field::JavaField, method::JavaMethod, vtable_builder::JavaVtableBuilder, JavaFullName, KtfJavaContext};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RawJavaClass {
    ptr_next: u32,
    unk1: u32,
    ptr_descriptor: u32,
    ptr_vtable: u32,
    vtable_count: u16,
    unk_flag: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RawJavaClassDescriptor {
    ptr_name: u32,
    unk1: u32,
    ptr_parent_class: u32,
    ptr_methods: u32,
    ptr_interfaces: u32,
    ptr_fields_or_element_type: u32, // for array class, this is element type
    method_count: u16,
    fields_size: u16,
    access_flag: u16,
    unk6: u16,
    unk7: u16,
    unk8: u16,
}

pub struct JavaClass {
    pub(crate) ptr_raw: u32,
}

impl JavaClass {
    pub fn from_raw(ptr_raw: u32) -> Self {
        Self { ptr_raw }
    }

    pub async fn new(context: &mut KtfJavaContext<'_>, name: &str, proto: JavaClassProto) -> JavaResult<Self> {
        let parent_class = if let Some(x) = proto.parent_class {
            Some(context.load_class(x).await?.unwrap())
        } else {
            None
        };

        let mut vtable_builder = JavaVtableBuilder::new(context, &parent_class)?;

        let ptr_raw = Allocator::alloc(context.core, size_of::<RawJavaClass>() as u32)?;

        let mut methods = Vec::new();
        for method in proto.methods.into_iter() {
            let method = JavaMethod::new(context, ptr_raw, method, &mut vtable_builder)?;

            methods.push(method.ptr_raw);
        }
        let ptr_methods = Allocator::alloc(context.core, ((methods.len() + 1) * size_of::<u32>()) as _)?;
        write_null_terminated_table(context.core, ptr_methods, &methods)?;

        let mut fields = Vec::new();
        for (index, field) in proto.fields.into_iter().enumerate() {
            let offset_or_value = if field.access_flag == JavaFieldAccessFlag::STATIC {
                0
            } else {
                (index as u32) * 4
            };

            let field = JavaField::new(context, ptr_raw, field, offset_or_value)?;

            fields.push(field.ptr_raw);
        }
        let ptr_fields = Allocator::alloc(context.core, ((fields.len() + 1) * size_of::<u32>()) as _)?;
        write_null_terminated_table(context.core, ptr_fields, &fields)?;

        let ptr_name = Allocator::alloc(context.core, (name.len() + 1) as u32)?;
        write_null_terminated_string(context.core, ptr_name, name)?;

        let ptr_descriptor = Allocator::alloc(context.core, size_of::<RawJavaClassDescriptor>() as u32)?;
        write_generic(
            context.core,
            ptr_descriptor,
            RawJavaClassDescriptor {
                ptr_name,
                unk1: 0,
                ptr_parent_class: parent_class.map(|x| x.ptr_raw).unwrap_or(0),
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
        let ptr_vtable = Allocator::alloc(context.core, ((vtable.len() + 1) * size_of::<u32>()) as _)?;
        write_null_terminated_table(context.core, ptr_vtable, &vtable)?;

        write_generic(
            context.core,
            ptr_raw,
            RawJavaClass {
                ptr_next: ptr_raw + 4,
                unk1: 0,
                ptr_descriptor,
                ptr_vtable,
                vtable_count: vtable.len() as u16,
                unk_flag: 8,
            },
        )?;

        let result = Self::from_raw(ptr_raw);

        context.register_class(&result).await?;

        Ok(result)
    }

    pub async fn new_array(context: &mut KtfJavaContext<'_>, name: &str) -> JavaResult<JavaClass> {
        let ptr_parent_class = context.load_class("java/lang/Object").await?.unwrap();
        let ptr_raw = Allocator::alloc(context.core, size_of::<RawJavaClass>() as u32)?;

        let element_type_name = &name[1..];
        let element_type = if element_type_name.starts_with('L') {
            Some(context.load_class(&element_type_name[1..element_type_name.len() - 1]).await?.unwrap())
        } else {
            None
        };

        let ptr_name = Allocator::alloc(context.core, (name.len() + 1) as u32)?;
        write_null_terminated_string(context.core, ptr_name, name)?;

        let ptr_descriptor = Allocator::alloc(context.core, size_of::<RawJavaClassDescriptor>() as u32)?;
        write_generic(
            context.core,
            ptr_descriptor,
            RawJavaClassDescriptor {
                ptr_name,
                unk1: 0,
                ptr_parent_class: ptr_parent_class.ptr_raw,
                ptr_methods: 0,
                ptr_interfaces: 0,
                ptr_fields_or_element_type: element_type.map(|x| x.ptr_raw).unwrap_or(0),
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
            ptr_raw,
            RawJavaClass {
                ptr_next: ptr_raw + 4,
                unk1: 0,
                ptr_descriptor,
                ptr_vtable: 0,
                vtable_count: 0,
                unk_flag: 8,
            },
        )?;

        let class = JavaClass::from_raw(ptr_raw);

        JavaContextData::register_class(context, &class)?;

        Ok(class)
    }

    pub fn read_class_hierarchy(&self, context: &KtfJavaContext<'_>) -> JavaResult<Vec<JavaClass>> {
        let mut result = vec![];

        let mut current_class = self.ptr_raw;
        loop {
            result.push(JavaClass::from_raw(current_class));

            let raw: RawJavaClass = read_generic(context.core, current_class)?;
            let descriptor: RawJavaClassDescriptor = read_generic(context.core, raw.ptr_descriptor)?;

            if descriptor.ptr_parent_class != 0 {
                current_class = descriptor.ptr_parent_class;
            } else {
                break;
            }
        }

        Ok(result)
    }

    pub fn ptr_vtable(&self, context: &KtfJavaContext<'_>) -> JavaResult<u32> {
        let raw: RawJavaClass = read_generic(context.core, self.ptr_raw)?;

        Ok(raw.ptr_vtable)
    }

    pub fn field_size(&self, context: &KtfJavaContext<'_>) -> JavaResult<JavaWord> {
        let class_hierarchy = self.read_class_hierarchy(context)?;

        Ok(class_hierarchy
            .into_iter()
            .map(|x| {
                let raw: RawJavaClass = read_generic(context.core, x.ptr_raw).unwrap();
                let descriptor: RawJavaClassDescriptor = read_generic(context.core, raw.ptr_descriptor).unwrap();

                descriptor.fields_size as JavaWord
            })
            .sum())
    }

    pub fn methods(&self, context: &KtfJavaContext<'_>) -> JavaResult<Vec<JavaMethod>> {
        let raw: RawJavaClass = read_generic(context.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(context.core, raw.ptr_descriptor)?;

        let ptr_methods = read_null_terminated_table(context.core, descriptor.ptr_methods)?;

        Ok(ptr_methods.into_iter().map(JavaMethod::from_raw).collect())
    }

    pub fn fields(&self, context: &KtfJavaContext<'_>) -> JavaResult<Vec<JavaField>> {
        let raw: RawJavaClass = read_generic(context.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(context.core, raw.ptr_descriptor)?;

        let ptr_fields = read_null_terminated_table(context.core, descriptor.ptr_fields_or_element_type)?;

        Ok(ptr_fields.into_iter().map(JavaField::from_raw).collect())
    }

    pub fn name(&self, context: &KtfJavaContext<'_>) -> JavaResult<String> {
        let raw: RawJavaClass = read_generic(context.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(context.core, raw.ptr_descriptor)?;

        read_null_terminated_string(context.core, descriptor.ptr_name)
    }

    pub fn parent_class(&self, context: &KtfJavaContext<'_>) -> JavaResult<Option<JavaClass>> {
        let raw: RawJavaClass = read_generic(context.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(context.core, raw.ptr_descriptor)?;

        if descriptor.ptr_parent_class != 0 {
            Ok(Some(JavaClass::from_raw(descriptor.ptr_parent_class)))
        } else {
            Ok(None)
        }
    }

    pub fn method(&self, context: &KtfJavaContext<'_>, fullname: &JavaFullName) -> JavaResult<Option<JavaMethod>> {
        let methods = self.methods(context)?;

        for method in methods {
            if method.name(context)? == *fullname {
                return Ok(Some(method));
            }
        }

        if let Some(x) = self.parent_class(context)? {
            x.method(context, fullname)
        } else {
            Ok(None)
        }
    }

    pub fn field(&self, context: &KtfJavaContext<'_>, name: &str) -> JavaResult<Option<JavaField>> {
        let fields = self.fields(context)?;

        for field in fields {
            if field.name(context)?.name == name {
                return Ok(Some(field));
            }
        }

        Ok(None)
    }
}
