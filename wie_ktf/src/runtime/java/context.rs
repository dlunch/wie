mod array_class;
mod array_class_instance;
mod class;
mod class_instance;
mod class_loader;
mod context_data;
mod field;
mod method;
mod name;
mod value;
mod vtable_builder;

use alloc::{borrow::ToOwned, boxed::Box, format, rc::Rc, vec::Vec};
use core::cell::RefCell;

use anyhow::Context;
use bytemuck::{Pod, Zeroable};

use jvm::{ArrayClass, Class, ClassInstanceRef, ClassRef, Field, JavaValue, Jvm, JvmDetail, JvmResult, ThreadContext, ThreadId};

use wie_backend::{
    task::{self, SleepFuture},
    AsyncCallable, Backend,
};
use wie_core_arm::ArmCore;
use wie_impl_java::{r#impl::java::lang::Object, Array, JavaContext, JavaError, JavaMethodBody, JavaObjectProxy, JavaResult, JavaWord};

pub use self::name::JavaFullName;
use self::{
    array_class::JavaArrayClass, array_class_instance::JavaArrayClassInstance, class::JavaClass, class_instance::JavaClassInstance,
    class_loader::ClassLoader, context_data::JavaContextData, field::JavaField, value::JavaValueExt,
};

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

struct KtfJvmDetail {
    core: ArmCore,
}

impl KtfJvmDetail {
    pub fn new(core: &ArmCore) -> Self {
        Self { core: core.clone() }
    }
}

#[async_trait::async_trait(?Send)]
impl JvmDetail for KtfJvmDetail {
    async fn load_class(&mut self, class_name: &str) -> JvmResult<Option<ClassRef>> {
        let class = ClassLoader::get_or_load_class(&mut self.core, class_name).await?;

        Ok(class.map(|x| Rc::new(RefCell::new(Box::new(x) as Box<dyn Class>))))
    }

    async fn load_array_class(&mut self, element_type_name: &str) -> JvmResult<Option<Box<dyn ArrayClass>>> {
        let class_name = format!("[{}", element_type_name);
        let class = JavaArrayClass::new(&mut self.core, &class_name).await?;

        Ok(Some(Box::new(class)))
    }

    fn get_class(&self, class_name: &str) -> JvmResult<Option<ClassRef>> {
        let class = JavaContextData::find_class(&self.core, class_name)?;

        Ok(class.map(|x| Rc::new(RefCell::new(Box::new(x) as Box<dyn Class>))))
    }

    fn thread_context(&mut self, _thread_id: ThreadId) -> &mut dyn ThreadContext {
        todo!()
    }
}

pub struct KtfJavaContext<'a> {
    core: &'a mut ArmCore,
    backend: &'a mut Backend,
    jvm: Jvm,
}

impl<'a> KtfJavaContext<'a> {
    pub fn init(core: &mut ArmCore, ptr_vtables_base: u32, fn_get_class: u32) -> JavaResult<u32> {
        JavaContextData::init(core, ptr_vtables_base, fn_get_class)
    }

    pub fn new(core: &'a mut ArmCore, backend: &'a mut Backend) -> Self {
        let jvm = Jvm::new(KtfJvmDetail::new(core));

        Self { core, backend, jvm }
    }

    pub async fn load_class(&mut self, name: &str) -> JavaResult<Option<JavaClass>> {
        ClassLoader::get_or_load_class(self.core, name).await
    }

    pub async fn load_array_class(&mut self, name: &str) -> JavaResult<Option<JavaArrayClass>> {
        ClassLoader::load_array_class(self.core, name).await
    }

    pub async fn register_class(core: &mut ArmCore, class: &JavaClass) -> JavaResult<()> {
        JavaContextData::register_class(core, class)?;

        let clinit = class.method(&JavaFullName {
            tag: 0,
            name: "<clinit>".into(),
            descriptor: "()V".into(),
        })?;

        if let Some(mut x) = clinit {
            tracing::trace!("Call <clinit>");

            class.invoke_static_method(&mut x, &[]).await?;
        }

        Ok(())
    }

    pub fn class_from_raw(&self, ptr_class: u32) -> JavaClass {
        JavaClass::from_raw(ptr_class, self.core)
    }
}

#[async_trait::async_trait(?Send)]
impl JavaContext for KtfJavaContext<'_> {
    fn jvm(&mut self) -> &mut Jvm {
        &mut self.jvm
    }

    fn instance_raw(&self, instance: &ClassInstanceRef) -> JavaWord {
        let instance = instance.borrow();
        let instance = instance.as_any().downcast_ref::<JavaClassInstance>().unwrap();

        instance.ptr_raw as _
    }

    fn instance_from_raw(&self, raw: JavaWord) -> ClassInstanceRef {
        Rc::new(RefCell::new(Box::new(JavaClassInstance::from_raw(raw as _, self.core))))
    }

    async fn instantiate(&mut self, type_name: &str) -> JavaResult<JavaObjectProxy<Object>> {
        anyhow::ensure!(type_name.as_bytes()[0] != b'[', "Array class should not be instantiated here");

        let class_name = &type_name[1..type_name.len() - 1]; // L{};

        let instance = self.jvm.instantiate_class(class_name).await?;
        let instance = instance.borrow();
        let instance = instance.as_any().downcast_ref::<JavaClassInstance>().unwrap();

        Ok(JavaObjectProxy::new(instance.ptr_raw as _))
    }

    async fn instantiate_array(&mut self, element_type_name: &str, count: JavaWord) -> JavaResult<JavaObjectProxy<Array>> {
        let instance = self.jvm.instantiate_array(element_type_name, count).await?;
        let instance = instance.borrow();
        let instance = instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap();

        Ok(JavaObjectProxy::new(instance.class_instance.ptr_raw as _))
    }

    fn destroy(&mut self, proxy: JavaObjectProxy<Object>) -> JavaResult<()> {
        let instance = JavaClassInstance::from_raw(proxy.ptr_instance as _, self.core);

        instance.destroy()
    }

    async fn call_method(&mut self, proxy: &JavaObjectProxy<Object>, method_name: &str, descriptor: &str, args: &[JavaWord]) -> JavaResult<JavaWord> {
        let instance = JavaClassInstance::from_raw(proxy.ptr_instance as _, self.core);
        let class = instance.class()?;

        tracing::trace!("Call {}::{}({})", class.name()?, method_name, descriptor);

        let mut method = class
            .method(&JavaFullName {
                tag: 0,
                name: method_name.to_owned(),
                descriptor: descriptor.to_owned(),
            })?
            .unwrap();

        Ok(instance.invoke_method(&mut method, args).await? as _)
    }

    async fn call_static_method(&mut self, class_name: &str, method_name: &str, descriptor: &str, args: &[JavaWord]) -> JavaResult<JavaWord> {
        tracing::trace!("Call {}::{}({})", class_name, method_name, descriptor);

        let class = self.load_class(class_name).await?.unwrap();

        let mut method = class
            .method(&JavaFullName {
                tag: 0,
                name: method_name.to_owned(),
                descriptor: descriptor.to_owned(),
            })?
            .unwrap();

        Ok(class.invoke_static_method(&mut method, args).await? as _)
    }

    fn backend(&mut self) -> &mut Backend {
        self.backend
    }

    fn get_field_id(&self, class_name: &str, field_name: &str, _descriptor: &str) -> JavaResult<JavaWord> {
        let class = JavaContextData::find_class(self.core, class_name)?.context("No such class")?;

        let field = class.field(field_name)?.unwrap();

        // TODO descriptor comparison

        Ok(field.ptr_raw as _)
    }

    fn get_field_by_id(&self, instance: &JavaObjectProxy<Object>, id: JavaWord) -> JavaResult<JavaWord> {
        let instance = JavaClassInstance::from_raw(instance.ptr_instance as _, self.core);

        let field = JavaField::from_raw(id as _, self.core);

        instance.read_field(&field)
    }

    fn put_field_by_id(&mut self, instance: &JavaObjectProxy<Object>, id: JavaWord, value: JavaWord) -> JavaResult<()> {
        let mut instance = JavaClassInstance::from_raw(instance.ptr_instance as _, self.core);

        let field = JavaField::from_raw(id as _, self.core);

        instance.write_field(&field, value)
    }

    fn get_field(&self, instance: &JavaObjectProxy<Object>, field_name: &str) -> JavaResult<JavaWord> {
        let instance = JavaClassInstance::from_raw(instance.ptr_instance as _, self.core);
        let class = instance.class()?;
        let field = class.field(field_name)?.unwrap();

        instance.read_field(&field)
    }

    fn put_field(&mut self, instance: &JavaObjectProxy<Object>, field_name: &str, value: JavaWord) -> JavaResult<()> {
        let instance = JavaClassInstance::from_raw(instance.ptr_instance as _, self.core);
        let field = instance.class()?.field(field_name)?.unwrap();

        self.jvm.put_field(
            &Rc::new(RefCell::new(Box::new(instance))),
            field_name,
            "",
            JavaValue::from_raw(value, &field.descriptor(), self.core),
        )?;

        Ok(())
    }

    fn store_array_i32(&mut self, array: &JavaObjectProxy<Array>, offset: JavaWord, values: &[i32]) -> JavaResult<()> {
        let mut instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _, self.core);
        instance.store_array(offset, values)
    }

    fn load_array_i32(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<i32>> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _, self.core);
        instance.load_array(offset, count)
    }

    fn store_array_i16(&mut self, array: &JavaObjectProxy<Array>, offset: JavaWord, values: &[i16]) -> JavaResult<()> {
        let mut instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _, self.core);
        instance.store_array(offset, values)
    }

    fn load_array_i16(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<i16>> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _, self.core);
        instance.load_array(offset, count)
    }

    fn store_array_i8(&mut self, array: &JavaObjectProxy<Array>, offset: JavaWord, values: &[i8]) -> JavaResult<()> {
        let mut instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _, self.core);
        instance.store_array(offset, values)
    }

    fn load_array_i8(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<i8>> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _, self.core);
        instance.load_array(offset, count)
    }

    fn array_element_size(&self, array: &JavaObjectProxy<Array>) -> JavaResult<JavaWord> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _, self.core);

        instance.array_element_size()
    }

    fn array_length(&self, array: &JavaObjectProxy<Array>) -> JavaResult<JavaWord> {
        let instance = JavaArrayClassInstance::from_raw(array.ptr_instance as _, self.core);

        instance.array_length()
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
