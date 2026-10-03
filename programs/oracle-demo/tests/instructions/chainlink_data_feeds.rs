use oracle_demo::instruction;
use solana_instruction::AccountMeta;
use solana_pubkey::{pubkey, Pubkey};

use crate::common::{format_ui_price, TestContext, TestResult};

const CHAINLINK_SOL_USD_FEED: Pubkey = pubkey!("99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR");

#[test]
fn reads_chainlink_data_feeds() -> TestResult<()> {
    let context = TestContext::new()?;
    let price = context.invoke(
        instruction::ReadChainlinkDataFeeds {
            feed_address: CHAINLINK_SOL_USD_FEED,
        },
        vec![AccountMeta::new_readonly(CHAINLINK_SOL_USD_FEED, false)],
    )?;
    assert!(price > 0);
    println!(
        "Chainlink Data Feed SOL/USD price: ${}",
        format_ui_price(price)
    );
    Ok(())
}
