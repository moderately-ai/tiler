// Claim 2, compiling direction: the denied lint accepts a match that keeps its
// wildcard and also names every variant. This is what a consumer would have to
// write to keep both `#[non_exhaustive]` and the compile error, and it is the
// cost side of the alternative ADR 0074 records but does not adopt.

#![feature(non_exhaustive_omitted_patterns_lint)]

use non_exhaustive_defining::Growing;

fn classify(value: &Growing) -> &'static str {
    #[deny(non_exhaustive_omitted_patterns)]
    match value {
        Growing::A => "a",
        Growing::B => "b",
        _ => "unsupported",
    }
}

fn main() {
    assert_eq!(classify(&Growing::A), "a");
    assert_eq!(classify(&Growing::B), "b");
}
