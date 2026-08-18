//! The first evidence-backed Metal subgroup realization declaration.
//!
//! One value binds the atomic subgroup subject Tom accepted on 2026-08-11 to
//! the only retained evidence that licenses a width for it: the frozen
//! 34-pipeline `threadExecutionWidth` measurement of 2026-08-13 on the Apple
//! M3 Pro (`spikes/target-profiles/metal-thread-execution-width`, retained at
//! `results/2026-08-13-apple-m3-pro-macos27.0-26A5388g/widths.json`). Every
//! prepared pipeline in that population reported 32, so this declaration
//! states exactly one `Realized` subject —
//! `SubgroupRealizationSubject { width: 32, arithmetic: F32, transfer: InRangeXorShuffle }`
//! — plus the mandatory prepared subgroup-width query, and nothing else.
//!
//! # Why this is not a row on [`BoundMetalCompileDeclaration`]
//!
//! The standard macOS Apple9 declaration's authority ledger measures every
//! measured row on an **Apple M4 Max** execution host, and the width
//! measurement's own frozen protocol scopes what its data may inform before a
//! single width was read: *"Profile this may later inform: a **new** M3 Pro
//! Apple9 compile-profile width claim over the frozen population only. It does
//! not edit `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`."* Its outcome
//! restates the boundary after the run: the result *"does not source
//! `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` or any M4 Max qualified
//! row"*, and *"is not an Apple9-family guarantee"*. Declaring the row there
//! would attribute an M3 Pro observation to a profile whose every measured row
//! names another device — the same inheritance the standard declaration's own
//! module refuses for the evaluation-order row measured on a neighbouring
//! toolchain. The standard profile therefore **stays subgroup-silent**, its
//! descriptor stays byte-identical, and a test below asserts both.
//!
//! # What this profile is, and is not
//!
//! It is a compile-profile carrier for one measured target fact and its
//! prepared-entry confirmation query, scoped to the host row that produced the
//! evidence: macOS 27.0 build `26A5388g`, `arm64`, Apple M3 Pro, under the
//! ledger-identical offline toolchain (metal `32023.883`, AIR-LLD `32023.883`,
//! Xcode 26.6 `17F113`, SDK 26.5 `25F70`). It deliberately declares no
//! dispatchability, numerical, quantitative, or synchronization row: none of
//! those was measured on this host, and a fact family gains a row from its own
//! evidence ticket or stays `Unknown`. Under ADR 0094 decision 7 the declared
//! width licenses nothing by itself — every subgroup-using entry still carries
//! an `ObservedEqualsRequired` prepared-width requirement the exact prepared
//! pipeline must satisfy before routing commit.
//!
//! # The Metal-owned validation the accepted decision requires
//!
//! The accepted decision places backend-family correspondence here rather than
//! in the generic builder: *"The generic builder does not guess a target
//! family. Metal-owned binding validates that any Metal row matches the Metal
//! compilation/profile evidence."* [`BoundMetalSubgroupDeclaration::declare`]
//! refuses, with a named reason:
//!
//! - a `Realized` width other than the retained record's 32
//!   ([`BoundMetalSubgroupDeclarationError::UnevidencedWidth`]);
//! - a `Realized` BF16 or F64 XOR-shuffle subject — MSL Table 6.14 excludes
//!   `bfloat` from `simd_shuffle_xor` and Metal has no `double`, and the
//!   retained record holds both compile failures as rows
//!   ([`BoundMetalSubgroupDeclarationError::ShuffleUndefinedForArithmetic`]);
//! - a `Realized` F16 subject — the frozen protocol designates F16 a control,
//!   not an authorized candidate family, so its prepared width is evidence
//!   about the *population* and licenses no realization claim
//!   ([`BoundMetalSubgroupDeclarationError::ControlOnlyArithmetic`]).
//!
//! The generic builder keeps what it already owns: duplicate/contradiction
//! refusal, the query's phase, and the missing/orphan query contract at
//! `build()`.
//!
//! Every public item here is a reviewed *draft* boundary (ADR 0074 §7 / ADR
//! 0075): built and tested at full fidelity while Tom reviews the surface.

use core::fmt;
use std::error::Error;

use tiler_compiler::target::{
    SubgroupSupport, TargetCompileProfileMeasurementSource, TargetCompilerBuild,
    TargetCompilerRole, TargetCompilerRoleIdentity, TargetExecutionEnvironment,
    TargetFactProducerIdentity, TargetMeasurementContext, TargetProfile, TargetProfileBuildError,
    TargetProfileBuilder, TargetProfileKey, TargetProfileKeyError,
};
use tiler_ir::program::abi::{
    AvailabilityPhase, TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
};
use tiler_ir::schedule::{
    ArithmeticType, SubgroupRealizationError, SubgroupRealizationSubject, SubgroupTransfer,
    SubgroupWidth,
};

use crate::metal_declaration::{
    ExecutionRow, OFFLINE_DISTRIBUTION_ROLE, OFFLINE_SDK_ROLE, OfflineToolchainRow,
    PREPARED_ENTRY_PROVIDER_NAME, PREPARED_ENTRY_PROVIDER_NAMESPACE,
};

/// Every retained-measurement row this declaration is assembled from.
///
/// Private, and taken by [`BoundMetalSubgroupDeclaration::declare`] rather than
/// read from constants inline, for the same reason the standard declaration's
/// `LedgerRows` is: the perturbation cases must be able to move exactly one row
/// and observe the refusal, or the descriptor movement, that row owns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SubgroupLedgerRows {
    profile_key: &'static str,
    /// The one width every prepared pipeline in the frozen population
    /// reported. `verdict.widths_observed` is `[32]` in the retained record;
    /// a declared width differing from this is a claim the evidence does not
    /// state and is refused before any row is declared.
    measured_width_lanes: u32,
    /// The width the one declared subject names. Equal to the measured width
    /// in the production rows; separate so the perturbation case can move it
    /// alone and watch [`BoundMetalSubgroupDeclarationError::UnevidencedWidth`].
    realized_width_lanes: u32,
    /// The arithmetic type of the one declared subject.
    realized_arithmetic: ArithmeticType,
    /// The verdict on that subject.
    ///
    /// Separate so a perturbation can flip it to
    /// [`SubgroupSupport::Unrealizable`] and observe the generic builder's
    /// orphan-query refusal at `build()` — the width query is mandatory with a
    /// `Realized` row and forbidden without one.
    support: SubgroupSupport,
    /// The governed prepared subgroup-width property key, or `None` to omit
    /// the mandatory query so a test can observe the builder's
    /// missing-query refusal at `build()`.
    subgroup_property_key: Option<&'static str>,
    offline: OfflineToolchainRow,
    execution: ExecutionRow,
}

/// The exact rows the retained M3 Pro width measurement admits.
///
/// Each value is transcribed from the retained record named beside it.
/// Changing one here is a claim about a source, not a tuning knob.
const FIRST_M3PRO_APPLE9_SUBGROUP: SubgroupLedgerRows = SubgroupLedgerRows {
    // Keyed by what bounds every row in it: the macOS artifact family, the one
    // M3 Pro Apple9 execution host the widths were read on, the MSL 4.0
    // offline flag set the pipelines were compiled under, and the single F32
    // subgroup claim. Deliberately not `macos-apple9`: the retained record is
    // explicit that it is not an Apple9-family guarantee.
    profile_key: "tiler.metal.macos-m3pro-apple9.msl4-0.subgroup-f32.v1",
    // "Every retained width is 32": 31 prepared identities, three repetitions
    // each, 93 observations, `widths_observed == [32]`.
    measured_width_lanes: 32,
    realized_width_lanes: 32,
    // The one authorized candidate family that compiled and prepared:
    // `xor_shuffle_f32/profile_strict/default` and its thirteen descriptor,
    // compiler-selection, and threadgroup-shape controls. BF16 — the other
    // authorized candidate — failed to compile (`no matching function for
    // call to 'simd_shuffle_xor'`) and stays undeclared, not `Unrealizable`
    // by silence: silence is `Unknown`, and an explicit negative row is a
    // separate decision nothing here takes.
    realized_arithmetic: ArithmeticType::F32,
    support: SubgroupSupport::Realized,
    // The governed key the accepted prepared-width gate dispatches on; the
    // Candle Metal adapter answers it from the exact retained pipeline's
    // `threadExecutionWidth` under the `tiler`/`prepared-entry-properties`@1
    // provider.
    subgroup_property_key: Some("tiler.target.prepared-entry.subgroup-width.v1"),
    // The retained record's offline compilation table — identical, field for
    // field, to the standard declaration's ledger row, and re-observed on the
    // measuring host before the run rather than inherited.
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
    // The retained record's execution environment. The hardware row is the
    // field that separates this declaration from the standard one.
    execution: ExecutionRow {
        platform: "macos",
        platform_version: "27.0",
        platform_build: "26A5388g",
        architecture: "arm64",
        hardware: "Apple M3 Pro",
    },
};

/// Producer of every measured row in this declaration.
const MEASURED_PRODUCER: &str = "tiler.metal.m3pro-apple9-subgroup.measured.v1";

/// One checked, versioned M3 Pro Apple9 subgroup-width declaration.
///
/// Constructed only by [`Self::first_m3_pro_apple9`]: there is no public
/// constructor taking rows, because a caller minting a subgroup fact for a
/// subject nobody measured is exactly what the retained record's frozen
/// protocol exists to prevent. Widening this to another device, arithmetic
/// type, width, or transfer is a new measurement rather than a new argument.
#[derive(Clone, Debug)]
pub struct BoundMetalSubgroupDeclaration {
    profile: TargetProfile,
    subject: SubgroupRealizationSubject,
}

impl BoundMetalSubgroupDeclaration {
    /// Assembles the first evidence-backed M3 Pro Apple9 subgroup declaration.
    ///
    /// # Errors
    ///
    /// Returns the exact refusing authority: an invalid profile key, a
    /// rejected provenance identity, a subject the checked constructor cannot
    /// form, a Metal-evidence correspondence refusal, a refused query, or the
    /// generic builder's own row refusal.
    pub fn first_m3_pro_apple9() -> Result<Self, BoundMetalSubgroupDeclarationError> {
        Self::declare(&FIRST_M3PRO_APPLE9_SUBGROUP)
    }

    /// Returns the checked compiler profile carrying the declared rows.
    #[must_use]
    pub const fn profile(&self) -> &TargetProfile {
        &self.profile
    }

    /// Returns the one subject this declaration states as `Realized`.
    ///
    /// The width a subgroup-using entry's prepared-width requirement must
    /// carry is `self.realized_subject().width()`; deriving it anywhere else
    /// would put the number under a second authority.
    #[must_use]
    pub const fn realized_subject(&self) -> SubgroupRealizationSubject {
        self.subject
    }

    fn declare(rows: &SubgroupLedgerRows) -> Result<Self, BoundMetalSubgroupDeclarationError> {
        let width = SubgroupWidth::new(rows.realized_width_lanes)
            .map_err(BoundMetalSubgroupDeclarationError::Subject)?;
        let subject = SubgroupRealizationSubject::new(
            width,
            rows.realized_arithmetic,
            SubgroupTransfer::InRangeXorShuffle,
        )
        .map_err(BoundMetalSubgroupDeclarationError::Subject)?;

        // ---- the Metal-owned evidence correspondence ------------------------
        // Validated before any row reaches the generic builder, because the
        // builder deliberately owns no backend family and cannot ask these
        // questions. Each arm names its authority; the match is exhaustive so
        // a widened `ArithmeticType` stops this build here rather than
        // inheriting an answer.
        if matches!(rows.support, SubgroupSupport::Realized) {
            match rows.realized_arithmetic {
                // `xor_shuffle_f32` compiled, prepared, and reported the
                // measured width in the retained record; `simd_shuffle_xor`
                // over `float` is MSL Table 6.14.
                ArithmeticType::F32 => {}
                // `xor_shuffle_f16` prepared as a *control*: the frozen
                // protocol names F32 and BF16 as the only authorized candidate
                // families, so a prepared control width is population
                // evidence, not a realization licence.
                ArithmeticType::F16 => {
                    return Err(BoundMetalSubgroupDeclarationError::ControlOnlyArithmetic {
                        arithmetic: rows.realized_arithmetic,
                    });
                }
                // `xor_shuffle_bf16` failed to compile (`no matching function
                // for call to 'simd_shuffle_xor'`; MSL Table 6.14 excludes
                // `bfloat`), and `xor_shuffle_f64` failed with `'double' is
                // not supported in Metal`. Both failures are retained rows.
                ArithmeticType::Bf16 | ArithmeticType::F64 => {
                    return Err(
                        BoundMetalSubgroupDeclarationError::ShuffleUndefinedForArithmetic {
                            arithmetic: rows.realized_arithmetic,
                        },
                    );
                }
            }
            if rows.realized_width_lanes != rows.measured_width_lanes {
                return Err(BoundMetalSubgroupDeclarationError::UnevidencedWidth {
                    declared: rows.realized_width_lanes,
                    measured: rows.measured_width_lanes,
                });
            }
        }

        let mut builder =
            TargetProfileBuilder::new(TargetProfileKey::new(rows.profile_key.to_owned())?);
        builder.declare_measured_subgroup_realization(
            subject,
            rows.support,
            measured_source(rows)?,
        )?;
        if let Some(key) = rows.subgroup_property_key {
            builder.declare_subgroup_width_query(
                TargetPropertyQuery::new(
                    TargetPropertyKey::new(key)
                        .map_err(|_| BoundMetalSubgroupDeclarationError::SubgroupQuery)?,
                    AvailabilityPhase::PreparedKernelPreflight,
                    TargetPropertyProviderIdentity::new(
                        PREPARED_ENTRY_PROVIDER_NAMESPACE,
                        PREPARED_ENTRY_PROVIDER_NAME,
                        1,
                    )
                    .map_err(|_| BoundMetalSubgroupDeclarationError::SubgroupQuery)?,
                )
                .map_err(|_| BoundMetalSubgroupDeclarationError::SubgroupQuery)?,
            )?;
        }
        let profile = builder.build()?;
        Ok(Self { profile, subject })
    }
}

/// Builds the one measurement source every row here shares.
///
/// One context pairing the four offline toolchain components with the M3 Pro
/// execution environment, because that pair *is* the measurement: this
/// toolchain compiled the frozen population and this host prepared it and
/// reported the widths. The producer identity is this declaration's own — the
/// standard declaration's measured producer names a different retained
/// population on a different device, and sharing it would merge two evidence
/// sets that share nothing but a toolchain.
fn measured_source(
    rows: &SubgroupLedgerRows,
) -> Result<TargetCompileProfileMeasurementSource, BoundMetalSubgroupDeclarationError> {
    let offline = &rows.offline;
    let producer_defined =
        |key: &str| -> Result<TargetCompilerRole, BoundMetalSubgroupDeclarationError> {
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

/// Why the bound M3 Pro subgroup declaration could not be assembled.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BoundMetalSubgroupDeclarationError {
    /// The declared profile key is not a valid target-profile key.
    ProfileKey(TargetProfileKeyError),
    /// A producer, compiler-role, compiler-build, or execution-environment
    /// identity was refused.
    Provenance(tiler_compiler::target::TargetFactSourceError),
    /// The checked subject constructor refused the width/transfer pair.
    Subject(SubgroupRealizationError),
    /// The declared `Realized` width is not the retained record's width.
    ///
    /// The Metal-owned half of validation: the generic builder accepts any
    /// checked subject, and only this binding knows which width the retained
    /// M3 Pro measurement observed.
    UnevidencedWidth {
        /// Lanes the rows declare.
        declared: u32,
        /// Lanes every prepared pipeline in the retained record reported.
        measured: u32,
    },
    /// MSL defines no XOR shuffle at this arithmetic type, and the retained
    /// record holds the compile failure as a row.
    ShuffleUndefinedForArithmetic {
        /// The refused arithmetic type.
        arithmetic: ArithmeticType,
    },
    /// The frozen protocol designates this arithmetic a control, not an
    /// authorized candidate family, so no realization claim is licensed.
    ControlOnlyArithmetic {
        /// The refused arithmetic type.
        arithmetic: ArithmeticType,
    },
    /// The prepared subgroup-width query is not statable at its own phase.
    SubgroupQuery,
    /// The compiler target profile refused a declared row.
    Profile(TargetProfileBuildError),
}

impl From<TargetProfileKeyError> for BoundMetalSubgroupDeclarationError {
    fn from(error: TargetProfileKeyError) -> Self {
        Self::ProfileKey(error)
    }
}

impl From<tiler_compiler::target::TargetFactSourceError> for BoundMetalSubgroupDeclarationError {
    fn from(error: tiler_compiler::target::TargetFactSourceError) -> Self {
        Self::Provenance(error)
    }
}

impl From<TargetProfileBuildError> for BoundMetalSubgroupDeclarationError {
    fn from(error: TargetProfileBuildError) -> Self {
        Self::Profile(error)
    }
}

impl fmt::Display for BoundMetalSubgroupDeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (what, cause): (_, &dyn fmt::Display) = match self {
            Self::ProfileKey(error) => ("profile key", error),
            Self::Provenance(error) => ("fact source", error),
            Self::Subject(error) => ("subgroup subject", error),
            Self::Profile(error) => ("compiler profile row", error),
            Self::UnevidencedWidth { declared, measured } => {
                return write!(
                    formatter,
                    "subgroup width: declared {declared} lane(s), the retained M3 Pro record \
                     observed {measured} on every prepared pipeline",
                );
            }
            Self::ShuffleUndefinedForArithmetic { arithmetic } => {
                return write!(
                    formatter,
                    "subgroup transfer: MSL defines no XOR shuffle at {}; the retained record \
                     holds the compile failure",
                    arithmetic.canonical_type_key(),
                );
            }
            Self::ControlOnlyArithmetic { arithmetic } => {
                return write!(
                    formatter,
                    "subgroup subject: {} is a control in the frozen population, not an \
                     authorized candidate family",
                    arithmetic.canonical_type_key(),
                );
            }
            Self::SubgroupQuery => {
                return formatter.write_str(
                    "prepared subgroup-width query: not statable at PreparedKernelPreflight",
                );
            }
        };
        write!(formatter, "{what}: {cause}")
    }
}

impl Error for BoundMetalSubgroupDeclarationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProfileKey(error) => Some(error),
            Self::Provenance(error) => Some(error),
            Self::Subject(error) => Some(error),
            Self::Profile(error) => Some(error),
            Self::UnevidencedWidth { .. }
            | Self::ShuffleUndefinedForArithmetic { .. }
            | Self::ControlOnlyArithmetic { .. }
            | Self::SubgroupQuery => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoundMetalSubgroupDeclaration, BoundMetalSubgroupDeclarationError,
        FIRST_M3PRO_APPLE9_SUBGROUP, MEASURED_PRODUCER, SubgroupLedgerRows,
    };
    use crate::metal_declaration::BoundMetalCompileDeclaration;
    use tiler_compiler::target::{
        SubgroupRealizationResolution, SubgroupSupport, TargetProfileBuildError,
    };
    use tiler_ir::program::abi::AvailabilityPhase;
    use tiler_ir::schedule::{
        ArithmeticType, SubgroupRealizationSubject, SubgroupTransfer, SubgroupWidth,
    };

    /// One named single-row mutation, mirroring the standard declaration's
    /// perturbation sweeps.
    type RowPerturbation = (&'static str, fn(&mut SubgroupLedgerRows));

    fn declared() -> BoundMetalSubgroupDeclaration {
        BoundMetalSubgroupDeclaration::first_m3_pro_apple9()
            .expect("the retained record's rows assemble one bound declaration")
    }

    fn subject(lanes: u32, arithmetic: ArithmeticType) -> SubgroupRealizationSubject {
        SubgroupRealizationSubject::new(
            SubgroupWidth::new(lanes).expect("a nonzero width"),
            arithmetic,
            SubgroupTransfer::InRangeXorShuffle,
        )
        .expect("a checked subject")
    }

    /// The declaration states exactly the retained record's rows.
    #[test]
    fn the_declaration_states_exactly_the_retained_rows() {
        let declaration = declared();
        assert_eq!(
            declaration.profile().profile_key().as_str(),
            "tiler.metal.macos-m3pro-apple9.msl4-0.subgroup-f32.v1",
        );
        let realized = declaration.realized_subject();
        assert_eq!(realized.width().get(), 32);
        assert_eq!(realized.arithmetic(), ArithmeticType::F32);
        assert_eq!(realized.transfer(), SubgroupTransfer::InRangeXorShuffle);
        assert_eq!(
            declaration
                .profile()
                .subgroup_realization(realized, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Realized,
        );
    }

    /// Every neighbouring subject resolves `Unknown`, never the declared row.
    ///
    /// One neighbour per dimension the whole-subject equality guards: a wider
    /// width, a narrower width, and each undeclared arithmetic type. This is
    /// the ticket's "differing in exactly one dimension" evidence restated
    /// against the production rows.
    #[test]
    fn every_neighbouring_subject_resolves_unknown() {
        let declaration = declared();
        for neighbour in [
            subject(64, ArithmeticType::F32),
            subject(16, ArithmeticType::F32),
            subject(32, ArithmeticType::F16),
            subject(32, ArithmeticType::Bf16),
            subject(32, ArithmeticType::F64),
        ] {
            assert_eq!(
                declaration
                    .profile()
                    .subgroup_realization(neighbour, AvailabilityPhase::CompileProfile),
                SubgroupRealizationResolution::Unknown,
                "a neighbouring subject must stay Unknown: {neighbour:?}",
            );
        }
    }

    /// The descriptor carries the subgroup families, the governed query key,
    /// and the M3 Pro measurement context.
    #[test]
    fn the_descriptor_carries_the_subgroup_families_and_the_m3_pro_context() {
        let declaration = declared();
        let text = String::from_utf8_lossy(declaration.profile().canonical_descriptor());
        assert!(
            text.contains("tiler.target-profile.subgroup-realization.v1"),
            "the subgroup-realization family is absent from the descriptor",
        );
        assert!(
            text.contains("tiler.target-profile.subgroup-width-query.v1"),
            "the subgroup-width-query family is absent from the descriptor",
        );
        assert!(
            text.contains("tiler.target.prepared-entry.subgroup-width.v1"),
            "the governed prepared-width key is absent from the descriptor",
        );
        assert!(text.contains(MEASURED_PRODUCER), "the producer is absent");
        assert!(text.contains("Apple M3 Pro"), "the hardware row is absent");
        assert!(text.contains("26A5388g"), "the OS build row is absent");
        assert!(text.contains("32023.883"), "the compiler row is absent");
    }

    /// The standard macOS Apple9 declaration stays subgroup-silent.
    ///
    /// The retained record's frozen protocol scopes its data away from
    /// `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` by name, so the standard
    /// profile must keep resolving every subgroup subject `Unknown` and its
    /// descriptor must write neither subgroup family — which is also what
    /// keeps every standard-path artifact identity and cache pin exactly
    /// where the prepared-width gate landing recomputed them.
    #[test]
    fn the_standard_declaration_stays_subgroup_silent() {
        let standard = BoundMetalCompileDeclaration::first_macos_apple9()
            .expect("the standard declaration assembles");
        assert_eq!(
            standard.profile().subgroup_realization(
                subject(32, ArithmeticType::F32),
                AvailabilityPhase::CompileProfile
            ),
            SubgroupRealizationResolution::Unknown,
            "the standard profile must not inherit the M3 Pro width row",
        );
        let text = String::from_utf8_lossy(standard.profile().canonical_descriptor());
        assert!(
            !text.contains("tiler.target-profile.subgroup-realization.v1"),
            "the standard descriptor gained a subgroup-realization section",
        );
        assert!(
            !text.contains("tiler.target-profile.subgroup-width-query.v1"),
            "the standard descriptor gained a subgroup-width-query section",
        );
    }

    /// A width the retained record did not observe is refused by name.
    #[test]
    fn an_unevidenced_width_is_refused() {
        let mut rows = FIRST_M3PRO_APPLE9_SUBGROUP;
        rows.realized_width_lanes = 64;
        assert_eq!(
            BoundMetalSubgroupDeclaration::declare(&rows).unwrap_err(),
            BoundMetalSubgroupDeclarationError::UnevidencedWidth {
                declared: 64,
                measured: 32,
            },
        );
    }

    /// A BF16 or F64 XOR-shuffle subject is refused with the MSL evidence.
    #[test]
    fn a_shuffle_undefined_arithmetic_is_refused() {
        for arithmetic in [ArithmeticType::Bf16, ArithmeticType::F64] {
            let mut rows = FIRST_M3PRO_APPLE9_SUBGROUP;
            rows.realized_arithmetic = arithmetic;
            assert_eq!(
                BoundMetalSubgroupDeclaration::declare(&rows).unwrap_err(),
                BoundMetalSubgroupDeclarationError::ShuffleUndefinedForArithmetic { arithmetic },
            );
        }
    }

    /// The F16 control's prepared width licenses no realization claim.
    #[test]
    fn the_f16_control_is_refused_as_a_candidate() {
        let mut rows = FIRST_M3PRO_APPLE9_SUBGROUP;
        rows.realized_arithmetic = ArithmeticType::F16;
        assert_eq!(
            BoundMetalSubgroupDeclaration::declare(&rows).unwrap_err(),
            BoundMetalSubgroupDeclarationError::ControlOnlyArithmetic {
                arithmetic: ArithmeticType::F16,
            },
        );
    }

    /// Omitting the mandatory width query refuses at `build()`.
    ///
    /// The generic builder owns this contract; what this case pins is that the
    /// Metal factory cannot assemble a `Realized` row without the query — the
    /// missing-query population the checked-profile contract closed.
    #[test]
    fn a_realized_row_without_the_query_is_refused() {
        let mut rows = FIRST_M3PRO_APPLE9_SUBGROUP;
        rows.subgroup_property_key = None;
        assert_eq!(
            BoundMetalSubgroupDeclaration::declare(&rows).unwrap_err(),
            BoundMetalSubgroupDeclarationError::Profile(
                TargetProfileBuildError::MissingSubgroupWidthQuery,
            ),
        );
    }

    /// An `Unrealizable`-only declaration must not carry the query, and the
    /// pair of cases pins both directions of the mandatory pairing.
    #[test]
    fn an_unrealizable_row_forbids_the_query_and_builds_without_it() {
        let mut orphan = FIRST_M3PRO_APPLE9_SUBGROUP;
        orphan.support = SubgroupSupport::Unrealizable;
        assert_eq!(
            BoundMetalSubgroupDeclaration::declare(&orphan).unwrap_err(),
            BoundMetalSubgroupDeclarationError::Profile(
                TargetProfileBuildError::OrphanSubgroupWidthQuery,
            ),
        );

        let mut negative = FIRST_M3PRO_APPLE9_SUBGROUP;
        negative.support = SubgroupSupport::Unrealizable;
        negative.subgroup_property_key = None;
        let negative = BoundMetalSubgroupDeclaration::declare(&negative)
            .expect("a stated negative without a query is a coherent profile");
        assert_eq!(
            negative.profile().subgroup_realization(
                subject(32, ArithmeticType::F32),
                AvailabilityPhase::CompileProfile,
            ),
            SubgroupRealizationResolution::Unrealizable,
        );
    }

    /// Every identity-bearing row moves the canonical descriptor.
    ///
    /// One perturbation per row rather than one for the set, exactly as the
    /// standard declaration's sweeps do. The support flip and the width move
    /// are refused before a descriptor exists, so the movable rows here are
    /// the measurement-context fields and the profile key.
    #[test]
    fn every_measurement_context_field_moves_the_descriptor() {
        let descriptor = |rows: &SubgroupLedgerRows| {
            BoundMetalSubgroupDeclaration::declare(rows)
                .expect("the perturbed rows still assemble")
                .profile()
                .canonical_descriptor()
                .to_vec()
        };
        let baseline = descriptor(&FIRST_M3PRO_APPLE9_SUBGROUP);
        let perturbations: [RowPerturbation; 5] = [
            ("the offline compiler build", |rows| {
                rows.offline.compiler_build = "metalfe-32023.884";
            }),
            ("the Xcode build", |rows| {
                rows.offline.xcode_build = "17F114";
            }),
            ("the execution OS build", |rows| {
                rows.execution.platform_build = "26A5406e";
            }),
            ("the execution hardware", |rows| {
                rows.execution.hardware = "Apple M4 Max";
            }),
            ("the profile key", |rows| {
                rows.profile_key = "tiler.metal.macos-m3pro-apple9.msl4-0.subgroup-f32.v2";
            }),
        ];
        for (name, perturb) in perturbations {
            let mut rows = FIRST_M3PRO_APPLE9_SUBGROUP;
            perturb(&mut rows);
            assert_ne!(
                descriptor(&rows),
                baseline,
                "{name} does not reach the profile descriptor",
            );
        }
    }
}
