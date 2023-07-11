use alloc::string::ToString;

use crate::hgd::hgd;
use crate::prng::{BlockCipher, Prng};
use crate::utils::{hmac, sha};
use crate::{OpeError, OpeKey};

struct OpeDomainRange
{
	d: u64,
	r_lo: u64,
	r_hi: u64,
}

impl OpeDomainRange
{
	pub fn new(d: u64, r_lo: u64, r_hi: u64) -> Self
	{
		Self {
			d,
			r_lo,
			r_hi,
		}
	}
}

fn domain_gap(ndomain: u64, nrange: u64, rgap: u64, prng: &mut impl Prng) -> Result<u64, OpeError>
{
	hgd(rgap, ndomain, nrange - ndomain, prng)
}

pub struct Ope<'a>
{
	key: &'a OpeKey,
	pbits: usize,
	cbits: usize,
}

impl<'a> Ope<'a>
{
	pub fn new(key: &'a OpeKey, plainbits: usize, cipherbits: usize) -> Self
	{
		Self {
			key,
			pbits: plainbits,
			cbits: cipherbits,
		}
	}

	fn lazy_sample(&self, d_lo: u64, d_hi: u64, r_lo: u64, r_hi: u64, plaintext: u64, prng: &mut BlockCipher) -> Result<OpeDomainRange, OpeError>
	{
		let ndomain = d_hi - d_lo + 1;
		let nrange = r_hi - r_lo + 1;

		if nrange >= ndomain {
			return Err(OpeError::OpeRange);
		}

		if ndomain == 1 {
			return Ok(OpeDomainRange::new(d_lo, r_lo, r_hi));
		}

		/*
		 * Deterministically reset the PRNG counter, regardless of
		 * whether we had to use it for HGD or not in previous round.
		 */
		let s = d_lo.to_string() + "/" + &d_hi.to_string() + "/" + &r_lo.to_string() + "/" + &r_hi.to_string();
		let v = hmac(s.as_bytes(), self.key);
		prng.set_ctr(v);

		let rgap = nrange / 2;

		//TODO make cache
		let dgap = domain_gap(ndomain, nrange, nrange / 2, prng)?;

		if plaintext < (d_lo + dgap) {
			self.lazy_sample(d_lo, d_lo + dgap - 1, r_lo, r_lo - 1, plaintext, prng)
		} else {
			self.lazy_sample(d_lo + dgap, d_hi, r_lo + rgap, r_hi, plaintext, prng)
		}
	}

	fn search(&self, plaintext: u64) -> Result<OpeDomainRange, OpeError>
	{
		self.lazy_sample(
			0,
			1 << self.pbits,
			0,
			1 << self.cbits,
			plaintext,
			&mut BlockCipher::new(self.key),
		)
	}

	pub fn encrypt(&self, ptext: u64) -> Result<u64, OpeError>
	{
		let dr = self.search(ptext)?;

		let v = sha(ptext.to_string().as_bytes());

		let mut aesrand = BlockCipher::new(self.key);
		aesrand.set_ctr(v);

		let nrange = dr.r_hi - dr.r_lo + 1;

		Ok(dr.r_lo + aesrand.rand_int_mod(nrange as usize))
	}

	pub fn decrypt(&self, ctext: u64) -> Result<u64, OpeError>
	{
		let dr = self.search(ctext)?;

		Ok(dr.d)
	}
}
