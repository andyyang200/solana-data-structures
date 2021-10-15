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

use crate::{error::DequeError, state::{MAX_ACCOUNT_SIZE, DEQUE_META_LEN, DequeMeta}};

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
    PushFront,
    PushBack,
    PopFront,
    PopBack,
    Get,
    Remove,
    Delete,
}

impl Instruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(tag: &u8) -> Result<Self, ProgramError> {

        Ok(match tag {
            0 => Self::Initialize,
            1 => Self::PushFront,
            2 => Self::PushBack,
            3 => Self::PopFront,
            4 => Self::PopBack,
            5 => Self::Get,
            6 => Self::Remove,
            7 => Self::Delete,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }   
}

pub fn initialize_deque(
    accounts: &[AccountInfo],
    max_length: u64,
    element_size: u64,
    program_id: &Pubkey,
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter().peekable();
    let auth = next_account_info(account_info_iter)?;
    let deque_meta_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;
    let mut deque_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        deque_accounts.push(next_account_info(account_info_iter)?);
    }
    
    // create deque meta account if it doesn't exist
    if deque_meta_account.data_len() == 0{

        let space = DEQUE_META_LEN;
        let required_lamports = rent.minimum_balance(space as usize);
        invoke(
            &solana_program::system_instruction::create_account(
                auth.key,
                deque_meta_account.key,
                required_lamports,
                space,
                program_id,
            ),
            &[
                auth.clone(),
                deque_meta_account.clone(),
                system_program.clone(),
            ],
        )?;
    }

    let mut deque_meta = DequeMeta::try_from_slice(&deque_meta_account.data.borrow())?;

    deque_meta.max_length = max_length;
    deque_meta.element_size = element_size;
    deque_meta.max_bytes = max_length * element_size;
    deque_meta.start = 0;
    deque_meta.length = 0;
    deque_meta.max_elements_per_account = MAX_ACCOUNT_SIZE / element_size;
    deque_meta.max_bytes_per_account = deque_meta.max_elements_per_account * element_size;

    deque_meta.serialize(&mut *deque_meta_account.data.borrow_mut())?;

    let mut size_to_allocate = max_length * element_size;
    let mut deque_accounts_index = 0;
    while size_to_allocate > 0 {
        if deque_accounts_index == deque_accounts.len(){
            msg!("Not enough accounts");
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        if deque_accounts[deque_accounts_index].data_len() == 0{
            let space = min(size_to_allocate, deque_meta.max_bytes_per_account);
            let required_lamports = rent.minimum_balance(space as usize);
            invoke(
                &solana_program::system_instruction::create_account(
                    auth.key,
                    deque_accounts[deque_accounts_index].key,
                    required_lamports,
                    space,
                    program_id,
                ),
                &[
                    auth.clone(),
                    deque_accounts[deque_accounts_index].clone(),
                    system_program.clone(),
                ]
            )?;

            size_to_allocate -= space;
            deque_accounts_index += 1;
        }
    }

    Ok(())
}

pub fn initialize_deque_signed(
    accounts: &[AccountInfo],
    max_length: u64,
    element_size: u64,
    program_id: &Pubkey,
    meta_seeds: &[&[u8]],
    deque_bump_seeds: &[u8],
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter().peekable();
    let auth = next_account_info(account_info_iter)?;
    let deque_meta_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;
    let mut deque_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        deque_accounts.push(next_account_info(account_info_iter)?);
    }
    
    // create deque meta account if it doesn't exist
    if deque_meta_account.data_len() == 0{

        let space = DEQUE_META_LEN;
        let required_lamports = rent.minimum_balance(space as usize);
        invoke_signed(
            &solana_program::system_instruction::create_account(
                auth.key,
                deque_meta_account.key,
                required_lamports,
                space,
                program_id,
            ),
            &[
                auth.clone(),
                deque_meta_account.clone(),
                system_program.clone(),
            ],
            &[meta_seeds]
        )?;
    }

    msg!("Created deque meta account");

    let mut deque_meta = DequeMeta::try_from_slice(&deque_meta_account.data.borrow())?;

    deque_meta.max_length = max_length;
    deque_meta.element_size = element_size;
    deque_meta.max_bytes = max_length * element_size;
    deque_meta.start = 0;
    deque_meta.length = 0;
    deque_meta.max_elements_per_account = MAX_ACCOUNT_SIZE / element_size;
    deque_meta.max_bytes_per_account = deque_meta.max_elements_per_account * element_size;

    deque_meta.serialize(&mut *deque_meta_account.data.borrow_mut())?;

    let mut size_to_allocate = max_length * element_size;
    let mut deque_accounts_index = 0;
    while size_to_allocate > 0 {
        if deque_accounts_index == deque_accounts.len(){
            msg!("Not enough accounts");
            return Err(ProgramError::NotEnoughAccountKeys);
        }


        let space = min(size_to_allocate, deque_meta.max_bytes_per_account);
        let required_lamports = rent.minimum_balance(space as usize);
        invoke_signed(
            &solana_program::system_instruction::create_account(
                auth.key,
                deque_accounts[deque_accounts_index].key,
                required_lamports,
                space,
                program_id,
            ),
            &[
                auth.clone(),
                deque_accounts[deque_accounts_index].clone(),
                system_program.clone(),
            ],
            &[&[deque_meta_account.key.as_ref(), &[deque_accounts_index as u8], 
              &[deque_bump_seeds[deque_accounts_index]]]],
        )?;

        size_to_allocate -= space;

        msg!("Created deque account {}", deque_accounts_index);

        deque_accounts_index += 1;
    }

    Ok(())
}

pub fn get_meta(
    accounts: &[AccountInfo],
) -> Result<DequeMeta, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let deque_meta_account = next_account_info(account_info_iter)?;
    let deque_meta = DequeMeta::try_from_slice(&deque_meta_account.data.borrow())?;

    Ok(deque_meta)
}

pub fn push_front(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter().peekable();

    let deque_meta_account = next_account_info(account_info_iter)?;

    let mut deque_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        deque_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut deque_meta = DequeMeta::try_from_slice(&deque_meta_account.data.borrow())?;

    if data.len() % (deque_meta.element_size) as usize != 0{
        msg!("Data length not multiple of element size");
        return Err(ProgramError::InvalidArgument);
    }

    let num_elements = data.len() as u64 / deque_meta.element_size;
    if deque_meta.length + num_elements > deque_meta.max_length{
        msg!("Not enough space");
        return Err(DequeError::InsufficientSpace.into());
    }
    let start = (deque_meta.start - num_elements + deque_meta.max_length) % deque_meta.max_length;

    let mut cur_byte = start * deque_meta.element_size;
    let mut deque_accounts_index = (start / deque_meta.max_elements_per_account) as usize;
    let mut deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
    let mut deque_data_index = ((start % deque_meta.max_elements_per_account) * deque_meta.element_size) as usize;

    for data_index in 0..data.len(){
        deque_data[deque_data_index] = data[data_index];
        deque_data_index += 1;
        cur_byte += 1;
        if cur_byte >= deque_meta.max_bytes{
            deque_accounts_index = 0;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
            cur_byte = 0;
        }
        else if deque_data_index as u64 >= deque_meta.max_bytes_per_account{
            deque_accounts_index += 1;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
        }
    }

    deque_meta.start = start;
    deque_meta.length += num_elements;
    deque_meta.serialize(&mut *deque_meta_account.data.borrow_mut())?;

    Ok(())
}

pub fn push_back(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter().peekable();

    let deque_meta_account = next_account_info(account_info_iter)?;

    let mut deque_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        deque_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut deque_meta = DequeMeta::try_from_slice(&deque_meta_account.data.borrow())?;

    if data.len() % (deque_meta.element_size) as usize != 0{
        msg!("Data length not multiple of element size");
        return Err(ProgramError::InvalidArgument);
    }

    let num_elements = data.len() as u64 / deque_meta.element_size;
    if deque_meta.length + num_elements > deque_meta.max_length{
        msg!("Not enough space");
        return Err(DequeError::InsufficientSpace.into());
    }
    let start = (deque_meta.start + deque_meta.length) % deque_meta.max_length;

    let mut cur_byte = start * deque_meta.element_size;
    let mut deque_accounts_index = (start / deque_meta.max_elements_per_account) as usize;
    let mut deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
    let mut deque_data_index = ((start % deque_meta.max_elements_per_account) * deque_meta.element_size) as usize;

    for data_index in 0..data.len(){
        deque_data[deque_data_index] = data[data_index];
        deque_data_index += 1;
        cur_byte += 1;
        if cur_byte >= deque_meta.max_bytes{
            deque_accounts_index = 0;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
            cur_byte = 0;
        }
        else if deque_data_index as u64 >= deque_meta.max_bytes_per_account{
            deque_accounts_index += 1;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
        }
    }

    deque_meta.length += num_elements;
    deque_meta.serialize(&mut *deque_meta_account.data.borrow_mut())?;

    Ok(())
}

pub fn pop_slice_front(
    accounts: &[AccountInfo],
    num_elements: u64,
) -> Result<Vec<Vec<u8>>, ProgramError> {
    let account_info_iter = &mut accounts.iter().peekable();

    let deque_meta_account = next_account_info(account_info_iter)?;

    let mut deque_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        deque_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut deque_meta = DequeMeta::try_from_slice(&deque_meta_account.data.borrow())?;

    if deque_meta.length < num_elements{
        msg!("Not enough elements to pop");
        return Err(DequeError::PopFromEmpty.into());
    }

    let mut ret = Vec::with_capacity(num_elements as usize);

    let new_length = deque_meta.length - num_elements;
    let start = deque_meta.start;

    let mut cur_byte = start * deque_meta.element_size;
    let mut deque_accounts_index = (start / deque_meta.max_elements_per_account) as usize;
    let mut deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
    let mut deque_data_index = ((start % deque_meta.max_elements_per_account) * deque_meta.element_size) as usize;

    for _x in 0..num_elements{
        ret.push(deque_data[deque_data_index..(deque_data_index + deque_meta.element_size as usize)].to_vec());
        deque_data_index += deque_meta.element_size as usize;
        cur_byte += deque_meta.element_size;
        if cur_byte >= deque_meta.max_bytes{
            deque_accounts_index = 0;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
            cur_byte = 0;
        }
        else if deque_data_index as u64 >= deque_meta.max_bytes_per_account{
            deque_accounts_index += 1;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
        }
    }

    deque_meta.length = new_length;
    deque_meta.start = (deque_meta.start + num_elements) % deque_meta.max_length;
    deque_meta.serialize(&mut *deque_meta_account.data.borrow_mut())?;

    Ok(ret)
}

pub fn pop_slice_back(
    accounts: &[AccountInfo],
    num_elements: u64,
) -> Result<Vec<Vec<u8>>, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let deque_meta_account = next_account_info(account_info_iter)?;

    let mut deque_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        deque_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut deque_meta = DequeMeta::try_from_slice(&deque_meta_account.data.borrow())?;

    if deque_meta.length < num_elements{
        msg!("Not enough elements to pop");
        return Err(DequeError::PopFromEmpty.into());
    }

    let mut ret = Vec::with_capacity(num_elements as usize);

    let new_length = deque_meta.length - num_elements;
    let start = (deque_meta.start + new_length + 1) % deque_meta.max_length;

    let mut cur_byte = start * deque_meta.element_size;
    let mut deque_accounts_index = (start / deque_meta.max_elements_per_account) as usize;
    let mut deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
    let mut deque_data_index = ((start % deque_meta.max_elements_per_account) * deque_meta.element_size) as usize;

    for _x in 0..num_elements{
        ret.push(deque_data[deque_data_index..(deque_data_index + deque_meta.element_size as usize)].to_vec());
        deque_data_index += deque_meta.element_size as usize;
        cur_byte += deque_meta.element_size;
        if cur_byte >= deque_meta.max_bytes{
            deque_accounts_index = 0;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
            cur_byte = 0;
        }
        else if deque_data_index as u64 >= deque_meta.max_bytes_per_account{
            deque_accounts_index += 1;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
        }
    }

    deque_meta.length = new_length;
    deque_meta.serialize(&mut *deque_meta_account.data.borrow_mut())?;

    Ok(ret)
}

pub fn pop_front(
    accounts: &[AccountInfo],
) -> Result<Vec<u8>, ProgramError> {

    Ok((pop_slice_front(
        accounts,
        1
    )?).pop().ok_or(ProgramError::InvalidArgument)?)

}

pub fn pop_back(
    accounts: &[AccountInfo],
) -> Result<Vec<u8>, ProgramError> {

    Ok((pop_slice_back(
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

    let deque_meta_account = next_account_info(account_info_iter)?;

    let mut deque_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        deque_accounts.push(next_account_info(account_info_iter)?);
    }

    let deque_meta = DequeMeta::try_from_slice(&deque_meta_account.data.borrow())?;

    if start >= deque_meta.length || end >= deque_meta.length || start > end {
        msg!("Index Out of Bounds");
        return Err(DequeError::IndexOutofBounds.into());
    }

    let num_elements = end - start;

    let mut ret = Vec::with_capacity(num_elements as usize);

    let start = (deque_meta.start + start) % deque_meta.max_length;
    let mut cur_byte = start * deque_meta.element_size;
    let mut deque_accounts_index = (start / deque_meta.max_elements_per_account) as usize;
    let mut deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
    let mut deque_data_index = ((start % deque_meta.max_elements_per_account) * deque_meta.element_size) as usize;

    for _x in 0..num_elements{
        ret.push(deque_data[deque_data_index..(deque_data_index + deque_meta.element_size as usize)].to_vec());
        deque_data_index += deque_meta.element_size as usize;
        cur_byte += deque_meta.element_size;
        if cur_byte >= deque_meta.max_bytes{
            deque_accounts_index = 0;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
            cur_byte = 0;
        }
        else if deque_data_index as u64 >= deque_meta.max_bytes_per_account{
            deque_accounts_index += 1;
            deque_data = deque_accounts[deque_accounts_index].data.borrow_mut();
            deque_data_index = 0;
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

    let deque_meta_account = next_account_info(account_info_iter)?;

    let mut deque_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        deque_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut deque_meta = DequeMeta::try_from_slice(&deque_meta_account.data.borrow())?;

    if start >= deque_meta.length || end >= deque_meta.length || start > end {
        msg!("Index Out of Bounds");
        return Err(DequeError::IndexOutofBounds.into());
    }

    let mut deque_account_refs = Vec::with_capacity(deque_accounts.len() as usize);
    for i in 0..deque_accounts.len(){
        deque_account_refs.push(deque_accounts[i].data.borrow_mut());
    }

    let num_elements = end - start;

    let mut ret = Vec::with_capacity(num_elements as usize);

    let bytes_to_shift = (deque_meta.length - end) * deque_meta.element_size; // number of bytes to shift forward

    let start = (deque_meta.start + start) % deque_meta.max_length;
    let end = (deque_meta.start + end) % deque_meta.max_length;

    let mut cur_byte = start * deque_meta.element_size;
    let mut deque_accounts_index = (start / deque_meta.max_elements_per_account) as usize;
    let mut deque_data_index = ((start % deque_meta.max_elements_per_account) * deque_meta.element_size) as usize;
    for _x in 0..num_elements{
        ret.push(deque_account_refs[deque_accounts_index][deque_data_index..(deque_data_index + deque_meta.element_size as usize)].to_vec());
        deque_data_index += deque_meta.element_size as usize;
        cur_byte += deque_meta.element_size;
        if cur_byte >= deque_meta.max_bytes{
            deque_accounts_index = 0;
            deque_data_index = 0;
            cur_byte = 0;
        }
        else if deque_data_index as u64 >= deque_meta.max_bytes_per_account{
            deque_accounts_index += 1;
            deque_data_index = 0;
        }
    }

    let new_length = deque_meta.length - num_elements;

    let mut cur_byte_a = start * deque_meta.element_size;
    let mut deque_accounts_index_a = (start / deque_meta.max_elements_per_account) as usize;
    let mut deque_data_index_a = ((start % deque_meta.max_elements_per_account) * deque_meta.element_size) as usize;
    let mut cur_byte_b = end * deque_meta.element_size;
    let mut deque_accounts_index_b = (end / deque_meta.max_elements_per_account) as usize;
    let mut deque_data_index_b = ((end % deque_meta.max_elements_per_account) * deque_meta.element_size) as usize;

    for _x in 0..bytes_to_shift{
        deque_account_refs[deque_accounts_index_a][deque_data_index_a] = deque_account_refs[deque_accounts_index_b][deque_data_index_b];

        deque_data_index_a += 1;
        cur_byte_a += 1;
        if cur_byte_a >= deque_meta.max_bytes{
            deque_accounts_index_a = 0;
            deque_data_index_a = 0;
            cur_byte_a = 0;
        }
        else if deque_data_index_a as u64 >= deque_meta.max_bytes_per_account{
            deque_accounts_index_a += 1;
            deque_data_index_a = 0;
        }
        
        deque_data_index_b += 1;
        cur_byte_b += 1;
        if cur_byte_b >= deque_meta.max_bytes{
            deque_accounts_index_b = 0;
            deque_data_index_b = 0;
            cur_byte_b = 0;
        }
        else if deque_data_index_b as u64 >= deque_meta.max_bytes_per_account{
            deque_accounts_index_b += 1;
            deque_data_index_b = 0;
        }
    }

    deque_meta.length = new_length;
    deque_meta.serialize(&mut *deque_meta_account.data.borrow_mut())?;

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
    let deque_meta_account = next_account_info(account_info_iter)?;
    let mut deque_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        deque_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut auth_lamports = auth.lamports.borrow_mut();
    let mut deque_meta_lamports = deque_meta_account.lamports.borrow_mut();

    **auth_lamports = auth_lamports.
        checked_add(**deque_meta_lamports)
        .ok_or(DequeError::Overflow)?;
    **deque_meta_lamports = 0;

    for i in 0..deque_accounts.len(){
        let mut account_lamports = deque_accounts[i].lamports.borrow_mut();
        **auth_lamports = auth_lamports
                .checked_add(**account_lamports)
                .ok_or(DequeError::Overflow)?;
        **account_lamports = 0;
    }

    Ok(())
}
