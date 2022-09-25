use solana_program::{
  borsh::try_from_slice_unchecked,
  account_info::AccountInfo,
  program_error::ProgramError,
  pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};

pub const PREFIX: &str = "ree-metadata";

pub trait MetadataData {
  fn size(&self) -> usize;
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Royalty {
  pub address: Pubkey,
  pub share: u16, // 2
  pub verified: bool // 1
}

impl Royalty {
  pub fn size() -> usize {3 + 32}
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CustomNft {
  pub data: Vec<u8>
}

impl MetadataData for CustomNft {
  fn size(&self) -> usize {
    return 4 + self.data.len()
  }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ArtNft {
  pub name: String, //4 + len
  pub symbol: String, //4 + len
  pub uri: String, //4 + len
  pub resale_fee: u16, // 2
  pub initial_sale: bool,
  pub collection: Option<Pubkey>,
  pub royalties: Vec<Royalty>, // 1 + 4 + (Royaty * len)
}

impl MetadataData for ArtNft {
  fn size(&self) -> usize {
    let mut size = 4 // name string size buffer
    + self.name.len()
    + 4 // symbol string size buffer
    + self.symbol.len()
    + 4 // uri string size buffer
    + self.uri.len()
    + 2 // resale fee
    + 1 // initial sale boolean
    + 1; // collection Option buffer

    size += match &self.collection {
      Some(_) => 32,
      None => 0
    };

    size += 4 + Royalty::size() * self.royalties.len();

    return size;
  }
}

impl ArtNft {
  pub fn from_slice(data: &[u8]) -> Result<ArtNft, ProgramError> {
    let an: ArtNft = try_from_slice_unchecked(data)?;
    Ok(an)
  }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum Kind {
  Uninitialized,
  RoyaltyArt,
}

impl From<&u8> for Kind {
  fn from(orig: &u8) -> Self {
    match orig {
      1 => Kind::RoyaltyArt,
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
  // TODO:should add in a version field
  pub version: u8,
  pub kind: Kind,
  pub mint: Pubkey,
  pub is_modifiable: bool,
  pub update_type: UpdateType,
  pub collection: Option<Pubkey>,
  pub update_authority: Option<Pubkey>,
  pub data: T,
}

impl<T> Metadata<T>
where
  T: MetadataData + BorshDeserialize + BorshSerialize + PartialEq + Clone
{
  pub fn from_account_info(account_info: &AccountInfo) -> Result<Metadata<T>, ProgramError> {
    let data = &account_info.data.borrow();
    let md: Metadata<T> = try_from_slice_unchecked(data)?;
    Ok(md)
  }

  pub fn get_kind(account_info: &AccountInfo) -> Result<Kind, ProgramError> {
    let data = &account_info.data.borrow();
    Ok(Kind::from(&data[0]))
  }
  
  pub fn size(&self) -> usize {
    let mut size = 
        1 // u8 for kind enum
      + 32 // pubkey mint
      + 1 // u8 for modifiable bool
      + 1 // u8 for update type enum
      + self.data.size()
      + 1; // collection optional buffer;
    
    size += match self.collection {
      None => 0,
      Some(_) => 32,
    };
      
    size += 1; // Update Authority Optional buffer;

    return match self.update_authority {
      None => {
        size
      }
      Some (_) => {
        size + 32 // pubkey of the authority
      }
    }
  }
}
