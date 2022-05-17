use thiserror::Error;

use num_derive::FromPrimitive;
use solana_program::{
  decode_error::DecodeError,
  msg,
  program_error::{ProgramError, PrintProgramError}
};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum ReeMetaError {
  #[error("Invalid Instruction")]
  InvalidInstruction,
  #[error("Uninitialized")]
  Uninitialized,
  #[error("invalid Mint authority")]
  InvalidMintAuthority,
  #[error("Invalid ownership")]
  IncorrectOwner,
}

impl PrintProgramError for ReeMetaError {
  fn print<E>(&self) {
      msg!(&self.to_string());
  }
}

impl From<ReeMetaError> for ProgramError {
  fn from(e: ReeMetaError) -> Self {
      ProgramError::Custom(e as u32)
  }
}

impl<T> DecodeError<T> for ReeMetaError {
  fn type_of() -> &'static str {
      "Metadata Error"
  }
}