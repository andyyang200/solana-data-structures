use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize};

use crate::instruction::{Instruction, InitializeParams,
                         initialize_heap_signed, push, pop, peek, delete};

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
                Self::process_pop(accounts)
            }
            Instruction::Peek => {
                msg!("Instruction: Peek");
                Self::process_peek(accounts)
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
        let (meta_bumper, heap_bumper_seeds) = seeds.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        let meta_seeds = &[auth.key.as_ref(), &max_length.to_le_bytes(), &element_size.to_le_bytes(), &[*meta_bumper]];
        initialize_heap_signed(accounts, max_length, element_size, program_id, meta_seeds, heap_bumper_seeds)?;
        Ok(())
    }

    fn process_push(
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> ProgramResult {
        push(accounts, data, &compare)?;
        Ok(())
    }

    fn process_pop(
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let res = pop(accounts, &compare)?;
        msg!("Popped the entry: {:?}", res);
        Ok(())
    }

    fn process_peek(
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let res = peek(accounts)?;
        msg!("Root entry: {:?}", res);
        Ok(())
    }

    fn process_delete(
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        delete(accounts)?;
        msg!("Removed the lamports from all the accounts");
        Ok(())
    }
}

fn compare(a: &Vec<u8>, b: &Vec<u8>) -> Result<i64, ProgramError> {
    if a.len() != b.len(){
        return Err(ProgramError::InvalidArgument)
    }

    for i in (0..a.len()).rev(){
        msg!("a: {}", a[i]);
        msg!("b: {}", b[i]);
        if a[i] > b[i]{
            return Ok(1)
        }
        else if a[i] < b[i]{
            return Ok(-1)
        }
    }

    return Ok(0)
}
