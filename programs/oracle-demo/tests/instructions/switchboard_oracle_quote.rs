use oracle_demo::instruction;
use solana_instruction::AccountMeta;
use solana_pubkey::{pubkey, Pubkey};

use crate::common::{feed_id_hex, format_ui_price, TestContext, TestResult};

const SWITCHBOARD_SOL_USD_ORACLE_QUOTE: Pubkey =
    pubkey!("CjvtRZJ13cufRYDda7Fej1JCrc9wuY87Ks72gfNH4YQJ");
const SWITCHBOARD_SOL_USD_ORACLE_QUOTE_FEED_ID: &str =
    "822512ee9add93518eca1c105a38422841a76c590db079eebb283deb2c14caa9";

#[test]
fn reads_switchboard_oracle_quote() -> TestResult<()> {
    let context = TestContext::new()?;
    let price = context.invoke(
        instruction::ReadSwitchboardOracleQuote {
            feed_id: feed_id_hex(SWITCHBOARD_SOL_USD_ORACLE_QUOTE_FEED_ID)?,
        },
        vec![AccountMeta::new_readonly(
            SWITCHBOARD_SOL_USD_ORACLE_QUOTE,
            false,
        )],
    )?;
    assert!(price > 0);
    println!(
        "Switchboard Oracle Quote SOL/USD price: ${}",
        format_ui_price(price)
    );
    Ok(())
}
