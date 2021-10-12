use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    system_instruction::*,
};

use borsh::{BorshDeserialize};

use crate::error::VectorError;
use crate::instruction::{Instruction, InitializeParams, PopParams, GetParams, SliceParams,
                         initialize_vector_signed, push, pop, get, slice, delete};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let (tag, rest) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        let instruction = Instruction::unpack(tag)?;
        match instruction {
            Instruction::Initialize => {
                msg!("Instruction: Initialize");
                let params = InitializeParams::try_from_slice(rest).unwrap();
                Self::process_initialize(accounts, params.element_size, params.max_length, program_id)
            },
            Instruction::Push => {
                msg!("Instruction: Push");
                Self::process_push(accounts, rest)
            }
            Instruction::Pop => {
                msg!("Instruction: Pop");
                let params = PopParams::try_from_slice(rest).unwrap();
                Self::process_pop(accounts, params.num_elements)
            }
            Instruction::Get => {
                msg!("Instruction: Get");
                let params = GetParams::try_from_slice(rest).unwrap();
                Self::process_get(accounts, params.index)
            }
            Instruction::Slice => {
                msg!("Instruction: Slice");
                let params = SliceParams::try_from_slice(rest).unwrap();
                Self::process_slice(accounts, params.start, params.end)
            }
            Instruction::Delete => {
                msg!("Instruction: Delete");
                Self::process_delete(accounts)
            }

        }
    }

    fn process_initialize(
        accounts: &[AccountInfo],
        element_size: u64,
        max_length: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {

        initialize_vector_signed(accounts, max_length, element_size, program_id, &[]);
        Ok(())
    }

    fn process_push(
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> ProgramResult {
        Ok(())
    }

    fn process_pop(
        accounts: &[AccountInfo],
        num_elements: u64,
    ) -> ProgramResult {
        Ok(())
    }

    fn process_get(
        accounts: &[AccountInfo],
        index: u64,
    ) -> ProgramResult {
        Ok(())
    }

    fn process_slice(
        accounts: &[AccountInfo],
        start: u64,
        end: u64,
    ) -> ProgramResult {
        Ok(())
    }

    fn process_delete(
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        Ok(())
    }
}
