use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum DequeError {
    #[error("Insufficient Space")]
    InsufficientSpace,
    #[error("Pop From Empty Deque")]
    PopFromEmpty,
    #[error("Overflow")]
    Overflow,
    #[error("IndexOutofBounds")]
    IndexOutofBounds,
}

impl From<DequeError> for ProgramError {
    fn from(e: DequeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
