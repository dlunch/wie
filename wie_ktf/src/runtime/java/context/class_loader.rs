use alloc::boxed::Box;

use java_runtime::get_class_proto;
use jvm::JvmResult;

use wie_backend::SystemHandle;
use wie_base::util::write_null_terminated_string;
use wie_core_arm::{Allocator, ArmCore};
use wie_impl_java::get_class_proto as get_wie_class_proto;

use super::{array_class::JavaArrayClass, class::JavaClass, context_data::JavaContextData, runtime::KtfRuntime, KtfJavaContext};

pub struct ClassLoader {}

impl ClassLoader {
    #[async_recursion::async_recursion(?Send)]
    pub async fn get_or_load_class(core: &mut ArmCore, system: &mut SystemHandle, name: &str) -> JvmResult<Option<JavaClass>> {
        anyhow::ensure!(name.as_bytes()[0] != b'[', "Should not be an array class");

        let class = JavaContextData::find_class(core, name)?;

        if let Some(class) = class {
            Ok(Some(class))
        } else if let Some(x) = get_class_proto(name) {
            let runtime = KtfRuntime::new(core.clone(), system.clone());

            Ok(Some(JavaClass::new(core, system, name, x, Box::new(runtime) as Box<_>).await?))
        } else if let Some(x) = get_wie_class_proto(name) {
            let context = KtfJavaContext::new(core, system);

            Ok(Some(JavaClass::new(core, system, name, x, Box::new(context) as Box<_>).await?))
        } else {
            // find from client.bin
            let fn_get_class = JavaContextData::fn_get_class(core)?;

            let ptr_name = Allocator::alloc(core, 50)?; // TODO size fix
            write_null_terminated_string(core, ptr_name, name)?;

            let ptr_raw = core.run_function(fn_get_class, &[ptr_name]).await?;
            Allocator::free(core, ptr_name)?;

            if ptr_raw != 0 {
                let class = JavaClass::from_raw(ptr_raw, core);
                KtfJavaContext::register_class(core, system, &class).await?;

                Ok(Some(class))
            } else {
                Ok(None)
            }
        }
    }

    pub async fn load_array_class(core: &mut ArmCore, system: &mut SystemHandle, name: &str) -> JvmResult<Option<JavaArrayClass>> {
        anyhow::ensure!(name.as_bytes()[0] == b'[', "Not an array class");

        let class = JavaContextData::find_class(core, name)?;

        if let Some(class) = class {
            Ok(Some(JavaArrayClass::from_raw(class.ptr_raw, core)))
        } else {
            Ok(Some(JavaArrayClass::new(core, system, name).await?))
        }
    }
}
