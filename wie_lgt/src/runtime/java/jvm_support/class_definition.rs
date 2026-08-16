use alloc::{boxed::Box, format, string::String, sync::Arc, vec, vec::Vec};
use core::{fmt, fmt::Debug, fmt::Formatter, mem::offset_of, mem::size_of, ops::Deref, ops::DerefMut};

use java_class_proto::JavaClassProto;
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassDefinition, ClassInstance, Field, JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use wipi_types::lgt::java::{
    LGT_JAVA_CLASS_SUPER_CLASS_IS_NAME, LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor,
    LgtJavaClassField as RawJavaField, LgtJavaClassFieldStorage as RawJavaClassFieldStorage, LgtJavaClassInstance as RawJavaClassInstance,
    LgtJavaClassMethod as RawJavaMethod, LgtJavaInterfaceReference as RawJavaInterfaceReference,
    LgtJavaInterfaceReferences as RawJavaInterfaceReferences,
};

use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, RegisteredFunction, RegisteredFunctionHolder};
use wie_jvm_support::native::NativeJavaValueCodec;
use wie_util::{
    ByteRead, ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes,
};

use crate::runtime::{
    SVC_CATEGORY_JAVA,
    java::{
        JavaSvcFunctions,
        abi::{CLASS_INITIALIZATION_STATE_FIELD, CLASS_NATIVE_NAME_FIELD, JAVA_ABI, WORD_FIELD_DESCRIPTOR},
    },
};

use super::{
    JavaClassInstance, JavaField, JavaMethod, JavaReferenceField, JavaStaticReferenceField, LgtJvmWord,
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
        let additional_fixed_fields = fixed_fields
            .iter()
            .filter(|fixed| {
                !field_protos.iter().any(|field| {
                    !field.access_flags.contains(FieldAccessFlags::STATIC) && field.name == fixed.name && field.descriptor == fixed.descriptor
                })
            })
            .collect::<Vec<_>>();

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

        let static_field_word_count = field_protos
            .iter()
            .filter(|field| field.access_flags.contains(FieldAccessFlags::STATIC))
            .map(|field| if field.descriptor == "J" || field.descriptor == "D" { 2 } else { 1 })
            .sum::<usize>();
        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;
        let class_fields_size = size_of::<RawJavaClassFieldStorage>() + static_field_word_count * size_of::<LgtJvmWord>();
        let ptr_class_fields = Allocator::alloc(core, class_fields_size as u32)?;
        let ptr_name = Allocator::alloc(core, (class_name.len() + 1) as u32)?;
        let ptr_super_class = parent_class.as_ref().map(|class| class.ptr_raw).unwrap_or(0);
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
        let field_count = field_protos.len() + additional_fixed_fields.len();
        let ptr_fields = if field_count == 0 {
            0
        } else {
            Allocator::alloc(core, (size_of::<u32>() + field_count * size_of::<RawJavaField>()) as u32)?
        };
        let ptr_descriptor = Allocator::alloc(core, size_of::<RawJavaClassDescriptor>() as u32)?;

        core.write_bytes(ptr_raw, &[0; size_of::<RawJavaClass>()])?;
        core.write_bytes(ptr_class_fields, &vec![0; class_fields_size])?;
        write_null_terminated_string_bytes(core, ptr_name, class_name.as_bytes())?;
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
            let word_count = if field.descriptor == "J" || field.descriptor == "D" { 2 } else { 1 };
            let word_index = if is_static {
                static_field_word_index
            } else if let Some(fixed) = fixed_fields
                .iter()
                .find(|fixed| fixed.name == field.name && fixed.descriptor == field.descriptor)
            {
                fixed.index as usize
            } else {
                instance_field_word_index
            };
            if is_static {
                static_field_word_index += word_count;
            } else {
                instance_field_word_index = instance_field_word_index.max(word_index + word_count);
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
        for (offset, field) in additional_fixed_fields.iter().enumerate() {
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
            let word_count = if field.descriptor == "J" || field.descriptor == "D" { 2 } else { 1 };
            instance_field_word_index = instance_field_word_index.max(field.index as usize + word_count);
        }
        let virtual_methods = if access_flags.contains(ClassAccessFlags::INTERFACE) {
            JavaVtable::build_interface_methods(&methods)?
        } else {
            JavaVtable::build_methods(jvm, class_name, parent_class.as_ref(), &methods).await?
        };
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
                ptr_super_class,
                ptr_interface_references: 0,
                instance_field_word_count: instance_field_word_index as u16,
                link_state: 0,
                unk7: 0,
                ptr_instance_reference_bitmap: 0,
                flags: 0,
                unk10: 0,
                vtable_count: virtual_methods.len() as u16,
                ptr_interface_names,
                fn_link_members: 0,
                fn_get_initialized_class: 0,
                fn_get_class: 0,
                ptr_methods,
                ptr_fields,
                ptr_class_fields,
                unk14: 0,
                static_field_word_count: static_field_word_count as u32,
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
        let ptr_class_fields = Allocator::alloc(core, size_of::<RawJavaClassFieldStorage>() as u32)?;
        core.write_bytes(ptr_class_fields, &[0; size_of::<RawJavaClassFieldStorage>()])?;
        let ptr_name = Allocator::alloc(core, (name.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_name, name.as_bytes())?;
        let ptr_super_class = parent_class.ptr_raw;
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
        write_generic(
            core,
            ptr_descriptor,
            RawJavaClassDescriptor {
                access_flags: (ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL).bits() as u32,
                ptr_next_class: 0,
                ptr_name,
                ptr_vtable: 0,
                ptr_super_class,
                ptr_interface_references: 0,
                instance_field_word_count: 0,
                link_state: 0,
                unk7: 0,
                ptr_instance_reference_bitmap: 0,
                flags: 0,
                unk10: 0,
                vtable_count: virtual_methods.len() as u16,
                ptr_interface_names,
                fn_link_members: 0,
                fn_get_initialized_class: 0,
                fn_get_class: 0,
                ptr_methods: 0,
                ptr_fields: 0,
                ptr_class_fields,
                unk14: 0,
                static_field_word_count: 0,
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
        // This prefix belongs to the LGT java/lang/Class ABI; it is not part of
        // the represented class's Java instance-field layout.
        Ok(self.descriptor()?.ptr_class_fields + size_of::<RawJavaClassFieldStorage>() as u32)
    }

    pub async fn bind_class_object_storage(&self, jvm: &Jvm, class_object: &mut Box<dyn ClassInstance>) -> Result<()> {
        let instance = class_object.as_any_mut().downcast_mut::<JavaClassInstance>().unwrap();
        let current_fields = instance.ptr_fields()?;
        let ptr_class_fields = self.descriptor()?.ptr_class_fields;

        // LGT AOT uses java/lang/Class.ptr_fields as one combined block: the class
        // object's five-word ABI prefix followed by the represented class's static
        // fields. Preserve RustJava's initialized class-object fields before rebinding
        // the object to this descriptor-owned storage.
        if current_fields != ptr_class_fields {
            let storage_size = instance.storage_size()?;
            let mut fields = vec![0; storage_size];
            let mut core = self.core.clone();
            core.read_bytes(current_fields, &mut fields)?;
            core.write_bytes(ptr_class_fields, &fields)?;
            write_generic(
                &mut core,
                instance.ptr_raw + offset_of!(RawJavaClassInstance, ptr_fields) as u32,
                ptr_class_fields,
            )?;
            Allocator::free(&mut core, current_fields, storage_size.max(size_of::<LgtJvmWord>()) as u32)?;
        }

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

    pub async fn prepare_generated(&mut self, core: &mut ArmCore, jvm: &Jvm, generated_classes: u32) -> Result<()> {
        self.patch_declared_instance_field_word_indices()?;

        let mut descriptor = self.descriptor()?;
        let class_name = ClassDefinition::name(self);
        let interface_names = if descriptor.ptr_interface_names != 0 {
            ClassDefinition::interface_names(self)
        } else if descriptor.ptr_interface_references != 0 {
            let references: RawJavaInterfaceReferences = read_generic(core, descriptor.ptr_interface_references)?;
            let mut interface_names = Vec::with_capacity(references.count as usize);
            let mut interface_name_pointers = Vec::with_capacity(references.count as usize);

            for index in 0..references.count as usize {
                let ptr_reference = read_generic(
                    core,
                    descriptor.ptr_interface_references + size_of::<RawJavaInterfaceReferences>() as u32 + (index * size_of::<u32>()) as u32,
                )?;
                let mut reference: RawJavaInterfaceReference = read_generic(core, ptr_reference)?;
                let mut ptr_generated_interface = 0;
                let last_bucket: u32 = read_generic(core, generated_classes)?;
                for bucket in 0..=last_bucket {
                    let mut ptr_class = read_generic(core, generated_classes + size_of::<u32>() as u32 + bucket * size_of::<u32>() as u32)?;
                    while ptr_class != 0 {
                        if ptr_class == reference.ptr_class_or_name {
                            ptr_generated_interface = ptr_class;
                            break;
                        }
                        let class: RawJavaClass = read_generic(core, ptr_class)?;
                        let class_descriptor: RawJavaClassDescriptor = read_generic(core, class.ptr_descriptor)?;
                        ptr_class = class_descriptor.ptr_next_class;
                    }
                    if ptr_generated_interface != 0 {
                        break;
                    }
                }
                if ptr_generated_interface == self.ptr_raw {
                    continue;
                }

                let ptr_name = if ptr_generated_interface != 0 {
                    let interface: RawJavaClass = read_generic(core, ptr_generated_interface)?;
                    let interface_descriptor: RawJavaClassDescriptor = read_generic(core, interface.ptr_descriptor)?;
                    interface_descriptor.ptr_name
                } else {
                    let ptr_name = reference.ptr_class_or_name;
                    let interface_name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_name)?)
                        .map_err(|error| WieError::FatalError(format!("Invalid LGT interface name: {error}")))?;
                    let interface_class = jvm.resolve_class(&interface_name).await.map_err(|error| match error {
                        JavaError::JavaException(instance) => WieError::JavaException(JavaValueCodec::new(core).object_to_raw(&*instance)),
                    })?;
                    reference.ptr_class_or_name = interface_class.definition.as_any().downcast_ref::<JavaClassDefinition>().unwrap().ptr_raw;
                    write_generic(core, ptr_reference, reference)?;
                    ptr_name
                };
                let interface_name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_name)?)
                    .map_err(|error| WieError::FatalError(format!("Invalid LGT interface name: {error}")))?;
                interface_names.push(interface_name);
                interface_name_pointers.push(ptr_name);
            }

            descriptor.ptr_interface_names = Allocator::alloc(core, (size_of::<u32>() + interface_name_pointers.len() * size_of::<u32>()) as u32)?;
            write_generic(core, descriptor.ptr_interface_names, interface_name_pointers.len() as u32)?;
            for (index, ptr_name) in interface_name_pointers.into_iter().enumerate() {
                write_generic(core, descriptor.ptr_interface_names + ((index + 1) * size_of::<u32>()) as u32, ptr_name)?;
            }
            interface_names
        } else {
            Vec::new()
        };
        let super_class_name = if descriptor.ptr_super_class == 0 {
            None
        } else if descriptor.flags & LGT_JAVA_CLASS_SUPER_CLASS_IS_NAME != 0 {
            Some(
                String::from_utf8(read_null_terminated_string_bytes(core, descriptor.ptr_super_class)?)
                    .map_err(|error| WieError::FatalError(format!("Invalid LGT superclass name: {error}")))?,
            )
        } else {
            ClassDefinition::super_class_name(self)
        };
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
        if descriptor.flags & LGT_JAVA_CLASS_SUPER_CLASS_IS_NAME != 0 {
            descriptor.ptr_super_class = parent_class.as_ref().unwrap().ptr_raw;
            descriptor.flags &= !LGT_JAVA_CLASS_SUPER_CLASS_IS_NAME;
        }
        let mut declared_methods = self.methods()?;
        let virtual_methods = if descriptor.ptr_vtable != 0 {
            let parent_methods = if let Some(parent_class) = &parent_class {
                parent_class.vtable_entries(jvm).await?
            } else {
                Vec::new()
            };

            let mut abi_classes = Vec::new();
            if let Some(class) = JAVA_ABI.class(&class_name) {
                abi_classes.push(class);
            }
            for interface_name in &interface_names {
                if let Some(class) = JAVA_ABI.class(interface_name) {
                    abi_classes.push(class);
                }
            }

            let mut aot_methods = Vec::new();
            for index in 0..descriptor.vtable_count as usize {
                let target = read_generic(core, descriptor.ptr_vtable + ((index + 1) * size_of::<u32>()) as u32)?;
                if target == 0 {
                    continue;
                }
                if parent_methods.get(index).is_some_and(|entry| entry.target == target) {
                    continue;
                }

                let method = if let Some(parent_method) = parent_methods.get(index).and_then(|entry| entry.method.as_ref()) {
                    Some((
                        parent_method.name(),
                        parent_method.descriptor(),
                        parent_method.access_flags() & !MethodAccessFlags::ABSTRACT,
                    ))
                } else {
                    abi_classes.iter().find_map(|class| {
                        class
                            .vtable
                            .iter()
                            .find(|method| method.index == index)
                            .map(|method| (method.name.clone(), method.descriptor.clone(), MethodAccessFlags::PUBLIC))
                    })
                };
                let Some((name, method_descriptor, access_flags)) = method else {
                    continue;
                };
                if declared_methods
                    .iter()
                    .any(|method| method.name() == name && method.descriptor() == method_descriptor)
                {
                    continue;
                }
                aot_methods.push((name, method_descriptor, access_flags, target));
            }

            if !aot_methods.is_empty() {
                let existing_count = declared_methods.len();
                let method_count = existing_count + aot_methods.len();
                let ptr_methods = Allocator::alloc(core, (size_of::<u32>() + method_count * size_of::<RawJavaMethod>()) as u32)?;
                write_generic(core, ptr_methods, method_count as u32)?;

                if existing_count != 0 {
                    let mut methods = vec![0; existing_count * size_of::<RawJavaMethod>()];
                    core.read_bytes(descriptor.ptr_methods + size_of::<u32>() as u32, &mut methods)?;
                    core.write_bytes(ptr_methods + size_of::<u32>() as u32, &methods)?;
                }

                for (offset, (name, method_descriptor, access_flags, target)) in aot_methods.into_iter().enumerate() {
                    let ptr_method = ptr_methods + size_of::<u32>() as u32 + ((existing_count + offset) * size_of::<RawJavaMethod>()) as u32;
                    declared_methods.push(JavaMethod::new_aot(
                        core,
                        ptr_method,
                        self.ptr_raw,
                        &name,
                        &method_descriptor,
                        access_flags,
                        target,
                    )?);
                }
                descriptor.ptr_methods = ptr_methods;
            }

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

        let static_field_word_count = descriptor.static_field_word_count as usize;
        let class_fields_size = size_of::<RawJavaClassFieldStorage>() + static_field_word_count * size_of::<LgtJvmWord>();
        let ptr_class_fields = Allocator::alloc(core, class_fields_size as u32)?;
        core.write_bytes(ptr_class_fields, &vec![0; class_fields_size])?;

        let ptr_vtable = JavaVtable::allocate(core, virtual_methods.len())?;
        JavaVtable::write(core, ptr_vtable, self.ptr_raw, &virtual_methods)?;
        descriptor.ptr_class_fields = ptr_class_fields;
        descriptor.vtable_count = virtual_methods.len() as u16;
        write_generic(core, self.raw()?.ptr_descriptor, descriptor)?;
        let mut raw = self.raw()?;
        raw.unk1 = ptr_vtable;
        write_generic(core, self.ptr_raw, raw)
    }

    pub fn set_vtable_entries(&self, entries: &[JavaVtableEntry]) -> Result<()> {
        let mut core = self.core.clone();
        let ptr_vtable = JavaVtable::allocate(&mut core, entries.len())?;
        JavaVtable::write(&mut core, ptr_vtable, self.ptr_raw, entries)?;
        let mut descriptor = self.descriptor()?;
        descriptor.vtable_count = entries.len() as u16;
        write_generic(&mut core, self.raw()?.ptr_descriptor, descriptor)?;
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
        JavaVtable::read(&self.core, self.ptr_vtable()?, self.descriptor()?.vtable_count as usize, &known_classes)
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
        self.class.bind_class_object_storage(&self.jvm, &mut java_class).await?;

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
        let descriptor = self.descriptor().unwrap();
        if descriptor.ptr_super_class == 0 {
            return None;
        }

        let super_class: RawJavaClass = read_generic(&self.core, descriptor.ptr_super_class).unwrap();
        let super_descriptor: RawJavaClassDescriptor = read_generic(&self.core, super_class.ptr_descriptor).unwrap();
        Some(String::from_utf8(read_null_terminated_string_bytes(&self.core, super_descriptor.ptr_name).unwrap()).unwrap())
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
        let mut fields = self
            .fields()
            .unwrap()
            .into_iter()
            .map(|field| Box::new(field) as Box<dyn Field>)
            .collect::<Vec<_>>();
        let descriptor = self.descriptor().unwrap();
        if descriptor.ptr_instance_reference_bitmap != 0 {
            for word_index in 0..descriptor.instance_field_word_count as u32 {
                let byte: u8 = read_generic(&self.core, descriptor.ptr_instance_reference_bitmap + word_index / 8).unwrap();
                if byte & (0x80 >> (word_index % 8)) != 0 {
                    fields.push(Box::new(JavaReferenceField { word_index }));
                }
            }
        }

        // LGT AOT static storage is untyped, so conservatively retain words that
        // point to allocated instances with an intact object-header chain.
        for word_index in 0..descriptor.static_field_word_count {
            let ptr_instance: u32 = read_generic(
                &self.core,
                self.ptr_static_fields().unwrap() + word_index * size_of::<LgtJvmWord>() as u32,
            )
            .unwrap();
            if ptr_instance == 0 {
                continue;
            }
            if !Allocator::is_allocated(&self.core, ptr_instance, size_of::<RawJavaClassInstance>() as u32).unwrap() {
                continue;
            }

            let Ok(instance): Result<RawJavaClassInstance> = read_generic(&self.core, ptr_instance) else {
                continue;
            };
            let Ok(ptr_class): Result<u32> = read_generic(&self.core, instance.ptr_dispatch_table) else {
                continue;
            };
            let Ok(class): Result<RawJavaClass> = read_generic(&self.core, ptr_class) else {
                continue;
            };
            if class.unk1 == instance.ptr_dispatch_table {
                fields.push(Box::new(JavaStaticReferenceField {
                    ptr_class: self.ptr_raw,
                    word_index,
                }));
            }
        }
        fields
    }

    fn get_static_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let (address, field_type) = if let Some(field) = field.as_any().downcast_ref::<JavaField>() {
            let raw_field = field.raw().unwrap();
            let declaring_class = Self::from_raw(raw_field.ptr_class, &self.core);
            (
                declaring_class.ptr_static_fields().unwrap() + raw_field.word_index * size_of::<LgtJvmWord>() as u32,
                JavaType::parse(&field.descriptor()),
            )
        } else {
            let field = field.as_any().downcast_ref::<JavaStaticReferenceField>().unwrap();
            let declaring_class = Self::from_raw(field.ptr_class, &self.core);
            (
                declaring_class.ptr_static_fields().unwrap() + field.word_index * size_of::<LgtJvmWord>() as u32,
                JavaType::parse(&field.descriptor()),
            )
        };
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
