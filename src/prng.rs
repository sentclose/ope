use alloc::vec;

use aes::cipher::{KeyIvInit, StreamCipher};

use crate::utils::{big_int_from_bytes, num_bits};
use crate::OpeKey;

type Aes128Ctr64LE = ctr::Ctr64LE<aes::Aes128>;

pub(crate) trait Prng
{
	fn rand_bytes(&mut self, n_bytes: usize, buf: &mut [u8]);

	fn rand_int_mod(&mut self, max: usize) -> u64
	{
		let mut buf = vec![0u8; num_bits(max as u64)];

		self.rand_bytes(buf.len(), &mut buf);

		big_int_from_bytes(&buf) % max as u64
	}
}

pub(crate) const BLOCK_SIZE: usize = 16;

pub(crate) struct BlockCipher
{
	ctr: [u8; BLOCK_SIZE],
	cipher: Aes128Ctr64LE,
}

impl BlockCipher
{
	pub fn new(key: &OpeKey) -> Self
	{
		let cipher = Aes128Ctr64LE::new(key.into(), &[0u8; 16].into());

		Self {
			ctr: [0u8; BLOCK_SIZE],
			cipher,
		}
	}

	pub fn set_ctr(&mut self, v: [u8; BLOCK_SIZE])
	{
		self.ctr = v;
	}
}

impl Prng for BlockCipher
{
	fn rand_bytes(&mut self, n_bytes: usize, buf: &mut [u8])
	{
		for i in (0..n_bytes).step_by(BLOCK_SIZE) {
			for j in 0..BLOCK_SIZE {
				self.ctr[j] += 1;
				if self.ctr[j] != 0 {
					break;
				}
			}

			let mut ct = [0u8; BLOCK_SIZE]; //cipher text; ctr is the plaintext
			ct.copy_from_slice(&self.ctr);
			//TODO aes encrypt in open ssl, self.ctr = ptext; ct = ctext

			self.cipher.apply_keystream(&mut ct);

			let copy_len = usize::min(BLOCK_SIZE, n_bytes - i);
			buf[i..i + copy_len].copy_from_slice(&ct[..copy_len])
		}
	}
}
