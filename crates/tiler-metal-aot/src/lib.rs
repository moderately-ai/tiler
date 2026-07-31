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
//! - **One selected platform.** A [`input::MetalTarget`] names exactly one
//!   artifact family and derives the SDK both tool invocations use.
//! - **No silent defaults.** Every output-affecting choice — target triple,
//!   language standard, optimization level, and the numerical realization flags
//!   (`math-mode`, `fp32-functions`, `ffp-contract`) — is an explicit input.
//!   [`input::NumericalRealization`] has no `Default`; the caller must state it.
//! - **One prepared compilation.** [`driver::Toolchain::prepare`] binds the
//!   canonical [`identity::CompilationIdentity`] to the request and exact
//!   resolved paths that [`driver::PreparedCompilation::compile`] consumes, so a
//!   cache lookup and its miss path cannot observe different selections.
//! - **One stated family selection.** [`family::ArtifactFamilySelection`] is the
//!   canonical typed request field ADR 0049 requires an inline AOT request to
//!   carry. It fans out to one compile target per selected family, so a request
//!   naming several families is several compilations and never one payload
//!   relabelled. It is a reviewed draft boundary; see that module.
//!
//! The driver fails closed. When the toolchain or SDK cannot be resolved, when a
//! tool reports failure, or when the linker's output does not begin with the
//! `MTLB` magic, it returns a typed [`diagnostic::DriverError`] and never a
//! partial artifact.
//!
//! # What a success here proves, and what it does not
//!
//! **This crate produces offline compilation evidence, not runtime
//! compatibility evidence.** A returned [`record::CompiledArtifact`] means the
//! selected toolchain linked a `metallib`-shaped file for the compilation
//! target the request named. It is not evidence that any device or deployment
//! target can load or execute that library.
//!
//! The distinction is not pedantic: nothing in this crate opens a Metal device,
//! and the only check applied to the linker's output is that it begins with the
//! four magic bytes `MTLB`. Whether a given GPU family can run the library is
//! decided by the declared family and profile checks and by successful runtime
//! preparation, all of which belong to the runtime contract and none of which
//! run here. Describing an artifact from this crate as "usable" would merge two
//! evidence classes that the rest of the workspace keeps apart.
//!
//! ```
//! use tiler_metal_aot::input::{
//!     ApplePlatform, CompileRequest, DeploymentMinimum, MetalTarget, MslVersion,
//!     NumericalRealization, OptimizationLevel,
//! };
//!
//! let target = MetalTarget::new(
//!     ApplePlatform::MacOs,
//!     DeploymentMinimum::new(14, 0),
//!     MslVersion::Metal3_1,
//! ).expect("MSL 3.1 is admitted from macOS 14");
//! let request = CompileRequest::new(
//!     "// MSL source",
//!     target,
//!     OptimizationLevel::Default,
//!     NumericalRealization::strict_baseline(),
//! );
//!
//! assert_eq!(request.target.triple(), "air64-apple-macos14.0");
//! assert_eq!(
//!     request.compile_flags(),
//!     [
//!         "-target",
//!         "air64-apple-macos14.0",
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
// Documented by the module's own `//!` header, which states why the surface is
// public here rather than copied into the frontend or moved beneath this crate,
// and which half of ADR 0053 belongs to the frontend proc-macro crate instead.
// Every item in it is a reviewed *draft* boundary (ADR 0074 convention 7).
pub mod family;
// Documented by the module's own `//!` header so its intra-doc links resolve in
// the identity module's scope rather than this crate-root scope.
pub mod identity;
/// Explicit, strongly typed compilation inputs.
pub mod input;
/// Provenance, fingerprint, and compiled-artifact records.
pub mod record;
