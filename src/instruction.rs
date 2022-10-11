use solana_program::{
  instruction::{AccountMeta, Instruction},
  pubkey::Pubkey,
  msg,
  program_error::ProgramError,
};
use borsh::{BorshSerialize};

use crate::{
  error::ReeMetaError::InvalidInstruction,
  state::{Kind, UpdateType},
  unpack::unpack_string,
};

#[repr(C)]
#[derive(PartialEq, Debug, Clone, BorshSerialize)]
pub struct MetadataArgs {
  pub kind: Kind,
  pub is_modifiable: bool,
  pub update_type: UpdateType,
  pub in_collection: bool,
}

// TODO: update to MetadataArgsRRA ResaleRoyaltyArt
#[repr(C)]
#[derive(PartialEq, Debug, Clone, BorshSerialize)]
pub struct MetadataArgsRRA {
  pub name: String,
  pub symbol: String,
  pub uri: String,
  pub resale_fee: u16,
  
}

#[repr(C)]
#[derive(PartialEq, Debug, Clone, BorshSerialize)]
pub struct CreateMetadataArgs {
  pub metadata: MetadataArgs,
  pub aar_data: MetadataArgsRRA,
}

#[repr(C)]
#[derive(PartialEq, Debug, Clone, BorshSerialize)]
pub struct AddRoyaltyArgs {
  pub share: u16,
}

#[repr(C)]
#[derive(PartialEq, Debug, Clone, BorshSerialize)]
pub struct NftTransactionArgs {
  pub amount: u64,
}

#[repr(C)]
#[derive(PartialEq, Debug, Clone, BorshSerialize)]
pub enum ReeMetadataInstruction {
  /* Create ArtNFT Metadata account
   * creates the metadata account data giving ownership to program and setting details
   * #[account(0), writable, name="metadata_account", desc="PDA of the new metadata account"]
   * #[account(1), writable, name="mint", desc="Mint of the token asset"]
   * #[account(2), read, name="royalty_owner", desc="Original royalty holder that starts with 100% of the shares"]
   * #[account(3), signer, name="created_mint_authority", desc="Mint authority of the mint"]
   * #[account(4), read, name="nft_mint_authority", desc="Pubkey of who created the mint"]
   * #[account(5), read, name="new_nft_mint_authority", desc="pubkey of who can mint the 1 nft"]
   * #[account(6), writable & signer, name="payer", desc="Payer of the transaction"]
   * #[account(7), optional, name="update_authority", desc="if the metadata is mutable then this needs to be either the wallet of the updater or an NFT wallet" ]
   * #[account(8), name="system_program", desc="System Program"]
   * #[account(9), name="rent", "Rent info"]
   * #[account(10), name="token_program", desc="Token Program"]
   * #[account(11), read & optional, name="collection", description="collection key if part of collection"]
   */
  CreateMetaData(CreateMetadataArgs),
  /* Mint one token of the given NFT and close the mint
   * #[account(0), writable, name="mint", desc="Mint of the NFT"]
   * #[account(1), signer, name="Mint_authority", desc="Mint authority and payer"] 
   * #[account(2), writable, name=recipient_ta", desc="Recipient token account"]
   * #[account(4), name="token_program", desc="token program"]
   */
  MintNFT(),
  /* Lock metadata
   * lock the metadata from any further changes other then Initial Sale
   * #[account(0), writable, name='metadata', desc="PDA of the NFT metadata"]
   * #[account(1), signer & writable, name="payer", desc="transaction payer"]
   * #[account(2), signer, name="update_authority", desc-"update authority of the NFT"]
   */
  LockNFT(),
  /* Add Royalty to ArtNFT
   * can add a Royalty to the ArtNFT Royalty list. Will take Share from the 
   * Royalty in position 0, NFT must me modifiable.
   * #[account(0), writable, name="metadata", desc="PDA of the NFT metadata"]
   * #[account(1), signer & writable, name="payer", desc="Transaction & Rent Payer" ]
   * #[account(2), signer, name="update_authority", desc="update authority of the NFT"]
   * #[account(3), read, name="new_royalty", desc="pubkey of royalty to add"]
   * #[account(4), read, name="system_program"]
   * #[account(5), read, name="rent_program"]
   */
  AddRoyalty(AddRoyaltyArgs),
  /* Perform an NFT payout
   * Process a Nft payment transfer.
   * If the kind of nft contains a royalty system apply the royalty system
   * -> if initial sale is false then ignore the target and 
   *      apply the full transfer to the royalty system. change the initial sale to true.
   * -> if initial sale is true the take the resale_fee out of the transfer
   *       give the remainder to the target address
   *       the resale fee goes through the royalty system
   * If no royalty system exists on the nft simply transfer the funds to the target.
   * #[account(0), writable, name="metadata", desc="PDA of the NFT metadata"]
   * #[account(1), signer & writable, name="payer", desc="Transaction payer and NFT buyer"]
   * #[account(2), writable, name="target", "seller of the NFT and possble recipient of funds"]
   * #[acconut(3), read, name="system_program"]
   * #[account(4-x), optional & writable, name="royalty accounts", "Inclued if needed"]
   */
  NftTransaction(NftTransactionArgs),

}

impl ReeMetadataInstruction {
  pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
    let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
    Ok( match tag {
      0 => Self::CreateMetaData(Self::unpack_create_metadata_args(rest)?),
      1 => Self::MintNFT(),
      2 => Self::LockNFT(),
      3 => Self::AddRoyalty(Self::unpack_add_royalty_args(rest)?),
      4 => Self::NftTransaction(Self::unpack_nft_transaction_args(rest)?),
      _ => return Err(InvalidInstruction.into())
    })
  }

  fn unpack_create_metadata_args(data: &[u8]) -> Result<CreateMetadataArgs, ProgramError> {
    msg!("get kind");
    let (kind_u8, rest) = data.split_first().ok_or(InvalidInstruction)?;
    let (modifiable, rest) = rest.split_first().ok_or(InvalidInstruction)?;
    let (update_type_u8, rest) = rest.split_first().ok_or(InvalidInstruction)?;
    let (in_collection, rest) = rest.split_first().ok_or(InvalidInstruction)?;

    msg!("setting metadata");
    let metadata: MetadataArgs = MetadataArgs{
      kind: kind_u8.into(),
      is_modifiable: *modifiable != 0,
      update_type: update_type_u8.into(),
      in_collection: *in_collection != 0,
    };
    msg!("get Name");
    let (name, rest) = unpack_string(rest).ok_or(InvalidInstruction)?;
    msg!("get symbol");
    let (symbol, rest) = unpack_string(rest).ok_or(InvalidInstruction)?;
    //get uri
    msg!("get uri");
    let (uri, rest) = unpack_string(rest).ok_or(InvalidInstruction)?;
    msg!("get resale");
    let (resale_u16, _rest) = rest.split_at(2); 
    let resale_fee = resale_u16.try_into().ok()
      .map(u16::from_le_bytes).ok_or(InvalidInstruction)?;
    
    msg!("create aar");
    let aar = MetadataArgsRRA{
      name: name,
      symbol: symbol,
      uri: uri,
      resale_fee: resale_fee,
    };

    Ok(CreateMetadataArgs{metadata: metadata, aar_data: aar})
  }

  fn unpack_add_royalty_args(data: &[u8]) -> Result<AddRoyaltyArgs, ProgramError> {
    let share = data.try_into().ok()
      .map(u16::from_le_bytes).ok_or(InvalidInstruction)?;
    Ok(AddRoyaltyArgs{share})
  }

  fn unpack_nft_transaction_args(data: &[u8]) -> Result<NftTransactionArgs, ProgramError> {
    let amount: u64 = data.try_into().ok()
      .map(u64::from_le_bytes).ok_or(InvalidInstruction)?;
    
    Ok(NftTransactionArgs{amount})
  }
}

pub fn mint_nft(
  program_id: &Pubkey,
  token_program: &Pubkey,
  metadata_pda: &Pubkey,
  mint_authority: &Pubkey,
  recipient: &Pubkey,
) -> Instruction {
  Instruction { 
    program_id: *program_id, 
    accounts: vec![
      AccountMeta::new(*metadata_pda, false),
      AccountMeta::new_readonly(*mint_authority, true),
      AccountMeta::new(*recipient, false),
      AccountMeta::new(*token_program, false)
    ], 
    data: ReeMetadataInstruction::MintNFT().try_to_vec().unwrap() 
  }
}

pub fn nft_funding_sol(
  program_id: &Pubkey,
  metadata_pda: &Pubkey,
  payer: &Pubkey,
  target: &Pubkey,
  royalties: Vec<Pubkey>,
  data: NftTransactionArgs,
) -> Instruction {
  let mut accounts = vec![
    AccountMeta::new_readonly(*metadata_pda, false),
    AccountMeta::new(*payer, true),
    AccountMeta::new(*target, false),
    AccountMeta::new_readonly(solana_program::system_program::id(), false)
  ];
  
  for account in royalties.iter() {
    accounts.push(
      AccountMeta::new(*account, false)
    );
  }
  
  Instruction {
    program_id: *program_id,
    accounts,
    data: ReeMetadataInstruction::NftTransaction((data)).try_to_vec().unwrap()
  }
}