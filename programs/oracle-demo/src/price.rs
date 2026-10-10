use anchor_lang::prelude::*;

use crate::{constants::SCALE, error::OracleDemoError};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Price(pub u128);

impl Price {
    pub fn scale(value: i128, exponent: i32) -> Result<Self> {
        if value <= 0 {
            return err!(OracleDemoError::InvalidPrice);
        }

        Self::scale_u128(value as u128, exponent)
    }

    pub fn scale_u128(value: u128, exponent: i32) -> Result<Self> {
        if value == 0 {
            return err!(OracleDemoError::InvalidPrice);
        }

        let scaled = value
            .checked_mul(u128::from(SCALE))
            .ok_or(OracleDemoError::ArithmeticOverflow)?;
        let scaled = if exponent >= 0 {
            scaled
                .checked_mul(Self::pow10(exponent as u32)?)
                .ok_or(OracleDemoError::ArithmeticOverflow)?
        } else {
            scaled / Self::pow10(exponent.unsigned_abs())?
        };

        if scaled == 0 {
            return err!(OracleDemoError::InvalidPrice);
        }

        Ok(Self(scaled))
    }

    pub fn scale_decimals(value: i128, decimals: u8) -> Result<Self> {
        Self::scale(value, -i32::from(decimals))
    }

    fn pow10(exponent: u32) -> Result<u128> {
        let mut result = 1u128;
        let mut index = 0;
        while index < exponent {
            result = result
                .checked_mul(10)
                .ok_or(OracleDemoError::ArithmeticOverflow)?;
            index += 1;
        }
        Ok(result)
    }
}
