use alloc::{boxed::Box, string::String, vec, vec::Vec};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use jvm::{JavaValue, Jvm, JvmResult, Method};

use wie_backend::Backend;
use wie_base::util::{read_generic, write_generic, ByteWrite};
use wie_core_arm::{Allocator, ArmCore, ArmEngineError, EmulatedFunction, EmulatedFunctionParam};
use wie_impl_java::{JavaMethodBody, JavaMethodFlag, JavaMethodProto, JavaResult, JavaWord};

use super::{value::JavaValueExt, vtable_builder::JavaVtableBuilder, JavaFullName, KtfJavaContext};

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
struct RawJavaMethod {
    fn_body: u32,
    ptr_class: u32,
    fn_body_native_or_exception_table: u32,
    ptr_name: u32,
    exception_table_count: u16,
    unk3: u16,
    index_in_vtable: u16,
    flag: JavaMethodFlagBit,
    unk6: u32,
}

pub struct JavaMethod {
    pub(crate) ptr_raw: u32,
    core: ArmCore,
}

impl JavaMethod {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn new(core: &mut ArmCore, ptr_class: u32, proto: JavaMethodProto, vtable_builder: &mut JavaVtableBuilder) -> JavaResult<Self> {
        let full_name = JavaFullName {
            tag: 0,
            name: proto.name,
            descriptor: proto.descriptor,
        };
        let full_name_bytes = full_name.as_bytes();

        let ptr_name = Allocator::alloc(core, full_name_bytes.len() as u32)?;
        core.write_bytes(ptr_name, &full_name_bytes)?;

        let fn_method = Self::register_java_method(core, proto.body, proto.flag == JavaMethodFlag::NATIVE)?;
        let (fn_body, fn_body_native) = if proto.flag == JavaMethodFlag::NATIVE {
            (0, fn_method)
        } else {
            (fn_method, 0)
        };

        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaMethod>() as u32)?;
        let index_in_vtable = vtable_builder.add(ptr_raw, &full_name) as u16;

        let flag = JavaMethodFlagBit::from_flag(proto.flag);

        write_generic(
            core,
            ptr_raw,
            RawJavaMethod {
                fn_body,
                ptr_class,
                fn_body_native_or_exception_table: fn_body_native,
                ptr_name,
                exception_table_count: 0,
                unk3: 0,
                index_in_vtable,
                flag,
                unk6: 0,
            },
        )?;

        Ok(Self::from_raw(ptr_raw, core))
    }

    pub fn name(&self) -> JavaResult<JavaFullName> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        JavaFullName::from_ptr(&self.core, raw.ptr_name)
    }

    pub async fn run(&self, args: &[JavaValue]) -> JavaResult<u32> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        let mut core = self.core.clone();

        if raw.flag.contains(JavaMethodFlagBit::NATIVE) {
            let arg_container = Allocator::alloc(&mut core, (args.len() as u32) * 4)?;
            for (i, arg) in args.iter().enumerate() {
                write_generic(&mut core, arg_container + (i * 4) as u32, arg.as_raw())?;
            }

            let result = core.run_function(raw.fn_body_native_or_exception_table, &[0, arg_container]).await;

            Allocator::free(&mut core, arg_container)?;

            result
        } else {
            let mut params = vec![0];
            params.extend(args.iter().map(|x| x.as_raw()));

            core.run_function(raw.fn_body, &params).await
        }
    }

    fn register_java_method(core: &mut ArmCore, body: JavaMethodBody, native: bool) -> JavaResult<u32> {
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
        impl EmulatedFunction<(u32, u32, u32), ArmEngineError, u32> for JavaMethodProxy {
            async fn call(&self, core: &mut ArmCore, backend: &mut Backend) -> Result<u32, ArmEngineError> {
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

                let args = args.into_iter().map(|x| x as JavaWord).collect::<Vec<_>>();

                let result = self.body.call(&mut context, &args).await?; // TODO do we need arg proxy?

                Ok(result as _)
            }
        }

        let proxy = JavaMethodProxy::new(body, native);

        core.register_function(proxy)
    }
}

#[async_trait::async_trait(?Send)]
impl Method for JavaMethod {
    fn name(&self) -> String {
        let name = self.name().unwrap();

        name.name
    }

    fn descriptor(&self) -> String {
        let name = self.name().unwrap();

        name.descriptor
    }

    async fn run(&self, _jvm: &mut Jvm, args: &[JavaValue]) -> JvmResult<JavaValue> {
        let result = self.run(args).await?;
        let return_type = &self.descriptor()[self.descriptor().find(')').unwrap() + 1..];

        Ok(JavaValue::from_raw(result, return_type, &self.core))
    }
}
