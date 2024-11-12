use anchor_lang::prelude::*;

#[error_code]
pub enum Presale {
  #[msg("Unauthorized Signer")]
  UnauthorizedSigner,
  #[msg("Signature verification failed.")]
  SignatureVerificationFailed,
  #[msg("Presale already open")]
  PresaleOpen,
  #[msg("Presale already closed")]
  PresaleClosed,
  #[msg("Presale not enabled")]
  PresaleNotEnabled,
  #[msg("Presale min buy not reached")]
  PresaleMinBuyNotReached,
  #[msg("Presale c adviser percent too large")]
  PresaleCAdviserPercentTooLarge,
  #[msg("Presale t adviser percent too large")]
  PresaleTAdviserPercentTooLarge,
  #[msg("Iteration supply is too small")]
  IterationSupplyTooSmall,
  #[msg("Iteration already open")]
  IterationOpen,
  #[msg("Iteration already closed")]
  IterationClosed,
  #[msg("Iteration total supply exceeded")]
  IterationSupplyExceeded,
  #[msg("Inactive step account")]
  InactiveIteration,
  #[msg("Wrong price feed account")]
  WrongPriceFeedId,
  #[msg("Wrong stablecoin account")]
  WrongStablecoin,
  #[msg("Wrong store address")]
  WrongStore,
  #[msg("Oracle price is down")]
  PriceIsDown,
  #[msg("Adviser no funds")]
  AdviserNoFunds,
  #[msg("Expired signature")]
  ExpiredSignature,
}
