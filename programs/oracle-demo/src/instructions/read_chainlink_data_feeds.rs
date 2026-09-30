use anchor_lang::prelude::*;
use chainlink_solana::v2::{read_feed_v2, ReadError};

use crate::{error::OracleDemoError, instructions::ensure_fresh_slot, price::Price};

#[derive(Accounts)]
pub struct ReadChainlinkDataFeeds {}

/// Remaining accounts:
/// - feed
pub fn handler(ctx: Context<ReadChainlinkDataFeeds>, feed_address: Pubkey) -> Result<Price> {
    let [feed, ..] = ctx.remaining_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // validate feed address
    if feed.key != &feed_address {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // validate account owner, discriminator and state
    let parsed =
        read_feed_v2(feed.try_borrow_data()?, feed.owner.to_bytes()).map_err(map_read_error)?;
    let round = parsed
        .latest_round_data()
        .ok_or(OracleDemoError::MissingPrice)?;
    if round.answer <= 0 {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // validate staleness
    ensure_fresh_slot(&Clock::get()?, round.slot)?;

    Price::scale_decimals(round.answer, parsed.decimals())
}

fn map_read_error(error: ReadError) -> Error {
    match error {
        ReadError::InvalidOwner => ProgramError::InvalidAccountOwner.into(),
        ReadError::FeedLengthInvalid | ReadError::TransmissionNotFound => {
            OracleDemoError::MissingPrice.into()
        }
        _ => ProgramError::InvalidAccountData.into(),
    }
}
