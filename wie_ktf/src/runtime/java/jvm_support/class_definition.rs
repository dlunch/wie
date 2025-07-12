use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    mem::size_of,
    ops::{Deref, DerefMut},
};

use java_class_proto::JavaClassProto;
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassDefinition, ClassInstance, Field, JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use wipi_types::ktf::java::{JavaClass as RawJavaClass, JavaClassDescriptor as RawJavaClassDescriptor};

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{
    read_generic, read_null_terminated_string_bytes, read_null_terminated_table, write_generic, write_null_terminated_string_bytes,
    write_null_terminated_table,
};

use super::{
    KtfJvmWord, Result, class_instance::JavaClassInstance, field::JavaField, method::JavaMethod, value::JavaValueExt,
    vtable_builder::JavaVtableBuilder,
};

#[derive(Clone)]
pub struct JavaClassDefinition {
    pub(crate) ptr_raw: u32,
    core: ArmCore,
}

impl JavaClassDefinition {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub async fn new<C, Context>(core: &mut ArmCore, jvm: &Jvm, proto: JavaClassProto<C>, context: Context) -> Result<Self>
    where
        C: ?Sized + 'static + Send,
        Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
    {
        let parent_class = if let Some(x) = proto.parent_class {
            let class = jvm.resolve_class(x).await.unwrap().definition;
            let class = class.as_any().downcast_ref::<JavaClassDefinition>().unwrap().clone();

            Some(class)
        } else {
            None
        };

        let field_offset_base: u32 = if let Some(x) = &parent_class { x.field_size()? as _ } else { 0 };

        let mut vtable_builder = JavaVtableBuilder::new(&parent_class)?;

        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;

        let mut methods = Vec::new();
        for method in proto.methods.into_iter() {
            let method = JavaMethod::new(core, jvm, ptr_raw, method, &mut vtable_builder, context.clone())?;

            methods.push(method.ptr_raw);
        }
        let ptr_methods = Allocator::alloc(core, ((methods.len() + 1) * size_of::<u32>()) as _)?;
        write_null_terminated_table(core, ptr_methods, &methods)?;

        let mut fields = Vec::new();
        let mut field_offset = field_offset_base;
        for field in proto.fields.into_iter() {
            let offset_or_value = if field.access_flags.contains(FieldAccessFlags::STATIC) {
                0
            } else {
                let field_size = if field.descriptor == "J" || field.descriptor == "D" { 8 } else { 4 };

                field_offset += field_size;

                field_offset - field_size
            };

            let field = JavaField::new(core, ptr_raw, field, offset_or_value)?;

            fields.push(field.ptr_raw);
        }
        let ptr_fields = Allocator::alloc(core, ((fields.len() + 1) * size_of::<u32>()) as _)?;
        write_null_terminated_table(core, ptr_fields, &fields)?;

        let ptr_name = Allocator::alloc(core, (proto.name.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_name, proto.name.as_bytes())?;

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
                fields_size: field_offset as u16,
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

        tracing::trace!("Wrote definition {} at {:#x}", proto.name, ptr_raw);

        let result = Self::from_raw(ptr_raw, core);

        Ok(result)
    }

    pub fn read_class_hierarchy(&self) -> Result<Vec<JavaClassDefinition>> {
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

    pub fn ptr_vtable(&self) -> Result<u32> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;

        Ok(raw.ptr_vtable)
    }

    pub fn field_size(&self) -> Result<usize> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        Ok(descriptor.fields_size as _)
    }

    pub fn methods(&self) -> Result<Vec<JavaMethod>> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        if descriptor.ptr_methods == 0 {
            return Ok(Vec::new());
        }

        let ptr_methods = read_null_terminated_table(&self.core, descriptor.ptr_methods)?;

        let mut result = Vec::with_capacity(ptr_methods.len());
        for method in ptr_methods {
            let method = JavaMethod::from_raw(method, &self.core);

            if method.ptr_class() == self.ptr_raw {
                result.push(method)
            }
        }

        Ok(result)
    }

    pub fn fields(&self) -> Result<Vec<JavaField>> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        if descriptor.ptr_fields_or_element_type == 0 {
            return Ok(Vec::new());
        }

        if self.name()?.starts_with("[") {
            return Ok(Vec::new());
        }

        let ptr_fields = read_null_terminated_table(&self.core, descriptor.ptr_fields_or_element_type)?;

        Ok(ptr_fields.into_iter().map(|x| JavaField::from_raw(x, &self.core)).collect())
    }

    pub fn name(&self) -> Result<String> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        let bytes = read_null_terminated_string_bytes(&self.core, descriptor.ptr_name)?;

        Ok(String::from_utf8(bytes).unwrap())
    }

    pub fn parent_class(&self) -> Result<Option<JavaClassDefinition>> {
        let raw: RawJavaClass = read_generic(&self.core, self.ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(&self.core, raw.ptr_descriptor)?;

        if descriptor.ptr_parent_class != 0 {
            Ok(Some(JavaClassDefinition::from_raw(descriptor.ptr_parent_class, &self.core)))
        } else {
            Ok(None)
        }
    }

    pub fn method(&self, name: &str, descriptor: &str, is_static: bool) -> Result<Option<JavaMethod>> {
        let methods = self.methods()?;

        for method in methods {
            let full_name = method.name()?;
            if full_name.name == name && full_name.descriptor == descriptor && method.access_flags().contains(MethodAccessFlags::STATIC) == is_static
            {
                return Ok(Some(method));
            }
        }

        Ok(None)
    }

    pub fn field(&self, name: &str, descriptor: &str, is_static: bool) -> Result<Option<JavaField>> {
        let fields = self.fields()?;

        for field in fields {
            let full_name = field.name()?;
            if full_name.name == name && full_name.descriptor == descriptor && field.access_flags().contains(FieldAccessFlags::STATIC) == is_static {
                return Ok(Some(field));
            }
        }

        Ok(None)
    }

    pub fn read_static_field(&self, field: &JavaField) -> Result<KtfJvmWord> {
        let address = field.static_address()?;
        let result: KtfJvmWord = read_generic(&self.core, address)?;

        Ok(result as _)
    }

    pub fn write_static_field(&mut self, field: &JavaField, value: KtfJvmWord) -> Result<()> {
        let address = field.static_address()?;

        write_generic(&mut self.core, address, value)
    }
}

#[async_trait::async_trait]
impl ClassDefinition for JavaClassDefinition {
    fn name(&self) -> String {
        self.name().unwrap()
    }

    fn super_class_name(&self) -> Option<String> {
        self.parent_class().unwrap().map(|x| x.name().unwrap())
    }

    fn instantiate(&self) -> JvmResult<Box<dyn ClassInstance>> {
        let instance = JavaClassInstance::new(&mut self.core.clone(), self).map_err(|x| JavaError::FatalError(format!("{x}")))?;

        Ok(Box::new(instance))
    }

    fn method(&self, name: &str, descriptor: &str, is_static: bool) -> Option<Box<dyn Method>> {
        self.method(name, descriptor, is_static).unwrap().map(|x| Box::new(x) as _)
    }

    fn field(&self, name: &str, descriptor: &str, is_static: bool) -> Option<Box<dyn Field>> {
        self.field(name, descriptor, is_static).unwrap().map(|x| Box::new(x) as _)
    }

    fn fields(&self) -> Vec<Box<dyn Field>> {
        self.fields().unwrap().into_iter().map(|x| Box::new(x) as _).collect()
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
