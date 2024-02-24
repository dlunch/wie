use alloc::string::String;

use wie_util::ByteReadWriteError;

#[derive(Debug)]
pub enum WIPICError {
    Unimplemented(String),
    InvalidMemoryAccess,
    BackendError(String),
}

impl From<ByteReadWriteError> for WIPICError {
    fn from(_: ByteReadWriteError) -> Self {
        WIPICError::InvalidMemoryAccess
    }
}

impl From<WIPICError> for anyhow::Error {
    fn from(e: WIPICError) -> Self {
        anyhow::anyhow!("{:?}", e)
    }
}
