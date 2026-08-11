//! Device-free validator and scorer for the fresh row-regime interaction study.

#[allow(
    dead_code,
    reason = "the shared analyzer keeps the predecessor entry point and helpers for exact replay"
)]
#[path = "shape_aware.rs"]
mod shared;

fn main() {
    shared::interaction_main();
}
