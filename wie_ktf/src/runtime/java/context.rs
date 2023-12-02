mod array_class_instance;
mod class;
mod class_instance;
mod class_loader;
mod field;
mod method;
mod name;
mod vtable_builder;

use alloc::{borrow::ToOwned, boxed::Box, format, vec, vec::Vec};
use core::mem::size_of;

use anyhow::Context;
use bytemuck::{Pod, Zeroable};

use wie_backend::{
    task::{self, SleepFuture},
    AsyncCallable, Backend,
};
use wie_base::util::{read_generic, read_null_terminated_table, write_generic};
use wie_core_arm::{Allocator, ArmCore, PEB_BASE};
use wie_impl_java::{r#impl::java::lang::Object, Array, JavaContext, JavaError, JavaMethodBody, JavaObjectProxy, JavaResult, JavaWord};

use crate::runtime::KtfPeb;

pub use self::name::JavaFullName;
use self::{
    array_class_instance::JavaArrayClassInstance, class::JavaClass, class_instance::JavaClassInstance, class_loader::ClassLoader, field::JavaField,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaContextData {
    pub classes_base: u32,
    pub ptr_vtables_base: u32,
    pub fn_get_class: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaExceptionHandler {
    ptr_method: u32,
    ptr_this: u32,
    ptr_old_handler: u32,
    current_state: u32, // state is returned on restore context
    unk3: u32,
    ptr_functions: u32, // function table to restore context and unk
    context: [u32; 11], // r4-lr
}

pub struct KtfJavaContext<'a> {
    core: &'a mut ArmCore,
    backend: &'a mut Backend,
}

impl<'a> KtfJavaContext<'a> {
    pub fn init(core: &mut ArmCore, ptr_vtables_base: u32, fn_get_class: u32) -> JavaResult<u32> {
        let classes_base = Allocator::alloc(core, 0x1000)?;

        let ptr_java_context_data = Allocator::alloc(core, size_of::<JavaContextData>() as _)?;
        write_generic(
            core,
            ptr_java_context_data,
            JavaContextData {
                classes_base,
                ptr_vtables_base,
                fn_get_class,
            },
        )?;

        Ok(ptr_java_context_data)
    }

    pub fn new(core: &'a mut ArmCore, backend: &'a mut Backend) -> Self {
        Self { core, backend }
    }

    pub async fn load_class_by_name(&mut self, ptr_target: u32, name: &str) -> JavaResult<()> {
        let class = ClassLoader::get_or_load_class(self, name).await?;

        write_generic(self.core, ptr_target, class.ptr_raw)?;

        Ok(())
    }

    pub async fn register_class(&mut self, class: &JavaClass) -> JavaResult<()> {
        let context_data = self.read_context_data()?;
        let ptr_classes = read_null_terminated_table(self.core, context_data.classes_base)?;
        if ptr_classes.contains(&class.ptr_raw) {
            return Ok(());
        }

        write_generic(
            self.core,
            context_data.classes_base + (ptr_classes.len() * size_of::<u32>()) as u32,
            class.ptr_raw,
        )?;

        let clinit = class.method(
            self,
            &JavaFullName {
                tag: 0,
                name: "<clinit>".into(),
                signature: "()V".into(),
            },
        )?;

        if let Some(x) = clinit {
            tracing::trace!("Call <clinit>");

            x.invoke(self, &[]).await?;
        }

        Ok(())
    }

    pub fn class_from_raw(&self, ptr_class: u32) -> JavaClass {
        JavaClass::from_raw(ptr_class)
    }

    fn get_vtable_index(&mut self, class: &JavaClass) -> anyhow::Result<u32> {
        let context_data = self.read_context_data()?;
        let ptr_vtables = read_null_terminated_table(self.core, context_data.ptr_vtables_base)?;

        let ptr_vtable = class.ptr_vtable(self)?;

        for (index, &current_ptr_vtable) in ptr_vtables.iter().enumerate() {
            if ptr_vtable == current_ptr_vtable {
                return Ok(index as _);
            }
        }

        let index = ptr_vtables.len();
        write_generic(self.core, context_data.ptr_vtables_base + (index * size_of::<u32>()) as u32, ptr_vtable)?;

        Ok(index as _)
    }

    fn read_context_data(&self) -> JavaResult<JavaContextData> {
        let peb: KtfPeb = read_generic(self.core, PEB_BASE)?;

        read_generic(self.core, peb.ptr_java_context_data)
    }
}

#[async_trait::async_trait(?Send)]
impl JavaContext for KtfJavaContext<'_> {
    async fn instantiate(&mut self, type_name: &str) -> JavaResult<JavaObjectProxy<Object>> {
        anyhow::ensure!(type_name.as_bytes()[0] != b'[', "Array class should not be instantiated here");

        let class_name = &type_name[1..type_name.len() - 1]; // L{};
        let class = ClassLoader::get_or_load_class(self, class_name).await?;

        let instance = JavaClassInstance::new(self, &class).await?;

        Ok(JavaObjectProxy::new(instance.ptr_raw as _))
    }

    async fn instantiate_array(&mut self, element_type_name: &str, count: JavaWord) -> JavaResult<JavaObjectProxy<Array>> {
        let array_type = format!("[{}", element_type_name);
        let array_class = ClassLoader::get_or_load_class(self, &array_type).await?;

        let instance = JavaArrayClassInstance::new(self, array_class, count).await?;

        Ok(JavaObjectProxy::new(instance.class_instance.ptr_raw as _))
    }

    fn destroy(&mut self, proxy: JavaObjectProxy<Object>) -> JavaResult<()> {
        let instance = JavaClassInstance::from_raw(proxy.ptr_instance as _);

        instance.destroy(self)
    }

    async fn call_method(&mut self, proxy: &JavaObjectProxy<Object>, method_name: &str, signature: &str, args: &[JavaWord]) -> JavaResult<JavaWord> {
        let instance = JavaClassInstance::from_raw(proxy.ptr_instance as _);
        let class = instance.class(self)?;

        tracing::trace!("Call {}::{}({})", class.name(self)?, method_name, signature);

        let mut params = vec![proxy.ptr_instance];
        params.extend(args);

        let method = class
            .method(
                self,
                &JavaFullName {
                    tag: 0,
                    name: method_name.to_owned(),
                    signature: signature.to_owned(),
                },
            )?
            .unwrap();

        Ok(method.invoke(self, &params).await? as _)
    }

    async fn call_static_method(&mut self, class_name: &str, method_name: &str, signature: &str, args: &[JavaWord]) -> JavaResult<JavaWord> {
        tracing::trace!("Call {}::{}({})", class_name, method_name, signature);

        let class = ClassLoader::get_or_load_class(self, class_name).await?;

        let method = class
            .method(
                self,
                &JavaFullName {
                    tag: 0,
                    name: method_name.to_owned(),
                    signature: signature.to_owned(),
                },
            )?
            .unwrap();

        Ok(method.invoke(self, args).await? as _)
    }

    fn backend(&mut self) -> &mut Backend {
        self.backend
    }

    fn get_field_id(&self, class_name: &str, field_name: &str, _signature: &str) -> JavaResult<JavaWord> {
        let class = ClassLoader::find_loaded_class(self, class_name)?.context("No such class")?;

        let field = class.field(self, field_name)?.unwrap();

        // TODO signature comparison

        Ok(field.ptr_raw as _)
    }

    fn get_field_by_id(&self, instance: &JavaObjectProxy<Object>, id: JavaWord) -> JavaResult<JavaWord> {
        let instance = JavaClassInstance::from_raw(instance.ptr_instance as _);

        let field = JavaField::from_raw(id as _);

        field.read_value(self, instance)
    }

    fn put_field_by_id(&mut self, instance: &JavaObjectProxy<Object>, id: JavaWord, value: JavaWord) -> JavaResult<()> {
        let instance = JavaClassInstance::from_raw(instance.ptr_instance as _);

        let field = JavaField::from_raw(id as _);

        field.write_value(self, instance, value)
    }

    fn get_field(&self, instance: &JavaObjectProxy<Object>, field_name: &str) -> JavaResult<JavaWord> {
        let instance = JavaClassInstance::from_raw(instance.ptr_instance as _);
        let class = instance.class(self)?;
        let field = class.field(self, field_name)?.unwrap();

        field.read_value(self, instance)
    }

    fn put_field(&mut self, instance: &JavaObjectProxy<Object>, field_name: &str, value: JavaWord) -> JavaResult<()> {
        let instance = JavaClassInstance::from_raw(instance.ptr_instance as _);
        let class = instance.class(self)?;
        let field = class.field(self, field_name)?.unwrap();

        field.write_value(self, instance, value)
    }

    fn get_static_field(&self, class_name: &str, field_name: &str) -> JavaResult<JavaWord> {
        let class = ClassLoader::find_loaded_class(self, class_name)?.with_context(|| format!("No such class {}", class_name))?;
        let field = class.field(self, field_name)?.unwrap();

        field.read_static_value(self)
    }

    fn put_static_field(&mut self, class_name: &str, field_name: &str, value: JavaWord) -> JavaResult<()> {
        let class = ClassLoader::find_loaded_class(self, class_name)?.with_context(|| format!("No such class {}", class_name))?;
        let field = class.field(self, field_name)?.unwrap();

        field.write_static_value(self, value)
    }

    fn store_array_i32(&mut self, array: &JavaObjectProxy<Array>, offset: JavaWord, values: &[i32]) -> JavaResult<()> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _);
        instance.store_array(self, offset, values)
    }

    fn load_array_i32(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<i32>> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _);
        instance.load_array(self, offset, count)
    }

    fn store_array_i16(&mut self, array: &JavaObjectProxy<Array>, offset: JavaWord, values: &[i16]) -> JavaResult<()> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _);
        instance.store_array(self, offset, values)
    }

    fn load_array_i16(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<i16>> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _);
        instance.load_array(self, offset, count)
    }

    fn store_array_i8(&mut self, array: &JavaObjectProxy<Array>, offset: JavaWord, values: &[i8]) -> JavaResult<()> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _);
        instance.store_array(self, offset, values)
    }

    fn load_array_i8(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<i8>> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _);
        instance.load_array(self, offset, count)
    }

    fn array_element_size(&self, array: &JavaObjectProxy<Array>) -> JavaResult<JavaWord> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _);

        instance.array_element_size(self)
    }

    fn array_length(&self, array: &JavaObjectProxy<Array>) -> JavaResult<JavaWord> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _);

        instance.array_length(self)
    }

    fn spawn(&mut self, callback: JavaMethodBody) -> JavaResult<()> {
        struct SpawnProxy {
            core: ArmCore,
            backend: Backend,
            callback: JavaMethodBody,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, JavaError> for SpawnProxy {
            #[allow(clippy::await_holding_refcell_ref)] // We manually drop RefMut https://github.com/rust-lang/rust-clippy/issues/6353
            async fn call(mut self) -> Result<u32, JavaError> {
                let mut context = KtfJavaContext::new(&mut self.core, &mut self.backend);

                Ok(self.callback.call(&mut context, &[]).await? as _)
            }
        }

        let backend = self.backend.clone();

        self.core.spawn(SpawnProxy {
            core: self.core.clone(),
            backend,
            callback,
        });

        Ok(())
    }

    fn sleep(&mut self, duration: u64) -> SleepFuture {
        let until = self.backend.time().now() + duration;

        task::sleep(until)
    }
}
