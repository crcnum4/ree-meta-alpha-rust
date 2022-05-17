use solana_program::{
  borsh::try_from_slice_unchecked,
  account_info::AccountInfo,
  program_error::ProgramError,
  pubkey::Pubkey,
  msg
};
use borsh::{BorshDeserialize, BorshSerialize};

pub const PREFIX: &str = "ree-metadata";

pub trait MetadataData {
  fn size(&self) -> usize;
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Royalty {
  pub address: Pubkey, // 32
  pub share: u16, // 2
  pub verified: bool // 1
}

impl Royalty {
  pub fn size() -> usize {35}
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ArtNft {
  pub name: String, //4 + len
  pub symbol: String, //4 + len
  pub uri: String, //4 + len
  pub resale_fee: u16, // 2
  // TODO: Add initial_sale boolean;
  pub initial_sale: bool,
  pub royalties: Option<Vec<Royalty>>, // 1 + 4 + (Royaty * len)
  // TODO: add Collection pubkey;
}

impl MetadataData for ArtNft {
  fn size(&self) -> usize {
    let size = 4 // name string size buffer
    + self.name.len()
    + 4 // symbol string size buffer
    + self.symbol.len()
    + 4 // uri string size buffer
    + self.uri.len()
    + 2 // resale fee
    + 1 // initial sale boolean
    + 1; // Royalty Option buffer

    return match &self.royalties {
      None => {
        size
      },
      Some(royalties) => {
        size + 4 + royalties.len() * Royalty::size()
      }
    }
  }
}

impl ArtNft {
  pub fn from_account_data(data: &[u8]) -> Result<ArtNft, ProgramError> {
    let an: ArtNft = try_from_slice_unchecked(data)?;
    Ok(an)
  }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum Kind {
  Uninitialized,
  RoyaltiyResaleArt,
}

impl From<&u8> for Kind {
  fn from(orig: &u8) -> Self {
    match orig {
      1 => Kind::RoyaltiyResaleArt,
      _ => Kind::Uninitialized,
    }
  }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum UpdateType {
  None,
  WalletSigner,
  NftToken,
}

impl From<&u8> for UpdateType {
  fn from(orig: &u8) -> Self {
    match orig {
      1 => UpdateType::WalletSigner,
      2 => UpdateType::NftToken,
      _ => UpdateType::None,
    }
  }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Metadata<
  T: MetadataData + BorshDeserialize + BorshSerialize + PartialEq + Clone
>{
  pub kind: Kind,
  pub mint: Pubkey,
  pub is_modifiable: bool,
  pub update_type: UpdateType,
  pub update_authority: Option<Pubkey>,
  pub data: T,
}

impl<T> Metadata<T>
where
  T: MetadataData + BorshDeserialize + BorshSerialize + PartialEq + Clone
{
  pub fn from_account_info(account_info: &AccountInfo) -> Result<Metadata<T>, ProgramError> {
    let md: Metadata<T> = try_from_slice_unchecked(&account_info.data.borrow_mut())?;
    Ok(md)
  }
  pub fn size(&self) -> usize {
    let size = 
        1 // u8 for kind enum
      + 32 // pubkey mint
      + 1 // u8 for modifiable bool
      + 1 // u8 for update type enum
      + self.data.size()
      + 1; // Update Authority Option

    return match self.update_authority {
      None => {
        size
      }
      Some (auth) => {
        size + 32 // pubkey of the authority
      }
    }
  }
}