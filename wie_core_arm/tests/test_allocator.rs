mod core;

use wie_core_arm::Allocator;

#[test]
fn test_allocator() -> anyhow::Result<()> {
    let mut core = core::test_core();

    Allocator::init(&mut core)?;
    let address = Allocator::alloc(&mut core, 10)?;

    assert_eq!(address, 0x40000004);

    Ok(())
}
