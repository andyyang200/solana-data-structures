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

use crate::{error::HeapError, state::{MAX_ACCOUNT_SIZE, HEAP_META_LEN, HeapMeta}};

#[derive(BorshDeserialize, PartialEq, Debug)]
pub struct InitializeParams{
    pub start_length: u64,
    pub max_length: u64,
    pub element_size: u64,
}

pub enum Instruction {
    Initialize,
    Push,
    Pop,
    Peek,
    Delete,
}

impl Instruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(tag: &u8) -> Result<Self, ProgramError> {

        Ok(match tag {
            0 => Self::Initialize,
            1 => Self::Push,
            2 => Self::Pop,
            3 => Self::Peek,
            4 => Self::Delete,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }   
}

fn push_down(
    data: &mut Vec<Vec<u8>>,
    i: usize,
    n: usize,
    compare: impl Fn(&Vec<u8>, &Vec<u8>) -> Result<i64, ProgramError>
) -> ProgramResult {

    let mut smallest = i; // Initialize largest as root
    let l = 2 * i + 1; // left = 2*i + 1
    let r = 2 * i + 2; // right = 2*i + 2
  
    // If left child is larger than root
    if l < n && compare(&data[l], &data[smallest])? < 0{
        smallest = l;
    }
  
    // If right child is larger than largest so far
    if r < n && compare(&data[r], &data[smallest])? < 0{
        smallest = r;
    }
  
    // If largest is not root
    if smallest != i {
        let tmp = data[i].clone();
        data[i] = data[smallest].clone();
        data[smallest] = tmp;
  
        // Recursively heapify the affected sub-tree
        push_down(data, smallest, n, compare)?;
    }

    Ok(())
}

fn heapify(
    data: &mut Vec<Vec<u8>>,
    compare: impl Fn(&Vec<u8>, &Vec<u8>) -> Result<i64, ProgramError>
) -> ProgramResult {
    let n = data.len();
    for i in (0..n/2 + 1).rev(){
        push_down(data, i, n, &compare)?;
    }

    Ok(())
}

pub fn initialize_heap(
    accounts: &[AccountInfo],
    max_length: u64,
    element_size: u64,
    data: &[u8],
    compare: impl Fn(&Vec<u8>, &Vec<u8>) -> Result<i64, ProgramError>,
    program_id: &Pubkey,
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter().peekable();
    let auth = next_account_info(account_info_iter)?;
    let heap_meta_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;
    let mut heap_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        heap_accounts.push(next_account_info(account_info_iter)?);
    }

    if data.len() % element_size as usize != 0{
        msg!("Data length not multiple of element size");
        return Err(ProgramError::InvalidArgument);
    }
    
    // create heap meta account if it doesn't exist
    if heap_meta_account.data_len() == 0{

        let space = HEAP_META_LEN;
        let required_lamports = rent.minimum_balance(space as usize);
        invoke(
            &solana_program::system_instruction::create_account(
                auth.key,
                heap_meta_account.key,
                required_lamports,
                space,
                program_id,
            ),
            &[
                auth.clone(),
                heap_meta_account.clone(),
                system_program.clone(),
            ],
        )?;
    }

    let mut heap_meta = HeapMeta::try_from_slice(&heap_meta_account.data.borrow())?;

    heap_meta.max_length = max_length;
    heap_meta.element_size = element_size;
    heap_meta.max_bytes = max_length * element_size;
    heap_meta.length = data.len() as u64 / element_size;
    heap_meta.max_elements_per_account = MAX_ACCOUNT_SIZE / element_size;
    heap_meta.max_bytes_per_account = heap_meta.max_elements_per_account * element_size;

    heap_meta.serialize(&mut *heap_meta_account.data.borrow_mut())?;

    let mut size_to_allocate = max_length * element_size;
    let mut heap_accounts_index = 0;
    while size_to_allocate > 0 {
        if heap_accounts_index == heap_accounts.len(){
            msg!("Not enough accounts");
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        if heap_accounts[heap_accounts_index].data_len() == 0{
            let space = min(size_to_allocate, heap_meta.max_bytes_per_account);
            let required_lamports = rent.minimum_balance(space as usize);
            invoke(
                &solana_program::system_instruction::create_account(
                    auth.key,
                    heap_accounts[heap_accounts_index].key,
                    required_lamports,
                    space,
                    program_id,
                ),
                &[
                    auth.clone(),
                    heap_accounts[heap_accounts_index].clone(),
                    system_program.clone(),
                ]
            )?;

            size_to_allocate -= space;
            heap_accounts_index += 1;
        }
    }

    let start_length = data.len() / element_size as usize;
    let mut data_vec = Vec::with_capacity(start_length);
    for i in 0..start_length as usize{
        data_vec.push(Vec::with_capacity(element_size as usize));
        for j in 0..element_size as usize{
            data_vec[i].push(data[i * element_size as usize + j]);
        }
    }

    heapify(&mut data_vec, &compare)?;

    let mut heap_accounts_index = 0;
    let mut heap_data = heap_accounts[heap_accounts_index].data.borrow_mut();
    let mut heap_data_index = 0;

    for i in 0..data.len(){
        heap_data[heap_data_index] = data_vec[i / element_size as usize][i % element_size as usize];
        heap_data_index += 1;
        if heap_data_index as u64 >= heap_meta.max_bytes_per_account{
            heap_accounts_index += 1;
            heap_data = heap_accounts[heap_accounts_index].data.borrow_mut();
            heap_data_index = 0;
        }
    }

    Ok(())
}

pub fn initialize_heap_signed(
    accounts: &[AccountInfo],
    max_length: u64,
    element_size: u64,
    data: &[u8],
    compare: impl Fn(&Vec<u8>, &Vec<u8>) -> Result<i64, ProgramError>,
    program_id: &Pubkey,
    meta_seeds: &[&[u8]],
    heap_bump_seeds: &[u8],
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter().peekable();
    let auth = next_account_info(account_info_iter)?;
    let heap_meta_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;
    let mut heap_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        heap_accounts.push(next_account_info(account_info_iter)?);
    }

    if data.len() % element_size as usize != 0{
        msg!("Data length not multiple of element size");
        return Err(ProgramError::InvalidArgument);
    }
    
    // create heap meta account if it doesn't exist
    if heap_meta_account.data_len() == 0{

        let space = HEAP_META_LEN;
        let required_lamports = rent.minimum_balance(space as usize);
        invoke_signed(
            &solana_program::system_instruction::create_account(
                auth.key,
                heap_meta_account.key,
                required_lamports,
                space,
                program_id,
            ),
            &[
                auth.clone(),
                heap_meta_account.clone(),
                system_program.clone(),
            ],
            &[meta_seeds]
        )?;
    }

    msg!("Created heap meta account");

    let mut heap_meta = HeapMeta::try_from_slice(&heap_meta_account.data.borrow())?;

    heap_meta.max_length = max_length;
    heap_meta.element_size = element_size;
    heap_meta.max_bytes = max_length * element_size;
    heap_meta.length = data.len() as u64 / element_size;
    heap_meta.max_elements_per_account = MAX_ACCOUNT_SIZE / element_size;
    heap_meta.max_bytes_per_account = heap_meta.max_elements_per_account * element_size;

    heap_meta.serialize(&mut *heap_meta_account.data.borrow_mut())?;


    let mut size_to_allocate = max_length * element_size;
    let mut heap_accounts_index = 0;
    while size_to_allocate > 0 {
        if heap_accounts_index == heap_accounts.len(){
            msg!("Not enough accounts");
            return Err(ProgramError::NotEnoughAccountKeys);
        }


        let space = min(size_to_allocate, heap_meta.max_bytes_per_account);
        let required_lamports = rent.minimum_balance(space as usize);
        invoke_signed(
            &solana_program::system_instruction::create_account(
                auth.key,
                heap_accounts[heap_accounts_index].key,
                required_lamports,
                space,
                program_id,
            ),
            &[
                auth.clone(),
                heap_accounts[heap_accounts_index].clone(),
                system_program.clone(),
            ],
            &[&[heap_meta_account.key.as_ref(), &[heap_accounts_index as u8], 
              &[heap_bump_seeds[heap_accounts_index]]]],
        )?;

        size_to_allocate -= space;

        msg!("Created heap account {}", heap_accounts_index);

        heap_accounts_index += 1;
    }

    let start_length = data.len() / element_size as usize;
    let mut data_vec = Vec::with_capacity(start_length);
    for i in 0..start_length as usize{
        data_vec.push(Vec::with_capacity(element_size as usize));
        for j in 0..element_size as usize{
            data_vec[i].push(data[i * element_size as usize + j]);
        }
    }

    heapify(&mut data_vec, &compare)?;

    let mut heap_accounts_index = 0;
    let mut heap_data = heap_accounts[heap_accounts_index].data.borrow_mut();
    let mut heap_data_index = 0;

    for i in 0..data.len(){
        heap_data[heap_data_index] = data_vec[i / element_size as usize][i % element_size as usize];
        heap_data_index += 1;
        if heap_data_index as u64 >= heap_meta.max_bytes_per_account{
            heap_accounts_index += 1;
            heap_data = heap_accounts[heap_accounts_index].data.borrow_mut();
            heap_data_index = 0;
        }
    }

    Ok(())
}

pub fn get_meta(
    accounts: &[AccountInfo],
) -> Result<HeapMeta, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let heap_meta_account = next_account_info(account_info_iter)?;
    let heap_meta = HeapMeta::try_from_slice(&heap_meta_account.data.borrow())?;

    Ok(heap_meta)
}

pub fn push(
    accounts: &[AccountInfo],
    data: &[u8],
    compare: impl Fn(&Vec<u8>, &Vec<u8>) -> Result<i64, ProgramError>
 ) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().peekable();

    let heap_meta_account = next_account_info(account_info_iter)?;

    let mut heap_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        heap_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut heap_meta = HeapMeta::try_from_slice(&heap_meta_account.data.borrow())?;

    if data.len() != heap_meta.element_size as usize{
        msg!("Not inserting a single element");
        return Err(ProgramError::InvalidArgument);
    }

    if heap_meta.length >= heap_meta.max_length{
        msg!("Not enough space");
        return Err(HeapError::InsufficientSpace.into());
    }

    let mut heap_account_refs = Vec::with_capacity(heap_accounts.len());
    for i in 0..heap_accounts.len(){
        heap_account_refs.push(heap_accounts[i].data.borrow_mut());
    }

    let mut cur = heap_meta.length;
    let mut heap_accounts_index_cur = (cur / heap_meta.max_elements_per_account) as usize;
    let mut heap_data_index_cur = ((cur % heap_meta.max_elements_per_account) * heap_meta.element_size) as usize;

    // write new element into account
    for i in 0..data.len(){
        heap_account_refs[heap_accounts_index_cur][heap_data_index_cur + i] = data[i];
    }

    // use swaps to maintain heap invariant
    let element_cur = (heap_account_refs[heap_accounts_index_cur][heap_data_index_cur..heap_data_index_cur + heap_meta.element_size as usize]).to_vec();
    while cur != 0{
        let par = (cur - 1) / 2;
        let heap_accounts_index_par = (par / heap_meta.max_elements_per_account) as usize;
        let heap_data_index_par = ((par % heap_meta.max_elements_per_account) * heap_meta.element_size) as usize;

        let element_par = (heap_account_refs[heap_accounts_index_par][heap_data_index_par..heap_data_index_par + heap_meta.element_size as usize]).to_vec();
        if compare(&element_cur, &element_par)? >= 0{
            break;
        }
        // swap with parent
        for i in 0..heap_meta.element_size as usize{
            let tmp = heap_account_refs[heap_accounts_index_cur][heap_data_index_cur + i];
            heap_account_refs[heap_accounts_index_cur][heap_data_index_cur + i] = heap_account_refs[heap_accounts_index_par][heap_data_index_par + i];
            heap_account_refs[heap_accounts_index_par][heap_data_index_par + i] = tmp;
        }
        cur = par;
        heap_accounts_index_cur = heap_accounts_index_par;
        heap_data_index_cur = heap_data_index_par;
    }

    heap_meta.length += 1;
    heap_meta.serialize(&mut *heap_meta_account.data.borrow_mut())?;

    Ok(())
}

pub fn pop(
    accounts: &[AccountInfo],
    compare: impl Fn(&Vec<u8>, &Vec<u8>) -> Result<i64, ProgramError>
 ) -> Result<Vec<u8>, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let heap_meta_account = next_account_info(account_info_iter)?;

    let mut heap_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        heap_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut heap_meta = HeapMeta::try_from_slice(&heap_meta_account.data.borrow())?;


    if heap_meta.length == 0{
        msg!("Removing from empty heap");
        return Err(HeapError::RemoveFromEmpty.into());
    }

    let mut heap_account_refs = Vec::with_capacity(heap_accounts.len());
    for i in 0..heap_accounts.len(){
        heap_account_refs.push(heap_accounts[i].data.borrow_mut());
    }

    // put root into heap
    let mut ret = Vec::with_capacity(heap_meta.element_size as usize);
    for i in 0..heap_meta.element_size{
        ret.push(heap_account_refs[0][i as usize]);
    }

    // write leaf into root
    let last = heap_meta.length - 1;
    let heap_accounts_index_leaf = (last / heap_meta.max_elements_per_account) as usize;
    let heap_data_index_leaf = ((last % heap_meta.max_elements_per_account) * heap_meta.element_size) as usize;
    for i in 0..heap_meta.element_size as usize{
        heap_account_refs[0][i] = heap_account_refs[heap_accounts_index_leaf][heap_data_index_leaf];
    }

    heap_meta.length -= 1;
    // swap root down to maintain heap invariant
    let mut cur = 0;
    let mut heap_accounts_index_cur = (cur / heap_meta.max_elements_per_account) as usize;
    let mut heap_data_index_cur = ((cur % heap_meta.max_elements_per_account) * heap_meta.element_size) as usize;
    let mut element_cur = (heap_account_refs[heap_accounts_index_cur][heap_data_index_cur..heap_data_index_cur + heap_meta.element_size as usize]).to_vec();
    loop {
        if 2 * cur + 1 >= heap_meta.length{ // no children
            break;
        }
        else if 2 * cur + 2 >= heap_meta.length{ // left child only
            let child = 2 * cur + 1;
            let heap_accounts_index_child = (child / heap_meta.max_elements_per_account) as usize;
            let heap_data_index_child = ((child % heap_meta.max_elements_per_account) * heap_meta.element_size) as usize;
            let element_child = (heap_account_refs[heap_accounts_index_child][heap_data_index_child..heap_data_index_child + heap_meta.element_size as usize]).to_vec();
            if compare(&element_cur, &element_child)? <= 0{
                break;
            }
            // swap with child
            for i in 0..heap_meta.element_size as usize{
                let tmp = heap_account_refs[heap_accounts_index_cur][heap_data_index_cur + i];
                heap_account_refs[heap_accounts_index_cur][heap_data_index_cur + i] = heap_account_refs[heap_accounts_index_child][heap_data_index_child + i];
                heap_account_refs[heap_accounts_index_child][heap_data_index_child + i] = tmp;
            }
        }
        else{ // two children
            let child_a = 2 * cur + 1;
            let mut heap_accounts_index_child_a = (child_a / heap_meta.max_elements_per_account) as usize;
            let mut heap_data_index_child_a = ((child_a % heap_meta.max_elements_per_account) * heap_meta.element_size) as usize;
            let mut element_child_a = (heap_account_refs[heap_accounts_index_child_a][heap_data_index_child_a..heap_data_index_child_a + heap_meta.element_size as usize]).to_vec();
            let child_b = 2 * cur + 2;
            let heap_accounts_index_child_b = (child_b / heap_meta.max_elements_per_account) as usize;
            let heap_data_index_child_b = ((child_b % heap_meta.max_elements_per_account) * heap_meta.element_size) as usize;
            let element_child_b = (heap_account_refs[heap_accounts_index_child_b][heap_data_index_child_b..heap_data_index_child_b + heap_meta.element_size as usize]).to_vec();
            if compare(&element_child_a, &element_child_b)? >= 0{ // let child a be the smaller one
                heap_accounts_index_child_a = heap_accounts_index_child_b;
                heap_data_index_child_a = heap_data_index_child_b;
                element_child_a = element_child_b;
            }
            if compare(&element_cur, &element_child_a)? <= 0{
                break;
            }
            // swap with child
            for i in 0..heap_meta.element_size as usize{
                let tmp = heap_account_refs[heap_accounts_index_cur][heap_data_index_cur + i];
                heap_account_refs[heap_accounts_index_cur][heap_data_index_cur + i] = heap_account_refs[heap_accounts_index_child_a][heap_data_index_child_a + i];
                heap_account_refs[heap_accounts_index_child_a][heap_data_index_child_a + i] = tmp;
            }

            cur = child_a;
            heap_accounts_index_cur = heap_accounts_index_child_a;
            heap_data_index_cur = heap_data_index_child_a;
            element_cur = element_child_a;
        }
    }

    heap_meta.serialize(&mut *heap_meta_account.data.borrow_mut())?;

    Ok(ret)
 }

 pub fn peek(
    accounts: &[AccountInfo],
 ) -> Result<Vec<u8>, ProgramError> {

    let account_info_iter = &mut accounts.iter().peekable();

    let heap_meta_account = next_account_info(account_info_iter)?;

    let mut heap_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        heap_accounts.push(next_account_info(account_info_iter)?);
    }

    let heap_meta = HeapMeta::try_from_slice(&heap_meta_account.data.borrow())?;

    if heap_meta.length == 0{
        msg!("Removing from empty heap");
        return Err(HeapError::RemoveFromEmpty.into());
    }

    let mut heap_account_refs = Vec::with_capacity(heap_accounts.len());
    for i in 0..heap_accounts.len(){
        heap_account_refs.push(heap_accounts[i].data.borrow_mut());
    }

    // put root into heap
    let mut ret = Vec::with_capacity(heap_meta.element_size as usize);
    for i in 0..heap_meta.element_size{
        ret.push(heap_account_refs[0][i as usize]);
    }
    Ok(ret)
}

pub fn delete(
    accounts: &[AccountInfo],
) -> ProgramResult{

    let account_info_iter = &mut accounts.iter().peekable();
    let auth = next_account_info(account_info_iter)?;
    let heap_meta_account = next_account_info(account_info_iter)?;
    let mut heap_accounts = Vec::new();
    while account_info_iter.peek().is_some(){
        heap_accounts.push(next_account_info(account_info_iter)?);
    }

    let mut auth_lamports = auth.lamports.borrow_mut();
    let mut heap_meta_lamports = heap_meta_account.lamports.borrow_mut();

    **auth_lamports = auth_lamports.
        checked_add(**heap_meta_lamports)
        .ok_or(HeapError::Overflow)?;
    **heap_meta_lamports = 0;

    for i in 0..heap_accounts.len(){
        let mut account_lamports = heap_accounts[i].lamports.borrow_mut();
        **auth_lamports = auth_lamports
                .checked_add(**account_lamports)
                .ok_or(HeapError::Overflow)?;
        **account_lamports = 0;
    }

    Ok(())
}
