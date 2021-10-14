use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar}
};
use borsh::{BorshSerialize, BorshDeserialize};
use std::{cmp::min};

use crate::{error::VectorError, state::{MAX_ACCOUNT_SIZE, VECTOR_META_LEN, VectorMeta}};

#[derive(BorshDeserialize, PartialEq, Debug)]
pub struct InitializeParams{
    pub max_length: u64,
    pub element_size: u64,
}

#[derive(BorshDeserialize, PartialEq, Debug)]
pub struct PopParams{
    pub num_elements: u64,
}

#[derive(BorshDeserialize, PartialEq, Debug)]
pub struct GetParams{
    pub start: u64,
    pub end: u64,
}

pub enum Instruction {
    Initialize,
    Push,
    Pop,
    Get,
    Remove,
    Delete,
}

impl Instruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(tag: &u8) -> Result<Self, ProgramError> {

        Ok(match tag {
            0 => Self::Initialize,
            1 => Self::Push,
            2 => Self::Pop,
            3 => Self::Get,
            4 => Self::Remove,
            5 => Self::Delete,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }   
}

pub fn initialize_vector(
    accounts: &[AccountInfo],
    max_length: u64,
    element_size: u64,
    program_id: &Pubkey,
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter().peekable();
    let auth = next_account_info(account_info_iter)?;
    let vector_meta_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;
    let mut vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    msg!("Done parsing accounts and instruction data");
    
    // create vector meta account if it doesn't exist
    if vector_meta_account.data_len() == 0{

        let space = VECTOR_META_LEN;
        let required_lamports = rent.minimum_balance(space as usize);
        invoke(
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
                system_program.clone(),
            ],
        )?;
    }

    let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    vector_meta.max_length = max_length;
    vector_meta.element_size = element_size;
    vector_meta.length = 0;
    vector_meta.max_elements_per_account = MAX_ACCOUNT_SIZE / element_size;
    vector_meta.max_bytes_per_account = vector_meta.max_elements_per_account * element_size;

    vector_meta.serialize(&mut *vector_meta_account.data.borrow_mut())?;

    let mut size_to_allocate = max_length * element_size;
    let mut vector_accounts_index = 0;
    while size_to_allocate > 0 {
        if vector_accounts_index == vector_accounts.len(){
            msg!("Not enough accounts");
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        if vector_accounts[vector_accounts_index].data_len() == 0{
            let space = min(size_to_allocate, vector_meta.max_bytes_per_account);
            let required_lamports = rent.minimum_balance(space as usize);
            invoke(
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
                    system_program.clone(),
                ]
            )?;

            size_to_allocate -= space;
            vector_accounts_index += 1;
        }
    }

    msg!("Completed initialize"); 

    Ok(())
}

pub fn initialize_vector_signed(
    accounts: &[AccountInfo],
    max_length: u64,
    element_size: u64,
    program_id: &Pubkey,
    meta_seeds: &[&[u8]],
    vector_bump_seeds: &[u8],
) -> ProgramResult {
    msg!("Initialize vector signed");

    let account_info_iter = &mut accounts.iter().peekable();
    let auth = next_account_info(account_info_iter)?;
    let vector_meta_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;
    let mut vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    msg!("Done parsing accounts and instruction data");
    
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
                system_program.clone(),
            ],
            &[meta_seeds]
        )?;
    }

    msg!("Created vector meta account");

    let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    vector_meta.max_length = max_length;
    vector_meta.element_size = element_size;
    vector_meta.length = 0;
    vector_meta.max_elements_per_account = MAX_ACCOUNT_SIZE / element_size;
    vector_meta.max_bytes_per_account = vector_meta.max_elements_per_account * element_size;

    vector_meta.serialize(&mut *vector_meta_account.data.borrow_mut())?;

    let mut size_to_allocate = max_length * element_size;
    let mut vector_accounts_index = 0;
    while size_to_allocate > 0 {
        if vector_accounts_index == vector_accounts.len(){
            msg!("Not enough accounts");
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
                system_program.clone(),
            ],
            &[&[vector_meta_account.key.as_ref(), &[vector_accounts_index as u8], 
              &[vector_bump_seeds[vector_accounts_index]]]],
        )?;

        size_to_allocate -= space;

        msg!("Created vector account {}", vector_accounts_index);

        vector_accounts_index += 1;
    }

    msg!("Completed initialize"); 

    Ok(())
}

pub fn get_meta(
    accounts: &[AccountInfo],
) -> Result<VectorMeta, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let vector_meta_account = next_account_info(account_info_iter)?;
    let vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    Ok(vector_meta)
}

pub fn push(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter().peekable();

    let vector_meta_account = next_account_info(account_info_iter)?;

    let mut vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    if data.len() % (vector_meta.element_size) as usize != 0{
        msg!("Data length not multiple of element size");
        return Err(ProgramError::InvalidArgument);
    }

    let num_elements = data.len() as u64 / vector_meta.element_size;
    if vector_meta.length + num_elements > vector_meta.max_length{
        msg!("Not enough space");
        return Err(VectorError::InsufficientSpace.into());
    }

    let mut vector_accounts_index = (vector_meta.length / vector_meta.max_elements_per_account) as usize;
    let mut vector_data = vector_accounts[vector_accounts_index].data.borrow_mut();
    let mut vector_data_index = ((vector_meta.length % vector_meta.max_elements_per_account) * vector_meta.element_size) as usize;

    for data_index in 0..data.len(){
        vector_data[vector_data_index] = data[data_index];
        vector_data_index += 1;
        if vector_data_index as u64 >= vector_meta.max_bytes_per_account{
            vector_accounts_index += 1;
            vector_data = vector_accounts[vector_accounts_index].data.borrow_mut();
            vector_data_index = 0;
        }
    }

    vector_meta.length += num_elements;
    vector_meta.serialize(&mut *vector_meta_account.data.borrow_mut())?;

    Ok(())
}

pub fn pop_slice(
    accounts: &[AccountInfo],
    num_elements: u64,
) -> Result<Vec<Vec<u8>>, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let vector_meta_account = next_account_info(account_info_iter)?;

    let mut vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    if vector_meta.length < num_elements{
        msg!("Not enough elements to pop");
        return Err(VectorError::PopFromEmpty.into());
    }

    let mut ret = Vec::with_capacity(num_elements as usize);

    let new_length = vector_meta.length - num_elements;
    let start = new_length + 1;

    let mut vector_accounts_index = (start / vector_meta.max_elements_per_account) as usize;
    let mut vector_data = vector_accounts[vector_accounts_index].data.borrow_mut();
    let mut vector_data_index = ((start % vector_meta.max_elements_per_account) * vector_meta.element_size) as usize;

    for _x in 0..num_elements{
        ret.push(vector_data[vector_data_index..(vector_data_index + vector_meta.element_size as usize)].to_vec());
        vector_data_index += vector_meta.element_size as usize;
        if vector_data_index as u64 >= vector_meta.max_bytes_per_account{
            vector_accounts_index += 1;
            vector_data = vector_accounts[vector_accounts_index].data.borrow_mut();
            vector_data_index = 0;
        }
    }

    vector_meta.length = new_length;
    vector_meta.serialize(&mut *vector_meta_account.data.borrow_mut())?;

    Ok(ret)
}

pub fn pop(
    accounts: &[AccountInfo],
) -> Result<Vec<u8>, ProgramError> {

    Ok((pop_slice(
        accounts,
        1
    )?).pop().ok_or(ProgramError::InvalidArgument)?)

}

pub fn slice(
    accounts: &[AccountInfo],
    start: u64,
    end: u64,
) -> Result<Vec<Vec<u8>>, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let vector_meta_account = next_account_info(account_info_iter)?;

    let mut vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    let vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    if start >= vector_meta.length || end >= vector_meta.length || start > end {
        msg!("Index Out of Bounds");
        return Err(VectorError::IndexOutofBounds.into());
    }

    let num_elements = end - start;

    let mut ret = Vec::with_capacity(num_elements as usize);

    let mut vector_accounts_index = (start / vector_meta.max_elements_per_account) as usize;
    let mut vector_data = vector_accounts[vector_accounts_index].data.borrow_mut();
    let mut vector_data_index = ((start % vector_meta.max_elements_per_account) * vector_meta.element_size) as usize;

    for _x in 0..num_elements{
        ret.push(vector_data[vector_data_index..(vector_data_index + vector_meta.element_size as usize)].to_vec());
        vector_data_index += vector_meta.element_size as usize;
        if vector_data_index as u64 >= vector_meta.max_bytes_per_account{
            vector_accounts_index += 1;
            vector_data = vector_accounts[vector_accounts_index].data.borrow_mut();
            vector_data_index = 0;
        }
    }

    Ok(ret)
}

pub fn get(
    accounts: &[AccountInfo],
    index: u64,
) -> Result<Vec<u8>, ProgramError> {
    Ok((slice(
        accounts,
        index,
        index + 1,
    )?).pop().ok_or(ProgramError::InvalidArgument)?)
}

pub fn remove_slice(
    accounts: &[AccountInfo],
    start: u64,
    end: u64,
) -> Result<Vec<Vec<u8>>, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let vector_meta_account = next_account_info(account_info_iter)?;

    let mut vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut vector_meta = VectorMeta::try_from_slice(&vector_meta_account.data.borrow())?;

    if start >= vector_meta.length || end >= vector_meta.length || start > end {
        msg!("Index Out of Bounds");
        return Err(VectorError::IndexOutofBounds.into());
    }

    let mut vector_account_refs = Vec::with_capacity(vector_accounts.len());
    for i in 0..vector_accounts.len(){
        vector_account_refs.push(vector_accounts[i].data.borrow_mut());
    }

    let num_elements = end - start;

    let mut ret = Vec::with_capacity(num_elements as usize);

    let mut vector_accounts_index = (start / vector_meta.max_elements_per_account) as usize;
    let mut vector_data_index = ((start % vector_meta.max_elements_per_account) * vector_meta.element_size) as usize;
    for _x in 0..num_elements{
        ret.push(vector_account_refs[vector_accounts_index][vector_data_index..(vector_data_index + vector_meta.element_size as usize)].to_vec());
        vector_data_index += vector_meta.element_size as usize;
        if vector_data_index as u64 >= vector_meta.max_bytes_per_account{
            vector_accounts_index += 1;
            vector_data_index = 0;
        }
    }

    let new_length = vector_meta.length - num_elements;

    let mut vector_accounts_index_a = (start / vector_meta.max_elements_per_account) as usize;
    let mut vector_data_index_a = ((start % vector_meta.max_elements_per_account) * vector_meta.element_size) as usize;
    let mut vector_accounts_index_b = (end / vector_meta.max_elements_per_account) as usize;
    let mut vector_data_index_b = ((end % vector_meta.max_elements_per_account) * vector_meta.element_size) as usize;
    for _x in 0..(new_length - start) * vector_meta.element_size{
        vector_account_refs[vector_accounts_index_a][vector_data_index_a] = vector_account_refs[vector_accounts_index_b][vector_data_index_b];

        vector_data_index_a += 1;
        if vector_data_index_a as u64 >= vector_meta.max_bytes_per_account{
            vector_accounts_index_a += 1;
            vector_data_index_a = 0;
        }

        vector_data_index_b += 1;
        if vector_data_index_b as u64 >= vector_meta.max_bytes_per_account{
            vector_accounts_index_b += 1;
            vector_data_index_b = 0;
        }
    }

    vector_meta.length = new_length;
    vector_meta.serialize(&mut *vector_meta_account.data.borrow_mut())?;

    Ok(ret)
}

pub fn remove(
    accounts: &[AccountInfo],
    index: u64,
) -> Result<Vec<u8>, ProgramError> {
    Ok((remove_slice(
        accounts,
        index,
        index + 1,
    )?).pop().ok_or(ProgramError::InvalidArgument)?)
}

pub fn delete(
    accounts: &[AccountInfo],
) -> ProgramResult{

    let account_info_iter = &mut accounts.iter().peekable();
    let auth = next_account_info(account_info_iter)?;
    let vector_meta_account = next_account_info(account_info_iter)?;
    let mut vector_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        vector_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut auth_lamports = auth.lamports.borrow_mut();
    let mut vector_meta_lamports = vector_meta_account.lamports.borrow_mut();

    **auth_lamports = auth_lamports.
        checked_add(**vector_meta_lamports)
        .ok_or(VectorError::Overflow)?;
    **vector_meta_lamports = 0;

    for i in 0..vector_accounts.len(){
        let mut account_lamports = vector_accounts[i].lamports.borrow_mut();
        **auth_lamports = auth_lamports
                .checked_add(**account_lamports)
                .ok_or(VectorError::Overflow)?;
        **account_lamports = 0;
    }

    Ok(())
}
