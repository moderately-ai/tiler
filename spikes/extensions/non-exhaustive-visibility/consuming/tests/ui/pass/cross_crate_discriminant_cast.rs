// Claim 3, compiling direction: a discriminant cast of a `#[non_exhaustive]`
// enum, written outside the defining crate. It compiles, so the attribute does
// not reach an `as` cast at all. ADR 0074 convention 5b calls such a cast a
// total map whose arms are implied rather than written; this fixture is the
// measurement that the implied arms are exactly what the attribute cannot see.
//
// Its failing companion is `fail/cross_crate_discriminant_tag_match.rs`, the
// byte-identical map written as the convention-3 match. The pair is the
// evidence, not either half: the same derived tag is a compile error in one
// form and silence in the other, across the same crate boundary and on the
// same compiler.

use non_exhaustive_defining::Growing;

fn cross_crate_discriminant_cast(value: Growing) -> u8 {
    value as u8
}

fn main() {
    assert_eq!(cross_crate_discriminant_cast(Growing::A), 0);
    assert_eq!(cross_crate_discriminant_cast(Growing::B), 1);
}
