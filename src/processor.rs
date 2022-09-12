use crate::{
  error::ReeMetaError,
  instruction::{
    MetadataArgs,
    MetadataArgsRRA,
    ReeMetadataInstruction,
    AddRoyaltyArgs, NftTransactionArgs,
  },
  state::{
    Metadata,
    ArtNft,
    CustomNft,
    Royalty,
    PREFIX,
    Kind,
    UpdateType
  },
  utils::{
    assert_initialized,
    assert_valid_mint_authority,
    assert_owned_by,
  },
  artNft,
  customNft
};
use borsh::BorshSerialize;

use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  msg,
  program::{invoke, invoke_signed},
  pubkey::Pubkey,
  system_instruction,
  sysvar::{rent::{Rent, ID as RENT_ID}, Sysvar},
  system_program, 
};

use spl_token::{
  state::{Mint, Account as TokenAccount}
};

pub struct Processor;
impl Processor {
  pub fn process (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
  ) -> ProgramResult {
    msg!("REEMETA Instruction");
    let instruction = ReeMetadataInstruction::unpack(input)?;
    match instruction {
      ReeMetadataInstruction::CreateMetaData(args) => {
        msg!("Create Metadata Account");
        process_create_metadata(
          program_id,
          accounts,
          args.metadata,
          args.aar_data,
        )
      },
      ReeMetadataInstruction::MintNFT() => {
        process_mint_nft(program_id, accounts)
      },
      ReeMetadataInstruction::LockNFT() => {
        process_lock_nft(program_id, accounts)
      },
      ReeMetadataInstruction::AddRoyalty(args) => {
        process_add_royalty(program_id, accounts, args)
      },
      ReeMetadataInstruction::NftTransaction(args) => {
        process_nft_transaction(program_id, accounts, args)
      }
    }
  }
}

pub fn process_create_metadata (
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  metadata_data: MetadataArgs,
  aar_data: MetadataArgsRRA,
) -> ProgramResult {
  msg!("Get accounts");
  let account_iter = &mut accounts.iter();
  let metadata_acount_info = next_account_info(account_iter)?;
  let mint_account_info = next_account_info(account_iter)?;
  let royalty_owner_account_info = next_account_info(account_iter)?;
  let mint_authority_account_info = next_account_info(account_iter)?;
  let new_mint_authority_account_info = next_account_info(account_iter)?;
  let payer_account_info = next_account_info(account_iter)?;
  let update_authority: Option<&AccountInfo> = match metadata_data.update_type {
    UpdateType::None => {
      None
    },
    _ => {
      Some(next_account_info(account_iter)?)
    }
  };
  let system_info = next_account_info(account_iter)?;
  let rent_info = next_account_info(account_iter)?;
  let token_info = next_account_info(account_iter)?;

  msg!("verify system accounts");
  if *system_info.key != system_program::ID
    || *rent_info.key != RENT_ID
    || *token_info.key != spl_token::ID
  {
    msg!("invalid system accounts");
    return Err(ReeMetaError::InvalidInstruction.into())
  }

  msg!("assert mint is a token program mint");
  assert_owned_by(mint_account_info, token_info.key)?;

  let genesis_royalty = Royalty{
    address: *royalty_owner_account_info.key,
    share: 100, 
    verified: true
  };

  let mut royalties: Vec<Royalty> = Vec::<Royalty>::new();
  royalties.push(genesis_royalty);
    
  let art_nft: ArtNft = ArtNft{
    name: aar_data.name,
    symbol: aar_data.symbol,
    uri: aar_data.uri,
    resale_fee: aar_data.resale_fee,
    initial_sale: false,
    collection: None,
    royalties: royalties,
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

  if metadata_key != *metadata_acount_info.key {
    msg!("Invalid PDA");
    return Err(ReeMetaError::InvalidInstruction.into())
  }

  msg!("build Metadata");

  let mut metadata: Metadata<ArtNft> = Metadata{
    kind: metadata_data.kind,
    mint: *mint_account_info.key,
    data: art_nft,
    is_modifiable: metadata_data.is_modifiable,
    update_type: metadata_data.update_type,
    collection: None,
    update_authority: match update_authority {
      None => None,
      Some(account_info) => Some(*account_info.key),
    },
  };

  if metadata_data.in_collection {
    let collection_account_info = next_account_info(account_iter)?;
    metadata.collection = Some(*collection_account_info.key);
  }

  msg!("Rent");
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
        system_info.clone(),
      ]
    )?;
  }

  let accounts = &[
    metadata_acount_info.clone(),
    system_info.clone(),
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

  if *mint_authority_account_info.key != *new_mint_authority_account_info.key {
    invoke(
      &spl_token::instruction::set_authority(
        token_info.key, 
        mint_account_info.key, 
        Some(new_mint_authority_account_info.key), 
        spl_token::instruction::AuthorityType::MintTokens, 
        mint_authority_account_info.key, 
        &[mint_authority_account_info.key]
      )?, 
      &[
        mint_account_info.clone(),
        mint_authority_account_info.clone(),
      ]
    )?;
  }

  msg!("write data to account");
  metadata.serialize(&mut *metadata_acount_info.data.borrow_mut())?;

  Ok(())
}

pub fn process_mint_nft(
  _program_id: &Pubkey,
  accounts: &[AccountInfo],
) -> ProgramResult {
  let account_iter = &mut accounts.iter();
  let mint_account_info = next_account_info(account_iter)?;
  let authority_account_info = next_account_info(account_iter)?;
  let recipient_token_account_info = next_account_info(account_iter)?;
  let token_program_info = next_account_info(account_iter)?;

  let mint: Mint = assert_initialized(mint_account_info)?;

  let recipient_token_account: TokenAccount = assert_initialized(recipient_token_account_info)?;

  assert_valid_mint_authority(&mint.mint_authority, &authority_account_info)?;

  assert_owned_by(mint_account_info, &spl_token::id())?;
  assert_owned_by(recipient_token_account_info, &spl_token::id())?;

  if recipient_token_account.mint != *mint_account_info.key {
    return Err(ReeMetaError::InvalidInstruction.into())
  }

  // mint the token then remove the mint authority from the mint.
  invoke(
    &spl_token::instruction::mint_to(
      token_program_info.key, 
      mint_account_info.key, 
      recipient_token_account_info.key, 
      authority_account_info.key, 
      &[authority_account_info.key], 
      1
    )?,
    &[
      mint_account_info.clone(),
      recipient_token_account_info.clone(),
      authority_account_info.clone(),
    ]
  )?;

  invoke(
    &spl_token::instruction::set_authority(
      token_program_info.key, 
      mint_account_info.key, 
      None, 
      spl_token::instruction::AuthorityType::MintTokens, 
      authority_account_info.key, 
      &[authority_account_info.key]
    )?,
    &[
      mint_account_info.clone(),
      authority_account_info.clone(),
    ]
  )?;

  Ok(())
}

pub fn process_lock_nft (
  program_id: &Pubkey,
  accounts: &[AccountInfo],
) -> ProgramResult {
  let account_iter = &mut accounts.iter();
  let metadata_account_info = next_account_info(account_iter)?;
  let _payer_account_info = next_account_info(account_iter)?;
  let update_authority_account_info = next_account_info(account_iter)?;

  assert_owned_by(metadata_account_info, program_id)?;

  let kind = Metadata::<ArtNft>::get_kind(metadata_account_info)?;

  match kind {
    Kind::RoyaltyArt => {
      artNft::lock_nft(
        Metadata::<ArtNft>::from_account_info(metadata_account_info)?, 
        metadata_account_info, 
        update_authority_account_info
      )
    },
    Kind::Uninitialized => {
      customNft::lock_nft(
        Metadata::<CustomNft>::from_account_info(metadata_account_info)?,
        metadata_account_info, 
        update_authority_account_info
      )
    } 
  }
}

pub fn process_add_royalty (
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  data: AddRoyaltyArgs,
) -> ProgramResult {
  let account_iter = &mut accounts.iter();
  let metadata_account_info = next_account_info(account_iter)?;

  assert_owned_by(metadata_account_info, program_id)?;

  match Metadata::<ArtNft>::get_kind(metadata_account_info)? {
    Kind::RoyaltyArt => artNft::add_royalty(
      accounts, 
      Metadata::<ArtNft>::from_account_info(metadata_account_info)?, 
      data
    ),
    Kind::Uninitialized => {
      msg!("This NFT Kind has no royalties");
      return Err(ReeMetaError::InvalidNFTKind.into())
    }
  }

}

// TODO: change to tokens
pub fn process_nft_transaction (
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  data: NftTransactionArgs
) -> ProgramResult {
  let account_iter = &mut accounts.iter();
  let metadata_account_info = next_account_info(account_iter)?;

  assert_owned_by(metadata_account_info, program_id)?;

  let kind = Metadata::<ArtNft>::get_kind(metadata_account_info)?;

  match kind {
    Kind::RoyaltyArt => {
      artNft::nft_transaction(
        accounts, 
        Metadata::<ArtNft>::from_account_info(metadata_account_info)?, 
        data
      )
    },
    _ => {
      // non Royalty transaction transfer full amount to target
      let payer_account_info = next_account_info(account_iter)?;
      let target_account_info = next_account_info(account_iter)?;
      let system_info = next_account_info(account_iter)?;

      if *system_info.key != system_program::ID {
        msg!("Invalid system account");
        return Err(ReeMetaError::InvalidInstruction.into())
      }

      invoke(
        &system_instruction::transfer(
          payer_account_info.key, 
          target_account_info.key,
          data.amount
        ),
        &[
          payer_account_info.clone(),
          target_account_info.clone(),
          system_info.clone()
        ]
      )?;

      Ok(())
    }
  }

}