use alloc::{boxed::Box, string::String, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    mem::size_of,
    ops::{Deref, DerefMut},
};

use bytemuck::{Pod, Zeroable};

use java_class_proto::JavaClassProto;
use java_constants::FieldAccessFlags;
use jvm::{ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Jvm, JvmResult, Method};

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{
    read_generic, read_null_terminated_string, read_null_terminated_table, write_generic, write_null_terminated_string, write_null_terminated_table,
};

use super::{
    class_instance::JavaClassInstance, field::JavaField, method::JavaMethod, value::JavaValueExt, vtable_builder::JavaVtableBuilder, KtfJvmWord,
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

#[derive(Clone)]
pub struct JavaClassDefinition {
    pub(crate) ptr_raw: u32,
    core: ArmCore,
}

impl JavaClassDefinition {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub async fn new<C, Context>(core: &mut ArmCore, jvm: &Jvm, name: &str, proto: JavaClassProto<C>, context: Context) -> anyhow::Result<Self>
    where
        C: ?Sized + 'static,
        Context: Deref<Target = C> + DerefMut + Clone + 'static,
    {
        let parent_class = if let Some(x) = proto.parent_class {
            let class = jvm.resolve_class(x).await?.definition;
            let class = class.as_any().downcast_ref::<JavaClassDefinition>().unwrap().clone();

            Some(class)
        } else {
            None
        };

        let mut vtable_builder = JavaVtableBuilder::new(&parent_class)?;

        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;

        let mut methods = Vec::new();
        for method in proto.methods.into_iter() {
            let method = JavaMethod::new(core, ptr_raw, method, &mut vtable_builder, context.clone())?;

            methods.push(method.ptr_raw);
        }
        let ptr_methods = Allocator::alloc(core, ((methods.len() + 1) * size_of::<u32>()) as _)?;
        write_null_terminated_table(core, ptr_methods, &methods)?;

        let mut fields = Vec::new();
        for (index, field) in proto.fields.into_iter().enumerate() {
            let offset_or_value = if field.access_flags.contains(FieldAccessFlags::STATIC) {
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

        Ok(result)
    }

    pub fn read_class_hierarchy(&self) -> anyhow::Result<Vec<JavaClassDefinition>> {
        let mut result = vec![];

        let mut current_class = self.ptr_raw;
        loop {
            result.push(JavaClassDefinition::from_raw(current_class, &self.core));

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

    pub fn ptr_vtable(&self) -> anyhow::Result<u32> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;

        Ok(raw.ptr_vtable)
    }

    pub fn field_size(&self) -> anyhow::Result<usize> {
        let class_hierarchy = self.read_class_hierarchy()?;

        Ok(class_hierarchy
            .into_iter()
            .map(|x| {
                let raw: RawJavaClass = read_generic(&self.core, x.ptr_raw).unwrap();
                let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor).unwrap();

                descriptor.fields_size as usize
            })
            .sum())
    }

    pub fn methods(&self) -> anyhow::Result<Vec<JavaMethod>> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        if descriptor.ptr_methods == 0 {
            return Ok(Vec::new());
        }

        let ptr_methods = read_null_terminated_table(&self.core, descriptor.ptr_methods)?;

        Ok(ptr_methods.into_iter().map(|x| JavaMethod::from_raw(x, &self.core)).collect())
    }

    pub fn fields(&self) -> anyhow::Result<Vec<JavaField>> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        let ptr_fields = read_null_terminated_table(&self.core, descriptor.ptr_fields_or_element_type)?;

        Ok(ptr_fields.into_iter().map(|x| JavaField::from_raw(x, &self.core)).collect())
    }

    pub fn name(&self) -> anyhow::Result<String> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        read_null_terminated_string(&self.core, descriptor.ptr_name)
    }

    pub fn parent_class(&self) -> anyhow::Result<Option<JavaClassDefinition>> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        if descriptor.ptr_parent_class != 0 {
            Ok(Some(JavaClassDefinition::from_raw(descriptor.ptr_parent_class, &self.core)))
        } else {
            Ok(None)
        }
    }

    pub fn method(&self, name: &str, descriptor: &str) -> anyhow::Result<Option<JavaMethod>> {
        let methods = self.methods()?;

        for method in methods {
            let full_name = method.name()?;
            if full_name.name == name && full_name.descriptor == descriptor {
                return Ok(Some(method));
            }
        }

        if let Some(x) = self.parent_class()? {
            x.method(name, descriptor)
        } else {
            Ok(None)
        }
    }

    pub fn field(&self, name: &str, descriptor: &str, is_static: bool) -> anyhow::Result<Option<JavaField>> {
        let fields = self.fields()?;

        for field in fields {
            let full_name = field.name()?;
            if full_name.name == name && full_name.descriptor == descriptor && field.access_flags().contains(FieldAccessFlags::STATIC) == is_static {
                return Ok(Some(field));
            }
        }

        Ok(None)
    }

    pub fn read_static_field(&self, field: &JavaField) -> anyhow::Result<KtfJvmWord> {
        let address = field.static_address()?;
        let result: KtfJvmWord = read_generic(&self.core, address)?;

        Ok(result as _)
    }

    pub fn write_static_field(&mut self, field: &JavaField, value: KtfJvmWord) -> anyhow::Result<()> {
        let address = field.static_address()?;

        write_generic(&mut self.core, address, value)
    }
}

impl ClassDefinition for JavaClassDefinition {
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
        self.method(name, descriptor).unwrap().map(|x| Box::new(x) as _)
    }

    fn field(&self, name: &str, descriptor: &str, is_static: bool) -> Option<Box<dyn Field>> {
        self.field(name, descriptor, is_static).unwrap().map(|x| Box::new(x) as _)
    }

    fn get_static_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let value = self.read_static_field(field).unwrap();

        let r#type = JavaType::parse(&field.descriptor());
        Ok(JavaValue::from_raw(value, &r#type, &self.core))
    }

    fn put_static_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let value = value.as_raw();

        self.write_static_field(field, value as _).unwrap();

        Ok(())
    }
}

impl Debug for JavaClassDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaMethod").field("ptr_raw", &self.ptr_raw).finish()
    }
}
