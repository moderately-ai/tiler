// `variant_count` is the one mechanism that makes an `ALL` array fail the build
// when a variant is added to its enum and not to the list. Every other site that
// has to know about a variant is an exhaustive `match`, which `rustc` already
// closes; a hand-written array has no such check, and an under-populated `ALL`
// is exactly the silently unprobed device this crate's applicability policy
// exists to refuse. Five declarations are sized by it:
// `applicability::MetalGpuFamily::ALL` and `applicability::MetalHostPredicate::ALL`,
// and `target::MslLanguageVersion::ALL`, `target::MetalPlatform::ALL`, and
// `target::MetalFloatArithmeticType::ALL`. The `BinaryOp` population in
// `tests::every_binary_construct_has_a_metal_realization` uses it for the same
// reason from the other direction: that vocabulary is `#[non_exhaustive]` and
// owned by another crate, so no match here can be closed on its behalf.
#![feature(variant_count)]
#![doc(test(attr(forbid(unsafe_code))))]
//! Pure structured-kernel-to-Metal-source lowering for Tiler.
//!
//! This crate owns deterministic source emission and target metadata, not live
//! device/runtime APIs, Apple tool discovery, offline compiler invocation,
//! artifact caching, or publication. Host-side AOT orchestration belongs in
//! `tiler-metal-aot`.
//!
//! # A second, smaller thing it owns
//!
//! [`applicability`] decides whether a *live host* is the exact macOS row the
//! first Metal profile was measured on. It is here rather than in a
//! backend-neutral crate because its vocabulary is Apple's, and it does not
//! break the rule above: it is a pure function over a normalized observation
//! some adapter made, so it reaches no device API itself. Everything else in
//! this crate is about the source a compilation consumes; that module is the
//! only part about the machine that runs the result.
//!
//! # What it consumes
//!
//! One or more [`tiler_ir::kernel::VerifiedKernel`]s, explicit
//! [`target::MetalTargetFacts`], and a selected
//! [`target::MetalEmissionRealization`]. A verified kernel is already proven
//! to be a refinement of its scheduled region, so this crate never consults
//! the semantic graph, re-derives an access relation, infers a reduction order,
//! or recognizes a fusion shape. It translates the structured operation
//! vocabulary mechanically, one operation at a time.
//!
//! # What it guarantees
//!
//! - **Deterministic bytes.** The same set of verified kernels, target facts, and selected emission realization always produce byte-identical source. Entry points are ordered by canonical identity, symbols are content-derived, local names come from a fixed structural walk, and only ordered containers are used.
//! - **Fail-closed translation.** A governed construct with no Metal
//!   realization is a typed [`diagnostic::MetalEmitError`] naming the rejected
//!   entity and a stable rule identifier, never best-effort source.
//! - **Explicit numerics.** Floating-point immediates are emitted as exact bit
//!   patterns — `f32` reinterpreting a `uint` and `bf16` a `ushort`, which the
//!   narrower width requires — NaN canonicalization is an emitted helper whose
//!   predicate is an integer test over reinterpreted bits rather than a
//!   floating-point one, and each arithmetic operation is its own statement.
//!   Those three hold under every math mode and at every emitted width; the
//!   canonicalization helper is per width, because a binary32 canonical pattern
//!   is not a `bfloat16` encoding at all. What the operations cannot carry is
//!   reported instead of
//!   assumed: compiler selections as [`record::MetalNumericalRequirement`]s,
//!   obligations no selection realizes as [`record::MetalNumericalGap`]s, and
//!   arithmetic types the target states no subnormal fact for through
//!   [`record::MetalTranslationUnit::unstated_subnormal_arithmetic`]. The last
//!   is `Unknown` rather than a verdict, and it is kept apart from the other
//!   two rather than collapsed into either.
//!
//! # Emitting is not claiming conformance
//!
//! [`emit::emit_translation_unit`] returning a unit means the structured
//! kernels translated. It does not mean the target can honour their declared
//! numerical contract. [`record::MetalTranslationUnit::require_declared_realization`]
//! is the conformance claim and fails closed.
//!
//! On every governed Apple family this rejects any kernel that performs `f32`
//! arithmetic under a subnormal-preserving realization, because Apple GPU `f32`
//! arithmetic flushes subnormals to zero in every math mode. That is a measured
//! hard feasibility limit, recorded rather than hidden behind a flag that would
//! not deliver it. A realization that accepts a flush to the zero the target
//! actually produces is honoured, and only a genuine sign mismatch stays a gap.
//!
//! The `f32` in that sentence is load-bearing. The same measured hardware
//! *preserves* subnormals through `f16` arithmetic, in the same math modes,
//! from modules declaring `air.compile.denorms_disable` identically — so the
//! subnormal fact is stated once per arithmetic type
//! ([`target::MetalSubnormalArithmeticFacts`]) and never read across types. A
//! type the target says nothing about is `Unknown`: emission records it and
//! the conformance claim fails closed on it, ahead of any gap, because a gap
//! set computed while a fact is missing is incomplete rather than merely
//! shorter.
//!
//! `bf16` is the third emitted width and the reason that record is keyed rather
//! than split in two. Its measured row *flushes*, like `f32`'s and unlike
//! `f16`'s, so "narrow formats preserve" is not a rule and a record answering
//! one narrow type from the other would be wrong half the time. A `bf16` kernel
//! reaching a target that states nothing for it is refused as `Unknown`, which
//! is what an unmeasured Apple family gets.
//!
//! **Emitting `bfloat` is not a claim that a device runs it.** Whether a target
//! family can dispatch a dtype is a target-profile capability, resolved before
//! routing commits and owned outside this crate; the measured Apple record has
//! one family compiling and linking a `bfloat` module and then refusing to
//! create a pipeline for it. Emission is identical on both, which is precisely
//! why that refusal cannot live here.
//!
//! This step is deliberately kept alongside the compiler's per-dimension
//! honourability declaration rather than retired in favour of it. The two are
//! not two answers to one question: the declaration is a claim about a target
//! and a contract and is answerable before emission, while this is a claim
//! about the operations one translation unit actually emitted. They also cannot
//! be collapsed by dependency — this crate does not depend on `tiler-compiler`,
//! so a compiler-side rejection is unreachable from a caller that drives
//! [`emit::emit_translation_unit`] from `tiler_ir` alone.
//! [`record::MetalNumericalGap`] records the full reasoning; the Metal fact
//! itself is declared exactly once per arithmetic type, on
//! [`target::MetalSubnormalArithmeticFacts`].
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
//! That development dependency carries a second obligation. The driver owns its
//! own MSL language version, Apple artifact family, and deployment minimum,
//! deliberately and for the reasons [`target`] records; this crate's
//! development edge is the only place in the workspace where both vocabularies
//! are visible at once, so it is where they are checked to name the same sets.
//! Those checks need no toolchain and never skip.
//!
//! # Boundary status
//!
//! Every public item in this crate is an accepted boundary: Tom accepted the
//! exact minimized whole facade on 2026-08-18 under ADR 0075, with the
//! provenance recorded in
//! `tickets/decide-the-tiler-metal-public-facade-surface.md` and the
//! application carried by `tickets/apply-the-accepted-tiler-metal-public-facade.md`.
//! Three subsets carry their own earlier, separately recorded acceptance and
//! were preserved byte for byte: [`direct_requirement`] (through
//! `carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit`),
//! [`applicability`] (the packet at `6c1cd1e` plus its recorded corrections),
//! and the ratified [`target::MetalSubnormalArithmetic::subnormal_mode`]
//! projection. The acceptance also narrowed eight out-of-crate-unused backend
//! spelling helpers to crate visibility and made
//! [`record::MetalNumericalRequirement`] exhaustive; each owning module
//! records its own consequence. Accepted is not stabilized — ADR 0075's
//! pre-alpha posture keeps a later source break cheap, explicit, and
//! reviewed, and acceptance says the boundary is intentional rather than
//! accidental.
//!
//! ```
//! use tiler_ir::kernel::lower_scheduled_region;
//! use tiler_ir::schedule::{
//!     Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId,
//!     ApproximationEnvelope, ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess,
//!     NumericalPermission, NumericalRealization, OwnershipProof, OwnershipProofKind,
//!     OwnershipWitnessId, PointwiseF32ExpressionBuilder, RegionId, ReductionTopology,
//!     AccessOrdinal, RegionProgram, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy,
//!     TensorRole,
//! };
//! use tiler_ir::shape::Shape;
//! use tiler_metal::emit::emit_translation_unit;
//! use tiler_metal::record::{MetalNumericalGap, MetalNumericalRequirement};
//! use tiler_metal::target::{
//!     LaunchIndexRealization, MetalDeploymentMinimum, MetalEmissionRealization,
//!     MetalFloatArithmeticType, MetalFlushedZeroSign, MetalPlatform,
//!     MetalSubnormalArithmetic, MetalSubnormalArithmeticFacts, MetalTargetFacts,
//!     MslLanguageVersion,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
//! builder.iteration_shape(Shape::from_dims([4]))?;
//! builder.push_access(Access {
//!     tensor: TensorRole::Input,
//!     component_role: None,
//!     mode: AccessMode::Read,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(0),
//!     ownership: None,
//! })?;
//! builder.push_access(Access {
//!     tensor: TensorRole::Intermediate,
//!     component_role: None,
//!     mode: AccessMode::Write,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(1),
//!     ownership: Some(OwnershipWitnessId::new(0)),
//! })?;
//! builder.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(0),
//!     tensor: TensorRole::Input,
//!     component_role: None,
//!     kind: BoundsProofKind::LinearRange { element_count: 4 },
//! })?;
//! builder.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(1),
//!     tensor: TensorRole::Intermediate,
//!     component_role: None,
//!     kind: BoundsProofKind::LinearRange { element_count: 4 },
//! })?;
//! builder.ownership_proof(OwnershipProof {
//!     id: OwnershipWitnessId::new(0),
//!     tensor: TensorRole::Intermediate,
//!     kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 4 },
//! })?;
//! let mut expression = PointwiseF32ExpressionBuilder::new();
//! let input = expression.input(AccessOrdinal::FIRST)?;
//! let scale = expression.constant(2.0_f32.to_bits())?;
//! let product = expression.multiply(input, scale)?;
//! let bias = expression.constant(1.0_f32.to_bits())?;
//! let root = expression.add(product, bias)?;
//! builder.program(RegionProgram::Numerical {
//!     scalar: ScalarProgram::PointwiseF32(expression.build(root)?),
//!     numerical: NumericalRealization::new(
//!         "tiler.doc.strict-f32",
//!         0x7fc0_0000,
//!         SubnormalMode::Preserve,
//!         SubnormalMode::Preserve,
//!         NumericalPermission::Forbidden,
//!         NumericalPermission::Forbidden,
//!         NumericalPermission::Forbidden,
//!         NumericalPermission::Forbidden,
//!         NumericalPermission::Forbidden,
//!         ApproximationEnvelope::Forbidden,
//!         ExceptionalValueAssumption::MakeNoAssumption,
//!         ExceptionalValueAssumption::MakeNoAssumption,
//!     ),
//! })?;
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
//!     MetalDeploymentMinimum::new(14, 0),
//!     // One measured behaviour per arithmetic type: the Apple row flushes in
//!     // f32 and preserves in f16. A type left out is Unknown, and emitting
//!     // arithmetic in it fails the conformance claim rather than borrowing
//!     // the other type's fact.
//!     MetalSubnormalArithmeticFacts::unmeasured()
//!         .stating(
//!             MetalFloatArithmeticType::F32,
//!             MetalSubnormalArithmetic::FlushesToZero {
//!                 zero_sign: MetalFlushedZeroSign::PreservesSign,
//!             },
//!         )
//!         .stating(
//!             MetalFloatArithmeticType::F16,
//!             MetalSubnormalArithmetic::PreservesSubnormals,
//!         ),
//!     31,
//! );
//! let emission = MetalEmissionRealization::new(
//!     LaunchIndexRealization::ThreadPositionInGridUInt,
//! );
//! let unit = emit_translation_unit(&[&kernel], &target, emission)?;
//!
//! // Emission is a pure function of the kernels, target facts, and selected
//! // source realization.
//! assert_eq!(
//!     unit.source(),
//!     emit_translation_unit(&[&kernel], &target, emission)?.source(),
//! );
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
//! // Every arithmetic type this unit used has a stated fact, so the gap list
//! // above is complete and the rejection below is about the gap.
//! assert!(unit.unstated_subnormal_arithmetic().is_empty());
//! assert!(unit.require_declared_realization().is_err());
//! # Ok(())
//! # }
//! ```

/// The pure macOS Metal host-applicability policy and its typed refusals.
pub mod applicability;
/// Typed fail-closed emission diagnostics.
pub mod diagnostic;
/// Apple-family comparison for the derived requirements a route already states.
pub mod direct_requirement;
/// Deterministic structured-kernel-to-MSL translation.
pub mod emit;
/// The emitted translation unit, its entry points, and its binding tables.
pub mod record;
/// Whether this backend realizes the synchronization a routed entry requires.
pub mod synchronization_requirement;
/// Explicit Metal target facts consumed by emission.
pub mod target;

#[cfg(test)]
mod applicability_tests;
#[cfg(test)]
mod direct_requirement_tests;
#[cfg(test)]
mod golden_compilation;
#[cfg(test)]
mod synchronization_requirement_tests;
#[cfg(test)]
mod target_correspondence;
#[cfg(test)]
mod tests;
