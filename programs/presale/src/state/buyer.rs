use anchor_lang::prelude::*;

#[account]
pub struct Buyer {
  balance: u128,
}

impl Buyer {
  pub const MAX_SIZE: usize = 16 + 1;

  pub fn init(
    &mut self,
  ) -> Result<()> {
    self.balance = 0;

    Ok(())
  }

  pub fn increase_balance(
    &mut self,
    amount: u128,
  ) -> Result<()> {
    self.balance += amount;

    Ok(())
  }

  pub fn get_balance(
    &mut self,
  ) -> u128 {
    self.balance
  }
}
