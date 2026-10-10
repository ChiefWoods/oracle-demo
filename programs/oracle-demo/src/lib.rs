pub mod constants;
pub mod error;
pub mod instructions;
pub mod price;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use price::*;

declare_id!("6RkFbpc1VcpMGJjxzEyDxTN2Xr5THdeegYUBxNf6vMoc");

#[program]
pub mod oracle_demo {
    use super::*;

    pub fn read_canary(ctx: Context<ReadCanary>) -> Result<Price> {
        instructions::read_canary::handler(ctx)
    }

    pub fn read_pyth_core(ctx: Context<ReadPythCore>, feed_id: [u8; 32]) -> Result<Price> {
        instructions::read_pyth_core::handler(ctx, feed_id)
    }

    pub fn read_pyth_lazer(
        ctx: Context<ReadPythLazer>,
        feed_id: u32,
        ed25519_instruction_index: u16,
        signature_index: u8,
        message: Vec<u8>,
    ) -> Result<Price> {
        instructions::read_pyth_lazer::handler(
            ctx,
            feed_id,
            ed25519_instruction_index,
            signature_index,
            message,
        )
    }

    pub fn read_chainlink_data_feeds(
        ctx: Context<ReadChainlinkDataFeeds>,
        feed_address: Pubkey,
    ) -> Result<Price> {
        instructions::read_chainlink_data_feeds::handler(ctx, feed_address)
    }

    pub fn read_chainlink_data_streams(
        ctx: Context<ReadChainlinkDataStreams>,
        feed_id: [u8; 32],
        decimals: u8,
        signed_report: Vec<u8>,
    ) -> Result<Price> {
        instructions::read_chainlink_data_streams::handler(ctx, feed_id, decimals, signed_report)
    }

    pub fn read_redstone(ctx: Context<ReadRedstone>, feed_id: [u8; 32]) -> Result<Price> {
        instructions::read_redstone::handler(ctx, feed_id)
    }

    pub fn read_stork(ctx: Context<ReadStork>, feed_id: [u8; 32]) -> Result<Price> {
        instructions::read_stork::handler(ctx, feed_id)
    }

    pub fn read_switchboard_on_demand(
        ctx: Context<ReadSwitchboardOnDemand>,
        feed_id: [u8; 32],
    ) -> Result<Price> {
        instructions::read_switchboard_on_demand::handler(ctx, feed_id)
    }

    pub fn read_switchboard_oracle_quote(
        ctx: Context<ReadSwitchboardOracleQuote>,
        feed_id: [u8; 32],
    ) -> Result<Price> {
        instructions::read_switchboard_oracle_quote::handler(ctx, feed_id)
    }
}
