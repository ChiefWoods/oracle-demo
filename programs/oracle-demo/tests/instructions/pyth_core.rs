use oracle_demo::instruction;
use solana_instruction::AccountMeta;
use solana_pubkey::{pubkey, Pubkey};

use crate::common::{feed_id_hex, format_ui_price, TestContext, TestResult};

const PYTH_SOL_USD_PRICE_FEED: Pubkey = pubkey!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
const PYTH_SOL_USD_FEED_ID: &str =
    "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

#[test]
fn reads_pyth_core_price() -> TestResult<()> {
    let context = TestContext::new()?;
    let price = context.invoke(
        instruction::ReadPythCore {
            feed_id: feed_id_hex(PYTH_SOL_USD_FEED_ID)?,
        },
        vec![AccountMeta::new_readonly(PYTH_SOL_USD_PRICE_FEED, false)],
    )?;
    assert!(price > 0);
    println!("Pyth Core SOL/USD price: ${}", format_ui_price(price));
    Ok(())
}
