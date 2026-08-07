// `variant_count` sizes this crate's exhaustive-injectivity enumerations from
// the enums themselves, so a vocabulary widened in `tiler-ir` is a build error
// in the test that claims to cover it rather than a population that quietly
// shrinks while still reporting no collision. A hand-written length has no such
// check, which is exactly the failure the enumerations exist to rule out. The
// same reasoning admitted it to `tiler-metal` and `tiler-ir`.
//
// Gated on `test` because the enumerations are test-local: the vocabularies are
// public but the lists of their inhabitants are not, so an unconditional
// declaration would widen this crate's nightly surface for nothing.
#![cfg_attr(test, feature(variant_count))]
//! Target-neutral artifact, ABI, validation, and routing contracts for Tiler.
//!
//! This crate depends on `tiler-ir` for the shared target-neutral
//! representation: it retains programs that crate has already verified, rebuilds
//! decoded values through its checked constructors, owns no second editable
//! program model, and never invokes compiler passes. Its job is to project a
//! verified [`tiler_ir::program::VerifiedKernelProgram`] into the bounded,
//! versioned artifact model a runtime or a codec consumes: entry points, the
//! neutral ABI and its launch expressions, plan portfolios and their routing
//! predicates, declared target requirements, the provenance actually reached by
//! the packaged plan, and backend payload descriptors.
//!
//! That separation is the point of the layer. Nothing here requires a consumer
//! to link `tiler-compiler`, to reconstruct a semantic graph, a region cover, a
//! cost model, or a search state, or to know which strategy produced the plan.
//!
//! # Public boundary status
//!
//! [`program`] is a **reviewed draft boundary** (ADR 0074 §7, ADR 0075). It is
//! `pub` so its shape can be reviewed as a whole; it is not an accepted public
//! facade until Tom accepts the exact interface.
//!
//! [`proof`] is an **accepted facade**, promoted on Tom's review of
//! 2026-07-25. It is deliberately *not* artifact semantics: a sidecar names an
//! artifact, an artifact never names a sidecar, and an artifact decodes,
//! validates, and dispatches with no sidecar present.

/// Public target-neutral artifact program model, verifier, and identity.
pub mod program;
/// The separate, versioned proof-case evidence sidecar.
pub mod proof;
