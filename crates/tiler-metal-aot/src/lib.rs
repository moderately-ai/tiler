//! Bounded offline Apple Metal AOT compiler driver for Tiler.
//!
//! This crate owns one bounded responsibility: invoke Apple's offline `xcrun
//! metal` and `xcrun metallib` to turn Metal Shading Language (MSL) text plus an
//! explicit target and explicit output-affecting flags into `metallib` bytes,
//! and record the compiler fingerprint and full provenance of that compilation.
//!
//! It deliberately does not do the neighbouring jobs. It does not emit MSL
//! (that is `tiler-metal`), it does not assemble the target-neutral artifact
//! bundle, and it does not implement the expansion cache or the proc-macro
//! layer. It takes MSL and a target as input and produces bytes and provenance
//! as output.
//!
//! Two contracts are load-bearing:
//!
//! - **One selected SDK.** A [`input::MetalTarget`] names exactly one Apple SDK
//!   and family; both the `metal` and `metallib` invocations use it.
//! - **No silent defaults.** Every output-affecting choice — target triple,
//!   language standard, optimization level, and the numerical realization flags
//!   (`math-mode`, `fp32-functions`, `ffp-contract`) — is an explicit input.
//!   [`input::NumericalRealization`] has no `Default`; the caller must state it.
//!
//! The driver fails closed. When the toolchain or SDK cannot be resolved, when a
//! tool reports failure, or when the linker yields no usable library, it returns
//! a typed [`diagnostic::DriverError`] and never a partial artifact.
//!
//! ```
//! use tiler_metal_aot::input::{
//!     AppleSdk, CompileRequest, DeploymentMinimum, MetalTarget, MslVersion,
//!     NumericalRealization, OptimizationLevel,
//! };
//!
//! let target = MetalTarget::new(
//!     AppleSdk::MacOs,
//!     DeploymentMinimum::new(13, 0),
//!     MslVersion::Metal3_1,
//! );
//! let request = CompileRequest::new(
//!     "// MSL source",
//!     target,
//!     OptimizationLevel::Default,
//!     NumericalRealization::strict_baseline(),
//! );
//!
//! assert_eq!(request.target.triple(), "air64-apple-macos13.0");
//! assert_eq!(
//!     request.compile_flags(),
//!     [
//!         "-target",
//!         "air64-apple-macos13.0",
//!         "-std=metal3.1",
//!         "-O2",
//!         "-fmetal-math-mode=safe",
//!         "-fmetal-math-fp32-functions=precise",
//!         "-ffp-contract=off",
//!     ],
//! );
//! ```

/// Typed fail-closed compilation diagnostics.
pub mod diagnostic;
/// The offline toolchain driver and its resolution and compilation entry points.
pub mod driver;
/// Explicit, strongly typed compilation inputs.
pub mod input;
/// Provenance, fingerprint, and compiled-artifact records.
pub mod record;
