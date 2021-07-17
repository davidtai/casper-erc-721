use alloc::{
    string::String,
};

use contract::{
    contract_api::{runtime},
    unwrap_or_revert::UnwrapOrRevert,
};

use types::{
    account::AccountHash,
    bytesrepr::{Bytes, ToBytes},
    CLTyped, U256, CLValue  
};

use super::mappings::*;


pub fn _base_uri() -> String {
    String::from("")
}

pub fn _approve(to: AccountHash, token_id: U256) {
    set_key(&token_approval_key(token_id), to);
}

pub fn _set_approval_for_all(owner:AccountHash, operator:AccountHash , approve:bool) {
    set_key(&operator_approvals_key(owner, operator), approve);
}

pub fn _is_approved_for_all(owner: AccountHash, operator: AccountHash) -> bool {
    let val:bool = get_key(&operator_approvals_key(owner, operator));
    val
}

pub fn _exists(token_id: U256) -> bool{
    let zero_addr:AccountHash = AccountHash::from_formatted_str("account-hash-0000000000000000000000000000000000000000000000000000000000000000").unwrap_or_default();
    let owner:AccountHash = get_key(&owner_key(token_id));

    owner != zero_addr
}

// Returns whether `spender` is allowed to manage `tokenId`.
pub fn _is_approved_or_owner(spender: AccountHash, token_id:U256) -> bool {

    // if tokenId doesnot exist, it will return an all-zero address for owner, and won't match any of the below conditions
    let owner:AccountHash = get_key(&owner_key(token_id));
    
    let is_approved_for_all:bool = get_key(&operator_approvals_key(owner, spender));
    let approved_account: AccountHash = get_key(&token_approval_key(token_id));
    
    (owner == spender) || 
    (is_approved_for_all == true) || 
    (approved_account == spender)
}

pub fn _transfer(from: AccountHash, to: AccountHash, token_id: U256) {

    let owner:AccountHash = get_key(&owner_key(token_id));

    // Zero Address is specific to etherium blockchain only. Do we need to handle this on casper blockchain????
    let zero_addr:AccountHash = AccountHash::from_formatted_str("account-hash-0000000000000000000000000000000000000000000000000000000000000000").unwrap_or_default();

    if ! (owner == from) {     // transfer of token that is not own - Returning
        return
    }

    if to == zero_addr {         // transfer to the zero address - Returing
        return
    }
    
    _before_token_transfer(from, to, token_id);
    _approve(zero_addr, token_id);                // Clear approvals from the previous owner

    let mut amount:U256 = get_key(&balance_key(&from));
    set_key(&balance_key(&from), amount-1);

    amount = get_key(&balance_key(&to));
    set_key(&balance_key(&to), amount+1);

    set_key(&owner_key(token_id), to);
}

pub fn _safe_transfer(from: AccountHash, to: AccountHash, token_id: U256, _data:Bytes)
{
    _transfer(from, to, token_id);

    if !_check_on_erc721_received(from, to, token_id, _data) {
        // "ERC721: transfer to non ERC721Receiver implementer"
    }
}

pub fn _safe_mint(to: AccountHash, token_id: U256, data: Option<Bytes>)
{
    _mint(to, token_id);
    match data{
        Some(val) => 
        {
            let zero_addr:AccountHash = AccountHash::from_formatted_str("account-hash-0000000000000000000000000000000000000000000000000000000000000000").unwrap_or_default();
            if ! _check_on_erc721_received(zero_addr, to, token_id, val) {
                // "ERC721: transfer to non ERC721Receiver implementer"
            }
        },
        None => {
            return
        }   
    }
}


// Mints `tokenId` and transfers it to `to`.
pub fn _mint(to:AccountHash, token_id: U256){
    let zero_addr:AccountHash = AccountHash::from_formatted_str("account-hash-0000000000000000000000000000000000000000000000000000000000000000").unwrap_or_default();

    if to == zero_addr {         // mint to 0 address
        return
    }

    if _exists(token_id) {       // token already minted
        return
    }

    _before_token_transfer(zero_addr, to, token_id);

    let amount:U256 = get_key(&balance_key(&to));
    set_key(&balance_key(&to), amount+1);
    set_key(&owner_key(token_id), to);
}

// Destroys `tokenId`.
pub fn _burn(token_id: U256, owner:AccountHash,  zero_addr:AccountHash){
    //let owner:AccountHash = get_key(&owner_key(token_id));
    //let zero_addr:AccountHash = AccountHash::from_formatted_str("account-hash-0000000000000000000000000000000000000000000000000000000000000000").unwrap_or_default();

    _before_token_transfer(owner, zero_addr, token_id);
    _approve(zero_addr, token_id);                      // Clear approvals

    let amount:U256 = get_key(&balance_key(&owner));
    set_key(&balance_key(&owner), amount - 1);          // decrements owners balance
    
    //delete _owners[tokenId];
    set_key(&owner_key(token_id), zero_addr);           // set owner key to null(zero) address
}


#[allow(unused)]
pub fn _before_token_transfer(from: AccountHash, to: AccountHash, token_id: U256) {}

#[allow(unused)]
pub fn _check_on_erc721_received(from: AccountHash, to: AccountHash, token_id: U256, data: Bytes) -> bool
{
    true
}

pub fn _encode_packed(base_uri: String, token_id: U256) ->String 
{
    let bytes_base_uri = base_uri.as_bytes();
    let token:String = format!("{:x}", token_id);
    let mut result:String = String::from("");

    result.push_str(&hex::encode(bytes_base_uri));
    result.push_str(&token);
    result.insert_str(0, "0x");
    result
}


pub fn ret<T: CLTyped + ToBytes>(value: T) {
    runtime::ret(CLValue::from_t(value).unwrap_or_revert())
}