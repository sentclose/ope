use hmac::{Hmac, Mac};
use num_traits::ToPrimitive;
use sha2::{Digest, Sha256};

use crate::prng::BLOCK_SIZE;
use crate::ZZ;

type HmacSha256 = Hmac<Sha256>;

pub(crate) fn big_int_from_bytes(bytes: &[u8]) -> u64
{
	ZZ::from_bytes_be(num_bigint::Sign::Plus, bytes)
		.to_u64()
		.unwrap()
}

// fn num_bits(n: &ZZ) -> u32
// {
// 	if n.is_zero() {
// 		return 0;
// 	}
//
// 	let mut bits = 0;
// 	let mut temp = n.bits();
//
// 	while !temp.is_zero() {
// 		bits += 1;
// 		temp >>= 1u32;
// 	}
//
// 	bits
// }
//
// fn num_bits3(n: &ZZ) -> usize
// {
// 	let bits = n.bits();
// 	bits.to_usize().unwrap()
// }

pub(crate) fn num_bits(mut n: u64) -> usize
{
	let mut bits = 0;

	while n > 0 {
		bits += 1;
		n >>= 1;
	}

	bits
}

pub(crate) fn hmac(v: &[u8], key: &[u8]) -> [u8; BLOCK_SIZE]
{
	let mut mac = HmacSha256::new_from_slice(key).unwrap();
	mac.update(v);
	let result = mac.finalize();

	let mut out = [0u8; BLOCK_SIZE];

	out.copy_from_slice(&result.into_bytes()[..BLOCK_SIZE]);

	out
}

pub(crate) fn sha(v: &[u8]) -> [u8; BLOCK_SIZE]
{
	let mut hasher = Sha256::new();
	hasher.update(v);
	let result = hasher.finalize();

	let mut out = [0u8; BLOCK_SIZE];

	out.copy_from_slice(&result[..BLOCK_SIZE]);

	out
}
