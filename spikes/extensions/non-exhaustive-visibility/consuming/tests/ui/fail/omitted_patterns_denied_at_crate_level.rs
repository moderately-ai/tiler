// Claim 4, failing direction, and the control that makes its compiling sibling
// evidence rather than an assumption.
//
// `pass/cast_ignores_denied_omitted_patterns.rs` concludes that denying
// `non_exhaustive_omitted_patterns` does not reach a cast, from the fact that
// such a crate compiles silently. That silence has a second possible cause: a
// crate-level lint level that the compiler accepts and then never consults.
// `fail/omitted_patterns_denied.rs` only rules this out for a `#[deny]` written
// on the match itself, which is a granularity no cast site has.
//
// So this file denies the lint at crate level, exactly as the compiling sibling
// does, and then gives it both constructs at once. The cast is the same one,
// and the match below omits `Growing::B` behind a wildcard. The retained
// diagnostic reports the match and says nothing about the cast — one error, not
// two — which is the contrast stated inside a single compilation rather than
// inferred across two.

#![feature(non_exhaustive_omitted_patterns_lint)]
#![deny(non_exhaustive_omitted_patterns)]

use non_exhaustive_defining::Growing;

fn tag(value: Growing) -> u8 {
    value as u8
}

fn classify(value: &Growing) -> &'static str {
    match value {
        Growing::A => "a",
        _ => "unsupported",
    }
}

fn main() {
    let _ = tag(Growing::A);
    let _ = classify(&Growing::A);
}
