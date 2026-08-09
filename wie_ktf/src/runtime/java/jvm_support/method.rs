use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{
    fmt::{self, Debug, Formatter},
    mem::size_of,
    ops::{Deref, DerefMut},
};
use futures::TryFutureExt;
use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{ClassInstance, JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use wipi_types::ktf::java::{
    JavaExceptionHandler as RawJavaExceptionHandler, JavaMethodDefinition as RawJavaMethod,
    JavaMethodExceptionTableEntry as RawJavaMethodExceptionTableEntry,
};

use alloc::sync::Arc;
use wie_core_arm::{
    Allocator, ArmCore, EmulatedFunction, EmulatedFunctionParam, RUN_FUNCTION_LR, RegisteredFunction, RegisteredFunctionHolder, ResultWriter,
};
use wie_jvm_support::native::{NativeJavaValueCodec, decode_method_arguments, encode_method_arguments, method_argument_word_count};
use wie_util::{ByteWrite, Result, WieError, read_generic, write_generic};

use crate::{
    emulator::IMAGE_BASE,
    runtime::{SVC_CATEGORY_JAVA, java::JavaSvcFunctions, java::jvm_support::JavaClassDefinition},
};

use super::{KtfJvmSupport, class_instance::JavaClassInstance, name::JavaFullName, value::JavaValueCodec};

pub struct JavaMethod {
    pub ptr_raw: u32,
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
        context: Context,
        java_functions: JavaSvcFunctions,
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

        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaMethod>() as u32)?;

        let access_flags = proto.access_flags;
        let fn_method = Self::register_java_method(core, jvm, ptr_raw, proto, context, java_functions)?;

        let (fn_body, fn_body_native) = if access_flags.contains(MethodAccessFlags::NATIVE) {
            (0, fn_method)
        } else {
            (fn_method, 0)
        };

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
                index_in_vtable: 0, // to be filled later
                access_flags: access_flags.bits(),
                unk6: 0,
            },
        )?;

        tracing::trace!("Wrote method {} at {ptr_raw:#x}", full_name.name);

        Ok(Self::from_raw(ptr_raw, core))
    }

    pub fn write_vtable_index(&mut self, new_index: u16) -> Result<()> {
        let mut raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        raw.index_in_vtable = new_index;

        write_generic(&mut self.core, self.ptr_raw, raw)?;

        Ok(())
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

        let codec = JavaValueCodec::new(&self.core);
        let raw_args = encode_method_arguments(&codec, &args);

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

        // Re-enters `run_function` if a Java catch handler matches the current ARM frame —
        // mirrors the trampoline path in `interface.rs::map_jump_result`, but for the
        // outermost frame whose caller is the Rust JVM rather than another ARM trampoline.
        async fn run_with_unwind(core: &mut ArmCore, mut pc: u32, mut args: Vec<u32>) -> Result<JavaMethodRunResult> {
            let caller_context = core.save_context();

            loop {
                match core.run_function::<JavaMethodRunResult>(pc, &args).await {
                    Ok(r) => {
                        core.restore_context(&caller_context);
                        return Ok(r);
                    }
                    Err(WieError::JavaExceptionUnwind {
                        context_base,
                        target,
                        next_pc,
                    }) => {
                        tracing::debug!("Resuming via exception restore: pc={next_pc:#x}, context_base={context_base:#x}, target={target:#x}");
                        pc = next_pc;
                        args = vec![context_base, target];
                    }
                    Err(e) => {
                        let error = match e {
                            error @ WieError::JavaException(_) => error,
                            error => {
                                let context = core.dump_reg_stack(IMAGE_BASE);
                                match error {
                                    WieError::Unimplemented(message) => WieError::Unimplemented(format!("{message}{context}")),
                                    WieError::FatalError(message) => WieError::FatalError(format!("{message}{context}")),
                                    error => WieError::FatalError(format!("{error}{context}")),
                                }
                            }
                        };
                        core.restore_context(&caller_context);
                        return Err(error);
                    }
                }
            }
        }

        let result: JavaMethodRunResult = if access_flags.contains(MethodAccessFlags::NATIVE) {
            let arg_container = Allocator::alloc(&mut core, (raw_args.len() as u32) * 4)?;
            for (i, arg) in raw_args.iter().enumerate() {
                write_generic(&mut core, arg_container + (i * 4) as u32, *arg)?;
            }

            tracing::trace!("Calling native method: {:#x}", raw.fn_body_native_or_exception_table);
            let result = run_with_unwind(&mut core, raw.fn_body_native_or_exception_table, vec![0, arg_container]).await;

            Allocator::free(&mut core, arg_container, (raw_args.len() as u32) * 4)?;

            result?
        } else {
            let mut params = vec![0];
            params.extend(raw_args);

            tracing::trace!("Calling method: {:#x}", raw.fn_body);
            run_with_unwind(&mut core, raw.fn_body, params).await?
        };

        if matches!(return_type, JavaType::Double | JavaType::Long) {
            Ok(codec.decode_wide(result.result, result.result_high, &return_type))
        } else {
            Ok(codec.decode_word(result.result, &return_type))
        }
    }

    fn exception_table(&self) -> Result<Vec<RawJavaMethodExceptionTableEntry>> {
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

    pub(super) fn exception_class_matches(core: &ArmCore, jvm: &Jvm, exception: &dyn ClassInstance, ptr_class: u32) -> Result<bool> {
        if ptr_class == 0 {
            return Ok(true);
        }

        if let Some(instance) = exception.as_any().downcast_ref::<JavaClassInstance>() {
            for class in instance.class()?.read_class_hierarchy()? {
                if class.ptr_raw == ptr_class || class.ptr_vtable()? == ptr_class {
                    return Ok(true);
                }
            }

            return Ok(false);
        }

        let class = JavaClassDefinition::from_raw(ptr_class, core);
        Ok(jvm.is_instance(exception, &class.name()?))
    }

    pub async fn handle_exception(core: &mut ArmCore, jvm: &Jvm, exception: Box<dyn ClassInstance>) -> Result<JavaMethodResult> {
        tracing::warn!("Java exception thrown: {exception:?}");

        let current_java_exception_handler = KtfJvmSupport::current_java_exception_handler(core)?;

        if current_java_exception_handler == 0 {
            return Err(WieError::JavaException(KtfJvmSupport::class_instance_raw(&exception)));
        }

        let exception_handler: RawJavaExceptionHandler = read_generic(core, current_java_exception_handler)?;

        let method = JavaMethod::from_raw(exception_handler.ptr_method, core);
        let exception_table = method.exception_table()?;

        for entry in exception_table {
            if entry.from_pc <= exception_handler.current_pc
                && exception_handler.current_pc < entry.to_pc
                && Self::exception_class_matches(core, jvm, &*exception, entry.ptr_class)?
            {
                let restore_context: u32 = read_generic(core, exception_handler.ptr_functions + 4)?;
                let contexts_base = current_java_exception_handler + 24;

                tracing::debug!(
                    "Java exception handler found: {:#x}, method: {:#x}",
                    entry.target,
                    exception_handler.ptr_method
                );

                return Err(WieError::JavaExceptionUnwind {
                    context_base: contexts_base,
                    target: entry.target,
                    next_pc: restore_context,
                });
            }
        }

        Err(WieError::JavaException(KtfJvmSupport::class_instance_raw(&exception)))
    }

    fn register_java_method<C, Context>(
        core: &mut ArmCore,
        jvm: &Jvm,
        ptr_method: u32,
        proto: JavaMethodProto<C>,
        context: Context,
        java_functions: JavaSvcFunctions,
    ) -> Result<u32>
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

        let proxy = RegisteredFunctionHolder::new(proxy, &());
        java_functions
            .lock()
            .insert(ptr_method, Arc::new(Box::new(proxy) as Box<dyn RegisteredFunction>));

        core.make_svc_stub(SVC_CATEGORY_JAVA, ptr_method)
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

    async fn run(&self, jvm: &Jvm, args: Box<[JavaValue]>) -> JvmResult<JavaValue> {
        let jvm_clone = jvm.clone();
        self.run(args)
            .or_else(async move |x| {
                Err(match x {
                    WieError::JavaException(x) => JavaError::JavaException(Box::new(JavaClassInstance::from_raw(x, &self.core))),
                    WieError::JavaExceptionUnwind { .. } => {
                        jvm_clone
                            .exception("net/wie/WieError", "Java exception unwind crossed into JVM caller")
                            .await
                    }
                    _ => jvm_clone.exception("net/wie/WieError", &x.to_string()).await,
                })
            })
            .await
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
        let param_count = method_argument_word_count(&self.parameter_types);

        let raw_args = if self.proto.access_flags.contains(MethodAccessFlags::NATIVE) {
            let param_base = u32::get(core, 1);
            (0..param_count)
                .map(|x| read_generic(core, param_base + (x as u32) * 4))
                .collect::<wie_util::Result<Vec<u32>>>()?
        } else {
            (0..param_count).map(|x| u32::get(core, x + 1)).collect::<Vec<_>>()
        };

        let codec = JavaValueCodec::new(core);
        let args = decode_method_arguments(&codec, &self.parameter_types, &raw_args);

        let mut context = self.context.clone();
        let (_, lr) = core.read_pc_lr()?;

        let result = self.proto.body.call(&self.jvm, &mut context, args.into_boxed_slice()).await;
        if let Err(JavaError::JavaException(x)) = result {
            // if we executed this from rust code, we should propagate this down
            if lr == RUN_FUNCTION_LR {
                let java_exception = KtfJvmSupport::class_instance_raw(&x);
                return Err(WieError::JavaException(java_exception));
            }
            return JavaMethod::handle_exception(core, &self.jvm, x).await;
        }

        let result = if matches!(self.return_type, JavaType::Double | JavaType::Long) {
            let (result, result_high) = codec.encode_wide(&result.unwrap());
            vec![result, result_high]
        } else {
            vec![codec.encode_word(&result.unwrap())]
        };

        Ok(JavaMethodResult { result, next_pc: None })
    }
}

pub struct JavaMethodResult {
    result: Vec<u32>,
    next_pc: Option<u32>,
}

impl JavaMethodResult {
    pub fn new(result: Vec<u32>, next_pc: Option<u32>) -> Self {
        Self { result, next_pc }
    }
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
