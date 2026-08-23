//! Bounded tests for the target-neutral kernel-program IR.
//!
//! Fixtures bind real verified structured kernels to real verified semantic
//! programs. Coverage assignments are structural partitions: this layer proves
//! that every operation of the bound graph is covered exactly once, never that
//! a given kernel computes the operations its stage claims.
//!
//! # Mapping rule
//!
//! Split from one 6,143-line file (`split-the-ir-program-test-monolith-into-focused-modules`)
//! mirroring the production seam this directory's parent already has —
//! `abi.rs`, `alignment.rs`, `builder.rs`, `contraction_witness.rs`, `error.rs`, `model.rs`,
//! `verify.rs` — plus the cross-cutting subjects (declared splits, publishing copies, staged
//! realizations, `bf16`, live contraction, the ADR 0013 plan-determinism witness, the folded
//! shape-environment subject) that span more than one of those production modules. Each child
//! module holds the tests for one property plus any fixture used only by that property. A
//! fixture used by tests in more than one child lives in [`support`] instead, so it is defined
//! exactly once.
//!
//! Two child names were adjusted from the production module they mirror, to avoid colliding
//! with that sibling module one level up when referenced as `super::super::<name>`: `abi.rs`'s
//! tests live in [`abi_contract`], and `alignment.rs`'s in [`view_alignment`].

mod abi_contract;
mod bf16;
mod builder_rules;
mod cooperative_contraction;
mod identity;
mod live_contraction;
mod partial_reduction;
mod plan_determinism;
mod publishing_copy;
mod shape_environment_fold;
mod staged_realization;
mod support;
mod view_alignment;
