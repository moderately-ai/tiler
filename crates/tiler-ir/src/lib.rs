#![feature(generic_const_parameter_types)]
#![feature(min_adt_const_params)]
// `variant_count` is what makes an exhaustive-injectivity test's domain
// enumeration a *proof* rather than a sample: it sizes the enumerating array
// from the enum itself, so a variant added to a vocabulary and not to the
// enumeration is a build error instead of a population that silently shrinks
// while the test keeps reporting no collision. Every other mechanism available
// here — a hand-written length, a successor chain, a wildcard-free match — can
// be satisfied by an enumeration that has stopped covering its domain. The same
// reasoning admitted it to `tiler-metal`; see that crate's `lib.rs`.
//
// Gated on `test` because the enumerations are test-local: the vocabularies
// themselves are public but the *lists* of their inhabitants are not, so
// declaring the feature unconditionally would widen this crate's nightly
// surface for nothing and warn as an unused feature on every non-test build.
#![cfg_attr(test, feature(variant_count))]
#![allow(incomplete_features)]
//! Target-independent representations and verifiers for Tiler.
//!
//! This crate currently implements the bounded slices selected for the first
//! value proof, layer by layer: shapes and numerics, the semantic graph, the
//! canonical index vocabulary, scheduled regions, structured kernel IR, and the
//! target-neutral kernel program. It is intentionally not yet the complete
//! tensor compiler IR.
//!
//! Construction and read APIs are grouped by the invariants they protect:
//!
//! ```
//! use tiler_ir::semantic::{F32, InputKey, OutputKey, SemanticProgramBuilder};
//! use tiler_ir::shape::Shape;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut draft = SemanticProgramBuilder::try_standard()?;
//! let input = draft.input::<F32>(InputKey::new("input")?, Shape::from_dims([4]))?;
//! let result = draft.output(OutputKey::new("result")?, input)?;
//! let program = draft.build()?;
//!
//! assert_eq!(program.input_count(), 1);
//! assert_eq!(program.output_count(), 1);
//! assert_eq!(program.resolve_typed_output(&result)?.key().as_str(), "result");
//! # Ok(())
//! # }
//! ```

mod convenience;

/// Counted enumerations shared by this crate's exhaustive-injectivity tests.
#[cfg(test)]
mod exhaustive_injectivity;

/// Canonical byte-encoding primitives shared by every identity derivation.
pub mod identity;
/// Public target-independent iteration, access, and scalar-region vocabulary.
pub mod index;
/// Public backend-consumable structured kernel IR, verifier, and identity.
pub mod kernel;
/// The one shared scalar-arithmetic policy, means, locus, and provenance
/// vocabulary.
pub mod numerics;
/// Public target-neutral kernel-program IR, verifier, and identity.
pub mod program;
/// Public target-neutral scheduled-region IR, verifier, and identity.
pub mod schedule;
/// Public semantic tensor-program vocabulary.
pub mod semantic;
/// Target-independent fixed shape vocabulary.
pub mod shape;

pub use convenience::CheckedBuildError;
