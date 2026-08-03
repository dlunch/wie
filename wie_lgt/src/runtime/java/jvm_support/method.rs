use alloc::{boxed::Box, string::String, string::ToString, sync::Arc, vec, vec::Vec};
use core::{fmt, fmt::Debug, fmt::Formatter, ops::Deref, ops::DerefMut};

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use wipi_types::lgt::java::LgtJavaClassMethod as RawJavaMethod;

use wie_core_arm::{ArmCore, EmulatedFunction, EmulatedFunctionParam, RegisteredFunction, RegisteredFunctionHolder, ResultWriter, RunFunctionResult};
use wie_jvm_support::native::{NativeJavaValueCodec, decode_method_arguments, encode_method_arguments, method_argument_slot_count};
use wie_util::{WieError, read_generic, write_generic, write_null_terminated_string_bytes};

use crate::runtime::{SVC_CATEGORY_JAVA, java::JavaSvcFunctions};

use super::{Result, value::JavaValueCodec};

#[derive(Clone)]
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
        ptr_raw: u32,
        ptr_class: u32,
        proto: JavaMethodProto<C>,
        context: Context,
        functions: JavaSvcFunctions,
    ) -> Result<Self>
    where
        C: ?Sized + 'static + Send,
        Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
    {
        let ptr_name = wie_core_arm::Allocator::alloc(core, (proto.name.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_name, proto.name.as_bytes())?;

        let ptr_descriptor = wie_core_arm::Allocator::alloc(core, (proto.descriptor.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_descriptor, proto.descriptor.as_bytes())?;

        let method_type = JavaType::parse(&proto.descriptor);
        let (parameter_types, _) = method_type.as_method();
        let argument_slot_count =
            method_argument_slot_count(parameter_types) as u16 + u16::from(!proto.access_flags.contains(MethodAccessFlags::STATIC));
        let access_flags = proto.access_flags;
        let target = Self::register_java_method(core, jvm, ptr_raw, proto, context, functions)?;

        write_generic(
            core,
            ptr_raw,
            RawJavaMethod {
                ptr_class,
                ptr_name,
                ptr_descriptor,
                access_flags: access_flags.bits(),
                argument_slot_count,
                unk3: 0,
                ptr_method: target,
                unk4: 0,
            },
        )?;

        Ok(Self::from_raw(ptr_raw, core))
    }

    fn raw(&self) -> Result<RawJavaMethod> {
        read_generic(&self.core, self.ptr_raw)
    }

    pub fn target(&self) -> Result<u32> {
        Ok(self.raw()?.ptr_method)
    }

    fn read_string(&self, address: u32) -> String {
        let bytes = wie_util::read_null_terminated_string_bytes(&self.core, address).unwrap();
        String::from_utf8(bytes).unwrap()
    }

    async fn run_async(&self, args: Box<[JavaValue]>) -> Result<JavaValue> {
        let raw = self.raw()?;
        let return_type = JavaType::parse(&self.descriptor()).as_method().1.clone();
        let codec = JavaValueCodec::new(&self.core);
        let raw_args = encode_method_arguments(&codec, &args);

        let mut core = self.core.clone();
        let result: JavaMethodRunResult = core.run_function(raw.ptr_method, &raw_args).await?;

        if matches!(return_type, JavaType::Double | JavaType::Long) {
            Ok(codec.decode_wide(result.low, result.high, &return_type))
        } else {
            Ok(codec.decode_word(result.low, &return_type))
        }
    }

    fn register_java_method<C, Context>(
        core: &mut ArmCore,
        jvm: &Jvm,
        ptr_method: u32,
        proto: JavaMethodProto<C>,
        context: Context,
        functions: JavaSvcFunctions,
    ) -> Result<u32>
    where
        C: ?Sized + 'static + Send,
        Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
    {
        let method_type = JavaType::parse(&proto.descriptor);
        let (parameter_types, return_type) = method_type.as_method();
        let mut parameter_types = parameter_types.to_vec();
        if !proto.access_flags.contains(MethodAccessFlags::STATIC) {
            parameter_types.insert(0, JavaType::Class(String::new()));
        }

        let proxy = JavaMethodProxy {
            jvm: jvm.clone(),
            proto,
            context,
            parameter_types,
            return_type: return_type.clone(),
        };
        let proxy = RegisteredFunctionHolder::new(proxy, &());
        functions
            .lock()
            .insert(ptr_method, Arc::new(Box::new(proxy) as Box<dyn RegisteredFunction>));

        core.make_svc_stub(SVC_CATEGORY_JAVA, ptr_method)
    }
}

#[async_trait::async_trait]
impl Method for JavaMethod {
    fn name(&self) -> String {
        self.read_string(self.raw().unwrap().ptr_name)
    }

    fn descriptor(&self) -> String {
        self.read_string(self.raw().unwrap().ptr_descriptor)
    }

    async fn run(&self, jvm: &Jvm, args: Box<[JavaValue]>) -> JvmResult<JavaValue> {
        match self.run_async(args).await {
            Ok(value) => Ok(value),
            Err(WieError::JavaException(ptr_raw)) => Err(JavaError::JavaException(JavaValueCodec::new(&self.core).object_from_raw(ptr_raw))),
            Err(error) => Err(jvm.exception("net/wie/WieError", &error.to_string()).await),
        }
    }

    fn access_flags(&self) -> MethodAccessFlags {
        MethodAccessFlags::from_bits_truncate(self.raw().unwrap().access_flags)
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
        let parameter_slot_count = method_argument_slot_count(&self.parameter_types);
        let raw_args = (0..parameter_slot_count)
            .map(|index| <u32 as EmulatedFunctionParam<u32>>::get(core, index))
            .collect::<Vec<_>>();

        let codec = JavaValueCodec::new(core);
        let args = decode_method_arguments(&codec, &self.parameter_types, &raw_args);

        let result = self.proto.body.call(&self.jvm, &mut self.context.clone(), args.into_boxed_slice()).await;
        let result = match result {
            Ok(value) => value,
            Err(JavaError::JavaException(instance)) => return Err(WieError::JavaException(codec.object_to_raw(&*instance))),
        };

        let result = if matches!(self.return_type, JavaType::Double | JavaType::Long) {
            let (low, high) = codec.encode_wide(&result);
            vec![low, high]
        } else {
            vec![codec.encode_word(&result)]
        };

        Ok(JavaMethodResult { result })
    }
}

struct JavaMethodRunResult {
    low: u32,
    high: u32,
}

impl RunFunctionResult<JavaMethodRunResult> for JavaMethodRunResult {
    fn get(core: &ArmCore) -> Self {
        Self {
            low: core.read_param(0).unwrap(),
            high: core.read_param(1).unwrap(),
        }
    }
}

pub struct JavaMethodResult {
    result: Vec<u32>,
}

impl ResultWriter<JavaMethodResult> for JavaMethodResult {
    fn write(self, core: &mut ArmCore, next_pc: u32) -> Result<()> {
        core.write_return_value(&self.result)?;
        core.set_next_pc(next_pc)
    }
}
