# Ope in rust

This is an Order-preserving encryption (OPE) lib inspired by cryptdb's ope implementation. 

It is a pure rust implementation, no c dependencies needed.

It is also written for no-std targets (thanks to num-traits) and works in wasm.

The max value to encrypt is `65532`

```rust
use ope::get_ope;

fn main()
{
	let k = b"this is a key 10".to_owned();

	let ope = get_ope(&k);

	let a = ope.encrypt(21).unwrap();
	let b = ope.encrypt(65531).unwrap();
	let c = ope.encrypt(65532).unwrap();

	assert!(a < b);
	assert!(b < c);
}
```