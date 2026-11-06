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

pub async fn ntohs(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilNtohs({val})");

    Ok(u16::from_be(val as u16) as _) // XXX we're always on little endian
}

pub async fn inet_addr(context: &mut dyn WIPICContext, address: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilInetAddrInt/MC_utilInetAddr({address:#x})");

    let bytes = read_null_terminated_string_bytes(context, address)?;
    let address = str::from_utf8(&bytes).ok().and_then(|value| value.parse::<Ipv4Addr>().ok());
    Ok(address.map_or(u32::MAX, |address| u32::from_le_bytes(address.octets())))
}

#[cfg(test)]
mod tests {
    use wie_util::{ByteWrite, Result};

    use crate::context::test::TestContext;

    use super::inet_addr;

    #[futures_test::test]
    async fn ipv4_conversion_returns_network_order() -> Result<()> {
        let mut context = TestContext::new();
        for (text, expected) in [
            (b"127.0.0.1\0".as_slice(), [127, 0, 0, 1]),
            (b"192.168.1.25\0".as_slice(), [192, 168, 1, 25]),
            (b"0.0.0.0\0".as_slice(), [0, 0, 0, 0]),
        ] {
            context.write_bytes(0x1000, text)?;
            assert_eq!(inet_addr(&mut context, 0x1000).await?.to_le_bytes(), expected);
        }
        for text in [b"256.0.0.1\0".as_slice(), b"invalid\0".as_slice(), b"\xff\0".as_slice()] {
            context.write_bytes(0x1000, text)?;
            assert_eq!(inet_addr(&mut context, 0x1000).await?, u32::MAX);
        }
        Ok(())
    }
}
