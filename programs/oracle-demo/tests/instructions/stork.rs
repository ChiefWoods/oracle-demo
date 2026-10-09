use oracle_demo::instruction;
use solana_instruction::AccountMeta;
use solana_pubkey::{pubkey, Pubkey};

use crate::common::{feed_id_hex, format_ui_price, TestContext, TestResult};

const STORK_PROGRAM_ID: Pubkey = pubkey!("stork1JUZMKYgjNagHiK2KdMmb42iTnYe9bYUCDUk8n");
const STORK_SOL_USD_PRICE_FEED: Pubkey = pubkey!("7yDnexYRRvSndPC5aR2cCoNB4ihc2qHGAgvJVx5m2Aj5");
const STORK_SOL_USD_FEED_ID: &str =
    "1dcd89dfded9e8a9b0fa1745a8ebbacbb7c81e33d5abc81616633206d932e837";

#[test]
fn reads_stork_price() -> TestResult<()> {
    let feed_id = feed_id_hex(STORK_SOL_USD_FEED_ID)?;
    let (feed, _) = Pubkey::find_program_address(&[b"stork_feed", &feed_id], &STORK_PROGRAM_ID);
    assert_eq!(feed, STORK_SOL_USD_PRICE_FEED);

    let context = TestContext::new()?;
    let price = context.invoke(
        instruction::ReadStork { feed_id },
        vec![AccountMeta::new_readonly(feed, false)],
    )?;
    assert!(price > 0);
    println!("Stork SOL/USD price: ${}", format_ui_price(price));
    Ok(())
}
