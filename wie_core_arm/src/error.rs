use alloc::string::String;
use wie_util::ByteReadWriteError;

#[derive(Debug)]
pub enum ArmCoreError {
    InvalidMemoryAccess,
    FunctionCallError(String),
    Other,
}

impl From<ByteReadWriteError> for ArmCoreError {
    fn from(_: ByteReadWriteError) -> Self {
        ArmCoreError::InvalidMemoryAccess
    }
}

impl From<ArmCoreError> for ByteReadWriteError {
    fn from(_: ArmCoreError) -> Self {
        ByteReadWriteError::InvalidAddress
    }
}

impl From<ArmCoreError> for anyhow::Error {
    fn from(e: ArmCoreError) -> Self {
        anyhow::anyhow!("{:?}", e)
    }
}
