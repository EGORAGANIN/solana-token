use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TransferError {
    #[error("Account is non writable")]
    AccountNonWritable
}

impl From<TransferError> for ProgramError {
    fn from(e: TransferError) -> Self {
        ProgramError::Custom(e as u32)
    }
}