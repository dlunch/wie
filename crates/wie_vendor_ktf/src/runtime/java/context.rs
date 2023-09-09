use alloc::{borrow::ToOwned, boxed::Box, format, string::String, vec, vec::Vec};
use core::{fmt::Display, iter, mem::size_of};

use bytemuck::{Pod, Zeroable};

use wie_backend::{
    task::{self, SleepFuture},
    AsyncCallable, Backend, Executor,
};
use wie_base::util::{read_generic, read_null_terminated_string, write_generic, write_null_terminated_string, ByteRead, ByteWrite};
use wie_core_arm::{Allocator, ArmCore, ArmCoreError, EmulatedFunction, EmulatedFunctionParam, PEB_BASE};
use wie_wipi_java::{
    get_class_proto, Array, JavaClassProto, JavaContext, JavaError, JavaFieldAccessFlag, JavaMethodBody, JavaMethodFlag, JavaObjectProxy, JavaResult,
    Object,
};

use crate::runtime::KtfPeb;

bitflags::bitflags! {
    struct JavaFieldAccessFlagBit: u32 {
        const NONE = 0;
        const STATIC = 8;
    }
}

impl JavaFieldAccessFlagBit {
    fn from_access_flag(access_flag: JavaFieldAccessFlag) -> JavaFieldAccessFlagBit {
        match access_flag {
            JavaFieldAccessFlag::NONE => JavaFieldAccessFlagBit::NONE,
            JavaFieldAccessFlag::STATIC => JavaFieldAccessFlagBit::STATIC,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaMethodFlagBit(u16);

bitflags::bitflags! {
    impl JavaMethodFlagBit: u16 {
        const NONE = 0;
        const STATIC = 8;
        const NATIVE = 0x100;
    }
}

impl JavaMethodFlagBit {
    fn from_flag(flag: JavaMethodFlag) -> JavaMethodFlagBit {
        match flag {
            JavaMethodFlag::NONE => JavaMethodFlagBit::NONE,
            JavaMethodFlag::STATIC => JavaMethodFlagBit::STATIC,
            JavaMethodFlag::NATIVE => JavaMethodFlagBit::NATIVE,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaClass {
    ptr_next: u32,
    unk1: u32,
    ptr_descriptor: u32,
    ptr_vtable: u32,
    vtable_count: u16,
    unk_flag: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaClassDescriptor {
    ptr_name: u32,
    unk1: u32,
    ptr_parent_class: u32,
    ptr_methods: u32,
    ptr_interfaces: u32,
    ptr_fields_or_element_type: u32, // for array class, this is element type
    method_count: u16,
    fields_size: u16,
    access_flag: u16,
    unk6: u16,
    unk7: u16,
    unk8: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaMethod {
    fn_body: u32,
    ptr_class: u32,
    fn_body_native: u32,
    ptr_name: u32,
    unk2: u16,
    unk3: u16,
    index_in_vtable: u16,
    flag: JavaMethodFlagBit,
    unk6: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaField {
    access_flag: u32,
    ptr_class: u32,
    ptr_name: u32,
    offset_or_value: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaClassInstance {
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
}

#[derive(Clone)]
pub struct JavaFullName {
    pub tag: u8,
    pub name: String,
    pub signature: String,
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

impl JavaFullName {
    pub fn from_ptr(core: &ArmCore, ptr: u32) -> JavaResult<Self> {
        let tag = read_generic(core, ptr)?;

        let value = read_null_terminated_string(core, ptr + 1)?;
        let value = value.split('+').collect::<Vec<_>>();

        Ok(JavaFullName {
            tag,
            name: value[1].into(),
            signature: value[0].into(),
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.tag);
        bytes.extend_from_slice(self.signature.as_bytes());
        bytes.push(b'+');
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.push(0);

        bytes
    }
}

impl Display for JavaFullName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.name.fmt(f)?;
        self.signature.fmt(f)?;
        write!(f, "@{}", self.tag)?;

        Ok(())
    }
}

impl PartialEq for JavaFullName {
    fn eq(&self, other: &Self) -> bool {
        self.signature == other.signature && self.name == other.name
    }
}

struct JavaVtableMethod {
    ptr_method: u32,
    name: JavaFullName,
}

struct JavaVtableBuilder {
    items: Vec<JavaVtableMethod>,
}

impl JavaVtableBuilder {
    pub fn new(context: &KtfJavaContext<'_>, ptr_parent_class: Option<u32>) -> JavaResult<Self> {
        let items = if let Some(x) = ptr_parent_class {
            Self::build_vtable(context, x)?
        } else {
            Vec::new()
        };

        Ok(Self { items })
    }

    pub fn add(&mut self, ptr_method: u32, name: &JavaFullName) -> usize {
        if let Some(index) = self.items.iter().position(|x| x.name == *name) {
            self.items[index] = JavaVtableMethod {
                ptr_method,
                name: name.to_owned(),
            };

            index
        } else {
            self.items.push(JavaVtableMethod {
                ptr_method,
                name: name.to_owned(),
            });

            self.items.len() - 1
        }
    }

    pub fn serialize(&self) -> Vec<u32> {
        self.items.iter().map(|x| x.ptr_method).collect()
    }

    fn build_vtable(context: &KtfJavaContext<'_>, ptr_class: u32) -> JavaResult<Vec<JavaVtableMethod>> {
        let mut class_hierarchy = Self::read_class_hierarchy(context, ptr_class)?;
        class_hierarchy.reverse();

        let mut vtable: Vec<JavaVtableMethod> = Vec::new();

        for ptr_class in class_hierarchy {
            let (_, class_descriptor, _) = context.read_ptr_class(ptr_class)?;

            let ptr_methods = context.read_null_terminated_table(class_descriptor.ptr_methods)?;

            let items = ptr_methods
                .into_iter()
                .map(|x| {
                    let method: JavaMethod = read_generic(context.core, x)?;
                    let name = JavaFullName::from_ptr(context.core, method.ptr_name)?;

                    Ok::<_, JavaError>(JavaVtableMethod { ptr_method: x, name })
                })
                .collect::<Result<Vec<_>, _>>()?;

            for item in items {
                if let Some(index) = vtable.iter().position(|x| x.name == item.name) {
                    vtable[index] = item;
                } else {
                    vtable.push(item);
                }
            }
        }

        Ok(vtable)
    }

    fn read_class_hierarchy(context: &KtfJavaContext<'_>, ptr_class: u32) -> JavaResult<Vec<u32>> {
        let mut result = vec![ptr_class];

        let mut current_class = ptr_class;
        loop {
            let (_, class_descriptor, _) = context.read_ptr_class(current_class)?;

            if class_descriptor.ptr_parent_class != 0 {
                result.push(class_descriptor.ptr_parent_class);

                current_class = class_descriptor.ptr_parent_class;
            } else {
                break;
            }
        }

        Ok(result)
    }
}

pub struct KtfJavaContext<'a> {
    core: &'a mut ArmCore,
    backend: &'a mut Backend,
}

impl<'a> KtfJavaContext<'a> {
    pub fn init(core: &mut ArmCore, ptr_param_2: u32) -> JavaResult<u32> {
        let ptr_vtables_base = ptr_param_2 + 12;
        let classes_base = Allocator::alloc(core, 0x1000)?;

        let ptr_java_context_data = Allocator::alloc(core, size_of::<JavaContextData>() as u32)?;
        write_generic(
            core,
            ptr_java_context_data,
            JavaContextData {
                classes_base,
                ptr_vtables_base,
            },
        )?;

        Ok(ptr_java_context_data)
    }

    pub fn new(core: &'a mut ArmCore, backend: &'a mut Backend) -> Self {
        Self { core, backend }
    }

    pub fn get_method(&mut self, ptr_class: u32, fullname: JavaFullName) -> JavaResult<u32> {
        let (_, class_descriptor, class_name) = self.read_ptr_class(ptr_class)?;

        let ptr_methods = self.read_null_terminated_table(class_descriptor.ptr_methods)?;
        for ptr_method in ptr_methods {
            let current_method: JavaMethod = read_generic(self.core, ptr_method)?;
            let current_fullname = JavaFullName::from_ptr(self.core, current_method.ptr_name)?;

            if current_fullname == fullname {
                return Ok(ptr_method);
            }
        }

        Err(anyhow::anyhow!("Can't find function {} from {}", fullname, class_name))
    }

    pub fn load_class(&mut self, ptr_target: u32, name: &str) -> JavaResult<()> {
        let ptr_class = self.get_ptr_class(name)?;

        write_generic(self.core, ptr_target, ptr_class)?;

        Ok(())
    }

    pub fn instantiate_from_ptr_class(&mut self, ptr_class: u32) -> JavaResult<JavaObjectProxy<Object>> {
        let (_, class_descriptor, class_name) = self.read_ptr_class(ptr_class)?;

        let proxy = self.instantiate_inner(ptr_class, class_descriptor.fields_size as u32)?;

        log::trace!("Instantiated {} at {:#x}", class_name, proxy.ptr_instance);

        Ok(proxy)
    }

    pub fn instantiate_array_from_ptr_class(&mut self, ptr_class_array: u32, count: u32) -> JavaResult<JavaObjectProxy<Array>> {
        let (_, _, class_name) = self.read_ptr_class(ptr_class_array)?;

        let proxy = self.instantiate_array_inner(ptr_class_array, count)?;

        log::trace!("Instantiated {} at {:#x}", class_name, proxy.ptr_instance);

        Ok(proxy)
    }

    fn instantiate_inner(&mut self, ptr_class: u32, fields_size: u32) -> JavaResult<JavaObjectProxy<Object>> {
        let ptr_instance = Allocator::alloc(self.core, size_of::<JavaClassInstance>() as u32)?;
        let ptr_fields = Allocator::alloc(self.core, fields_size + 4)?;

        let zero = iter::repeat(0).take((fields_size + 4) as usize).collect::<Vec<_>>();
        self.core.write_bytes(ptr_fields, &zero)?;

        let vtable_index = self.get_vtable_index(ptr_class)?;

        write_generic(self.core, ptr_instance, JavaClassInstance { ptr_fields, ptr_class })?;
        write_generic(self.core, ptr_fields, (vtable_index * 4) << 5)?;

        log::trace!("Instantiate {:#x}, vtable_index {:#x}", ptr_instance, vtable_index);

        Ok(JavaObjectProxy::<Object>::new(ptr_instance))
    }

    fn instantiate_array_inner(&mut self, ptr_class_array: u32, count: u32) -> JavaResult<JavaObjectProxy<Array>> {
        let element_size = self.get_array_element_size(ptr_class_array)?;
        let proxy = self.instantiate_inner(ptr_class_array, count * element_size + 4)?;
        let instance: JavaClassInstance = read_generic(self.core, proxy.ptr_instance)?;

        write_generic(self.core, instance.ptr_fields + 4, count)?;

        Ok(proxy.cast())
    }

    fn get_array_element_size(&self, ptr_class_array: u32) -> JavaResult<u32> {
        let (_, _, class_name) = self.read_ptr_class(ptr_class_array)?;

        if class_name.starts_with("[L") {
            Ok(4)
        } else {
            let element = class_name.as_bytes()[1];
            Ok(match element {
                b'B' => 1,
                b'C' => 1,
                b'I' => 4,
                b'Z' => 1,
                _ => unimplemented!("get_array_element_size {}", class_name),
            })
        }
    }

    fn get_vtable_index(&mut self, ptr_class: u32) -> anyhow::Result<u32> {
        let (class, _, class_name) = self.read_ptr_class(ptr_class)?;

        log::trace!("get_vtable_index {} {:#x} {:#x}", class_name, ptr_class, class.ptr_vtable);

        let context_data = self.read_context_data()?;
        let ptr_vtables = self.read_null_terminated_table(context_data.ptr_vtables_base)?;

        for (index, &ptr_vtable) in ptr_vtables.iter().enumerate() {
            if ptr_vtable == class.ptr_vtable {
                return Ok(index as u32);
            }
        }

        let index = ptr_vtables.len();
        write_generic(
            self.core,
            context_data.ptr_vtables_base + (index * size_of::<u32>()) as u32,
            class.ptr_vtable,
        )?;

        Ok(index as u32)
    }

    fn get_ptr_class(&mut self, name: &str) -> JavaResult<u32> {
        let context_data = self.read_context_data()?;
        let ptr_classes = self.read_null_terminated_table(context_data.classes_base)?;
        for ptr_class in ptr_classes {
            let (_, _, class_name) = self.read_ptr_class(ptr_class)?;

            if class_name == name {
                return Ok(ptr_class);
            }
        }

        // array class is created dynamically
        if name.as_bytes()[0] == b'[' {
            let ptr_class = self.load_array_class_into_vm(name)?;

            Ok(ptr_class)
        } else {
            let proto = get_class_proto(name).ok_or_else(|| anyhow::anyhow!("No such class {}", name))?;

            self.load_class_into_vm(name, proto)
        }
    }

    fn load_array_class_into_vm(&mut self, name: &str) -> JavaResult<u32> {
        let ptr_parent_class = self.get_ptr_class("java/lang/Object")?;
        let ptr_class = Allocator::alloc(self.core, size_of::<JavaClass>() as u32)?;

        /* TODO we need load class from client.bin
        let element_type_name = &name[1..];
        let element_type = if element_type_name.starts_with('L') {
            self.get_ptr_class(&element_type_name[1..element_type_name.len() - 1])?
        } else {
            0
        };
        */
        let element_type = 0;

        let ptr_name = Allocator::alloc(self.core, (name.len() + 1) as u32)?;
        write_null_terminated_string(self.core, ptr_name, name)?;

        let ptr_descriptor = Allocator::alloc(self.core, size_of::<JavaClassDescriptor>() as u32)?;
        write_generic(
            self.core,
            ptr_descriptor,
            JavaClassDescriptor {
                ptr_name,
                unk1: 0,
                ptr_parent_class,
                ptr_methods: 0,
                ptr_interfaces: 0,
                ptr_fields_or_element_type: element_type,
                method_count: 0,
                fields_size: 0,
                access_flag: 0x21, // ACC_PUBLIC | ACC_SUPER
                unk6: 0,
                unk7: 0,
                unk8: 0,
            },
        )?;

        write_generic(
            self.core,
            ptr_class,
            JavaClass {
                ptr_next: ptr_class + 4,
                unk1: 0,
                ptr_descriptor,
                ptr_vtable: 0,
                vtable_count: 0,
                unk_flag: 8,
            },
        )?;

        let context_data = self.read_context_data()?;
        let ptr_classes = self.read_null_terminated_table(context_data.classes_base)?;
        write_generic(
            self.core,
            context_data.classes_base + (ptr_classes.len() * size_of::<u32>()) as u32,
            ptr_class,
        )?;

        Ok(ptr_class)
    }

    fn load_class_into_vm(&mut self, name: &str, proto: JavaClassProto) -> JavaResult<u32> {
        let ptr_parent_class = proto.parent_class.map(|x| self.get_ptr_class(x)).transpose()?;
        let mut vtable_builder = JavaVtableBuilder::new(self, ptr_parent_class)?;

        let ptr_class = Allocator::alloc(self.core, size_of::<JavaClass>() as u32)?;

        let mut methods = Vec::new();
        for method in proto.methods.into_iter() {
            let full_name = JavaFullName {
                tag: 0,
                name: method.name,
                signature: method.signature,
            };
            let full_name_bytes = full_name.as_bytes();

            let ptr_name = Allocator::alloc(self.core, full_name_bytes.len() as u32)?;
            self.core.write_bytes(ptr_name, &full_name_bytes)?;

            let fn_method = self.register_java_method(method.body, method.flag == JavaMethodFlag::NATIVE)?;
            let (fn_body, fn_body_native) = if method.flag == JavaMethodFlag::NATIVE {
                (0, fn_method)
            } else {
                (fn_method, 0)
            };

            let ptr_method = Allocator::alloc(self.core, size_of::<JavaMethod>() as u32)?;
            let index_in_vtable = vtable_builder.add(ptr_method, &full_name) as u16;

            let flag = JavaMethodFlagBit::from_flag(method.flag);

            write_generic(
                self.core,
                ptr_method,
                JavaMethod {
                    fn_body,
                    ptr_class,
                    fn_body_native,
                    ptr_name,
                    unk2: 0,
                    unk3: 0,
                    index_in_vtable,
                    flag,
                    unk6: 0,
                },
            )?;

            methods.push(ptr_method);
        }
        let ptr_methods = self.write_null_terminated_table(&methods)?;

        let mut fields = Vec::new();
        for (index, field) in proto.fields.into_iter().enumerate() {
            let full_name = (JavaFullName {
                tag: 0,
                name: field.name,
                signature: field.signature,
            })
            .as_bytes();

            let ptr_name = Allocator::alloc(self.core, full_name.len() as u32)?;
            self.core.write_bytes(ptr_name, &full_name)?;

            let ptr_field = Allocator::alloc(self.core, size_of::<JavaField>() as u32)?;
            let offset_or_value = if field.access_flag == JavaFieldAccessFlag::STATIC {
                0
            } else {
                (index as u32) * 4
            };

            write_generic(
                self.core,
                ptr_field,
                JavaField {
                    access_flag: JavaFieldAccessFlagBit::from_access_flag(field.access_flag).bits(),
                    ptr_class,
                    ptr_name,
                    offset_or_value,
                },
            )?;

            fields.push(ptr_field);
        }
        let ptr_fields = self.write_null_terminated_table(&fields)?;

        let ptr_name = Allocator::alloc(self.core, (name.len() + 1) as u32)?;
        write_null_terminated_string(self.core, ptr_name, name)?;

        let ptr_descriptor = Allocator::alloc(self.core, size_of::<JavaClassDescriptor>() as u32)?;
        write_generic(
            self.core,
            ptr_descriptor,
            JavaClassDescriptor {
                ptr_name,
                unk1: 0,
                ptr_parent_class: ptr_parent_class.unwrap_or(0),
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
        let ptr_vtable = self.write_null_terminated_table(&vtable)?;

        write_generic(
            self.core,
            ptr_class,
            JavaClass {
                ptr_next: ptr_class + 4,
                unk1: 0,
                ptr_descriptor,
                ptr_vtable,
                vtable_count: vtable.len() as u16,
                unk_flag: 8,
            },
        )?;

        let context_data = self.read_context_data()?;
        let ptr_classes = self.read_null_terminated_table(context_data.classes_base)?;
        write_generic(
            self.core,
            context_data.classes_base + (ptr_classes.len() * size_of::<u32>()) as u32,
            ptr_class,
        )?;

        Ok(ptr_class)
    }

    fn register_java_method(&mut self, body: JavaMethodBody, native: bool) -> JavaResult<u32> {
        struct JavaMethodProxy {
            body: JavaMethodBody,
            native: bool,
        }

        impl JavaMethodProxy {
            pub fn new(body: JavaMethodBody, native: bool) -> Self {
                Self { body, native }
            }
        }

        #[async_trait::async_trait(?Send)]
        impl EmulatedFunction<(u32, u32, u32), ArmCoreError, Backend, u32> for JavaMethodProxy {
            async fn call(&self, core: &mut ArmCore, backend: &mut Backend) -> Result<u32, ArmCoreError> {
                let a1 = u32::get(core, 1);
                let a2 = u32::get(core, 2);
                let a3 = u32::get(core, 3);
                let a4 = u32::get(core, 4);
                let a5 = u32::get(core, 5);
                let a6 = u32::get(core, 6);

                let args = if self.native {
                    (0..6).map(|x| read_generic(core, a1 + x * 4)).collect::<JavaResult<Vec<u32>>>()?
                } else {
                    vec![a1, a2, a3, a4, a5, a6]
                };

                let mut context = KtfJavaContext::new(core, backend);

                let result = self.body.call(&mut context, &args).await?; // TODO do we need arg proxy?

                Ok(result)
            }
        }

        let proxy = JavaMethodProxy::new(body, native);

        self.core.register_function(proxy, self.backend)
    }

    fn read_ptr_class(&self, ptr_class: u32) -> JavaResult<(JavaClass, JavaClassDescriptor, String)> {
        let class: JavaClass = read_generic(self.core, ptr_class)?;
        let class_descriptor: JavaClassDescriptor = read_generic(self.core, class.ptr_descriptor)?;
        let class_name = read_null_terminated_string(self.core, class_descriptor.ptr_name)?;

        Ok((class, class_descriptor, class_name))
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
        let base_address = Allocator::alloc(self.core, ((items.len() + 1) * size_of::<u32>()) as u32)?;

        let mut cursor = base_address;
        for &item in items {
            write_generic(self.core, cursor, item)?;

            cursor += 4;
        }
        write_generic(self.core, cursor, 0u32)?;

        Ok(base_address)
    }

    fn get_ptr_field(&self, ptr_class: u32, field_name: &str) -> JavaResult<u32> {
        let (_, class_descriptor, class_name) = self.read_ptr_class(ptr_class)?;

        let ptr_fields = self.read_null_terminated_table(class_descriptor.ptr_fields_or_element_type)?;
        for ptr_field in ptr_fields {
            let field: JavaField = read_generic(self.core, ptr_field)?;

            let fullname = JavaFullName::from_ptr(self.core, field.ptr_name)?;
            if fullname.name == field_name {
                return Ok(ptr_field);
            }
        }

        Err(anyhow::anyhow!("Cannot find field {} from {}", field_name, class_name))
    }

    async fn call_method_inner(&mut self, ptr_class: u32, method_name: &str, signature: &str, args: &[u32]) -> JavaResult<u32> {
        let fullname = JavaFullName {
            tag: 0,
            name: method_name.to_owned(),
            signature: signature.to_owned(),
        };

        let ptr_method = self.get_method(ptr_class, fullname)?;

        let method: JavaMethod = read_generic(self.core, ptr_method)?;

        if method.flag.contains(JavaMethodFlagBit::NATIVE) {
            let arg_container = Allocator::alloc(self.core, (args.len() as u32) * 4)?;
            for (i, arg) in args.iter().enumerate() {
                write_generic(self.core, arg_container + (i * 4) as u32, *arg)?;
            }

            let result = self.core.run_function(method.fn_body_native, &[0, arg_container]).await;

            Allocator::free(self.core, arg_container)?;

            result
        } else {
            let mut params = vec![0];
            params.extend(args);

            self.core.run_function(method.fn_body, &params).await
        }
    }

    fn read_context_data(&self) -> JavaResult<JavaContextData> {
        let peb: KtfPeb = read_generic(self.core, PEB_BASE)?;

        read_generic(self.core, peb.ptr_java_context_data)
    }
}

#[async_trait::async_trait(?Send)]
impl JavaContext for KtfJavaContext<'_> {
    fn instantiate(&mut self, type_name: &str) -> JavaResult<JavaObjectProxy<Object>> {
        if type_name.as_bytes()[0] == b'[' {
            return Err(anyhow::anyhow!("Array class should not be instantiated here"));
        }
        let class_name = &type_name[1..type_name.len() - 1]; // L{};
        let ptr_class = self.get_ptr_class(class_name)?;

        let (_, class_descriptor, _) = self.read_ptr_class(ptr_class)?;

        let proxy = self.instantiate_inner(ptr_class, class_descriptor.fields_size as u32)?;

        log::trace!("Instantiated {} at {:#x}", class_name, proxy.ptr_instance);

        Ok(proxy)
    }

    fn instantiate_array(&mut self, element_type_name: &str, count: u32) -> JavaResult<JavaObjectProxy<Array>> {
        let array_type = format!("[{}", element_type_name);
        let ptr_class_array = self.get_ptr_class(&array_type)?;

        let proxy = self.instantiate_array_inner(ptr_class_array, count)?;

        log::trace!("Instantiated {} at {:#x}", array_type, proxy.ptr_instance);

        Ok(proxy)
    }

    fn destroy(&mut self, proxy: JavaObjectProxy<Object>) -> JavaResult<()> {
        let instance: JavaClassInstance = read_generic(self.core, proxy.ptr_instance)?;

        log::trace!("Destroying {:#x}", proxy.ptr_instance);

        Allocator::free(self.core, instance.ptr_fields)?;
        Allocator::free(self.core, proxy.ptr_instance)?;

        Ok(())
    }

    async fn call_method(&mut self, instance_proxy: &JavaObjectProxy<Object>, name: &str, signature: &str, args: &[u32]) -> JavaResult<u32> {
        let instance: JavaClassInstance = read_generic(self.core, instance_proxy.ptr_instance)?;
        let (_, _, class_name) = self.read_ptr_class(instance.ptr_class)?;

        log::trace!("Call {}::{}({})", class_name, name, signature);

        let mut params = vec![instance_proxy.ptr_instance];
        params.extend(args);

        self.call_method_inner(instance.ptr_class, name, signature, &params).await
    }

    async fn call_static_method(&mut self, class_name: &str, method_name: &str, signature: &str, args: &[u32]) -> JavaResult<u32> {
        log::trace!("Call {}::{}({})", class_name, method_name, signature);

        let ptr_class = self.get_ptr_class(class_name)?;

        self.call_method_inner(ptr_class, method_name, signature, args).await
    }

    fn backend(&mut self) -> &mut Backend {
        self.backend
    }

    fn get_field(&mut self, instance: &JavaObjectProxy<Object>, field_name: &str) -> JavaResult<u32> {
        let instance: JavaClassInstance = read_generic(self.core, instance.ptr_instance)?;
        let ptr_field = self.get_ptr_field(instance.ptr_class, field_name)?;
        let field: JavaField = read_generic(self.core, ptr_field)?;

        assert!(field.access_flag & 0x0008 == 0, "Field is static");

        let offset = field.offset_or_value;

        read_generic(self.core, instance.ptr_fields + offset + 4)
    }

    fn put_field(&mut self, instance: &JavaObjectProxy<Object>, field_name: &str, value: u32) -> JavaResult<()> {
        let instance: JavaClassInstance = read_generic(self.core, instance.ptr_instance)?;
        let ptr_field = self.get_ptr_field(instance.ptr_class, field_name)?;
        let field: JavaField = read_generic(self.core, ptr_field)?;

        assert!(field.access_flag & 0x0008 == 0, "Field is static");

        let offset = field.offset_or_value;

        write_generic(self.core, instance.ptr_fields + offset + 4, value)
    }

    fn get_static_field(&mut self, class_name: &str, field_name: &str) -> JavaResult<u32> {
        let ptr_class = self.get_ptr_class(class_name)?;
        let ptr_field = self.get_ptr_field(ptr_class, field_name)?;
        let field: JavaField = read_generic(self.core, ptr_field)?;

        assert!(field.access_flag & 0x0008 != 0, "Field is not static");

        Ok(field.offset_or_value)
    }

    fn put_static_field(&mut self, class_name: &str, field_name: &str, value: u32) -> JavaResult<()> {
        let ptr_class = self.get_ptr_class(class_name)?;
        let ptr_field = self.get_ptr_field(ptr_class, field_name)?;
        let mut field: JavaField = read_generic(self.core, ptr_field)?;

        assert!(field.access_flag & 0x0008 != 0, "Field is not static");

        field.offset_or_value = value;

        write_generic(self.core, ptr_field, field)?;

        Ok(())
    }

    fn store_array(&mut self, array: &JavaObjectProxy<Array>, index: u32, values: &[u32]) -> JavaResult<()> {
        let instance: JavaClassInstance = read_generic(self.core, array.ptr_instance)?;
        let items_offset = instance.ptr_fields + 8;

        let element_size = self.get_array_element_size(instance.ptr_class)?;
        let bytes = values
            .iter()
            .flat_map(|x| x.to_le_bytes().into_iter().take(element_size as usize))
            .collect::<Vec<_>>();

        self.core.write_bytes(items_offset + element_size * index, &bytes)
    }

    fn load_array(&mut self, array: &JavaObjectProxy<Array>, offset: u32, count: u32) -> JavaResult<Vec<u32>> {
        let instance: JavaClassInstance = read_generic(self.core, array.ptr_instance)?;
        let items_offset = instance.ptr_fields + 8;

        let element_size = self.get_array_element_size(instance.ptr_class)?;

        let values_raw = self.core.read_bytes(items_offset + element_size * offset, count * element_size)?;
        let values = values_raw
            .chunks(element_size as usize)
            .map(|x| {
                let bytes = x.iter().chain(iter::repeat(&0)).take(4).copied().collect::<Vec<_>>();
                u32::from_le_bytes(bytes.try_into().unwrap())
            })
            .collect::<Vec<_>>();
        Ok(values)
    }

    fn array_length(&mut self, array: &JavaObjectProxy<Array>) -> JavaResult<u32> {
        let instance: JavaClassInstance = read_generic(self.core, array.ptr_instance)?;

        read_generic(self.core, instance.ptr_fields + 4)
    }

    fn spawn(&mut self, callback: JavaMethodBody) -> JavaResult<()> {
        struct SpawnProxy {
            backend: Backend,
            callback: JavaMethodBody,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, JavaError> for SpawnProxy {
            #[allow(clippy::await_holding_refcell_ref)] // We manually drop RefMut https://github.com/rust-lang/rust-clippy/issues/6353
            async fn call(self) -> Result<u32, JavaError> {
                let executor: Executor = Executor::current();
                let mut module = executor.module_mut();
                let mut core = ArmCore::from_core_mut(module.core_mut()).unwrap().clone();

                core::mem::drop(module);

                let mut backend = self.backend.clone();
                let mut context = KtfJavaContext::new(&mut core, &mut backend);

                self.callback.call(&mut context, &[]).await
            }
        }

        let backend = self.backend.clone();

        task::spawn(SpawnProxy { backend, callback });

        Ok(())
    }

    fn sleep(&mut self, duration: u64) -> SleepFuture {
        let until = self.backend.time().now() + duration;

        task::sleep(until)
    }
}
