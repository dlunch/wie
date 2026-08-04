use alloc::{boxed::Box, string::String, string::ToString, sync::Arc, vec, vec::Vec};
use core::{fmt, fmt::Debug, fmt::Formatter, ops::Deref, ops::DerefMut};

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use wipi_types::lgt::java::LgtJavaClassMethod as RawJavaMethod;

use wie_core_arm::{
    Allocator, ArmCore, EmulatedFunction, EmulatedFunctionParam, RegisteredFunction, RegisteredFunctionHolder, ResultWriter, RunFunctionResult,
};
use wie_jvm_support::native::{NativeJavaValueCodec, decode_method_arguments, encode_method_arguments, method_argument_word_count};
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes};

use crate::runtime::{SVC_CATEGORY_JAVA, java::JavaSvcFunctions};

use super::value::JavaValueCodec;

#[derive(Clone)]
pub struct JavaMethod {
    ptr_raw: u32,
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
        let ptr_name = Allocator::alloc(core, (proto.name.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_name, proto.name.as_bytes())?;

        let ptr_descriptor = Allocator::alloc(core, (proto.descriptor.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_descriptor, proto.descriptor.as_bytes())?;

        let method_type = JavaType::parse(&proto.descriptor);
        let (parameter_types, _) = method_type.as_method();
        let argument_word_count =
            method_argument_word_count(parameter_types) as u16 + u16::from(!proto.access_flags.contains(MethodAccessFlags::STATIC));
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
                argument_word_count,
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

    async fn run_async(&self, args: Box<[JavaValue]>) -> Result<JavaValue> {
        let raw = self.raw()?;
        let return_type = JavaType::parse(&self.descriptor()).as_method().1.clone();
        let codec = JavaValueCodec::new(&self.core);
        let raw_args = encode_method_arguments(&codec, &args);

        let result: JavaMethodRunResult = self.core.clone().run_function(raw.ptr_method, &raw_args).await?;

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
        String::from_utf8(read_null_terminated_string_bytes(&self.core, self.raw().unwrap().ptr_name).unwrap()).unwrap()
    }

    fn descriptor(&self) -> String {
        String::from_utf8(read_null_terminated_string_bytes(&self.core, self.raw().unwrap().ptr_descriptor).unwrap()).unwrap()
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
        let parameter_word_count = method_argument_word_count(&self.parameter_types);
        let raw_args = (0..parameter_word_count)
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

struct JavaMethodResult {
    result: Vec<u32>,
}

impl ResultWriter<JavaMethodResult> for JavaMethodResult {
    fn write(self, core: &mut ArmCore, next_pc: u32) -> Result<()> {
        core.write_return_value(&self.result)?;
        core.set_next_pc(next_pc)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};
    use core::{
        mem::size_of,
        sync::atomic::{AtomicBool, Ordering},
    };

    use java_class_proto::{JavaClassProto, JavaMethodProto};
    use java_constants::MethodAccessFlags;
    use jvm::{JavaValue, Jvm, Method, Result as JvmResult, runtime::JavaLangString};
    use spin::Mutex;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_jvm_support::{JvmImplementation, JvmSupport};
    use wie_util::{Result, write_generic, write_null_terminated_string_bytes};

    use super::{JavaMethod, JavaMethodRunResult, RawJavaMethod};
    use crate::runtime::java::jvm_support::{JavaClassInstance, LgtJvmImplementation};

    async fn rust_wide_bridge(_jvm: &Jvm, observed: &mut Arc<Mutex<Option<(i64, u64)>>>, integer: i64, floating: f64) -> JvmResult<i64> {
        *observed.lock() = Some((integer, floating.to_bits()));
        Ok(integer ^ floating.to_bits() as i64)
    }

    #[test]
    fn aot_method_bridge_preserves_reference_and_wide_values_in_both_directions() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let mut core = ArmCore::new(false, None)?;
            Allocator::init(&mut core)?;

            let mut context = core.save_context();
            let stack = Allocator::alloc(&mut core, 0x100)?;
            context.sp = stack + 0x100;
            core.restore_context(&context);

            let protos = [wie_midp::get_protos().into(), wie_wipi_java::get_protos().into()];
            let implementation = LgtJvmImplementation::new(&mut core)?;
            let jvm = JvmSupport::new_jvm(&system_clone, None, Box::new(protos), &[], implementation.clone()).await?;

            let ptr_aot = Allocator::alloc(&mut core, 6)?;
            write_generic(&mut core, ptr_aot, 0x1808u16)?; // adds r0, r1, r0
            write_generic(&mut core, ptr_aot + 2, 0x0011u16)?; // movs r1, r2
            write_generic(&mut core, ptr_aot + 4, 0x4770u16)?; // bx lr

            let ptr_name = Allocator::alloc(&mut core, 7)?;
            write_null_terminated_string_bytes(&mut core, ptr_name, b"bridge")?;
            let descriptor = b"(Ljava/lang/String;J)J";
            let ptr_descriptor = Allocator::alloc(&mut core, descriptor.len() as u32 + 1)?;
            write_null_terminated_string_bytes(&mut core, ptr_descriptor, descriptor)?;
            let ptr_method = Allocator::alloc(&mut core, size_of::<RawJavaMethod>() as u32)?;
            write_generic(
                &mut core,
                ptr_method,
                RawJavaMethod {
                    ptr_class: 0,
                    ptr_name,
                    ptr_descriptor,
                    access_flags: MethodAccessFlags::STATIC.bits(),
                    argument_word_count: 3,
                    unk3: 0,
                    ptr_method: ptr_aot + 1,
                    unk4: 0,
                },
            )?;

            let reference = JavaLangString::from_rust_string(&jvm, "guest-reference").await.unwrap();
            let ptr_reference = reference.as_any().downcast_ref::<JavaClassInstance>().unwrap().ptr_raw;
            let wide = 0x1234_5678_0102_0304i64;
            let result = JavaMethod::from_raw(ptr_method, &core)
                .run(&jvm, vec![JavaValue::Object(Some(reference)), JavaValue::Long(wide)].into_boxed_slice())
                .await
                .unwrap();
            assert_eq!(
                i64::from(result),
                ((wide as u64 & 0xffff_ffff_0000_0000) | ((wide as u32).wrapping_add(ptr_reference) as u64)) as i64
            );

            let observed = Arc::new(Mutex::new(None));
            let class = implementation
                .define_class_rust(
                    &jvm,
                    JavaClassProto {
                        name: "net/wie/test/RustBridge",
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec![],
                        methods: vec![JavaMethodProto::new("wide", "(JD)J", rust_wide_bridge, MethodAccessFlags::STATIC)],
                        fields: vec![],
                        access_flags: Default::default(),
                    },
                    Box::new(observed.clone()),
                )
                .await
                .unwrap();
            let method = class.method("wide", "(JD)J", true).unwrap();
            let target = method.as_any().downcast_ref::<JavaMethod>().unwrap().target()?;

            let ptr_wrapper = Allocator::alloc(&mut core, 12)?;
            write_generic(&mut core, ptr_wrapper, 0xb500u16)?; // push {lr}
            write_generic(&mut core, ptr_wrapper + 2, 0x4c01u16)?; // ldr r4, [pc, #4]
            write_generic(&mut core, ptr_wrapper + 4, 0x47a0u16)?; // blx r4
            write_generic(&mut core, ptr_wrapper + 6, 0xbd00u16)?; // pop {pc}
            write_generic(&mut core, ptr_wrapper + 8, target)?;

            let integer = 0x1357_9bdf_2468_ace0i64;
            let floating = -13.25f64;
            let integer_bits = integer as u64;
            let floating_bits = floating.to_bits();
            let result: JavaMethodRunResult = core
                .run_function(
                    ptr_wrapper + 1,
                    &[
                        integer_bits as u32,
                        (integer_bits >> 32) as u32,
                        floating_bits as u32,
                        (floating_bits >> 32) as u32,
                    ],
                )
                .await?;
            assert_eq!(*observed.lock(), Some((integer, floating_bits)));
            assert_eq!(((result.high as u64) << 32) | result.low as u64, (integer ^ floating_bits as i64) as u64);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }
}
