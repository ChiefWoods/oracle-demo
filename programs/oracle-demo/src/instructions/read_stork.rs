use anchor_lang::prelude::*;
use borsh::BorshDeserialize;

use crate::{instructions::ensure_fresh_ts, price::Price};

const STORK_PROGRAM_ID: Pubkey = pubkey!("stork1JUZMKYgjNagHiK2KdMmb42iTnYe9bYUCDUk8n");
const STORK_FEED_SEED: &[u8] = b"stork_feed";
const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;
const STORK_QUANTIZED_VALUE_DECIMALS: u8 = 18;
const TEMPORAL_NUMERIC_VALUE_FEED_DISCRIMINATOR: [u8; 8] = [37, 166, 65, 196, 29, 105, 97, 161];

#[derive(Debug, PartialEq, Eq)]
struct TemporalNumericValue {
    timestamp_ns: u64,
    quantized_value: i128,
}

#[derive(Debug, PartialEq, Eq)]
struct TemporalNumericValueFeed {
    id: [u8; 32],
    latest_value: TemporalNumericValue,
}

impl TemporalNumericValueFeed {
    fn decode(data: &[u8]) -> Result<Self> {
        if data.get(..8) != Some(TEMPORAL_NUMERIC_VALUE_FEED_DISCRIMINATOR.as_slice()) {
            return Err(ProgramError::InvalidAccountData.into());
        }

        let mut account_data = data.get(8..).ok_or(ProgramError::InvalidAccountData)?;
        let id = <[u8; 32]>::deserialize(&mut account_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let timestamp_ns =
            u64::deserialize(&mut account_data).map_err(|_| ProgramError::InvalidAccountData)?;
        let quantized_value =
            i128::deserialize(&mut account_data).map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(Self {
            id,
            latest_value: TemporalNumericValue {
                timestamp_ns,
                quantized_value,
            },
        })
    }

    fn get_latest_canonical_temporal_numeric_value_unchecked(
        &self,
        feed_id: &[u8; 32],
    ) -> Result<&TemporalNumericValue> {
        if self.id != *feed_id {
            return Err(ProgramError::InvalidAccountData.into());
        }

        Ok(&self.latest_value)
    }
}

#[derive(Accounts)]
pub struct ReadStork {}

/// Remaining accounts:
/// - temporal numeric value feed
pub fn handler(ctx: Context<ReadStork>, feed_id: [u8; 32]) -> Result<Price> {
    let [temporal_numeric_value_feed, ..] = ctx.remaining_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // validate account owner and PDA
    if temporal_numeric_value_feed.owner != &STORK_PROGRAM_ID {
        return Err(ProgramError::InvalidAccountOwner.into());
    }
    let expected_feed =
        Pubkey::find_program_address(&[STORK_FEED_SEED, feed_id.as_ref()], &STORK_PROGRAM_ID).0;
    if temporal_numeric_value_feed.key != &expected_feed {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // validate account discriminator and state
    let data = temporal_numeric_value_feed.try_borrow_data()?;
    let feed = TemporalNumericValueFeed::decode(&data)?;
    let latest_value = feed.get_latest_canonical_temporal_numeric_value_unchecked(&feed_id)?;

    // validate staleness
    let timestamp_seconds = i64::try_from(latest_value.timestamp_ns / NANOSECONDS_PER_SECOND)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    ensure_fresh_ts(&Clock::get()?, timestamp_seconds)?;

    Price::scale_decimals(latest_value.quantized_value, STORK_QUANTIZED_VALUE_DECIMALS)
}
