//! Target-independent representations and verifiers for Tiler.
//!
//! This crate currently implements only the bounded semantic/reference slice
//! selected for the first value proof. It is intentionally not a general
//! tensor IR yet.
//!
//! Construction and read APIs are grouped by the invariants they protect:
//!
//! ```
//! use tiler_ir::semantic::{InputKey, OutputKey, SemanticProgramBuilder};
//! use tiler_ir::shape::Shape;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut draft = SemanticProgramBuilder::try_standard()?;
//! let input = draft.input_f32(InputKey::new("input")?, Shape::from_dims([4]))?;
//! let result = draft.output(OutputKey::new("result")?, input)?;
//! let program = draft.build()?;
//!
//! assert_eq!(program.input_count(), 1);
//! assert_eq!(program.output_count(), 1);
//! assert_eq!(program.resolve_output(&result)?.key().as_str(), "result");
//! # Ok(())
//! # }
//! ```

/// Host reference values and evaluation.
pub mod reference;
/// Public semantic tensor-program vocabulary.
pub mod semantic;
/// Target-independent fixed shape vocabulary.
pub mod shape;
