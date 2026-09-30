use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program::{get_return_data, invoke},
    },
};
use chainlink_data_streams_report::report::v3::ReportDataV3;
use chainlink_solana_data_streams::discriminator::VERIFY;

use crate::{
    constants::MAX_PAYLOAD_LEN, error::OracleDemoError, instructions::ensure_fresh_ts, price::Price,
};

const REPORT_V3_SCHEMA: [u8; 2] = [0, 3];

#[derive(Accounts)]
pub struct ReadChainlinkDataStreams {}

/// Remaining accounts:
/// - verifier_account
/// - access_controller
/// - user
/// - report_config
/// - verifier_program
pub fn handler(
    ctx: Context<ReadChainlinkDataStreams>,
    feed_id: [u8; 32],
    decimals: u8,
    signed_report: Vec<u8>,
) -> Result<Price> {
    // validate instruction data
    if signed_report.len() > MAX_PAYLOAD_LEN as usize {
        return Err(ProgramError::InvalidInstructionData.into());
    }

    let [verifier_account, access_controller, user, report_config, verifier_program, ..] =
        ctx.remaining_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    let mut data = Vec::with_capacity(VERIFY.len() + 4 + signed_report.len());
    data.extend_from_slice(&VERIFY);
    // instruction data arg: https://github.com/smartcontractkit/chainlink-data-streams-solana/blob/main/crates/chainlink-solana-data-streams/src/lib.rs
    data.extend_from_slice(&(signed_report.len() as u32).to_le_bytes());
    data.extend_from_slice(&signed_report);

    // instruction accounts: https://github.com/smartcontractkit/chainlink-data-streams-solana/blob/main/crates/chainlink-solana-data-streams/src/lib.rs
    let instruction = Instruction {
        program_id: *verifier_program.key,
        accounts: vec![
            AccountMeta::new_readonly(*verifier_account.key, false),
            AccountMeta::new_readonly(*access_controller.key, false),
            AccountMeta::new_readonly(*user.key, true),
            AccountMeta::new_readonly(*report_config.key, false),
        ],
        data,
    };
    invoke(
        &instruction,
        &[
            verifier_account.clone(),
            access_controller.clone(),
            user.clone(),
            report_config.clone(),
        ],
    )?;

    let (program_id, report) = get_return_data().ok_or(ProgramError::InvalidAccountData)?;

    // validate report schema and state
    if program_id != *verifier_program.key || report.get(..2) != Some(REPORT_V3_SCHEMA.as_slice()) {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let report = ReportDataV3::decode(&report).map_err(|_| ProgramError::InvalidAccountData)?;

    // validate feed id
    if report.feed_id.0 != feed_id {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // validate staleness
    ensure_fresh_ts(&Clock::get()?, i64::from(report.observations_timestamp))?;

    let benchmark_price =
        i128::try_from(&report.benchmark_price).map_err(|_| OracleDemoError::InvalidPrice)?;
    Price::scale_decimals(benchmark_price, decimals)
}
