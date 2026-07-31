//! The one crate a Tiler consumer depends on.
//!
//! A consumer writes `tiler = { … }` in its manifest and reaches the inline
//! frontend through [`tensor!`]. Nothing else in the workspace is part of that
//! contract: the crates under `crates/tiler-*` are the compiler's internals,
//! and a consumer that has to name one of them to make generated code compile
//! would be holding a dependency it never agreed to.
//!
//! # Why the macro lives in another crate
//!
//! Rust restricts a `proc-macro` crate to exporting macros, so the crate that
//! implements `tensor!` can never also carry the runtime and frontend types a
//! consumer needs. Making `tiler` the proc-macro crate would therefore cap the
//! facade at macros forever; leaving `tiler-macros` as the crate consumers
//! import would either fix the public path as `tiler_macros::tensor!` or force
//! generated tokens to name internal crates the consumer did not declare.
//!
//! A normal facade re-exporting the macro is the standard direction and the
//! one that keeps both properties: the public path stays `tiler::tensor!`, and
//! generated tokens resolve through `tiler` — a crate the consumer already
//! named.
//!
//! # Public boundary status
//!
//! This crate is an **accepted public boundary**: Tom ratified the two-crate
//! topology and the `tiler::tensor!` path on 2026-07-30 and accepted the exact
//! surface — the manifests, the dependency direction, the re-export, and
//! [`__private`] — on 2026-07-31 under
//! `admit-the-tiler-facade-and-proc-macro-crate-boundary`. Acceptance
//! stabilizes neither the macro grammar nor the runtime adapter, which their
//! own tickets own, and the admission record in the design corpus is owned by
//! `record-the-frontend-crate-admission-in-the-design-corpus`.
//!
//! # What is here today
//!
//! The re-export and the generated-path anchor, and nothing else. No frontend
//! or runtime types are re-exported yet: those are selected by
//! `define-inline-symbol-binding-and-runtime-value-adaptation`, and
//! re-exporting anything before then would publish a boundary its ticket did
//! not review.
//!
//! # The artifact-family selection is deliberately *not* re-exported here
//!
//! `promote-artifact-family-selection-for-the-frontend` reviewed the canonical
//! typed `ArtifactFamilySelection` that ADR 0049 requires every inline AOT
//! request to carry, and placed the frontend's edge to it on `tiler-macros`
//! rather than on this crate.
//!
//! The reason is what the edge would cost here. Its one canonical encoder lives
//! in `tiler-metal-aot`, whose dependency closure ADR 0077 item 2 decides
//! empty, so the vocabulary can be neither copied nor moved beneath the driver;
//! the frontend must depend on the driver to state a selection at all. A
//! `proc-macro` crate and its dependencies are built for the host and never
//! enter a consumer's target build graph, so `tiler-macros` can hold that edge
//! for free. This crate cannot: a normal dependency here would link a
//! process-spawning Apple toolchain driver into every consumer on every
//! platform, and would publish Apple backend policy on a consumer-neutral
//! boundary — the same cost ADR 0077 item 4 already refused for `tiler-metal`.
//!
//! Nothing a consumer writes needs the type. A delivery policy is stated in
//! region syntax, and generated tokens name `#[cfg]` predicates and byte
//! literals, not the selection. `tests/dependency_direction.rs` is what keeps
//! this property true rather than merely intended.
//!
//! ```
//! // The re-export resolves, and the tokens it expands to reach back into
//! // this crate. The value is inert: `tensor!` has no grammar yet.
//! let _region = tiler::tensor!();
//! ```

pub use tiler_macros::tensor;

/// Implementation detail named by generated code; not a public interface.
///
/// A procedural macro has no `$crate`, so its expansion has to spell an
/// absolute path, and that path has to land somewhere. It lands here rather
/// than in an internal crate so a consumer's generated code depends on exactly
/// the one crate the consumer declared.
///
/// Nothing in this module is covered by any compatibility claim, and no
/// consumer should write these paths by hand.
#[doc(hidden)]
pub mod __private {
    /// The inert value a current `tensor!` expansion evaluates to.
    ///
    /// It carries no tensor semantics and holds no data. It exists so that the
    /// facade re-export and the generated path are checked by the compiler
    /// before there is a grammar to check them with, and the grammar tickets
    /// replace it with a real expansion result.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ExpansionAnchor;

    /// Returns the anchor that `tiler::tensor!` currently expands to.
    #[must_use]
    pub const fn expansion_anchor() -> ExpansionAnchor {
        ExpansionAnchor
    }
}
