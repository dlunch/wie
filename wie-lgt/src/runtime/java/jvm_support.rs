mod array_class_definition;
mod array_class_instance;
mod class_definition;
mod class_instance;
mod field;
mod jvm_implementation;
mod method;
mod value;
mod vtable;

use alloc::{boxed::Box, format, string::String};

use jvm::{ClassDefinition, ClassInstance, JavaError, Jvm, Method};

use wie_backend::System;
use wie_core_arm::ArmCore;
use wie_jvm_support::{JvmImplementation, JvmSupport, native::NativeJavaValueCodec};
use wie_midp::get_protos as get_midp_protos;
use wie_util::{Result, WieError};
use wie_wipi_java::get_protos as get_wipi_java_protos;

use super::classes::net::wie::{CletWrapper, CletWrapperCard, CletWrapperContext, LgtClassLoader};

use jvm_implementation::LgtJvmImplementation;

use self::{
    array_class_definition::JavaArrayClassDefinition,
    array_class_instance::JavaArrayClassInstance,
    class_definition::JavaClassDefinition,
    class_instance::JavaClassInstance,
    field::{JavaField, JavaReferenceField, JavaStaticReferenceField},
    method::JavaMethod,
    value::JavaValueCodec,
};

type LgtJvmWord = u32;

pub struct LgtJvmSupport;

impl LgtJvmSupport {
    pub async fn init(core: &mut ArmCore, system: &System, jar_name: Option<&str>) -> Result<Jvm> {
        let protos = [get_midp_protos().into(), get_wipi_java_protos().into()];
        let implementation = LgtJvmImplementation::new(core)?;
        let jvm = JvmSupport::new_jvm(system, jar_name, Box::new(protos), &[], implementation.clone()).await?;

        let class = match implementation
            .define_class_rust(&jvm, LgtClassLoader::as_proto(), Box::new(core.clone()))
            .await
        {
            Ok(class) => class,
            Err(error) => return Err(JvmSupport::to_wie_err(&jvm, error).await),
        };
        if let Err(error) = jvm.register_class(class, None).await {
            return Err(JvmSupport::to_wie_err(&jvm, error).await);
        }

        let context = CletWrapperContext { core: core.clone() };
        for proto in [CletWrapper::as_proto(), CletWrapperCard::as_proto()] {
            let class = match implementation.define_class_rust(&jvm, proto, Box::new(context.clone()) as Box<_>).await {
                Ok(class) => class,
                Err(error) => return Err(JvmSupport::to_wie_err(&jvm, error).await),
            };
            jvm.register_class(class, None).await.unwrap();
        }

        Ok(jvm)
    }

    pub fn class_from_raw(core: &ArmCore, ptr_class: u32) -> JavaClassDefinition {
        JavaClassDefinition::from_raw(ptr_class, core)
    }

    pub fn class_instance_from_raw(core: &ArmCore, ptr_instance: u32) -> Box<dyn ClassInstance> {
        JavaValueCodec::new(core).object_from_raw(ptr_instance)
    }

    pub fn class_instance_raw(instance: &dyn ClassInstance) -> u32 {
        if let Some(instance) = instance.as_any().downcast_ref::<JavaClassInstance>() {
            instance.ptr_raw
        } else {
            instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap().class_instance.ptr_raw
        }
    }

    pub fn class_definition_raw(definition: &dyn ClassDefinition) -> u32 {
        if let Some(definition) = definition.as_any().downcast_ref::<JavaClassDefinition>() {
            definition.ptr_raw
        } else {
            definition.as_any().downcast_ref::<JavaArrayClassDefinition>().unwrap().class.ptr_raw
        }
    }

    pub fn field_word_index(jvm: &Jvm, class_name: &str, name: &str, descriptor: &str, is_static: bool) -> Result<u16> {
        let mut current_name = String::from(class_name);
        loop {
            let class = jvm
                .get_class(&current_name)
                .ok_or_else(|| WieError::FatalError(format!("Class not loaded while linking field: {current_name}")))?;
            if let Some(field) = class.definition.field(name, descriptor, is_static) {
                let field = field
                    .as_any()
                    .downcast_ref::<JavaField>()
                    .ok_or_else(|| WieError::FatalError(format!("Unsupported field implementation for {current_name}.{name}{descriptor}")))?;
                return u16::try_from(field.word_index()?)
                    .map_err(|_| WieError::FatalError(format!("Field word index does not fit LGT ABI for {current_name}.{name}{descriptor}")));
            }

            let Some(parent_name) = class.definition.super_class_name() else {
                return Err(WieError::FatalError(format!(
                    "Unable to resolve {} field {class_name}.{name}{descriptor}",
                    if is_static { "static" } else { "instance" }
                )));
            };
            current_name = parent_name;
        }
    }

    pub async fn virtual_method_index(jvm: &Jvm, class_name: &str, name: &str, descriptor: &str) -> Result<i16> {
        let class = jvm
            .get_class(class_name)
            .ok_or_else(|| WieError::FatalError(format!("Class not loaded while linking virtual method: {class_name}")))?;
        let definition = class
            .definition
            .as_any()
            .downcast_ref::<JavaClassDefinition>()
            .ok_or_else(|| WieError::FatalError(format!("Unsupported class implementation while linking virtual method: {class_name}")))?
            .clone();
        let methods = definition.vtable_entries(jvm).await?;
        if let Some(index) = methods.iter().position(|entry| {
            entry
                .method
                .as_ref()
                .is_some_and(|method| method.name() == name && method.descriptor() == descriptor)
        }) {
            return i16::try_from(index)
                .map_err(|_| WieError::FatalError(format!("Virtual method index does not fit LGT ABI for {class_name}.{name}{descriptor}")));
        }

        let methods = definition.runtime_methods(jvm).await?;
        let index = methods
            .iter()
            .position(|method| method.name() == name && method.descriptor() == descriptor)
            .ok_or_else(|| WieError::FatalError(format!("Unable to resolve virtual method {class_name}.{name}{descriptor}")))?;

        // Native callers use ptr_vtable + (index + 1) * 4. The first runtime
        // entry is at ptr_vtable - 4, so its index is -2; -1 is the class header.
        i16::try_from(-2 - index as i32)
            .map_err(|_| WieError::FatalError(format!("Virtual method index does not fit LGT ABI for {class_name}.{name}{descriptor}")))
    }

    pub async fn interface_dispatch_table(jvm: &mut Jvm, class_name: &str) -> Result<u32> {
        let class = jvm
            .resolve_class(class_name)
            .await
            .map_err(|JavaError::JavaException(instance)| WieError::JavaException(Self::class_instance_raw(&*instance)))?;
        let definition = class
            .definition
            .as_any()
            .downcast_ref::<JavaClassDefinition>()
            .ok_or_else(|| WieError::FatalError(format!("Unsupported interface class implementation: {class_name}")))?;

        definition.ptr_vtable()
    }

    pub fn non_virtual_method_target(jvm: &Jvm, class_name: &str, name: &str, descriptor: &str) -> Result<u32> {
        let mut current_name = String::from(class_name);
        loop {
            let class = jvm
                .get_class(&current_name)
                .ok_or_else(|| WieError::FatalError(format!("Class not loaded while linking direct method: {current_name}")))?;
            if let Some(method) = class
                .definition
                .method(name, descriptor, true)
                .or_else(|| class.definition.method(name, descriptor, false))
            {
                let method = method
                    .as_any()
                    .downcast_ref::<JavaMethod>()
                    .ok_or_else(|| WieError::FatalError(format!("Unsupported method implementation for {current_name}.{name}{descriptor}")))?;
                return method.target();
            }

            let Some(parent_name) = class.definition.super_class_name() else {
                return Err(WieError::FatalError(format!(
                    "Unable to resolve non-virtual method {class_name}.{name}{descriptor}"
                )));
            };
            current_name = parent_name;
        }
    }

    pub fn class_getter_targets(jvm: &Jvm, class_name: &str) -> Result<(u32, u32)> {
        let class = jvm
            .get_class(class_name)
            .ok_or_else(|| WieError::FatalError(format!("Class not loaded while linking class getters: {class_name}")))?;
        let definition = class
            .definition
            .as_any()
            .downcast_ref::<JavaClassDefinition>()
            .ok_or_else(|| WieError::FatalError(format!("Unsupported class implementation while linking class getters: {class_name}")))?;
        let descriptor = definition.descriptor()?;
        Ok((descriptor.fn_get_initialized_class, descriptor.fn_get_class))
    }

    pub async fn register_generated_class(
        core: &mut ArmCore,
        jvm: &Jvm,
        ptr_class: u32,
        generated_classes: u32,
        loader: Box<dyn ClassInstance>,
    ) -> Result<Box<dyn ClassInstance>> {
        let mut definition = JavaClassDefinition::from_raw(ptr_class, core);
        let class_name = ClassDefinition::name(&definition);
        if let Some(existing) = jvm.get_class(&class_name) {
            if existing
                .definition
                .as_any()
                .downcast_ref::<JavaClassDefinition>()
                .is_some_and(|definition| definition.ptr_raw == ptr_class)
                && definition.descriptor()?.link_state == 3
            {
                return Ok(existing.java_class());
            }
            return Err(wie_util::WieError::FatalError(format!(
                "Class {class_name} is already registered from a different definition"
            )));
        }

        let previous_link_state = definition.descriptor()?.link_state;
        definition.prepare_generated(core, jvm, generated_classes).await?;
        let registered_definition = definition.clone();
        let mut java_class = match jvm.register_class(Box::new(definition), Some(loader)).await {
            Ok(Some(java_class)) => java_class,
            Ok(None) => unreachable!(),
            Err(JavaError::JavaException(instance)) => {
                return Err(wie_util::WieError::JavaException(JavaValueCodec::new(core).object_to_raw(&*instance)));
            }
        };
        registered_definition.bind_class_object_storage(jvm, &mut java_class).await?;
        registered_definition.set_link_state(3)?;

        let fn_link_members = registered_definition.descriptor()?.fn_link_members;
        if fn_link_members != 0
            && let Err(error) = core.run_function::<()>(fn_link_members, &[Self::class_instance_raw(&*java_class)]).await
        {
            registered_definition.set_link_state(previous_link_state)?;
            return Err(error);
        }

        Ok(java_class)
    }
}
