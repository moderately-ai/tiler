// Claim 4, compiling direction: the escape hatch does not reach the cast
// either. ADR 0074 records `non_exhaustive_omitted_patterns` as the mechanism
// that would restore a compile error without giving up `#[non_exhaustive]`;
// `fail/omitted_patterns_denied.rs` measures it firing. Here the same feature
// gate is on and the same lint is denied crate-wide, and the cast still
// compiles, because the lint is a lint about match arms and a cast writes none.
//
// So the hole convention 3 closes is not one a stricter lint level closes.

#![feature(non_exhaustive_omitted_patterns_lint)]
#![deny(non_exhaustive_omitted_patterns)]

use non_exhaustive_defining::Growing;

fn tag(value: Growing) -> u8 {
    value as u8
}

fn main() {
    assert_eq!(tag(Growing::A), 0);
    assert_eq!(tag(Growing::B), 1);
}
