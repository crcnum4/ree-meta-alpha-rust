use crate::{
  error::ReeMetaError,
  state::{
    Metadata,
    ArtNft, Royalty,
  }, instruction::{AddRoyaltyArgs, NftTransactionArgs},
};
use borsh::BorshSerialize;

use solana_program::{
  account_info::{AccountInfo, next_account_info},
  entrypoint::ProgramResult,
  system_program,
  sysvar::{rent::{Rent, ID as RENT_ID}, Sysvar}, 
  program::invoke, 
  system_instruction, 
  msg,
};

use percentage::Percentage;

pub fn lock_nft(
  mut metadata: Metadata<ArtNft>,
  metadata_account_info: &AccountInfo,
  update_authority_account_info: &AccountInfo,
) -> ProgramResult {
  if !metadata.is_modifiable {
    return Err(ReeMetaError::AlreadyLocked.into())
  }

  if metadata.update_authority == Option::None {
    return Err(ReeMetaError::NoUpdateAuthority.into())
  }

  if !update_authority_account_info.is_signer || metadata.update_authority != Some(*update_authority_account_info.key) {
    return Err(ReeMetaError::InvalidUpdateAuthority.into())
  }

  // validated data lock the NFT
  metadata.is_modifiable = false;

  metadata.serialize(&mut *metadata_account_info.data.borrow_mut())?;
  Ok(())
}

pub fn add_royalty(
  accounts: &[AccountInfo],
  mut metadata: Metadata<ArtNft>,
  data: AddRoyaltyArgs,
) -> ProgramResult {
  let account_iter = &mut accounts.iter();
  let metadata_account_info = next_account_info(account_iter)?;
  let payer_account_info = next_account_info(account_iter)?;
  let update_authority_account_info = next_account_info(account_iter)?;
  let new_royalty_account_info = next_account_info(account_iter)?;
  let system_info = next_account_info(account_iter)?;
  let rent_info = next_account_info(account_iter)?;

  if *system_info.key != system_program::ID || *rent_info.key != RENT_ID {
    return Err(ReeMetaError::InvalidInstruction.into())
  }
  
  if !metadata.is_modifiable {
    return Err(ReeMetaError::AlreadyLocked.into())
  }

  if metadata.update_authority == Option::None {
    return Err(ReeMetaError::NoUpdateAuthority.into())
  }

  if !update_authority_account_info.is_signer || metadata.update_authority != Some(*update_authority_account_info.key) {
    return Err(ReeMetaError::InvalidUpdateAuthority.into())
  }

  let mut art_nft = metadata.data.clone();

  if art_nft.royalties.len() == 0 || art_nft.royalties[0].share < data.share {
    return Err(ReeMetaError::InsufficientShare.into())
  }

  // validations complete
  let new_royalty = Royalty{
    address: *new_royalty_account_info.key,
    share: data.share,
    verified: true,
  };

  art_nft.royalties[0].share = art_nft.royalties[0].share - data.share;
  art_nft.royalties.push(new_royalty);

  metadata.data = art_nft;

  // calculate new rent
  let rent = &Rent::from_account_info(rent_info)?;

  let required_lamports = rent
    .minimum_balance(metadata.clone().size())
    .max(1)
    .saturating_sub(metadata_account_info.lamports());

  if required_lamports > 0 {
    invoke(
      &system_instruction::transfer(
        payer_account_info.key, 
        metadata_account_info.key,
        required_lamports
      ),
      &[
        payer_account_info.clone(),
        metadata_account_info.clone(),
        system_info.clone(),
      ]
    )?;
  }

  metadata_account_info.realloc(metadata.clone().size(), false)?;

  metadata.serialize(&mut *metadata_account_info.data.borrow_mut())?;

  Ok(())
}

pub fn nft_transaction (
  // program_id: &Pubkey,
  accounts: &[AccountInfo],
  mut metadata: Metadata<ArtNft>,
  data: NftTransactionArgs,
) -> ProgramResult {
  let account_iter = &mut accounts.iter();
  let metadata_account_info = next_account_info(account_iter)?;
  let payer_account_info = next_account_info(account_iter)?;
  let target_account_info = next_account_info(account_iter)?;
  let system_info = next_account_info(account_iter)?;

  if *system_info.key != system_program::ID {
    msg!("Invalid system account");
    return Err(ReeMetaError::InvalidInstruction.into())
  }

  // check the initial sale 
  // non mut vars can be set once then unchanged.
  let royalty_payout: u64;
  let mut target_payout: u64 = 0;

  // TODO: add to the instruction a royalty flag so non initial sales can go full royalty;
  if !metadata.data.initial_sale {
    msg!("initial sale detected");
    // initial sale has not been done yet all goes to royalties
    royalty_payout = data.amount;
  } else {
    msg!("not an initial sale");
    // initial sale occured this is a secondary market transaction
    let percentage_rate = Percentage::from(metadata.data.resale_fee);
    royalty_payout = percentage_rate.apply_to(data.amount);
    target_payout = data.amount - royalty_payout;
  }
  msg!("amount was: {}", data.amount);
  msg!("royalty_payout is: {}", royalty_payout);

  // for v1 the remaining accounts much be in the same order for the accounts
  let mut current_payout = 0;
  for (i, royalty) in metadata.data.royalties.iter().enumerate() {
    let royalty_account_info = next_account_info(account_iter)?;
    if royalty.address != *royalty_account_info.key {
      return Err(ReeMetaError::RoyaltyAddressInvalid.into())
    }
    let mut amount: u64 = 0;
    if i == metadata.data.royalties.len() - 1 {
      // make sure the last royalty gets the remaining payout to ensure full transfer of funds
      amount = royalty_payout - current_payout;
    } else {
      let percentage = Percentage::from(royalty.share);
      amount = percentage.apply_to(royalty_payout);
      current_payout += amount;
    }

    msg!("royalty {} getting {} percentage totaling {} lamports", royalty_account_info.key.to_string(), royalty.share, amount);

    // pay amount to this user
    invoke(
      &system_instruction::transfer(
        payer_account_info.key, 
        royalty_account_info.key, 
        amount
      ), 
      &[
        payer_account_info.clone(),
        royalty_account_info.clone(),
        system_info.clone()
      ]
    )?;
  }

  // recheck initial sale. 
  msg!("update nft metatdata initial sale if needed");

  if !metadata.data.initial_sale {
    // initial sale all went to royalty. change initial sale to true
    metadata.data.initial_sale = true;
    metadata.serialize(&mut *metadata_account_info.data.borrow_mut())?;
    return Ok(())
  }

  msg!("payout to target {}", target_payout);
  if target_payout > 0 {
    invoke(
      &system_instruction::transfer(
        payer_account_info.key, 
        target_account_info.key,
        target_payout
      ),
      &[
        payer_account_info.clone(),
        target_account_info.clone(),
        system_info.clone()
      ]
    )?;
  }

  Ok(())
}