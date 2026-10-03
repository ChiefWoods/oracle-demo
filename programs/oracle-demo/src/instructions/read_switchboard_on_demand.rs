use anchor_lang::prelude::*;
use switchboard_on_demand::{
    OnDemandError, PullFeedAccountData, ON_DEMAND_DEVNET_PID, ON_DEMAND_MAINNET_PID,
};

#[cfg(not(feature = "disable-staleness-check"))]
use crate::constants::MAX_SLOT_STALENESS;
#[cfg(not(feature = "disable-staleness-check"))]
use crate::instructions::ensure_fresh_slot;
use crate::{error::OracleDemoError, price::Price};

#[derive(Accounts)]
pub struct ReadSwitchboardOnDemand {}

/// Remaining accounts:
/// - pull_feed
pub fn handler(ctx: Context<ReadSwitchboardOnDemand>, feed_id: [u8; 32]) -> Result<Price> {
    let [pull_feed, ..] = ctx.remaining_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // validate account owner
    let owner = pull_feed.owner.to_bytes();
    if owner != ON_DEMAND_MAINNET_PID.to_bytes() && owner != ON_DEMAND_DEVNET_PID.to_bytes() {
        return Err(ProgramError::InvalidAccountOwner.into());
    }

    // validate account discriminator and state
    let parsed =
        PullFeedAccountData::parse(pull_feed.try_borrow_data()?).map_err(map_on_demand_error)?;

    // validate feed id
    if parsed.feed_hash != feed_id {
        return Err(ProgramError::InvalidAccountData.into());
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "disable-staleness-check")] {
            let value = parsed.result.value().ok_or(OracleDemoError::MissingPrice)?;
        } else {
            let clock = Clock::get()?;
            let value = parsed
                .get_value(
                    clock.slot,
                    u64::from(MAX_SLOT_STALENESS),
                    u32::from(parsed.min_sample_size.max(1)),
                    true,
                )
                .map_err(map_on_demand_error)?;

            ensure_fresh_slot(&clock, parsed.result.slot)?;
        }
    }

    Price::scale_decimals(
        value.mantissa(),
        u8::try_from(value.scale()).map_err(|_| OracleDemoError::InvalidPrice)?,
    )
}

fn map_on_demand_error(error: OnDemandError) -> Error {
    match error {
        OnDemandError::NotEnoughSamples => OracleDemoError::MissingPrice.into(),
        OnDemandError::StaleResult => OracleDemoError::StalePrice.into(),
        OnDemandError::IllegalFeedValue => OracleDemoError::InvalidPrice.into(),
        _ => ProgramError::InvalidAccountData.into(),
    }
}
