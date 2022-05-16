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