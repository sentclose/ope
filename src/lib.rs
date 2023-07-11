#![no_std]

extern crate alloc;

mod hgd;
mod ope;
mod prng;
mod utils;

use num_bigint::BigInt as ZZ;

use crate::ope::Ope;
use crate::prng::BLOCK_SIZE;

pub enum OpeError
{
	HdgInvalidInputs,
	OpeRange,
}

const P_BITS: usize = 32;
const C_BITS: usize = 64;

pub type OpeKey = [u8; BLOCK_SIZE];

pub fn get_ope(key: &OpeKey) -> Ope
{
	Ope::new(key, P_BITS, C_BITS)
}
