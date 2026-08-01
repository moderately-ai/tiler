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
//! The re-export, [`value`] — the runtime-value boundary an integration
//! implements, selected by
//! `define-inline-symbol-binding-and-runtime-value-adaptation` — and the
//! [`__private`] items a `tensor!` expansion names. Every item in [`value`] is
//! a **reviewed draft boundary** (ADR 0074 §7, ADR 0075): it is `pub` because
//! the seam only works if a crate outside this one can implement it, and it is
//! not an accepted public facade until Tom accepts the exact interface.
//!
//! The inert `expansion_anchor` this crate carried while `tensor!` had no
//! grammar is gone rather than retained beside the real expansion:
//! `prototype-inline-proc-macro-frontend` delivered the grammar it was standing
//! in for, and a superseded path kept for company is a second thing a reader has
//! to rule out.
//!
//! # Why this crate depends on `tiler-ir`
//!
//! For one type. An adapter reports the scalar its storage holds, and a region
//! declares the scalar an operand must hold; that subject already has an
//! authority, `tiler_ir::program::StorageScalar`, and [`value`] re-exports it
//! rather than minting a second one.
//!
//! The alternative was a facade-local element-type enum, and it fails on
//! correctness rather than on taste. `tiler-macros` cannot name anything in this
//! crate — the facade depends on it, so the edge cannot run back — which means
//! the correspondence between what an expansion decides and what this crate
//! means would be held by nothing but the text of the emitted tokens. Sharing
//! one enum instead makes the macro's token emitter an exhaustive match over the
//! real vocabulary, so widening it is a build error rather than a variant no
//! expansion can spell.
//!
//! The cost is real and bounded: `tiler-ir` and its three `num-*` dependencies
//! enter a consumer's build graph. It is also a cost a consumer that executes an
//! embedded artifact pays anyway, because decoding one goes through
//! `tiler-artifact`, which depends on `tiler-ir`. `tests/dependency_direction.rs`
//! is what keeps the *forbidden* edge — the process-spawning Apple toolchain
//! driver — off this crate.
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
//! # use tiler::value::{
//! #     AdapterCapability, ResultRequest, StorageScalar, Tensor, TensorAdapter, ValueMetadata,
//! # };
//! # #[derive(Debug, PartialEq)]
//! # struct Buffer { scalar: StorageScalar, extents: Vec<u64> }
//! # #[derive(Debug)]
//! # struct Refused;
//! # impl core::fmt::Display for Refused {
//! #     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { f.write_str("refused") }
//! # }
//! # impl std::error::Error for Refused {}
//! # struct Toy;
//! # impl TensorAdapter for Toy {
//! #     type Value = Buffer;
//! #     type Context = ();
//! #     type Error = Refused;
//! #     fn supports(capability: AdapterCapability) -> bool {
//! #         match capability {
//! #             AdapterCapability::DenseRowMajorStorage | AdapterCapability::ResultConstruction => true,
//! #         }
//! #     }
//! #     fn metadata(value: &Buffer) -> Result<ValueMetadata, Refused> {
//! #         Ok(ValueMetadata::new(value.scalar, value.extents.iter().copied()))
//! #     }
//! #     fn build(_: &(), request: &ResultRequest<'_>) -> Result<Buffer, Refused> {
//! #         Ok(Buffer { scalar: request.storage_scalar(), extents: request.extents().to_vec() })
//! #     }
//! # }
//! # fn operand(extent: u64) -> Tensor<Toy> {
//! #     Tensor::new(Buffer { scalar: StorageScalar::F32, extents: vec![extent] }, ())
//! # }
//! let (a, b, c) = (operand(3), operand(3), operand(3));
//!
//! let d = tiler::tensor! {
//!     sym n;
//!     in a: f32[n], b: f32[n], c: f32[n];
//!     out (a * b) + c
//! };
//!
//! assert_eq!(d.expect("the operands agree").extents, vec![3]);
//! ```

pub use tiler_macros::tensor;

mod expansion;
mod route;

// Deliberately no outer doc comment here: the module documents itself with
// `//!`, and adding a `///` on the item would move intra-doc link resolution for
// the whole merged doc string up to the crate root, where the module's own item
// names do not resolve.
pub mod value;

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
    pub use crate::expansion::{
        AxisRef, BoundExtents, OperandFacts, RegionFacts, ResultAxis, ResultFacts, SymbolFacts,
        bind_and_build, bind_region, build_result,
    };
    pub use crate::route::{RouteFacts, RouteOutcome, bind_route_and_build, select_embedded_route};
}
