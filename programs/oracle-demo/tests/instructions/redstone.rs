use oracle_demo::instruction;
use solana_instruction::AccountMeta;
use solana_pubkey::{pubkey, Pubkey};

use crate::common::{feed_id_hex, format_ui_price, TestContext, TestResult};

const REDSTONE_SOL_USD_PRICE_FEED: Pubkey = pubkey!("DsCuXFfDqGga1KcATNt1aqDGndoUnCHEvAeRAFHvb7yp");
const REDSTONE_SOL_USD_FEED_ID: &str =
    "534f4c0000000000000000000000000000000000000000000000000000000000";

#[test]
fn reads_redstone_price() -> TestResult<()> {
    let context = TestContext::new()?;
    let price = context.invoke(
        instruction::ReadRedstone {
            feed_id: feed_id_hex(REDSTONE_SOL_USD_FEED_ID)?,
        },
        vec![AccountMeta::new_readonly(
            REDSTONE_SOL_USD_PRICE_FEED,
            false,
        )],
    )?;
    assert!(price > 0);
    println!("RedStone SOL/USD price: ${}", format_ui_price(price));
    Ok(())
}
