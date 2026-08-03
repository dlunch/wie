use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{fmt, fmt::Debug, fmt::Formatter, mem::size_of, ops::Deref, ops::DerefMut};

use java_class_proto::JavaClassProto;
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use spin::Mutex;
use wipi_types::lgt::java::{
    LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor, LgtJavaClassField as RawJavaField,
    LgtJavaClassMethod as RawJavaMethod,
};

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{ByteWrite, read_generic, write_generic, write_null_terminated_string_bytes, write_null_terminated_table};

use crate::runtime::java::JavaSvcFunctions;

use super::{JavaClassInstance, JavaField, JavaMethod, LgtJvmWord, Result, value::JavaValueExt, vtable::JavaVtable};

pub(super) type ClassRegistry = Arc<Mutex<BTreeMap<u32, Weak<ClassMetadata>>>>;

pub(super) struct ClassMetadata {
    pub ptr_raw: u32,
    pub ptr_dispatch_table: u32,
    pub ptr_static_fields: u32,
    pub instance_field_slot_count: usize,
    pub name: String,
    pub super_class: Option<JavaClassDefinition>,
    pub interface_names: Vec<String>,
    pub access_flags: ClassAccessFlags,
    pub methods: Vec<JavaMethod>,
    pub fields: Vec<JavaField>,
    pub virtual_methods: Vec<JavaMethod>,
}

#[derive(Clone)]
pub struct JavaClassDefinition {
    metadata: Arc<ClassMetadata>,
    core: ArmCore,
    registry: ClassRegistry,
}

impl JavaClassDefinition {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore, registry: &ClassRegistry) -> Self {
        let metadata = registry.lock().get(&ptr_raw).unwrap().upgrade().unwrap();
        Self {
            metadata,
            core: core.clone(),
            registry: registry.clone(),
        }
    }

    pub async fn new<C, Context>(
        core: &mut ArmCore,
        jvm: &Jvm,
        proto: JavaClassProto<C>,
        context: Context,
        functions: JavaSvcFunctions,
        registry: ClassRegistry,
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

        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;
        let ptr_name = Self::allocate_string(core, proto.name)?;
        let ptr_super_class_name = if let Some(parent_class) = &parent_class {
            Self::allocate_string(core, &parent_class.metadata.name)?
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
                registry.clone(),
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
        let mut instance_field_slot = parent_class.as_ref().map(|x| x.metadata.instance_field_slot_count).unwrap_or(0);
        let mut static_field_slot = 0usize;
        let mut fields = Vec::with_capacity(proto.fields.len());
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
            fields.push(JavaField::new(core, ptr_field, ptr_raw, field, slot as u32, registry.clone())?);
        }

        let ptr_static_fields = if static_field_slot == 0 {
            0
        } else {
            let address = Allocator::alloc(core, (static_field_slot * size_of::<LgtJvmWord>()) as u32)?;
            core.write_bytes(address, &alloc::vec![0; static_field_slot * size_of::<LgtJvmWord>()])?;
            address
        };

        let vtable = JavaVtable::new(core, ptr_raw, parent_class.as_ref(), &methods)?;
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

        let metadata = Arc::new(ClassMetadata {
            ptr_raw,
            ptr_dispatch_table: vtable.ptr_raw,
            ptr_static_fields,
            instance_field_slot_count: instance_field_slot,
            name: proto.name.to_string(),
            super_class: parent_class,
            interface_names,
            access_flags: proto.access_flags,
            methods,
            fields,
            virtual_methods: vtable.methods,
        });
        registry.lock().insert(ptr_raw, Arc::downgrade(&metadata));

        tracing::trace!("Wrote LGT Java definition {} at {ptr_raw:#x}", proto.name);

        Ok(Self {
            metadata,
            core: core.clone(),
            registry,
        })
    }

    pub async fn new_array(core: &mut ArmCore, jvm: &Jvm, name: &str, registry: ClassRegistry) -> Result<Self> {
        let parent_class = jvm
            .resolve_class("java/lang/Object")
            .await
            .unwrap()
            .definition
            .as_any()
            .downcast_ref::<JavaClassDefinition>()
            .unwrap()
            .clone();
        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;
        let ptr_name = Self::allocate_string(core, name)?;
        let ptr_super_class_name = Self::allocate_string(core, "java/lang/Object")?;
        let interface_names = alloc::vec!["java/lang/Cloneable".to_string(), "java/io/Serializable".to_string()];
        let ptr_interface_names = Self::allocate_string_table(core, &interface_names)?;
        let vtable = JavaVtable::new(core, ptr_raw, Some(&parent_class), &[])?;
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

        let metadata = Arc::new(ClassMetadata {
            ptr_raw,
            ptr_dispatch_table: vtable.ptr_raw,
            ptr_static_fields: 0,
            instance_field_slot_count: 0,
            name: name.to_string(),
            super_class: Some(parent_class),
            interface_names,
            access_flags,
            methods: Vec::new(),
            fields: Vec::new(),
            virtual_methods: vtable.methods,
        });
        registry.lock().insert(ptr_raw, Arc::downgrade(&metadata));

        Ok(Self {
            metadata,
            core: core.clone(),
            registry,
        })
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

    #[cfg(test)]
    pub fn ptr_raw(&self) -> u32 {
        self.metadata.ptr_raw
    }

    pub fn ptr_dispatch_table(&self) -> u32 {
        self.metadata.ptr_dispatch_table
    }

    pub fn instance_field_slot_count(&self) -> usize {
        self.metadata.instance_field_slot_count
    }

    pub fn virtual_methods(&self) -> &[JavaMethod] {
        &self.metadata.virtual_methods
    }

    pub fn registry(&self) -> &ClassRegistry {
        &self.registry
    }
}

#[async_trait::async_trait]
impl ClassDefinition for JavaClassDefinition {
    fn name(&self) -> String {
        self.metadata.name.clone()
    }

    fn super_class_name(&self) -> Option<String> {
        self.metadata.super_class.as_ref().map(|x| x.metadata.name.clone())
    }

    fn interface_names(&self) -> Vec<String> {
        self.metadata.interface_names.clone()
    }

    fn access_flags(&self) -> ClassAccessFlags {
        self.metadata.access_flags
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
        self.metadata
            .methods
            .iter()
            .find(|method| {
                method.name() == name && method.descriptor() == descriptor && method.access_flags().contains(MethodAccessFlags::STATIC) == is_static
            })
            .cloned()
            .map(|x| Box::new(x) as Box<_>)
    }

    fn field(&self, name: &str, descriptor: &str, is_static: bool) -> Option<Box<dyn Field>> {
        self.metadata
            .fields
            .iter()
            .find(|field| {
                field.name() == name && field.descriptor() == descriptor && field.access_flags().contains(FieldAccessFlags::STATIC) == is_static
            })
            .cloned()
            .map(|x| Box::new(x) as Box<_>)
    }

    fn fields(&self) -> Vec<Box<dyn Field>> {
        self.metadata.fields.iter().cloned().map(|x| Box::new(x) as Box<_>).collect()
    }

    fn get_static_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let field_type = JavaType::parse(&field.descriptor());
        let address = field.static_address().unwrap();
        let low = read_generic(&self.core, address).unwrap();
        Ok(if matches!(field_type, JavaType::Long | JavaType::Double) {
            let high = read_generic(&self.core, address + 4).unwrap();
            JavaValue::from_raw64(low, high, &field_type)
        } else {
            JavaValue::from_raw(low, &field_type, &self.core, &self.registry)
        })
    }

    fn put_static_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let address = field.static_address().unwrap();
        if matches!(value, JavaValue::Long(_) | JavaValue::Double(_)) {
            let (low, high) = value.as_raw64();
            write_generic(&mut self.core, address, low).unwrap();
            write_generic(&mut self.core, address + 4, high).unwrap();
        } else {
            write_generic(&mut self.core, address, value.as_raw()).unwrap();
        }
        Ok(())
    }
}

impl Debug for JavaClassDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaClassDefinition")
            .field("name", &self.metadata.name)
            .field("ptr_raw", &self.metadata.ptr_raw)
            .finish()
    }
}
