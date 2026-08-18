//! The first authoritative macOS Metal compile-time declaration.
//!
//! One value binds five things that are otherwise stated independently and can
//! therefore disagree: the checked compiler [`TargetProfile`], the exact
//! [`MetalTargetFacts`] emission runs against, the
//! [`MetalEmissionRealization`] the translation unit selects, the total
//! projection of those facts onto the AOT driver's [`MetalTarget`], and the
//! structured sources every projected row is attributed to.
//!
//! # Every row comes from the ledger, and only from the ledger
//!
//! [`docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md`](../../../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)
//! enumerates every compile-phase fact the bounded serial-sum path consumes and
//! names, per row, the authority that establishes it. Its first outcome states
//! the constraint this module implements: the bound declaration is constructed
//! *from exactly those rows and no others*. A row the ledger leaves absent stays
//! absent here, and therefore `Unknown`:
//!
//! - **Device address width** has no consumer in the current buffer-relative
//!   kernel IR and no exact authority, so no `CapabilityAxis::DeviceAddressWidthBits`
//!   row is declared. The reconsideration trigger is the first KIR operation
//!   converting between a device pointer and an integer.
//! - **Workgroup threads** is declared as a `PreparedKernelPreflight` *query*
//!   rather than a fact. Apple's feature tables report 1,024 for Apple9 and
//!   footnote the reader to `MTLComputePipelineState.maxTotalThreadsPerThreadgroup`
//!   for the compiled-function maximum; a theoretical family limit is not a
//!   prepared pipeline's capacity.
//! - **Synchronization** carries exactly one row: the workgroup control barrier
//!   over threadgroup memory, realized by
//!   `threadgroup_barrier(mem_flags::mem_threadgroup)` and sourced from MSL 4.0
//!   §6.9.1. It is a whole-subject fact rather than a capacity — a numeric
//!   barrier-count axis was removed outright by
//!   `replace-or-justify-the-barrier-count-axis` and is not what returned. Any
//!   subject differing in even one of its five dimensions still resolves as
//!   `NoPath`, so this row admits the single-workgroup tree and nothing else.
//! - **F16** gets no dispatchability and no numerical row. F32 and BF16 each
//!   carry their own measured dispatchability and subnormal rows; neither is
//!   inherited by the omitted neighbour.
//! - **Evaluation-order preservation** — whether the backend compiler executes
//!   the order the emitted program pins — has a vocabulary
//!   ([`TargetProfileBuilder::declare_measured_evaluation_order_preservation`])
//!   and **no row here**, so this profile resolves it `Unknown` for every
//!   subject and licence. It is measured: [the Apple record's finding
//!   34](../../../docs/research/apple-targets/numerical-behaviour.md) records a
//!   written two-by-two split re-serialized under `relaxed` and `fast` on both
//!   compilation paths and preserved under `safe` in every measured cell. It is
//!   measured on a **different toolchain row**: Xcode 27.0 build `27A5228h`,
//!   macOS SDK 27.0, and an offline `metalfe-32023.921`, where every row this
//!   declaration carries is Xcode 26.6 build `17F113`, SDK 26.5, and an offline
//!   `metalfe-32023.883`. The property is a property *of the backend compiler
//!   build*, and finding 8 records that build moving independently of everything
//!   else, so declaring build `.921`'s behaviour on a profile whose plans build
//!   `.883` compiles would be the inheritance this ledger refuses everywhere
//!   else — a guess wearing a measurement's provenance. The ledger's
//!   "Evaluation-order preservation" row records the deferral and its two
//!   closing measurements, each of which needs a toolchain move this repository
//!   reserves to Tom.
//!
//! # Which authority class each row carries
//!
//! Most quantitative rows are **normative**: they come from primary Apple
//! documents, so they are declared through
//! [`TargetFactSource::external_guarantee`] naming the exact document as a
//! versioned normative reference. The dispatchability, numerical, **and
//! grid-axis** rows are **measured**: they come from retained runs on one exact
//! environment, so they are declared through
//! [`TargetCompileProfileMeasurementSource`], whose phase, authority, and
//! validity are fixed by construction and cannot widen into a portable claim.
//!
//! **The grid axis is the row that crossed that line, and the reason generalizes.**
//! It is consumed as a guarantee — every extent up to the bound is admissible —
//! so it needs an authority stating a floor on capability. Every normative
//! source available states a ceiling on the space instead: the SDK's
//! `dispatchThreads:` contract proves representability without a maximum, the
//! feature tables carry no compute-grid row at all, and MSL 4.0 Table 5.8 caps
//! the addressable grid at `2^32` by typing `[[thread_position_in_grid]]` no
//! wider than `uint`. A ceiling forbids declaring more; it licenses nothing. So
//! the number comes from `spikes/target-profiles/metal-grid-axis-extent` and
//! travels with that run's exact validity.
//!
//! Three normative references rather than one, because three different documents
//! establish the remaining rows and a reader repairing a stale row needs to know
//! which: the Metal Feature Set Tables for the family limits and 64-bit integer
//! math, the MSL 4.0 specification for the `device` address space, and its
//! threadgroup-synchronization section for the barrier realization.
//!
//! # The selected realizations are not capabilities
//!
//! [`MetalEmissionRealization`] and [`NumericalRealization`] travel *with* the
//! declaration because the rows are only true of them — a numerical row read
//! from a `relaxed` or `fast` case would be a different fact about a different
//! compilation — but neither is projected into the compiler profile.
//! [`BoundMetalCompileDeclaration`]'s own documentation carries the three
//! compile-fail proofs that a selected launch declaration establishes neither
//! grid capacity, nor index-arithmetic support, nor device-address width.
//!
//! Every public item here is a reviewed *draft* boundary (ADR 0074 §7 / ADR
//! 0075): built and tested at full fidelity while Tom reviews the surface.

use core::fmt;
use std::error::Error;

use tiler_artifact::program::{
    ArtifactBuildError, TargetProfileDescriptorDigest,
    TargetProfileKey as ArtifactTargetProfileKey, TargetProfileRef,
};
use tiler_compiler::target::{
    DTypeDispatchability, DTypeDispatchabilityResolution, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, SynchronizationSupport, TargetCompileProfileMeasurementSource,
    TargetCompilerBuild, TargetCompilerRole, TargetCompilerRoleIdentity,
    TargetExecutionEnvironment, TargetFactProducerIdentity, TargetFactSource,
    TargetMeasurementContext, TargetNormativeReferenceIdentity, TargetProfile,
    TargetProfileBuildError, TargetProfileBuilder, TargetProfileKey, TargetProfileKeyError,
    WorkgroupTreeWidthPolicy,
};
use tiler_ir::numerics::{ScalarArithmeticSubjectError, registered_arithmetic_value_type};
use tiler_ir::program::abi::{
    AvailabilityPhase, TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
};
use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FencedSpaces,
    MemoryOrdering, NumericalPermission, SynchronizationKind, SynchronizationScope,
    SynchronizationSubject,
};
use tiler_ir::semantic::{Bf16, F32};
use tiler_metal::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalEmissionRealization,
    MetalFloatArithmeticType, MetalFlushedZeroSign, MetalPlatform, MetalSubnormalArithmetic,
    MetalSubnormalArithmeticFacts, MetalTargetFacts, MslLanguageVersion,
};
use tiler_metal_aot::input::{MetalTarget, MetalTargetError, NumericalRealization};

use crate::metal_assembly::compile_target;
use crate::{MetalF32TargetProfileError, declare_metal_f32_subnormal_behaviour};

/// The ledger's offline compilation environment, one exact component per field.
///
/// Four components rather than one "toolchain" string, because the ledger
/// tabulates four and a reader diagnosing a moved row needs to know which one
/// moved. The Xcode distribution and the SDK carry producer-defined roles: they
/// are not compilers, and folding them into the code-generator's version string
/// would make two different changes look like one.
///
/// `pub(crate)` because [`crate::metal_subgroup_declaration`] transcribes the
/// same four offline components for a measurement whose *execution* host
/// differs; sharing the row shape keeps "same toolchain, different device" a
/// statement two records make in one vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OfflineToolchainRow {
    pub(crate) compiler_version: &'static str,
    pub(crate) compiler_build: &'static str,
    pub(crate) linker_version: &'static str,
    pub(crate) linker_build: &'static str,
    pub(crate) xcode_version: &'static str,
    pub(crate) xcode_build: &'static str,
    pub(crate) sdk_version: &'static str,
    pub(crate) sdk_build: &'static str,
}

/// The ledger's execution environment, the host that ran the measured kernels.
///
/// `pub(crate)` for the same reason as [`OfflineToolchainRow`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExecutionRow {
    pub(crate) platform: &'static str,
    pub(crate) platform_version: &'static str,
    pub(crate) platform_build: &'static str,
    pub(crate) architecture: &'static str,
    pub(crate) hardware: &'static str,
}

/// Every ledger row this declaration is assembled from, in one place.
///
/// Private, and taken by [`BoundMetalCompileDeclaration::declare`] rather than
/// read from constants inline, for one reason: the identity mutation cases the
/// owning ticket requires must be able to move exactly one row and observe the
/// descriptor move with it. A construction that inlined the constants would
/// leave "the descriptor depends on this row" untestable, which is the same as
/// unproven.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LedgerRows {
    profile_key: &'static str,
    bf16_dispatchability: Option<DTypeDispatchability>,
    grid_axis_threads: u64,
    workgroup_property_key: &'static str,
    buffer_bindings: u32,
    index_arithmetic: IndexArithmeticSupport,
    device_address_space: bool,
    local_memory_bytes: u64,
    /// The one complete synchronization subject this target realizes.
    ///
    /// Stated literally rather than derived from a strategy's own tile, and the
    /// difference is the whole point of a *capability* row. Deriving it from
    /// `workgroup_tree_tile` would make the profile agree with whatever the
    /// strategy happens to ask for, which is a profile that cannot refuse;
    /// `TargetProfile::workgroup_tree_target_for_test` derives it precisely
    /// because a *test* profile must not be able to over-declare, and a
    /// production profile has the opposite obligation. A strategy requiring any
    /// other subject resolves `NoPath` against this row, which is correct.
    synchronization: SynchronizationSubject,
    /// The verdict on that subject, carried apart from the subject itself.
    ///
    /// Separate so a mutation case can move the verdict without moving the
    /// subject: a target that names the right realization and refuses it is a
    /// different fact from one that names a realization nobody asked about, and
    /// the two produce different typed rejections.
    synchronization_support: SynchronizationSupport,
    /// Fold steps this device retires at once when its launch saturates it.
    ///
    /// A **cost row**, and the only one this ledger carries. It is not a
    /// capability: nothing reads it for feasibility, and a profile omitting it
    /// states no preference rather than no plan. `Option` for exactly that
    /// reason — the mutation cases must be able to remove the row and observe a
    /// profile that still compiles every program it compiled before, selecting
    /// what it selected before.
    saturated_parallel_fold_steps: Option<u64>,
    /// Closed tree-width policy the single-workgroup tree may run under.
    ///
    /// `Option` so a mutation case can omit it and observe a typed tree
    /// decline rather than an inherited `256`. This is not a public numeric
    /// cap.
    workgroup_tree_width_policy: Option<WorkgroupTreeWidthPolicy>,
    facts: MetalTargetFacts,
    emission: MetalEmissionRealization,
    numerical: NumericalRealization,
    offline: OfflineToolchainRow,
    execution: ExecutionRow,
}

/// The exact rows the authority ledger admits for this profile.
///
/// Each value is transcribed from the ledger section named beside it. Changing
/// one here is a claim about a source, not a tuning knob.
const FIRST_MACOS_APPLE9: LedgerRows = LedgerRows {
    // The profile is keyed by what bounds every row in it: the macOS artifact
    // family, the Apple9 GPU family whose feature-table column supplies the
    // quantitative limits, the MSL 4.0 standard, and the two dtypes with
    // dispatchability and numerical evidence. This is a new content key rather
    // than a revision of the old F32-only family: keeping `.f32` in a key for a
    // profile that also states BF16 would make the key's own description false.
    profile_key: "tiler.metal.macos-apple9.msl4-0.f32-bf16.v1",
    // The unified MSL 4.0/macOS 26 record reports Apple9 bfloat support and
    // executes the BF16 kernels on this exact host/toolchain row.
    bf16_dispatchability: Some(DTypeDispatchability::Dispatchable),
    // "Grid-axis threads — 268,435,456, measured".
    //
    // **The row is a guarantee, so its authority has to be a floor, and every
    // normative source available is a ceiling.** Feasibility reads this row as
    // *every dispatch with axis extent at most this works*. The superseded
    // value was 4, sourced from the macOS SDK's `dispatchThreads:` contract —
    // which proves an extent is *representable* and states no maximum at all,
    // so it licensed no number and four was chosen to cover a program. The
    // Metal Feature Set Tables carry no compute-grid row (only object- and
    // mesh-shader grids), and MSL 4.0 Table 5.8 types
    // `[[thread_position_in_grid]]` as `ushort` or `uint` and nothing wider,
    // which caps the addressable grid at `2^32` without licensing any value
    // inside it. A bounded measurement is the only class that can supply a
    // floor, so this row is measured and carries the measurement's own
    // validity rather than a normative reference.
    //
    // `spikes/target-profiles/metal-grid-axis-extent` dispatched a ladder of
    // extents through this profile's own compilation, launch realization, and
    // dispatch route on the exact offline and execution environments below,
    // verifying every slot of a poisoned buffer at three threadgroup widths.
    // All 6,294 rows completed and verified; `2^28` is the widest extent
    // verified at every width, and it is the run's stop condition rather than
    // an observed limit — nothing measured a failure, so nothing here says
    // where one is.
    grid_axis_threads: 268_435_456,
    // "Workgroup threads — absent as a fact, declared as a prepared-kernel query".
    workgroup_property_key: "tiler.target.prepared-entry.max-threads-per-workgroup.v1",
    // "Buffer bindings per entry — 31", feature tables, `Apple9` column.
    buffer_bindings: 31,
    // "Index arithmetic — `CompleteU64`", feature tables row `64-bit integer math`.
    index_arithmetic: IndexArithmeticSupport::CompleteU64,
    // "Device address space — available", MSL 4.0's `device` address space.
    device_address_space: true,
    // "Local memory bytes — 32,768", feature tables, `Apple9` column, 32 KB.
    local_memory_bytes: 32_768,
    // "Synchronization — the workgroup control barrier over threadgroup memory".
    // Realized by `threadgroup_barrier(mem_flags::mem_threadgroup)`, from MSL 4.0
    // §6.9.1 and its two tables. Four of the five dimensions are quoted normative
    // facts and the fifth is an elimination, and the ledger row states which is
    // which rather than presenting one authority for all five:
    //
    // - `kind`: Table 6.12 — every thread of the threadgroup "need[s] to execute
    //   this function before any thread can continue execution beyond the
    //   threadgroup_barrier", and §6.9.1 calls it "an execution and memory
    //   barrier". That is the control-barrier contract.
    // - `execution_scope`: Table 6.12, "All threads in a threadgroup".
    // - `visibility_scope`: Table 6.13, `mem_threadgroup` orders threadgroup
    //   memory operations "for threads in a threadgroup".
    // - `fenced_spaces`: the emitted flag is exactly `mem_threadgroup`.
    //   `mem_device` is a separate flag this realization does not pass, so the
    //   device domain is deliberately false rather than conservatively true — a
    //   superset fence is a different realization with a different cost.
    // - `ordering`: an **inference**, and the one row here that is not a
    //   quotation. MSL declares `enum memory_order { memory_order_relaxed,
    //   memory_order_seq_cst }` (§6.15.1) and applies it to atomics and
    //   `atomic_thread_fence` (§6.15.3), never to `threadgroup_barrier`, so no
    //   sentence assigns this barrier an ordering. `Relaxed` is refuted by
    //   §6.9.1's memory fence "for reads and writes"; `SequentiallyConsistent`
    //   is withheld, being what the spec reserves for an explicit
    //   `memory_order_seq_cst` fence; `AcquireRelease` is what remains and is
    //   exactly the quoted content.
    synchronization: SynchronizationSubject {
        kind: SynchronizationKind::ControlBarrier,
        execution_scope: SynchronizationScope::Workgroup,
        visibility_scope: SynchronizationScope::Workgroup,
        fenced_spaces: FencedSpaces {
            workgroup: true,
            device: false,
        },
        ordering: MemoryOrdering::AcquireRelease,
    },
    synchronization_support: SynchronizationSupport::Realized,
    // "Saturated parallel fold steps — 1,056, measured".
    //
    // **Measurement, 2026-08-07** —
    // `spikes/program-planning/reduction-dispatch-crossover`, retained at
    // `results/2026-08-07-apple-m4-max-macos27.0-26A5388g/`, on a host matching
    // this ledger's offline and execution rows in every field. The sweep timed
    // all three reduction strategies over a 92-cell matrix, 276 dispatched
    // alternatives, and fitted a three-parameter work-span model
    // `sum over stages of ( encoder + max(work / P, depth) * step )` on the
    // perfect-square contributor counts. `P` fits at `1.056e3` and is the number
    // declared here: the fold steps the device retires at once when saturated.
    //
    // **This is a cost row and deliberately not a capability axis.** Every
    // `CapabilityAxis` is a hard bound whose silence is an `Unknown` that never
    // reaches an executable frontier, and
    // `docs/research/program-planning/flash-class-capability-set.md` already
    // eliminated that shape for a bandwidth number. Declared as an axis this row
    // would make silence render a profile unexecutable for a quantity no
    // feasibility predicate reads — the wrong failure direction. It is declared
    // through the same `TargetCompileProfileMeasurementSource` the grid-axis,
    // dispatchability, and numerical rows carry, so its validity stays
    // `MeasuredEnvironment` and cannot widen into a portable claim.
    //
    // **What the measurement establishes, and what it does not.** The model
    // reproduces the measured verdict on 24 of the 26 held-out cells whose
    // serial-or-parallel verdict is separated, worst measured penalty 1.81x, and
    // the sweep's own perturbation table shows that *only* `P` moves a decision:
    // scaling `encoder` by twenty or `step` by a tenth leaves every predicted
    // winner unchanged, while scaling `P` by a quarter drops agreement to 20 of
    // 26 and the worst penalty to 3.04x. **`P` is determined only to about a
    // factor of four** — quadrupling it leaves fit-set agreement where it was and
    // *improves* the held-out worst penalty to 1.20x — so this is a contour
    // position rather than a tight constant, and it is a quantity of this host
    // row alone. Another Apple family, OS row, dtype, or device declares its own
    // or declares none.
    saturated_parallel_fold_steps: Some(1_056),
    // "Workgroup-tree-width policy — MeasuredNearestCap256V1, measured".
    //
    // **Measurement, 2026-08-07** —
    // `spikes/program-planning/reduction-partition-calibration`, retained at
    // `results/2026-08-07-apple-m4-max-macos27.0-26A5388g/`, on a host matching
    // this ledger's offline and execution rows in every field. The sweep selected
    // the nearest-admissible-width rule around the fixed internal 256. The
    // number stays private to that rule; this row declares the closed policy,
    // not a public cap. A profile omitting the policy does not inherit 256.
    workgroup_tree_width_policy: Some(WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1),
    // "Metal target facts, and which of them project". The deployment minimum is
    // 26.0 and the standard MSL 4.0 because `probe.fixed_flags -std=metal4.0`
    // and `requested_target air64-apple-macos26.0` are the inputs the retained
    // measurement used; the older MSL 3.1 / macOS 14.0 record would attribute
    // these measurements to a compilation that did not produce them. F32 and
    // BF16 are stated independently; F16 was not measured under MSL 4.0.
    facts: MetalTargetFacts::new(
        MslLanguageVersion::Metal4_0,
        MetalPlatform::MacOs,
        MetalDeploymentMinimum::new(26, 0),
        MetalSubnormalArithmeticFacts::unmeasured()
            .stating(
                MetalFloatArithmeticType::F32,
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
            )
            .stating(
                MetalFloatArithmeticType::Bf16,
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
            ),
        31,
    ),
    // "Selected, not a capability": MSL 4.0 Table 5.8 permits `ushort` or `uint`
    // for `[[thread_position_in_grid]]`; Tiler selects `uint`.
    emission: MetalEmissionRealization::new(LaunchIndexRealization::ThreadPositionInGridUInt),
    // "The flags are part of the row": the retained cases this ledger reads are
    // the `safe`, `contract-off` ones, which is exactly this realization.
    numerical: NumericalRealization::strict_baseline(),
    // "The offline compilation environment" table.
    offline: OfflineToolchainRow {
        compiler_version: "32023.883",
        compiler_build: "metalfe-32023.883",
        linker_version: "32023.883",
        linker_build: "AIR-LLD 32023.883 (metalfe-32023.883)",
        xcode_version: "26.6",
        xcode_build: "17F113",
        sdk_version: "26.5",
        sdk_build: "25F70",
    },
    // "The execution environment" table.
    execution: ExecutionRow {
        platform: "macos",
        platform_version: "27.0",
        platform_build: "26A5388g",
        architecture: "arm64",
        hardware: "Apple M4 Max",
    },
};

/// Producer of every normatively sourced row in this declaration.
const NORMATIVE_PRODUCER: &str = "tiler.metal.first-macos-apple9-msl4.normative.v1";
/// Producer of every measured row in this declaration.
const MEASURED_PRODUCER: &str = "tiler.metal.first-macos-apple9-msl4.measured.v1";
/// Apple's Metal Feature Set Tables, the vendored 2025-10-20 revision.
const FEATURE_TABLES_REFERENCE: &str = "apple.metal-feature-set-tables.2025-10-20";
/// The MSL 4.0 specification's address-space chapter.
const MSL_ADDRESS_SPACE_REFERENCE: &str = "apple.metal-shading-language.4-0.device-address-space";
/// The MSL 4.0 specification's threadgroup-synchronization section.
///
/// A reference of its own rather than a second use of the address-space one: a
/// reader repairing a stale synchronization row needs §6.9.1 and its two tables,
/// not the chapter that establishes the `device` address space, and two rows
/// sharing one reference would leave neither able to say which section moved.
const MSL_BARRIER_REFERENCE: &str = "apple.metal-shading-language.4-0.threadgroup-barrier";
/// Provider of the prepared-entry property the workgroup query names.
///
/// `pub(crate)`: the subgroup-width query in
/// [`crate::metal_subgroup_declaration`] is answered by the same provider
/// family, and two spellings of one provider would let them drift apart.
pub(crate) const PREPARED_ENTRY_PROVIDER_NAMESPACE: &str = "tiler";
/// Property family the prepared-entry workgroup query is answered from.
pub(crate) const PREPARED_ENTRY_PROVIDER_NAME: &str = "prepared-entry-properties";
/// Producer-defined compiler role naming the Xcode distribution.
pub(crate) const OFFLINE_DISTRIBUTION_ROLE: &str = "tiler.metal.offline-toolchain-distribution";
/// Producer-defined compiler role naming the platform SDK.
pub(crate) const OFFLINE_SDK_ROLE: &str = "tiler.metal.offline-platform-sdk";

/// One checked, versioned macOS Metal compile-time declaration.
///
/// Constructed only by [`Self::first_macos_apple9`]: there is no public
/// constructor taking rows, because a caller minting a profile for a row nobody
/// measured is exactly what the authority ledger exists to prevent. Widening
/// this to another Apple family, OS row, or dtype is a new measurement rather
/// than a new argument.
///
/// # A selected launch declaration is not a launch-capacity fact
///
/// ```compile_fail
/// use tiler_compiler::target::{
///     TargetFactSource, TargetProfileBuilder, TargetProfileKey,
/// };
/// use tiler_metal::target::{LaunchIndexRealization, MetalEmissionRealization};
///
/// let emission = MetalEmissionRealization::new(
///     LaunchIndexRealization::ThreadPositionInGridUInt,
/// );
/// let source: TargetFactSource = unimplemented!();
/// let mut profile =
///     TargetProfileBuilder::new(TargetProfileKey::new("example.target.v1".to_owned()).unwrap());
/// profile
///     .declare_max_threads_per_grid_axis(emission, source)
///     .unwrap();
/// ```
///
/// # Nor is it index-arithmetic support
///
/// ```compile_fail
/// use tiler_compiler::target::{
///     TargetFactSource, TargetProfileBuilder, TargetProfileKey,
/// };
/// use tiler_metal::target::{LaunchIndexRealization, MetalEmissionRealization};
///
/// let emission = MetalEmissionRealization::new(
///     LaunchIndexRealization::ThreadPositionInGridUInt,
/// );
/// let source: TargetFactSource = unimplemented!();
/// let mut profile =
///     TargetProfileBuilder::new(TargetProfileKey::new("example.target.v1".to_owned()).unwrap());
/// profile
///     .declare_index_arithmetic(emission, source)
///     .unwrap();
/// ```
///
/// # Nor a device-address width
///
/// ```compile_fail
/// use tiler_compiler::target::{
///     TargetFactSource, TargetProfileBuilder, TargetProfileKey,
/// };
/// use tiler_metal::target::{LaunchIndexRealization, MetalEmissionRealization};
///
/// let emission = MetalEmissionRealization::new(
///     LaunchIndexRealization::ThreadPositionInGridUInt,
/// );
/// let source: TargetFactSource = unimplemented!();
/// let mut profile =
///     TargetProfileBuilder::new(TargetProfileKey::new("example.target.v1".to_owned()).unwrap());
/// profile
///     .declare_device_address_width(emission, source)
///     .unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct BoundMetalCompileDeclaration {
    profile: TargetProfile,
    facts: MetalTargetFacts,
    emission: MetalEmissionRealization,
    numerical: NumericalRealization,
    aot_target: MetalTarget,
}

impl BoundMetalCompileDeclaration {
    /// Assembles the first authoritative macOS Apple9 MSL 4.0 declaration.
    ///
    /// # Errors
    ///
    /// Returns the exact refusing authority: an invalid profile key, a rejected
    /// provenance identity, a compiler-profile construction refusal, the
    /// buffer-capacity overlap check, either dtype's subnormal projection, or
    /// the AOT driver's own target validation.
    pub fn first_macos_apple9() -> Result<Self, BoundMetalDeclarationError> {
        Self::declare(&FIRST_MACOS_APPLE9)
    }

    /// A second *artifact family* over this profile's rows, for tests only.
    ///
    /// `#[cfg(test)]` and crate-private: it is unreachable from any dependent
    /// crate and from every production path in this one, which is the whole of
    /// its safety. It exists so a test can ask what happens when one selection
    /// names two artifact families, without inventing a measured row.
    ///
    /// **It is not a second measured declaration, and could not be.** It moves
    /// exactly one field — `MetalTargetFacts::platform` — which the ledger's
    /// projection table records as *not* projecting into the compiler profile.
    /// So this shares `first_macos_apple9`'s profile key, descriptor, measured
    /// dispatchability, and every numerical row, and differs only in the AOT
    /// target it compiles for: `air64-apple-ios26.0` rather than
    /// `air64-apple-macos26.0`. Those measured rows were taken on a macOS host,
    /// and the ledger refuses their inheritance by name — "No iOS family,
    /// physical or simulated, gains a row from this one" — which is exactly why
    /// this may not escape `cfg(test)`.
    ///
    /// `first-authoritative-ios-metal-compile-declaration` is the ticket that
    /// would produce a real one, and it is blocked on a measurement.
    #[cfg(test)]
    pub(crate) fn second_artifact_family_fixture() -> Result<Self, BoundMetalDeclarationError> {
        let mut rows = FIRST_MACOS_APPLE9;
        rows.facts = MetalTargetFacts::new(
            rows.facts.language,
            MetalPlatform::IOsDevice,
            rows.facts.deployment_minimum,
            rows.facts.subnormal_arithmetic,
            rows.facts.buffer_binding_limit,
        );
        Self::declare(&rows)
    }

    /// Returns the checked compiler profile every projected row entered.
    #[must_use]
    pub const fn profile(&self) -> &TargetProfile {
        &self.profile
    }

    /// Returns the exact Metal target facts emission runs against.
    ///
    /// Only the buffer capacity and the F32/BF16 subnormal entries of this
    /// record project into [`Self::profile`]. The language standard, artifact
    /// family, and deployment minimum have no compiler counterpart and must
    /// never be described as compiler-assessed.
    #[must_use]
    pub const fn metal_facts(&self) -> &MetalTargetFacts {
        &self.facts
    }

    /// Returns the source-level realization the translation unit selects.
    ///
    /// A selection carried by the emission unit, not a target capability. It
    /// affects payload identity and proves nothing about grid capacity,
    /// arithmetic support, or address width.
    #[must_use]
    pub const fn emission(&self) -> MetalEmissionRealization {
        self.emission
    }

    /// Returns the numerical realization every measured row is scoped to.
    #[must_use]
    pub const fn numerical_realization(&self) -> NumericalRealization {
        self.numerical
    }

    /// Returns the dtype-dispatchability verdicts [`Self::profile`] declares, in
    /// [`ArithmeticType`]'s canonical order.
    ///
    /// # Why a consumer reads this instead of transcribing the ledger
    ///
    /// A consumer that states which dtypes it can dispatch has to get the rows
    /// from somewhere, and the two candidates are not equivalent. Transcribing
    /// this module's `FIRST_MACOS_APPLE9` rows into a call-site literal makes the consumer a
    /// second authority over rows this declaration already owns, so a widened,
    /// narrowed, or retracted measurement leaves the copy stating a verdict the
    /// profile no longer holds. Reading them here cannot drift: the answer comes
    /// from the same [`TargetProfile`] the compile gate consults, through the
    /// same lookup.
    ///
    /// # Silence is omitted rather than defaulted
    ///
    /// Only an exact declaration produces a row. A dtype the profile resolves
    /// `Unknown` — `f16`, which this ledger deliberately does not measure — and
    /// one it resolves `Deferred` are both **absent** from the result, never
    /// present with a permissive verdict. That keeps a consumer's fail-closed
    /// rule intact: a row it never receives is a dtype it never claims, and the
    /// runtime's own `Unknown` refuses exactly as `Unsupported` does.
    ///
    /// `Deferred` is dropped rather than reported for the reason the runtime
    /// vocabulary has no spelling for it: an answer that only resolves after a
    /// later phase is one this consumer cannot hold *now*, and stating it as a
    /// verdict would offer a fact the phase it names has not yet produced.
    ///
    /// # The phase is the compile profile, and that is the whole of its authority
    ///
    /// [`AvailabilityPhase::CompileProfile`] is the phase
    /// `tiler_compiler`'s own request admission resolves this fact at, so a row
    /// returned here is exactly the row that decided whether a program in that
    /// dtype could be compiled for this target at all. It is **not** an
    /// observation about any host: nothing here binds a device, and ADR 0086
    /// keeps the applicability question a host would have to answer refused on
    /// every macOS row.
    #[must_use]
    pub fn dtype_dispatchability_rows(&self) -> Vec<(ArithmeticType, DTypeDispatchability)> {
        ArithmeticType::ALL
            .into_iter()
            .filter_map(|arithmetic| {
                let resolved_type = registered_arithmetic_value_type(arithmetic)?;
                let verdict = match self
                    .profile
                    .dtype_dispatchability(&resolved_type, AvailabilityPhase::CompileProfile)
                {
                    DTypeDispatchabilityResolution::Dispatchable => {
                        DTypeDispatchability::Dispatchable
                    }
                    DTypeDispatchabilityResolution::Unsupported => {
                        DTypeDispatchability::Unsupported
                    }
                    DTypeDispatchabilityResolution::Deferred { .. }
                    | DTypeDispatchabilityResolution::Unknown => return None,
                };
                Some((arithmetic, verdict))
            })
            .collect()
    }

    /// Returns the total projection of [`Self::metal_facts`] onto the AOT driver.
    ///
    /// Resolved once at declaration time, so a family, standard, and deployment
    /// minimum that do not form a governed compiler target refuse here rather
    /// than after emission has already run.
    #[must_use]
    pub const fn aot_target(&self) -> MetalTarget {
        self.aot_target
    }

    /// Returns the artifact-vocabulary reference to this exact profile.
    ///
    /// Key *and* exact descriptor, because ADR 0043 makes a key alone
    /// insufficient evidence that a variant is legal against a profile revision.
    ///
    /// # Errors
    ///
    /// Returns the artifact layer's typed refusal if this profile's key or
    /// descriptor exceeds an artifact identity bound.
    pub fn target_profile_ref(&self) -> Result<TargetProfileRef, ArtifactBuildError> {
        Ok(TargetProfileRef {
            key: ArtifactTargetProfileKey::new(self.profile.profile_key().as_str())?,
            descriptor: TargetProfileDescriptorDigest::from_bytes(
                self.profile.canonical_descriptor(),
            )?,
        })
    }

    /// Reports whether a compilation was assessed against exactly this profile.
    ///
    /// Key first, then descriptor, matching the runtime's own classification
    /// order: a different key is a compilation for another target family, while
    /// the same key under another descriptor is this family under a profile
    /// revision this declaration does not carry.
    ///
    /// # Errors
    ///
    /// Returns the exact half that disagreed.
    pub fn require_compiled_under(
        &self,
        compiled_key: &str,
        compiled_descriptor: &[u8],
    ) -> Result<(), MetalPlanProfileMismatch> {
        let declared_key = self.profile.profile_key().as_str();
        if compiled_key != declared_key {
            return Err(MetalPlanProfileMismatch::ProfileKey {
                declared: declared_key.to_owned(),
                compiled: compiled_key.to_owned(),
            });
        }
        if compiled_descriptor != self.profile.canonical_descriptor() {
            return Err(MetalPlanProfileMismatch::ProfileDescriptor {
                key: declared_key.to_owned(),
                declared_bytes: self.profile.canonical_descriptor().len(),
                compiled_bytes: compiled_descriptor.len(),
            });
        }
        Ok(())
    }

    fn declare(rows: &LedgerRows) -> Result<Self, BoundMetalDeclarationError> {
        let normative = NormativeSources::declare(rows)?;
        let measured = measured_source(rows)?;

        let mut builder =
            TargetProfileBuilder::new(TargetProfileKey::new(rows.profile_key.to_owned())?);

        // ---- quantitative rows -----------------------------------------------
        // Five of the six are normatively sourced. The grid axis is not, and it
        // is the one row whose class the ledger had to change: an authority that
        // caps the space cannot fill a row consumed as a guarantee, so this one
        // carries the same `TargetCompileProfileMeasurementSource` the
        // dispatchability and numerical rows do — the same one, not a second,
        // because the extent ladder ran on exactly the offline and execution
        // environments those rows were taken on.
        builder
            .declare_measured_max_threads_per_grid_axis(rows.grid_axis_threads, measured.clone())?;
        builder.declare_max_threads_per_workgroup_query(
            TargetPropertyQuery::new(
                TargetPropertyKey::new(rows.workgroup_property_key)
                    .map_err(|_| BoundMetalDeclarationError::PreparedEntryQuery)?,
                AvailabilityPhase::PreparedKernelPreflight,
                TargetPropertyProviderIdentity::new(
                    PREPARED_ENTRY_PROVIDER_NAMESPACE,
                    PREPARED_ENTRY_PROVIDER_NAME,
                    1,
                )
                .map_err(|_| BoundMetalDeclarationError::PreparedEntryQuery)?,
            )
            .map_err(|_| BoundMetalDeclarationError::PreparedEntryQuery)?,
        )?;
        builder.declare_max_buffer_bindings_per_entry(
            rows.buffer_bindings,
            normative.feature_tables.clone(),
        )?;
        builder
            .declare_index_arithmetic(rows.index_arithmetic, normative.feature_tables.clone())?;
        builder.declare_device_memory(rows.device_address_space, normative.msl_address_space)?;
        builder.declare_local_memory_bytes(rows.local_memory_bytes, normative.feature_tables)?;

        // ---- the one synchronization row, normatively sourced ---------------
        // Declared at `CompileProfile` because that is the phase the fact is
        // true at: MSL defines the barrier's contract in the language, so
        // nothing about a live device or a prepared pipeline is consulted to
        // know it. That is what separates this row from the workgroup-threads
        // one above, which is a query precisely because its value does not
        // exist until a pipeline does.
        //
        // One subject, one verdict. A target realizing a second subject would
        // declare a second fact; there is deliberately no per-dimension
        // spelling, so this conjunction can never be assembled out of five
        // facts none of which is about the barrier.
        builder.declare_synchronization_realization(
            rows.synchronization,
            rows.synchronization_support,
            &normative.msl_barrier,
        )?;

        // ---- the one cost row, measured -------------------------------------
        // Not a capability, and the section is separate so a reader cannot mistake
        // it for one: the rows above are hard bounds a feasibility predicate
        // reads, and this one is a preference physical selection consults. A
        // profile omitting it states *no preference* and selects exactly as it did
        // before the row existed, which is why the field is an `Option` and why
        // the descriptor section it encodes into is written only when it holds a
        // row.
        //
        // The same measured source, not a second one: the dispatch sweep ran on
        // exactly the offline and execution environments the rows above were taken
        // on, so a second source would claim a second population that does not
        // exist.
        if let Some(steps) = rows.saturated_parallel_fold_steps {
            builder.declare_measured_saturated_parallel_fold_steps(steps, measured.clone())?;
        }

        // ---- the one tree-width policy, measured ---------------------------
        // Not a cost row and not a capability. Silence makes the tree
        // unavailable; it does not inherit 256 or substitute the balanced
        // partition. The same measured source, not a second one: the partition
        // calibration ran on exactly the offline and execution environments
        // the rows above were taken on.
        if let Some(policy) = rows.workgroup_tree_width_policy {
            builder.declare_measured_workgroup_tree_width_policy(policy, measured.clone())?;
        }

        // ---- dispatchability, measured and exact ----------------------------
        // The retained run dispatched `f32` compute kernels on the execution
        // environment above and read results back, with execution witnesses on
        // non-subnormal operands separating "the arithmetic ran" from "the
        // kernel was optimized away". F32 and BF16 are independent rows; F16
        // is deliberately absent.
        builder.declare_measured_dtype_dispatchability(
            F32::resolved_type(),
            DTypeDispatchability::Dispatchable,
            measured.clone(),
        )?;
        if let Some(dispatchability) = rows.bf16_dispatchability {
            builder.declare_measured_dtype_dispatchability(
                Bf16::resolved_type(),
                dispatchability,
                measured.clone(),
            )?;
        }

        // ---- projected numerical overlaps: exact dtype subnormal rows -------
        // F32 uses the ratified low-level seam; BF16 stays private to this bound
        // declaration. Both read the Metal record rather than restating its mode
        // and both install complete exclusive input/result tables.
        declare_metal_f32_subnormal_behaviour(&mut builder, &rows.facts, measured.clone())?;
        declare_metal_bf16_subnormal_behaviour(&mut builder, &rows.facts, measured.clone())?;

        // ---- the remaining measured numerical rows --------------------------
        // Each is what the *selected* realization delivered through the *exact*
        // offline compiler, isolated by the retained record's per-case
        // `float_operations` field rather than by the flag names on the command
        // line: `safe` cases emit bare operations, `relaxed` adds
        // `reassoc nsz arcp afn`, `fast` adds `nnan ninf`, and `+contract`
        // tracks contraction independently.
        let f32 = ScalarArithmetic::f32();
        builder.declare_measured_contraction(
            f32.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            measured.clone(),
        )?;
        builder.declare_measured_reassociation(
            f32.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            measured.clone(),
        )?;
        // The *other* resolution of the same dimension, from the same retained
        // case read for its other consequence — not a second measurement and not
        // a widening of the row above.
        //
        // `reassociation_chain` shows that under `safe`/`contract-off` the
        // offline compiler emits no `reassoc` attribute and returns the source's
        // own fold order, where `relaxed` and `fast` return the regrouped one.
        // That single observation answers two different questions. Asked "can a
        // contract that *forbids* regrouping be delivered here?" it says yes,
        // because the compiler adds none. Asked "can a contract that *permits*
        // regrouping be delivered here?" it says yes *exactly*, and for the same
        // reason: the permission licenses Tiler to choose a grouping, the chosen
        // grouping is what the emitted source expresses, and the target runs that
        // one rather than substituting another. A permitted contract is honoured
        // by delivering some legal grouping, and this target delivers the one
        // Tiler selected.
        //
        // Declaring both resolutions of a permission dimension is the governed
        // profile's own idiom (`governed_target_honourability` declares both for
        // contraction and reassociation), and it is not the exclusive-table shape
        // the subnormal dimensions use: a target flushes or preserves and cannot
        // do both, whereas `Forbidden` and `Permitted` name two caller contracts
        // that one non-reassociating target satisfies at once.
        //
        // This row is what makes the two parallel reduction strategies reachable
        // on this profile at all: both a multi-pass split and a single-workgroup
        // tree regroup the declared contributor sequence, so both are refused by
        // name on a profile that answers only `Forbidden`.
        builder.declare_measured_reassociation(
            f32.clone(),
            NumericalPermission::Permitted,
            ScalarSupport::Exact,
            measured.clone(),
        )?;
        // Isolated by a retained pair, not inferred from its neighbours. The
        // `permutation_chain` and `permutation_chain_reordered` kernels carry the
        // same three contributors in two orders and differ in nothing else, so
        // under `safe` their results — `00000000` and `40000000` on every lane,
        // from three bare `fadd`s each — are separated by contributor order
        // alone. The permuted value is the discriminator because reassociating
        // the canonical order cannot reach it: four leaves have exactly five
        // parenthesizations and the harness enumerates all five for every
        // operand. The reassociation row above therefore does not stand in for
        // this one, which is what ADR 0014's separate permissions require.
        builder.declare_measured_permutation(
            f32.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            measured.clone(),
        )?;
        builder.declare_measured_signed_zero(
            f32.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            measured.clone(),
        )?;
        // The two elementary dimensions, each declared at its *strict*
        // resolution only. The ledger's attribute-string section is the
        // isolation: across all 688 retained cases the `safe` strings carry
        // no `arcp` — LLVM's allow-reciprocal relaxation, the licence a
        // reciprocal substitution needs — over a retained population that
        // includes a bare `fdiv`, and no `afn` — allow-approximate-functions
        // — anywhere. So the selected `safe`/`contract-off` realization
        // delivers a forbidding contract exactly: the compiler adds no
        // substitution and no approximation to the operations Tiler emits.
        //
        // The widened resolutions — `Permitted` reciprocal replacement and the
        // `BackendElementary` envelope — are deliberately *not* declared, and
        // stay `Unknown` with the ledger recording the reconsideration
        // trigger: no retained case isolates a delivered substitution or an
        // approximate intrinsic on this exact toolchain row, and a row read
        // from the governed profile's delivered-realization argument rather
        // than from a retained measurement would be a guess wearing a
        // measurement's provenance. A contract authorizing either freedom is
        // therefore refused by name on this profile, which is the fail-closed
        // answer rather than an invented one.
        builder.declare_measured_reciprocal_transform(
            f32.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            measured.clone(),
        )?;
        builder.declare_measured_approximate_intrinsics(
            f32.clone(),
            ApproximationEnvelope::Forbidden,
            ScalarSupport::Exact,
            measured.clone(),
        )?;
        builder.declare_measured_nan_assumptions(
            f32.clone(),
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            measured.clone(),
        )?;
        builder.declare_measured_infinity_assumptions(
            f32,
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            measured,
        )?;

        let profile = builder.build()?;

        // ---- the other genuine overlap: buffer capacity ---------------------
        // The compiler's offered capacity must be no greater than the emission
        // limit, or the compiler would admit a signature the emitter must then
        // reject. Checked one-directionally rather than for equality, because a
        // conservative compiler capacity below the emitter's is sound while the
        // reverse is not.
        if u64::from(rows.buffer_bindings) > u64::from(rows.facts.buffer_binding_limit) {
            return Err(
                BoundMetalDeclarationError::BufferCapacityExceedsEmissionLimit {
                    compiler: rows.buffer_bindings,
                    emission: rows.facts.buffer_binding_limit,
                },
            );
        }

        // Nothing else is an overlap. Language, artifact family, and deployment
        // minimum have no compiler counterpart, and validating them against one
        // would invent an agreement neither vocabulary states.
        let aot_target = compile_target(rows.facts)?;

        Ok(Self {
            profile,
            facts: rows.facts,
            emission: rows.emission,
            numerical: rows.numerical,
            aot_target,
        })
    }
}

/// Projects the declaration-owned BF16 measurement without widening the
/// ratified public F32 adapter.
///
/// Both dimensions are transactional, matching the public F32 seam: a conflict
/// in the result table cannot leave the input table partially installed.
fn declare_metal_bf16_subnormal_behaviour(
    builder: &mut TargetProfileBuilder,
    facts: &MetalTargetFacts,
    source: TargetCompileProfileMeasurementSource,
) -> Result<(), BoundMetalDeclarationError> {
    let behaviour = facts
        .subnormal_arithmetic
        .behaviour(MetalFloatArithmeticType::Bf16)
        .map_err(|_| BoundMetalDeclarationError::UnstatedBf16SubnormalBehaviour)?;
    let subject = ScalarArithmetic::new(ArithmeticType::Bf16, Bf16::resolved_type())
        .map_err(BoundMetalDeclarationError::Bf16Subject)?;
    let mut staged = builder.clone();
    staged
        .declare_measured_input_subnormal_behaviour(
            subject.clone(),
            behaviour.subnormal_mode(),
            source.clone(),
        )
        .map_err(BoundMetalDeclarationError::Bf16SubnormalProjection)?;
    staged
        .declare_measured_result_subnormal_behaviour(subject, behaviour.subnormal_mode(), source)
        .map_err(BoundMetalDeclarationError::Bf16SubnormalProjection)?;
    *builder = staged;
    Ok(())
}

/// The three normative references the normatively sourced rows are attributed to.
///
/// Three, and the grid axis is deliberately not among them: its authority is a
/// measurement, so it is declared through the measured source beside the
/// dispatchability and numerical rows rather than through an external guarantee.
/// The macOS SDK's `dispatchThreads:` contract remains why any extent is
/// *expressible* on this target, but a precondition is not a source for a value,
/// and carrying it here would attribute a measured number to a document that
/// states none.
struct NormativeSources {
    feature_tables: TargetFactSource,
    msl_address_space: TargetFactSource,
    msl_barrier: TargetFactSource,
}

impl NormativeSources {
    fn declare(_rows: &LedgerRows) -> Result<Self, BoundMetalDeclarationError> {
        let producer = || TargetFactProducerIdentity::new(NORMATIVE_PRODUCER.to_owned(), 1);
        let reference = |key: &str| TargetNormativeReferenceIdentity::new(key.to_owned(), 1);
        Ok(Self {
            feature_tables: TargetFactSource::external_guarantee(
                producer()?,
                reference(FEATURE_TABLES_REFERENCE)?,
            ),
            msl_address_space: TargetFactSource::external_guarantee(
                producer()?,
                reference(MSL_ADDRESS_SPACE_REFERENCE)?,
            ),
            msl_barrier: TargetFactSource::external_guarantee(
                producer()?,
                reference(MSL_BARRIER_REFERENCE)?,
            ),
        })
    }
}

/// Builds the one measurement source every measured row shares.
///
/// One context pairing the four offline toolchain components with the execution
/// environment, because that pair *is* the measurement: these compilers produced
/// bytes that this host ran to produce these observations. Splitting them into
/// two contexts would suggest either half could stand alone.
///
/// `metalfe-32023.921` is deliberately absent. The retained record holds it as
/// the build the host loads for `newLibraryWithSource:options:`; Tiler's AOT
/// route supplies no source, so that build is evidence about a comparison path
/// and ADR 0086 item 4 excludes it by name.
fn measured_source(
    rows: &LedgerRows,
) -> Result<TargetCompileProfileMeasurementSource, BoundMetalDeclarationError> {
    let offline = &rows.offline;
    let producer_defined = |key: &str| -> Result<TargetCompilerRole, BoundMetalDeclarationError> {
        Ok(TargetCompilerRole::ProducerDefined(
            TargetCompilerRoleIdentity::new(key.to_owned(), 1)?,
        ))
    };
    let builds = [
        TargetCompilerBuild::new(
            TargetCompilerRole::CodeGenerator,
            "apple.metal-offline-compiler".to_owned(),
            offline.compiler_version.to_owned(),
            Some(offline.compiler_build.to_owned()),
        )?,
        TargetCompilerBuild::new(
            TargetCompilerRole::Linker,
            "apple.air-lld".to_owned(),
            offline.linker_version.to_owned(),
            Some(offline.linker_build.to_owned()),
        )?,
        TargetCompilerBuild::new(
            producer_defined(OFFLINE_DISTRIBUTION_ROLE)?,
            "apple.xcode".to_owned(),
            offline.xcode_version.to_owned(),
            Some(offline.xcode_build.to_owned()),
        )?,
        TargetCompilerBuild::new(
            producer_defined(OFFLINE_SDK_ROLE)?,
            "apple.macos-sdk".to_owned(),
            offline.sdk_version.to_owned(),
            Some(offline.sdk_build.to_owned()),
        )?,
    ];
    let environment = TargetExecutionEnvironment::builder()
        .platform(rows.execution.platform.to_owned())
        .platform_version(rows.execution.platform_version.to_owned())
        .platform_build(rows.execution.platform_build.to_owned())
        .architecture(rows.execution.architecture.to_owned())
        .hardware(rows.execution.hardware.to_owned())
        .build()?;
    let context = TargetMeasurementContext::new(builds, environment)?;
    Ok(TargetCompileProfileMeasurementSource::new(
        TargetFactProducerIdentity::new(MEASURED_PRODUCER.to_owned(), 1)?,
        [context],
    )?)
}

/// Why a compilation was not assessed against the declared profile.
///
/// Two variants rather than one, matching `tiler_runtime`'s own classification:
/// a different key is a compilation for another target family and the repair is
/// a different profile, while the same key under a different descriptor is this
/// family under a revision the declaration does not carry and the repair is a
/// recompilation.
///
/// The descriptor variant reports byte lengths rather than the bytes: the
/// descriptors run to kilobytes, and a diagnostic that rendered both would bury
/// the one fact a reader needs, which is that they differ.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MetalPlanProfileMismatch {
    /// The compilation names a different target-profile family.
    ProfileKey {
        /// Key the bound declaration carries.
        declared: String,
        /// Key the compilation was assessed under.
        compiled: String,
    },
    /// The family agrees and the exact profile descriptor does not.
    ProfileDescriptor {
        /// The governed profile key both sides agree on.
        key: String,
        /// Encoded descriptor length the declaration carries.
        declared_bytes: usize,
        /// Encoded descriptor length the compilation was assessed under.
        compiled_bytes: usize,
    },
}

impl MetalPlanProfileMismatch {
    /// Returns the stable rule identifier for this mismatch.
    #[must_use]
    pub const fn rule(&self) -> &'static str {
        match self {
            Self::ProfileKey { .. } => "metal.plan.declared-profile-key-mismatch",
            Self::ProfileDescriptor { .. } => "metal.plan.declared-profile-descriptor-mismatch",
        }
    }
}

impl fmt::Display for MetalPlanProfileMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: ", self.rule())?;
        match self {
            Self::ProfileKey { declared, compiled } => {
                write!(
                    formatter,
                    "declared {declared:?}, compiled under {compiled:?}"
                )
            }
            Self::ProfileDescriptor {
                key,
                declared_bytes,
                compiled_bytes,
            } => write!(
                formatter,
                "{key:?} descriptors differ ({declared_bytes} declared, {compiled_bytes} \
                 compiled byte(s)); recompile the plan",
            ),
        }
    }
}

impl Error for MetalPlanProfileMismatch {}

/// Why the bound macOS Metal declaration could not be assembled.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BoundMetalDeclarationError {
    /// The declared profile key is not a valid target-profile key.
    ProfileKey(TargetProfileKeyError),
    /// A producer, normative-reference, compiler-role, compiler-build, or
    /// execution-environment identity was refused.
    Provenance(tiler_compiler::target::TargetFactSourceError),
    /// The prepared-entry workgroup query is not statable at its own phase.
    PreparedEntryQuery,
    /// The compiler target profile refused a declared row.
    Profile(TargetProfileBuildError),
    /// The `f32` subnormal projection was refused.
    SubnormalProjection(MetalF32TargetProfileError),
    /// The Metal record omitted the BF16 behaviour this declaration requires.
    UnstatedBf16SubnormalBehaviour,
    /// The governed scalar catalog refused the BF16 policy subject.
    ///
    /// Distinct from [`Self::Bf16SubnormalProjection`] because a different
    /// authority refuses: the catalog decides whether `bf16` arithmetic is
    /// defined over `tiler::bf16@1` at all, while the profile decides whether a
    /// row over that subject may be declared. Folding the two into one variant
    /// would make a rejection unable to name which one said no.
    Bf16Subject(ScalarArithmeticSubjectError),
    /// The compiler profile refused the declaration-owned BF16 projection.
    Bf16SubnormalProjection(TargetProfileBuildError),
    /// The compiler's offered buffer capacity exceeds the emission limit.
    ///
    /// A compiler admitting more bindings than the emitter can address would
    /// pass a signature to emission that emission must then reject, which turns
    /// a target fact into a late failure.
    BufferCapacityExceedsEmissionLimit {
        /// Capacity the compiler profile offers.
        compiler: u32,
        /// Capacity the Metal emission record states.
        emission: u32,
    },
    /// The Metal facts do not project onto a governed AOT compiler target.
    AotTarget(MetalTargetError),
}

impl From<TargetProfileKeyError> for BoundMetalDeclarationError {
    fn from(error: TargetProfileKeyError) -> Self {
        Self::ProfileKey(error)
    }
}

impl From<tiler_compiler::target::TargetFactSourceError> for BoundMetalDeclarationError {
    fn from(error: tiler_compiler::target::TargetFactSourceError) -> Self {
        Self::Provenance(error)
    }
}

impl From<TargetProfileBuildError> for BoundMetalDeclarationError {
    fn from(error: TargetProfileBuildError) -> Self {
        Self::Profile(error)
    }
}

impl From<MetalF32TargetProfileError> for BoundMetalDeclarationError {
    fn from(error: MetalF32TargetProfileError) -> Self {
        Self::SubnormalProjection(error)
    }
}

impl From<MetalTargetError> for BoundMetalDeclarationError {
    fn from(error: MetalTargetError) -> Self {
        Self::AotTarget(error)
    }
}

/// One line, and the refusing authority is named before its own text.
///
/// Every delegating arm shares one `write!`, so a new variant carrying a cause
/// adds a name rather than a sentence. The two arms below it are the only ones
/// with nothing to delegate to.
impl fmt::Display for BoundMetalDeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (what, cause): (_, &dyn fmt::Display) = match self {
            Self::ProfileKey(error) => ("profile key", error),
            Self::Provenance(error) => ("fact source", error),
            Self::Profile(error) => ("compiler profile row", error),
            Self::SubnormalProjection(error) => ("f32 subnormal projection", error),
            Self::Bf16Subject(error) => ("BF16 policy subject", error),
            Self::Bf16SubnormalProjection(error) => ("BF16 subnormal projection", error),
            Self::AotTarget(error) => ("AOT target", error),
            Self::UnstatedBf16SubnormalBehaviour => {
                return formatter.write_str(
                    "BF16 subnormal projection: Metal target facts do not state BF16 behaviour",
                );
            }
            Self::PreparedEntryQuery => {
                return formatter
                    .write_str("prepared-entry query: not statable at PreparedKernelPreflight");
            }
            Self::BufferCapacityExceedsEmissionLimit { compiler, emission } => {
                return write!(
                    formatter,
                    "buffer capacity: compiler offers {compiler}, emission admits {emission}",
                );
            }
        };
        write!(formatter, "{what}: {cause}")
    }
}

impl Error for BoundMetalDeclarationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProfileKey(error) => Some(error),
            Self::Provenance(error) => Some(error),
            Self::Profile(error) | Self::Bf16SubnormalProjection(error) => Some(error),
            Self::Bf16Subject(error) => Some(error),
            Self::SubnormalProjection(error) => Some(error),
            Self::AotTarget(error) => Some(error),
            Self::PreparedEntryQuery
            | Self::UnstatedBf16SubnormalBehaviour
            | Self::BufferCapacityExceedsEmissionLimit { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoundMetalCompileDeclaration, BoundMetalDeclarationError, FIRST_MACOS_APPLE9, LedgerRows,
        MEASURED_PRODUCER, MetalPlanProfileMismatch,
    };
    use tiler_compiler::session::{
        CompileRequest, NumericalContract, TargetCompileRefusal, TargetNumericalContractRefusal,
        TargetNumericalDeclaredMeans, TargetNumericalHonouredBehaviour,
        TargetNumericalRefusalDisposition, TargetNumericalRequirement, compile,
    };
    use tiler_compiler::target::{
        BackendArithmeticLicence, DTypeDispatchability, DTypeDispatchabilityResolution,
        EvaluationOrderResolution, IndexArithmeticSupport, ScalarArithmetic, ScalarSupport,
        SynchronizationSupport, TargetCompilerRoleReference, TargetCostRowResolution,
        TargetFactAuthority, TargetFactProducerIdentity, TargetFactSource, TargetFactValidityScope,
        TargetNormativeReferenceIdentity, TargetNumericalEvidenceBasis, TargetProfileBuildError,
        TargetProfileBuilder, TargetProfileKey, TargetRequest, WorkgroupTreeWidthPolicy,
        WorkgroupTreeWidthPolicyResolution,
    };
    use tiler_ir::program::abi::{
        AvailabilityPhase, TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
    };
    use tiler_ir::schedule::{
        ArithmeticType, FlushedZeroSign, MemoryOrdering, NumericalPermission, SubnormalMode,
        SynchronizationKind, SynchronizationScope,
    };
    use tiler_ir::semantic::{
        Bf16, Bf16Add, Bf16Constant, Bf16Multiply, F32, F32Add, F32Constant, F32Multiply, InputKey,
        OutputKey, ResolvedValueType, SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum,
        TypeKey,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_metal::target::{
        MetalDeploymentMinimum, MetalFloatArithmeticType, MetalFlushedZeroSign, MetalPlatform,
        MetalSubnormalArithmetic, MetalSubnormalArithmeticFacts, MetalTargetFacts,
        MslLanguageVersion,
    };

    /// The subnormal behaviour the ledger's measured Apple row delivers, for
    /// `f32` and `bf16` alike.
    const SIGN_PRESERVING_FLUSH: SubnormalMode = SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    };

    /// One named single-row mutation of the ledger transcription.
    ///
    /// A named pair rather than an inline tuple type, so the two mutation cases
    /// below read as a table of rows and the label travels with the mutation it
    /// describes -- a failure names which row did not reach the descriptor.
    type RowPerturbation = (&'static str, fn(&mut LedgerRows));

    fn declared() -> BoundMetalCompileDeclaration {
        BoundMetalCompileDeclaration::first_macos_apple9()
            .expect("the ledger's rows assemble one bound declaration")
    }

    fn descriptor(rows: &LedgerRows) -> Vec<u8> {
        BoundMetalCompileDeclaration::declare(rows)
            .expect("the perturbed rows still assemble")
            .profile()
            .canonical_descriptor()
            .to_vec()
    }

    fn program() -> SemanticProgram {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
        let input = builder
            .input::<F32>(
                InputKey::new("input").expect("the input key is valid"),
                Shape::from_dims([1, 3]),
            )
            .expect("the input binds");
        let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
        let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
        let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
        let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)])
            .expect("the sum applies");
        builder
            .output(
                OutputKey::new("result").expect("the output key is valid"),
                sum,
            )
            .expect("the output binds");
        builder.build().expect("the program verifies")
    }

    /// A pure-BF16 constant/multiply/add program.
    ///
    /// Deliberately not the `f32` program above in another width: no reduction,
    /// because the claim under test is which *numerical rows* answer for `bf16`,
    /// and a program is the smallest thing that can consume them. Every value in
    /// it is `bf16`, so nothing here can be answered by the profile's complete
    /// neighbouring `f32` table.
    fn bf16_program() -> SemanticProgram {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
        let input = builder
            .input::<Bf16>(
                InputKey::new("input").expect("the input key is valid"),
                Shape::from_dims([2, 2]),
            )
            .expect("the input binds");
        // 1.0 and 2.0 in bf16.
        let scale = Bf16Constant::apply(&mut builder, 0x3f80).expect("the scale applies");
        let bias = Bf16Constant::apply(&mut builder, 0x4000).expect("the bias applies");
        let product = Bf16Multiply::apply(&mut builder, input, scale).expect("the product applies");
        let mapped = Bf16Add::apply(&mut builder, product, bias).expect("the bias applies");
        builder
            .output(
                OutputKey::new("result").expect("the output key is valid"),
                mapped,
            )
            .expect("the output binds");
        builder.build().expect("the program verifies")
    }

    /// Returns the numerical refusal the *authoritative* profile answers
    /// `contract` with, or panics naming what came back instead.
    fn bf16_numerical_refusal(contract: NumericalContract) -> TargetNumericalContractRefusal {
        let batch = compile(CompileRequest::new(
            &bf16_program(),
            contract,
            TargetRequest::new([declared().profile().clone()]).expect("a singleton target request"),
        ))
        .expect("a target-local numerical refusal is a batch outcome, not a request error");
        let result = batch.targets().next().expect("one target outcome");
        let failure = result
            .outcome()
            .expect_err("the authoritative target refused")
            .refusal()
            .expect("a pre-trace contract refusal retains typed detail");
        match failure {
            TargetCompileRefusal::NumericalContract(refusal) => refusal.clone(),
            other => panic!("expected a numerical-contract refusal, got {other:?}"),
        }
    }

    /// The declaration carries the ledger's rows, in both vocabularies.
    #[test]
    fn the_declaration_states_exactly_the_ledger_rows() {
        let declaration = declared();
        assert_eq!(
            declaration.profile().profile_key().as_str(),
            "tiler.metal.macos-apple9.msl4-0.f32-bf16.v1",
        );
        let facts = declaration.metal_facts();
        assert_eq!(facts.language, MslLanguageVersion::Metal4_0);
        assert_eq!(facts.platform, MetalPlatform::MacOs);
        assert_eq!(facts.deployment_minimum, MetalDeploymentMinimum::new(26, 0));
        assert_eq!(facts.buffer_binding_limit, 31);
        assert_eq!(
            facts
                .subnormal_arithmetic
                .behaviour(MetalFloatArithmeticType::F32),
            Ok(MetalSubnormalArithmetic::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::PreservesSign,
            }),
        );
        assert_eq!(
            facts
                .subnormal_arithmetic
                .behaviour(MetalFloatArithmeticType::Bf16),
            Ok(MetalSubnormalArithmetic::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::PreservesSign,
            }),
        );
        assert_eq!(
            declaration.aot_target().triple(),
            "air64-apple-macos26.0",
            "the AOT projection is the total map of the same facts",
        );
        assert_eq!(declaration.aot_target().std_token(), "metal4.0");
    }

    /// The deployment record moved to MSL 4.0 / macOS 26.0, and must stay there.
    ///
    /// The retained measurement compiled `-std=metal4.0` for
    /// `air64-apple-macos26.0`. Reusing the older MSL 3.1 / macOS 14.0 record
    /// would attribute these measurements to a compilation that did not produce
    /// them, so the two values the prototypes previously stated are asserted
    /// *absent* rather than merely unchecked.
    #[test]
    fn the_declaration_does_not_carry_the_superseded_msl_3_1_record() {
        let facts = *declared().metal_facts();
        assert_ne!(facts.language, MslLanguageVersion::Metal3_1);
        assert_ne!(facts.deployment_minimum, MetalDeploymentMinimum::new(14, 0));
    }

    /// Every omitted fact resolves as `Unknown`, never as a default.
    ///
    /// Three of them, and each for a different reason the ledger records: the
    /// device-address-width row has no consumer, the workgroup row lives on a
    /// prepared pipeline, and `f16` was not measured under MSL 4.0.
    #[test]
    fn omitted_rows_resolve_unknown_rather_than_defaulted() {
        let declaration = declared();
        let f16 = ResolvedValueType::nominal(
            TypeKey::new("tiler", "f16", 1).expect("a well-formed neighbouring type key"),
        );
        assert_eq!(
            declaration
                .profile()
                .dtype_dispatchability(&f16, AvailabilityPhase::LaunchPreflight),
            DTypeDispatchabilityResolution::Unknown,
            "an unmeasured dtype must not inherit the f32 row at any phase",
        );
        assert_eq!(
            declaration
                .profile()
                .dtype_dispatchability(&F32::resolved_type(), AvailabilityPhase::CompileProfile,),
            DTypeDispatchabilityResolution::Dispatchable,
        );
        assert_eq!(
            declaration
                .profile()
                .dtype_dispatchability(&Bf16::resolved_type(), AvailabilityPhase::CompileProfile,),
            DTypeDispatchabilityResolution::Dispatchable,
        );
        // The descriptor spells each quantitative axis by its governed key, so
        // an absent axis is absent from the bytes and a deferred one is present
        // only in the query table. Read from the encoding rather than asserted
        // about the builder, because the encoding is what artifact and cache
        // identity are taken over.
        let text = String::from_utf8_lossy(declaration.profile().canonical_descriptor());
        assert!(
            !text.contains("device-address-bits"),
            "the absent device-address-width row must not appear in the descriptor",
        );
        assert!(
            text.contains("threads-per-workgroup"),
            "the workgroup axis is present as a deferred query, not as a fact",
        );
        assert!(text.contains("grid-axis"));
        assert!(text.contains("buffer-bindings"));
        assert!(text.contains("index-arithmetic-u64"));
        assert!(text.contains("local-memory-bytes"));
        assert!(text.contains("device-memory"));
    }

    /// Deleting or substituting the BF16 dispatch row changes both the answer
    /// and identity without affecting F32.
    #[test]
    fn bf16_dispatchability_is_an_exact_independent_row() {
        let baseline = declared();
        let mut absent = FIRST_MACOS_APPLE9;
        absent.bf16_dispatchability = None;
        let absent = BoundMetalCompileDeclaration::declare(&absent)
            .expect("an omitted BF16 row remains a valid conservative profile");
        assert_eq!(
            absent
                .profile()
                .dtype_dispatchability(&Bf16::resolved_type(), AvailabilityPhase::CompileProfile,),
            DTypeDispatchabilityResolution::Unknown,
        );
        assert_eq!(
            absent
                .profile()
                .dtype_dispatchability(&F32::resolved_type(), AvailabilityPhase::CompileProfile,),
            DTypeDispatchabilityResolution::Dispatchable,
        );
        assert_ne!(
            absent.profile().canonical_descriptor(),
            baseline.profile().canonical_descriptor(),
        );

        let mut unsupported = FIRST_MACOS_APPLE9;
        unsupported.bf16_dispatchability = Some(DTypeDispatchability::Unsupported);
        let unsupported = BoundMetalCompileDeclaration::declare(&unsupported)
            .expect("the substituted refusal remains a coherent profile");
        assert_eq!(
            unsupported
                .profile()
                .dtype_dispatchability(&Bf16::resolved_type(), AvailabilityPhase::CompileProfile,),
            DTypeDispatchabilityResolution::Unsupported,
        );
        assert_ne!(
            unsupported.profile().canonical_descriptor(),
            baseline.profile().canonical_descriptor(),
        );
    }

    /// The published rows are the profile's own declarations and nothing else.
    ///
    /// Both halves matter and neither implies the other: the two measured dtypes
    /// appear with the verdict the ledger states, and the two the ledger does not
    /// declare are **absent** rather than present with a default. An accessor
    /// that returned a row per [`ArithmeticType`] would hand a consumer a verdict
    /// for `f16`, which nothing on this profile measured.
    #[test]
    fn the_published_dispatchability_rows_are_the_declared_ones_only() {
        assert_eq!(
            declared().dtype_dispatchability_rows(),
            vec![
                (ArithmeticType::Bf16, DTypeDispatchability::Dispatchable),
                (ArithmeticType::F32, DTypeDispatchability::Dispatchable),
            ],
            "the rows must be exactly the ledger's two measured dtypes, in canonical order",
        );
    }

    /// Every ledger dispatchability perturbation reaches the published rows.
    ///
    /// The accessor's whole purpose is that a consumer reading it cannot state a
    /// verdict this declaration stopped holding, so each way a row can move is
    /// driven separately: retracting it must remove the row, and refuting it must
    /// change the verdict in place. A single case would pass for an accessor that
    /// only noticed one of the two.
    #[test]
    fn a_moved_ledger_dispatchability_row_moves_the_published_rows() {
        let rows = |ledger: &LedgerRows| {
            BoundMetalCompileDeclaration::declare(ledger)
                .expect("the perturbed rows still assemble")
                .dtype_dispatchability_rows()
        };
        let mut retracted = FIRST_MACOS_APPLE9;
        retracted.bf16_dispatchability = None;
        assert_eq!(
            rows(&retracted),
            vec![(ArithmeticType::F32, DTypeDispatchability::Dispatchable)],
            "a retracted measurement must leave silence, not a stale verdict",
        );

        let mut refuted = FIRST_MACOS_APPLE9;
        refuted.bf16_dispatchability = Some(DTypeDispatchability::Unsupported);
        assert_eq!(
            rows(&refuted),
            vec![
                (ArithmeticType::Bf16, DTypeDispatchability::Unsupported),
                (ArithmeticType::F32, DTypeDispatchability::Dispatchable),
            ],
            "a refuted measurement is a stated negative, distinct from silence",
        );
    }

    /// The BF16 table encodes its exact subject and the unified measurement
    /// source; neither can be supplied by the neighbouring F32 declaration.
    #[test]
    fn bf16_subnormal_rows_carry_the_exact_subject_and_source() {
        let declaration = declared();
        let descriptor = declaration.profile().canonical_descriptor();
        let subject = Bf16::resolved_type().canonical_encoding();
        let subject = subject.as_bytes();
        assert!(
            descriptor
                .windows(subject.len())
                .any(|window| window == subject),
            "the exact BF16 subject is absent",
        );
        let text = String::from_utf8_lossy(descriptor);
        assert!(
            text.contains(super::MEASURED_PRODUCER),
            "the measured producer is absent",
        );
        assert!(text.contains("32023.883"), "the compiler row is absent");
        assert!(text.contains("26A5388g"), "the execution row is absent");
    }

    /// The workgroup row is a prepared-kernel query and cannot become a fact.
    ///
    /// Perturbation: declaring the same axis as an available quantitative fact
    /// beside the query is refused. Without this, "the row is deferred" would be
    /// a comment rather than a checked property.
    #[test]
    fn a_prepared_kernel_row_cannot_be_declared_as_a_compile_profile_fact() {
        let source = TargetFactSource::external_guarantee(
            TargetFactProducerIdentity::new("test.phase-probe.v1".to_owned(), 1).unwrap(),
            TargetNormativeReferenceIdentity::new("test.phase-spec.v1".to_owned(), 1).unwrap(),
        );
        let mut builder =
            TargetProfileBuilder::new(TargetProfileKey::new("test.phase.v1".to_owned()).unwrap());
        builder
            .declare_max_threads_per_workgroup_query(workgroup_query(
                AvailabilityPhase::PreparedKernelPreflight,
            ))
            .expect("the prepared-kernel query is statable at its own phase");
        assert_eq!(
            builder.declare_max_threads_per_workgroup(1_024, source),
            Err(
                TargetProfileBuildError::ConflictingQuantitativeFactAndQuery {
                    axis: "threads-per-workgroup",
                }
            ),
            "a compiled pipeline's capacity is not a compile-profile fact",
        );
    }

    /// Live-device evidence cannot be stated as the workgroup compile query.
    ///
    /// Perturbation: the same query at `LiveDevicePreflight` is refused by
    /// phase. A device maximum and a prepared function's maximum are different
    /// numbers on this very hardware — 1,024 against 1 for a kernel constrained
    /// at `(1, 1, 1)` — so accepting the earlier phase would answer the axis
    /// with a number that is not its answer.
    #[test]
    fn a_live_device_phase_cannot_answer_the_prepared_entry_query() {
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.phase-live.v1".to_owned()).unwrap(),
        );
        assert_eq!(
            builder.declare_max_threads_per_workgroup_query(workgroup_query(
                AvailabilityPhase::LiveDevicePreflight,
            )),
            Err(TargetProfileBuildError::InvalidQuantitativeQueryPhase {
                axis: "threads-per-workgroup",
                required: AvailabilityPhase::PreparedKernelPreflight,
                actual: AvailabilityPhase::LiveDevicePreflight,
            }),
        );
    }

    fn workgroup_query(phase: AvailabilityPhase) -> TargetPropertyQuery {
        TargetPropertyQuery::new(
            TargetPropertyKey::new(FIRST_MACOS_APPLE9.workgroup_property_key)
                .expect("the property key is valid"),
            phase,
            TargetPropertyProviderIdentity::new("tiler", "prepared-entry-properties", 1)
                .expect("the provider identity is valid"),
        )
        .expect("the query is constructible at every phase this test states")
    }

    /// A second `f32` subnormal row is refused before anything is emitted.
    ///
    /// Perturbation of the projection overlap: the ledger requires the Metal
    /// record's `f32` entry to reach the compiler's two subnormal dimensions
    /// exactly once, and declaring it twice would put two rows at one phase.
    #[test]
    fn a_duplicated_subnormal_projection_is_refused() {
        let declaration = declared();
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.duplicate-subnormal.v1".to_owned()).unwrap(),
        );
        let source = super::measured_source(&FIRST_MACOS_APPLE9).expect("the measured source");
        crate::declare_metal_f32_subnormal_behaviour(
            &mut builder,
            declaration.metal_facts(),
            source.clone(),
        )
        .expect("the first projection lands");
        let error = crate::declare_metal_f32_subnormal_behaviour(
            &mut builder,
            declaration.metal_facts(),
            source,
        )
        .expect_err("the second projection is refused");
        assert!(
            error
                .to_string()
                .contains("compiler target profile refused Metal f32 facts"),
            "unexpected refusal: {error}",
        );
    }

    /// The BF16 projection owns complete exclusive input and result tables.
    ///
    /// Reapplying the exact measured row must conflict, rather than silently
    /// adding another answer or replacing the first source.
    #[test]
    fn a_duplicated_bf16_projection_is_refused() {
        let declaration = declared();
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.duplicate-bf16-subnormal.v1".to_owned()).unwrap(),
        );
        let source = super::measured_source(&FIRST_MACOS_APPLE9).expect("the measured source");
        super::declare_metal_bf16_subnormal_behaviour(
            &mut builder,
            declaration.metal_facts(),
            source.clone(),
        )
        .expect("the first complete BF16 projection lands");
        let subject = ScalarArithmetic::new(ArithmeticType::Bf16, Bf16::resolved_type())
            .expect("the governed BF16 arithmetic subject");
        let mut input_probe = builder.clone();
        assert!(matches!(
            input_probe.declare_measured_input_subnormal_behaviour(
                subject.clone(),
                SubnormalMode::Preserve,
                source.clone(),
            ),
            Err(TargetProfileBuildError::ConflictingSubnormalDeclaration { .. })
        ));
        let mut result_probe = builder;
        assert!(matches!(
            result_probe.declare_measured_result_subnormal_behaviour(
                subject,
                SubnormalMode::Preserve,
                source,
            ),
            Err(TargetProfileBuildError::ConflictingSubnormalDeclaration { .. })
        ));
    }

    /// A missing BF16 Metal row refuses the declaration before profile build.
    #[test]
    fn an_unstated_bf16_subnormal_row_is_refused() {
        let mut rows = FIRST_MACOS_APPLE9;
        rows.facts = MetalTargetFacts::new(
            rows.facts.language,
            rows.facts.platform,
            rows.facts.deployment_minimum,
            MetalSubnormalArithmeticFacts::unmeasured().stating(
                MetalFloatArithmeticType::F32,
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
            ),
            rows.facts.buffer_binding_limit,
        );
        assert_eq!(
            BoundMetalCompileDeclaration::declare(&rows).unwrap_err(),
            BoundMetalDeclarationError::UnstatedBf16SubnormalBehaviour,
        );
    }

    /// A contradictory pre-existing subnormal row is refused before emission.
    #[test]
    fn a_contradictory_subnormal_row_is_refused_before_emission() {
        let declaration = declared();
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.contradictory-subnormal.v1".to_owned()).unwrap(),
        );
        builder
            .declare_input_subnormals(
                ScalarArithmetic::f32(),
                SubnormalMode::Preserve,
                ScalarSupport::Exact,
                TargetFactSource::external_guarantee(
                    TargetFactProducerIdentity::new("test.contradiction.v1".to_owned(), 1).unwrap(),
                    TargetNormativeReferenceIdentity::new(
                        "test.contradiction-spec.v1".to_owned(),
                        1,
                    )
                    .unwrap(),
                ),
            )
            .expect("a preserving row exists first");
        crate::declare_metal_f32_subnormal_behaviour(
            &mut builder,
            declaration.metal_facts(),
            super::measured_source(&FIRST_MACOS_APPLE9).expect("the measured source"),
        )
        .expect_err("a flushing measurement cannot join a preserving declaration");
    }

    /// A compiler capacity above the emission limit is refused.
    ///
    /// Perturbation of the buffer overlap in the only unsound direction. The
    /// reverse — a conservative compiler capacity *below* the emitter's — is
    /// accepted, and asserting both is what shows the check is directional
    /// rather than an equality test wearing a comparison's clothes.
    #[test]
    fn a_compiler_capacity_above_the_emission_limit_is_refused() {
        let mut rows = FIRST_MACOS_APPLE9;
        rows.buffer_bindings = 32;
        assert_eq!(
            BoundMetalCompileDeclaration::declare(&rows).unwrap_err(),
            BoundMetalDeclarationError::BufferCapacityExceedsEmissionLimit {
                compiler: 32,
                emission: 31,
            },
        );
        let mut conservative = FIRST_MACOS_APPLE9;
        conservative.buffer_bindings = 8;
        BoundMetalCompileDeclaration::declare(&conservative)
            .expect("a compiler capacity below the emission limit is sound");
    }

    /// The Metal facts that do not project are not validated as though they did.
    ///
    /// Changing the language standard changes the AOT target and the payload
    /// identity it governs, and must leave the compiler profile's descriptor
    /// untouched — it has no compiler counterpart to move.
    #[test]
    fn nonprojected_metal_facts_do_not_reach_the_compiler_descriptor() {
        let mut rows = FIRST_MACOS_APPLE9;
        rows.facts = MetalTargetFacts::new(
            MslLanguageVersion::Metal3_2,
            MetalPlatform::MacOs,
            MetalDeploymentMinimum::new(26, 0),
            rows.facts.subnormal_arithmetic,
            rows.facts.buffer_binding_limit,
        );
        assert_eq!(
            descriptor(&rows),
            descriptor(&FIRST_MACOS_APPLE9),
            "a language standard has no compiler-profile counterpart",
        );
        assert_ne!(
            BoundMetalCompileDeclaration::declare(&rows)
                .expect("MSL 3.2 is a governed macOS target")
                .aot_target(),
            declared().aot_target(),
            "and it must still move the AOT target it does govern",
        );
    }

    /// Every projected row moves the descriptor, and therefore artifact identity.
    ///
    /// One perturbation per row rather than one for the set: a descriptor that
    /// moved for four of six rows would pass any check that only asked whether
    /// *some* change moved it.
    #[test]
    fn every_projected_row_moves_the_profile_descriptor() {
        let baseline = descriptor(&FIRST_MACOS_APPLE9);
        let perturbations: [RowPerturbation; 8] = [
            // The superseded value, so the perturbation is also the check that
            // the row's own movement is what the descriptor recorded.
            ("grid-axis threads", |rows| rows.grid_axis_threads = 4),
            ("buffer bindings", |rows| rows.buffer_bindings = 16),
            ("local memory bytes", |rows| {
                rows.local_memory_bytes = 16_384;
            }),
            ("index arithmetic", |rows| {
                rows.index_arithmetic = IndexArithmeticSupport::Unsupported;
            }),
            ("device address space", |rows| {
                rows.device_address_space = false;
            }),
            ("the BF16 dispatchability row", |rows| {
                rows.bf16_dispatchability = None;
            }),
            ("the projected f32 subnormal behaviour", |rows| {
                rows.facts = MetalTargetFacts::new(
                    rows.facts.language,
                    rows.facts.platform,
                    rows.facts.deployment_minimum,
                    MetalSubnormalArithmeticFacts::unmeasured()
                        .stating(
                            MetalFloatArithmeticType::F32,
                            MetalSubnormalArithmetic::PreservesSubnormals,
                        )
                        .stating(
                            MetalFloatArithmeticType::Bf16,
                            MetalSubnormalArithmetic::FlushesToZero {
                                zero_sign: MetalFlushedZeroSign::PreservesSign,
                            },
                        ),
                    rows.facts.buffer_binding_limit,
                );
            }),
            ("the projected BF16 subnormal behaviour", |rows| {
                rows.facts = MetalTargetFacts::new(
                    rows.facts.language,
                    rows.facts.platform,
                    rows.facts.deployment_minimum,
                    MetalSubnormalArithmeticFacts::unmeasured()
                        .stating(
                            MetalFloatArithmeticType::F32,
                            MetalSubnormalArithmetic::FlushesToZero {
                                zero_sign: MetalFlushedZeroSign::PreservesSign,
                            },
                        )
                        .stating(
                            MetalFloatArithmeticType::Bf16,
                            MetalSubnormalArithmetic::PreservesSubnormals,
                        ),
                    rows.facts.buffer_binding_limit,
                );
            }),
        ];
        for (name, perturb) in perturbations {
            let mut rows = FIRST_MACOS_APPLE9;
            perturb(&mut rows);
            assert_ne!(
                descriptor(&rows),
                baseline,
                "{name} does not reach the profile descriptor",
            );
        }
    }

    /// Every measurement-context field moves the descriptor.
    ///
    /// The offline compiler build, the linker build, the Xcode distribution, the
    /// SDK, and each execution-environment field are separately identity-bearing,
    /// because each of them can move while the others stay identical and each
    /// bounds what the measured rows are valid for.
    #[test]
    fn every_measurement_context_field_moves_the_profile_descriptor() {
        let baseline = descriptor(&FIRST_MACOS_APPLE9);
        let perturbations: [RowPerturbation; 9] = [
            ("the offline compiler version", |rows| {
                rows.offline.compiler_version = "32023.884";
            }),
            ("the offline compiler build", |rows| {
                rows.offline.compiler_build = "metalfe-32023.884";
            }),
            ("the offline linker build", |rows| {
                rows.offline.linker_build = "AIR-LLD 32023.884 (metalfe-32023.884)";
            }),
            ("the Xcode build", |rows| {
                rows.offline.xcode_build = "17F114";
            }),
            ("the macOS SDK build", |rows| {
                rows.offline.sdk_build = "25F71";
            }),
            ("the execution OS version", |rows| {
                rows.execution.platform_version = "27.1";
            }),
            ("the execution OS build", |rows| {
                rows.execution.platform_build = "26A5389x";
            }),
            ("the execution architecture", |rows| {
                rows.execution.architecture = "x86_64";
            }),
            ("the execution hardware", |rows| {
                rows.execution.hardware = "Apple M3 Max";
            }),
        ];
        for (name, perturb) in perturbations {
            let mut rows = FIRST_MACOS_APPLE9;
            perturb(&mut rows);
            assert_ne!(
                descriptor(&rows),
                baseline,
                "{name} does not reach the profile descriptor",
            );
        }
    }

    /// Every dimension of the synchronization row separately moves the descriptor.
    ///
    /// The subject is matched as one atomic value, so a profile that agreed with
    /// a caller on four dimensions and differed on the fifth must be a *different
    /// profile* rather than a near miss. Driving all five plus the verdict is
    /// what makes that atomicity a checked property instead of a comment: if any
    /// one of them failed to reach the encoding, two targets with genuinely
    /// different barrier contracts would share an artifact identity, and a cache
    /// entry built for one would be served for the other.
    #[test]
    fn every_synchronization_dimension_moves_the_profile_descriptor() {
        let baseline = descriptor(&FIRST_MACOS_APPLE9);
        let perturbations: [RowPerturbation; 6] = [
            ("the synchronization kind", |rows| {
                rows.synchronization.kind = SynchronizationKind::SplitPhaseBarrier;
            }),
            ("the arrival scope", |rows| {
                rows.synchronization.execution_scope = SynchronizationScope::Subgroup;
            }),
            ("the publication scope", |rows| {
                rows.synchronization.visibility_scope = SynchronizationScope::Device;
            }),
            ("the fenced workgroup domain", |rows| {
                rows.synchronization.fenced_spaces.workgroup = false;
                // The subject must still fence something, or construction refuses
                // it as vacuous before the descriptor is reached and this case
                // would prove nothing.
                rows.synchronization.fenced_spaces.device = true;
            }),
            ("the fenced device domain", |rows| {
                rows.synchronization.fenced_spaces.device = true;
            }),
            ("the established ordering", |rows| {
                rows.synchronization.ordering = MemoryOrdering::SequentiallyConsistent;
            }),
        ];
        for (name, perturb) in perturbations {
            let mut rows = FIRST_MACOS_APPLE9;
            perturb(&mut rows);
            assert_ne!(
                descriptor(&rows),
                baseline,
                "{name} does not reach the profile descriptor",
            );
        }
    }

    /// A refused realization is a different profile from a realized one.
    ///
    /// Separate from the dimension sweep above because it perturbs the *verdict*
    /// rather than the subject. A target that names this exact barrier and
    /// declares it unrealizable is making a different claim from one that
    /// realizes it, and the two must not share a descriptor — otherwise a plan
    /// admitted against the realizing profile would validate against the
    /// refusing one.
    #[test]
    fn a_refused_synchronization_verdict_moves_the_profile_descriptor() {
        let mut rows = FIRST_MACOS_APPLE9;
        rows.synchronization_support = SynchronizationSupport::Unrealizable;
        assert_ne!(descriptor(&rows), descriptor(&FIRST_MACOS_APPLE9));
    }

    /// The declared profile carries exactly one synchronization row.
    ///
    /// Read from the canonical encoding rather than from the builder, because
    /// the encoding is what artifact and cache identity are taken over. The
    /// governed domain separator is what a reader greps for; a profile that
    /// declared none would omit the count this asserts.
    #[test]
    fn the_declared_profile_states_one_barrier_realization() {
        let declaration = declared();
        let descriptor = declaration.profile().canonical_descriptor();
        let text = String::from_utf8_lossy(descriptor);
        assert!(
            text.contains("tiler.target-profile.synchronization-realization.v1"),
            "the synchronization row family is absent from the descriptor",
        );
        // The reference the row is attributed to travels in the source table, so
        // a row sourced from the wrong document is visible here.
        assert!(
            text.contains("apple.metal-shading-language.4-0.threadgroup-barrier"),
            "the barrier row is not attributed to the MSL synchronization section",
        );
        // Pinned because the authority ledger quotes this number, and a
        // document citing a byte count nothing checks drifts silently. It moved
        // from 1,963 when the independent BF16 dispatchability and input/result
        // subnormal tables were added (and from 1,741 before the barrier and
        // permitted-reassociation rows).
        //
        // It **shrank** to 1,999 when the grid-axis row became measured, and the
        // direction is the point: the bound itself is a fixed-width `u64` in the
        // encoding, so its value moves no bytes. What moved is the source table.
        // The macOS SDK dispatch reference was the grid row's only user, so
        // retiring it removed one whole `external_guarantee` record — producer
        // identity and normative-reference identity — while the measured source
        // the row joined was already present for the dispatchability and
        // numerical rows and cost nothing to share.
        //
        // It grew to **2,099** when the measured saturated-parallel-fold-step
        // cost row landed, and the delta is encoding-predicted to the byte rather
        // than observed: the new section writes its length-prefixed 33-byte domain
        // separator (41), a row count (8), the length-prefixed 34-byte row key
        // (42), a fixed-width `u64` value (8), and a one-byte compact source
        // index — 100 bytes exactly. The source table does not grow, because the
        // row shares the measured source the grid-axis, dispatchability, and
        // numerical rows already carry.
        //
        // It grew to **2,169** when the closed workgroup-tree-width policy
        // landed: the new section writes its length-prefixed 52-byte domain
        // separator (60), a row count (8), a one-byte policy tag, and a one-byte
        // compact source index — 70 bytes exactly. The source table does not
        // grow, because the policy shares the same measured source. **That the
        // arithmetic closes is the evidence no layout moved**, not an assertion
        // beside one.
        //
        // It grew to **2,181** when the two measured elementary rows landed:
        // each scalar honourability row writes the subject's one-byte
        // arithmetic tag, its framed resolved-type identity, the dimension
        // tag, the two behaviour bytes, the means tag, and a one-byte compact
        // source index — six bytes here because the `f32` subject and the
        // shared measured source are already in their tables — so two rows are
        // twelve bytes exactly. The source table does not grow, because both
        // rows share the measured source every numerical row already carries.
        assert_eq!(
            descriptor.len(),
            2_181,
            "the canonical descriptor length moved; update the authority ledger with it",
        );
    }

    /// The declared profile states the measured cost row, and states it as a
    /// cost row rather than as a capability.
    ///
    /// **This is what keeps `tiler-compiler`'s own tests honest.** That crate
    /// cannot depend on this one, so `pipeline::tests`' fixtures restate the
    /// fitted 1,056 as a constant; this asserts the declaration carries the same
    /// number, so the two cannot drift apart silently.
    ///
    /// The descriptor assertion beside it is the one that separates the two
    /// kinds: a capability axis encodes into the quantitative section under its
    /// axis key, and this row encodes into a section of its own behind its own
    /// domain separator. A reader greps for the separator, and a row that had
    /// been declared as an axis would not carry one.
    #[test]
    fn the_declared_profile_states_the_measured_cost_row() {
        let declaration = declared();
        let profile = declaration.profile();
        assert_eq!(
            profile.saturated_parallel_fold_steps(AvailabilityPhase::CompileProfile),
            TargetCostRowResolution::Declared { value: 1_056 },
            "the declared saturated-fold-step row moved off the retained fit",
        );
        let text = String::from_utf8_lossy(profile.canonical_descriptor());
        assert!(
            text.contains("tiler.target-profile.cost-row.v1"),
            "the cost-row family is absent from the descriptor",
        );
        assert!(
            text.contains("cost.saturated-parallel-fold-steps"),
            "the descriptor does not name the declared row",
        );

        // Removing the row leaves a profile that resolves it `Unknown` — no
        // preference, never a refusal — and whose descriptor is exactly the
        // hundred bytes shorter that the conditional section accounts for. This is
        // the silence rule at the identity level: a target that says nothing about
        // cost encodes as though the family did not exist.
        let mut silent = FIRST_MACOS_APPLE9;
        silent.saturated_parallel_fold_steps = None;
        let silent = BoundMetalCompileDeclaration::declare(&silent)
            .expect("removing a cost row cannot make the declaration invalid");
        assert_eq!(
            silent
                .profile()
                .saturated_parallel_fold_steps(AvailabilityPhase::CompileProfile),
            TargetCostRowResolution::Unknown,
        );
        assert_eq!(
            silent.profile().canonical_descriptor().len(),
            profile.canonical_descriptor().len() - 100,
            "the cost-row section is not the hundred bytes its encoding predicts",
        );
    }

    /// The declared profile states the closed tree-width policy, and omitting
    /// it withdraws the family rather than inheriting `256`.
    #[test]
    fn the_declared_profile_states_the_qualified_tree_width_policy() {
        let declaration = declared();
        let profile = declaration.profile();
        assert_eq!(
            profile.workgroup_tree_width_policy(AvailabilityPhase::CompileProfile),
            WorkgroupTreeWidthPolicyResolution::Declared(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1
            ),
            "the declared tree-width policy moved off the retained qualification",
        );
        let text = String::from_utf8_lossy(profile.canonical_descriptor());
        assert!(
            text.contains("tiler.target-profile.workgroup-tree-width-policy.v1"),
            "the tree-width-policy family is absent from the descriptor",
        );

        let mut silent = FIRST_MACOS_APPLE9;
        silent.workgroup_tree_width_policy = None;
        let silent = BoundMetalCompileDeclaration::declare(&silent)
            .expect("removing the tree-width policy cannot make the declaration invalid");
        assert_eq!(
            silent
                .profile()
                .workgroup_tree_width_policy(AvailabilityPhase::CompileProfile),
            WorkgroupTreeWidthPolicyResolution::Unknown,
        );
        assert_eq!(
            silent.profile().canonical_descriptor().len(),
            profile.canonical_descriptor().len() - 70,
            "the tree-width-policy section is not the seventy bytes its encoding predicts",
        );
        assert!(
            !String::from_utf8_lossy(silent.profile().canonical_descriptor())
                .contains("tiler.target-profile.workgroup-tree-width-policy.v1"),
            "omitting the policy must write none of the family's bytes",
        );
    }

    /// The declared profile says nothing about evaluation-order preservation,
    /// and saying nothing is the honest answer rather than an oversight.
    ///
    /// Finding 34 measured the property on a **neighbouring toolchain row** —
    /// offline `metalfe-32023.921` under Xcode 27.0 — where every row this
    /// declaration carries was taken under `metalfe-32023.883` and Xcode 26.6.
    /// The property is a property of that compiler build, so the measurement
    /// cannot be attributed to a profile whose plans a different build compiles.
    /// The row therefore resolves `Unknown`, at every phase, for both licences
    /// and for both dtypes this profile does measure — and `Unknown` is what the
    /// oracle's refusal class 3 acts on.
    ///
    /// The negative is asserted at `LaunchPreflight`, the latest phase there is,
    /// so a row declared at *any* phase would break this test rather than only a
    /// compile-profile one.
    #[test]
    fn the_declared_profile_answers_unknown_on_evaluation_order_preservation() {
        let declaration = declared();
        let bf16 = ScalarArithmetic::new(ArithmeticType::Bf16, Bf16::resolved_type())
            .expect("the bf16 policy subject is validated");
        for subject in [ScalarArithmetic::f32(), bf16] {
            for licence in [
                BackendArithmeticLicence::Withheld,
                BackendArithmeticLicence::Granted,
            ] {
                assert_eq!(
                    declaration.profile().evaluation_order_preservation(
                        &subject,
                        licence,
                        AvailabilityPhase::LaunchPreflight,
                    ),
                    EvaluationOrderResolution::Unknown,
                    "this profile declares no {} evaluation-order row; finding 34 is on \
                     another toolchain row",
                    licence.key(),
                );
            }
        }
        assert!(
            !String::from_utf8_lossy(declaration.profile().canonical_descriptor())
                .contains("tiler.target-profile.evaluation-order-preservation.v1"),
            "an undeclared row family writes no bytes, which is why this descriptor \
             length did not move",
        );
    }

    /// A profile-key change moves both halves of the artifact reference.
    #[test]
    fn the_artifact_reference_carries_the_exact_key_and_descriptor() {
        let declaration = declared();
        let reference = declaration
            .target_profile_ref()
            .expect("the artifact reference composes");
        assert_eq!(
            reference.key.as_str(),
            declaration.profile().profile_key().as_str(),
        );
        let mut rows = FIRST_MACOS_APPLE9;
        rows.profile_key = "tiler.metal.macos-apple9.msl4-0.f32-bf16.v2";
        let other = BoundMetalCompileDeclaration::declare(&rows)
            .expect("a rekeyed declaration assembles")
            .target_profile_ref()
            .expect("its artifact reference composes");
        assert_ne!(other.key.as_str(), reference.key.as_str());
        assert_ne!(other.descriptor, reference.descriptor);
    }

    /// The mismatch check distinguishes a wrong family from a stale revision.
    #[test]
    fn a_profile_mismatch_names_the_half_that_disagreed() {
        let declaration = declared();
        let key = declaration.profile().profile_key().as_str().to_owned();
        let descriptor = declaration.profile().canonical_descriptor();
        declaration
            .require_compiled_under(&key, descriptor)
            .expect("the declaration accepts its own profile");
        assert_eq!(
            declaration
                .require_compiled_under("tiler.other-family.v1", descriptor)
                .unwrap_err(),
            MetalPlanProfileMismatch::ProfileKey {
                declared: key.clone(),
                compiled: "tiler.other-family.v1".to_owned(),
            },
        );
        let mut stale = descriptor.to_vec();
        stale.push(0x00);
        let error = declaration
            .require_compiled_under(&key, &stale)
            .unwrap_err();
        assert!(
            matches!(error, MetalPlanProfileMismatch::ProfileDescriptor { .. }),
            "unexpected mismatch: {error:?}",
        );
        assert!(
            error.to_string().contains("recompile the plan"),
            "the descriptor mismatch must be actionable: {error}",
        );
    }

    /// The declared profile compiles the bounded program under its own contract.
    ///
    /// This is what makes the profile authoritative rather than decorative: the
    /// serial-sum program the prototypes package reaches a selected plan through
    /// it. The strict contract is separately refused below, so the acceptance is
    /// not the profile honouring everything.
    #[test]
    fn the_declared_profile_compiles_the_bounded_program() {
        let declaration = declared();
        let batch = compile(CompileRequest::new(
            &program(),
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            TargetRequest::new([declaration.profile().clone()])
                .expect("a singleton target request"),
        ))
        .expect("the bounded program compiles against the authoritative profile");
        let result = batch.targets().next().expect("one target outcome");
        let compilation = result.outcome().expect("the authoritative target compiles");
        assert!(compilation.selected().is_some());
        declaration
            .require_compiled_under(
                compilation.target_profile_key(),
                compilation.target_profile_descriptor(),
            )
            .expect("the compilation was assessed against exactly this declaration");
    }

    /// The exclusive subnormal table refuses the contract it cannot deliver.
    ///
    /// The perturbation is the caller's stated contract rather than the profile:
    /// a profile that accepted both contracts would prove nothing about the
    /// measured flush, and the measured row's whole content is that preservation
    /// is *unsupported*.
    #[test]
    fn the_declared_profile_refuses_a_subnormal_preserving_contract() {
        let declaration = declared();
        let batch = compile(CompileRequest::new(
            &program(),
            NumericalContract::STRICT_F32,
            TargetRequest::new([declaration.profile().clone()])
                .expect("a singleton target request"),
        ))
        .expect("the request is well formed");
        let result = batch.targets().next().expect("one target outcome");
        result
            .outcome()
            .expect_err("a measured flushing target cannot honour preserved subnormals");
    }

    /// Renders one measured build's role, producer-defined identity included.
    ///
    /// The wildcard arm panics naming the role rather than comparing unequal to a
    /// string, so a widened role vocabulary reports what arrived instead of
    /// reading as a moved toolchain row.
    fn role_label(role: TargetCompilerRoleReference<'_>) -> String {
        match role {
            TargetCompilerRoleReference::CodeGenerator => "code-generator".to_owned(),
            TargetCompilerRoleReference::Linker => "linker".to_owned(),
            TargetCompilerRoleReference::ProducerDefined(identity) => {
                format!(
                    "producer-defined:{}@{}",
                    identity.key(),
                    identity.revision()
                )
            }
            other => panic!("the ledger declares no {other:?} build"),
        }
    }

    /// The ledger's own rows refuse a strict `bf16` contract, and the refusal
    /// cites the ledger's own measurement.
    ///
    /// **This is the half `crates/tiler-compiler/tests/bf16_numerical_contract.rs`
    /// structurally cannot prove.** `tiler-build` depends on `tiler-compiler`, so
    /// that file cannot reach `FIRST_MACOS_APPLE9` and instead restates the same
    /// measured behaviour under its own test provenance — which shows the
    /// compiler boundary answers correctly for that behaviour, not that these
    /// rows produce that answer. Asserted here: the transcribed ledger rows yield
    /// the identical typed refusal, and its evidence names the ledger's measured
    /// producer, its four offline toolchain components, and the host they ran on.
    ///
    /// Every expected value is read from `FIRST_MACOS_APPLE9` rather than
    /// transcribed a second time, so what this pins is that each row *reaches*
    /// the refusal. A row's value is pinned separately, by
    /// `the_declaration_states_exactly_the_ledger_rows` and the descriptor
    /// perturbation sweeps.
    #[test]
    fn the_ledger_rows_refuse_a_strict_bf16_contract_with_their_own_measured_evidence() {
        let declaration = declared();
        let refusal = bf16_numerical_refusal(NumericalContract::STRICT_BF16);
        assert_eq!(
            refusal.target_profile(),
            declaration.profile().profile_key(),
            "the refusal is attributed to the authoritative profile itself",
        );
        let [rejection] = refusal.rejections() else {
            panic!("one stated contract, one rejection");
        };
        assert_eq!(
            rejection.contract_key(),
            NumericalContract::STRICT_BF16.key(),
        );

        let TargetNumericalRequirement::InputSubnormals { subject, required } =
            rejection.requirement()
        else {
            panic!(
                "the canonical-first unhonourable dimension is input subnormals, got {:?}",
                rejection.requirement(),
            );
        };
        assert_eq!(subject.arithmetic(), ArithmeticType::Bf16);
        assert_eq!(subject.resolved_type(), &Bf16::resolved_type());
        assert_eq!(*required, SubnormalMode::Preserve);

        let TargetNumericalRefusalDisposition::DeclaredUnhonourable(declared_row) =
            rejection.disposition()
        else {
            panic!(
                "the ledger declares this row, so it must refuse by name rather than \
                 degrade to Unknown: {:?}",
                rejection.disposition(),
            );
        };
        assert_eq!(declared_row.subject().arithmetic(), ArithmeticType::Bf16);
        assert_eq!(
            declared_row.subject().resolved_type(),
            &Bf16::resolved_type(),
        );
        assert_eq!(
            *declared_row.means(),
            TargetNumericalDeclaredMeans::Unsupported,
        );
        assert_eq!(
            declared_row.honoured(),
            Some(&TargetNumericalHonouredBehaviour::InputSubnormals(
                SIGN_PRESERVING_FLUSH
            )),
            "the refusal reports the flush the ledger's Metal record states",
        );
        assert_eq!(
            declared_row.target_profile(),
            declaration.profile().profile_key(),
        );

        // ---- the evidence is the ledger's measurement, not a fixture's ------
        // A caller cannot act on "this target refuses preserved bf16 subnormals"
        // — every flushing target says that. It can act on the exact offline
        // toolchain and host below, because it can compare them against its own
        // deployment, which is why the whole context is walked here.
        let evidence = declared_row.evidence();
        assert_eq!(evidence.available_at(), AvailabilityPhase::CompileProfile);
        assert_eq!(evidence.authority(), TargetFactAuthority::MeasuredProfile);
        assert_eq!(
            evidence.validity(),
            TargetFactValidityScope::MeasuredEnvironment,
            "a measured row may not widen into a portable claim",
        );
        assert_eq!(evidence.authority_identity().key(), MEASURED_PRODUCER);
        assert_eq!(evidence.authority_identity().revision(), 1);
        assert_eq!(
            evidence.target_profile(),
            declaration.profile().profile_key(),
        );

        let TargetNumericalEvidenceBasis::Measurement { contexts } = evidence.basis() else {
            panic!("a measured ledger row rests on measurement contexts");
        };
        assert_eq!(
            contexts.len(),
            1,
            "the ledger pairs one offline toolchain with one execution environment",
        );
        let context = contexts.get(0).expect("the one measurement context");
        let builds = context.compiler_builds();
        let mut observed = builds
            .iter()
            .map(|build| {
                (
                    build.implementation(),
                    role_label(build.role()),
                    build.version(),
                    build.build(),
                )
            })
            .collect::<Vec<_>>();
        observed.sort_unstable();
        let offline = &FIRST_MACOS_APPLE9.offline;
        let mut expected = vec![
            (
                "apple.metal-offline-compiler",
                "code-generator".to_owned(),
                offline.compiler_version,
                Some(offline.compiler_build),
            ),
            (
                "apple.air-lld",
                "linker".to_owned(),
                offline.linker_version,
                Some(offline.linker_build),
            ),
            (
                "apple.xcode",
                "producer-defined:tiler.metal.offline-toolchain-distribution@1".to_owned(),
                offline.xcode_version,
                Some(offline.xcode_build),
            ),
            (
                "apple.macos-sdk",
                "producer-defined:tiler.metal.offline-platform-sdk@1".to_owned(),
                offline.sdk_version,
                Some(offline.sdk_build),
            ),
        ];
        expected.sort_unstable();
        assert_eq!(
            observed, expected,
            "the refusal does not cite the ledger's four offline components",
        );

        let environment = context.environment();
        let execution = &FIRST_MACOS_APPLE9.execution;
        assert_eq!(environment.platform(), execution.platform);
        assert_eq!(environment.platform_version(), execution.platform_version);
        assert_eq!(environment.platform_build(), execution.platform_build);
        assert_eq!(environment.architecture(), execution.architecture);
        assert_eq!(environment.hardware(), execution.hardware);
    }

    /// A flush-accepting `bf16` contract clears the subnormal dimensions and
    /// meets the first the ledger does not declare.
    ///
    /// **The ledger's current boundary, asserted rather than described.** The
    /// macOS Apple9 declaration states BF16 dispatchability and the two subnormal
    /// tables and nothing else, so the next dimension an admitted operation
    /// consumes — contraction — has no BF16 row and resolves `Unknown`. That is
    /// the correct answer: the measurement covers subnormals, and the complete
    /// neighbouring `f32` table must not answer a `bf16` question. Asserting it
    /// here means widening the ledger's BF16 rows changes this test rather than
    /// passing silently.
    #[test]
    fn the_ledger_bf16_rows_leave_the_remaining_dimensions_unknown() {
        let refusal = bf16_numerical_refusal(NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16);
        let [rejection] = refusal.rejections() else {
            panic!("one stated contract, one rejection");
        };
        assert_eq!(
            rejection.contract_key(),
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16.key(),
        );
        assert_eq!(
            *rejection.disposition(),
            TargetNumericalRefusalDisposition::Unknown,
            "an undeclared bf16 dimension is Unknown, never the f32 row's answer",
        );
        let TargetNumericalRequirement::Contraction { subject, required } = rejection.requirement()
        else {
            panic!(
                "the first undeclared consumable dimension is contraction, got {:?}",
                rejection.requirement(),
            );
        };
        assert_eq!(subject.arithmetic(), ArithmeticType::Bf16);
        assert_eq!(
            subject.resolved_type(),
            &Bf16::resolved_type(),
            "the unanswered question names the bf16 subject, not a substituted f32 one",
        );
        assert_eq!(*required, NumericalPermission::Forbidden);
    }

    /// The projection carries the Metal record's own mode, not a restated one.
    ///
    /// The seam reads `MetalSubnormalArithmetic::subnormal_mode`, so a change to
    /// the Metal vocabulary's projection reaches the compiler profile rather
    /// than being shadowed by a second spelling of the same mode here.
    #[test]
    fn the_projection_carries_the_metal_records_own_mode() {
        let declaration = declared();
        let stated = declaration
            .metal_facts()
            .subnormal_arithmetic
            .behaviour(MetalFloatArithmeticType::F32)
            .expect("the ledger states the f32 row");
        assert_eq!(stated.subnormal_mode(), SIGN_PRESERVING_FLUSH);
    }
}
