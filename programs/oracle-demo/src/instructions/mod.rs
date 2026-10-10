pub mod read_canary;
pub mod read_chainlink_data_feeds;
pub mod read_chainlink_data_streams;
pub mod read_pyth_core;
pub mod read_pyth_lazer;
pub mod read_redstone;
pub mod read_stork;
pub mod read_switchboard_on_demand;
pub mod read_switchboard_oracle_quote;

#[allow(ambiguous_glob_reexports)]
pub use read_canary::*;
pub use read_chainlink_data_feeds::*;
pub use read_chainlink_data_streams::*;
pub use read_pyth_core::*;
pub use read_pyth_lazer::*;
pub use read_redstone::*;
pub use read_stork::*;
pub use read_switchboard_on_demand::*;
pub use read_switchboard_oracle_quote::*;

use anchor_lang::prelude::*;

#[cfg(not(feature = "disable-staleness-check"))]
use crate::{
    constants::{MAX_SLOT_STALENESS, MAX_TS_STALENESS},
    error::OracleDemoError,
};

#[cfg(not(feature = "disable-staleness-check"))]
pub(crate) fn ensure_fresh_slot(clock: &Clock, slot: u64) -> Result<()> {
    if clock.slot.saturating_sub(slot) > u64::from(MAX_SLOT_STALENESS) {
        err!(OracleDemoError::StalePrice)
    } else {
        Ok(())
    }
}

#[cfg(feature = "disable-staleness-check")]
pub(crate) fn ensure_fresh_slot(_: &Clock, _: u64) -> Result<()> {
    Ok(())
}

#[cfg(not(feature = "disable-staleness-check"))]
pub(crate) fn ensure_fresh_ts(clock: &Clock, unix_ts: i64) -> Result<()> {
    match u64::try_from(clock.unix_timestamp.saturating_sub(unix_ts)) {
        Ok(age) if age > MAX_TS_STALENESS => err!(OracleDemoError::StalePrice),
        _ => Ok(()),
    }
}

#[cfg(feature = "disable-staleness-check")]
pub(crate) fn ensure_fresh_ts(_: &Clock, _: i64) -> Result<()> {
    Ok(())
}
