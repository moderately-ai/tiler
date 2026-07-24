// Claim 2, failing direction: with the unstable feature gate enabled, denying
// `non_exhaustive_omitted_patterns` restores a hard compile error for a
// cross-crate match that keeps its wildcard but omits a known variant. This is
// the mechanism ADR 0074 records as the proven alternative to dropping
// `#[non_exhaustive]`, and the reason it is recorded rather than adopted is the
// `#![feature]` gate on line 1.

#![feature(non_exhaustive_omitted_patterns_lint)]

use non_exhaustive_defining::Growing;

fn classify(value: &Growing) -> &'static str {
    #[deny(non_exhaustive_omitted_patterns)]
    match value {
        Growing::A => "a",
        _ => "unsupported",
    }
}

fn main() {
    let _ = classify(&Growing::A);
}
