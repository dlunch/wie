mod context;

use jvm::JavaValue;
use wie_impl_java::{r#impl::java::lang::String, JavaContext};

#[futures_test::test]
async fn test_string() -> anyhow::Result<()> {
    let mut context = context::TestContext::new();

    let string = String::from_rust_string(&mut context, "test").await?;

    let string = String::to_rust_string(&mut context, &string).unwrap();

    assert_eq!(string, "test");

    Ok(())
}

#[futures_test::test]
async fn test_string_concat() -> anyhow::Result<()> {
    let mut context = context::TestContext::new();

    let string1 = String::from_rust_string(&mut context, "test1").await?;
    let string2 = String::from_rust_string(&mut context, "test2").await?;

    let result = context
        .jvm()
        .invoke_method(
            &string1,
            "java/lang/String",
            "concat",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JavaValue::Object(string2.instance)],
        )
        .await?;

    let string = String::to_rust_string(&mut context, result.as_object_ref().unwrap()).unwrap();

    assert_eq!(string, "test1test2");

    Ok(())
}
