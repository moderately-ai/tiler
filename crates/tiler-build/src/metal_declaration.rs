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
//! - **F16 and BF16** get no dispatchability and no numerical row. The measured
//!   Apple row *disagrees* across dtypes — `f32` arithmetic flushes where `f16`
//!   preserves on the same hardware in the same math modes — so inheritance is
//!   not merely unproven here, it is known to be unsound in one direction.
//!
//! # Which authority class each row carries
//!
//! The quantitative rows are **normative**: they come from primary Apple
//! documents and SDK headers, so they are declared through
//! [`TargetFactSource::external_guarantee`] naming the exact document as a
//! versioned normative reference. The dispatchability and numerical rows are
//! **measured**: they come from one retained MSL 4.0 run, so they are declared
//! through [`TargetCompileProfileMeasurementSource`], whose phase, authority,
//! and validity are fixed by construction and cannot widen into a portable
//! claim.
//!
//! Three normative references rather than one, because three different documents
//! establish the rows and a reader repairing a stale row needs to know which:
//! the macOS 26.5 SDK header for the grid-axis API contract, the Metal Feature
//! Set Tables for the family limits and 64-bit integer math, and the MSL 4.0
//! specification for the `device` address space.
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
    DTypeDispatchability, IndexArithmeticSupport, ScalarArithmetic, ScalarSupport,
    SynchronizationSupport, TargetCompileProfileMeasurementSource, TargetCompilerBuild,
    TargetCompilerRole, TargetCompilerRoleIdentity, TargetExecutionEnvironment,
    TargetFactProducerIdentity, TargetFactSource, TargetMeasurementContext,
    TargetNormativeReferenceIdentity, TargetProfile, TargetProfileBuildError, TargetProfileBuilder,
    TargetProfileKey, TargetProfileKeyError,
};
use tiler_ir::program::abi::{
    AvailabilityPhase, TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
};
use tiler_ir::schedule::{
    ExceptionalValueAssumption, FencedSpaces, MemoryOrdering, NumericalPermission,
    SynchronizationKind, SynchronizationScope, SynchronizationSubject,
};
use tiler_ir::semantic::F32;
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OfflineToolchainRow {
    compiler_version: &'static str,
    compiler_build: &'static str,
    linker_version: &'static str,
    linker_build: &'static str,
    xcode_version: &'static str,
    xcode_build: &'static str,
    sdk_version: &'static str,
    sdk_build: &'static str,
}

/// The ledger's execution environment, the host that ran the measured kernels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ExecutionRow {
    platform: &'static str,
    platform_version: &'static str,
    platform_build: &'static str,
    architecture: &'static str,
    hardware: &'static str,
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
    // quantitative limits, the MSL 4.0 standard, and `f32` as the one dtype with
    // dispatchability and numerical evidence.
    profile_key: "tiler.metal.macos-apple9.msl4-0.f32.v1",
    // "Grid-axis threads — 4": the macOS 26.5 SDK's `dispatchThreads:` contract
    // proves extent 4 is representable and establishes no upper bound at all, so
    // 4 is a deliberately conservative compile guarantee rather than a maximum.
    grid_axis_threads: 4,
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
    // "Metal target facts, and which of them project". The deployment minimum is
    // 26.0 and the standard MSL 4.0 because `probe.fixed_flags -std=metal4.0`
    // and `requested_target air64-apple-macos26.0` are the inputs the retained
    // measurement used; the older MSL 3.1 / macOS 14.0 record would attribute
    // these measurements to a compilation that did not produce them. Only `f32`
    // is stated: `f16` and `bf16` were not measured under MSL 4.0.
    facts: MetalTargetFacts::new(
        MslLanguageVersion::Metal4_0,
        MetalPlatform::MacOs,
        MetalDeploymentMinimum::new(26, 0),
        MetalSubnormalArithmeticFacts::unmeasured().stating(
            MetalFloatArithmeticType::F32,
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
/// The macOS 26.5 SDK header establishing the grid-axis API contract.
const SDK_DISPATCH_REFERENCE: &str =
    "apple.macos-sdk-26.5.mtlcomputecommandencoder.dispatch-threads";
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
const PREPARED_ENTRY_PROVIDER_NAMESPACE: &str = "tiler";
/// Property family the prepared-entry workgroup query is answered from.
const PREPARED_ENTRY_PROVIDER_NAME: &str = "prepared-entry-properties";

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
    /// Assembles the first authoritative macOS Apple9 MSL 4.0 `f32` declaration.
    ///
    /// # Errors
    ///
    /// Returns the exact refusing authority: an invalid profile key, a rejected
    /// provenance identity, a compiler-profile construction refusal, the
    /// buffer-capacity overlap check, the `f32` subnormal projection, or the AOT
    /// driver's own target validation.
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
    /// Only the buffer capacity and the `f32` subnormal entry of this record
    /// project into [`Self::profile`]. The language standard, artifact family,
    /// and deployment minimum have no compiler counterpart and must never be
    /// described as compiler-assessed.
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

        // ---- quantitative rows, every one normatively sourced ---------------
        builder
            .declare_max_threads_per_grid_axis(rows.grid_axis_threads, normative.sdk_dispatch)?;
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

        // ---- dispatchability, measured and exact ----------------------------
        // The retained run dispatched `f32` compute kernels on the execution
        // environment above and read results back, with an execution witness on
        // a non-subnormal operand separating "the arithmetic ran" from "the
        // kernel was optimized away". `f16` and `bf16` are deliberately absent.
        builder.declare_measured_dtype_dispatchability(
            F32::resolved_type(),
            DTypeDispatchability::Dispatchable,
            measured.clone(),
        )?;

        // ---- the one projected overlap: `f32` subnormal behaviour -----------
        // Declared through the ratified low-level seam, exactly once, and by
        // reading the Metal record rather than by restating the mode. Declaring
        // it twice would put two rows at one phase and is refused by the
        // complete-table conflict check inside that seam.
        declare_metal_f32_subnormal_behaviour(&mut builder, &rows.facts, measured.clone())?;

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

/// The three normative references the quantitative rows are attributed to.
struct NormativeSources {
    sdk_dispatch: TargetFactSource,
    feature_tables: TargetFactSource,
    msl_address_space: TargetFactSource,
    msl_barrier: TargetFactSource,
}

impl NormativeSources {
    fn declare(_rows: &LedgerRows) -> Result<Self, BoundMetalDeclarationError> {
        let producer = || TargetFactProducerIdentity::new(NORMATIVE_PRODUCER.to_owned(), 1);
        let reference = |key: &str| TargetNormativeReferenceIdentity::new(key.to_owned(), 1);
        Ok(Self {
            sdk_dispatch: TargetFactSource::external_guarantee(
                producer()?,
                reference(SDK_DISPATCH_REFERENCE)?,
            ),
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
            producer_defined("tiler.metal.offline-toolchain-distribution")?,
            "apple.xcode".to_owned(),
            offline.xcode_version.to_owned(),
            Some(offline.xcode_build.to_owned()),
        )?,
        TargetCompilerBuild::new(
            producer_defined("tiler.metal.offline-platform-sdk")?,
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
            Self::AotTarget(error) => ("AOT target", error),
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
            Self::Profile(error) => Some(error),
            Self::SubnormalProjection(error) => Some(error),
            Self::AotTarget(error) => Some(error),
            Self::PreparedEntryQuery | Self::BufferCapacityExceedsEmissionLimit { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoundMetalCompileDeclaration, BoundMetalDeclarationError, FIRST_MACOS_APPLE9, LedgerRows,
        MetalPlanProfileMismatch,
    };
    use tiler_compiler::session::{CompileRequest, NumericalContract, compile};
    use tiler_compiler::target::{
        DTypeDispatchabilityResolution, IndexArithmeticSupport, ScalarArithmetic, ScalarSupport,
        SynchronizationSupport, TargetFactProducerIdentity, TargetFactSource,
        TargetNormativeReferenceIdentity, TargetProfileBuildError, TargetProfileBuilder,
        TargetProfileKey, TargetRequest,
    };
    use tiler_ir::program::abi::{
        AvailabilityPhase, TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
    };
    use tiler_ir::schedule::{
        FlushedZeroSign, MemoryOrdering, SubnormalMode, SynchronizationKind, SynchronizationScope,
    };
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, ResolvedValueType,
        SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum, TypeKey,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_metal::target::{
        MetalDeploymentMinimum, MetalFloatArithmeticType, MetalFlushedZeroSign, MetalPlatform,
        MetalSubnormalArithmetic, MetalSubnormalArithmeticFacts, MetalTargetFacts,
        MslLanguageVersion,
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

    /// The declaration carries the ledger's rows, in both vocabularies.
    #[test]
    fn the_declaration_states_exactly_the_ledger_rows() {
        let declaration = declared();
        assert_eq!(
            declaration.profile().profile_key().as_str(),
            "tiler.metal.macos-apple9.msl4-0.f32.v1",
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
        let perturbations: [RowPerturbation; 6] = [
            ("grid-axis threads", |rows| rows.grid_axis_threads = 8),
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
            ("the projected f32 subnormal behaviour", |rows| {
                rows.facts = MetalTargetFacts::new(
                    rows.facts.language,
                    rows.facts.platform,
                    rows.facts.deployment_minimum,
                    MetalSubnormalArithmeticFacts::unmeasured().stating(
                        MetalFloatArithmeticType::F32,
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
        // from 1,741 when the barrier row and the permitted resolution of
        // reassociation were added.
        assert_eq!(
            descriptor.len(),
            1_963,
            "the canonical descriptor length moved; update the authority ledger with it",
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
        rows.profile_key = "tiler.metal.macos-apple9.msl4-0.f32.v2";
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
            NumericalContract::FlushSubnormalsToZeroF32,
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
            NumericalContract::StrictF32,
            TargetRequest::new([declaration.profile().clone()])
                .expect("a singleton target request"),
        ))
        .expect("the request is well formed");
        let result = batch.targets().next().expect("one target outcome");
        result
            .outcome()
            .expect_err("a measured flushing target cannot honour preserved subnormals");
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
        assert_eq!(
            stated.subnormal_mode(),
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
        );
    }
}
