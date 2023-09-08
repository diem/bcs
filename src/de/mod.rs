// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, Result};

use std::{
    convert::TryFrom,
    ops::{AddAssign, SubAssign},
};

mod from_bytes;
pub use from_bytes::*;
mod from_reader;

trait BcsDeserializer {
    fn max_remaining_depth(&mut self) -> usize;
    fn max_remaining_depth_mut(&mut self) -> &mut usize;

    fn parse_bool(&mut self) -> Result<bool> {
        let byte = self.next()?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::ExpectedBoolean),
        }
    }

    fn fill_slice(&mut self, slice: &mut [u8]) -> Result<()>;

    fn next(&mut self) -> Result<u8> {
        let mut byte = [0u8; 1];
        self.fill_slice(&mut byte)?;
        Ok(byte[0])
    }

    fn parse_u8(&mut self) -> Result<u8> {
        self.next()
    }

    fn parse_u16(&mut self) -> Result<u16> {
        let mut le_bytes = [0; 2];
        self.fill_slice(&mut le_bytes)?;
        Ok(u16::from_le_bytes(le_bytes))
    }

    fn parse_u32(&mut self) -> Result<u32> {
        let mut le_bytes = [0; 4];
        self.fill_slice(&mut le_bytes)?;
        Ok(u32::from_le_bytes(le_bytes))
    }

    fn parse_u64(&mut self) -> Result<u64> {
        let mut le_bytes = [0; 8];
        self.fill_slice(&mut le_bytes)?;
        Ok(u64::from_le_bytes(le_bytes))
    }

    fn parse_u128(&mut self) -> Result<u128> {
        let mut le_bytes = [0; 16];
        self.fill_slice(&mut le_bytes)?;
        Ok(u128::from_le_bytes(le_bytes))
    }

    #[allow(clippy::integer_arithmetic)]
    fn parse_u32_from_uleb128(&mut self) -> Result<u32> {
        let mut value: u64 = 0;
        for shift in (0..32).step_by(7) {
            let byte = self.next()?;
            let digit = byte & 0x7f;
            value |= u64::from(digit) << shift;
            // If the highest bit of `byte` is 0, return the final value.
            if digit == byte {
                if shift > 0 && digit == 0 {
                    // We only accept canonical ULEB128 encodings, therefore the
                    // heaviest (and last) base-128 digit must be non-zero.
                    return Err(Error::NonCanonicalUleb128Encoding);
                }
                // Decoded integer must not overflow.
                return u32::try_from(value)
                    .map_err(|_| Error::IntegerOverflowDuringUleb128Decoding);
            }
        }
        // Decoded integer must not overflow.
        Err(Error::IntegerOverflowDuringUleb128Decoding)
    }

    fn parse_length(&mut self) -> Result<usize> {
        let len = self.parse_u32_from_uleb128()? as usize;
        if len > crate::MAX_SEQUENCE_LENGTH {
            return Err(Error::ExceededMaxLen(len));
        }
        Ok(len)
    }

    fn enter_named_container(&mut self, name: &'static str) -> Result<()> {
        if self.max_remaining_depth() == 0 {
            return Err(Error::ExceededContainerDepthLimit(name));
        }
        self.max_remaining_depth_mut().sub_assign(1);
        Ok(())
    }

    fn leave_named_container(&mut self) {
        self.max_remaining_depth_mut().add_assign(1);
    }
}
