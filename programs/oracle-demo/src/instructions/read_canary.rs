use anchor_lang::prelude::*;
use borsh::BorshDeserialize;

use crate::{instructions::ensure_fresh_ts, price::Price};

const CANARY_PROGRAM_ID: Pubkey = pubkey!("CanarFxHDSnbrPmrE79Qq6hL2p7ZMyyV4ZLTKQ6g7tpK");
const PRICE_FEED_DISCRIMINATOR: [u8; 8] = [189, 103, 252, 23, 152, 35, 243, 156];
const FEED_STATUS_LIVE: u8 = 1;

#[derive(BorshDeserialize)]
pub struct PriceFeed {
    pub value: [u8; 16],
    pub unix_timestamp: u64,
    pub signer_group: Pubkey,
    pub entry_id: u16,
    pub exp: u8,
    pub status: u8,
    pub bump: u8,
    _reserved: [u8; 131],
}

impl PriceFeed {
    const DATA_LEN: usize = 192;
    pub const ACCOUNT_LEN: usize = 8 + Self::DATA_LEN;

    fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < Self::ACCOUNT_LEN
            || data.get(..8) != Some(PRICE_FEED_DISCRIMINATOR.as_slice())
        {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let mut account_data = data.get(8..).ok_or(ProgramError::InvalidAccountData)?;
        let feed =
            Self::deserialize(&mut account_data).map_err(|_| ProgramError::InvalidAccountData)?;

        if feed.status != FEED_STATUS_LIVE {
            return Err(ProgramError::InvalidAccountData.into());
        }

        Ok(feed)
    }

    pub fn value(&self) -> u128 {
        u128::from_le_bytes(self.value)
    }
}

#[derive(Accounts)]
pub struct ReadCanary {}

/// Remaining accounts:
/// - price feed
pub fn handler(ctx: Context<ReadCanary>) -> Result<Price> {
    let [price_feed, ..] = ctx.remaining_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // validate account owner
    if price_feed.owner != &CANARY_PROGRAM_ID {
        return Err(ProgramError::InvalidAccountOwner.into());
    }

    // validate account discriminator and state
    let data = price_feed.try_borrow_data()?;
    let feed = PriceFeed::decode(&data)?;

    // validate staleness
    ensure_fresh_ts(
        &Clock::get()?,
        i64::try_from(feed.unix_timestamp).map_err(|_| ProgramError::InvalidAccountData)?,
    )?;

    Price::scale_u128(feed.value(), -i32::from(feed.exp))
}
