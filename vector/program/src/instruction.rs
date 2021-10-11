use borsh::{BorshDeserialize};

#[derive(BorshDeserialize)]
pub enum VectorInstruction {
    Initialize{
        max_length: u64,
        element_size: u64
    },

    Push {
        input: &[u8]
    },

    Pop,

    Get {
        index: u64
    },
}
