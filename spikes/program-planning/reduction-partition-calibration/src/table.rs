//! Device-free validator and scorer for the target-private width-table study.

#[allow(
    dead_code,
    reason = "the shared analyzer keeps both predecessor entry points and helpers for exact replay"
)]
#[path = "shape_aware.rs"]
mod shared;

fn main() {
    shared::table_main();
}
