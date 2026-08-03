use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{fmt, fmt::Debug, fmt::Formatter, mem::size_of, ops::Deref, ops::DerefMut};

use java_class_proto::JavaClassProto;
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use wipi_types::lgt::java::{
    LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor, LgtJavaClassField as RawJavaField,
    LgtJavaClassMethod as RawJavaMethod,
};

use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::native::NativeJavaValueCodec;
use wie_util::{
    ByteWrite, read_generic, read_null_terminated_string_bytes, read_null_terminated_table, write_generic, write_null_terminated_string_bytes,
    write_null_terminated_table,
};

use crate::runtime::java::JavaSvcFunctions;

use super::{JavaClassInstance, JavaField, JavaMethod, LgtJvmWord, Result, value::JavaValueCodec, vtable::JavaVtable};

#[derive(Clone)]
pub struct JavaClassDefinition {
    pub ptr_raw: u32,
    core: ArmCore,
}

impl JavaClassDefinition {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub async fn new<C, Context>(
        core: &mut ArmCore,
        jvm: &Jvm,
        proto: JavaClassProto<C>,
        context: Context,
        functions: JavaSvcFunctions,
    ) -> Result<Self>
    where
        C: ?Sized + 'static + Send,
        Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
    {
        let parent_class = if let Some(parent_name) = proto.parent_class {
            Some(
                jvm.resolve_class(parent_name)
                    .await
                    .unwrap()
                    .definition
                    .as_any()
                    .downcast_ref::<JavaClassDefinition>()
                    .unwrap()
                    .clone(),
            )
        } else {
            None
        };

        let parent_virtual_methods = if let Some(parent_class) = &parent_class {
            parent_class.virtual_methods(jvm).await?
        } else {
            Vec::new()
        };
        let mut virtual_method_signatures = parent_virtual_methods
            .iter()
            .map(|method| (method.name(), method.descriptor()))
            .collect::<Vec<_>>();
        for method in &proto.methods {
            if method.access_flags.intersects(MethodAccessFlags::STATIC | MethodAccessFlags::PRIVATE) || method.name.starts_with('<') {
                continue;
            }
            if !virtual_method_signatures
                .iter()
                .any(|(name, descriptor)| name == &method.name && descriptor == &method.descriptor)
            {
                virtual_method_signatures.push((method.name.to_string(), method.descriptor.to_string()));
            }
        }
        let virtual_method_count = virtual_method_signatures.len();
        let static_field_slot_count = proto
            .fields
            .iter()
            .filter(|field| field.access_flags.contains(FieldAccessFlags::STATIC))
            .map(|field| if field.descriptor == "J" || field.descriptor == "D" { 2 } else { 1 })
            .sum::<usize>();
        let class_storage_size = size_of::<RawJavaClass>() + (static_field_slot_count + virtual_method_count + 1) * size_of::<LgtJvmWord>();
        let ptr_raw = Allocator::alloc(core, class_storage_size as u32)?;
        core.write_bytes(ptr_raw, &vec![0; class_storage_size])?;
        let ptr_name = Self::allocate_string(core, proto.name)?;
        let ptr_super_class_name = if let Some(parent_class) = &parent_class {
            Self::allocate_string(core, &ClassDefinition::name(parent_class))?
        } else {
            0
        };

        let interface_names = proto.interfaces.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
        let ptr_interface_names = Self::allocate_string_table(core, &interface_names)?;

        let ptr_methods = if proto.methods.is_empty() {
            0
        } else {
            Allocator::alloc(core, (size_of::<u32>() + proto.methods.len() * size_of::<RawJavaMethod>()) as u32)?
        };
        if ptr_methods != 0 {
            write_generic(core, ptr_methods, proto.methods.len() as u32)?;
        }
        let mut methods = Vec::with_capacity(proto.methods.len());
        for (index, method) in proto.methods.into_iter().enumerate() {
            let ptr_method = ptr_methods + size_of::<u32>() as u32 + (index * size_of::<RawJavaMethod>()) as u32;
            methods.push(JavaMethod::new(
                core,
                jvm,
                ptr_method,
                ptr_raw,
                method,
                context.clone(),
                functions.clone(),
            )?);
        }

        let ptr_fields = if proto.fields.is_empty() {
            0
        } else {
            Allocator::alloc(core, (size_of::<u32>() + proto.fields.len() * size_of::<RawJavaField>()) as u32)?
        };
        if ptr_fields != 0 {
            write_generic(core, ptr_fields, proto.fields.len() as u32)?;
        }
        let mut instance_field_slot = parent_class
            .as_ref()
            .map(|class| class.instance_field_slot_count())
            .transpose()?
            .unwrap_or(0);
        let mut static_field_slot = 0usize;
        for (index, field) in proto.fields.into_iter().enumerate() {
            let is_static = field.access_flags.contains(FieldAccessFlags::STATIC);
            let slot = if is_static { static_field_slot } else { instance_field_slot };
            let slot_count = if field.descriptor == "J" || field.descriptor == "D" { 2 } else { 1 };
            if is_static {
                static_field_slot += slot_count;
            } else {
                instance_field_slot += slot_count;
            }

            let ptr_field = ptr_fields + size_of::<u32>() as u32 + (index * size_of::<RawJavaField>()) as u32;
            JavaField::new(core, ptr_field, ptr_raw, field, slot as u32)?;
        }

        let virtual_methods = JavaVtable::build_methods(&parent_virtual_methods, &methods);
        let ptr_dispatch_table = ptr_raw + (size_of::<RawJavaClass>() + static_field_slot_count * size_of::<LgtJvmWord>()) as u32;
        JavaVtable::write(core, ptr_dispatch_table, ptr_raw, &virtual_methods)?;
        let ptr_descriptor = Allocator::alloc(core, size_of::<RawJavaClassDescriptor>() as u32)?;
        write_generic(
            core,
            ptr_descriptor,
            RawJavaClassDescriptor {
                access_flags: proto.access_flags.bits() as u32,
                ptr_next_class: 0,
                ptr_name,
                ptr_instance_field_initializer_class: 0,
                ptr_super_class_name,
                unk4: 0,
                instance_field_slot_count: instance_field_slot as u16,
                link_state: 0,
                unk7: 0,
                ptr_instance_field_initializer_record: 0,
                unk9: 0,
                unk10: 0,
                unk11: 0,
                ptr_interface_names,
                fn_link_members: 0,
                fn_get_initialized_class: 0,
                fn_get_class: 0,
                ptr_methods,
                ptr_fields,
                unk13: 0,
                unk14: 0,
                unk15: 0,
            },
        )?;
        write_generic(
            core,
            ptr_raw,
            RawJavaClass {
                unk1: 0,
                unk2: 0,
                ptr_descriptor,
            },
        )?;

        tracing::trace!("Wrote LGT Java definition {} at {ptr_raw:#x}", proto.name);

        Ok(Self::from_raw(ptr_raw, core))
    }

    pub async fn new_array(core: &mut ArmCore, jvm: &Jvm, name: &str) -> Result<Self> {
        let parent_class = jvm
            .resolve_class("java/lang/Object")
            .await
            .unwrap()
            .definition
            .as_any()
            .downcast_ref::<JavaClassDefinition>()
            .unwrap()
            .clone();
        let virtual_methods = parent_class.virtual_methods(jvm).await?;
        let virtual_method_count = virtual_methods.len();
        let class_storage_size = size_of::<RawJavaClass>() + (virtual_method_count + 1) * size_of::<LgtJvmWord>();
        let ptr_raw = Allocator::alloc(core, class_storage_size as u32)?;
        core.write_bytes(ptr_raw, &vec![0; class_storage_size])?;
        let ptr_name = Self::allocate_string(core, name)?;
        let ptr_super_class_name = Self::allocate_string(core, "java/lang/Object")?;
        let interface_names = vec!["java/lang/Cloneable".to_string(), "java/io/Serializable".to_string()];
        let ptr_interface_names = Self::allocate_string_table(core, &interface_names)?;
        let ptr_dispatch_table = ptr_raw + size_of::<RawJavaClass>() as u32;
        JavaVtable::write(core, ptr_dispatch_table, ptr_raw, &virtual_methods)?;
        let ptr_descriptor = Allocator::alloc(core, size_of::<RawJavaClassDescriptor>() as u32)?;
        let access_flags = ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL;
        write_generic(
            core,
            ptr_descriptor,
            RawJavaClassDescriptor {
                access_flags: access_flags.bits() as u32,
                ptr_next_class: 0,
                ptr_name,
                ptr_instance_field_initializer_class: 0,
                ptr_super_class_name,
                unk4: 0,
                instance_field_slot_count: 0,
                link_state: 0,
                unk7: 0,
                ptr_instance_field_initializer_record: 0,
                unk9: 0,
                unk10: 0,
                unk11: 0,
                ptr_interface_names,
                fn_link_members: 0,
                fn_get_initialized_class: 0,
                fn_get_class: 0,
                ptr_methods: 0,
                ptr_fields: 0,
                unk13: 0,
                unk14: 0,
                unk15: 0,
            },
        )?;
        write_generic(
            core,
            ptr_raw,
            RawJavaClass {
                unk1: 0,
                unk2: 0,
                ptr_descriptor,
            },
        )?;

        Ok(Self::from_raw(ptr_raw, core))
    }

    fn allocate_string(core: &mut ArmCore, value: &str) -> Result<u32> {
        let address = Allocator::alloc(core, (value.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, address, value.as_bytes())?;
        Ok(address)
    }

    fn allocate_string_table(core: &mut ArmCore, values: &[String]) -> Result<u32> {
        if values.is_empty() {
            return Ok(0);
        }
        let pointers = values
            .iter()
            .map(|value| Self::allocate_string(core, value))
            .collect::<Result<Vec<_>>>()?;
        let address = Allocator::alloc(core, ((pointers.len() + 1) * size_of::<u32>()) as u32)?;
        write_null_terminated_table(core, address, &pointers)?;
        Ok(address)
    }

    fn raw(&self) -> Result<RawJavaClass> {
        read_generic(&self.core, self.ptr_raw)
    }

    fn descriptor(&self) -> Result<RawJavaClassDescriptor> {
        read_generic(&self.core, self.raw()?.ptr_descriptor)
    }

    fn read_string(&self, address: u32) -> Result<String> {
        Ok(String::from_utf8(read_null_terminated_string_bytes(&self.core, address)?).unwrap())
    }

    fn methods(&self) -> Result<Vec<JavaMethod>> {
        let ptr_methods = self.descriptor()?.ptr_methods;
        if ptr_methods == 0 {
            return Ok(Vec::new());
        }
        let count: u32 = read_generic(&self.core, ptr_methods)?;
        Ok((0..count as usize)
            .map(|index| {
                JavaMethod::from_raw(
                    ptr_methods + size_of::<u32>() as u32 + (index * size_of::<RawJavaMethod>()) as u32,
                    &self.core,
                )
            })
            .collect())
    }

    fn fields(&self) -> Result<Vec<JavaField>> {
        let ptr_fields = self.descriptor()?.ptr_fields;
        if ptr_fields == 0 {
            return Ok(Vec::new());
        }
        let count: u32 = read_generic(&self.core, ptr_fields)?;
        Ok((0..count as usize)
            .map(|index| {
                JavaField::from_raw(
                    ptr_fields + size_of::<u32>() as u32 + (index * size_of::<RawJavaField>()) as u32,
                    &self.core,
                )
            })
            .collect())
    }

    pub fn ptr_dispatch_table(&self) -> Result<u32> {
        let static_field_slot_count = self
            .fields()?
            .into_iter()
            .filter(|field| field.access_flags().contains(FieldAccessFlags::STATIC))
            .map(|field| if field.descriptor() == "J" || field.descriptor() == "D" { 2 } else { 1 })
            .sum::<usize>();
        Ok(self.ptr_raw + (size_of::<RawJavaClass>() + static_field_slot_count * size_of::<LgtJvmWord>()) as u32)
    }

    pub(super) fn ptr_static_fields(&self) -> Result<u32> {
        Ok(self.ptr_raw + size_of::<RawJavaClass>() as u32)
    }

    pub fn instance_field_slot_count(&self) -> Result<usize> {
        Ok(self.descriptor()?.instance_field_slot_count as usize)
    }

    pub(super) async fn virtual_methods(&self, jvm: &Jvm) -> Result<Vec<JavaMethod>> {
        let mut hierarchy = vec![self.clone()];
        while let Some(parent_name) = ClassDefinition::super_class_name(hierarchy.last().unwrap()) {
            let parent_class = jvm
                .resolve_class(&parent_name)
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<JavaClassDefinition>()
                .unwrap()
                .clone();
            hierarchy.push(parent_class);
        }

        let mut virtual_methods = Vec::new();
        for class in hierarchy.into_iter().rev() {
            virtual_methods = JavaVtable::build_methods(&virtual_methods, &class.methods()?);
        }
        Ok(virtual_methods)
    }
}

#[async_trait::async_trait]
impl ClassDefinition for JavaClassDefinition {
    fn name(&self) -> String {
        self.read_string(self.descriptor().unwrap().ptr_name).unwrap()
    }

    fn super_class_name(&self) -> Option<String> {
        let ptr_name = self.descriptor().unwrap().ptr_super_class_name;
        (ptr_name != 0).then(|| self.read_string(ptr_name).unwrap())
    }

    fn interface_names(&self) -> Vec<String> {
        let ptr_names = self.descriptor().unwrap().ptr_interface_names;
        if ptr_names == 0 {
            return Vec::new();
        }
        read_null_terminated_table(&self.core, ptr_names)
            .unwrap()
            .into_iter()
            .map(|ptr_name| self.read_string(ptr_name).unwrap())
            .collect()
    }

    fn access_flags(&self) -> ClassAccessFlags {
        ClassAccessFlags::from_bits_truncate(self.descriptor().unwrap().access_flags as u16)
    }

    async fn instantiate(&self, jvm: &Jvm) -> JvmResult<Box<dyn ClassInstance>> {
        match JavaClassInstance::new(&mut self.core.clone(), self) {
            Ok(instance) => Ok(Box::new(instance)),
            Err(error) => Err(jvm.exception("net/wie/WieError", &format!("Failed to instantiate class: {error}")).await),
        }
    }

    async fn prepare(&self, _: &Jvm) -> JvmResult<()> {
        Ok(())
    }

    fn method(&self, name: &str, descriptor: &str, is_static: bool) -> Option<Box<dyn Method>> {
        self.methods()
            .unwrap()
            .into_iter()
            .find(|method| {
                method.name() == name && method.descriptor() == descriptor && method.access_flags().contains(MethodAccessFlags::STATIC) == is_static
            })
            .map(|method| Box::new(method) as Box<_>)
    }

    fn field(&self, name: &str, descriptor: &str, is_static: bool) -> Option<Box<dyn Field>> {
        self.fields()
            .unwrap()
            .into_iter()
            .find(|field| {
                field.name() == name && field.descriptor() == descriptor && field.access_flags().contains(FieldAccessFlags::STATIC) == is_static
            })
            .map(|field| Box::new(field) as Box<_>)
    }

    fn fields(&self) -> Vec<Box<dyn Field>> {
        self.fields().unwrap().into_iter().map(|field| Box::new(field) as Box<_>).collect()
    }

    fn get_static_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let field_type = JavaType::parse(&field.descriptor());
        let address = field.static_address().unwrap();
        let low = read_generic(&self.core, address).unwrap();
        let codec = JavaValueCodec::new(&self.core);
        Ok(if matches!(field_type, JavaType::Long | JavaType::Double) {
            let high = read_generic(&self.core, address + 4).unwrap();
            codec.decode_wide(low, high, &field_type)
        } else {
            codec.decode_word(low, &field_type)
        })
    }

    fn put_static_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let address = field.static_address().unwrap();
        let codec = JavaValueCodec::new(&self.core);
        if matches!(value, JavaValue::Long(_) | JavaValue::Double(_)) {
            let (low, high) = codec.encode_wide(&value);
            write_generic(&mut self.core, address, low).unwrap();
            write_generic(&mut self.core, address + 4, high).unwrap();
        } else {
            write_generic(&mut self.core, address, codec.encode_word(&value)).unwrap();
        }
        Ok(())
    }
}

impl Debug for JavaClassDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaClassDefinition")
            .field("name", &ClassDefinition::name(self))
            .field("ptr_raw", &self.ptr_raw)
            .finish()
    }
}
