use alloc::boxed::Box;

use wie_base::util::write_null_terminated_string;
use wie_core_arm::Allocator;
use wie_impl_java::{get_class_proto, JavaResult};

use super::{class::JavaClass, context_data::JavaContextData, KtfJavaContext};

pub struct ClassLoader {}

impl ClassLoader {
    #[async_recursion::async_recursion(?Send)]
    pub async fn get_or_load_class(context: &mut KtfJavaContext<'_>, name: &str) -> JavaResult<Option<JavaClass>> {
        let class = JavaContextData::find_class(context.core, name)?;

        if let Some(class) = class {
            Ok(Some(class))
        } else {
            // array class is created dynamically
            if name.as_bytes()[0] == b'[' {
                Ok(Some(JavaClass::new_array(context, name).await?))
            } else {
                let proto = get_class_proto(name);
                if let Some(x) = proto {
                    Ok(Some(JavaClass::new(context, name, x).await?))
                } else {
                    // find from client.bin
                    let fn_get_class = JavaContextData::fn_get_class(context.core)?;

                    let ptr_name = Allocator::alloc(context.core, 50)?; // TODO size fix
                    write_null_terminated_string(context.core, ptr_name, name)?;

                    let ptr_raw = context.core.run_function(fn_get_class, &[ptr_name]).await?;
                    Allocator::free(context.core, ptr_name)?;

                    if ptr_raw != 0 {
                        let class = JavaClass::from_raw(ptr_raw, context.core);
                        context.register_class(&class).await?;

                        Ok(Some(class))
                    } else {
                        Ok(None)
                    }
                }
            }
        }
    }
}
