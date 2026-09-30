use anchor_lang::prelude::*;

#[constant]
pub const SCALE: u64 = 1_000_000_000_000u64;
#[constant]
pub const MAX_TS_STALENESS: u64 = 30;
#[constant]
pub const MAX_SLOT_STALENESS: u8 = 10;
#[constant]
pub const MAX_PAYLOAD_LEN: u16 = 1024;
