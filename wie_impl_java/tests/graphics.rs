mod context;

#[futures_test::test]
async fn test_graphics() -> anyhow::Result<()> {
    let mut core = context::test_core();
    let mut _jvm = core.jvm();

    Ok(()) // TODO
}
