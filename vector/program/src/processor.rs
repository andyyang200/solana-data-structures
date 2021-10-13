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
                let (inputs, seeds) = rest.split_at(16);
                let params = InitializeParams::try_from_slice(inputs).unwrap();
                Self::process_initialize(accounts, params.max_length, params.element_size, program_id, seeds)
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
        max_length: u64,
        element_size: u64,
        program_id: &Pubkey,
        seeds: &[u8],
    ) -> ProgramResult {
        let auth = next_account_info(&mut accounts.iter())?;
        let (meta_bumper, vector_bumper_seeds) = seeds.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        let meta_seeds = &[auth.key.as_ref(), &element_size.to_le_bytes(), &max_length.to_le_bytes(), &[*meta_bumper]];
        initialize_vector_signed(accounts, max_length, element_size, program_id, meta_seeds, vector_bumper_seeds)?;
        Ok(())
    }

    fn process_push(
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> ProgramResult {
        push(accounts, data)?;
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
