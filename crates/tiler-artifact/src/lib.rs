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

/// Public target-neutral artifact program model, verifier, and identity.
pub mod program;
// The proof-case evidence sidecar. Crate-private under ADR 0074 convention 7
// until its facade is reviewed; its module documentation names what it reserves
// and states why it is deliberately *not* artifact semantics — a sidecar names
// an artifact, an artifact never names a sidecar, and an artifact decodes,
// validates, and dispatches with no sidecar present.
mod proof;
