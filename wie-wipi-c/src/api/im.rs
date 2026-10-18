use wie_util::{Result, read_generic, write_generic, write_null_terminated_table};

use wipi_types::wipic::WIPICWord;

use crate::{context::WIPICContext, globals::INPUT_MODES_PTR};

const INPUT_MODES: [&[u8]; 4] = [b"EN/L\0", b"EN/S\0", b"KO\0", b"N123\0"];

pub async fn get_support_mode_count(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::debug!("MC_imGetSurpportModeCount()");
    Ok(INPUT_MODES.len() as i32)
}

pub async fn get_supported_modes(context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::debug!("MC_imGetSupportedModes()");

    let modes: u32 = read_generic(context, INPUT_MODES_PTR)?;
    if modes != 0 {
        return Ok(modes);
    }

    let table_size = ((INPUT_MODES.len() + 1) * size_of::<u32>()) as u32;
    let strings_size = INPUT_MODES.iter().map(|mode| mode.len() as u32).sum::<u32>();
    let modes = context.alloc_raw(table_size + strings_size)?;
    let mut string_ptr = modes + table_size;
    let mut pointers = [0; INPUT_MODES.len()];
    for (pointer, mode) in pointers.iter_mut().zip(INPUT_MODES) {
        *pointer = string_ptr;
        context.write_bytes(string_ptr, mode)?;
        string_ptr += mode.len() as u32;
    }
    write_null_terminated_table(context, modes, &pointers)?;
    write_generic(context, INPUT_MODES_PTR, modes)?;
    Ok(modes)
}

pub async fn set_current_mode(_context: &mut dyn WIPICContext, mode: i32) -> Result<i32> {
    tracing::warn!("stub MC_imSetCurrentMode({mode})");
    Ok(-1)
}

pub async fn get_current_mode(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_imGetCurrentMode()");
    Ok(-1)
}

pub async fn handle_input(
    context: &mut dyn WIPICContext,
    key: WIPICWord,
    event_type: i32,
    ptr_committed: WIPICWord,
    ptr_committed_size: WIPICWord,
    ptr_composing: WIPICWord,
    ptr_composing_size: WIPICWord,
) -> Result<i32> {
    tracing::warn!(
        "stub MC_imHandleInput({key:#x}, {event_type}, {ptr_committed:#x}, {ptr_committed_size:#x}, {ptr_composing:#x}, {ptr_composing_size:#x})"
    );

    let capacity: i32 = read_generic(context, ptr_committed_size)?;
    // shortcut: numeric key presses commit a placeholder until text input is implemented.
    let size = if event_type == 2 && (u32::from(b'0')..=u32::from(b'9')).contains(&key) {
        capacity.clamp(0, 4)
    } else {
        0
    };
    if size > 0 {
        context.write_bytes(ptr_committed, &b"temp"[..size as usize])?;
    }
    write_generic(context, ptr_committed_size, size)?;
    write_generic(context, ptr_composing_size, 0i32)?;

    Ok(i32::from(size > 0))
}

#[cfg(test)]
mod tests {
    use wie_util::{Result, read_null_terminated_string_bytes, read_null_terminated_table};

    use crate::context::test::TestContext;

    use super::{get_support_mode_count, get_supported_modes};

    #[futures_test::test]
    async fn input_mode_queries_reuse_valid_guest_strings() -> Result<()> {
        let mut context = TestContext::new();
        let count = get_support_mode_count(&mut context).await?;
        let modes = get_supported_modes(&mut context).await?;
        let pointers = read_null_terminated_table(&context, modes)?;
        let expected = [b"EN/L".as_slice(), b"EN/S", b"KO", b"N123"];
        assert_eq!(count, expected.len() as i32);
        assert_eq!(pointers.len(), expected.len());
        for (pointer, name) in pointers.into_iter().zip(expected) {
            assert_eq!(read_null_terminated_string_bytes(&context, pointer)?, name);
        }
        assert_eq!(get_supported_modes(&mut context).await?, modes);
        Ok(())
    }
}
