use alloc::{str, vec::Vec};

use crate::{JavaContext, JavaObjectProxy, JavaResult};

pub fn from_java_string(context: &mut dyn JavaContext, instance: &JavaObjectProxy) -> JavaResult<alloc::string::String> {
    let java_value = JavaObjectProxy::new(context.get_field(instance, "value")?);
    let length = context.array_length(&java_value)?;
    let name = context
        .load_array(&java_value, 0, length)?
        .into_iter()
        .map(|x| x as u8)
        .collect::<Vec<_>>();

    Ok(str::from_utf8(&name)?.into())
}

pub async fn to_java_string(context: &mut dyn JavaContext, string: &str) -> JavaResult<JavaObjectProxy> {
    let bytes = string.bytes().map(|x| x as u32).collect::<Vec<_>>();
    let java_value = context.instantiate_array("C", bytes.len() as u32)?;
    context.store_array(&java_value, 0, &bytes)?;

    let instance: JavaObjectProxy = context.instantiate("Ljava/lang/String;")?;
    context.call_method(&instance, "<init>", "([C)V", &[java_value.ptr_instance]).await?;

    Ok(instance)
}
