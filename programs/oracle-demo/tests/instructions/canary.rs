use oracle_demo::instruction;
use solana_instruction::AccountMeta;
use solana_pubkey::{pubkey, Pubkey};

use crate::common::{format_ui_price, TestContext, TestResult};

const CANARY_SOL_USD_PRICE_FEED: Pubkey = pubkey!("FoX5A6NuS8W5RJkoUEsPJLiFxnvj2woW1BAPJKVn6ThJ");

#[test]
fn reads_canary_price() -> TestResult<()> {
    let context = TestContext::new()?;
    let price = context.invoke(
        instruction::ReadCanary {},
        vec![AccountMeta::new_readonly(CANARY_SOL_USD_PRICE_FEED, false)],
    )?;
    assert!(price > 0);
    println!("Canary SOL/USD price: ${}", format_ui_price(price));
    Ok(())
}
