use alloc::{format, string::ToString, sync::Arc, vec, vec::Vec};

use spin::Mutex;

use wie_backend::extract_zip;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{ByteRead, Result, WieError};

use crate::{
    adf::{KtfAdf, find_client_bin, parse_bss_size},
    emulator::IMAGE_BASE,
};

pub async fn dump_image(zip: &[u8]) -> Result<Vec<u8>> {
    let files = extract_zip(zip)?;

    let adf_bytes = files
        .get("__adf__")
        .ok_or_else(|| WieError::FatalError("Missing __adf__ in KTF archive".to_string()))?;
    let adf = KtfAdf::parse(adf_bytes);
    if adf.aid.is_empty() {
        return Err(WieError::FatalError("Missing AID in __adf__".to_string()));
    }

    let jar_filename = format!("{}.jar", adf.aid);
    let jar = files
        .get(&jar_filename)
        .ok_or_else(|| WieError::FatalError(format!("Missing {jar_filename} in KTF archive")))?;

    let (client_bin_name, data) = find_client_bin(jar)?;
    let bss_size = parse_bss_size(&client_bin_name)?;

    let mut core = ArmCore::new(false, None)?;
    Allocator::init(&mut core)?;

    let buf_slot: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));

    let mut core_inner = core.clone();
    let buf_slot_clone = buf_slot.clone();
    core.run_in_thread(async move || {
        core_inner.load(&data, IMAGE_BASE, data.len() + bss_size as usize)?;

        let _ = core_inner.run_function::<u32>(IMAGE_BASE + 1, &[bss_size]).await?;

        let total = data.len() + bss_size as usize;
        let mut buf = vec![0u8; total];
        let read = core_inner.read_bytes(IMAGE_BASE, &mut buf)?;
        if read != total {
            return Err(WieError::FatalError(format!("Short read from IMAGE_BASE: expected {total}, got {read}")));
        }

        buf_slot_clone.lock().replace(buf);
        Ok(())
    })?
    .await?;

    Ok(buf_slot.lock().take().expect("buf populated by closure"))
}
