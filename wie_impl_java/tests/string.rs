mod context;

use jvm::ClassInstanceRef;
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

    let result: ClassInstanceRef = context
        .jvm()
        .invoke_virtual(
            &string1,
            "java/lang/String",
            "concat",
            "(Ljava/lang/String;)Ljava/lang/String;",
            [string2.clone().into()],
        )
        .await?;

    let string = String::to_rust_string(&mut context, &result).unwrap();

    assert_eq!(string, "test1test2");

    Ok(())
}
