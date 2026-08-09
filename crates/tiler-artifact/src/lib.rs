// This crate's exhaustive-injectivity populations are derived from their Rust
// types: `variant_count` sizes fieldless-enum arrays, exhaustive outer-arm sums
// size payload-carrying-enum arrays, and an exhaustive bool-field census sizes
// a struct product. A vocabulary widened in `tiler-ir` is therefore a build
// error in the test that claims to cover it rather than a population that
// quietly shrinks while still reporting no collision.
//
// Gated on `test` because the enumerations are test-local: the vocabularies are
// public but the lists of their inhabitants are not, so an unconditional
// declaration would widen this crate's nightly surface for nothing.
#![cfg_attr(test, feature(variant_count))]
#![doc(test(attr(forbid(unsafe_code))))]
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

// The crate's governed domain separators, enumerated from a type so the
// no-prefix obligation `docs/artifact-abi.md` states is checked over the whole
// admitted set rather than over whichever subset a hand-written list still names.
// Crate-level rather than inside either container because the property is global:
// one algorithm hashes both containers and the program identity encoding in one
// process, so a domain added to any of them could merge two subjects.
#[cfg(test)]
mod domains;

/// Public target-neutral artifact program model, verifier, and identity.
pub mod program;
/// The separate, versioned proof-case evidence sidecar.
pub mod proof;
