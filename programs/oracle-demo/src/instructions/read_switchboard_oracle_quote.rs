use anchor_lang::prelude::*;
use switchboard_on_demand::{
    QuoteVerifier, PRECISION, QUOTE_DISCRIMINATOR, QUOTE_PROGRAM_ID, QUOTE_TAIL_DISCRIMINATOR,
};

use crate::{error::OracleDemoError, instructions::ensure_fresh_slot, price::Price};

#[derive(Accounts)]
pub struct ReadSwitchboardOracleQuote {}

/// Remaining accounts:
/// - oracle_quote
pub fn handler(ctx: Context<ReadSwitchboardOracleQuote>, feed_id: [u8; 32]) -> Result<Price> {
    let [oracle_quote, ..] = ctx.remaining_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // validate account owner
    if oracle_quote.owner.to_bytes() != QUOTE_PROGRAM_ID.to_bytes() {
        return Err(ProgramError::InvalidAccountOwner.into());
    }

    // validate account discriminator
    let data = oracle_quote.try_borrow_data()?;
    if data.len() < 42 || data[..8] != QUOTE_DISCRIMINATOR {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // validate account state
    let payload_len = usize::from(u16::from_le_bytes(
        data[40..42]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    ));
    let payload_end = 42usize
        .checked_add(payload_len)
        .ok_or(ProgramError::InvalidAccountData)?;
    let payload = data
        .get(42..payload_end)
        .ok_or(ProgramError::InvalidAccountData)?;

    if !payload.ends_with(&QUOTE_TAIL_DISCRIMINATOR) {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let quote = QuoteVerifier::new()
        .parse_account_unverified(oracle_quote)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // validate staleness
    ensure_fresh_slot(&Clock::get()?, quote.slot())?;

    // validate feed id
    let quoted_feed = quote
        .feeds()
        .iter()
        .find(|feed| feed.feed_id() == &feed_id)
        .ok_or(ProgramError::InvalidAccountData)?;
    let decimals = u8::try_from(PRECISION).map_err(|_| OracleDemoError::InvalidPrice)?;
    Price::scale_decimals(quoted_feed.feed_value(), decimals)
}
