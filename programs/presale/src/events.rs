use anchor_lang::prelude::*;

#[event]
pub struct BoughtWithSol {
  pub iteration: i16,
  pub buyer: Pubkey,
  pub adviser: String,
  pub amount: u64,
  pub token_amount: u128,
}

#[event]
pub struct BoughtWithUsdt {
  pub iteration: i16,
  pub buyer: Pubkey,
  pub adviser: String,
  pub amount: u64,
  pub token_amount: u128,
}

#[event]
pub struct BoughtWithUsdc {
  pub iteration: i16,
  pub buyer: Pubkey,
  pub adviser: String,
  pub amount: u64,
  pub token_amount: u128,
}

#[event]
pub struct ClaimedSol {
  pub code: String,
  pub amount: u64,
}

#[event]
pub struct ClaimedUsdc {
  pub code: String,
  pub amount: u64,
}

#[event]
pub struct ClaimedUsdt {
  pub code: String,
  pub amount: u64,
}
