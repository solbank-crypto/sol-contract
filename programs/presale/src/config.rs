use anchor_lang::prelude::*;

pub const ITERATION_TAG: &[u8]   = b"ITERATION";
pub const BUYER_TAG: &[u8]       = b"BUYER";
pub const ADVISER_TAG: &[u8]     = b"ADVISER";
pub const STORE: &str            = ""; // TODO: change

pub const SOL_USD_PRICEFEED: &str   = "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE";
pub const FEED_ID: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
pub const FEED_MAX_AGE: u64 = 36000; // 10 hours

pub const PRECISION: u32            = 9;
pub const STABLE_PRECISION: u32     = 3;

pub const USDT: &str                = ""; // TODO: change
pub const USDC: &str                = ""; // TODO: change

pub const SIGNATURE_SIGNER: &str     = ""; // TODO: change
const OWNERS: &[&str] = &[""]; // TODO: change

pub fn only_owners(address: Pubkey) -> bool {
  return OWNERS.contains(&address.to_string().as_str());
}
