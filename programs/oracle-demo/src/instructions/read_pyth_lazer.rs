use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program::{get_return_data, invoke},
    },
};
use pyth_lazer_protocol::payload::{PayloadData, PayloadPropertyValue};

use crate::{
    constants::MAX_PAYLOAD_LEN, error::OracleDemoError, instructions::ensure_fresh_ts, price::Price,
};

const VERIFY_MESSAGE_DISCRIMINATOR: [u8; 8] = [180, 193, 120, 55, 189, 135, 203, 83];

#[derive(AnchorDeserialize)]
struct VerifiedMessage {
    _public_key: Pubkey,
    payload: Vec<u8>,
}

#[derive(Accounts)]
pub struct ReadPythLazer {}

/// Remaining accounts:
/// - payer
/// - storage
/// - treasury
/// - system_program
/// - instructions_sysvar
/// - lazer_program
pub fn handler(
    ctx: Context<ReadPythLazer>,
    feed_id: u32,
    ed25519_instruction_index: u16,
    signature_index: u8,
    message: Vec<u8>,
) -> Result<Price> {
    if message.len() > MAX_PAYLOAD_LEN as usize {
        return Err(ProgramError::InvalidInstructionData.into());
    }

    let [payer, storage, treasury, system_program, instructions_sysvar, lazer_program, ..] =
        ctx.remaining_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // 8 - discriminator
    // 4 - message vec header
    // n - message bytes
    // 2 - ed25519 instruction index
    // 1 - signature index
    let mut data = Vec::with_capacity(8 + 4 + message.len() + 2 + 1);
    data.extend_from_slice(&VERIFY_MESSAGE_DISCRIMINATOR);
    // instruction data arg: https://docs.rs/pyth-lazer-solana-contract/latest/pyth_lazer_solana_contract/instruction/struct.VerifyMessage.html
    data.extend_from_slice(
        &u32::try_from(message.len())
            .map_err(|_| ProgramError::InvalidInstructionData)?
            .to_le_bytes(),
    );
    data.extend_from_slice(&message);
    data.extend_from_slice(&ed25519_instruction_index.to_le_bytes());
    data.push(signature_index);

    // instruction accounts: https://docs.rs/pyth-lazer-solana-contract/latest/pyth_lazer_solana_contract/accounts/struct.VerifyMessage.html
    let instruction = Instruction {
        program_id: *lazer_program.key,
        accounts: vec![
            AccountMeta::new(*payer.key, true),
            AccountMeta::new_readonly(*storage.key, false),
            AccountMeta::new(*treasury.key, false),
            AccountMeta::new_readonly(*system_program.key, false),
            AccountMeta::new_readonly(*instructions_sysvar.key, false),
        ],
        data,
    };
    invoke(
        &instruction,
        &[
            payer.clone(),
            storage.clone(),
            treasury.clone(),
            system_program.clone(),
            instructions_sysvar.clone(),
        ],
    )?;

    let (program_id, return_data) = get_return_data().ok_or(ProgramError::InvalidAccountData)?;
    if program_id != *lazer_program.key {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // validate report
    // payload layout: https://docs.rs/pyth-lazer-sdk/latest/pyth_lazer_sdk/struct.VerifiedMessage.html
    // 32 - public_key
    // 4 - payload vec header
    // n - payload bytes
    let verified = VerifiedMessage::try_from_slice(&return_data)?;
    let payload = PayloadData::deserialize_slice_le(&verified.payload)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // validate staleness
    ensure_fresh_ts(
        &Clock::get()?,
        i64::try_from(payload.timestamp_us.as_secs())
            .map_err(|_| ProgramError::InvalidAccountData)?,
    )?;

    let feed = payload
        .feeds
        .iter()
        .find(|feed| feed.feed_id.0 == feed_id)
        .ok_or(ProgramError::InvalidAccountData)?;

    let mut price = None;
    let mut exponent = None;
    for property in &feed.properties {
        match property {
            PayloadPropertyValue::Price(Some(value)) => price = Some(value.mantissa_i64()),
            PayloadPropertyValue::Exponent(value) => exponent = Some(*value),
            _ => {}
        }
    }

    Price::scale(
        i128::from(price.ok_or(OracleDemoError::MissingPrice)?),
        i32::from(exponent.ok_or(OracleDemoError::MissingPrice)?),
    )
}
