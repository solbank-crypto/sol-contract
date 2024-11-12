use anchor_lang::prelude::*;
use crate::errors;

#[derive(Clone, PartialEq, AnchorDeserialize, AnchorSerialize)]
pub enum Status {
  None,
  Closed,
  Open,
}

#[account]
pub struct Presale {
  min_buy: u64,
  c_percent: u64,
  t_percent: u64,
  total_released: u128,
  iteration: i16,
  status: Status,
}

impl Presale {
  pub const MAX_SIZE: usize = (4 * 8) + 16 + 2 + (32 + 1) * 1;

  pub fn init(
    &mut self,
  ) -> Result<()> {
    self.iteration = -1;
    self.min_buy = 1_000_000_000; // $1

    self.c_percent = 50000000; // 5%
    self.t_percent = 50000000; // 5%

    self.total_released = 0;
    self.status = Status::None;

    Ok(())
  }

  pub fn set_min_buy(
    &mut self,
    min: u64,
  ) -> Result<()> {
    self.min_buy = min;

    Ok(())
  }

  pub fn set_percents(
    &mut self,
    c: u64,
    t: u64,
  ) -> Result<()> {
    if c > 1000_000_000 {
      return err!(errors::Presale::PresaleCAdviserPercentTooLarge);
    }

    if t > 1000_000_000 {
      return err!(errors::Presale::PresaleTAdviserPercentTooLarge);
    }

    self.c_percent = c;
    self.t_percent = t;

    Ok(())
  }

  pub fn open_presale(
    &mut self,
  ) -> Result<()> {
    if self.status == Status::Open || self.status == Status::Closed {
      return err!(errors::Presale::PresaleOpen);
    }

    self.status = Status::Open;

    Ok(())
  }

  pub fn close_presale(
    &mut self,
  ) -> Result<()> {
    if self.status != Status::Open {
      return err!(errors::Presale::PresaleClosed);
    }

    self.status = Status::Closed;

    Ok(())
  }

  pub fn set_iteration(
    &mut self,
    iteration: i16,
  ) -> Result<()> {
    self.iteration = iteration;

    Ok(())
  }

  pub fn add_sold(
    &mut self,
    amount: u128,
  ) -> Result<()> {
    self.total_released += amount;

    Ok(())
  }

  pub fn get_current_iteration(
    &self,
  ) -> i16 {
    self.iteration
  }

  pub fn get_min_buy(
    &self,
  ) -> u128 {
    u128::from(self.min_buy)
  }

  pub fn get_total_released(
    &self,
  ) -> u128 {
    self.total_released
  }

  pub fn get_percents(
    &mut self,
  ) -> (u64, u64) {
    (self.c_percent, self.t_percent)
  }

  pub fn is_open(
    &self,
  ) -> bool {
    self.status == Status::Open
  }
}
