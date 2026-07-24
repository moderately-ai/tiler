// Claim 2, inertness direction: the same `#[deny]` without the feature gate.
// The lint name is unknown, so the attribute constrains nothing and the omitted
// `Growing::B` is never reported. The retained `.stderr` is the evidence for
// that second half: it must contain the unknown-lint diagnostic and must not
// contain "some variants are not matched explicitly".
//
// `#![deny(unknown_lints)]` is what turns the warning into an error here. It
// mirrors the repository gate, which compiles with `-D warnings`, so a consumer
// that spelled the attribute without the gate would fail the gate rather than
// carry a silent hole.

#![deny(unknown_lints)]

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
