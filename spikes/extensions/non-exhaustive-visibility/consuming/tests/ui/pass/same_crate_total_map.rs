// Claim 1, compiling direction, in one file: a `#[non_exhaustive]` enum and a
// total map over it in the same crate. The attribute constrains other crates
// only, so no wildcard arm is required and none is written.

#[non_exhaustive]
enum Growing {
    A,
    B,
}

fn same_crate_total_map(value: &Growing) -> u8 {
    match value {
        Growing::A => 1,
        Growing::B => 2,
    }
}

fn main() {
    assert_eq!(same_crate_total_map(&Growing::A), 1);
    assert_eq!(same_crate_total_map(&Growing::B), 2);
}
