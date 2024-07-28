use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    mem::size_of,
    ops::{Deref, DerefMut},
};

use bytemuck::{Pod, Zeroable};

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};

use wie_core_arm::{Allocator, ArmCore, ArmCoreError, EmulatedFunction, EmulatedFunctionParam};
use wie_util::{read_generic, write_generic, ByteWrite};

use super::{name::JavaFullName, value::JavaValueExt, vtable_builder::JavaVtableBuilder, JvmSupportResult};

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
    access_flags: u16,
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

    pub fn new<C, Context>(
        core: &mut ArmCore,
        jvm: &Jvm,
        ptr_class: u32,
        proto: JavaMethodProto<C>,
        vtable_builder: &mut JavaVtableBuilder,
        context: Context,
    ) -> JvmSupportResult<Self>
    where
        C: ?Sized + 'static + Send,
        Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
    {
        let full_name = JavaFullName {
            tag: 0,
            name: proto.name.clone(),
            descriptor: proto.descriptor.clone(),
        };
        let full_name_bytes = full_name.as_bytes();

        let ptr_name = Allocator::alloc(core, full_name_bytes.len() as u32)?;
        core.write_bytes(ptr_name, &full_name_bytes)?;

        let access_flags = proto.access_flags;
        let fn_method = Self::register_java_method(core, jvm, proto, context)?;

        let (fn_body, fn_body_native) = if access_flags.contains(MethodAccessFlags::NATIVE) {
            (0, fn_method)
        } else {
            (fn_method, 0)
        };

        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaMethod>() as u32)?;
        let index_in_vtable = vtable_builder.add(ptr_raw, &full_name.name, &full_name.descriptor) as u16;

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
                access_flags: access_flags.bits(),
                unk6: 0,
            },
        )?;

        Ok(Self::from_raw(ptr_raw, core))
    }

    pub fn name(&self) -> JvmSupportResult<JavaFullName> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        JavaFullName::from_ptr(&self.core, raw.ptr_name)
    }

    pub async fn run(&self, args: Box<[JavaValue]>) -> JvmSupportResult<u32> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        let mut core = self.core.clone();

        let access_flags = MethodAccessFlags::from_bits_truncate(raw.access_flags);

        if access_flags.contains(MethodAccessFlags::NATIVE) {
            let arg_container = Allocator::alloc(&mut core, (args.len() as u32) * 4)?;
            for (i, arg) in args.iter().enumerate() {
                write_generic(&mut core, arg_container + (i * 4) as u32, arg.as_raw())?;
            }

            tracing::trace!("Calling native method: {:#x}", raw.fn_body_native_or_exception_table);
            let result = core.run_function(raw.fn_body_native_or_exception_table, &[0, arg_container]).await;

            Allocator::free(&mut core, arg_container)?;

            Ok(result?)
        } else {
            let mut params = vec![0];
            params.extend(args.iter().map(|x| x.as_raw())); // TODO double/long handling

            tracing::trace!("Calling method: {:#x}", raw.fn_body);
            Ok(core.run_function(raw.fn_body, &params).await?)
        }
    }

    fn register_java_method<C, Context>(core: &mut ArmCore, jvm: &Jvm, proto: JavaMethodProto<C>, context: Context) -> JvmSupportResult<u32>
    where
        C: ?Sized + 'static + Send,
        Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
    {
        struct JavaMethodProxy<C, Context>
        where
            C: ?Sized + Send,
            Context: Deref<Target = C> + DerefMut + Clone,
        {
            jvm: Jvm,
            proto: JavaMethodProto<C>,
            context: Context,
            parameter_types: Vec<JavaType>,
        }

        #[async_trait::async_trait]
        impl<C, Context> EmulatedFunction<(), u32, ArmCoreError, ()> for JavaMethodProxy<C, Context>
        where
            C: ?Sized + Send,
            Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
        {
            async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<u32, ArmCoreError> {
                let param_count = self.parameter_types.len() as u32;

                let args = if self.proto.access_flags.contains(MethodAccessFlags::NATIVE) {
                    let param_base = u32::get(core, 1);
                    (0..param_count)
                        .map(|x| read_generic(core, param_base + x * 4))
                        .collect::<wie_util::Result<Vec<u32>>>()?
                } else {
                    (0..param_count).map(|x| u32::get(core, (x + 1) as _)).collect::<Vec<_>>()
                };

                let args = args
                    .into_iter()
                    .zip(self.parameter_types.iter())
                    .map(|(x, r#type)| JavaValue::from_raw(x, r#type, core)) // TODO double/long handling
                    .collect::<Vec<_>>();

                let mut context = self.context.clone();

                let result = self.proto.body.call(&self.jvm, &mut context, args.into_boxed_slice()).await.unwrap();

                Ok(result.as_raw())
            }
        }

        let mut parameter_types = JavaType::parse(&proto.descriptor).as_method().0.to_vec();

        if !proto.access_flags.contains(MethodAccessFlags::STATIC) {
            // TODO proper flag handling
            parameter_types.insert(0, JavaType::Class("".into())); // TODO name
        }

        let proxy = JavaMethodProxy {
            jvm: jvm.clone(),
            proto,
            context,
            parameter_types,
        };

        Ok(core.register_function(proxy, &())?)
    }
}

#[async_trait::async_trait]
impl Method for JavaMethod {
    fn name(&self) -> String {
        let name = self.name().unwrap();

        name.name
    }

    fn descriptor(&self) -> String {
        let name = self.name().unwrap();

        name.descriptor
    }

    async fn run(&self, _jvm: &Jvm, args: Box<[JavaValue]>) -> JvmResult<JavaValue> {
        let result = self.run(args).await.map_err(|x| JavaError::FatalError(format!("{:?}", x)))?;
        let r#type = JavaType::parse(&self.descriptor());
        let (_, return_type) = r#type.as_method();

        Ok(JavaValue::from_raw(result, return_type, &self.core))
    }

    fn access_flags(&self) -> MethodAccessFlags {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw).unwrap();

        MethodAccessFlags::from_bits_truncate(raw.access_flags)
    }
}

impl Debug for JavaMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaMethod").field("ptr_raw", &self.ptr_raw).finish()
    }
}
