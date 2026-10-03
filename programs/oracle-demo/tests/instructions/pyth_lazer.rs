use oracle_demo::instruction;
use pyth_lazer_solana_contract::{ed25519_program_args, Ed25519SignatureOffsets, STORAGE_ID};
use reqwest::blocking::Client as HttpClient;
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::{pubkey, Pubkey};
use solana_sdk_ids::{ed25519_program, system_program, sysvar};

use crate::common::{format_ui_price, required, TestContext, TestResult};

const PYTH_LAZER_PROGRAM_ID: Pubkey = pubkey!("pytd2yyk641x7ak7mkaasSJVXh6YYZnC7wTmtgAyxPt");
const PYTH_LAZER_TREASURY: Pubkey = pubkey!("opsLibxVY7Vz5eYMmSfX8cLFCFVYTtH6fr6MiifMpA7");
const PYTH_LAZER_SOL_USD_FEED_ID: u32 = 6;
const PYTH_LAZER_MESSAGE_OFFSET: u16 = 19;
const PYTH_LAZER_REST_URL: &str = "https://pyth-lazer.dourolabs.app/v1/latest_price";

#[test]
fn reads_pyth_lazer_price() -> TestResult<()> {
    let context = TestContext::new()?;
    let message = fetch_pyth_lazer_solana_message(PYTH_LAZER_SOL_USD_FEED_ID)?;
    let signature_offsets = Ed25519SignatureOffsets::new(&message, 1, PYTH_LAZER_MESSAGE_OFFSET);
    let ed25519_instruction = Instruction::new_with_bytes(
        ed25519_program::ID,
        &ed25519_program_args(&[signature_offsets]),
        Vec::new(),
    );
    let price = context.invoke_with_prefix(
        vec![ed25519_instruction],
        instruction::ReadPythLazer {
            feed_id: PYTH_LAZER_SOL_USD_FEED_ID,
            ed25519_instruction_index: 0,
            signature_index: 0,
            message,
        },
        vec![
            AccountMeta::new(context.payer(), true),
            AccountMeta::new_readonly(Pubkey::new_from_array(STORAGE_ID.to_bytes()), false),
            AccountMeta::new(PYTH_LAZER_TREASURY, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(sysvar::instructions::ID, false),
            AccountMeta::new_readonly(PYTH_LAZER_PROGRAM_ID, false),
        ],
    )?;
    assert!(price > 0);
    println!("Pyth Lazer SOL/USD price: ${}", format_ui_price(price));
    Ok(())
}

fn fetch_pyth_lazer_solana_message(feed_id: u32) -> TestResult<Vec<u8>> {
    let api_key = required("PYTH_API_KEY")?;
    let request = json!({
        "priceFeedIds": [feed_id],
        "properties": ["price", "exponent", "feedUpdateTimestamp"],
        "formats": ["solana"],
        "jsonBinaryEncoding": "hex",
        "channel": "real_time",
        "ignoreInvalidFeeds": false,
    });

    let response = HttpClient::new()
        .post(PYTH_LAZER_REST_URL)
        .bearer_auth(api_key)
        .json(&request)
        .send()?
        .error_for_status()?
        .json::<Value>()?;

    decode_solana_message(&response)
}

fn decode_solana_message(response: &Value) -> TestResult<Vec<u8>> {
    let data = response
        .pointer("/solana/data")
        .and_then(Value::as_str)
        .ok_or("Pyth Lazer REST response did not include a Solana payload")?;
    Ok(hex::decode(data)?)
}
