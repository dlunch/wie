use alloc::{boxed::Box, format, string::String, sync::Arc, vec, vec::Vec};
use core::{fmt, fmt::Debug, fmt::Formatter, mem::offset_of, mem::size_of, ops::Deref, ops::DerefMut};

use java_class_proto::JavaClassProto;
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassDefinition, ClassInstance, Field, JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use wipi_types::lgt::java::{
    LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor, LgtJavaClassField as RawJavaField,
    LgtJavaClassMethod as RawJavaMethod,
};

use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, RegisteredFunction, RegisteredFunctionHolder};
use wie_jvm_support::native::NativeJavaValueCodec;
use wie_util::{ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes};

use crate::runtime::{
    SVC_CATEGORY_JAVA,
    java::{
        JavaSvcFunctions,
        abi::{CLASS_INITIALIZATION_STATE_FIELD, CLASS_NATIVE_NAME_FIELD, JAVA_ABI, WORD_FIELD_DESCRIPTOR},
    },
};

use super::{
    JavaClassInstance, JavaField, JavaMethod, LgtJvmWord,
    value::JavaValueCodec,
    vtable::{JavaVtable, JavaVtableEntry},
};

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
        let JavaClassProto {
            name: class_name,
            parent_class: parent_class_name,
            interfaces: interface_names,
            methods: method_protos,
            fields: field_protos,
            access_flags,
        } = proto;
        let class_abi = JAVA_ABI.class(class_name);
        let fixed_fields = class_abi.map(|class| class.field.as_slice()).unwrap_or_default();

        let parent_class = if let Some(parent_name) = parent_class_name {
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

        let parent_name = parent_class.as_ref().map(ClassDefinition::name);
        let static_field_word_count = field_protos
            .iter()
            .filter(|field| field.access_flags.contains(FieldAccessFlags::STATIC))
            .map(|field| if field.descriptor == "J" || field.descriptor == "D" { 2 } else { 1 })
            .sum::<usize>();
        let class_storage_size = size_of::<RawJavaClass>() + static_field_word_count * size_of::<LgtJvmWord>();

        let ptr_raw = Allocator::alloc(core, class_storage_size as u32)?;
        let ptr_name = Allocator::alloc(core, (class_name.len() + 1) as u32)?;
        let ptr_super_class_name = if let Some(parent_name) = &parent_name {
            Allocator::alloc(core, (parent_name.len() + 1) as u32)?
        } else {
            0
        };
        let interface_name_pointers = interface_names
            .iter()
            .map(|interface_name| Allocator::alloc(core, (interface_name.len() + 1) as u32))
            .collect::<Result<Vec<_>>>()?;
        let ptr_interface_names = Allocator::alloc(core, ((interface_name_pointers.len() + 1) * size_of::<u32>()) as u32)?;
        let ptr_methods = if method_protos.is_empty() {
            0
        } else {
            Allocator::alloc(core, (size_of::<u32>() + method_protos.len() * size_of::<RawJavaMethod>()) as u32)?
        };
        let field_count = field_protos.len() + fixed_fields.len();
        let ptr_fields = if field_count == 0 {
            0
        } else {
            Allocator::alloc(core, (size_of::<u32>() + field_count * size_of::<RawJavaField>()) as u32)?
        };
        let ptr_descriptor = Allocator::alloc(core, size_of::<RawJavaClassDescriptor>() as u32)?;

        core.write_bytes(ptr_raw, &vec![0; class_storage_size])?;
        write_null_terminated_string_bytes(core, ptr_name, class_name.as_bytes())?;
        if let Some(parent_name) = &parent_name {
            write_null_terminated_string_bytes(core, ptr_super_class_name, parent_name.as_bytes())?;
        }
        for (interface_name, ptr_interface_name) in interface_names.iter().zip(&interface_name_pointers) {
            write_null_terminated_string_bytes(core, *ptr_interface_name, interface_name.as_bytes())?;
        }
        write_generic(core, ptr_interface_names, interface_name_pointers.len() as u32)?;
        for (index, ptr_interface_name) in interface_name_pointers.into_iter().enumerate() {
            write_generic(core, ptr_interface_names + ((index + 1) * size_of::<u32>()) as u32, ptr_interface_name)?;
        }
        if ptr_methods != 0 {
            write_generic(core, ptr_methods, method_protos.len() as u32)?;
        }
        if ptr_fields != 0 {
            write_generic(core, ptr_fields, field_count as u32)?;
        }

        let mut methods = Vec::with_capacity(method_protos.len());
        for (index, method) in method_protos.into_iter().enumerate() {
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
        let mut instance_field_word_index = parent_class
            .as_ref()
            .map(|class| class.instance_field_word_count())
            .transpose()?
            .unwrap_or(0);
        let mut static_field_word_index = 0usize;
        for (index, field) in field_protos.iter().enumerate() {
            let is_static = field.access_flags.contains(FieldAccessFlags::STATIC);
            let word_index = if is_static { static_field_word_index } else { instance_field_word_index };
            let word_count = if field.descriptor == "J" || field.descriptor == "D" { 2 } else { 1 };
            if is_static {
                static_field_word_index += word_count;
            } else {
                instance_field_word_index += word_count;
            }

            let ptr_field = ptr_fields + size_of::<u32>() as u32 + (index * size_of::<RawJavaField>()) as u32;
            JavaField::new(
                core,
                ptr_field,
                ptr_raw,
                &field.name,
                &field.descriptor,
                field.access_flags,
                word_index as u32,
            )?;
        }
        for (offset, field) in fixed_fields.iter().enumerate() {
            let index = field_protos.len() + offset;
            let ptr_field = ptr_fields + size_of::<u32>() as u32 + (index * size_of::<RawJavaField>()) as u32;
            JavaField::new(
                core,
                ptr_field,
                ptr_raw,
                &field.name,
                &field.descriptor,
                FieldAccessFlags::PRIVATE,
                field.index,
            )?;
        }
        if let Some(field_size) = class_abi.and_then(|class| class.field_size) {
            instance_field_word_index = instance_field_word_index.max(field_size);
        }
        let virtual_methods = JavaVtable::build_methods(jvm, class_name, parent_class.as_ref(), &methods).await?;
        let ptr_vtable = JavaVtable::allocate(core, virtual_methods.len())?;
        JavaVtable::write(core, ptr_vtable, ptr_raw, &virtual_methods)?;

        write_generic(
            core,
            ptr_descriptor,
            RawJavaClassDescriptor {
                access_flags: access_flags.bits() as u32,
                ptr_next_class: 0,
                ptr_name,
                ptr_vtable: 0,
                ptr_super_class_name,
                unk4: 0,
                instance_field_word_count: instance_field_word_index as u16,
                link_state: 0,
                unk7: 0,
                ptr_instance_field_initializer_record: 0,
                unk9: 0,
                unk10: 0,
                vtable_count: 0,
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
                unk1: ptr_vtable,
                unk2: 0,
                ptr_descriptor,
            },
        )?;

        let class = Self::from_raw(ptr_raw, core);
        class.register_class_getters(core, jvm, functions)?;

        tracing::trace!("Wrote LGT Java definition {class_name} at {ptr_raw:#x}");

        Ok(class)
    }

    pub async fn new_array(core: &mut ArmCore, jvm: &Jvm, name: &str, functions: JavaSvcFunctions) -> Result<Self> {
        let parent_class = jvm
            .resolve_class("java/lang/Object")
            .await
            .unwrap()
            .definition
            .as_any()
            .downcast_ref::<JavaClassDefinition>()
            .unwrap()
            .clone();
        let virtual_methods = parent_class.vtable_entries(jvm).await?;
        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;
        let ptr_name = Allocator::alloc(core, (name.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_name, name.as_bytes())?;
        let ptr_super_class_name = Allocator::alloc(core, "java/lang/Object".len() as u32 + 1)?;
        write_null_terminated_string_bytes(core, ptr_super_class_name, b"java/lang/Object")?;
        let interface_name_pointers = ["java/lang/Cloneable", "java/io/Serializable"]
            .iter()
            .map(|interface_name| {
                let address = Allocator::alloc(core, (interface_name.len() + 1) as u32)?;
                write_null_terminated_string_bytes(core, address, interface_name.as_bytes())?;
                Ok(address)
            })
            .collect::<Result<Vec<_>>>()?;
        let ptr_interface_names = Allocator::alloc(core, ((interface_name_pointers.len() + 1) * size_of::<u32>()) as u32)?;
        write_generic(core, ptr_interface_names, interface_name_pointers.len() as u32)?;
        for (index, ptr_name) in interface_name_pointers.into_iter().enumerate() {
            write_generic(core, ptr_interface_names + ((index + 1) * size_of::<u32>()) as u32, ptr_name)?;
        }
        let ptr_vtable = JavaVtable::allocate(core, virtual_methods.len())?;
        JavaVtable::write(core, ptr_vtable, ptr_raw, &virtual_methods)?;
        let ptr_descriptor = Allocator::alloc(core, size_of::<RawJavaClassDescriptor>() as u32)?;
        let access_flags = ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL;
        write_generic(
            core,
            ptr_descriptor,
            RawJavaClassDescriptor {
                access_flags: access_flags.bits() as u32,
                ptr_next_class: 0,
                ptr_name,
                ptr_vtable: 0,
                ptr_super_class_name,
                unk4: 0,
                instance_field_word_count: 0,
                link_state: 0,
                unk7: 0,
                ptr_instance_field_initializer_record: 0,
                unk9: 0,
                unk10: 0,
                vtable_count: 0,
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
                unk1: ptr_vtable,
                unk2: 0,
                ptr_descriptor,
            },
        )?;

        let class = Self::from_raw(ptr_raw, core);
        class.register_class_getters(core, jvm, functions)?;

        Ok(class)
    }

    pub fn raw(&self) -> Result<RawJavaClass> {
        read_generic(&self.core, self.ptr_raw)
    }

    pub fn descriptor(&self) -> Result<RawJavaClassDescriptor> {
        read_generic(&self.core, self.raw()?.ptr_descriptor)
    }

    pub fn methods(&self) -> Result<Vec<JavaMethod>> {
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

    pub fn ptr_vtable(&self) -> Result<u32> {
        Ok(self.raw()?.unk1)
    }

    pub fn ptr_static_fields(&self) -> Result<u32> {
        Ok(self.ptr_raw + size_of::<RawJavaClass>() as u32)
    }

    pub async fn initialize_class_object(&self, jvm: &Jvm, class_object: &mut Box<dyn ClassInstance>) -> Result<()> {
        jvm.put_field(
            class_object,
            CLASS_NATIVE_NAME_FIELD,
            WORD_FIELD_DESCRIPTOR,
            self.descriptor()?.ptr_name as i32,
        )
        .await
        .map_err(|error| match error {
            JavaError::JavaException(instance) => WieError::JavaException(JavaValueCodec::new(&self.core).object_to_raw(&*instance)),
        })
    }

    pub fn instance_field_word_count(&self) -> Result<usize> {
        Ok(self.descriptor()?.instance_field_word_count as usize)
    }

    pub async fn prepare_generated(&mut self, core: &mut ArmCore, jvm: &Jvm) -> Result<()> {
        self.patch_declared_instance_field_word_indices()?;

        let class_name = ClassDefinition::name(self);
        let super_class_name = ClassDefinition::super_class_name(self);
        let parent_class = if let Some(parent_name) = &super_class_name {
            Some(
                jvm.resolve_class(parent_name)
                    .await
                    .map_err(|error| match error {
                        JavaError::JavaException(instance) => WieError::JavaException(JavaValueCodec::new(core).object_to_raw(&*instance)),
                    })?
                    .definition
                    .as_any()
                    .downcast_ref::<JavaClassDefinition>()
                    .unwrap()
                    .clone(),
            )
        } else {
            None
        };
        let descriptor = self.descriptor()?;
        let declared_methods = self.methods()?;
        let virtual_methods = if descriptor.ptr_vtable != 0 {
            let parent_methods = if let Some(parent_class) = &parent_class {
                parent_class.vtable_entries(jvm).await?
            } else {
                Vec::new()
            };
            JavaVtable::build_from_compiler_vtable(
                core,
                descriptor.ptr_vtable,
                descriptor.vtable_count as usize,
                &parent_methods,
                &declared_methods,
            )?
        } else {
            JavaVtable::build_methods(jvm, &class_name, parent_class.as_ref(), &declared_methods).await?
        };

        self.set_vtable_entries(&virtual_methods)
    }

    pub fn set_vtable_entries(&self, entries: &[JavaVtableEntry]) -> Result<()> {
        let mut core = self.core.clone();
        let ptr_vtable = JavaVtable::allocate(&mut core, entries.len())?;
        JavaVtable::write(&mut core, ptr_vtable, self.ptr_raw, entries)?;
        let mut raw = self.raw()?;
        raw.unk1 = ptr_vtable;
        write_generic(&mut core, self.ptr_raw, raw)
    }

    pub fn set_link_state(&self, link_state: u16) -> Result<()> {
        write_generic(
            &mut self.core.clone(),
            self.raw()?.ptr_descriptor + offset_of!(RawJavaClassDescriptor, link_state) as u32,
            link_state,
        )
    }

    pub fn patch_declared_instance_field_word_indices(&self) -> Result<()> {
        let fields = self.fields()?;
        let own_word_count = fields
            .iter()
            .filter(|field| !field.access_flags().contains(FieldAccessFlags::STATIC))
            .map(|field| if field.descriptor() == "J" || field.descriptor() == "D" { 2 } else { 1 })
            .sum::<usize>();
        let total_word_count = self.instance_field_word_count()?;
        let base = total_word_count.checked_sub(own_word_count).ok_or_else(|| {
            WieError::FatalError(format!(
                "LGT class {} has more declared field words than total field words",
                ClassDefinition::name(self)
            ))
        })? as u32;

        let mut relative_word_index = 0u32;
        let mut declared_fields = Vec::new();
        for field in fields
            .into_iter()
            .filter(|field| !field.access_flags().contains(FieldAccessFlags::STATIC))
        {
            let width = if field.descriptor() == "J" || field.descriptor() == "D" { 2 } else { 1 };
            let observed = field.word_index()?;
            declared_fields.push((field, observed, relative_word_index, base + relative_word_index));
            relative_word_index += width;
        }

        let is_relative = declared_fields.iter().all(|(_, observed, relative, _)| observed == relative);
        let is_absolute = declared_fields.iter().all(|(_, observed, _, absolute)| observed == absolute);
        if is_absolute {
            return Ok(());
        }
        if !is_relative {
            let observed = declared_fields.iter().map(|(_, index, _, _)| *index).collect::<Vec<_>>();
            let relative = declared_fields.iter().map(|(_, _, index, _)| *index).collect::<Vec<_>>();
            let absolute = declared_fields.iter().map(|(_, _, _, index)| *index).collect::<Vec<_>>();
            return Err(WieError::FatalError(format!(
                "LGT class {} has invalid declared instance field word indices {observed:?}; expected relative {relative:?} or absolute {absolute:?}",
                ClassDefinition::name(self)
            )));
        }

        for (field, _, _, absolute) in declared_fields {
            write_generic(
                &mut self.core.clone(),
                field.ptr_raw + offset_of!(RawJavaField, word_index) as u32,
                absolute,
            )?;
        }

        Ok(())
    }

    fn register_class_getters(&self, core: &mut ArmCore, jvm: &Jvm, functions: JavaSvcFunctions) -> Result<()> {
        let mut descriptor = self.descriptor()?;
        let get_initialized_class_id = self.raw()?.ptr_descriptor + offset_of!(RawJavaClassDescriptor, fn_get_initialized_class) as u32;
        let get_class_id = self.raw()?.ptr_descriptor + offset_of!(RawJavaClassDescriptor, fn_get_class) as u32;

        for (id, initialize) in [(get_initialized_class_id, true), (get_class_id, false)] {
            let proxy = RegisteredFunctionHolder::new(
                JavaClassGetterProxy {
                    jvm: jvm.clone(),
                    class: self.clone(),
                    initialize,
                },
                &(),
            );
            functions.lock().insert(id, Arc::new(Box::new(proxy) as Box<dyn RegisteredFunction>));
            let target = core.make_svc_stub(SVC_CATEGORY_JAVA, id)?;
            if initialize {
                descriptor.fn_get_initialized_class = target;
            } else {
                descriptor.fn_get_class = target;
            }
        }

        write_generic(core, self.raw()?.ptr_descriptor, descriptor)
    }

    pub async fn vtable_entries(&self, jvm: &Jvm) -> Result<Vec<JavaVtableEntry>> {
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

        let known_classes = hierarchy
            .into_iter()
            .map(|class| Ok((ClassDefinition::name(&class), class.methods()?)))
            .collect::<Result<Vec<_>>>()?;
        JavaVtable::read(&self.core, self.ptr_vtable()?, &known_classes)
    }
}

struct JavaClassGetterProxy {
    jvm: Jvm,
    class: JavaClassDefinition,
    initialize: bool,
}

#[async_trait::async_trait]
impl EmulatedFunction<(), u32, ()> for JavaClassGetterProxy {
    async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<u32> {
        let class_name = ClassDefinition::name(&self.class);
        let class = self.jvm.resolve_class(&class_name).await.map_err(|error| match error {
            JavaError::JavaException(instance) => WieError::JavaException(JavaValueCodec::new(core).object_to_raw(&*instance)),
        })?;
        let mut java_class = class.java_class();
        self.class.initialize_class_object(&self.jvm, &mut java_class).await?;

        if self.initialize {
            self.jvm.ensure_initialized(&class).await.map_err(|error| match error {
                JavaError::JavaException(instance) => WieError::JavaException(JavaValueCodec::new(core).object_to_raw(&*instance)),
            })?;
            self.jvm
                .put_field(&mut java_class, CLASS_INITIALIZATION_STATE_FIELD, WORD_FIELD_DESCRIPTOR, 5i32)
                .await
                .map_err(|error| match error {
                    JavaError::JavaException(instance) => WieError::JavaException(JavaValueCodec::new(core).object_to_raw(&*instance)),
                })?;
        }

        Ok(JavaValueCodec::new(core).object_to_raw(&*java_class))
    }
}

#[async_trait::async_trait]
impl ClassDefinition for JavaClassDefinition {
    fn name(&self) -> String {
        String::from_utf8(read_null_terminated_string_bytes(&self.core, self.descriptor().unwrap().ptr_name).unwrap()).unwrap()
    }

    fn super_class_name(&self) -> Option<String> {
        let ptr_name = self.descriptor().unwrap().ptr_super_class_name;
        (ptr_name != 0).then(|| String::from_utf8(read_null_terminated_string_bytes(&self.core, ptr_name).unwrap()).unwrap())
    }

    fn interface_names(&self) -> Vec<String> {
        let ptr_names = self.descriptor().unwrap().ptr_interface_names;
        if ptr_names == 0 {
            return Vec::new();
        }

        let count: u32 = read_generic(&self.core, ptr_names).unwrap();
        (0..count as usize)
            .map(|index| {
                let ptr_name = read_generic(&self.core, ptr_names + ((index + 1) * size_of::<u32>()) as u32).unwrap();
                String::from_utf8(read_null_terminated_string_bytes(&self.core, ptr_name).unwrap()).unwrap()
            })
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
        let raw_field = field.raw().unwrap();
        let declaring_class = Self::from_raw(raw_field.ptr_class, &self.core);
        let address = declaring_class.ptr_static_fields().unwrap() + raw_field.word_index * size_of::<LgtJvmWord>() as u32;
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
        let raw_field = field.raw().unwrap();
        let declaring_class = Self::from_raw(raw_field.ptr_class, &self.core);
        let address = declaring_class.ptr_static_fields().unwrap() + raw_field.word_index * size_of::<LgtJvmWord>() as u32;
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
