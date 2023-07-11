#![no_std]
#![doc = include_str!("../README.md")]

extern crate alloc;

mod hgd;
mod ope;
mod prng;
mod utils;

use num_bigint::BigInt as ZZ;

use crate::ope::Ope;
use crate::prng::BLOCK_SIZE;

#[derive(Debug)]
pub enum OpeError
{
	HdgInvalidInputs,
	OpeRange,
}

// const P_BITS: usize = 32;
// const C_BITS: usize = 64;

const DOMAIN: usize = u16::max_value() as usize - 1;
const RANGE: usize = u32::max_value() as usize - 1;

pub type OpeKey = [u8; BLOCK_SIZE];

pub fn get_ope(key: &OpeKey) -> Ope
{
	Ope::new(key, DOMAIN, RANGE)
}
