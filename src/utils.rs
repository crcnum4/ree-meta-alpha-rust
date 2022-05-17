use crate::{
  error::ReeMetaError,
};
use solana_program::{
  account_info::{AccountInfo},
  entrypoint::ProgramResult,
  program_error::ProgramError,
  program_option::COption,
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
};


pub fn assert_initialized<T: Pack + IsInitialized> (
  account_info: &AccountInfo
) -> Result<T, ProgramError> {
  let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
  if !account.is_initialized() {
      Err(ReeMetaError::Uninitialized.into())
  } else {
      Ok(account)
  }
}

pub fn assert_valid_mint_authority(
  mint_authority: &COption<Pubkey>,
  mint_authority_info: &AccountInfo
) -> ProgramResult {
  match mint_authority {
      COption::None => {
          return Err(ReeMetaError::InvalidMintAuthority.into())
      }
      COption::Some(key) => {
          if mint_authority_info.key != key {
              return Err(ReeMetaError::InvalidMintAuthority.into())
          } else {
              return Ok(())
          }
      }
  }
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
  if account.owner != owner {
      Err(ReeMetaError::IncorrectOwner.into())
  } else {
      Ok(())
  }
}