use anchor_lang::prelude::*;
use crate::state::iteration::Iteration;
use crate::state::presale::Presale;

use crate::config::ITERATION_TAG;

pub fn create_iteration(
  ctx: Context<CreateIteration>,
  id: i16,
  price: u64,
  total: u128,
) -> Result<()> {
  let iteration = &mut ctx.accounts.iteration;
  iteration.init(id, price, total)
}

pub fn set_iteration_price(
  ctx: Context<SetIterationPrice>,
  price: u64,
) -> Result<()> {
  let iteration = &mut ctx.accounts.iteration;
  iteration.set_price(price)
}

pub fn set_iteration_total(
  ctx: Context<SetIterationTotal>,
  total: u128
) -> Result<()> {
  let iteration = &mut ctx.accounts.iteration;
  iteration.set_total(total)
}

pub fn open_iteration(
  ctx: Context<OpenIteration>,
) -> Result<()> {
  let iteration = &mut ctx.accounts.iteration;
  iteration.open().unwrap();

  let presale = &mut ctx.accounts.presale;
  presale.set_iteration(iteration.get_id())
}

pub fn close_iteration(
  ctx: Context<CloseIteration>,
) -> Result<()> {
  let iteration = &mut ctx.accounts.iteration;
  iteration.close_iteration()
}

#[derive(Accounts)]
#[instruction(id: i16)]
pub struct CreateIteration<'info> {
  #[account(
    init,
    payer = payer,
    space = 8 + Iteration::MAX_SIZE,
    seeds = [
      ITERATION_TAG,
      b"_",
      &id.to_le_bytes()
    ],
    bump,
  )]
  pub iteration: Account<'info, Iteration>,
  #[account(mut)]
  pub payer: Signer<'info>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(price: u64)]
pub struct SetIterationPrice<'info> {
  #[account(mut)]
  pub iteration: Account<'info, Iteration>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(total: u128)]
pub struct SetIterationTotal<'info> {
  #[account(mut)]
  pub iteration: Account<'info, Iteration>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct OpenIteration<'info> {
  #[account(mut)]
  pub iteration: Account<'info, Iteration>,
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseIteration<'info> {
  #[account(mut)]
  pub iteration: Account<'info, Iteration>,
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
}