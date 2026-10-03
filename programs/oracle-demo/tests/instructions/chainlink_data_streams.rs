use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use oracle_demo::instruction;
use reqwest::blocking::Client as HttpClient;
use serde_json::Value;
use sha2::{Digest, Sha256};
use solana_instruction::AccountMeta;
use solana_pubkey::{pubkey, Pubkey};

use crate::common::{feed_id_hex, format_ui_price, required, TestContext, TestResult};

const CHAINLINK_SOL_USD_FEED_ID: &str =
    "0003b778d3f6b2ac4991302b89cb313f99a42467d6c9c5f96f57c29c0d2bc24f";
const CHAINLINK_DATA_STREAMS_DECIMALS: u8 = 18;
const CHAINLINK_DATA_STREAMS_VERIFIER_PROGRAM: Pubkey =
    pubkey!("Gt9S41PtjR58CbG9JhJ3J6vxesqrNAswbWYbLNTMZA3c");
const CHAINLINK_DATA_STREAMS_ACCESS_CONTROLLER: Pubkey =
    pubkey!("2k3DsgwBoqrnvXKVvd7jX7aptNxdcRBdcd5HkYsGgbrb");
const CHAINLINK_DATA_STREAMS_REST_URL: &str = "https://api.testnet-dataengine.chain.link";

#[test]
fn reads_chainlink_data_streams() -> TestResult<()> {
    let context = TestContext::new()?;
    let signed_report = fetch_chainlink_data_streams_report(CHAINLINK_SOL_USD_FEED_ID)?;
    let verifier_program = CHAINLINK_DATA_STREAMS_VERIFIER_PROGRAM;
    let verifier_account = derive_verifier(&verifier_program);
    let report_config = derive_report_config(&signed_report, &verifier_program)?;
    let price = context.invoke(
        instruction::ReadChainlinkDataStreams {
            feed_id: feed_id_hex(CHAINLINK_SOL_USD_FEED_ID)?,
            decimals: CHAINLINK_DATA_STREAMS_DECIMALS,
            signed_report,
        },
        vec![
            AccountMeta::new_readonly(verifier_account, false),
            AccountMeta::new_readonly(CHAINLINK_DATA_STREAMS_ACCESS_CONTROLLER, false),
            AccountMeta::new_readonly(context.payer(), true),
            AccountMeta::new_readonly(report_config, false),
            AccountMeta::new_readonly(verifier_program, false),
        ],
    )?;
    assert!(price > 0);
    println!(
        "Chainlink Data Streams SOL/USD price: ${}",
        format_ui_price(price)
    );
    Ok(())
}

fn derive_verifier(verifier_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"verifier"], verifier_program).0
}

fn derive_report_config(signed_report: &[u8], verifier_program: &Pubkey) -> TestResult<Pubkey> {
    Ok(Pubkey::find_program_address(
        &[signed_report
            .get(..32)
            .ok_or("Chainlink signed report is shorter than its config seed")?],
        verifier_program,
    )
    .0)
}

fn fetch_chainlink_data_streams_report(feed_id: &str) -> TestResult<Vec<u8>> {
    let api_key = required("CHAINLINK_DATA_STREAMS_API_KEY")?;
    let hmac_secret = required("CHAINLINK_DATA_STREAMS_HMAC_SECRET")?;
    let feed_id = feed_id.strip_prefix("0x").unwrap_or(feed_id);
    let path = format!("/api/v1/reports/latest?feedID=0x{feed_id}");
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let body_hash = hex::encode(Sha256::digest([]));
    let string_to_sign = format!("GET {path} {body_hash} {api_key} {timestamp}");
    let mut mac = Hmac::<Sha256>::new_from_slice(hmac_secret.as_bytes())?;
    mac.update(string_to_sign.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    let endpoint = format!("{CHAINLINK_DATA_STREAMS_REST_URL}{path}");
    let response: Value = HttpClient::new()
        .get(endpoint)
        .header("Authorization", api_key)
        .header("X-Authorization-Timestamp", timestamp.to_string())
        .header("X-Authorization-Signature-SHA256", signature)
        .send()?
        .error_for_status()?
        .json()?;
    let signed_report = response
        .pointer("/report/fullReport")
        .and_then(Value::as_str)
        .ok_or("Chainlink response did not include report.fullReport")?;

    Ok(hex::decode(
        signed_report.strip_prefix("0x").unwrap_or(signed_report),
    )?)
}
