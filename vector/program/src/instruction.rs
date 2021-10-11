use solana_program::{
    program_error::ProgramError,
    msg,
};
use std::convert::TryInto;

use borsh::{BorshSerialize, BorshDeserialize};

use std::str;

use crate::{error::VectorError::*, state::*};



#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct InitializeParams{
    pub element_size: u64,
    pub max_length: u64
}


pub enum Instruction {
    


    /// Accounts:
    /// 1. Authority/feepayer 
    /// 2. Vector Meta Account
    /// 3. system program
    /// 4. rent
    Initialize {
        params: InitializeParams,
    },


}

impl Instruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::Initialize {
                params: Self::unpack_initialize_params(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_initialize_params(input: &[u8]) -> Result<InitializeParams, ProgramError> {
        let params = InitializeParams::try_from_slice(&input).unwrap();
        Ok(params)
    }
}
