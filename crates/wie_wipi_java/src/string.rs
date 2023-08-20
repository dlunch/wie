use alloc::{str, string::String, vec::Vec};

use crate::{r#impl::java::lang::String as JavaString, JavaContext, JavaObjectProxy, JavaResult};

pub fn from_java_string(context: &mut dyn JavaContext, instance: &JavaObjectProxy<JavaString>) -> JavaResult<String> {
    let java_value = JavaObjectProxy::new(context.get_field(&instance.cast(), "value")?);
    let length = context.array_length(&java_value)?;
    let name = context
        .load_array(&java_value, 0, length)?
        .into_iter()
        .map(|x| x as u8)
        .collect::<Vec<_>>();

    Ok(str::from_utf8(&name)?.into())
}

pub async fn to_java_string(context: &mut dyn JavaContext, string: &str) -> JavaResult<JavaObjectProxy<JavaString>> {
    let bytes = string.bytes().map(|x| x as u32).collect::<Vec<_>>();
    let java_value = context.instantiate_array("C", bytes.len() as u32)?;
    context.store_array(&java_value, 0, &bytes)?;

    let instance = context.instantiate("Ljava/lang/String;")?.cast();
    context
        .call_method(&instance.cast(), "<init>", "([C)V", &[java_value.ptr_instance])
        .await?;

    Ok(instance)
}
