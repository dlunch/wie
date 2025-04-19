use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    mem::size_of,
    ops::{Deref, DerefMut},
};
use wie_jvm_support::JvmSupport;

use bytemuck::{Pod, Zeroable};

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{ClassInstance, JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};

use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, EmulatedFunctionParam, RUN_FUNCTION_LR, ResultWriter};
use wie_util::{ByteWrite, Result, WieError, read_generic, write_generic};

use crate::runtime::java::jvm_support::JavaClassDefinition;

use super::{KtfJvmSupport, class_instance::JavaClassInstance, name::JavaFullName, value::JavaValueExt, vtable_builder::JavaVtableBuilder};

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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MethodExceptionTableEntry {
    from_pc: u32,
    to_pc: u32,
    target: u32,
    ptr_class: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct KtfJvmExceptionHandler {
    ptr_method: u32,
    ptr_this: u32,
    ptr_old_handler: u32,
    current_pc: u32,
    unk3: u32,
    ptr_functions: u32, // function table to restore context
    context: [u32; 11], // r4-lr
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
    ) -> Result<Self>
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

        tracing::trace!("Wrote method {} at {:#x}", full_name.name, ptr_raw);

        Ok(Self::from_raw(ptr_raw, core))
    }

    pub fn ptr_class(&self) -> u32 {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw).unwrap();

        raw.ptr_class
    }

    pub fn name(&self) -> Result<JavaFullName> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        JavaFullName::from_ptr(&self.core, raw.ptr_name)
    }

    pub async fn run(&self, args: Box<[JavaValue]>) -> Result<JavaValue> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;
        let return_type = JavaType::parse(&self.descriptor()).as_method().1.clone();

        let mut core = self.core.clone();

        let mut raw_args = Vec::with_capacity(args.len());
        for arg in args.iter() {
            if matches!(arg, JavaValue::Double(_) | JavaValue::Long(_)) {
                let (arg, arg_high) = arg.as_raw64();
                raw_args.push(arg);
                raw_args.push(arg_high);
            } else {
                raw_args.push(arg.as_raw());
            }
        }

        struct JavaMethodRunResult {
            result: u32,
            result_high: u32,
        }

        impl wie_core_arm::RunFunctionResult<JavaMethodRunResult> for JavaMethodRunResult {
            fn get(core: &ArmCore) -> Self {
                let result = core.read_param(0).unwrap();
                let result_high = core.read_param(1).unwrap();

                Self { result, result_high }
            }
        }

        let access_flags = MethodAccessFlags::from_bits_truncate(raw.access_flags);

        let result: JavaMethodRunResult = if access_flags.contains(MethodAccessFlags::NATIVE) {
            let arg_container = Allocator::alloc(&mut core, (raw_args.len() as u32) * 4)?;
            for (i, arg) in raw_args.iter().enumerate() {
                write_generic(&mut core, arg_container + (i * 4) as u32, *arg)?;
            }

            tracing::trace!("Calling native method: {:#x}", raw.fn_body_native_or_exception_table);
            let result = core.run_function(raw.fn_body_native_or_exception_table, &[0, arg_container]).await;

            Allocator::free(&mut core, arg_container, (raw_args.len() as u32) * 4)?;

            result?
        } else {
            let mut params = vec![0];
            params.extend(raw_args);

            tracing::trace!("Calling method: {:#x}", raw.fn_body);
            core.run_function(raw.fn_body, &params).await?
        };

        if matches!(return_type, JavaType::Double | JavaType::Long) {
            Ok(JavaValue::from_raw64(result.result, result.result_high, &return_type))
        } else {
            Ok(JavaValue::from_raw(result.result, &return_type, &core))
        }
    }

    fn exception_table(&self) -> Result<Vec<MethodExceptionTableEntry>> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        let mut result = Vec::with_capacity(raw.exception_table_count as _);

        if raw.exception_table_count == 0 {
            return Ok(result);
        }

        let mut cursor = raw.fn_body_native_or_exception_table;
        for _ in 0..raw.exception_table_count {
            let address = read_generic(&self.core, cursor)?;
            cursor += 4;

            result.push(read_generic(&self.core, address)?);
        }

        Ok(result)
    }

    pub async fn handle_exception(core: &mut ArmCore, jvm: &Jvm, exception: Box<dyn ClassInstance>) -> Result<JavaMethodResult> {
        tracing::warn!("Java exception thrown: {:?}", exception);

        let current_java_exception_handler = KtfJvmSupport::current_java_exception_handler(core)?;

        if current_java_exception_handler == 0 {
            return Err(JvmSupport::to_wie_err(jvm, JavaError::JavaException(exception)).await);
        }

        let exception_handler: KtfJvmExceptionHandler = read_generic(core, current_java_exception_handler)?;

        let method = JavaMethod::from_raw(exception_handler.ptr_method, core);
        let exception_table = method.exception_table()?;

        for entry in exception_table {
            if entry.from_pc <= exception_handler.current_pc && exception_handler.current_pc < entry.to_pc {
                let class = JavaClassDefinition::from_raw(entry.ptr_class, core);
                if entry.ptr_class == 0 || jvm.is_instance(&*exception, &class.name()?) {
                    let restore_context: u32 = read_generic(core, exception_handler.ptr_functions + 4)?;
                    let contexts_base = current_java_exception_handler + 24;

                    tracing::debug!(
                        "Java exception handler found: {:#x}, method: {:#x}",
                        entry.target,
                        exception_handler.ptr_method
                    );

                    return Ok(JavaMethodResult {
                        result: vec![contexts_base, entry.target],
                        next_pc: Some(restore_context),
                    });
                }
            }
        }

        Err(JvmSupport::to_wie_err(jvm, JavaError::JavaException(exception)).await)
    }

    fn register_java_method<C, Context>(core: &mut ArmCore, jvm: &Jvm, proto: JavaMethodProto<C>, context: Context) -> Result<u32>
    where
        C: ?Sized + 'static + Send,
        Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
    {
        let java_type = JavaType::parse(&proto.descriptor);
        let (parameter_types, return_type) = java_type.as_method();

        let mut parameter_types = parameter_types.to_vec();
        if !proto.access_flags.contains(MethodAccessFlags::STATIC) {
            // TODO proper flag handling
            parameter_types.insert(0, JavaType::Class("".into())); // TODO name
        }

        let proxy = JavaMethodProxy {
            jvm: jvm.clone(),
            proto,
            context,
            parameter_types,
            return_type: return_type.clone(),
        };

        core.register_function(proxy, &())
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
        self.run(args).await.map_err(|x| match x {
            WieError::FatalError(x) => JavaError::FatalError(x),
            WieError::JavaException(x) => JavaError::JavaException(Box::new(JavaClassInstance::from_raw(x, &self.core))),
            _ => JavaError::FatalError(format!("{}", x)),
        })
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

struct JavaMethodProxy<C, Context>
where
    C: ?Sized + Send,
    Context: Deref<Target = C> + DerefMut + Clone,
{
    jvm: Jvm,
    proto: JavaMethodProto<C>,
    context: Context,
    parameter_types: Vec<JavaType>,
    return_type: JavaType,
}

#[async_trait::async_trait]
impl<C, Context> EmulatedFunction<(), JavaMethodResult, ()> for JavaMethodProxy<C, Context>
where
    C: ?Sized + Send,
    Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
{
    async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<JavaMethodResult> {
        let double_long_count = self
            .parameter_types
            .iter()
            .filter(|x| matches!(x, JavaType::Double | JavaType::Long))
            .count();

        let param_count = self.parameter_types.len() + double_long_count;

        let raw_args = if self.proto.access_flags.contains(MethodAccessFlags::NATIVE) {
            let param_base = u32::get(core, 1);
            (0..param_count)
                .map(|x| read_generic(core, param_base + (x as u32) * 4))
                .collect::<wie_util::Result<Vec<u32>>>()?
        } else {
            (0..param_count).map(|x| u32::get(core, x + 1)).collect::<Vec<_>>()
        };

        let mut args = Vec::with_capacity(self.parameter_types.len());

        let mut it = raw_args.into_iter();
        for param in self.parameter_types.iter() {
            let arg = it.next().unwrap();

            let value = if matches!(param, JavaType::Double | JavaType::Long) {
                let arg_high = it.next().unwrap();

                JavaValue::from_raw64(arg, arg_high, param)
            } else {
                JavaValue::from_raw(arg, param, core)
            };
            args.push(value);
        }

        let mut context = self.context.clone();
        let (_, lr) = core.read_pc_lr()?;

        let result = self.proto.body.call(&self.jvm, &mut context, args.into_boxed_slice()).await;
        if let Err(x) = result {
            if let JavaError::JavaException(x) = x {
                // if we executed this from rust code, we should propagate this down
                if lr == RUN_FUNCTION_LR {
                    let java_exception = KtfJvmSupport::class_instance_raw(&x);
                    return Err(WieError::JavaException(java_exception));
                }
                return JavaMethod::handle_exception(core, &self.jvm, x).await;
            }
            return Err(JvmSupport::to_wie_err(&self.jvm, x).await);
        }

        let result = if matches!(self.return_type, JavaType::Double | JavaType::Long) {
            let (result, result_high) = result.unwrap().as_raw64();
            vec![result, result_high]
        } else {
            vec![result.unwrap().as_raw()]
        };

        Ok(JavaMethodResult { result, next_pc: None })
    }
}

pub struct JavaMethodResult {
    result: Vec<u32>,
    next_pc: Option<u32>,
}

impl ResultWriter<JavaMethodResult> for JavaMethodResult {
    fn write(self, core: &mut ArmCore, next_pc: u32) -> Result<()> {
        core.write_return_value(&self.result)?;

        if let Some(x) = self.next_pc {
            core.set_next_pc(x)?;
        } else {
            core.set_next_pc(next_pc)?;
        }

        Ok(())
    }
}
