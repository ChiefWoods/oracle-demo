use anchor_lang::prelude::*;
use borsh::BorshDeserialize;

use crate::{error::OracleDemoError, instructions::ensure_fresh_slot, price::Price};

const REDSTONE_PROGRAM_ID: Pubkey = pubkey!("REDSTBDUecGjwXd6YGPzHSvEUBHQqVRfCcjUVgPiHsr");
const PRICE_DATA_DISCRIMINATOR: [u8; 8] = [232, 113, 193, 231, 133, 209, 206, 154];

/// https://docs.redstone.finance/docs/technical-reference/non-evm-chains/solana/price-feed-account/
#[derive(BorshDeserialize)]
struct PriceData {
    feed_id: [u8; 32],
    value: [u8; 32],
    _timestamp: u64,
    _write_timestamp: Option<u64>,
    update_slot: u64,
    decimals: u8,
    _reserved: [u8; 64],
}

impl PriceData {
    fn decode(data: &[u8]) -> Result<Self> {
        if data.get(..8) != Some(PRICE_DATA_DISCRIMINATOR.as_slice()) {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let mut account_data = data.get(8..).ok_or(ProgramError::InvalidAccountData)?;
        Self::deserialize(&mut account_data).map_err(|_| ProgramError::InvalidAccountData.into())
    }

    fn value(&self) -> Result<u64> {
        if !self.value.iter().take(24).all(|byte| *byte == 0) {
            return err!(OracleDemoError::ArithmeticOverflow);
        }
        let bytes: [u8; 8] = self.value[24..]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(u64::from_be_bytes(bytes))
    }
}

#[derive(Accounts)]
pub struct ReadRedstone {}

/// Remaining accounts:
/// - price_data
pub fn handler(ctx: Context<ReadRedstone>, feed_id: [u8; 32]) -> Result<Price> {
    let [price_data, ..] = ctx.remaining_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // validate account owner
    if price_data.owner != &REDSTONE_PROGRAM_ID {
        return Err(ProgramError::InvalidAccountOwner.into());
    }

    // validate account discriminator and state
    let data = price_data.try_borrow_data()?;
    let price = PriceData::decode(&data)?;

    // validate feed id
    if price.feed_id != feed_id {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // validate staleness
    ensure_fresh_slot(&Clock::get()?, price.update_slot)?;

    Price::scale_decimals(i128::from(price.value()?), price.decimals)
}
