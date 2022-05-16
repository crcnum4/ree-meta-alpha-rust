use crate::{
  error::ReeMetaError,
  instruction::{
    CreateMetadataArgs,
    MetadataArgs,
    MetadataArgsAAR,
    ReeMetadataInstruction
  },
  state::{
    Metadata,
    ArtNft,
    Royalty,
    PREFIX,
    Kind,
    UpdateType
  },
  utils::{
    assert_initialized
  }
};
use borsh::BorshSerialize;

use solana_program::{
  account_info::{next_account_info, AccountInfo},
  instruction::{AccountMeta},
  entrypoint::ProgramResult,
  msg,
  program::{invoke, invoke_signed},
  pubkey::Pubkey,
  system_instruction,
  sysvar::{rent::Rent, Sysvar},
  clock
};

use spl_token::{
  state::{Mint, Account as TokenAccount}
};
use percentage::Percentage;

pub struct Processor;
impl Processor {
  pub fn process (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
  ) -> ProgramResult {
    msg!("get instruction");
    let instruction = ReeMetadataInstruction::unpack(input)?;
    match instruction {
      ReeMetadataInstruction::CreateMetaData(args) => {
        msg!("Create Metadata Account");
        process_create_metadata(
          program_id,
          accounts,
          args.metadata,
          args.arr_data,
        )
      }
    }
  }
}

pub fn process_create_metadata (
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  metadata_data: MetadataArgs,
  aar_data: MetadataArgsAAR,
) -> ProgramResult {
  let account_iter = &mut accounts.iter();
  let metadata_acount_info = next_account_info(account_iter)?;
  let mint_account_info = next_account_info(account_iter)?;
  let royalty_owner_account_info = next_account_info(account_iter)?;
  let mint_authority_account_info = next_account_info(account_iter)?;
  let payer_account_info = next_account_info(account_iter)?;
  let update_authority: Option<&AccountInfo> = match metadata_data.update_type {
    UpdateType::None => {
      None
    },
    _ => {
      Some(next_account_info(account_iter)?)
    }
  };
  let system_account_info = next_account_info(account_iter)?;
  let rent_info = next_account_info(account_iter)?;

  let genesis_royalty = Royalty{
    address: *royalty_owner_account_info.key, 
    share: 100, 
    verified: true
  };
    
  let mut artNft: ArtNft = ArtNft{
    name: aar_data.name,
    symbol: aar_data.symbol,
    uri: aar_data.uri,
    resale_fee: aar_data.resale_fee,
    royalties: Some(vec![genesis_royalty]),
  };

  let metadata_seeds = &[
    PREFIX.as_bytes(),
    program_id.as_ref(),
    mint_account_info.key.as_ref(),
  ];

  let (metadata_key, metadata_bump_seed) = Pubkey::find_program_address(metadata_seeds, program_id);

  let metadata_authority_seeds = &[
    PREFIX.as_bytes(),
    program_id.as_ref(),
    mint_account_info.key.as_ref(),
    &[metadata_bump_seed]
  ];

  let metadata: Metadata<ArtNft> = Metadata{
    kind: metadata_data.kind,
    mint: *mint_account_info.key,
    data: artNft,
    is_modifiable: metadata_data.is_modifiable,
    update_type: metadata_data.update_type,
    update_authority: match update_authority {
      None => None,
      Some(account_info) => Some(*account_info.key),
    }
  };

  msg!("Metadata size is: {}", metadata.size());

  let rent = &Rent::from_account_info(rent_info)?;

  let required_lamports = rent
    .minimum_balance(metadata.clone().size())
    .max(1)
    .saturating_sub(metadata_acount_info.lamports());

  if required_lamports > 0 {
    msg!("Tranfer {} lamports", required_lamports);
    invoke(
      &system_instruction::transfer(
        payer_account_info.key, 
        metadata_acount_info.key, 
        required_lamports
      ),
      &[
        payer_account_info.clone(),
        metadata_acount_info.clone(),
        system_account_info.clone(),
      ]
    )?;
  }

  let accounts = &[
    metadata_acount_info.clone(),
    system_account_info.clone(),
  ];

  msg!("allocate and assign");
  invoke_signed(
    &system_instruction::allocate(
      metadata_acount_info.key, 
      metadata.clone().size().try_into().unwrap()
    ), 
    accounts, 
    &[metadata_authority_seeds]
  )?;
  invoke_signed(
    &system_instruction::assign(metadata_acount_info.key, program_id),
    accounts, 
    &[metadata_authority_seeds]
  )?;

  msg!("write data to account");
  metadata.serialize(&mut *metadata_acount_info.data.borrow_mut())?;

  Ok(())
}