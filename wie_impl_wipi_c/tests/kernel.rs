use wie_base::util::{read_null_terminated_string, write_null_terminated_string};
use wie_impl_wipi_c::{r#impl::kernel::get_kernel_method_table, WIPICContext};

mod context;

#[futures_test::test]
async fn test_sprintk() -> anyhow::Result<()> {
    let mut context = context::TestContext::new();

    let kernel_methods = get_kernel_method_table(|_: &mut dyn WIPICContext| async { anyhow::Ok(()) });

    let format = context.alloc_raw(10)?;
    write_null_terminated_string(&mut context, format, "%d")?;

    let dest = context.alloc_raw(10)?;

    kernel_methods[1].call(&mut context, Box::new([dest, format, 1234, 0, 0, 0, 0])).await?;

    let result = read_null_terminated_string(&context, dest)?;

    assert_eq!(result, "1234");

    Ok(())
}
