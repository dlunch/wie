use alloc::boxed::Box;

use wie_base::util::write_null_terminated_string;
use wie_core_arm::{Allocator, ArmCore};
use wie_impl_java::{get_class_proto, JavaResult};

use super::{array_class::JavaArrayClass, class::JavaClass, context_data::JavaContextData, KtfJavaContext};

pub struct ClassLoader {}

impl ClassLoader {
    #[async_recursion::async_recursion(?Send)]
    pub async fn get_or_load_class(core: &mut ArmCore, name: &str) -> JavaResult<Option<JavaClass>> {
        anyhow::ensure!(name.as_bytes()[0] != b'[', "Should not be an array class");

        let class = JavaContextData::find_class(core, name)?;

        if let Some(class) = class {
            Ok(Some(class))
        } else {
            let proto = get_class_proto(name);
            if let Some(x) = proto {
                Ok(Some(JavaClass::new(core, name, x).await?))
            } else {
                // find from client.bin
                let fn_get_class = JavaContextData::fn_get_class(core)?;

                let ptr_name = Allocator::alloc(core, 50)?; // TODO size fix
                write_null_terminated_string(core, ptr_name, name)?;

                let ptr_raw = core.run_function(fn_get_class, &[ptr_name]).await?;
                Allocator::free(core, ptr_name)?;

                if ptr_raw != 0 {
                    let class = JavaClass::from_raw(ptr_raw, core);
                    KtfJavaContext::register_class(core, &class).await?;

                    Ok(Some(class))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn load_array_class(core: &mut ArmCore, name: &str) -> JavaResult<Option<JavaArrayClass>> {
        anyhow::ensure!(name.as_bytes()[0] == b'[', "Not an array class");

        let class = JavaContextData::find_class(core, name)?;

        if let Some(class) = class {
            Ok(Some(JavaArrayClass::from_raw(class.ptr_raw, core)))
        } else {
            Ok(Some(JavaArrayClass::new(core, name).await?))
        }
    }
}
