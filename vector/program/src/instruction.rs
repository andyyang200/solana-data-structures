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

use std::convert::TryInto;

use borsh::{BorshSerialize, BorshDeserialize};

use std::{str, cmp::min};

use crate::{error::VectorError, state::*};



#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct InitializeParams{
    pub element_size: u64,
    pub max_length: u64,

}


pub enum Instruction {
    


    /// Accounts:
    /// 1. Authority/feepayer 
    /// 2. Vector Meta Account
    /// 3. system program
    /// 4. rent
    /// 5. accounts for the list
    Initialize {
        params: InitializeParams,
    },


}

impl Instruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => Self::Initialize {
                params: Self::unpack_initialize_params(rest)?,
            },
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    fn unpack_initialize_params(input: &[u8]) -> Result<InitializeParams, ProgramError> {
        let params = InitializeParams::try_from_slice(&input).unwrap();
        Ok(params)
    }
}

fn initialize_vector(
    accounts: &[AccountInfo],
    max_length: u64,
    element_size: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    initialize_vector_signed(
        accounts,
        max_length,
        element_size,
        program_id,
        &[]
    )
}


fn initialize_vector_signed(
    accounts: &[AccountInfo],
    max_length: u64,
    element_size: u64,
    program_id: &Pubkey,
    signers_seeds: &[&[u8]],
) -> ProgramResult {

    // parse
    
    let account_info_iter = &mut accounts.iter().peekable();
    let auth = next_account_info(account_info_iter)?;
    let vector_meta_account = next_account_info(account_info_iter)?;
    // let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;
    let vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    msg!("done parsing accounts and instruction data");
    
    // create vector meta account if it doesn't exist
    if vector_meta_account.data_len() == 0{

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
            &[signers_seeds]
        )?;
    }

    let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    vector_meta.max_length = max_length;
    vector_meta.element_size = element_size;
    vector_meta.length = 0;
    vector_meta.max_elements_per_account = MAX_ACCOUNT_SIZE / element_size;
    vector_meta.max_bytes_per_account = vector_meta.max_elements_per_account * element_size;

    vector_meta.serialize(&mut *vector_meta_account.data.borrow_mut())?;

    let size_to_allocate = max_length * element_size;
    let vector_accounts_index = 0;
    while size_to_allocate > 0 {
        if vector_accounts_index == vector_accounts.len(){
            msg!("not enough accounts");
            return Err(ProgramError::NotEnoughAccountKeys);
        }


        let space = min(size_to_allocate, vector_meta.max_bytes_per_account);
        let required_lamports = rent.minimum_balance(space as usize);
        invoke_signed(
            &solana_program::system_instruction::create_account(
                auth.key,
                vector_accounts[vector_accounts_index].key,
                required_lamports,
                space,
                program_id,
            ),
            &[
                auth.clone(),
                vector_accounts[vector_accounts_index].clone(),
            ],
            &[signers_seeds]
        )?;

        size_to_allocate -= space;
        vector_accounts_index += 1;
    }



    msg!("completed initialize"); 

    Ok(())
}


fn push(
    accounts: &[AccountInfo],
    data: &[u8],
    program_id: &Pubkey,
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter().peekable();

    let vector_meta_account = next_account_info(account_info_iter)?;

    let vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    if data.len() % (vector_meta.element_size) as usize != 0{
        msg!("data length not multiple of element size");
        return Err(ProgramError::InvalidArgument);
    }

    let delta = data.len() as u64 / vector_meta.element_size;
    if vector_meta.length + delta > vector_meta.max_length{
        msg!("not enough space");
        return Err(VectorError::InsufficientSpace.into());
    }

    let vector_accounts_index = ((vector_meta.length + 1) / vector_meta.max_elements_per_account) as usize;
    let vector_data = &vector_accounts[vector_accounts_index].data.borrow();
    let vector_data_index = (vector_meta.length % vector_meta.max_elements_per_account) as usize;
    let data_index = 0;

    for data_index in 0..data.len(){
        if vector_data_index as u64 > vector_meta.max_bytes_per_account{
            vector_accounts_index += 1;
            let vector_data = &vector_accounts[vector_accounts_index].data.borrow();
            let vector_data_index = 0;
        }
        vector_data[vector_data_index] = data[data_index];
        vector_data_index += 1;
    }

    vector_meta.length += delta;
    vector_meta.serialize(&mut *vector_meta_account.data.borrow_mut())?;

    Ok(())
}

fn pop(
    accounts: &[AccountInfo],
    num_elements: u64,
    program_id: &Pubkey,
) -> Result<Vec<u8>, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let vector_meta_account = next_account_info(account_info_iter)?;

    let vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    if vector_meta.length < num_elements{
        msg!("not enough elements to pop");
        return Err(VectorError::InsufficientSpace.into());
    }

    let mut ret = Vec::new();

    let new_length = vector_meta.length - num_elements;
    let start = new_length + 1;

    let vector_accounts_index = (start / vector_meta.max_elements_per_account) as usize;
    let vector_data = &vector_accounts[vector_accounts_index].data.borrow();
    let vector_data_index = (start % vector_meta.max_elements_per_account) as usize;
    let data_index = 0;

    for x in 0..num_elements{
        if vector_data_index as u64 > vector_meta.max_bytes_per_account{
            vector_accounts_index += 1;
            let vector_data = &vector_accounts[vector_accounts_index].data.borrow();
            let vector_data_index = 0;
        }
        ret.push(vector_data[vector_data_index]);
        vector_data_index += 1;
    }

    vector_meta.length = new_length;
    vector_meta.serialize(&mut *vector_meta_account.data.borrow_mut())?;


    Ok(ret)
}

fn slice(
    accounts: &[AccountInfo],
    start: u64,
    end: u64,
    program_id: &Pubkey,
) -> Result<Vec<Vec<u8>>, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let vector_meta_account = next_account_info(account_info_iter)?;

    let vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;


    let mut ret = Vec::new();

    let num_elements = end - start;

    let vector_accounts_index = (start / vector_meta.max_elements_per_account) as usize;
    let vector_data = &vector_accounts[vector_accounts_index].data.borrow();
    let vector_data_index = (start % vector_meta.max_elements_per_account) as usize;
    let data_index = 0;

    for x in 0..num_elements{
        if vector_data_index as u64 > vector_meta.max_bytes_per_account{
            vector_accounts_index += 1;
            let vector_data = &vector_accounts[vector_accounts_index].data.borrow();
            let vector_data_index = 0;
        }
        ret.push(vector_data[vector_data_index..(vector_data_index + vector_meta.element_size as usize)].to_vec());
        vector_data_index += vector_meta.element_size as usize;
    }

    Ok(ret)
}

fn get(
    accounts: &[AccountInfo],
    index: u64,
    program_id: &Pubkey,
) -> Result<Vec<u8>, ProgramError> {
    Ok(slice(
        accounts,
        index,
        index + 1,
        program_id,
    )?[0])
}








