mod class;
mod class_loader;
mod field;
mod method;
mod name;
mod vtable_builder;

use alloc::{borrow::ToOwned, boxed::Box, format, vec, vec::Vec};
use core::{iter, mem::size_of};

use anyhow::Context;
use bytemuck::{cast_slice, cast_vec, Pod, Zeroable};
use num_traits::FromBytes;

use wie_backend::{
    task::{self, SleepFuture},
    AsyncCallable, Backend,
};
use wie_base::util::{read_generic, write_generic, ByteRead, ByteWrite};
use wie_core_arm::{Allocator, ArmCore, PEB_BASE};
use wie_impl_java::{r#impl::java::lang::Object, Array, JavaContext, JavaError, JavaMethodBody, JavaObjectProxy, JavaResult, JavaWord};

use crate::runtime::KtfPeb;

pub use self::name::JavaFullName;
use self::{class::JavaClass, class_loader::ClassLoader, field::JavaField};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct JavaClassInstance {
    ptr_fields: u32,
    ptr_class: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaClassInstanceFields {
    vtable_index: u32, // left shifted by 5
    fields: [u32; 1],
}

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

    pub async fn instantiate_from_class(&mut self, class: JavaClass) -> JavaResult<JavaObjectProxy<Object>> {
        let field_size = class.field_size(self)?;

        let proxy = self.instantiate_inner(&class, field_size).await?;

        tracing::trace!("Instantiated {} at {:#x}", class.name(self)?, proxy.ptr_instance);

        Ok(proxy)
    }

    pub async fn instantiate_array_from_ptr_class(&mut self, array_class: JavaClass, count: JavaWord) -> JavaResult<JavaObjectProxy<Array>> {
        let proxy = self.instantiate_array_inner(&array_class, count).await?;

        tracing::trace!("Instantiated {} at {:#x}", array_class.name(self)?, proxy.ptr_instance);

        Ok(proxy)
    }

    pub async fn register_class(&mut self, class: &JavaClass) -> JavaResult<()> {
        let context_data = self.read_context_data()?;
        let ptr_classes = self.read_null_terminated_table(context_data.classes_base)?;
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

    async fn instantiate_inner(&mut self, class: &JavaClass, field_size: JavaWord) -> JavaResult<JavaObjectProxy<Object>> {
        let ptr_instance = Allocator::alloc(self.core, size_of::<JavaClassInstance>() as _)?;
        let ptr_fields = Allocator::alloc(self.core, (field_size + 4) as _)?;

        let zero = iter::repeat(0).take((field_size + 4) as _).collect::<Vec<_>>();
        self.core.write_bytes(ptr_fields, &zero)?;

        let vtable_index = self.get_vtable_index(class)?;

        write_generic(
            self.core,
            ptr_instance,
            JavaClassInstance {
                ptr_fields,
                ptr_class: class.ptr_raw,
            },
        )?;
        write_generic(self.core, ptr_fields, (vtable_index * 4) << 5)?;

        tracing::trace!("Instantiate {:#x}, vtable_index {:#x}", ptr_instance, vtable_index);

        let instance = JavaObjectProxy::<Object>::new(ptr_instance as _);

        Ok(instance)
    }

    async fn instantiate_array_inner(&mut self, class: &JavaClass, count: JavaWord) -> JavaResult<JavaObjectProxy<Array>> {
        let element_size = self.get_array_element_size(class)?;
        let proxy = self.instantiate_inner(class, count * element_size + 4).await?;
        let instance: JavaClassInstance = read_generic(self.core, proxy.ptr_instance as _)?;

        write_generic(self.core, instance.ptr_fields + 4, count as u32)?;

        Ok(proxy.cast())
    }

    fn get_array_element_size(&self, class: &JavaClass) -> JavaResult<JavaWord> {
        let class_name = class.name(self)?;

        assert!(class_name.starts_with('['), "Not an array class {}", class_name);

        if class_name.starts_with("[L") || class_name.starts_with("[[") {
            Ok(4)
        } else {
            let element = class_name.as_bytes()[1];
            Ok(match element {
                b'B' => 1,
                b'C' => 2,
                b'I' => 4,
                b'Z' => 1,
                b'S' => 2,
                b'J' => 8,
                _ => unimplemented!("get_array_element_size {}", class_name),
            })
        }
    }

    fn get_vtable_index(&mut self, class: &JavaClass) -> anyhow::Result<u32> {
        let context_data = self.read_context_data()?;
        let ptr_vtables = self.read_null_terminated_table(context_data.ptr_vtables_base)?;

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

    fn read_null_terminated_table(&self, base_address: u32) -> JavaResult<Vec<u32>> {
        let mut cursor = base_address;
        let mut result = Vec::new();
        loop {
            let item: u32 = read_generic(self.core, cursor)?;
            if item == 0 {
                break;
            }
            result.push(item);

            cursor += 4;
        }

        Ok(result)
    }

    fn write_null_terminated_table(&mut self, items: &[u32]) -> JavaResult<u32> {
        let base_address = Allocator::alloc(self.core, ((items.len() + 1) * size_of::<u32>()) as _)?;

        let mut cursor = base_address;
        for &item in items {
            write_generic(self.core, cursor, item)?;

            cursor += 4;
        }
        write_generic(self.core, cursor, 0u32)?;

        Ok(base_address)
    }

    fn read_context_data(&self) -> JavaResult<JavaContextData> {
        let peb: KtfPeb = read_generic(self.core, PEB_BASE)?;

        read_generic(self.core, peb.ptr_java_context_data)
    }

    fn load_array<T, const B: usize>(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<T>>
    where
        T: FromBytes<Bytes = [u8; B]> + Pod,
    {
        let array_length = self.array_length(array)?;
        if offset + count > array_length {
            anyhow::bail!("Array index out of bounds");
        }

        let instance: JavaClassInstance = read_generic(self.core, array.ptr_instance as _)?;
        let class = JavaClass::from_raw(instance.ptr_class);
        let items_offset = instance.ptr_fields + 8;

        let element_size = self.get_array_element_size(&class)?;
        assert!(element_size == core::mem::size_of::<T>() as _, "Incorrect element size");

        let values_raw = self
            .core
            .read_bytes(items_offset + (element_size * offset) as u32, (count * element_size) as _)?;
        if B != 1 {
            Ok(values_raw
                .chunks(element_size as _)
                .map(|x| T::from_le_bytes(x.try_into().unwrap()))
                .collect::<Vec<_>>())
        } else {
            Ok(cast_vec(values_raw))
        }
    }

    fn store_array<T>(&mut self, array: &JavaObjectProxy<Array>, offset: JavaWord, values: &[T]) -> JavaResult<()>
    where
        T: Pod,
    {
        let array_length = self.array_length(array)?;
        if offset + values.len() as JavaWord > array_length {
            anyhow::bail!("Array index out of bounds");
        }

        let instance: JavaClassInstance = read_generic(self.core, array.ptr_instance as _)?;
        let class = JavaClass::from_raw(instance.ptr_class);
        let items_offset = instance.ptr_fields + 8;

        let element_size = self.get_array_element_size(&class)?;
        assert!(element_size == core::mem::size_of::<T>() as _, "Incorrect element size");

        let values_u8 = cast_slice(values);

        self.core.write_bytes(items_offset + (element_size * offset) as u32, values_u8)
    }
}

#[async_trait::async_trait(?Send)]
impl JavaContext for KtfJavaContext<'_> {
    async fn instantiate(&mut self, type_name: &str) -> JavaResult<JavaObjectProxy<Object>> {
        anyhow::ensure!(type_name.as_bytes()[0] != b'[', "Array class should not be instantiated here");

        let class_name = &type_name[1..type_name.len() - 1]; // L{};
        let class = ClassLoader::get_or_load_class(self, class_name).await?;

        self.instantiate_from_class(class).await
    }

    async fn instantiate_array(&mut self, element_type_name: &str, count: JavaWord) -> JavaResult<JavaObjectProxy<Array>> {
        let array_type = format!("[{}", element_type_name);
        let array_class = ClassLoader::get_or_load_class(self, &array_type).await?;

        let proxy = self.instantiate_array_inner(&array_class, count).await?;

        tracing::trace!("Instantiated {} at {:#x}", array_type, proxy.ptr_instance);

        Ok(proxy)
    }

    fn destroy(&mut self, proxy: JavaObjectProxy<Object>) -> JavaResult<()> {
        let instance: JavaClassInstance = read_generic(self.core, proxy.ptr_instance as _)?;

        tracing::trace!("Destroying {:#x}", proxy.ptr_instance);

        Allocator::free(self.core, instance.ptr_fields)?;
        Allocator::free(self.core, proxy.ptr_instance as _)?;

        Ok(())
    }

    async fn call_method(
        &mut self,
        instance_proxy: &JavaObjectProxy<Object>,
        method_name: &str,
        signature: &str,
        args: &[JavaWord],
    ) -> JavaResult<JavaWord> {
        let instance: JavaClassInstance = read_generic(self.core, instance_proxy.ptr_instance as _)?;
        let class = JavaClass::from_raw(instance.ptr_class);

        tracing::trace!("Call {}::{}({})", class.name(self)?, method_name, signature);

        let mut params = vec![instance_proxy.ptr_instance];
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
        let instance: JavaClassInstance = read_generic(self.core, instance.ptr_instance as _)?;

        let field = JavaField::from_raw(id as _);

        field.read_value(self, instance)
    }

    fn put_field_by_id(&mut self, instance: &JavaObjectProxy<Object>, id: JavaWord, value: JavaWord) -> JavaResult<()> {
        let instance: JavaClassInstance = read_generic(self.core, instance.ptr_instance as _)?;

        let field = JavaField::from_raw(id as _);

        field.write_value(self, instance, value)
    }

    fn get_field(&self, instance: &JavaObjectProxy<Object>, field_name: &str) -> JavaResult<JavaWord> {
        let instance: JavaClassInstance = read_generic(self.core, instance.ptr_instance as _)?;
        let class = JavaClass::from_raw(instance.ptr_class);
        let field = class.field(self, field_name)?.unwrap();

        field.read_value(self, instance)
    }

    fn put_field(&mut self, instance: &JavaObjectProxy<Object>, field_name: &str, value: JavaWord) -> JavaResult<()> {
        let instance: JavaClassInstance = read_generic(self.core, instance.ptr_instance as _)?;
        let class = JavaClass::from_raw(instance.ptr_class);
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
        self.store_array(array, offset, values)
    }

    fn load_array_i32(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<i32>> {
        self.load_array(array, offset, count)
    }

    fn store_array_i16(&mut self, array: &JavaObjectProxy<Array>, offset: JavaWord, values: &[i16]) -> JavaResult<()> {
        self.store_array(array, offset, values)
    }

    fn load_array_i16(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<i16>> {
        self.load_array(array, offset, count)
    }

    fn store_array_i8(&mut self, array: &JavaObjectProxy<Array>, offset: JavaWord, values: &[i8]) -> JavaResult<()> {
        self.store_array(array, offset, values)
    }

    fn load_array_i8(&self, array: &JavaObjectProxy<Array>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<i8>> {
        self.load_array(array, offset, count)
    }

    fn array_element_size(&self, array: &JavaObjectProxy<Array>) -> JavaResult<JavaWord> {
        let instance: JavaClassInstance = read_generic(self.core, array.ptr_instance as _)?;
        let class = JavaClass::from_raw(instance.ptr_class);

        self.get_array_element_size(&class)
    }

    fn array_length(&self, array: &JavaObjectProxy<Array>) -> JavaResult<JavaWord> {
        let instance: JavaClassInstance = read_generic(self.core, array.ptr_instance as _)?;

        let result: u32 = read_generic(self.core, instance.ptr_fields + 4)?;

        Ok(result as _)
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
