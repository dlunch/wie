use alloc::boxed::Box;

use wie_base::util::write_null_terminated_string;
use wie_core_arm::Allocator;
use wie_impl_java::{get_class_proto, JavaResult};

use super::{class::JavaClass, KtfJavaContext};

pub struct ClassLoader {}

impl ClassLoader {
    #[async_recursion::async_recursion(?Send)]
    pub async fn get_or_load_class(context: &mut KtfJavaContext<'_>, name: &str) -> JavaResult<JavaClass> {
        let class = Self::find_loaded_class(context, name)?;

        if let Some(class) = class {
            Ok(class)
        } else {
            // array class is created dynamically
            if name.as_bytes()[0] == b'[' {
                JavaClass::new_array(context, name).await
            } else {
                let proto = get_class_proto(name);
                if let Some(x) = proto {
                    JavaClass::new(context, name, x).await
                } else {
                    // find from client.bin
                    let fn_get_class = context.read_context_data()?.fn_get_class;

                    let ptr_name = Allocator::alloc(context.core, 50)?; // TODO size fix
                    write_null_terminated_string(context.core, ptr_name, name)?;

                    let ptr_raw = context.core.run_function(fn_get_class, &[ptr_name]).await?;
                    Allocator::free(context.core, ptr_name)?;

                    if ptr_raw != 0 {
                        let class = JavaClass::from_raw(ptr_raw);
                        context.register_class(&class).await?;

                        Ok(class)
                    } else {
                        anyhow::bail!("Cannot find class {}", name);
                    }
                }
            }
        }
    }

    pub fn find_loaded_class(context: &KtfJavaContext<'_>, name: &str) -> JavaResult<Option<JavaClass>> {
        let context_data = context.read_context_data()?;
        let classes = context.read_null_terminated_table(context_data.classes_base)?;
        for ptr_raw in classes {
            let class = JavaClass::from_raw(ptr_raw);

            if class.name(context)? == name {
                return Ok(Some(JavaClass::from_raw(ptr_raw)));
            }
        }

        Ok(None)
    }
}
