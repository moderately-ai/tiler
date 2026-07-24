//! Pure structured-kernel-to-Metal-source lowering for Tiler.
//!
//! This crate owns deterministic source emission and target metadata, not live
//! device/runtime APIs, Apple tool discovery, offline compiler invocation,
//! artifact caching, or publication. Host-side AOT orchestration belongs in
//! `tiler-metal-aot`.
//!
//! # What it consumes
//!
//! One or more [`tiler_ir::kernel::VerifiedKernel`]s and an explicit
//! [`target::MetalTargetFacts`]. A verified kernel is already proven to be a
//! refinement of its scheduled region, so this crate never consults the
//! semantic graph, re-derives an access relation, infers a reduction order, or
//! recognizes a fusion shape. It translates the structured operation vocabulary
//! mechanically, one operation at a time.
//!
//! # What it guarantees
//!
//! - **Deterministic bytes.** The same set of verified kernels and the same
//!   target facts always produce byte-identical source. Entry points are
//!   ordered by canonical identity, symbols are content-derived, local names
//!   come from a fixed structural walk, and only ordered containers are used.
//! - **Fail-closed translation.** A governed construct with no Metal
//!   realization is a typed [`diagnostic::MetalEmitError`] naming the rejected
//!   entity and a stable rule identifier, never best-effort source.
//! - **Explicit numerics.** `f32` immediates are emitted as exact bit patterns,
//!   NaN canonicalization is an emitted helper whose predicate is an integer
//!   test over reinterpreted bits rather than a floating-point one, and each
//!   arithmetic operation is its own statement. Those three hold under every
//!   math mode. What the operations cannot carry is reported instead of
//!   assumed: compiler selections as [`record::MetalNumericalRequirement`]s,
//!   and obligations no selection realizes as [`record::MetalNumericalGap`]s.
//!
//! # Emitting is not claiming conformance
//!
//! [`emit::emit_translation_unit`] returning a unit means the structured
//! kernels translated. It does not mean the target can honour their declared
//! numerical contract. [`record::MetalTranslationUnit::require_declared_realization`]
//! is the conformance claim and fails closed.
//!
//! On every governed Apple family this currently rejects any kernel that
//! performs `f32` arithmetic under a subnormal-preserving realization, because
//! Apple GPU `f32` arithmetic flushes subnormals to zero in every math mode.
//! That is a measured hard feasibility limit, recorded rather than hidden
//! behind a flag that would not deliver it.
//!
//! # What it does not decide
//!
//! Launch geometry, pipeline limits, occupancy, fusion, reduction order, and
//! numerical policy are owned elsewhere. Compiling the emitted source is owned
//! by `tiler-metal-aot`, which this crate uses as a development dependency to
//! compile its golden fixtures through the real offline toolchain. Those tests
//! self-skip where no qualified Apple toolchain resolves, so a green run is
//! compiler evidence only on a host that has one.
//!
//! # Boundary status
//!
//! Every public item in this crate is a reviewed *draft* boundary under ADR
//! 0074 §7: it is built and tested at full fidelity while the facade is under
//! review, rather than presented as an accepted public API.
//!
//! ```
//! use tiler_ir::kernel::lower_scheduled_region;
//! use tiler_ir::schedule::{
//!     Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ExecutionBinding,
//!     KernelSchedule, LaunchPlan, LogicalAccess, NumericalPermission, NumericalRealization,
//!     OwnershipProof, OwnershipProofKind, OwnershipWitnessId, RegionId, ReductionTopology,
//!     ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
//! };
//! use tiler_ir::shape::Shape;
//! use tiler_metal::emit::emit_translation_unit;
//! use tiler_metal::record::{MetalNumericalGap, MetalNumericalRequirement};
//! use tiler_metal::target::{
//!     LaunchIndexRealization, MetalDeploymentMinimum, MetalPlatform, MetalSubnormalArithmetic,
//!     MetalTargetFacts, MslLanguageVersion,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
//! builder.iteration_shape(Shape::from_dims([4]))?;
//! builder.push_access(Access {
//!     tensor: TensorRole::Input,
//!     mode: AccessMode::Read,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(0),
//!     ownership: None,
//! })?;
//! builder.push_access(Access {
//!     tensor: TensorRole::Intermediate,
//!     mode: AccessMode::Write,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(1),
//!     ownership: Some(OwnershipWitnessId::new(0)),
//! })?;
//! builder.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(0),
//!     tensor: TensorRole::Input,
//!     kind: BoundsProofKind::LinearRange { element_count: 4 },
//! })?;
//! builder.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(1),
//!     tensor: TensorRole::Intermediate,
//!     kind: BoundsProofKind::LinearRange { element_count: 4 },
//! })?;
//! builder.ownership_proof(OwnershipProof {
//!     id: OwnershipWitnessId::new(0),
//!     tensor: TensorRole::Intermediate,
//!     kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 4 },
//! })?;
//! builder.scalar_program(ScalarProgram::MultiplyThenAdd {
//!     scale_bits: 2.0_f32.to_bits(),
//!     bias_bits: 1.0_f32.to_bits(),
//!     canonical_nan_bits: 0x7fc0_0000,
//!     contraction: false,
//! })?;
//! builder.numerical(NumericalRealization::new(
//!     "tiler.doc.strict-f32",
//!     0x7fc0_0000,
//!     SubnormalMode::Preserve,
//!     SubnormalMode::Preserve,
//!     NumericalPermission::Forbidden,
//!     NumericalPermission::Forbidden,
//! ))?;
//! builder.schedule(KernelSchedule {
//!     binding: ExecutionBinding::GlobalLinearInvocation,
//!     work_items: 4,
//!     threads_per_workgroup: 1,
//!     tail: TailPolicy::Exact,
//!     output_owner: OwnershipWitnessId::new(0),
//!     reduction: ReductionTopology::None,
//!     launch: LaunchPlan {
//!         grid_threads: 4,
//!         threads_per_workgroup: 1,
//!         zero_work_skips_dispatch: true,
//!     },
//! })?;
//! let kernel = lower_scheduled_region(&builder.build()?)?;
//!
//! let target = MetalTargetFacts::new(
//!     MslLanguageVersion::Metal3_1,
//!     MetalPlatform::MacOs,
//!     MetalDeploymentMinimum::new(13, 0),
//!     LaunchIndexRealization::ThreadPositionInGridUInt,
//!     MetalSubnormalArithmetic::FlushesToZero,
//!     31,
//! );
//! let unit = emit_translation_unit(&[&kernel], &target)?;
//!
//! // Emission is a pure function of the kernels and the target facts.
//! assert_eq!(unit.source(), emit_translation_unit(&[&kernel], &target)?.source());
//! let entry = &unit.entry_points()[0];
//! assert!(unit.source().contains(&format!("kernel void {}(", entry.symbol())));
//! assert_eq!(entry.buffers().len(), 2);
//! // The canonical NaN is emitted, never inherited from a compiler default,
//! // and its predicate is an integer test no math mode can relax.
//! assert!(unit.source().contains("as_type<float>(0x7fc00000u)"));
//! assert!(!unit.source().contains("isnan"));
//! assert_eq!(
//!     unit.numerical_requirements(),
//!     [
//!         MetalNumericalRequirement::SafeMathMode,
//!         MetalNumericalRequirement::NoFloatingPointContraction,
//!     ],
//! );
//! // Emitting is not claiming conformance: Apple GPU f32 arithmetic flushes
//! // subnormals, so this strict realization is not fully realizable.
//! assert_eq!(
//!     unit.numerical_gaps(),
//!     [MetalNumericalGap::SubnormalFlushInArithmetic],
//! );
//! assert!(unit.require_declared_realization().is_err());
//! # Ok(())
//! # }
//! ```

/// Typed fail-closed emission diagnostics.
pub mod diagnostic;
/// Deterministic structured-kernel-to-MSL translation.
pub mod emit;
/// The emitted translation unit, its entry points, and its binding tables.
pub mod record;
/// Explicit Metal target facts consumed by emission.
pub mod target;

#[cfg(test)]
mod golden_compilation;
#[cfg(test)]
mod tests;
