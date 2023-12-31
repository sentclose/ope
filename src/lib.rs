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

const DOMAIN: u64 = u16::max_value() as u64 - 1;
const RANGE: u64 = u32::max_value() as u64 - 1;

pub type OpeKey = [u8; BLOCK_SIZE];

pub fn get_ope(key: &OpeKey) -> Ope
{
	Ope::new(key, DOMAIN, RANGE)
}

#[cfg(test)]
mod tests
{
	use alloc::borrow::ToOwned;
	use alloc::vec::Vec;

	use super::*;

	const MAX_SIZE: u64 = 65531;

	#[test]
	#[ignore]
	fn test_all_numbers()
	{
		let k = b"this is a key 10".to_owned();

		let ope = get_ope(&k);

		let mut v = Vec::with_capacity(MAX_SIZE as usize);

		//encrypt
		for i in 23..MAX_SIZE {
			let en = ope.encrypt(i).unwrap();
			v.push(en);
		}

		//check
		let mut past_item = 0;

		for item in v {
			assert!(past_item < item);

			past_item = item;
		}
	}
}
