// Claim 1, failing direction: the body of `same_crate_total_map` in the
// defining crate, moved across the crate boundary and otherwise unchanged.
// It must fail with E0004. This is the fact that makes ADR 0074's convention 3
// ("exhaustively matched, no wildcard arm") and its convention 5 contradict
// each other for any encoder that does not live in the defining crate.

use non_exhaustive_defining::Growing;

fn cross_crate_total_map(value: &Growing) -> u8 {
    match value {
        Growing::A => 1,
        Growing::B => 2,
    }
}

fn main() {
    let _ = cross_crate_total_map(&Growing::A);
}
