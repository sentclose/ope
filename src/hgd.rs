#[allow(unused_imports)]
use num_traits::Float;

use crate::prng::Prng;
use crate::utils::num_bits;
use crate::OpeError;

const CON: f64 = 57.56462733;
const DELTAL: f64 = 0.0078;
const DELTAU: f64 = 0.0034;
const SCALE: f64 = 1.0e25;

/*
 * FUNCTION TO EVALUATE LOGARITHM OF THE FACTORIAL I
 * IF (I .GT. 7), USE STIRLING'S APPROXIMATION
 * OTHERWISE,  USE TABLE LOOKUP
 */
fn afc(i: f64) -> f64
{
	// if lte 7 use computed table
	match i.round() as u64 {
		0 => 0.0,
		1 => 0.0,
		#[allow(clippy::approx_constant)]
		2 => 0.6931471806,
		3 => 1.791759469,
		4 => 3.178053830,
		5 => 4.787491743,
		6 => 6.579251212,
		7 => 8.525161361,
		_ => (i + 0.5) * i.ln() - i + 0.08333333333333 / i - 0.00277777777777 / i / i / i + 0.9189385332,
	}
}

fn rand(prng: &mut impl Prng, precision: usize) -> f64
{
	let div = 1usize << precision;
	let rzz = prng.rand_int_mod(div);

	rzz as f64 / div as f64
}

/*
 * KK is the number of elements drawn from an urn where there are NN1 white
 * balls and NN2 black balls; the result is the number of white balls in
 * the KK sample.
 *
 * The implementation is based on an adaptation of the H2PEC alg for large
 * numbers; see hgd.cc for details
 */
pub(crate) fn hgd(kk: u64, nn1: u64, nn2: u64, prng: &mut impl Prng) -> Result<u64, OpeError>
{
	if kk > (nn1 + nn2) {
		return Err(OpeError::HdgInvalidInputs);
	}

	let precision = num_bits(nn1 + nn2 + kk);

	let (n1, n2) = if nn1 >= nn2 {
		(nn2 as f64, nn1 as f64)
	} else {
		(nn1 as f64, nn2 as f64)
	};

	let tn = n1 + n2;

	let k = if (kk + kk) as f64 >= tn { tn - kk as f64 } else { kk as f64 };

	let m = (k + 1.0) * (n1 + 1.0) / (tn + 2.0);

	let minjx = if (k - n2) < 0.0 { 0.0 } else { k - n2 };
	let maxjx = if n1 < k { n1 } else { k };

	/*
	 * GENERATE RANDOM VARIATE
	 */
	let ix = if minjx == maxjx {
		/*
		 * ...DEGENERATE DISTRIBUTION...
		 */
		maxjx
	} else if (m - minjx) < 10.0 {
		/*
		 * ...INVERSE TRANSFORMATION...
		 * Shouldn't really happen in OPE because M will be on the order of N1.
		 * In practice, this does get invoked.
		 */

		let w = if k < n2 {
			(CON + afc(n2) + afc(n1 + n2 - k) - afc(n2 - k) - afc(n1 + n2)).exp()
		} else {
			(CON + afc(n1) + afc(k) - afc(k - n2) - afc(n1 + n2)).exp()
		};

		let mut ix;
		let mut p;
		let mut u;

		'label10: loop {
			p = w;
			ix = minjx;
			u = rand(prng, precision) * SCALE;

			'label20: loop {
				if u > p {
					u -= p;
					p = p * (n1 - ix) * (k - ix);
					ix += 1.0;
					p = p / ix / (n2 - k + ix);

					if ix > maxjx {
						continue 'label10;
					}
					continue 'label20;
				}
				break 'label10;
			}
		}

		ix
	} else {
		/*
			* ...H2PE...
			*/

		let mut ix;

		let s = ((tn - k) * k * n1 * n2 / (tn - 1.0) / tn / tn).sqrt();

		/*
		 * ...REMARK:  D IS DEFINED IN REFERENCE WITHOUT INT.
		 * THE TRUNCATION CENTERS THE CELL BOUNDARIES AT 0.5
		 */
		let d = (1.5 * s).trunc() + 0.5;

		let xl = (m - d + 0.5).trunc();
		let xr = (m + d + 0.5).trunc();

		let a = afc(m) + afc(n1 - m) + afc(k - m) + afc(n2 - k + m);

		let kl = (a - afc(xl) - afc(n1 - xl) - afc(k - xl) - afc(n2 - k + xl)).exp();
		let kr = (a - afc(xr - 1.0) - afc(n1 - xr + 1.0) - afc(k - xr + 1.0) - afc(n2 - k + xr - 1.0)).exp();

		let lamdl = -(xl * (n2 - k + xl) / (n1 - xl + 1.0) / (k - xl + 1.0)).ln();
		let lamdr = -((n1 - xr + 1.0) * (k - xr + 1.0) / xr / (n2 - k + xr)).ln();

		let p1 = d + d;
		let p2 = p1 + kl / lamdl;
		let p3 = p2 + kr / lamdr;

		let mut reject = true;

		'label30: loop {
			let u = rand(prng, precision) * p3;
			let mut v = rand(prng, precision);

			if u < p1 {
				/* ...RECTANGULAR REGION... */
				ix = xl + u;
			} else if u <= p2 {
				/* ...LEFT TAIL... */
				ix = xl + v.ln() / lamdl;

				if ix < minjx {
					continue 'label30;
				}

				v = v * (u - p1) * lamdl;
			} else {
				/* ...RIGHT TAIL... */
				ix = xr - v.ln() / lamdr;

				if ix > maxjx {
					continue 'label30;
				}

				v = v * (u - p2) * lamdr;
			}

			/*
			 * ...ACCEPTANCE/REJECTION TEST...
			 */
			if m < 100.0 || ix <= 50.0 {
				/* ...EXPLICIT EVALUATION... */
				let mut f = 1.0;

				if m < ix {
					let mut i = m + 1.0;
					while i < ix {
						/*40*/
						f = f * (n1 - i + 1.0) * (k - i + 1.0) / (n2 - k + i) / i;
						i += 1.0;
					}
				} else if m > ix {
					let mut i = ix + 1.0;
					while i < m {
						/*50*/
						f = f * i * (n2 - k + i) / (n1 - i) / (k - i);
						i += 1.0;
					}
				}

				if v <= f {
					reject = false;
				}
			} else {
				/* ...SQUEEZE USING UPPER AND LOWER BOUNDS... */

				let y = ix;
				let y1 = y + 1.0;
				let ym = y - m;
				let yn = n1 - y + 1.0;
				let yk = k - y + 1.0;
				let nk = n2 - k + y1;
				let r = -ym / y1;
				let s = ym / yn;
				let t = ym / yk;
				let e = -ym / nk;
				let g = yn * yk / (y1 * nk) - 1.0;
				let dg = if g < 0.0 { 1.0 + g } else { 1.0 };
				let gu = g * (1.0 + g * (-0.5 + g / 3.0));
				let gl = gu - 0.25 * ((g * g) * (g * g)) / dg;
				let xm = m + 0.5;
				let xn = n1 - m + 0.5;
				let xk = k - m + 0.5;
				let nm = n2 - k + xm;

				#[rustfmt::skip]
				let ub = y * gu - m * gl + DELTAU +
					xm * r * (1.0 + r * (-0.5 + r / 3.0)) +
					xn * s * (1.0 + s * (-0.5 + s / 3.0)) +
					xk * t * (1.0 + t * (-0.5 + t / 3.0)) +
					nm * e * (1.0 + e * (-0.5 + e / 3.0));

				/* ...TEST AGAINST UPPER BOUND... */
				let alv = v.ln();

				if alv > ub {
					reject = true;
				} else {
					/* ...TEST AGAINST LOWER BOUND... */

					let mut dr = xm * ((r * r) * (r * r));
					if r < 0.0 {
						dr = dr / (1.0 + r);
					}

					let mut ds = xn * ((s * s) * (s * s));
					if s < 0.0 {
						ds = ds / (1.0 + s);
					}

					let mut dt = xk * ((t * t) * (t * t));
					if t < 0.0 {
						dt = dt / (1.0 + t);
					}

					let mut de = nm * ((e * e) * (e * e));
					if e < 0.0 {
						de = de / (1.0 + e);
					}

					if alv < ub - 0.25 * (dr + ds + dt + de) + (y + m) * (gl - gu) - DELTAL {
						reject = false;
					} else {
						/* ...STIRLING'S FORMULA TO MACHINE ACCURACY... */

						if alv <= (a - afc(ix) - afc(n1 - ix) - afc(k - ix) - afc(n2 - k + ix)) {
							reject = false
						} else {
							reject = true;
						}
					}
				}
			}
			if reject {
				continue 'label30;
			}
			break;
		}

		ix
	};

	/*
	 * RETURN APPROPRIATE VARIATE
	 */

	let jx = if kk + kk >= (tn as u64) {
		if nn1 > nn2 {
			(kk - nn2) as f64 + ix
		} else {
			nn1 as f64 - ix
		}
	} else if nn1 > nn2 {
		kk as f64 - ix
	} else {
		ix
	};

	Ok(jx as u64)
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn test_afc()
	{
		assert_eq!(afc(1.0), 0.0);
		assert_eq!(afc(7.0), 8.525161361);
		assert_eq!(afc(8.0), 10.604602878798048);
	}
}
