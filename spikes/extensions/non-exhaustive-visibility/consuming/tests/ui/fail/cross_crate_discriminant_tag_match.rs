// Claim 3, failing direction: the cast in `pass/cross_crate_discriminant_cast.rs`
// rewritten as the match ADR 0074 convention 3 requires, deriving the same tag
// for the same variant. It must fail with E0004.
//
// The contrast is the point. Both files map `Growing` totally onto a `u8`
// identity tag from outside the defining crate, and both agree on every value
// the enum has today. Only the written match is a construct `#[non_exhaustive]`
// constrains, so only the written match fails. That asymmetry is why convention
// 3 rather than convention 5 is what closes a discriminant-cast encoder: adding
// a variant is a build error here and nothing at all in the sibling file.

use non_exhaustive_defining::Growing;

fn cross_crate_discriminant_tag(value: Growing) -> u8 {
    match value {
        Growing::A => 0,
        Growing::B => 1,
    }
}

fn main() {
    let _ = cross_crate_discriminant_tag(Growing::A);
}
