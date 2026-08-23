//! Structured-kernel construction, verification, and identity tests.
//!
//! Positive tests prove that the canonical lowering and an independently
//! hand-built producer kernel reach the same verified product and identity.
//! Each verification rule then has a negative test that builds a deliberately
//! wrong kernel through the public builder and asserts the exact typed
//! diagnostic, so a rejected kernel names the obligation it violated.
//!
//! # Mapping rule
//!
//! Split from one 8,399-line file (`split-the-kernel-test-monolith-into-focused-modules`)
//! by subject: each child module holds the tests for one construction,
//! verification, or lowering property, plus any fixture used only by that
//! property. A fixture used by tests in more than one child lives in
//! [`support`] instead, so it is defined exactly once. `support` is not
//! organized by production module (`builder.rs`, `lower.rs`, `verify.rs`, …)
//! because most fixtures here are reused across many of those — a scheduled
//! region built once is lowered, verified, and identity-checked in the same
//! test family.

mod attention_batched;
mod bf16;
mod builder_rules;
mod contraction;
mod cooperative_contraction_operands;
mod cooperative_execution;
mod cooperative_staging_rules;
mod cooperative_synchronization;
mod cooperative_tiles;
mod extent_operands;
mod extrema_reduction;
mod guarded_loads_predicated;
mod kernel_identity;
mod live_contraction;
mod live_row_major;
mod pointwise;
mod reduction;
mod strict_affine;
mod strict_cooperative_contraction;
mod support;
mod verify_rules;
mod vocabulary_closure;
