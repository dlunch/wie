use core::{net::Ipv4Addr, str};

use wie_util::{Result, read_null_terminated_string_bytes};

use wipi_types::wipic::WIPICWord;

use crate::context::WIPICContext;

pub async fn htonl(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilHtonl({val:#x})");

    Ok(val.swap_bytes())
}

pub async fn htons(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilHtons({val})");

    Ok((val as u16).to_be() as _) // XXX we're always on little endian
}

pub async fn inet_addr(context: &mut dyn WIPICContext, address: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilInetAddr({address:#x})");

    let bytes = read_null_terminated_string_bytes(context, address)?;
    let address = str::from_utf8(&bytes).ok().and_then(|value| value.parse::<Ipv4Addr>().ok());
    Ok(address.map_or(u32::MAX, |address| u32::from_le_bytes(address.octets())))
}
