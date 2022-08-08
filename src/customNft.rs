use crate::{
  error::ReeMetaError,
  state::{
    Metadata,
    CustomNft,
  },
};
use borsh::BorshSerialize;

use solana_program::{
  account_info::AccountInfo,
  entrypoint::ProgramResult,
};

pub fn lock_nft(
  mut metadata: Metadata<CustomNft>,
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