use rs_blocks_macros::*;
use serde::{self, Deserialize};

fn default_alpha() -> f32 {
	0.0
}

// Looks like `with_fields` needs to be used outside of `derive`. The `Deserialize` trait still
// works for fields added  by `with_fields` though.
#[with_fields(alpha)]
#[derive(Clone, Copy, Deserialize)]
struct A {
	a: u64,
}

#[test]
fn add_attribute() {
	let a = A { a: 4, alpha: 3.2 };
	assert_eq!(a.a, 4);
	assert_eq!(a.alpha, 3.2);
}
