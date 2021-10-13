use borsh::{BorshSerialize, BorshDeserialize};

pub const MAX_ACCOUNT_SIZE : u64 = 50;
pub const VECTOR_META_LEN : u64 = 40;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct VectorMeta{
    pub max_length: u64,
    pub element_size: u64,
    pub length: u64,
    pub max_elements_per_account: u64,
    pub max_bytes_per_account: u64,
}



