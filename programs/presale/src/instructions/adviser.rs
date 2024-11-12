use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer as SplTransfer};
use solana_program::sysvar::instructions::ID as IX_ID;
use crate::config::{ USDC, USDT, ADVISER_TAG };

use crate::events;
use crate::errors;
use crate::state::adviser::*;

pub fn init_adviser(
  ctx: Context<InitAdviser>,
  c_percent: u64,
  t_percent: u64,
) -> Result<()> {
  let adviser = &mut ctx.accounts.adviser;
  adviser.init(c_percent, t_percent)
}

pub fn set_adviser_interest(
  ctx: Context<SetAdviserInterest>,
  c_percent: u64,
  t_percent: u64,
) -> Result<()> {
  let adviser = &mut ctx.accounts.adviser;
  adviser.set_interest(c_percent, t_percent)
}

pub fn enable_adviser(
  ctx: Context<SetAdviserEnabled>,
) -> Result<()> {
  let adviser = &mut ctx.accounts.adviser;
  adviser.enable()
}

pub fn disable_adviser(
  ctx: Context<SetAdviserDisabled>,
) -> Result<()> {
  let adviser = &mut ctx.accounts.adviser;
  adviser.disable()
}

pub fn claim_sol(
  ctx: Context<ClaimSol>,
  adviser_code: String,
) -> Result<()> {
  let payer = &mut ctx.accounts.payer;
  let adviser = &mut ctx.accounts.adviser;
  
  let sol_interest = adviser.get_sol_reward();
  if sol_interest > 0 {
    adviser.reset_sol_reward().unwrap();

    adviser.sub_lamports(sol_interest).unwrap();
    payer.add_lamports(sol_interest).unwrap();

    emit!(events::ClaimedSol {
      code: adviser_code,
      amount: sol_interest,
    });
  }

  Ok(())
}

pub fn claim_usdc(
  ctx: Context<ClaimUsdc>,
  adviser_code: String,
) -> Result<()> {
  let adviser = &mut ctx.accounts.adviser;
  
  let adviser_ata = &ctx.accounts.adviser_ata;
  let adviser_pda_ata = &ctx.accounts.adviser_pda_ata;
  let program = &ctx.accounts.token_program;

  let amount = adviser.get_usdc_reward();
  if amount == 0 {
    return err!(errors::Presale::AdviserNoFunds);
  }

  adviser.reset_usdc_reward().unwrap();

  let bump = &[ctx.bumps.adviser];
  let seeds: &[&[u8]] = &[ADVISER_TAG, b"_", adviser_code.as_ref(), bump];
  let signer_seeds = &[&seeds[..]];

  let cpi_accounts = SplTransfer {
    from: adviser_pda_ata.to_account_info(),
    to: adviser_ata.to_account_info(),
    authority: adviser.to_account_info(),
  };
  let ctx = CpiContext::new_with_signer(program.to_account_info(), cpi_accounts, signer_seeds);
  token::transfer(ctx, amount).unwrap();

  emit!(events::ClaimedUsdc {
    code: adviser_code,
    amount: amount,
  });

  Ok(())
}

pub fn claim_usdt(
  ctx: Context<ClaimUsdt>,
  adviser_code: String,
) -> Result<()> {
  let adviser = &mut ctx.accounts.adviser;
  
  let adviser_ata = &ctx.accounts.adviser_ata;
  let adviser_pda_ata = &ctx.accounts.adviser_pda_ata;
  let program = &ctx.accounts.token_program;

  let amount = adviser.get_usdt_reward();
  if amount == 0 {
    return err!(errors::Presale::AdviserNoFunds);
  }

  adviser.reset_usdt_reward().unwrap();

  let bump = &[ctx.bumps.adviser];
  let seeds: &[&[u8]] = &[ADVISER_TAG, b"_", adviser_code.as_ref(), bump];
  let signer_seeds = &[&seeds[..]];

  let cpi_accounts = SplTransfer {
    from: adviser_pda_ata.to_account_info(),
    to: adviser_ata.to_account_info(),
    authority: adviser.to_account_info(),
  };
  let ctx = CpiContext::new_with_signer(program.to_account_info(), cpi_accounts, signer_seeds);
  token::transfer(ctx, amount).unwrap();

  emit!(events::ClaimedUsdt {
    code: adviser_code,
    amount: amount,
  });

  Ok(())
}

#[derive(Accounts)]
#[instruction(adviser_code: String)]
pub struct InitAdviser<'info> {
  #[account(
    init,
    payer = payer,
    space = 8 + Adviser::MAX_SIZE,
    seeds = [
      ADVISER_TAG,
      b"_",
      adviser_code.as_ref()
    ],
    bump
  )]
  pub adviser: Account<'info, Adviser>,
  #[account(mut)]
  pub payer: Signer<'info>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdviserInterest<'info> {
  #[account(mut)]
  pub adviser: Account<'info, Adviser>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetAdviserEnabled<'info> {
  #[account(mut)]
  pub adviser: Account<'info, Adviser>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetAdviserDisabled<'info> {
  #[account(mut)]
  pub adviser: Account<'info, Adviser>,
  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(adviser_code: String)]
pub struct ClaimSol<'info> {
  #[account(
    mut,
    seeds = [
      ADVISER_TAG,
      b"_",
      adviser_code.as_ref()
    ],
    bump
  )]
  pub adviser: Account<'info, Adviser>,

  #[account(address = IX_ID)]
  /// CHECK: we need this for sign
  pub ix_sysvar: AccountInfo<'info>,

  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(adviser_code: String)]
pub struct ClaimUsdc<'info> {
  #[account(
    mut,
    seeds = [
      ADVISER_TAG,
      b"_",
      adviser_code.as_ref()
    ],
    bump
  )]
  pub adviser: Account<'info, Adviser>,
  #[account(
    mut,
    constraint = adviser_ata.mint == USDC.parse::<Pubkey>().unwrap(),
    constraint = adviser_ata.owner == payer.key(),
  )]
  pub adviser_ata: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = adviser_pda_ata.mint == USDC.parse::<Pubkey>().unwrap(),
    constraint = adviser_pda_ata.owner == adviser.key(),
  )]
  pub adviser_pda_ata: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,

  #[account(address = IX_ID)]
  /// CHECK: we need this for sign
  pub ix_sysvar: AccountInfo<'info>,

  #[account(mut)]
  pub payer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(adviser_code: String)]
pub struct ClaimUsdt<'info> {
  #[account(
    mut,
    seeds = [
      ADVISER_TAG,
      b"_",
      adviser_code.as_ref()
    ],
    bump
  )]
  pub adviser: Account<'info, Adviser>,
  #[account(
    mut,
    constraint = adviser_ata.mint == USDT.parse::<Pubkey>().unwrap(),
    constraint = adviser_ata.owner == payer.key(),
  )]
  pub adviser_ata: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = adviser_pda_ata.mint == USDT.parse::<Pubkey>().unwrap(),
    constraint = adviser_pda_ata.owner == adviser.key(),
  )]
  pub adviser_pda_ata: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,

  #[account(address = IX_ID)]
  /// CHECK: we need this for sign
  pub ix_sysvar: AccountInfo<'info>,

  #[account(mut)]
  pub payer: Signer<'info>,
}
