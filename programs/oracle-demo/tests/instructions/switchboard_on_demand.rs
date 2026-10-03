use oracle_demo::instruction;
use solana_instruction::AccountMeta;
use solana_pubkey::{pubkey, Pubkey};

use crate::common::{feed_id_hex, format_ui_price, TestContext, TestResult};

const SWITCHBOARD_SOL_USD_PULL_FEED: Pubkey =
    pubkey!("5mXfTYitRFsWPhdJfp2fc8N6hK8cw6NB5jAYpronQasj");
const SWITCHBOARD_SOL_USD_PULL_FEED_ID: &str =
    "7cd6d6b567ffe7293d7f679d4ebf2f185707ad52e366377392719795ff5a3303";

#[test]
fn reads_switchboard_on_demand_price() -> TestResult<()> {
    let context = TestContext::new()?;
    let price = context.invoke(
        instruction::ReadSwitchboardOnDemand {
            feed_id: feed_id_hex(SWITCHBOARD_SOL_USD_PULL_FEED_ID)?,
        },
        vec![AccountMeta::new_readonly(
            SWITCHBOARD_SOL_USD_PULL_FEED,
            false,
        )],
    )?;
    assert!(price > 0);
    println!(
        "Switchboard On-Demand SOL/USD price: ${}",
        format_ui_price(price)
    );
    Ok(())
}
