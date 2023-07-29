use alloc::{str, vec::Vec};

use crate::{JavaContext, JavaObjectProxy, JavaResult};

pub fn from_java_string(context: &mut dyn JavaContext, instance: &JavaObjectProxy) -> JavaResult<alloc::string::String> {
    let name_array = JavaObjectProxy::new(context.get_field(instance, "value")?);
    let length = context.get_field(instance, "length")?;
    let name = context
        .load_array(&name_array, 0, length)?
        .into_iter()
        .map(|x| x as u8)
        .collect::<Vec<_>>();

    Ok(str::from_utf8(&name)?.into())
}

pub async fn to_java_string(context: &mut dyn JavaContext, string: &str) -> JavaResult<JavaObjectProxy> {
    let instance: JavaObjectProxy = context.instantiate("Ljava/lang/String;")?;
    context.call_method(&instance, "<init>", "(I)V", &[string.len() as u32]).await?;

    let field_value = JavaObjectProxy::new(context.get_field(&instance, "value")?);

    let bytes = string.bytes().map(|x| x as u32).collect::<Vec<_>>();
    context.store_array(&field_value, 0, &bytes)?;

    Ok(instance)
}
