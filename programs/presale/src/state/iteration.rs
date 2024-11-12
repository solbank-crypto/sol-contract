use anchor_lang::prelude::*;
use crate::errors;

#[derive(Clone, PartialEq, AnchorDeserialize, AnchorSerialize)]
pub enum Status {
  None,
  Closed,
  Open,
}

#[account]
pub struct Iteration {
  id: i16,
  price: u64,
  sold: u128,
  total: u128,
  status: Status,
}

impl Iteration {
  pub const MAX_SIZE: usize = 2 + 8 + (2 * 16) + (32 + 1) + 2;

  pub fn init(
    &mut self,
    id: i16,
    price: u64,
    total: u128,
  ) -> Result<()> {
    self.id = id;
    self.price = price;
    self.total = total;
    self.sold = 0;
    self.status = Status::None;

    Ok(())
  }

  pub fn set_price(
    &mut self,
    price: u64,
  ) -> Result<()> {
    if self.status == Status::Open || self.status == Status::Closed {
      return err!(errors::Presale::IterationOpen);
    }

    self.price = price;

    Ok(())
  }

  pub fn set_total(
    &mut self,
    new_total: u128,
  ) -> Result<()> {
    if self.sold > new_total {
      return err!(errors::Presale::IterationSupplyTooSmall);
    }

    self.total = new_total;

    Ok(())
  }

  pub fn open(
    &mut self,
  ) -> Result<()> {
    if self.status == Status::Open || self.status == Status::Closed {
      return err!(errors::Presale::IterationOpen);
    }

    self.status = Status::Open;

    Ok(())
  }

  pub fn close_iteration(
    &mut self,
  ) -> Result<()> {
    if self.status != Status::Open {
      return err!(errors::Presale::IterationClosed);
    }

    self.status = Status::Closed;

    Ok(())
  }

  pub fn increase_sold(
    &mut self,
    amount: u128,
  ) -> Result<()> {
    self.sold += amount;

    Ok(())
  }

  pub fn get_id(
    &mut self,
  ) -> i16 {
    self.id
  }

  pub fn get_price(
    &mut self,
  ) -> u64 {
    self.price
  }

  pub fn get_sold(
    &mut self,
  ) -> u128 {
    self.sold
  }

  pub fn get_total(
    &mut self,
  ) -> u128 {
    self.total
  }

  pub fn is_open(
    &self,
  ) -> bool {
    self.status == Status::Open
  }
}
