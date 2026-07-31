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
//! This crate is a **reviewed draft boundary** (ADR 0074 §7, ADR 0075), on the
//! same footing as `tiler_artifact::program` and `tiler_runtime::load`: it is
//! `pub` so its shape can be reviewed as a whole, and it is not an accepted
//! public facade until Tom accepts the exact interface. ADR 0075 classifies a
//! new workspace member and a new crate-root `pub mod` as always requiring that
//! review, and this crate is both.
//!
//! Tom ratified the two-crate topology and the `tiler::tensor!` path on
//! 2026-07-30, which is narrower than accepting this surface: the manifests,
//! the dependency direction, the re-export, and [`__private`] are the review
//! packet, and unlike every other crate in the workspace these two carry no
//! admission record yet. Admitting them stabilizes neither the macro grammar
//! nor the runtime adapter, which their own tickets own.
//!
//! # What is here today
//!
//! The re-export and the generated-path anchor, and nothing else. No frontend
//! or runtime types are re-exported yet — those are selected by
//! `define-inline-symbol-binding-and-runtime-value-adaptation` and
//! `promote-artifact-family-selection-for-the-frontend`, and re-exporting
//! anything before then would publish a boundary this ticket did not review.
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
