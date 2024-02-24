use wie_util::{read_null_terminated_string, write_null_terminated_string};
use wie_wipi_c::{api::kernel::get_kernel_method_table, WIPICContext, WIPICError};

mod context;

#[futures_test::test]
async fn test_sprintk() -> anyhow::Result<()> {
    let mut context = context::TestContext::new();

    let kernel_methods = get_kernel_method_table(|_: &mut dyn WIPICContext| async { Ok::<_, WIPICError>(()) });

    let format = context.alloc_raw(10).unwrap();
    write_null_terminated_string(&mut context, format, "%d").unwrap();

    let dest = context.alloc_raw(10).unwrap();

    kernel_methods[1]
        .call(&mut context, Box::new([dest, format, 1234, 0, 0, 0, 0]))
        .await
        .unwrap();

    let result = read_null_terminated_string(&context, dest).unwrap();

    assert_eq!(result, "1234");

    Ok(())
}
