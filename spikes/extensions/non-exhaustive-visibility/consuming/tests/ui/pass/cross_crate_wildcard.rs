// Claim 1, compiling direction across the crate boundary: the wildcard arm
// `#[non_exhaustive]` makes mandatory. It compiles, and nothing reports that
// `Growing::B` was never named — which is exactly the silence ADR 0074's
// amended convention 5c is about.

use non_exhaustive_defining::Growing;

fn recognize(value: &Growing) -> Option<u8> {
    match value {
        Growing::A => Some(1),
        _ => None,
    }
}

fn main() {
    assert_eq!(recognize(&Growing::A), Some(1));
    assert_eq!(recognize(&Growing::B), None);
}
