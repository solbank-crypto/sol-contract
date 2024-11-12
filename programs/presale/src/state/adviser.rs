use anchor_lang::prelude::*;

#[account]
pub struct Adviser {
  c_percent: u64,
  t_percent: u64,

  sol_reward: u64,
  usdt_reward: u64,
  usdc_reward: u64,
  token_reward: u128,

  enabled: bool,
}

impl Adviser {
  pub const MAX_SIZE: usize = (5 * 8) + 16 + 1 + 3;

  pub fn init(
    &mut self,
    c_percent: u64,
    t_percent: u64,
  ) -> Result<()> {
    self.c_percent = c_percent;
    self.t_percent = t_percent;

    self.sol_reward = 0;
    self.usdt_reward = 0;
    self.usdc_reward = 0;
    self.token_reward = 0;

    self.enabled = true;

    Ok(())
  }

  pub fn set_interest(
    &mut self,
    c_percent: u64,
    t_percent: u64,
  ) -> Result<()> {
    self.c_percent = c_percent;
    self.t_percent = t_percent;

    Ok(())
  }

  pub fn set_sol_reward(
    &mut self,
    amount: u64,
  ) -> Result<()> {
    self.sol_reward += amount;

    Ok(())
  }

  pub fn reset_sol_reward(
    &mut self,
  ) -> Result<()> {
    self.sol_reward = 0;

    Ok(())
  }

  pub fn set_usdt_reward(
    &mut self,
    amount: u64,
  ) -> Result<()> {
    self.usdt_reward += amount;

    Ok(())
  }

  pub fn reset_usdt_reward(
    &mut self,
  ) -> Result<()> {
    self.usdt_reward = 0;

    Ok(())
  }

  pub fn set_usdc_reward(
    &mut self,
    amount: u64,
  ) -> Result<()> {
    self.usdc_reward += amount;

    Ok(())
  }

  pub fn reset_usdc_reward(
    &mut self,
  ) -> Result<()> {
    self.usdc_reward = 0;

    Ok(())
  }

  pub fn set_token_reward(
    &mut self,
    amount: u128,
  ) -> Result<()> {
    self.token_reward += amount;

    Ok(())
  }

  pub fn get_percents(
    &mut self,
  ) -> (u64, u64) {
    (self.c_percent, self.t_percent)
  }

  pub fn get_sol_reward(
    &mut self,
  ) -> u64 {
    self.sol_reward
  }

  pub fn get_usdt_reward(
    &mut self,
  ) -> u64 {
    self.usdt_reward
  }

  pub fn get_usdc_reward(
    &mut self,
  ) -> u64 {
    self.usdc_reward
  }

  pub fn get_token_reward(
    &mut self,
  ) -> u128 {
    self.token_reward
  }

  pub fn enable(
    &mut self,
  ) -> Result<()> {
    self.enabled = true;

    Ok(())
  }

  pub fn disable(
    &mut self,
  ) -> Result<()> {
    self.enabled = false;

    Ok(())
  }
}
