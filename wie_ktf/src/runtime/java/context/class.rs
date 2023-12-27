use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use jvm::{Class, ClassInstance, Field, JavaValue, JvmResult, Method};

use wie_base::util::{
    read_generic, read_null_terminated_string, read_null_terminated_table, write_generic, write_null_terminated_string, write_null_terminated_table,
};
use wie_core_arm::{Allocator, ArmCore};
use wie_impl_java::{JavaClassProto, JavaFieldAccessFlag, JavaResult, JavaWord};

use super::{
    class_instance::JavaClassInstance, class_loader::ClassLoader, field::JavaField, method::JavaMethod, value::JavaValueExt,
    vtable_builder::JavaVtableBuilder, JavaFullName, KtfJavaContext,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct RawJavaClass {
    pub(super) ptr_next: u32,
    pub(super) unk1: u32,
    pub(super) ptr_descriptor: u32,
    pub(super) ptr_vtable: u32,
    pub(super) vtable_count: u16,
    pub(super) unk_flag: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct RawJavaClassDescriptor {
    pub(super) ptr_name: u32,
    pub(super) unk1: u32,
    pub(super) ptr_parent_class: u32,
    pub(super) ptr_methods: u32,
    pub(super) ptr_interfaces: u32,
    pub(super) ptr_fields_or_element_type: u32, // for array class, this is element type
    pub(super) method_count: u16,
    pub(super) fields_size: u16,
    pub(super) access_flag: u16,
    pub(super) unk6: u16,
    pub(super) unk7: u16,
    pub(super) unk8: u16,
}

pub struct JavaClass {
    pub(crate) ptr_raw: u32,
    core: ArmCore,
}

impl JavaClass {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub async fn new(core: &mut ArmCore, name: &str, proto: JavaClassProto) -> JavaResult<Self> {
        let parent_class = if let Some(x) = proto.parent_class {
            Some(ClassLoader::get_or_load_class(core, x).await?.unwrap())
        } else {
            None
        };

        let mut vtable_builder = JavaVtableBuilder::new(&parent_class)?;

        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;

        let mut methods = Vec::new();
        for method in proto.methods.into_iter() {
            let method = JavaMethod::new(core, ptr_raw, method, &mut vtable_builder)?;

            methods.push(method.ptr_raw);
        }
        let ptr_methods = Allocator::alloc(core, ((methods.len() + 1) * size_of::<u32>()) as _)?;
        write_null_terminated_table(core, ptr_methods, &methods)?;

        let mut fields = Vec::new();
        for (index, field) in proto.fields.into_iter().enumerate() {
            let offset_or_value = if field.access_flag == JavaFieldAccessFlag::STATIC {
                0
            } else {
                (index as u32) * 4
            };

            let field = JavaField::new(core, ptr_raw, field, offset_or_value)?;

            fields.push(field.ptr_raw);
        }
        let ptr_fields = Allocator::alloc(core, ((fields.len() + 1) * size_of::<u32>()) as _)?;
        write_null_terminated_table(core, ptr_fields, &fields)?;

        let ptr_name = Allocator::alloc(core, (name.len() + 1) as u32)?;
        write_null_terminated_string(core, ptr_name, name)?;

        let ptr_descriptor = Allocator::alloc(core, size_of::<RawJavaClassDescriptor>() as u32)?;
        write_generic(
            core,
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
        let ptr_vtable = Allocator::alloc(core, ((vtable.len() + 1) * size_of::<u32>()) as _)?;
        write_null_terminated_table(core, ptr_vtable, &vtable)?;

        write_generic(
            core,
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

        let result = Self::from_raw(ptr_raw, core);

        KtfJavaContext::register_class(core, &result).await?;

        Ok(result)
    }

    pub fn read_class_hierarchy(&self) -> JavaResult<Vec<JavaClass>> {
        let mut result = vec![];

        let mut current_class = self.ptr_raw;
        loop {
            result.push(JavaClass::from_raw(current_class, &self.core));

            let raw: RawJavaClass = read_generic(&self.core, current_class)?;
            let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

            if descriptor.ptr_parent_class != 0 {
                current_class = descriptor.ptr_parent_class;
            } else {
                break;
            }
        }

        Ok(result)
    }

    pub fn ptr_vtable(&self) -> JavaResult<u32> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;

        Ok(raw.ptr_vtable)
    }

    pub fn field_size(&self) -> JavaResult<JavaWord> {
        let class_hierarchy = self.read_class_hierarchy()?;

        Ok(class_hierarchy
            .into_iter()
            .map(|x| {
                let raw: RawJavaClass = read_generic(&self.core, x.ptr_raw).unwrap();
                let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor).unwrap();

                descriptor.fields_size as JavaWord
            })
            .sum())
    }

    pub fn methods(&self) -> JavaResult<Vec<JavaMethod>> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        let ptr_methods = read_null_terminated_table(&self.core, descriptor.ptr_methods)?;

        Ok(ptr_methods.into_iter().map(|x| JavaMethod::from_raw(x, &self.core)).collect())
    }

    pub fn fields(&self) -> JavaResult<Vec<JavaField>> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        let ptr_fields = read_null_terminated_table(&self.core, descriptor.ptr_fields_or_element_type)?;

        Ok(ptr_fields.into_iter().map(|x| JavaField::from_raw(x, &self.core)).collect())
    }

    pub fn name(&self) -> JavaResult<String> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        read_null_terminated_string(&self.core, descriptor.ptr_name)
    }

    pub fn parent_class(&self) -> JavaResult<Option<JavaClass>> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        if descriptor.ptr_parent_class != 0 {
            Ok(Some(JavaClass::from_raw(descriptor.ptr_parent_class, &self.core)))
        } else {
            Ok(None)
        }
    }

    pub fn method(&self, fullname: &JavaFullName) -> JavaResult<Option<JavaMethod>> {
        let methods = self.methods()?;

        for method in methods {
            if method.name()? == *fullname {
                return Ok(Some(method));
            }
        }

        if let Some(x) = self.parent_class()? {
            x.method(fullname)
        } else {
            Ok(None)
        }
    }

    pub fn field(&self, name: &str) -> JavaResult<Option<JavaField>> {
        let fields = self.fields()?;

        for field in fields {
            if field.name()?.name == name {
                return Ok(Some(field));
            }
        }

        Ok(None)
    }

    pub fn read_static_field(&self, field: &JavaField) -> JavaResult<JavaWord> {
        let address = field.static_address()?;
        let result: u32 = read_generic(&self.core, address)?;

        Ok(result as _)
    }

    pub fn write_static_field(&mut self, field: &JavaField, value: JavaWord) -> JavaResult<()> {
        let address = field.static_address()?;

        write_generic(&mut self.core, address, value as u32)
    }

    pub async fn invoke_static_method(&self, method: &mut JavaMethod, args: &[JavaWord]) -> JavaResult<u32> {
        method.run(args).await
    }
}

impl Class for JavaClass {
    fn name(&self) -> String {
        self.name().unwrap()
    }

    fn super_class_name(&self) -> Option<String> {
        self.parent_class().unwrap().map(|x| x.name().unwrap())
    }

    fn instantiate(&self) -> Box<dyn ClassInstance> {
        let instance = JavaClassInstance::new(&mut self.core.clone(), self).unwrap();

        Box::new(instance)
    }

    fn method(&self, name: &str, descriptor: &str) -> Option<Box<dyn Method>> {
        self.method(&JavaFullName {
            name: name.to_string(),
            descriptor: descriptor.to_string(),
            tag: 0,
        })
        .unwrap()
        .map(|x| Box::new(x) as _)
    }

    fn field(&self, name: &str, _descriptor: &str, _is_static: bool) -> Option<Box<dyn Field>> {
        self.field(name).unwrap().map(|x| Box::new(x) as _)
    }

    fn get_static_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let value = self.read_static_field(field)?;

        Ok(JavaValue::from_raw(value, &field.descriptor(), &self.core))
    }

    fn put_static_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let value = value.as_raw(&field.descriptor());

        self.write_static_field(field, value as _)
    }
}
