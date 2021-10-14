use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum HeapError {
    #[error("Insufficient Space")]
    InsufficientSpace,
    #[error("Removing From Empty Heap")]
    RemoveFromEmpty,
    #[error("Overflow")]
    Overflow,
    #[error("IndexOutofBounds")]
    IndexOutofBounds,
}

impl From<HeapError> for ProgramError {
    fn from(e: HeapError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
