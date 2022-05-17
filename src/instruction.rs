use solana_program::{
  instruction::{AccountMeta, Instruction},
  pubkey::Pubkey,
  sysvar,
  msg,
  program_error::ProgramError,
};
use borsh::{BorshDeserialize, BorshSerialize};
use std::str;

use crate::{
  error::ReeMetaError::InvalidInstruction,
  state::{Kind, UpdateType}
};

#[repr(C)]
#[derive(PartialEq, Debug, Clone)]
pub struct MetadataArgs {
  pub kind: Kind,
  pub is_modifiable: bool,
  pub update_type: UpdateType
}

// TODO: update to MetadataArgsRRA ResaleRoyaltyArt
#[repr(C)]
#[derive(PartialEq, Debug, Clone)]
pub struct MetadataArgsAAR {
  pub name: String,
  pub symbol: String,
  pub uri: String,
  pub resale_fee: u16,
}

#[repr(C)]
#[derive(PartialEq, Debug, Clone)]
pub struct CreateMetadataArgs {
  pub metadata: MetadataArgs,
  pub aar_data: MetadataArgsAAR,
}

#[repr(C)]
#[derive(PartialEq, Debug, Clone)]
pub enum ReeMetadataInstruction {
  /* Create Metadata account
   * creates the metadata account data giving ownership to program and setting details
   * #[account(0), writable, name="metadata_account", desc="PDA of the new metadata account"]
   * #[account(1), writable, name="mint", desc="Mint of the token asset"]
   * #[account(2), name="royalty_owner", desc="Original royalty holder that starts with 100% of the shares"]
   * #[account(3), signer, name="mint_authority", desc="Mint authority of the mint"]
   * #[account(4), writable & signer, name="payer", desc="Payer of the transaction"]
   * #[account(5), optional, name="update_authority", desc="if the metadata is mutable then this needs to be either the wallet of the updater or an NFT wallet" ]
   * #[account(6), name="system_program", desc="System Program"]
   * #[account(7), name="rent", "Rent info"]
   */
  CreateMetaData(CreateMetadataArgs)
}

impl ReeMetadataInstruction {
  pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
    let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
    Ok( match tag {
      0 => Self::CreateMetaData(Self::unpack_create_metadata_args(rest)?),
      _ => return Err(InvalidInstruction.into())
    })
  }

  fn unpack_create_metadata_args(data: &[u8]) -> Result<CreateMetadataArgs, ProgramError> {
    msg!("get kind");
    let (kind_u8, rest) = data.split_first().ok_or(InvalidInstruction)?;
    msg!("got {}, getting modifiable", kind_u8);
    let (modifiable, rest) = rest.split_first().ok_or(InvalidInstruction)?;
    msg!("got {}, getting update type");
    let (update_type_u8, rest) = rest.split_first().ok_or(InvalidInstruction)?;

    msg!("setting metadata");
    let mut metadata: MetadataArgs = MetadataArgs{
      kind: kind_u8.into(),
      is_modifiable: *modifiable != 0,
      update_type: update_type_u8.into(),
    };

    msg!("get name length");
    let (name_len_chunk, rest) = rest.split_at(4);
    let name_len = name_len_chunk.try_into().ok()
      .map(u32::from_le_bytes).ok_or(InvalidInstruction)? as usize;
    msg!("got {}", name_len);
    let (name_chunk, rest) = rest.split_at(name_len);
    let name = match name_chunk.try_into().ok()
      .map(String::from_utf8).ok_or(InvalidInstruction)? {
        Ok(n) => n,
        _ => return Err(InvalidInstruction.into())
      };
    msg!("got {}", name);

    msg!("get Symbol length");
    let (symbol_len_chunk, rest) = rest.split_at(4);
    let symbol_len = symbol_len_chunk.try_into().ok()
      .map(u32::from_le_bytes).ok_or(InvalidInstruction)? as usize;
    let (symbol_chunk, rest) = rest.split_at(symbol_len);
    let symbol = match symbol_chunk.try_into().ok()
      .map(String::from_utf8).ok_or(InvalidInstruction)? {
        Ok(n) => n,
        _ => return Err(InvalidInstruction.into())
      };
    msg!("got {}", symbol);
    
    let (uri_len_chunk, rest) = rest.split_at(4);
    let uri_len = uri_len_chunk.try_into().ok()
      .map(u32::from_le_bytes).ok_or(InvalidInstruction)? as usize;
    let (uri_chunk, rest) = rest.split_at(uri_len);
    let uri = match uri_chunk.try_into().ok()
      .map(String::from_utf8).ok_or(InvalidInstruction)? {
        Ok(n) => n,
        _ => return Err(InvalidInstruction.into())
      };
    msg!("got {}", uri);
    
    let resale_fee = rest.try_into().ok()
      .map(u16::from_le_bytes).ok_or(InvalidInstruction)?;

    msg!("create aar");
    let aar = MetadataArgsAAR{
      name: name,
      symbol: symbol,
      uri: uri,
      resale_fee: resale_fee,
    };

    Ok(CreateMetadataArgs{metadata: metadata, aar_data: aar})
  }
}