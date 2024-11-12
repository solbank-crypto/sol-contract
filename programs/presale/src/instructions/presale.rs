use anchor_lang::{
  prelude::*,
  solana_program::{ program::invoke, system_instruction::transfer },
};
use std::str::FromStr;
use anchor_spl::token::{ self, Token, TokenAccount, Transfer as SplTransfer };
use pyth_solana_receiver_sdk::price_update::{ get_feed_id_from_hex, PriceUpdateV2 };

use crate::errors;
use crate::events;
use crate::state::presale::Presale;
use crate::state::iteration::Iteration;
use crate::state::adviser::Adviser;
use crate::state::buyer::Buyer;

use crate::config::{
  SOL_USD_PRICEFEED, STORE, USDC, USDT,
  PRECISION, STABLE_PRECISION, ADVISER_TAG,
  BUYER_TAG, FEED_MAX_AGE, FEED_ID,
};

pub fn init_presale(
  ctx: Context<InitPresale>,
) -> Result<()> {
  let presale = &mut ctx.accounts.presale;
  presale.init()
}

pub fn set_presale_min_buy(
  ctx: Context<SetPresaleMinBuy>,
  min: u64,
) -> Result<()> {
  let presale = &mut ctx.accounts.presale;
  presale.set_min_buy(min)
}

pub fn set_presale_interest(
  ctx: Context<SetPresaleInterest>,
  c_percent: u64,
  t_percent: u64,
) -> Result<()> {
  let presale = &mut ctx.accounts.presale;
  presale.set_percents(c_percent, t_percent)
}

pub fn open_presale(
  ctx: Context<OpenPresale>,
) -> Result<()> {
  let presale = &mut ctx.accounts.presale;
  presale.open_presale()
}

pub fn close_presale(
  ctx: Context<ClosePresale>,
) -> Result<()> {
  let presale = &mut ctx.accounts.presale;
  presale.close_presale()
}

pub fn buy_sol(
  ctx: Context<BuySol>,
  code: String,
  amount: u64,
) -> Result<()> {
  let to_account_infos = &mut ctx.accounts.to_account_infos();
  let payer = &mut ctx.accounts.payer;
  let presale = &mut ctx.accounts.presale;
  let iteration = &mut ctx.accounts.iteration;
  let buyer = &mut ctx.accounts.buyer;
  let adviser = &mut ctx.accounts.adviser;
  let price_update = &ctx.accounts.price_update;
  let store_info = &mut ctx.accounts.store_info;

  if !presale.is_open() {
    return err!(errors::Presale::PresaleNotEnabled);
  }

  if !iteration.is_open() {
    return err!(errors::Presale::IterationClosed);
  }

  if presale.get_current_iteration() != iteration.get_id() {
    return err!(errors::Presale::InactiveIteration);
  }

  if Pubkey::from_str(STORE) != Ok(store_info.key()){
    return Err(error!(errors::Presale::WrongStore))
  };

  if Pubkey::from_str(SOL_USD_PRICEFEED) != Ok(price_update.key()){
    return Err(error!(errors::Presale::WrongPriceFeedId))
  };
  
  let (price, expo) = get_price_test(price_update).unwrap();
  let usd_amount = u128::from(amount) * price / 10u128.pow(expo);
  let token_amount = usd_amount * 10u128.pow(PRECISION) / u128::from(iteration.get_price());

  if presale.get_min_buy() > usd_amount {
    return err!(errors::Presale::PresaleMinBuyNotReached);
  }

  if iteration.get_sold() + token_amount > iteration.get_total() {
    return err!(errors::Presale::IterationSupplyExceeded);
  }
  
  let (adviser_sol_reward, adviser_token_reward) = get_interest(presale, &code, adviser, amount, token_amount).unwrap();
  let mut to_amount = amount;
  if adviser_sol_reward > 0 {
    to_amount = to_amount - adviser_sol_reward;
  }

  let instruction = &transfer(&payer.key(), &store_info.key(), to_amount);
  invoke(instruction, to_account_infos).unwrap();

  if adviser_sol_reward > 0 {
    let instruction = &transfer(&payer.key(), &adviser.key(), adviser_sol_reward);
    invoke(instruction, to_account_infos).unwrap();
  }

  // Updating presale details
  presale.add_sold(token_amount).unwrap();

  // Updating iteration details
  iteration.increase_sold(token_amount).unwrap();

  // Updating buyer details
  buyer.increase_balance(token_amount).unwrap();

  // Updating adviser details
  if !code.is_empty() {
    adviser.set_sol_reward(adviser_sol_reward).unwrap();
    adviser.set_token_reward(adviser_token_reward).unwrap();
  };

  emit!(events::BoughtWithSol {
    iteration: iteration.get_id(),
    buyer: payer.key(),
    adviser: code,
    amount: amount,
    token_amount: token_amount,
  });
  Ok(())
}

pub fn buy_usdc(
  ctx: Context<BuyUsdc>,
  code: String,
  amount: u64,
) -> Result<()> {
  let payer = &mut ctx.accounts.payer;
  let presale = &mut ctx.accounts.presale;
  let iteration = &mut ctx.accounts.iteration;
  let buyer = &mut ctx.accounts.buyer;
  let adviser = &mut ctx.accounts.adviser;

  let buyer_ata = &ctx.accounts.buyer_ata;
  let store_ata = &ctx.accounts.store_ata;
  let adviser_pda_ata = &ctx.accounts.adviser_pda_ata;
  let token_program = &ctx.accounts.token_program;

  if !presale.is_open() {
    return err!(errors::Presale::PresaleNotEnabled);
  }

  if !iteration.is_open() {
    return err!(errors::Presale::IterationClosed);
  }

  if presale.get_current_iteration() != iteration.get_id() {
    return err!(errors::Presale::InactiveIteration);
  }

  let usd_amount = u128::from(amount) * 10u128.pow(STABLE_PRECISION);
  let token_amount = usd_amount * 10u128.pow(PRECISION) / u128::from(iteration.get_price());

  if presale.get_min_buy() > usd_amount {
    return err!(errors::Presale::PresaleMinBuyNotReached);
  }

  if iteration.get_sold() + token_amount > iteration.get_total() {
    return err!(errors::Presale::IterationSupplyExceeded);
  }

  let (adviser_usdc_reward, adviser_token_reward) = get_interest(presale, &code, adviser, amount, token_amount).unwrap();
  let mut to_amount = amount;
  if adviser_usdc_reward > 0 {
    to_amount = to_amount - adviser_usdc_reward;
  }

  let cpi_accounts = SplTransfer {
    from: buyer_ata.to_account_info(),
    to: store_ata.to_account_info(),
    authority: payer.to_account_info(),
  };
  let cpi_program = token_program.to_account_info();
  token::transfer(CpiContext::new(cpi_program, cpi_accounts), to_amount).unwrap();
  
  if adviser_usdc_reward > 0 {
    let cpi_accounts = SplTransfer {
      from: buyer_ata.to_account_info(),
      to: adviser_pda_ata.to_account_info(),
      authority: payer.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    token::transfer(CpiContext::new(cpi_program, cpi_accounts), adviser_usdc_reward).unwrap();
  }

  // Updating presale details
  presale.add_sold(token_amount).unwrap();

  // Updating iteration details
  iteration.increase_sold(token_amount).unwrap();

  // Updating buyer details
  buyer.increase_balance(token_amount).unwrap();

  // Updating adviser details
  if !code.is_empty() {
    adviser.set_usdc_reward(adviser_usdc_reward).unwrap();
    adviser.set_token_reward(adviser_token_reward).unwrap();
  };

  emit!(events::BoughtWithUsdc {
    iteration: iteration.get_id(),
    buyer: payer.key(),
    adviser: code,
    amount: amount,
    token_amount: token_amount,
  });

  Ok(())
}

pub fn buy_usdt(
  ctx: Context<BuyUsdt>,
  code: String,
  amount: u64,
) -> Result<()> {
  let payer = &mut ctx.accounts.payer;
  let presale = &mut ctx.accounts.presale;
  let iteration = &mut ctx.accounts.iteration;
  let buyer = &mut ctx.accounts.buyer;
  let adviser = &mut ctx.accounts.adviser;

  let buyer_ata = &ctx.accounts.buyer_ata;
  let store_ata = &ctx.accounts.store_ata;
  let adviser_pda_ata = &ctx.accounts.adviser_pda_ata;
  let token_program = &ctx.accounts.token_program;

  if !presale.is_open() {
    return err!(errors::Presale::PresaleNotEnabled);
  }

  if !iteration.is_open() {
    return err!(errors::Presale::IterationClosed);
  }

  if presale.get_current_iteration() != iteration.get_id() {
    return err!(errors::Presale::InactiveIteration);
  }

  let usd_amount = u128::from(amount) * 10u128.pow(STABLE_PRECISION);
  let token_amount = usd_amount * 10u128.pow (PRECISION) / u128::from(iteration.get_price());

  if presale.get_min_buy() > usd_amount {
    return err!(errors::Presale::PresaleMinBuyNotReached);
  }

  if iteration.get_sold() + token_amount > iteration.get_total() {
    return err!(errors::Presale::IterationSupplyExceeded);
  }

  let (adviser_usdt_reward, adviser_token_reward) = get_interest(presale, &code, adviser, amount, token_amount).unwrap();
  let mut to_amount = amount;
  if adviser_usdt_reward > 0 {
    to_amount = to_amount - adviser_usdt_reward;
  }

  let cpi_accounts = SplTransfer {
    from: buyer_ata.to_account_info(),
    to: store_ata.to_account_info(),
    authority: payer.to_account_info(),
  };
  let cpi_program = token_program.to_account_info();
  token::transfer(CpiContext::new(cpi_program, cpi_accounts), to_amount).unwrap();
  
  if adviser_usdt_reward > 0 {
    let cpi_accounts = SplTransfer {
      from: buyer_ata.to_account_info(),
      to: adviser_pda_ata.to_account_info(),
      authority: payer.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    token::transfer(CpiContext::new(cpi_program, cpi_accounts), adviser_usdt_reward).unwrap();
  }

  // Updating presale details
  presale.add_sold(token_amount).unwrap();

  // Updating iteration details
  iteration.increase_sold(token_amount).unwrap();

  // Updating buyer details
  buyer.increase_balance(token_amount).unwrap();

  // Updating adviser details
  if !code.is_empty() {
    adviser.set_usdt_reward(adviser_usdt_reward).unwrap();
    adviser.set_token_reward(adviser_token_reward).unwrap();
  };

  emit!(events::BoughtWithUsdt {
    iteration: iteration.get_id(),
    buyer: payer.key(),
    adviser: code,
    amount: amount,
    token_amount: token_amount,
  });

  Ok(())
}

fn get_price(price_update: &Account<PriceUpdateV2>)
  -> Result<(u128, u32)>
{
  let feed_id = &get_feed_id_from_hex(FEED_ID)?;
  let current_price = price_update.get_price_no_older_than(
      &Clock::get()?,
      FEED_MAX_AGE,
      feed_id,
  ).unwrap();
  let price = u64::try_from(current_price.price).unwrap();
  let expo = u32::try_from(-current_price.exponent).unwrap();
  Ok((u128::from(price), expo))
}

fn get_price_test(_price_update: &AccountInfo)
  -> Result<(u128, u32)>
{
  Ok((144000000000, 9))
}

fn get_interest(
  presale: &mut Account<Presale>,
  code: &str,
  adviser: &mut Account<Adviser>,
  amount: u64,
  token_amount: u128,
)
  -> Result<(u64, u128)>
{
  if code.is_empty() {
    return Ok((0, 0));
  };

  let (p_c_percent, p_t_percent) = presale.get_percents();
  let (a_c_percent, a_t_percent) = adviser.get_percents();

  let c_percent = u64::max(p_c_percent, a_c_percent);
  let t_percent = u64::max(p_t_percent, a_t_percent);

  let amount = amount * c_percent / 10u64.pow(PRECISION);
  let reward_token_amount = token_amount * u128::from(t_percent) / 10u128.pow(PRECISION);

  Ok((amount, reward_token_amount))
}

#[derive(Accounts)]
pub struct InitPresale<'info> {
  #[account(
    init,
    payer = payer,
    space = 8 + Presale::MAX_SIZE,
    seeds = [],
    bump,
  )]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(min: u64)]
pub struct SetPresaleMinBuy<'info> {
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(main_interest: u64, secondary_interest: u64)]
pub struct SetPresaleInterest<'info> {
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(threshold: u64, percent: u64)]
pub struct SetPresaleBonus<'info> {
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct OpenPresale<'info> {
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClosePresale<'info> {
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(code: String, amount: u64)]
pub struct BuySol<'info> {
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
  #[account(mut)]
  pub iteration: Account<'info, Iteration>,
  #[account(
    init_if_needed,
    payer = payer,
    space = 8 + Buyer::MAX_SIZE,
    seeds = [
      BUYER_TAG,
      b"_",
      payer.key().as_ref()
    ],
    bump
  )]
  pub buyer: Account<'info, Buyer>,
  #[account(
    init_if_needed,
    payer = payer,
    space = 8 + Adviser::MAX_SIZE,
    seeds = [
      ADVISER_TAG,
      b"_",
      code.as_ref()
    ],
    bump
  )]
  pub adviser: Account<'info, Adviser>,
  /// CHECK: price oracle
  pub price_update: AccountInfo<'info>,
  #[account(mut)]
    /// CHECK: store info
  pub store_info: AccountInfo<'info>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(code: String, amount: u64)]
pub struct BuyUsdc<'info> {
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
  #[account(mut)]
  pub iteration: Account<'info, Iteration>,
  #[account(
    init_if_needed,
    payer = payer,
    space = 8 + Buyer::MAX_SIZE,
    seeds = [
      BUYER_TAG,
      b"_",
      payer.key().as_ref()
    ],
    bump
  )]
  pub buyer: Account<'info, Buyer>,
  #[account(
    init_if_needed,
    payer = payer,
    space = 8 + Adviser::MAX_SIZE,
    seeds = [
      ADVISER_TAG,
      b"_",
      code.as_ref()
    ],
    bump
  )]
  pub adviser: Account<'info, Adviser>,
  #[account(
    mut,
    constraint = buyer_ata.mint == USDC.parse::<Pubkey>().unwrap(),
    constraint = buyer_ata.owner == payer.key(),
  )]
  pub buyer_ata: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = store_ata.mint == USDC.parse::<Pubkey>().unwrap(),
    constraint = store_ata.owner == STORE.parse::<Pubkey>().unwrap(),
  )]
  pub store_ata: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = adviser_pda_ata.mint == USDC.parse::<Pubkey>().unwrap(),
    constraint = adviser_pda_ata.owner == adviser.key(),
  )]
  pub adviser_pda_ata: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(code: String, amount: u64)]
pub struct BuyUsdt<'info> {
  #[account(mut)]
  pub presale: Account<'info, Presale>,
  #[account(mut)]
  pub payer: Signer<'info>,
  #[account(mut)]
  pub iteration: Account<'info, Iteration>,
  #[account(
    init_if_needed,
    payer = payer,
    space = 8 + Buyer::MAX_SIZE,
    seeds = [
      BUYER_TAG,
      b"_",
      payer.key().as_ref()
    ],
    bump
  )]
  pub buyer: Account<'info, Buyer>,
  #[account(
    init_if_needed,
    payer = payer,
    space = 8 + Adviser::MAX_SIZE,
    seeds = [
      ADVISER_TAG,
      b"_",
      code.as_ref()
    ],
    bump
  )]
  pub adviser: Account<'info, Adviser>,
  #[account(
    mut,
    constraint = buyer_ata.mint == USDT.parse::<Pubkey>().unwrap(),
    constraint = buyer_ata.owner == payer.key(),
  )]
  pub buyer_ata: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = store_ata.mint == USDT.parse::<Pubkey>().unwrap(),
    constraint = store_ata.owner == STORE.parse::<Pubkey>().unwrap(),
  )]
  pub store_ata: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = adviser_pda_ata.mint == USDT.parse::<Pubkey>().unwrap(),
    constraint = adviser_pda_ata.owner == adviser.key(),
  )]
  pub adviser_pda_ata: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}
