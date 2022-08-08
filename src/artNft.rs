use crate::{
  error::ReeMetaError,
  state::{
    Metadata,
    ArtNft, Royalty,
  }, instruction::AddRoyaltyArgs,
};
use borsh::BorshSerialize;

use solana_program::{
  account_info::{AccountInfo, next_account_info},
  entrypoint::ProgramResult,
  pubkey::Pubkey, system_program,
  sysvar::{rent::{Rent, ID as RENT_ID}, Sysvar}, program::invoke, system_instruction,
};

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
  program_id: &Pubkey,
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

  let mut artNft = metadata.data.clone();

  if artNft.royalties.len() == 0 || artNft.royalties[0].share < data.share {
    return Err(ReeMetaError::InsufficientShare.into())
  }

  // validations complete
  let new_royalty = Royalty{
    address: *new_royalty_account_info.key,
    share: data.share,
    verified: true,
  };

  artNft.royalties[0].share = artNft.royalties[0].share - data.share;
  artNft.royalties.push(new_royalty);

  metadata.data = artNft;

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

  metadata_account_info.realloc(metadata.clone().size(), false);

  metadata.serialize(&mut *metadata_account_info.data.borrow_mut())?;

  Ok(())
}