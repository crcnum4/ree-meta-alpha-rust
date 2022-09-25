use solana_program::{
  entrypoint::ProgramResult, 
  program_error::ProgramError, 
  pubkey::Pubkey
};

pub mod processor;
pub mod error;
pub mod instruction;
pub mod state;
pub mod utils;
pub mod artNft;
pub mod customNft;
pub mod unpack;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("Epf86va8B3wDCJ2t47nV6EVT9fVipnnTbLihtLZiL7am");

pub fn check_program_account(ree_meta_program_id: &Pubkey) -> ProgramResult {
  if ree_meta_program_id != &id() {
    return Err(ProgramError::IncorrectProgramId);
  }
  Ok(())
}