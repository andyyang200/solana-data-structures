use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use borsh::{BorshSerialize, BorshDeserialize};


const MAX_ACCOUNT_SIZE : u64 = 10000000;

pub const VECTOR_META_LEN : u64 = 48;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct VectorMeta{
    pub auth: Pubkey,
    pub element_size: u64,
    pub length: u64,
}



