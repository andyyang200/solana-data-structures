use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize};

use crate::instruction::{Instruction, InitializeParams, PopParams, GetParams,
                         initialize_deque_signed, push_front, push_back, pop_slice_front, pop_slice_back, 
                         slice, remove_slice, delete};

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
            Instruction::PushFront => {
                msg!("Instruction: PushFront");
                Self::process_push_front(accounts, rest)
            }
            Instruction::PushBack => {
                msg!("Instruction: PushBack");
                Self::process_push_back(accounts, rest)
            }
            Instruction::PopFront => {
                msg!("Instruction: PopFront");
                let params = PopParams::try_from_slice(rest).unwrap();
                Self::process_pop_front(accounts, params.num_elements)
            }
            Instruction::PopBack => {
                msg!("Instruction: PopBack");
                let params = PopParams::try_from_slice(rest).unwrap();
                Self::process_pop_back(accounts, params.num_elements)
            }
            Instruction::Get => {
                msg!("Instruction: Get");
                let params = GetParams::try_from_slice(rest).unwrap();
                Self::process_get(accounts, params.start, params.end)
            }
            Instruction::Remove => {
                msg!("Instruction: Remove");
                let params = GetParams::try_from_slice(rest).unwrap();
                Self::process_remove(accounts, params.start, params.end)
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
        let (meta_bumper, deque_bumper_seeds) = seeds.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        let meta_seeds = &[auth.key.as_ref(), &max_length.to_le_bytes(), &element_size.to_le_bytes(), &[*meta_bumper]];
        initialize_deque_signed(accounts, max_length, element_size, program_id, meta_seeds, deque_bumper_seeds)?;
        Ok(())
    }

    fn process_push_front(
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> ProgramResult {
        push_front(accounts, data)?;
        Ok(())
    }

    fn process_push_back(
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> ProgramResult {
        push_back(accounts, data)?;
        Ok(())
    }

    fn process_pop_front(
        accounts: &[AccountInfo],
        num_elements: u64,
    ) -> ProgramResult {
        let res = pop_slice_front(accounts, num_elements)?;
        msg!("Popped the entries from front:");
        for i in 0..res.len(){
            msg!{"{:?}", res[i]};
        }
        Ok(())
    }

    fn process_pop_back(
        accounts: &[AccountInfo],
        num_elements: u64,
    ) -> ProgramResult {
        let res = pop_slice_back(accounts, num_elements)?;
        msg!("Popped the entries from back:");
        for i in 0..res.len(){
            msg!{"{:?}", res[i]};
        }
        Ok(())
    }

    fn process_get(
        accounts: &[AccountInfo],
        start: u64,
        end: u64,
    ) -> ProgramResult {
        let res = slice(accounts, start, end)?;
        msg!("Got the entries:");
        for i in 0..res.len(){
            msg!{"{:?}", res[i]};
        }
        Ok(())
    }

    fn process_remove(
        accounts: &[AccountInfo],
        start: u64,
        end: u64,
    ) -> ProgramResult {
        let res = remove_slice(accounts, start, end)?;
        msg!("Removed the entries:");
        for i in 0..res.len(){
            msg!{"{:?}", res[i]};
        }
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
