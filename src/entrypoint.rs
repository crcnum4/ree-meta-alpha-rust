#![cfg(all(target_arch = "bpf", not(feature = "no-entrypoint")))]

use crate::{error::ReeMetaError, processor::Processor};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError,
    pubkey::Pubkey,
    msg,
};

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("entrypoint");
    Processor::process(program_id, accounts, instruction_data)
}