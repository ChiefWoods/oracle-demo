use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::{price_update::PriceUpdateV2, ID as PYTH_RECEIVER_PROGRAM_ID};

#[cfg(not(feature = "disable-staleness-check"))]
use crate::constants::MAX_TS_STALENESS;
#[cfg(not(feature = "disable-staleness-check"))]
use crate::instructions::ensure_fresh_slot;
use crate::price::Price;

#[derive(Accounts)]
pub struct ReadPythCore {}

/// Remaining accounts:
/// - price_update_v2
pub fn handler(ctx: Context<ReadPythCore>, feed_id: [u8; 32]) -> Result<Price> {
    let [price_update, ..] = ctx.remaining_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // validate account owner
    if price_update.owner != &PYTH_RECEIVER_PROGRAM_ID {
        return Err(ProgramError::InvalidAccountOwner.into());
    }

    // validate account discriminator and state
    let data = price_update.try_borrow_data()?;
    let mut account_data: &[u8] = &data;
    let price_update = PriceUpdateV2::try_deserialize(&mut account_data)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "disable-staleness-check")] {
            let price = price_update.get_price_unchecked(&feed_id)?;
        } else {
            let clock = Clock::get()?;
            let price = price_update.get_price_no_older_than(&clock, MAX_TS_STALENESS, &feed_id)?;

            ensure_fresh_slot(&clock, price_update.posted_slot)?;
        }
    }

    Price::scale(i128::from(price.price), price.exponent)
}
