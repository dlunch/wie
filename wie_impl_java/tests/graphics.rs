use test_utils::test_jvm_core;

#[futures_test::test]
async fn test_graphics() -> anyhow::Result<()> {
    let mut core = test_jvm_core();
    let mut _jvm = core.jvm();

    Ok(()) // TODO
}
