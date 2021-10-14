use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum VectorError {
    #[error("Insufficient Space")]
    InsufficientSpace,
    #[error("Pop From Empty Vector")]
    PopFromEmpty,
    #[error("Overflow")]
    Overflow,
    #[error("IndexOutofBounds")]
    IndexOutofBounds,
}

impl From<VectorError> for ProgramError {
    fn from(e: VectorError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
