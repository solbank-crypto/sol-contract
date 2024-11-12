use instructions::*;
use anchor_lang::prelude::*;
use signature::check_signature;

pub mod config;
pub mod signature;
pub mod errors;
pub mod events;
pub mod state;
pub mod instructions;

declare_id!("7QzR3zsNQwn27MbVwyCBavBX7xZnBczmHuaUh8ViNPLS");

#[program]
pub mod presale {
  use super::*;

  pub fn init(
    ctx: Context<InitPresale>,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::presale::init_presale(ctx)
  }

  pub fn set_presale_min_buy(
    ctx: Context<SetPresaleMinBuy>,
    min: u64,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::presale::set_presale_min_buy(ctx, min)
  }

  pub fn set_presale_adviser_interest(
    ctx: Context<SetPresaleInterest>,
    c_percent: u64,
    t_percent: u64,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::presale::set_presale_interest(ctx, c_percent, t_percent)
  }

  pub fn open_presale(
    ctx: Context<OpenPresale>,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::presale::open_presale(ctx)
  }

  pub fn close_presale(
    ctx: Context<ClosePresale>,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::presale::close_presale(ctx)
  }

  pub fn buy_sol(
    ctx: Context<BuySol>,
    adviser_code: String,
    amount: u64,
  ) -> Result<()> {
    instructions::presale::buy_sol(ctx, adviser_code, amount)
  }

  pub fn buy_usdc(
    ctx: Context<BuyUsdc>,
    adviser_code: String,
    amount: u64,
  ) -> Result<()> {
    instructions::presale::buy_usdc(ctx, adviser_code, amount)
  }

  pub fn buy_usdt(
    ctx: Context<BuyUsdt>,
    adviser_code: String,
    amount: u64,
  ) -> Result<()> {
    instructions::presale::buy_usdt(ctx, adviser_code, amount)
  }

  pub fn create_iteration(
    ctx: Context<CreateIteration>,
    id: i16,
    price: u64,
    total_supply: u128,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::iteration::create_iteration(ctx, id, price, total_supply)
  }

  pub fn set_iteration_price(
    ctx: Context<SetIterationPrice>,
    price: u64,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::iteration::set_iteration_price(ctx, price)
  }

  pub fn set_iteration_total(
    ctx: Context<SetIterationTotal>,
    total_supply: u128,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::iteration::set_iteration_total(ctx, total_supply)
  }

  pub fn open_iteration(
    ctx: Context<OpenIteration>,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::iteration::open_iteration(ctx)
  }

  pub fn close_iteration(
    ctx: Context<CloseIteration>,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::iteration::close_iteration(ctx)
  }

  pub fn init_adviser(
    ctx: Context<InitAdviser>,
    _adviser_code: String,
    main_interest: u64,
    secondary_interest: u64,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::adviser::init_adviser(ctx, main_interest, secondary_interest)
  }

  pub fn set_adviser_interest(
    ctx: Context<SetAdviserInterest>,
    main_interest: u64,
    secondary_interest: u64,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::adviser::set_adviser_interest(ctx, main_interest, secondary_interest)
  }

  pub fn enable_adviser(
    ctx: Context<SetAdviserEnabled>,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::adviser::enable_adviser(ctx)
  }

  pub fn disable_adviser(
    ctx: Context<SetAdviserDisabled>,
  ) -> Result<()> {
    if !config::only_owners(ctx.accounts.payer.key()) {
      return err!(errors::Presale::UnauthorizedSigner);
    }

    instructions::adviser::disable_adviser(ctx)
  }

  pub fn claim_sol(
    ctx: Context<ClaimSol>,
    adviser: String,
    deadline: u128,
    sig: [u8; 64],
    sig_index: u32
  ) -> Result<()> {
    check_signature(&adviser, &ctx.accounts.payer, sig, &ctx.accounts.ix_sysvar, deadline, sig_index).unwrap();
    instructions::adviser::claim_sol(ctx, adviser)
  }

  pub fn claim_usdc(
    ctx: Context<ClaimUsdc>,
    adviser: String,
    deadline: u128,
    sig: [u8; 64],
    sig_index: u32
  ) -> Result<()> {
    check_signature(&adviser, &ctx.accounts.payer, sig, &ctx.accounts.ix_sysvar, deadline, sig_index).unwrap();
    instructions::adviser::claim_usdc(ctx, adviser)
  }

  pub fn claim_usdt(
    ctx: Context<ClaimUsdt>,
    adviser: String,
    deadline: u128,
    sig: [u8; 64],
    sig_index: u32
  ) -> Result<()> {
    check_signature(&adviser, &ctx.accounts.payer, sig, &ctx.accounts.ix_sysvar, deadline, sig_index).unwrap();
    instructions::adviser::claim_usdt(ctx, adviser)
  }
}
