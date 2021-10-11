use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    system_instruction::*,
};

use borsh::{BorshSerialize, BorshDeserialize};

use std::str;

use crate::{error::*, instruction::*, state::*};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = Instruction::unpack(instruction_data)?;
        match instruction {
            Instruction::Initialize { params } => {
                msg!("Instruction: Initialize");
                Self::process_initialize(accounts, params, program_id)
            },
        }
    }

    fn process_initialize(
        accounts: &[AccountInfo],
        params: InitializeParams,
        program_id: &Pubkey,
    ) -> ProgramResult {

        // parse
        
        let account_info_iter = &mut accounts.iter();

        let auth = next_account_info(account_info_iter)?;
        let vector_meta_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;

        let element_size = params.element_size;
        let max_length = params.max_length;

        msg!("done parsing accounts and instruction data");
        
        // create vector meta account if it doesn't exist
        if (vector_meta_account.data_len() == 0){

            let seeds_vector_meta = &[b"vector_meta", auth.key.as_ref()];
            let (key, bump_seed) = Pubkey::find_program_address(seeds_vector_meta, program_id);
            if key != *vector_meta_account.key {
                msg!("wrong key");
                return Err(VectorError::UnexpectedAccount.into());;
            }
            let seeds_vector_meta_with_bump = &[b"vector_meta", auth.key.as_ref(), &[bump_seed]];

            let space = VECTOR_META_LEN;
            let required_lamports = rent.minimum_balance(space as usize);
            invoke_signed(
                &solana_program::system_instruction::create_account(
                    auth.key,
                    vector_meta_account.key,
                    required_lamports,
                    space,
                    program_id,
                ),
                &[
                    auth.clone(),
                    vector_meta_account.clone(),
                ],
                &[seeds_vector_meta_with_bump],
            )?;
        }

        let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

        vector_meta.auth = *auth.key;
        vector_meta.element_size = element_size;
        vector_meta.length = 0;

        vector_meta.serialize(&mut *vector_meta_account.data.borrow_mut())?;

        msg!("completed initialize"); 

        Ok(())
    }


}
