#![feature(generic_const_parameter_types)]
#![feature(min_adt_const_params)]
#![allow(incomplete_features)]
//! Target-independent representations and verifiers for Tiler.
//!
//! This crate currently implements the bounded semantic and canonical-index
//! slices selected for the first value proof. It is intentionally not yet the
//! complete tensor compiler IR.
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

/// Public target-independent iteration, access, and scalar-region vocabulary.
pub mod index;
/// Public semantic tensor-program vocabulary.
pub mod semantic;
/// Target-independent fixed shape vocabulary.
pub mod shape;
