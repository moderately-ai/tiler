//! Public target-profile declaration vocabulary.
//!
//! Request coordination consumes an immutable checked profile; it does not own
//! the vocabulary by which target producers state capability, numerical, or
//! dtype-dispatchability facts.
//!
//! Tom accepted this public boundary at commit `4ad5a2e` on 2026-07-30.
//! It exposes externally attributed normative guarantees and measurements.
//! The compiler-governed source and exact-emulation proof constructors remain
//! private because a caller cannot manufacture either authority.
//!
//! [`ScalarArithmetic::new`] is **not** covered by that acceptance. It is the
//! validated arithmetic/value-type construction route added later so a BF16
//! numerical row could be stated at all, and a new public method on an
//! already-public type falls in the gap ADR 0075's two category lists leave —
//! neither a new namespace, trait, promotion, or breaking signature change, nor
//! one of the categories a coordinator may merge unaided. It is a tested
//! concrete draft of that interface awaiting a boundary decision of its own, and
//! this sentence is what a reader should find rather than an acceptance that was
//! never given.
//!
//! The **evaluation-order-preservation** family carries an acceptance of its
//! own rather than the `4ad5a2e` one. Tom accepted it on 2026-08-06 at the live
//! session's decision round, as-is with no exclusion, under
//! `accept-the-evaluation-order-preservation-target-fact`:
//! [`BackendArithmeticLicence`], [`EvaluationOrderPreservation`],
//! [`EvaluationOrderResolution`],
//! [`TargetProfileBuilder::declare_evaluation_order_preservation`],
//! [`TargetProfileBuilder::declare_measured_evaluation_order_preservation`],
//! [`TargetProfile::evaluation_order_preservation`], and
//! [`TargetProfileBuildError::DuplicateEvaluationOrderPreservation`].
//!
//! The **elementary-realization** family is a **labelled draft** under ADR
//! 0075. Tom accepted the whole-subject *shape* on 2026-08-11 — one validated
//! record, operation derived from a verified contract, both complete evidence
//! records, compile-profile-phase source, stored canonical rows, no governed
//! shortcut — and has not accepted this crate's exact method, type, or
//! refusal-candidate spelling. [`ElementaryRealization`],
//! [`TargetProfileBuilder::declare_elementary_realization`],
//! [`TargetProfile::declared_elementary_realizations`], and
//! [`TargetProfileBuildError::DuplicateElementaryRealization`] are that draft.
//!
//! The **measured-cost-row** family is an **Accepted public surface**. Tom
//! accepted it on 2026-08-07 under
//! `accept-the-measured-cost-row-public-surface`:
//! [`TargetCostRowResolution`],
//! [`TargetProfileBuilder::declare_saturated_parallel_fold_steps`],
//! [`TargetProfileBuilder::declare_measured_saturated_parallel_fold_steps`],
//! [`TargetProfile::saturated_parallel_fold_steps`], and
//! [`TargetProfileBuildError::DuplicateCostRow`]. Its selection may consult a
//! measured term where a qualified profile declares one, carried as a kind
//! distinct from a capability axis, with silence meaning *no preference* rather
//! than *no plan*.
//!
//! The **subgroup-realization** family is a **labelled draft** under ADR 0075.
//! Tom accepted the whole-subject *shape* on 2026-08-11 — one checked subject
//! over a literal width, an exact arithmetic type, and an operation-specific
//! transfer, matched only by equality, with `Realized` and `Unrealizable`
//! explicit and silence/`Unknown` for neighbours — and has not accepted this
//! crate's exact type, constructor, or error spelling.
//! [`SubgroupSupport`], [`SubgroupRealizationResolution`],
//! [`TargetProfileBuilder::declare_subgroup_realization`],
//! [`TargetProfileBuilder::declare_measured_subgroup_realization`],
//! [`TargetProfile::subgroup_realization`], and
//! [`TargetProfileBuildError::DuplicateSubgroupRealization`] are that draft.
//! The **prepared subgroup-width query** joined the same family under the
//! accepted 2026-08-11 prepared-width gate
//! (`decide-the-prepared-subgroup-width-equality-gate`): a profile declaring
//! any subject `Realized` carries exactly one profile-level
//! `PreparedKernelPreflight` subgroup-width query, and a missing, duplicate,
//! wrong-phase, or orphan query refuses at construction.
//! [`TargetProfileBuilder::declare_subgroup_width_query`] and the
//! [`TargetProfileBuildError::MissingSubgroupWidthQuery`],
//! [`TargetProfileBuildError::OrphanSubgroupWidthQuery`],
//! [`TargetProfileBuildError::DuplicateSubgroupWidthQuery`], and
//! [`TargetProfileBuildError::InvalidSubgroupQueryPhase`] spellings are the
//! labelled draft of that accepted contract.
//!
//! The **workgroup-tree-width-policy** family is an **Accepted public surface**.
//! Tom delegated the choice to the coordinator on 2026-08-11 under
//! `gate-the-workgroup-tree-on-an-explicit-qualified-width-policy`:
//! [`WorkgroupTreeWidthPolicy`], [`WorkgroupTreeWidthPolicyResolution`],
//! [`TargetProfileBuilder::declare_workgroup_tree_width_policy`],
//! [`TargetProfileBuilder::declare_measured_workgroup_tree_width_policy`],
//! [`TargetProfile::workgroup_tree_width_policy`], and
//! [`TargetProfileBuildError::DuplicateWorkgroupTreeWidthPolicy`]. One closed
//! variant, no omitted/default case, and no public numeric cap: a profile that
//! does not declare an accepted policy makes the single-workgroup tree
//! unavailable with a typed reason. Silence is not a preference and is not a
//! clamp onto `256`.
//!
//! Four exclusions were accepted with the evaluation-order-preservation family
//! and are deliberate rather than gaps: no
//! math-mode spelling, because `safe`/`relaxed`/`fast` are one backend driver's
//! option tokens and the licence is what the measurement attributes the
//! behaviour to; no twelfth numerical dimension, because this states what
//! Tiler's emission grants the backend translator rather than what a caller's
//! contract grants Tiler; no `Unknown` variant on the verdict, because absence
//! is the `Unknown` as it is for dtype dispatchability; and no feasibility
//! consumer, the fact being declared and resolvable while nothing yet admits or
//! refuses on it.
//!
//! ```
//! use tiler_compiler::target::{
//!     DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport,
//!     TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
//!     TargetProfileBuilder, TargetProfileKey, TargetRequest,
//! };
//! use tiler_ir::semantic::F32;
//!
//! let producer = TargetFactProducerIdentity::new("acme.gpu-profile.v1".to_owned(), 1)?;
//! let specification =
//!     TargetNormativeReferenceIdentity::new("acme.gpu-specification.v3".to_owned(), 3)?;
//! let source = TargetFactSource::external_guarantee(producer, specification);
//! let mut builder =
//!     TargetProfileBuilder::new(TargetProfileKey::new("acme.gpu.family-a.v1".to_owned())?);
//! builder.declare_max_threads_per_grid_axis(65_535, source.clone())?;
//! builder.declare_max_threads_per_workgroup(256, source.clone())?;
//! builder.declare_max_buffer_bindings_per_entry(31, source.clone())?;
//! builder.declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())?;
//! builder.declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())?;
//! builder.declare_device_memory(true, source.clone())?;
//! builder.declare_local_memory_bytes(32_768, source.clone())?;
//! builder.declare_dtype_dispatchability(
//!     F32::resolved_type(),
//!     DTypeDispatchability::Dispatchable,
//!     source,
//! )?;
//! let profile = builder.build()?;
//! let targets = TargetRequest::new([profile])?;
//! assert_eq!(targets.profiles().len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A dimension-specific method cannot be paired with another dimension's
//! behaviour vocabulary:
//!
//! ```compile_fail
//! # use tiler_compiler::target::*;
//! # use tiler_ir::schedule::NumericalPermission;
//! # let producer = TargetFactProducerIdentity::new("acme.profile.v1".to_owned(), 1).unwrap();
//! # let reference = TargetNormativeReferenceIdentity::new("acme.spec.v1".to_owned(), 1).unwrap();
//! # let source = TargetFactSource::external_guarantee(producer, reference);
//! # let mut builder =
//! #     TargetProfileBuilder::new(TargetProfileKey::new("acme.gpu.v1".to_owned()).unwrap());
//! builder.declare_input_subnormals(
//!     ScalarArithmetic::f32(),
//!     NumericalPermission::Forbidden,
//!     ScalarSupport::Exact,
//!     source,
//! );
//! ```
//!
//! Nor can an external producer assert that the compiler proved an exact
//! emulation:
//!
//! ```compile_fail
//! use tiler_compiler::target::ScalarSupport;
//! let _ = ScalarSupport::SupportedWithExactEmulation;
//! ```
//!
//! Producer and normative-reference identities cannot be silently swapped:
//!
//! ```compile_fail
//! use tiler_compiler::target::{
//!     TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
//! };
//! let producer = TargetFactProducerIdentity::new("acme.profile.v1".to_owned(), 1).unwrap();
//! let reference =
//!     TargetNormativeReferenceIdentity::new("acme.spec.v1".to_owned(), 1).unwrap();
//! let _ = TargetFactSource::external_guarantee(reference, producer);
//! ```
//!
//! Measurement authority fixes its phase and validity; callers cannot spell a
//! portable measured fact:
//!
//! ```compile_fail
//! use tiler_compiler::target::{
//!     MeasuredFactAuthority, TargetFactProducerIdentity, TargetFactSource,
//! };
//! let producer = TargetFactProducerIdentity::new("acme.probe.v1".to_owned(), 1).unwrap();
//! let _ = TargetFactSource::measured(
//!     producer,
//!     MeasuredFactAuthority::PortableProfile,
//!     [],
//! );
//! ```
//!
//! Compile-profile measurement provenance cannot be constructed as a tuple or
//! erased into the general source type:
//!
//! ```compile_fail
//! use tiler_compiler::target::TargetCompileProfileMeasurementSource;
//! let _ = TargetCompileProfileMeasurementSource;
//! ```
//!
//! ```compile_fail
//! use tiler_compiler::target::{
//!     TargetCompileProfileMeasurementSource, TargetFactSource,
//! };
//! fn erase(source: TargetCompileProfileMeasurementSource) -> TargetFactSource {
//!     source.into()
//! }
//! ```

// The two crate-private children of this cluster, declared here rather than at
// the crate root so the direction runs one way from a single visible place: a
// producer declares facts through this root, `feasibility` turns them into hard
// admit/reject predicates, and `honourability` owns the per-dimension numerical
// vocabulary those predicates quantify over. Cost lives in `component_cost` and
// stays outside the cluster: feasibility is not a cost.
//
// `pub(crate)` rather than private because a private child of a module is
// visible only within that module and its descendants, and these two are
// consumed across the compiler — at the crate root the same declarations were
// crate-visible for free. `pub(crate)` restores exactly that reach and no more:
// neither module is nameable outside this crate, so nothing here widens the
// reviewed public `target` facade.
pub(crate) mod accuracy;
pub(crate) mod feasibility;
pub(crate) mod honourability;

// The declaration vocabulary itself, split by cohesion and kept private to this
// module. Every path a consumer holds is one of the re-exports below, so a seam
// can move without moving anyone's `use`, and no submodule is nameable from
// outside this facade.
mod builder;
mod descriptor;
mod error;
mod evidence;
mod key;
mod profile;
mod request;
mod rows;
mod source;

/// One scalar-arithmetic policy subject with its complete semantic dtype.
///
/// The type, and the catalog validation behind [`ScalarArithmetic::new`], are
/// `tiler_ir::numerics::ScalarArithmeticSubject`. Only the siting changed: the
/// governed built-in scalar catalog the constructor consults lives in
/// `tiler-ir`, and the artifact record has to name the same subject this
/// compiler declares about, so a subject minted here and one read off a record
/// are one type rather than two that must be kept in agreement.
pub use tiler_ir::numerics::ScalarArithmeticSubject as ScalarArithmetic;
/// The registered value identity one arithmetic type names.
pub(crate) use tiler_ir::numerics::registered_arithmetic_value_type;

pub use accuracy::{ElementaryRealization, ElementaryRealizationError};
pub use builder::TargetProfileBuilder;
pub use error::TargetProfileBuildError;
pub use evidence::{
    TargetCompileProfileMeasurementContextReference, TargetCompileProfileMeasurementContexts,
    TargetCompilerBuildReference, TargetCompilerBuilds, TargetCompilerRoleReference,
    TargetExecutionEnvironmentReference, TargetFactAuthority, TargetFactValidityScope,
    TargetMeasurementContextReference, TargetMeasurementContexts, TargetNumericalEvidenceBasis,
    TargetNumericalRefusalEvidence, TargetProvenanceReference,
};
pub use key::{MAX_TARGET_PROFILE_KEY_BYTES, TargetProfileKey, TargetProfileKeyError};
pub use profile::TargetProfile;
pub use request::{MAX_TARGET_PROFILES_PER_REQUEST, TargetRequest, TargetRequestError};
pub use rows::{
    BackendArithmeticLicence, DTypeDispatchability, DTypeDispatchabilityResolution,
    DeviceAddressWidth, EvaluationOrderPreservation, EvaluationOrderResolution,
    IndexArithmeticSupport, ScalarSupport, SubgroupRealizationResolution, SubgroupSupport,
    SynchronizationSupport, TargetCostRowResolution, WorkgroupTreeWidthPolicy,
    WorkgroupTreeWidthPolicyResolution,
};
pub use source::{
    MAX_TARGET_COMPILATION_SELECTION_IDENTITY_BYTES, MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT,
    MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE, MAX_TARGET_PROVENANCE_TEXT_BYTES,
    MeasuredFactAuthority, TargetCompilationSelectionIdentity,
    TargetCompileProfileMeasurementContext, TargetCompileProfileMeasurementSource,
    TargetCompilerBuild, TargetCompilerRole, TargetCompilerRoleIdentity,
    TargetExecutionEnvironment, TargetExecutionEnvironmentBuilder, TargetFactProducerIdentity,
    TargetFactSource, TargetFactSourceError, TargetMeasurementContext,
    TargetNormativeReferenceIdentity,
};

// The crate-internal surface, re-exported so every `crate::target::X` path in
// this crate keeps resolving from the one facade rather than naming a seam.
pub(crate) use key::{GOVERNED_TARGET_PROFILE_KEY, TargetProfileIdentity};

// What the test module below reaches through `use super::*`. These are the
// private names it named before the split, imported here so the seams stay
// invisible to it and no assertion had to move with the code it covers.
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use tiler_ir::program::abi::{
    TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
};
#[cfg(test)]
use tiler_ir::schedule::{
    ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission, SubnormalMode,
    SynchronizationSubject,
};
#[cfg(test)]
use tiler_ir::semantic::{F32, ResolvedValueType};

#[cfg(test)]
use crate::target::descriptor::{
    COMPLETE_PROFILE_DESCRIPTOR_DOMAIN, ELEMENTARY_REALIZATION_DOMAIN, EVALUATION_ORDER_DOMAIN,
    SUBGROUP_REALIZATION_DOMAIN, SUBGROUP_WIDTH_QUERY_DOMAIN, WORKGROUP_TREE_WIDTH_POLICY_DOMAIN,
};
#[cfg(test)]
use crate::target::feasibility::{
    CapabilityAxis, DeclaredSubgroupRealization, DeclaredSynchronizationRealization,
    MAX_TARGET_PROFILE_DESCRIPTOR_BYTES, SynchronizationRealization,
};
#[cfg(test)]
use crate::target::honourability::{
    DimensionBehaviour, FactSourceProvenance, HonouringMeans, NumericalDimension,
    governed_profile_source,
};
#[cfg(test)]
use crate::target::rows::{DTypeDispatchabilityFact, ScalarHonourabilityDeclaration};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::feasibility::{
        AvailabilityPhase, AxisRequirement, FactAuthority, FactValidityScope, FeasibilityOutcome,
        FeasibilityProposal,
    };
    use crate::target::honourability::{
        CANONICAL_DIMENSIONS, CompilerBuildIdentity, CompilerBuildRole,
        ExecutionEnvironmentIdentity, MeasurementContext, NumericalRequirement,
        PostCompileMeasurementAuthority, ProvenanceIdentity,
    };
    use tiler_ir::numerics::{
        RelaxationRequirement, ScalarArithmeticSubjectError, ScalarArithmeticSubjectIdentity,
        registered_arithmetic_facts, registered_scalar_format,
    };
    use tiler_ir::schedule::{
        ApproximationEnvelope, ArithmeticType, FencedSpaces, MaterializationRounding,
        MemoryOrdering, SubgroupRealizationSubject, SubgroupTransfer, SubgroupWidth,
        SynchronizationKind, SynchronizationScope,
    };
    use tiler_ir::semantic::{
        CanonicalValue, TypeArguments, TypeKey, builtin_scalar_value_type_facts,
        builtin_scalar_value_types,
    };

    fn nominal(name: impl AsRef<str>) -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("test", name, 1).unwrap())
    }

    fn dispatch_fact(
        resolved_type: ResolvedValueType,
        verdict: DTypeDispatchability,
    ) -> DTypeDispatchabilityFact {
        DTypeDispatchabilityFact {
            resolved_type,
            verdict,
            source: governed_profile_source(),
        }
    }

    fn measured_capability_source() -> Arc<FactSourceProvenance> {
        Arc::new(FactSourceProvenance::post_compile_measured(
            PostCompileMeasurementAuthority::DeviceRuntime,
            ProvenanceIdentity::new("test.capability-producer.v1", 1),
            vec![MeasurementContext::new(
                vec![CompilerBuildIdentity::new(
                    CompilerBuildRole::RuntimeCompiler,
                    "test-compiler",
                    "1.0",
                    None,
                )],
                ExecutionEnvironmentIdentity::new(
                    "test-platform",
                    "1.0",
                    "build-1",
                    "test-architecture",
                    "test-hardware",
                ),
            )],
        ))
    }

    fn public_external_source(reference_revision: u32) -> TargetFactSource {
        TargetFactSource::external_guarantee(
            TargetFactProducerIdentity::new("test.external-profile-producer.v1".to_owned(), 1)
                .unwrap(),
            TargetNormativeReferenceIdentity::new(
                "test.external-profile-specification.v1".to_owned(),
                reference_revision,
            )
            .unwrap(),
        )
    }

    fn public_builder(key: &str) -> TargetProfileBuilder {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
        builder
            .declare_max_threads_per_grid_axis(65_535, source.clone())
            .unwrap();
        builder
            .declare_max_threads_per_workgroup(256, source.clone())
            .unwrap();
        builder
            .declare_max_buffer_bindings_per_entry(31, source.clone())
            .unwrap();
        builder
            .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
            .unwrap();
        builder
            .declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())
            .unwrap();
        builder.declare_device_memory(true, source.clone()).unwrap();
        builder
            .declare_local_memory_bytes(32_768, source.clone())
            .unwrap();
        builder
    }

    fn compile_profile_measurement_source(
        compiler_version: &str,
        platform_build: &str,
    ) -> TargetCompileProfileMeasurementSource {
        compile_profile_measurement_source_with(1, compiler_version, platform_build)
    }

    fn compile_profile_measurement_source_with(
        producer_revision: u32,
        compiler_version: &str,
        platform_build: &str,
    ) -> TargetCompileProfileMeasurementSource {
        compile_profile_measurement_source_with_selection(
            producer_revision,
            compiler_version,
            platform_build,
            b"test-selection.v1",
        )
    }

    fn compile_profile_measurement_source_with_selection(
        producer_revision: u32,
        compiler_version: &str,
        platform_build: &str,
        selection: &[u8],
    ) -> TargetCompileProfileMeasurementSource {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::CodeGenerator,
            "test-code-generator".to_owned(),
            compiler_version.to_owned(),
            Some("exact-build".to_owned()),
        )
        .unwrap();
        let environment = TargetExecutionEnvironment::builder()
            .platform("test-platform".to_owned())
            .platform_version("1.0".to_owned())
            .platform_build(platform_build.to_owned())
            .architecture("test-architecture".to_owned())
            .hardware("test-hardware".to_owned())
            .build()
            .unwrap();
        let context = TargetCompileProfileMeasurementContext::new(
            [compiler],
            environment,
            TargetCompilationSelectionIdentity::from_bytes(selection).unwrap(),
        )
        .unwrap();
        TargetCompileProfileMeasurementSource::new(
            TargetFactProducerIdentity::new(
                "test.compile-profile-measurement.v1".to_owned(),
                producer_revision,
            )
            .unwrap(),
            [context],
        )
        .unwrap()
    }

    #[test]
    fn caller_profile_keys_are_owned_and_validated() {
        let source = String::from("acme.family-a.v1");
        let key = TargetProfileKey::declared(source.clone()).unwrap();
        drop(source);
        assert_eq!(key.as_str(), "acme.family-a.v1");
        assert_eq!(
            TargetProfileKey::declared(String::new()),
            Err(TargetProfileKeyError::Empty)
        );
        assert_eq!(
            TargetProfileKey::declared("Acme family".to_owned()),
            Err(TargetProfileKeyError::InvalidByte {
                index: 0,
                value: b'A',
            })
        );
        assert_eq!(
            TargetProfileKey::declared("a".repeat(MAX_TARGET_PROFILE_KEY_BYTES + 1)),
            Err(TargetProfileKeyError::TooLong {
                actual: MAX_TARGET_PROFILE_KEY_BYTES + 1,
                max: MAX_TARGET_PROFILE_KEY_BYTES,
            })
        );
    }

    /// The governed identity `arithmetic` names, as the catalog spells it.
    fn governed_scalar(name: &str) -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("tiler", name, 1).unwrap())
    }

    /// Every arithmetic type constructs a subject over its own governed identity.
    ///
    /// All four, not BF16 alone: a route that admitted one named dtype would be
    /// the widened equality check under another name, and the point of deriving
    /// admissibility from the catalog is that no dtype is special-cased in it.
    #[test]
    fn every_arithmetic_type_constructs_a_subject_over_its_own_governed_identity() {
        for (arithmetic, name) in [
            (ArithmeticType::F16, "f16"),
            (ArithmeticType::Bf16, "bf16"),
            (ArithmeticType::F32, "f32"),
            (ArithmeticType::F64, "f64"),
        ] {
            let resolved_type = governed_scalar(name);
            let subject = ScalarArithmetic::new(arithmetic, resolved_type.clone())
                .unwrap_or_else(|error| panic!("{name} is a governed identity: {error}"));
            assert_eq!(subject.arithmetic(), arithmetic);
            assert_eq!(subject.resolved_type(), &resolved_type);
        }
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F32, F32::resolved_type()),
            Ok(ScalarArithmetic::f32()),
            "the governed F32 subject keeps the exact pair every existing profile names",
        );
    }

    /// Each way a pair can fail the catalog is refused, and none is a build error.
    ///
    /// The width and class cases are chosen so that exactly one field
    /// disagrees: `tiler::f16@1` shares `f32`'s `ieee-binary` class and states a
    /// different width, and `tiler::u32@1` states `f32`'s width and a different
    /// class. A rule reading only one of the two fields would admit one of them.
    #[test]
    fn a_pair_the_catalog_does_not_back_is_refused_for_a_stated_reason() {
        let refused = Err(ScalarArithmeticSubjectError::UnvalidatedScalarArithmetic);

        // An unregistered identity, against every arithmetic type: a `test`
        // namespace is not the governed catalog however the name is spelled.
        for arithmetic in ArithmeticType::ALL {
            for name in ["f16", "bf16", "f32", "f64", "u4"] {
                assert_eq!(
                    ScalarArithmetic::new(arithmetic, nominal(name)),
                    refused,
                    "test::{name}@1 is not a registered governed identity",
                );
            }
        }

        // A registered identity whose stated width disagrees with the
        // arithmetic type's.
        let f32_width = registered_scalar_format(
            &registered_arithmetic_facts(ArithmeticType::F32).expect("f32 is governed"),
        )
        .expect("the governed f32 row states a format")
        .1;
        let f16_width = registered_scalar_format(
            &builtin_scalar_value_type_facts(&governed_scalar("f16")).expect("f16 is governed"),
        )
        .expect("the governed f16 row states a format")
        .1;
        assert_eq!((f32_width, f16_width), (32, 16));
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F32, governed_scalar("f16")),
            refused,
        );
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F64, governed_scalar("f32")),
            refused,
        );

        // A registered identity of the arithmetic type's exact width whose
        // format class is another family's.
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F32, governed_scalar("u32")),
            refused,
        );
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F32, governed_scalar("decimal32")),
            refused,
        );
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::Bf16, governed_scalar("f16")),
            refused,
            "bf16 and f16 share a width and differ in class",
        );

        // A registered identity whose descriptor states no width at all.
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F16, governed_scalar("bool")),
            refused,
        );

        // An identity that is not a nominal scalar row.
        let complex = ResolvedValueType::parameterized(
            TypeKey::new("tiler", "complex", 1).unwrap(),
            TypeArguments::new([CanonicalValue::value_type(F32::resolved_type())]).unwrap(),
        )
        .unwrap();
        assert_eq!(ScalarArithmetic::new(ArithmeticType::F32, complex), refused);
    }

    /// A format is unique to one governed identity, over the whole catalog.
    ///
    /// This is the invariant that lets admissibility be decided on class and
    /// width: were two governed rows to share both, each would be constructible
    /// as the other's arithmetic subject. Nothing in `tiler-ir` promises that,
    /// so it is counted here rather than assumed, and a catalog row added with a
    /// colliding format fails this test instead of silently widening a subject.
    #[test]
    fn every_arithmetic_type_names_exactly_one_governed_format() {
        for arithmetic in ArithmeticType::ALL {
            let facts = registered_arithmetic_facts(arithmetic)
                .unwrap_or_else(|| panic!("{} is registered", arithmetic.canonical_type_key()));
            let format = registered_scalar_format(&facts)
                .unwrap_or_else(|| panic!("{} states a format", arithmetic.canonical_type_key()));
            let sharing: Vec<_> = builtin_scalar_value_types()
                .into_iter()
                .filter(|value| {
                    builtin_scalar_value_type_facts(value)
                        .is_some_and(|facts| registered_scalar_format(&facts) == Some(format))
                })
                .filter_map(|value| value.nominal_key().map(TypeKey::to_string))
                .collect();
            assert_eq!(
                sharing,
                vec![arithmetic.canonical_type_key().to_owned()],
                "{} shares its format class and width with another governed identity",
                arithmetic.canonical_type_key(),
            );
        }
    }

    /// Constructing a subject declares nothing about it.
    ///
    /// A profile carrying the complete governed F32 declaration says nothing
    /// about BF16, and the fail-closed clause applies to the subject coordinate
    /// exactly as it does to the dimension one. Every dimension is required in
    /// one proposal and the undeclared set is counted, so a resolution that
    /// answered some of them would not be mistaken for silence about all.
    #[test]
    fn a_constructed_subject_no_profile_declares_is_unknown_on_every_dimension() {
        let subject = ScalarArithmetic::new(ArithmeticType::Bf16, governed_scalar("bf16"))
            .expect("bf16 is a governed identity");
        let behaviour = |dimension| match dimension {
            NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals => {
                DimensionBehaviour::Subnormals(SubnormalMode::Preserve)
            }
            NumericalDimension::Contraction
            | NumericalDimension::Reassociation
            | NumericalDimension::Permutation
            | NumericalDimension::SignedZero
            | NumericalDimension::ReciprocalTransform => {
                DimensionBehaviour::Transform(NumericalPermission::Forbidden)
            }
            NumericalDimension::ApproximateIntrinsics => {
                DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden)
            }
            NumericalDimension::NanAssumptions | NumericalDimension::InfinityAssumptions => {
                DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption)
            }
            NumericalDimension::MaterializationRounding => {
                DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven)
            }
        };
        let proposal = FeasibilityProposal::new(
            "undeclared-bf16-subject",
            Vec::new(),
            CANONICAL_DIMENSIONS
                .iter()
                .map(|dimension| {
                    NumericalRequirement::new(
                        *dimension,
                        subject.arithmetic(),
                        subject.resolved_type().clone(),
                        behaviour(*dimension),
                    )
                })
                .collect(),
        )
        .unwrap();
        let profile = TargetProfileBuilder::governed().try_build().unwrap();
        let FeasibilityOutcome::Unknown(unknown) = profile
            .checked()
            .assess(&proposal, AvailabilityPhase::CompileProfile)
        else {
            panic!("a profile declaring only F32 rows answers nothing about BF16");
        };
        let undeclared: Vec<_> = unknown
            .dimensions()
            .iter()
            .map(|dimension| {
                assert_eq!(dimension.arithmetic(), ArithmeticType::Bf16);
                assert_eq!(dimension.resolved_type(), subject.resolved_type());
                dimension.dimension()
            })
            .collect();
        assert_eq!(undeclared, CANONICAL_DIMENSIONS);
    }

    #[test]
    fn scalar_declarations_reject_invalid_behaviour_relaxation_and_exact_emulation() {
        let subject = ScalarArithmetic::f32();
        let base = |dimension, behaviour, means| ScalarHonourabilityDeclaration {
            subject: subject.clone(),
            dimension,
            behaviour,
            means,
            source: governed_profile_source(),
        };
        assert_eq!(
            base(
                NumericalDimension::Contraction,
                DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
                HonouringMeans::SupportedExactly,
            )
            .validate(),
            Err(TargetProfileBuildError::InvalidDimensionBehaviour)
        );
        assert_eq!(
            base(
                NumericalDimension::InputSubnormals,
                DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
                HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
                    relaxation: RelaxationRequirement::new(
                        ScalarArithmeticSubjectIdentity::from_parts(
                            ArithmeticType::F64,
                            nominal("future-f64").canonical_encoding().as_bytes(),
                        )
                        .expect("a nominal identity is well formed"),
                        NumericalDimension::Contraction,
                        DimensionBehaviour::Transform(NumericalPermission::Permitted),
                    ),
                },
            )
            .validate(),
            Err(TargetProfileBuildError::InvalidRelaxation)
        );
        assert_eq!(
            base(
                NumericalDimension::InputSubnormals,
                DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
                HonouringMeans::SupportedWithExactEmulation,
            )
            .validate(),
            Err(TargetProfileBuildError::UnverifiedExactEmulation)
        );
    }

    #[test]
    fn scalar_duplicate_detection_compares_the_complete_subject() {
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.scalar-subject.v1".to_owned()).unwrap(),
        );
        let source = public_external_source(1);
        // Two *governed* subjects, because the relocated validator refuses a
        // subject over an unregistered type. Holding the arithmetic fixed while
        // varying the resolved type is no longer constructible at all — the
        // catalog admits exactly one value identity per arithmetic type — which
        // is a stronger guarantee than this test used to assert against a
        // hand-built pair.
        for subject in [
            ScalarArithmetic::f32(),
            ScalarArithmetic::new(ArithmeticType::F16, governed_scalar("f16"))
                .expect("the catalog registers f16 over tiler::f16@1"),
        ] {
            builder.scalar.push(ScalarHonourabilityDeclaration {
                subject,
                dimension: NumericalDimension::Contraction,
                behaviour: DimensionBehaviour::Transform(NumericalPermission::Forbidden),
                means: HonouringMeans::SupportedExactly,
                source: Arc::clone(&source.0),
            });
        }
        assert_eq!(builder.validate_declarations(), Ok(()));
    }

    #[test]
    fn malformed_structured_producer_attribution_is_rejected() {
        let mut builder = TargetProfileBuilder::governed();
        builder.quantitative[0].source = Arc::new(FactSourceProvenance::post_compile_measured(
            PostCompileMeasurementAuthority::DeviceRuntime,
            ProvenanceIdentity::new("test.empty-measurement.v1", 1),
            Vec::new(),
        ));
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::InvalidProducerClaim)
        );

        let mut builder = TargetProfileBuilder::governed();
        builder.scalar[0].source = Arc::new(FactSourceProvenance::post_compile_measured(
            PostCompileMeasurementAuthority::DeviceRuntime,
            ProvenanceIdentity::new("test.empty-scalar-measurement.v1", 1),
            Vec::new(),
        ));
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::InvalidProducerClaim)
        );

        let mut builder = TargetProfileBuilder::governed();
        builder.dispatchability = vec![DTypeDispatchabilityFact {
            resolved_type: F32::resolved_type(),
            verdict: DTypeDispatchability::Dispatchable,
            source: Arc::new(FactSourceProvenance::post_compile_measured(
                PostCompileMeasurementAuthority::DeviceRuntime,
                ProvenanceIdentity::new("test.empty-dispatch-measurement.v1", 1),
                Vec::new(),
            )),
        }];
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::InvalidProducerClaim)
        );
    }

    #[test]
    fn external_guarantees_are_not_compiler_governed_or_measurements() {
        let external = public_external_source(1);
        assert_eq!(external.0.authority(), FactAuthority::ExternalProfile);
        assert!(matches!(
            external.0.basis(),
            crate::target::honourability::FactEvidenceBasis::ExternalGuarantee { .. }
        ));
        assert_ne!(
            external.0.canonical_bytes(),
            governed_profile_source().canonical_bytes()
        );

        let first = public_builder("test.external-a.v1").build().unwrap();
        let mut second = TargetProfileBuilder::new(
            TargetProfileKey::new("test.external-a.v1".to_owned()).unwrap(),
        );
        let revised_source = public_external_source(2);
        second
            .declare_max_threads_per_grid_axis(65_535, revised_source.clone())
            .unwrap();
        second
            .declare_max_threads_per_workgroup(256, revised_source.clone())
            .unwrap();
        second
            .declare_max_buffer_bindings_per_entry(31, revised_source.clone())
            .unwrap();
        second
            .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, revised_source.clone())
            .unwrap();
        second
            .declare_device_address_width(DeviceAddressWidth::Bits64, revised_source.clone())
            .unwrap();
        second
            .declare_device_memory(true, revised_source.clone())
            .unwrap();
        second
            .declare_local_memory_bytes(32_768, revised_source.clone())
            .unwrap();
        let second = second.build().unwrap();
        assert_ne!(
            first.canonical_descriptor(),
            second.canonical_descriptor(),
            "the normative reference revision is identity-bearing"
        );
    }

    #[test]
    fn measured_authorities_derive_the_only_valid_phase_and_scope() {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-compiler".to_owned(),
            "1.0".to_owned(),
            None,
        )
        .unwrap();
        let environment = TargetExecutionEnvironment::builder()
            .platform("test-platform".to_owned())
            .platform_version("1.0".to_owned())
            .platform_build("build-1".to_owned())
            .architecture("test-architecture".to_owned())
            .hardware("test-hardware".to_owned())
            .build()
            .unwrap();
        let context = TargetMeasurementContext::new([compiler], environment).unwrap();
        for (index, (authority, phase, internal_authority, validity)) in [
            (
                MeasuredFactAuthority::ArtifactEvidence,
                AvailabilityPhase::ArtifactEvidence,
                FactAuthority::ArtifactEvidence,
                FactValidityScope::PreparedArtifact,
            ),
            (
                MeasuredFactAuthority::DeviceRuntime,
                AvailabilityPhase::LiveDevicePreflight,
                FactAuthority::DeviceRuntime,
                FactValidityScope::DeviceInstance,
            ),
            (
                MeasuredFactAuthority::PreparedKernel,
                AvailabilityPhase::PreparedKernelPreflight,
                FactAuthority::PreparedKernel,
                FactValidityScope::PreparedArtifact,
            ),
            (
                MeasuredFactAuthority::LaunchInstance,
                AvailabilityPhase::LaunchPreflight,
                FactAuthority::LaunchInstance,
                FactValidityScope::LaunchInstance,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let producer =
                TargetFactProducerIdentity::new(format!("test.measurement-producer-{index}.v1"), 1)
                    .unwrap();
            let source =
                TargetFactSource::measured(producer, authority, [context.clone()]).unwrap();
            assert_eq!(source.0.phase(), phase);
            assert_eq!(source.0.authority(), internal_authority);
            assert_eq!(source.0.validity(), validity);
        }
    }

    #[test]
    fn compiler_profile_measurement_source_fixes_empirical_authority_and_scope() {
        let source = compile_profile_measurement_source("1.0", "build-1");
        assert_eq!(source.0.phase(), AvailabilityPhase::CompileProfile);
        assert_eq!(source.0.authority(), FactAuthority::MeasuredProfile);
        assert_eq!(source.0.validity(), FactValidityScope::MeasuredEnvironment);
        assert!(matches!(
            source.0.basis(),
            crate::target::honourability::FactEvidenceBasis::CompileProfileMeasurement { contexts }
                if contexts.len() == 1
                    && contexts[0].compiler_builds()[0].version() == "1.0"
                    && contexts[0].environment().platform_build() == "build-1"
                    && contexts[0].compilation_selection().as_bytes() == b"test-selection.v1"
        ));
    }

    #[test]
    fn compiler_profile_measurement_source_reaches_every_profile_fact_family() {
        let source = compile_profile_measurement_source("1.0", "build-1");
        let subject = ScalarArithmetic::f32();
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.measured-all-families.v1".to_owned()).unwrap(),
        );
        builder
            .declare_measured_max_threads_per_grid_axis(65_535, source.clone())
            .unwrap();
        builder
            .declare_measured_max_threads_per_workgroup(256, source.clone())
            .unwrap();
        builder
            .declare_measured_max_buffer_bindings_per_entry(31, source.clone())
            .unwrap();
        builder
            .declare_measured_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
            .unwrap();
        builder
            .declare_measured_device_address_width(DeviceAddressWidth::Bits64, source.clone())
            .unwrap();
        builder
            .declare_measured_device_memory(true, source.clone())
            .unwrap();
        builder
            .declare_measured_local_memory_bytes(32_768, source.clone())
            .unwrap();
        builder
            .declare_measured_input_subnormal_behaviour(
                subject.clone(),
                SubnormalMode::Preserve,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_result_subnormal_behaviour(
                subject.clone(),
                SubnormalMode::Preserve,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_contraction(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_reassociation(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_permutation(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_signed_zero(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_reciprocal_transform(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_approximate_intrinsics(
                subject.clone(),
                tiler_ir::schedule::ApproximationEnvelope::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_nan_assumptions(
                subject.clone(),
                ExceptionalValueAssumption::MakeNoAssumption,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_infinity_assumptions(
                subject.clone(),
                ExceptionalValueAssumption::MakeNoAssumption,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_materialization_rounding(
                subject,
                tiler_ir::schedule::MaterializationRounding::NearestTiesToEven,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_dtype_dispatchability(
                F32::resolved_type(),
                DTypeDispatchability::Dispatchable,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_elementary_realization(ElementaryRealization::measured(
                &verified_silu_contract(),
                discharging_evidence(
                    "measured-family bound half",
                    b"fixture:measured-family-bound-v1",
                ),
                discharging_evidence(
                    "measured-family exceptional half",
                    b"fixture:measured-family-exceptional-v1",
                ),
                &source,
            ))
            .unwrap();
        builder
            .declare_measured_subgroup_realization(
                subgroup_subject(32, ArithmeticType::F32),
                SubgroupSupport::Realized,
                source.clone(),
            )
            .unwrap();
        declare_query_for(&mut builder, SubgroupSupport::Realized);

        assert_eq!(builder.quantitative.len(), 7);
        assert_eq!(builder.scalar.len(), 15);
        assert_eq!(builder.dispatchability.len(), 1);
        assert_eq!(builder.elementary.len(), 1);
        assert_eq!(builder.subgroup.len(), 1);
        for provenance in builder
            .quantitative
            .iter()
            .map(|declaration| declaration.source.as_ref())
            .chain(
                builder
                    .scalar
                    .iter()
                    .map(|declaration| declaration.source.as_ref()),
            )
            .chain(
                builder
                    .dispatchability
                    .iter()
                    .map(|declaration| declaration.source.as_ref()),
            )
            .chain(builder.elementary.iter().map(ElementaryRealization::source))
            .chain(
                builder
                    .subgroup
                    .iter()
                    .map(DeclaredSubgroupRealization::source_ref),
            )
        {
            assert_eq!(provenance.phase(), AvailabilityPhase::CompileProfile);
            assert_eq!(provenance.authority(), FactAuthority::MeasuredProfile);
            assert_eq!(
                provenance.validity(),
                FactValidityScope::MeasuredEnvironment
            );
        }
        builder.build().unwrap();
    }

    #[test]
    fn measured_profile_declarations_reject_conflicts_without_partial_insertion() {
        let source = || compile_profile_measurement_source("1.0", "build-1");
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.measured-conflicts.v1".to_owned()).unwrap(),
        );
        builder
            .declare_measured_max_threads_per_workgroup(256, source())
            .unwrap();
        let quantitative = builder.quantitative.clone();
        assert_eq!(
            builder.declare_measured_max_threads_per_workgroup(128, source()),
            Err(TargetProfileBuildError::DuplicateQuantitativeCapability {
                axis: "threads-per-workgroup",
                phase: AvailabilityPhase::CompileProfile,
            })
        );
        assert_eq!(builder.quantitative, quantitative);

        builder
            .declare_measured_contraction(
                ScalarArithmetic::f32(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source(),
            )
            .unwrap();
        let scalar = builder.scalar.clone();
        assert_eq!(
            builder.declare_measured_contraction(
                ScalarArithmetic::f32(),
                NumericalPermission::Forbidden,
                ScalarSupport::Unsupported,
                source(),
            ),
            Err(TargetProfileBuildError::DuplicateScalarDeclaration)
        );
        assert_eq!(builder.scalar, scalar);

        builder
            .declare_measured_dtype_dispatchability(
                F32::resolved_type(),
                DTypeDispatchability::Dispatchable,
                source(),
            )
            .unwrap();
        let dispatchability = builder.dispatchability.clone();
        assert_eq!(
            builder.declare_measured_dtype_dispatchability(
                F32::resolved_type(),
                DTypeDispatchability::Unsupported,
                source(),
            ),
            Err(TargetProfileBuildError::DuplicateDispatchability)
        );
        assert_eq!(builder.dispatchability, dispatchability);
    }

    #[test]
    fn quantitative_facts_and_queries_reject_overlap_atomically_in_both_orders() {
        let query = || {
            TargetPropertyQuery::new(
                TargetPropertyKey::new("test.prepared-entry.workgroup-limit.v1").unwrap(),
                AvailabilityPhase::PreparedKernelPreflight,
                TargetPropertyProviderIdentity::new("test", "prepared-entry", 1).unwrap(),
            )
            .unwrap()
        };

        let mut fact_first = TargetProfileBuilder::new(
            TargetProfileKey::new("test.fact-first.v1".to_owned()).unwrap(),
        );
        fact_first
            .declare_max_threads_per_workgroup(256, public_external_source(1))
            .unwrap();
        assert_eq!(
            fact_first.declare_max_threads_per_workgroup_query(query()),
            Err(
                TargetProfileBuildError::ConflictingQuantitativeFactAndQuery {
                    axis: "threads-per-workgroup",
                }
            )
        );
        assert!(fact_first.queries.is_empty());

        let mut query_first = TargetProfileBuilder::new(
            TargetProfileKey::new("test.query-first.v1".to_owned()).unwrap(),
        );
        query_first
            .declare_max_threads_per_workgroup_query(query())
            .unwrap();
        assert_eq!(
            query_first.declare_max_threads_per_workgroup(256, public_external_source(1)),
            Err(
                TargetProfileBuildError::ConflictingQuantitativeFactAndQuery {
                    axis: "threads-per-workgroup",
                }
            )
        );
        assert!(query_first.quantitative.is_empty());
    }

    #[test]
    fn measured_profile_identity_binds_source_and_fact_values() {
        let descriptor =
            |producer_revision, compiler_version, platform_build, threads, verdict, support| {
                let source = compile_profile_measurement_source_with(
                    producer_revision,
                    compiler_version,
                    platform_build,
                );
                let mut builder = TargetProfileBuilder::new(
                    TargetProfileKey::new("test.measured-identity.v1".to_owned()).unwrap(),
                );
                builder
                    .declare_measured_max_threads_per_workgroup(threads, source.clone())
                    .unwrap();
                builder
                    .declare_measured_contraction(
                        ScalarArithmetic::f32(),
                        NumericalPermission::Forbidden,
                        support,
                        source.clone(),
                    )
                    .unwrap();
                builder
                    .declare_measured_dtype_dispatchability(F32::resolved_type(), verdict, source)
                    .unwrap();
                builder.build().unwrap().canonical_descriptor().to_vec()
            };
        let baseline = descriptor(
            1,
            "1.0",
            "build-1",
            256,
            DTypeDispatchability::Dispatchable,
            ScalarSupport::Exact,
        );
        for changed in [
            descriptor(
                2,
                "1.0",
                "build-1",
                256,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "2.0",
                "build-1",
                256,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "1.0",
                "build-2",
                256,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "1.0",
                "build-1",
                128,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "1.0",
                "build-1",
                256,
                DTypeDispatchability::Unsupported,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "1.0",
                "build-1",
                256,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Unsupported,
            ),
        ] {
            assert_ne!(baseline, changed);
        }
    }

    #[test]
    fn measured_scalar_subnormal_declarations_build_independent_exclusive_tables() {
        let behaviours = [
            SubnormalMode::Preserve,
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
        ];
        for delivered in behaviours {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.measured-subnormal-table.v1".to_owned()).unwrap(),
            );
            builder
                .declare_measured_input_subnormal_behaviour(
                    ScalarArithmetic::f32(),
                    delivered,
                    compile_profile_measurement_source("1.0", "build-1"),
                )
                .unwrap();
            builder
                .declare_measured_result_subnormal_behaviour(
                    ScalarArithmetic::f32(),
                    delivered,
                    compile_profile_measurement_source("1.0", "build-1"),
                )
                .unwrap();
            assert_eq!(builder.scalar.len(), 6);
            for dimension in [
                NumericalDimension::InputSubnormals,
                NumericalDimension::ResultSubnormals,
            ] {
                for behaviour in behaviours {
                    let row = builder
                        .scalar
                        .iter()
                        .find(|row| {
                            row.dimension == dimension
                                && row.behaviour == DimensionBehaviour::Subnormals(behaviour)
                        })
                        .expect("every destination row is explicit");
                    assert_eq!(
                        row.means,
                        if behaviour == delivered {
                            HonouringMeans::SupportedExactly
                        } else {
                            HonouringMeans::Unsupported
                        }
                    );
                    assert_eq!(row.source.phase(), AvailabilityPhase::CompileProfile);
                    assert_eq!(row.source.authority(), FactAuthority::MeasuredProfile);
                    assert_eq!(
                        row.source.validity(),
                        FactValidityScope::MeasuredEnvironment
                    );
                }
            }
        }
    }

    #[test]
    fn measured_scalar_subnormal_dimension_rejects_cross_phase_rows_atomically() {
        let subject = ScalarArithmetic::f32();
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.measured-subnormal-conflict.v1".to_owned()).unwrap(),
        );
        builder
            .declare_result_subnormals(
                subject.clone(),
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::AlwaysPositive,
                },
                ScalarSupport::Unsupported,
                TargetFactSource(measured_capability_source()),
            )
            .unwrap();
        let before = builder.scalar.clone();
        assert_eq!(
            builder.declare_measured_result_subnormal_behaviour(
                subject.clone(),
                SubnormalMode::Preserve,
                compile_profile_measurement_source("1.0", "build-1"),
            ),
            Err(TargetProfileBuildError::ConflictingSubnormalDeclaration {
                subject: Box::new(subject),
                dimension: "numerics.result-subnormals",
                phase: AvailabilityPhase::LiveDevicePreflight,
            })
        );
        assert_eq!(
            builder.scalar, before,
            "refusal must insert no partial table"
        );
    }

    #[test]
    fn measured_subnormal_table_identity_binds_behaviour_build_and_environment() {
        let descriptor = |delivered, compiler_version, platform_build| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.measured-subnormal-identity.v1".to_owned()).unwrap(),
            );
            builder
                .declare_measured_input_subnormal_behaviour(
                    ScalarArithmetic::f32(),
                    delivered,
                    compile_profile_measurement_source(compiler_version, platform_build),
                )
                .unwrap();
            builder
                .declare_measured_result_subnormal_behaviour(
                    ScalarArithmetic::f32(),
                    delivered,
                    compile_profile_measurement_source(compiler_version, platform_build),
                )
                .unwrap();
            builder.build().unwrap().canonical_descriptor().to_vec()
        };
        let baseline = descriptor(SubnormalMode::Preserve, "1.0", "build-1");
        assert_ne!(
            baseline,
            descriptor(
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::AlwaysPositive,
                },
                "1.0",
                "build-1",
            )
        );
        assert_ne!(
            baseline,
            descriptor(SubnormalMode::Preserve, "2.0", "build-1")
        );
        assert_ne!(
            baseline,
            descriptor(SubnormalMode::Preserve, "1.0", "build-2")
        );
    }

    #[test]
    fn public_provenance_bounds_stop_after_the_first_excess_item() {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-compiler".to_owned(),
            "1.0".to_owned(),
            None,
        )
        .unwrap();
        let environment = || {
            TargetExecutionEnvironment::builder()
                .platform("test-platform".to_owned())
                .platform_version("1.0".to_owned())
                .platform_build("build-1".to_owned())
                .architecture("test-architecture".to_owned())
                .hardware("test-hardware".to_owned())
                .build()
                .unwrap()
        };
        let compiler_items = std::cell::Cell::new(0);
        let compiler_stream = std::iter::repeat_with(|| {
            compiler_items.set(compiler_items.get() + 1);
            compiler.clone()
        });
        assert_eq!(
            TargetMeasurementContext::new(compiler_stream, environment()),
            Err(TargetFactSourceError::TooManyCompilerBuilds {
                actual: MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT + 1,
                max: MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT,
            })
        );
        assert_eq!(
            compiler_items.get(),
            MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT + 1
        );

        let context = TargetMeasurementContext::new([compiler], environment()).unwrap();
        let context_items = std::cell::Cell::new(0);
        let context_stream = std::iter::repeat_with(|| {
            context_items.set(context_items.get() + 1);
            context.clone()
        });
        assert_eq!(
            TargetFactSource::measured(
                TargetFactProducerIdentity::new("test.measurement-bound.v1".to_owned(), 1).unwrap(),
                MeasuredFactAuthority::DeviceRuntime,
                context_stream,
            ),
            Err(TargetFactSourceError::TooManyMeasurementContexts {
                actual: MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE + 1,
                max: MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE,
            })
        );
        assert_eq!(
            context_items.get(),
            MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE + 1
        );
    }

    #[test]
    fn public_provenance_sets_reject_empty_and_duplicate_members() {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-compiler".to_owned(),
            "1.0".to_owned(),
            None,
        )
        .unwrap();
        let environment = || {
            TargetExecutionEnvironment::builder()
                .platform("test-platform".to_owned())
                .platform_version("1.0".to_owned())
                .platform_build("build-1".to_owned())
                .architecture("test-architecture".to_owned())
                .hardware("test-hardware".to_owned())
                .build()
                .unwrap()
        };
        assert_eq!(
            TargetMeasurementContext::new(std::iter::empty(), environment()),
            Err(TargetFactSourceError::EmptyCompilerBuildSet)
        );
        assert_eq!(
            TargetMeasurementContext::new([compiler.clone(), compiler.clone()], environment()),
            Err(TargetFactSourceError::DuplicateCompilerBuild)
        );

        let context = TargetMeasurementContext::new([compiler], environment()).unwrap();
        let producer =
            || TargetFactProducerIdentity::new("test.measurement-set.v1".to_owned(), 1).unwrap();
        assert_eq!(
            TargetFactSource::measured(
                producer(),
                MeasuredFactAuthority::DeviceRuntime,
                std::iter::empty(),
            ),
            Err(TargetFactSourceError::EmptyMeasurementContextSet)
        );
        assert_eq!(
            TargetFactSource::measured(
                producer(),
                MeasuredFactAuthority::DeviceRuntime,
                [context.clone(), context],
            ),
            Err(TargetFactSourceError::DuplicateMeasurementContext)
        );
        assert_eq!(
            TargetCompileProfileMeasurementSource::new(producer(), std::iter::empty()),
            Err(TargetFactSourceError::EmptyMeasurementContextSet)
        );
        let context = TargetCompileProfileMeasurementContext::new(
            [TargetCompilerBuild::new(
                TargetCompilerRole::RuntimeCompiler,
                "test-compiler".to_owned(),
                "1.0".to_owned(),
                None,
            )
            .unwrap()],
            environment(),
            TargetCompilationSelectionIdentity::from_bytes(b"test-selection.v1").unwrap(),
        )
        .unwrap();
        assert_eq!(
            TargetCompileProfileMeasurementSource::new(producer(), [context.clone(), context],),
            Err(TargetFactSourceError::DuplicateMeasurementContext)
        );
    }

    #[test]
    fn public_provenance_errors_name_the_exact_field_and_bound() {
        assert_eq!(
            TargetFactProducerIdentity::new("Bad".to_owned(), 1),
            Err(TargetFactSourceError::InvalidFieldByte {
                field: "producer.key",
                index: 0,
                value: b'B',
            })
        );
        assert_eq!(
            TargetNormativeReferenceIdentity::new("test.reference.v1".to_owned(), 0),
            Err(TargetFactSourceError::ZeroRevision {
                field: "normative-reference.revision",
            })
        );
        assert_eq!(
            TargetCompilerRoleIdentity::new("Bad".to_owned(), 1),
            Err(TargetFactSourceError::InvalidFieldByte {
                field: "compiler-role.key",
                index: 0,
                value: b'B',
            })
        );
        assert_eq!(
            TargetCompilerRoleIdentity::new("test.compiler-role.v1".to_owned(), 0),
            Err(TargetFactSourceError::ZeroRevision {
                field: "compiler-role.revision",
            })
        );
        assert_eq!(
            TargetCompilerBuild::new(
                TargetCompilerRole::RuntimeCompiler,
                "x".repeat(MAX_TARGET_PROVENANCE_TEXT_BYTES + 1),
                "1".to_owned(),
                None,
            ),
            Err(TargetFactSourceError::FieldTooLong {
                field: "compiler-build.implementation",
                actual: MAX_TARGET_PROVENANCE_TEXT_BYTES + 1,
                max: MAX_TARGET_PROVENANCE_TEXT_BYTES,
            })
        );
        assert_eq!(
            TargetCompilerBuild::new(
                TargetCompilerRole::RuntimeCompiler,
                "test-runtime".to_owned(),
                "version 1 ".to_owned(),
                None,
            ),
            Err(TargetFactSourceError::InvalidFieldByte {
                field: "compiler-build.version",
                index: 9,
                value: b' ',
            })
        );
        assert_eq!(
            TargetExecutionEnvironment::builder().build(),
            Err(TargetFactSourceError::MissingField {
                field: "environment.platform",
            })
        );
    }

    #[test]
    fn public_declarations_reject_duplicates_atomically_before_insertion() {
        let mut builder = public_builder("test.atomic.v1");
        let source = public_external_source(1);
        let quantitative_len = builder.quantitative.len();
        assert_eq!(
            builder.declare_index_arithmetic(IndexArithmeticSupport::Unsupported, source.clone(),),
            Err(TargetProfileBuildError::DuplicateQuantitativeCapability {
                axis: "index-arithmetic-u64",
                phase: AvailabilityPhase::CompileProfile,
            })
        );
        assert_eq!(builder.quantitative.len(), quantitative_len);

        builder
            .declare_input_subnormals(
                ScalarArithmetic::f32(),
                SubnormalMode::Preserve,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        let scalar_len = builder.scalar.len();
        assert_eq!(
            builder.declare_input_subnormals(
                ScalarArithmetic::f32(),
                SubnormalMode::Preserve,
                ScalarSupport::Unsupported,
                source.clone(),
            ),
            Err(TargetProfileBuildError::DuplicateScalarDeclaration)
        );
        assert_eq!(builder.scalar.len(), scalar_len);

        let f32 = F32::resolved_type();
        builder
            .declare_dtype_dispatchability(
                f32.clone(),
                DTypeDispatchability::Dispatchable,
                source.clone(),
            )
            .unwrap();
        let dispatch_len = builder.dispatchability.len();
        assert_eq!(
            builder.declare_dtype_dispatchability(f32, DTypeDispatchability::Unsupported, source,),
            Err(TargetProfileBuildError::DuplicateDispatchability)
        );
        assert_eq!(builder.dispatchability.len(), dispatch_len);
        builder
            .build()
            .expect("every retained declaration is valid");
    }

    #[test]
    fn declaration_order_does_not_change_the_canonical_profile() {
        let key = TargetProfileKey::new("test.canonical-order.v1".to_owned()).unwrap();
        let source = public_external_source(1);
        let first_type = nominal("canonical-a");
        let second_type = nominal("canonical-b");
        let mut forward = TargetProfileBuilder::new(key.clone());
        forward
            .declare_max_threads_per_grid_axis(64, source.clone())
            .unwrap();
        forward
            .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
            .unwrap();
        forward
            .declare_dtype_dispatchability(
                first_type.clone(),
                DTypeDispatchability::Dispatchable,
                source.clone(),
            )
            .unwrap();
        forward
            .declare_dtype_dispatchability(
                second_type.clone(),
                DTypeDispatchability::Unsupported,
                source.clone(),
            )
            .unwrap();
        let mut reverse = TargetProfileBuilder::new(key);
        reverse
            .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
            .unwrap();
        reverse
            .declare_max_threads_per_grid_axis(64, source.clone())
            .unwrap();
        reverse
            .declare_dtype_dispatchability(
                second_type,
                DTypeDispatchability::Unsupported,
                source.clone(),
            )
            .unwrap();
        reverse
            .declare_dtype_dispatchability(first_type, DTypeDispatchability::Dispatchable, source)
            .unwrap();
        let forward = forward.build().unwrap();
        let reverse = reverse.build().unwrap();
        assert_eq!(forward, reverse);
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        assert_eq!(
            forward.request_subject_bytes(),
            reverse.request_subject_bytes()
        );
    }

    fn atomic_realization_subject(kind: SynchronizationKind) -> SynchronizationSubject {
        SynchronizationSubject {
            kind,
            execution_scope: SynchronizationScope::Workgroup,
            visibility_scope: SynchronizationScope::Workgroup,
            fenced_spaces: FencedSpaces {
                workgroup: true,
                device: false,
            },
            ordering: MemoryOrdering::AcquireRelease,
        }
    }

    fn atomic_realization_neighbour() -> SynchronizationSubject {
        SynchronizationSubject {
            fenced_spaces: FencedSpaces {
                workgroup: true,
                device: true,
            },
            ..atomic_realization_subject(SynchronizationKind::ControlBarrier)
        }
    }

    fn declare_atomic_pair(
        key: &str,
        first: &(
            SynchronizationSubject,
            SynchronizationSupport,
            TargetFactSource,
        ),
        second: &(
            SynchronizationSubject,
            SynchronizationSupport,
            TargetFactSource,
        ),
    ) -> TargetProfile {
        let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
        builder
            .declare_synchronization_realization(first.0, first.1, &first.2)
            .unwrap();
        builder
            .declare_synchronization_realization(second.0, second.1, &second.2)
            .unwrap();
        builder.build().unwrap()
    }

    /// Insertion order is not identity for the atomic realization family.
    ///
    /// Two profiles that declare the same two rows in opposite order share one
    /// complete descriptor and one checked descriptor, and the stored
    /// population is uniqueness-key order — `(subject, phase)` — not
    /// declaration order.
    #[test]
    fn atomic_realization_insertion_order_is_not_identity() {
        let source = public_external_source(1);
        let control = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let collective = atomic_realization_subject(SynchronizationKind::Collective);
        assert!(control < collective, "the fixture relies on kind order");
        let forward = declare_atomic_pair(
            "test.atomic-order.v1",
            &(control, SynchronizationSupport::Realized, source.clone()),
            &(
                collective,
                SynchronizationSupport::Unrealizable,
                source.clone(),
            ),
        );
        let reverse = declare_atomic_pair(
            "test.atomic-order.v1",
            &(
                collective,
                SynchronizationSupport::Unrealizable,
                source.clone(),
            ),
            &(control, SynchronizationSupport::Realized, source),
        );
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        assert_eq!(
            forward.checked().canonical_descriptor(),
            reverse.checked().canonical_descriptor()
        );
        for profile in [&forward, &reverse] {
            let subjects: Vec<_> = profile
                .checked()
                .synchronization()
                .iter()
                .map(crate::target::feasibility::SynchronizationRealizationFact::subject)
                .collect();
            assert_eq!(subjects, [control, collective]);
        }
    }

    #[test]
    fn an_exact_duplicate_atomic_realization_is_refused_before_insertion() {
        let source = public_external_source(1);
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.atomic-duplicate.v1".to_owned()).unwrap(),
        );
        builder
            .declare_synchronization_realization(subject, SynchronizationSupport::Realized, &source)
            .unwrap();
        let len = builder.synchronization.len();
        assert_eq!(
            builder.declare_synchronization_realization(
                subject,
                SynchronizationSupport::Realized,
                &source,
            ),
            Err(TargetProfileBuildError::DuplicateSynchronizationRealization)
        );
        assert_eq!(builder.synchronization.len(), len);
    }

    #[test]
    fn a_contradictory_atomic_realization_verdict_is_refused_before_insertion() {
        let source = public_external_source(1);
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        for (first, second) in [
            (
                SynchronizationSupport::Realized,
                SynchronizationSupport::Unrealizable,
            ),
            (
                SynchronizationSupport::Unrealizable,
                SynchronizationSupport::Realized,
            ),
        ] {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.atomic-contradiction.v1".to_owned()).unwrap(),
            );
            builder
                .declare_synchronization_realization(subject, first, &source)
                .unwrap();
            let len = builder.synchronization.len();
            assert_eq!(
                builder.declare_synchronization_realization(subject, second, &source),
                Err(TargetProfileBuildError::DuplicateSynchronizationRealization),
                "sort order must not choose a winner between {first:?} then {second:?}"
            );
            assert_eq!(builder.synchronization.len(), len);
        }
    }

    /// Freeze-time validation refuses both cases even when insert-time is
    /// bypassed, so a mutated draft cannot encode a contradiction.
    #[test]
    fn freeze_refuses_duplicate_and_contradictory_atomic_realizations_independently() {
        let source = public_external_source(1).provenance();
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let realized = DeclaredSynchronizationRealization::new(
            subject,
            SynchronizationRealization::Realized,
            Arc::clone(&source),
        );
        let unrealizable = DeclaredSynchronizationRealization::new(
            subject,
            SynchronizationRealization::Unrealizable,
            source,
        );
        for rows in [
            vec![realized.clone(), realized.clone()],
            vec![realized, unrealizable],
        ] {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.atomic-freeze.v1".to_owned()).unwrap(),
            );
            builder.synchronization = rows;
            assert_eq!(
                builder.try_build(),
                Err(TargetProfileBuildError::DuplicateSynchronizationRealization)
            );
        }
    }

    /// Distinct phases of one subject coexist, and declaring the later phase
    /// first does not move either descriptor.
    #[test]
    fn atomic_realization_phase_is_part_of_the_uniqueness_key_and_not_insertion_order() {
        let compile = public_external_source(1);
        let later = device_runtime_source();
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let forward = declare_atomic_pair(
            "test.atomic-phase.v1",
            &(subject, SynchronizationSupport::Realized, compile.clone()),
            &(subject, SynchronizationSupport::Unrealizable, later.clone()),
        );
        let reverse = declare_atomic_pair(
            "test.atomic-phase.v1",
            &(subject, SynchronizationSupport::Unrealizable, later),
            &(subject, SynchronizationSupport::Realized, compile),
        );
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        assert_eq!(
            forward.checked().canonical_descriptor(),
            reverse.checked().canonical_descriptor()
        );
        for profile in [&forward, &reverse] {
            let phases: Vec<_> = profile
                .checked()
                .synchronization()
                .iter()
                .map(crate::target::feasibility::SynchronizationRealizationFact::phase)
                .collect();
            assert_eq!(
                phases,
                [
                    AvailabilityPhase::CompileProfile,
                    AvailabilityPhase::LiveDevicePreflight
                ]
            );
        }
    }

    /// Source is identity-bearing in the complete declaration and not a
    /// uniqueness-key component: two sources at one `(subject, phase)` refuse,
    /// and two profiles that differ only in source revision do not share a
    /// complete descriptor.
    #[test]
    fn atomic_realization_source_participates_in_complete_identity_independently() {
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let first = public_external_source(1);
        let second = public_external_source(2);
        let mut colliding = TargetProfileBuilder::new(
            TargetProfileKey::new("test.atomic-source.v1".to_owned()).unwrap(),
        );
        colliding
            .declare_synchronization_realization(subject, SynchronizationSupport::Realized, &first)
            .unwrap();
        assert_eq!(
            colliding.declare_synchronization_realization(
                subject,
                SynchronizationSupport::Realized,
                &second,
            ),
            Err(TargetProfileBuildError::DuplicateSynchronizationRealization)
        );

        let descriptor = |source: TargetFactSource| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.atomic-source.v1".to_owned()).unwrap(),
            );
            builder
                .declare_synchronization_realization(
                    subject,
                    SynchronizationSupport::Realized,
                    &source,
                )
                .unwrap();
            builder.build().unwrap()
        };
        let left = descriptor(first);
        let right = descriptor(second);
        assert_ne!(
            left.canonical_descriptor(),
            right.canonical_descriptor(),
            "a source-revision change must move the complete declaration"
        );
        assert_eq!(
            left.checked().canonical_descriptor(),
            right.checked().canonical_descriptor(),
            "the checked descriptor encodes phase, authority, and validity, not the source identity"
        );
    }

    /// Every dimension of the subject, and the verdict, participates in both
    /// descriptors. A neighbouring subject is a different row, not a
    /// restatement.
    #[test]
    fn atomic_realization_subject_and_verdict_participate_in_identity_independently() {
        let source = public_external_source(1);
        let baseline_subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let descriptor = |subject, support| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.atomic-subject.v1".to_owned()).unwrap(),
            );
            builder
                .declare_synchronization_realization(subject, support, &source)
                .unwrap();
            builder.build().unwrap()
        };
        let realized = descriptor(baseline_subject, SynchronizationSupport::Realized);
        let refused = descriptor(baseline_subject, SynchronizationSupport::Unrealizable);
        assert_ne!(
            realized.canonical_descriptor(),
            refused.canonical_descriptor()
        );
        assert_ne!(
            realized.checked().canonical_descriptor(),
            refused.checked().canonical_descriptor()
        );
        let neighbour = descriptor(
            atomic_realization_neighbour(),
            SynchronizationSupport::Realized,
        );
        assert_ne!(
            realized.canonical_descriptor(),
            neighbour.canonical_descriptor()
        );
        assert_ne!(
            realized.checked().canonical_descriptor(),
            neighbour.checked().canonical_descriptor()
        );
        let collective = descriptor(
            atomic_realization_subject(SynchronizationKind::Collective),
            SynchronizationSupport::Realized,
        );
        assert_ne!(
            realized.canonical_descriptor(),
            collective.canonical_descriptor()
        );
    }

    fn subgroup_width(lanes: u32) -> SubgroupWidth {
        SubgroupWidth::new(lanes).expect("nonzero width")
    }

    fn subgroup_subject(lanes: u32, arithmetic: ArithmeticType) -> SubgroupRealizationSubject {
        SubgroupRealizationSubject::new(
            subgroup_width(lanes),
            arithmetic,
            SubgroupTransfer::InRangeXorShuffle,
        )
        .expect("power-of-two width at least 2 defines an XOR shuffle")
    }

    /// The one prepared subgroup-width query every realized fixture declares,
    /// in the governed prepared-entry spelling the Metal adapters dispatch.
    fn subgroup_width_query() -> TargetPropertyQuery {
        TargetPropertyQuery::new(
            TargetPropertyKey::new("tiler.target.prepared-entry.subgroup-width.v1").unwrap(),
            AvailabilityPhase::PreparedKernelPreflight,
            TargetPropertyProviderIdentity::new("tiler", "prepared-entry-properties", 1).unwrap(),
        )
        .unwrap()
    }

    /// Declares the width query exactly when `support` licenses a schedule,
    /// which is the owning contract's own condition.
    fn declare_query_for(builder: &mut TargetProfileBuilder, support: SubgroupSupport) {
        if matches!(support, SubgroupSupport::Realized) {
            builder
                .declare_subgroup_width_query(subgroup_width_query())
                .unwrap();
        }
    }

    /// Silence is `Unknown` for every subject, and it costs a profile that
    /// declares nothing not one descriptor byte.
    #[test]
    fn a_profile_declaring_no_subgroup_row_resolves_unknown() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        for profile in [
            TargetProfile::governed(),
            public_builder("acme.silent-subgroup.v1")
                .try_build()
                .unwrap(),
        ] {
            assert_eq!(
                profile.subgroup_realization(required, AvailabilityPhase::LaunchPreflight),
                SubgroupRealizationResolution::Unknown,
            );
            assert!(
                !profile
                    .canonical_descriptor()
                    .windows(SUBGROUP_REALIZATION_DOMAIN.len())
                    .any(|window| window == SUBGROUP_REALIZATION_DOMAIN),
                "an undeclaring profile writes none of the family's bytes, which is \
                 why the complete-declaration domain did not step"
            );
        }
    }

    #[test]
    fn declared_subgroup_rows_resolve_by_whole_subject_equality() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-realized.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(required, SubgroupSupport::Realized, source)
            .unwrap();
        declare_query_for(&mut builder, SubgroupSupport::Realized);
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Realized
        );
        assert_eq!(
            profile.subgroup_realization(
                subgroup_subject(64, ArithmeticType::F32),
                AvailabilityPhase::CompileProfile,
            ),
            SubgroupRealizationResolution::Unknown,
            "a neighbouring width must not satisfy the required subject"
        );
        assert_eq!(
            profile.subgroup_realization(
                subgroup_subject(32, ArithmeticType::Bf16),
                AvailabilityPhase::CompileProfile,
            ),
            SubgroupRealizationResolution::Unknown,
            "a neighbouring arithmetic type must not satisfy the required subject"
        );
        assert!(
            profile
                .canonical_descriptor()
                .windows(SUBGROUP_REALIZATION_DOMAIN.len())
                .any(|window| window == SUBGROUP_REALIZATION_DOMAIN)
        );
    }

    #[test]
    fn a_declared_unrealizable_subgroup_is_explicit() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-unrealizable.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(required, SubgroupSupport::Unrealizable, source)
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Unrealizable
        );
    }

    #[test]
    fn a_later_phase_subgroup_row_is_unknown_rather_than_deferred() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-later.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(
                required,
                SubgroupSupport::Realized,
                device_runtime_source(),
            )
            .unwrap();
        declare_query_for(&mut builder, SubgroupSupport::Realized);
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Unknown,
        );
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::LiveDevicePreflight),
            SubgroupRealizationResolution::Realized,
        );
    }

    #[test]
    fn an_exact_duplicate_subgroup_realization_is_refused_before_insertion() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-duplicate.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(required, SubgroupSupport::Realized, source.clone())
            .unwrap();
        let len = builder.subgroup.len();
        assert_eq!(
            builder.declare_subgroup_realization(required, SubgroupSupport::Realized, source),
            Err(TargetProfileBuildError::DuplicateSubgroupRealization)
        );
        assert_eq!(builder.subgroup.len(), len);
    }

    #[test]
    fn a_contradictory_subgroup_verdict_is_refused_before_insertion() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        for (first, second) in [
            (SubgroupSupport::Realized, SubgroupSupport::Unrealizable),
            (SubgroupSupport::Unrealizable, SubgroupSupport::Realized),
        ] {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-contradiction.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(required, first, source.clone())
                .unwrap();
            let len = builder.subgroup.len();
            assert_eq!(
                builder.declare_subgroup_realization(required, second, source.clone()),
                Err(TargetProfileBuildError::DuplicateSubgroupRealization),
                "sort order must not choose a winner between {first:?} then {second:?}"
            );
            assert_eq!(builder.subgroup.len(), len);
        }
    }

    #[test]
    fn subgroup_insertion_order_is_not_identity() {
        let source = public_external_source(1);
        let first = subgroup_subject(32, ArithmeticType::F32);
        let second = subgroup_subject(64, ArithmeticType::F32);
        assert!(first < second, "the fixture relies on subject order");
        let declare = |rows: [(SubgroupRealizationSubject, SubgroupSupport); 2]| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-order.v1".to_owned()).unwrap(),
            );
            for (subject, support) in rows {
                builder
                    .declare_subgroup_realization(subject, support, source.clone())
                    .unwrap();
            }
            declare_query_for(&mut builder, SubgroupSupport::Realized);
            builder.try_build().unwrap()
        };
        let forward = declare([
            (first, SubgroupSupport::Realized),
            (second, SubgroupSupport::Unrealizable),
        ]);
        let reverse = declare([
            (second, SubgroupSupport::Unrealizable),
            (first, SubgroupSupport::Realized),
        ]);
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        assert_eq!(
            forward.checked().canonical_descriptor(),
            reverse.checked().canonical_descriptor()
        );
    }

    #[test]
    fn subgroup_source_participates_in_complete_identity_independently() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        let first = public_external_source(1);
        let second = public_external_source(2);
        let mut colliding = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-source.v1".to_owned()).unwrap(),
        );
        colliding
            .declare_subgroup_realization(required, SubgroupSupport::Realized, first.clone())
            .unwrap();
        assert_eq!(
            colliding.declare_subgroup_realization(
                required,
                SubgroupSupport::Realized,
                second.clone(),
            ),
            Err(TargetProfileBuildError::DuplicateSubgroupRealization)
        );

        let descriptor = |source: TargetFactSource| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-source.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(required, SubgroupSupport::Realized, source)
                .unwrap();
            declare_query_for(&mut builder, SubgroupSupport::Realized);
            builder.try_build().unwrap()
        };
        let left = descriptor(first);
        let right = descriptor(second);
        assert_ne!(
            left.canonical_descriptor(),
            right.canonical_descriptor(),
            "a source-revision change must move the complete declaration"
        );
        assert_eq!(
            left.checked().canonical_descriptor(),
            right.checked().canonical_descriptor(),
            "the checked descriptor encodes phase, authority, and validity, not the source identity"
        );
    }

    #[test]
    fn measured_subgroup_declaration_uses_the_measured_source_authority() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        let source = compile_profile_measurement_source("1.0", "build-1");
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-measured.v1".to_owned()).unwrap(),
        );
        builder
            .declare_measured_subgroup_realization(required, SubgroupSupport::Realized, source)
            .unwrap();
        declare_query_for(&mut builder, SubgroupSupport::Realized);
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Realized
        );
        let fact = &profile.checked().subgroup()[0];
        assert_eq!(fact.authority(), FactAuthority::MeasuredProfile);
        assert_eq!(fact.validity(), FactValidityScope::MeasuredEnvironment);
    }

    #[test]
    fn subgroup_subject_and_verdict_participate_in_identity_independently() {
        let source = public_external_source(1);
        let baseline = subgroup_subject(32, ArithmeticType::F32);
        let descriptor = |subject, support| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-subject.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(subject, support, source.clone())
                .unwrap();
            declare_query_for(&mut builder, support);
            builder.try_build().unwrap()
        };
        let realized = descriptor(baseline, SubgroupSupport::Realized);
        let refused = descriptor(baseline, SubgroupSupport::Unrealizable);
        assert_ne!(
            realized.canonical_descriptor(),
            refused.canonical_descriptor()
        );
        assert_ne!(
            realized.checked().canonical_descriptor(),
            refused.checked().canonical_descriptor()
        );
        for (dimension, neighbour) in [
            ("width", subgroup_subject(64, ArithmeticType::F32)),
            ("arithmetic", subgroup_subject(32, ArithmeticType::Bf16)),
        ] {
            let other = descriptor(neighbour, SubgroupSupport::Realized);
            assert_ne!(
                realized.canonical_descriptor(),
                other.canonical_descriptor(),
                "the {dimension} dimension does not reach the complete descriptor"
            );
            assert_ne!(
                realized.checked().canonical_descriptor(),
                other.checked().canonical_descriptor(),
                "the {dimension} dimension does not reach the checked descriptor"
            );
        }
    }

    #[test]
    fn subgroup_perturbations_quote_distinct_failures() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        let source = public_external_source(1);
        let profile = |subject, support: SubgroupSupport| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-perturb.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(subject, support, source.clone())
                .unwrap();
            declare_query_for(&mut builder, support);
            builder.try_build().unwrap()
        };
        let realized = profile(required, SubgroupSupport::Realized);
        assert_eq!(
            format!(
                "{:?}",
                realized.subgroup_realization(required, AvailabilityPhase::CompileProfile)
            ),
            "Realized"
        );
        assert_eq!(
            format!(
                "{:?}",
                realized.subgroup_realization(
                    subgroup_subject(64, ArithmeticType::F32),
                    AvailabilityPhase::CompileProfile,
                )
            ),
            "Unknown",
            "width perturbation must be Unknown"
        );
        assert_eq!(
            format!(
                "{:?}",
                realized.subgroup_realization(
                    subgroup_subject(32, ArithmeticType::Bf16),
                    AvailabilityPhase::CompileProfile,
                )
            ),
            "Unknown",
            "arithmetic perturbation must be Unknown"
        );
        assert_eq!(
            format!(
                "{:?}",
                profile(required, SubgroupSupport::Unrealizable)
                    .subgroup_realization(required, AvailabilityPhase::CompileProfile)
            ),
            "Unrealizable"
        );
        let later = {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-perturb-phase.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(
                    required,
                    SubgroupSupport::Realized,
                    device_runtime_source(),
                )
                .unwrap();
            declare_query_for(&mut builder, SubgroupSupport::Realized);
            builder.try_build().unwrap()
        };
        assert_eq!(
            format!(
                "{:?}",
                later.subgroup_realization(required, AvailabilityPhase::CompileProfile)
            ),
            "Unknown",
            "compile-phase lookup of a later-phase row must be Unknown"
        );
        let silent = public_builder("test.subgroup-perturb-silence.v1")
            .try_build()
            .unwrap();
        assert_eq!(
            format!(
                "{:?}",
                silent.subgroup_realization(required, AvailabilityPhase::LaunchPreflight)
            ),
            "Unknown",
            "silence must be Unknown"
        );
        let mut colliding = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-perturb-source.v1".to_owned()).unwrap(),
        );
        colliding
            .declare_subgroup_realization(required, SubgroupSupport::Realized, source)
            .unwrap();
        assert_eq!(
            format!(
                "{:?}",
                colliding.declare_subgroup_realization(
                    required,
                    SubgroupSupport::Realized,
                    public_external_source(2),
                )
            ),
            "Err(DuplicateSubgroupRealization)",
            "a second source at the same subject and phase must refuse"
        );
    }

    #[test]
    fn independently_true_subgroup_neighbours_compose_into_no_permission() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-compose.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(
                subgroup_subject(64, ArithmeticType::F32),
                SubgroupSupport::Realized,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_subgroup_realization(
                subgroup_subject(32, ArithmeticType::Bf16),
                SubgroupSupport::Realized,
                source,
            )
            .unwrap();
        declare_query_for(&mut builder, SubgroupSupport::Realized);
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Unknown,
            "independently true neighbouring facts must not compose into a permission"
        );
    }

    /// The owning profile contract of the prepared width query, one
    /// perturbation per refusal: missing, orphan (against silence and against
    /// an explicit refusal), duplicate, and wrong phase each name their own
    /// typed error, and none inserts a repairable draft.
    #[test]
    fn the_subgroup_width_query_contract_refuses_each_perturbation_by_name() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        // Perturb the query's presence alone.
        let mut missing = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-query-missing.v1".to_owned()).unwrap(),
        );
        missing
            .declare_subgroup_realization(required, SubgroupSupport::Realized, source.clone())
            .unwrap();
        assert_eq!(
            missing.try_build().unwrap_err(),
            TargetProfileBuildError::MissingSubgroupWidthQuery,
        );
        // Perturb the realization alone: silence, then an explicit refusal.
        let mut orphan = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-query-orphan.v1".to_owned()).unwrap(),
        );
        orphan
            .declare_subgroup_width_query(subgroup_width_query())
            .unwrap();
        assert_eq!(
            orphan.try_build().unwrap_err(),
            TargetProfileBuildError::OrphanSubgroupWidthQuery,
        );
        let mut refused = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-query-unrealizable.v1".to_owned()).unwrap(),
        );
        refused
            .declare_subgroup_realization(required, SubgroupSupport::Unrealizable, source.clone())
            .unwrap();
        refused
            .declare_subgroup_width_query(subgroup_width_query())
            .unwrap();
        assert_eq!(
            refused.try_build().unwrap_err(),
            TargetProfileBuildError::OrphanSubgroupWidthQuery,
        );
        // Perturb the cardinality alone.
        let mut duplicate = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-query-duplicate.v1".to_owned()).unwrap(),
        );
        duplicate
            .declare_subgroup_width_query(subgroup_width_query())
            .unwrap();
        assert_eq!(
            duplicate.declare_subgroup_width_query(subgroup_width_query()),
            Err(TargetProfileBuildError::DuplicateSubgroupWidthQuery),
        );
        // Perturb the phase alone: a live-device query cannot answer a
        // prepared-pipeline property, and the refusal happens before insertion.
        let mut wrong_phase = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-query-phase.v1".to_owned()).unwrap(),
        );
        assert_eq!(
            wrong_phase.declare_subgroup_width_query(
                TargetPropertyQuery::new(
                    TargetPropertyKey::new("tiler.target.prepared-entry.subgroup-width.v1")
                        .unwrap(),
                    AvailabilityPhase::LiveDevicePreflight,
                    TargetPropertyProviderIdentity::new("tiler", "prepared-entry-properties", 1)
                        .unwrap(),
                )
                .unwrap(),
            ),
            Err(TargetProfileBuildError::InvalidSubgroupQueryPhase {
                required: AvailabilityPhase::PreparedKernelPreflight,
                actual: AvailabilityPhase::LiveDevicePreflight,
            }),
        );
        assert!(wrong_phase.subgroup_query.is_none());
    }

    /// The query is identity in the complete declaration exactly as in the
    /// checked descriptor: its family bytes exist only when it does, and two
    /// realized profiles differing only in the query contract differ.
    #[test]
    fn the_subgroup_width_query_moves_the_complete_declaration() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        let with_query = |query: TargetPropertyQuery| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-query-identity.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(required, SubgroupSupport::Realized, source.clone())
                .unwrap();
            builder.declare_subgroup_width_query(query).unwrap();
            builder.try_build().unwrap()
        };
        let baseline = with_query(subgroup_width_query());
        let renamed = with_query(
            TargetPropertyQuery::new(
                TargetPropertyKey::new("tiler.target.prepared-entry.subgroup-width-second.v1")
                    .unwrap(),
                AvailabilityPhase::PreparedKernelPreflight,
                TargetPropertyProviderIdentity::new("tiler", "prepared-entry-properties", 1)
                    .unwrap(),
            )
            .unwrap(),
        );
        assert_ne!(
            baseline.canonical_descriptor(),
            renamed.canonical_descriptor(),
            "the query contract does not reach the complete declaration"
        );
        assert!(
            baseline
                .canonical_descriptor()
                .windows(SUBGROUP_WIDTH_QUERY_DOMAIN.len())
                .any(|window| window == SUBGROUP_WIDTH_QUERY_DOMAIN),
        );
        let silent = public_builder("test.subgroup-query-silent.v1")
            .try_build()
            .unwrap();
        assert!(
            !silent
                .canonical_descriptor()
                .windows(SUBGROUP_WIDTH_QUERY_DOMAIN.len())
                .any(|window| window == SUBGROUP_WIDTH_QUERY_DOMAIN),
            "a profile with no query must write none of the family's bytes"
        );
    }

    #[test]
    fn malformed_capability_declarations_fail_at_the_checked_boundary() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::IndexArithmeticU64)
            .unwrap()
            .bound = 2;
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::MalformedProfile { rule: "fact-bound" })
        );
    }

    #[test]
    fn quantitative_facts_retain_and_derive_from_their_structured_producer() {
        let mut builder = TargetProfileBuilder::governed();
        let source = measured_capability_source();
        for declaration in &mut builder.quantitative {
            declaration.source = Arc::clone(&source);
        }
        builder.dispatchability.clear();
        let profile = builder.try_build().unwrap();
        assert!(
            profile
                .data
                .quantitative
                .iter()
                .all(|declaration| declaration.source == source)
        );
        for fact in profile.checked().facts() {
            assert_eq!(fact.phase(), AvailabilityPhase::LiveDevicePreflight);
            assert_eq!(fact.authority(), FactAuthority::DeviceRuntime);
            assert_eq!(fact.validity(), FactValidityScope::DeviceInstance);
        }
    }

    #[test]
    fn sparse_quantitative_omission_resolves_to_unknown() {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.sparse-quantitative.v1".to_owned()).unwrap(),
        );
        builder
            .declare_max_threads_per_grid_axis(64, source)
            .unwrap();
        let profile = builder.build().unwrap();
        let proposal = FeasibilityProposal::new(
            "requires-workgroup",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1)],
            Vec::new(),
        )
        .unwrap();
        assert!(matches!(
            profile
                .checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Unknown(_)
        ));
    }

    #[test]
    fn measured_profile_omissions_remain_unknown_across_fact_families() {
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.sparse-measured.v1".to_owned()).unwrap(),
        );
        builder
            .declare_measured_max_threads_per_grid_axis(
                64,
                compile_profile_measurement_source("1.0", "build-1"),
            )
            .unwrap();
        let profile = builder.build().unwrap();
        let proposal = FeasibilityProposal::new(
            "requires-omitted-facts",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1)],
            vec![NumericalRequirement::new(
                NumericalDimension::Contraction,
                ArithmeticType::F32,
                F32::resolved_type(),
                DimensionBehaviour::Transform(NumericalPermission::Forbidden),
            )],
        )
        .unwrap();
        assert!(matches!(
            profile
                .checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Unknown(_)
        ));
        assert_eq!(
            profile.dtype_dispatchability(&F32::resolved_type(), AvailabilityPhase::CompileProfile),
            DTypeDispatchabilityResolution::Unknown
        );
    }

    #[test]
    fn each_quantitative_axis_binds_its_own_source_into_identity() {
        let baseline = public_builder("test.mixed-sources.v1").build().unwrap();
        let mut mixed = public_builder("test.mixed-sources.v1");
        let revised = public_external_source(2);
        mixed
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::LocalMemoryBytes)
            .unwrap()
            .source = Arc::clone(&revised.0);
        let mixed = mixed.build().unwrap();
        assert_ne!(
            baseline.canonical_descriptor(),
            mixed.canonical_descriptor(),
            "changing one axis's source must change complete identity"
        );
        assert_ne!(
            baseline.request_subject_bytes(),
            mixed.request_subject_bytes(),
            "the request subject must bind per-axis provenance"
        );
    }

    #[test]
    fn one_axis_may_be_refined_at_a_later_availability_phase() {
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.staged-quantitative.v1".to_owned()).unwrap(),
        );
        builder
            .declare_max_threads_per_workgroup(64, public_external_source(1))
            .unwrap();
        builder
            .declare_max_threads_per_workgroup(32, TargetFactSource(measured_capability_source()))
            .unwrap();
        let profile = builder
            .build()
            .expect("facts at distinct phases do not collide");
        let proposal = FeasibilityProposal::new(
            "requires-workgroup",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 48)],
            Vec::new(),
        )
        .unwrap();
        assert!(matches!(
            profile
                .checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Proven(_)
        ));
        assert!(matches!(
            profile
                .checked()
                .assess(&proposal, AvailabilityPhase::LiveDevicePreflight),
            FeasibilityOutcome::Rejected(_)
        ));
    }

    #[test]
    fn request_subject_binds_local_memory() {
        let baseline = public_builder("test.request-subject.v1").build().unwrap();
        let mut changed = public_builder("test.request-subject.v1");
        changed
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::LocalMemoryBytes)
            .unwrap()
            .bound += 1;
        let changed = changed.build().unwrap();
        assert_ne!(
            baseline.request_subject_bytes(),
            changed.request_subject_bytes()
        );
        assert_eq!(
            baseline.request_subject_bytes(),
            baseline.canonical_descriptor()
        );
    }

    #[test]
    fn arithmetic_support_and_device_address_width_move_identity_independently() {
        let baseline = public_builder("test.width-independence.v1")
            .build()
            .unwrap();
        let mut arithmetic = public_builder("test.width-independence.v1");
        arithmetic
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::IndexArithmeticU64)
            .unwrap()
            .bound = IndexArithmeticSupport::Unsupported.bound();
        let arithmetic = arithmetic.build().unwrap();
        let mut address = public_builder("test.width-independence.v1");
        address
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::DeviceAddressWidthBits)
            .unwrap()
            .bound = u64::from(DeviceAddressWidth::Bits32.bits());
        let address = address.build().unwrap();

        assert_ne!(
            baseline.canonical_descriptor(),
            arithmetic.canonical_descriptor()
        );
        assert_ne!(
            baseline.canonical_descriptor(),
            address.canonical_descriptor()
        );
        assert_ne!(
            arithmetic.canonical_descriptor(),
            address.canonical_descriptor()
        );
    }

    #[test]
    fn governed_profile_does_not_invent_a_device_address_width() {
        assert!(
            TargetProfile::governed()
                .checked()
                .facts()
                .iter()
                .all(|fact| fact.axis() != CapabilityAxis::DeviceAddressWidthBits)
        );
    }

    #[test]
    fn dtype_dispatch_is_exact_sparse_and_has_no_inheritance() {
        let f32 = F32::resolved_type();
        let parameterized_f32 = ResolvedValueType::parameterized(
            TypeKey::new("test", "wrapped", 1).unwrap(),
            TypeArguments::new([CanonicalValue::value_type(f32.clone())]).unwrap(),
        )
        .unwrap();
        let mut builder = TargetProfileBuilder::governed();
        builder.dispatchability = vec![dispatch_fact(
            f32.clone(),
            DTypeDispatchability::Dispatchable,
        )];
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.dtype_dispatchability(&f32, AvailabilityPhase::CompileProfile),
            DTypeDispatchabilityResolution::Dispatchable
        );
        assert_eq!(
            profile.dtype_dispatchability(&parameterized_f32, AvailabilityPhase::CompileProfile,),
            DTypeDispatchabilityResolution::Unknown
        );

        let mut unsupported = TargetProfileBuilder::governed();
        unsupported.dispatchability = vec![dispatch_fact(
            parameterized_f32.clone(),
            DTypeDispatchability::Unsupported,
        )];
        assert_eq!(
            unsupported
                .try_build()
                .unwrap()
                .dtype_dispatchability(&parameterized_f32, AvailabilityPhase::CompileProfile,),
            DTypeDispatchabilityResolution::Unsupported
        );
    }

    #[test]
    fn dtype_dispatch_refines_by_phase_and_defers_before_its_first_fact() {
        let f32 = F32::resolved_type();
        let mut staged = TargetProfileBuilder::new(
            TargetProfileKey::new("test.staged-dispatch.v1".to_owned()).unwrap(),
        );
        staged.dispatchability.push(dispatch_fact(
            f32.clone(),
            DTypeDispatchability::Dispatchable,
        ));
        staged.dispatchability.push(DTypeDispatchabilityFact {
            resolved_type: f32.clone(),
            verdict: DTypeDispatchability::Unsupported,
            source: measured_capability_source(),
        });
        let staged = staged.build().unwrap();
        assert_eq!(
            staged.dtype_dispatchability(&f32, AvailabilityPhase::CompileProfile),
            DTypeDispatchabilityResolution::Dispatchable
        );
        assert_eq!(
            staged.dtype_dispatchability(&f32, AvailabilityPhase::LiveDevicePreflight),
            DTypeDispatchabilityResolution::Unsupported
        );

        let mut later_only = TargetProfileBuilder::new(
            TargetProfileKey::new("test.later-dispatch.v1".to_owned()).unwrap(),
        );
        later_only.dispatchability.push(DTypeDispatchabilityFact {
            resolved_type: f32.clone(),
            verdict: DTypeDispatchability::Dispatchable,
            source: measured_capability_source(),
        });
        let later_only = later_only.build().unwrap();
        assert_eq!(
            later_only.dtype_dispatchability(&f32, AvailabilityPhase::CompileProfile),
            DTypeDispatchabilityResolution::Deferred {
                available_at: AvailabilityPhase::LiveDevicePreflight,
            }
        );
    }

    #[test]
    fn scalar_subject_row_swaps_change_identity_and_exact_feasibility() {
        let source = governed_profile_source();
        // Two governed subjects rather than two hand-built nominal ones: the
        // relocated validator admits exactly one value identity per arithmetic
        // type, so a subject over an unregistered type cannot be constructed.
        let a = ScalarArithmetic::f32();
        let b = ScalarArithmetic::new(ArithmeticType::F16, governed_scalar("f16"))
            .expect("the catalog registers f16 over tiler::f16@1");
        let row = |subject, behaviour| ScalarHonourabilityDeclaration {
            subject,
            dimension: NumericalDimension::InputSubnormals,
            behaviour,
            means: HonouringMeans::SupportedExactly,
            source: Arc::clone(&source),
        };
        let preserve = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
        let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        });
        let profile = |key: &str,
                       first: ScalarHonourabilityDeclaration,
                       second: ScalarHonourabilityDeclaration| {
            let mut builder =
                TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
            builder.scalar = vec![first, second];
            builder.build().unwrap()
        };
        let left = profile(
            "test.scalar-row-swap.v1",
            row(a.clone(), preserve),
            row(b.clone(), flush),
        );
        let right = profile(
            "test.scalar-row-swap.v1",
            row(a.clone(), flush),
            row(b, preserve),
        );
        assert_ne!(left.canonical_descriptor(), right.canonical_descriptor());

        let proposal = FeasibilityProposal::new(
            "exact-scalar-subject",
            Vec::new(),
            vec![NumericalRequirement::new(
                NumericalDimension::InputSubnormals,
                a.arithmetic(),
                a.resolved_type().clone(),
                preserve,
            )],
        )
        .unwrap();
        assert!(matches!(
            left.checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Proven(_)
        ));
        assert!(matches!(
            right
                .checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Unknown(_)
        ));
    }

    #[test]
    fn duplicate_exact_dtype_dispatch_claims_are_rejected() {
        let value_type = nominal("same");
        let mut builder = TargetProfileBuilder::governed();
        builder.dispatchability = vec![
            dispatch_fact(value_type.clone(), DTypeDispatchability::Dispatchable),
            dispatch_fact(value_type, DTypeDispatchability::Unsupported),
        ];
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::DuplicateDispatchability)
        );
    }

    fn bf16_subject() -> ScalarArithmetic {
        ScalarArithmetic::new(
            ArithmeticType::Bf16,
            registered_arithmetic_value_type(ArithmeticType::Bf16)
                .expect("the governed catalog registers bf16"),
        )
        .expect("the bf16 policy subject is validated")
    }

    fn device_runtime_source() -> TargetFactSource {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-runtime-compiler".to_owned(),
            "1.0".to_owned(),
            None,
        )
        .unwrap();
        let environment = TargetExecutionEnvironment::builder()
            .platform("test-platform".to_owned())
            .platform_version("1.0".to_owned())
            .platform_build("build-1".to_owned())
            .architecture("test-architecture".to_owned())
            .hardware("test-hardware".to_owned())
            .build()
            .unwrap();
        TargetFactSource::measured(
            TargetFactProducerIdentity::new("test.evaluation-order-observer.v1".to_owned(), 1)
                .unwrap(),
            MeasuredFactAuthority::DeviceRuntime,
            [TargetMeasurementContext::new([compiler], environment).unwrap()],
        )
        .unwrap()
    }

    /// The fail-closed half: silence is `Unknown` for every subject and licence,
    /// and it costs a profile that declares nothing not one descriptor byte.
    #[test]
    fn a_profile_declaring_no_evaluation_order_row_resolves_unknown() {
        for profile in [
            TargetProfile::governed(),
            public_builder("acme.silent.v1").try_build().unwrap(),
        ] {
            for subject in [ScalarArithmetic::f32(), bf16_subject()] {
                for licence in [
                    BackendArithmeticLicence::Withheld,
                    BackendArithmeticLicence::Granted,
                ] {
                    assert_eq!(
                        profile.evaluation_order_preservation(
                            &subject,
                            licence,
                            AvailabilityPhase::LaunchPreflight,
                        ),
                        EvaluationOrderResolution::Unknown,
                        "an undeclared {} row must not resolve",
                        licence.key()
                    );
                }
            }
            assert!(
                !profile
                    .canonical_descriptor()
                    .windows(EVALUATION_ORDER_DOMAIN.len())
                    .any(|window| window == EVALUATION_ORDER_DOMAIN),
                "an undeclaring profile writes none of the family's bytes, which is \
                 why the complete-declaration domain did not step"
            );
        }
    }

    /// The observing half: a declared row resolves per licence, and neither the
    /// other licence nor a neighbouring arithmetic type inherits it.
    #[test]
    fn declared_evaluation_order_rows_resolve_per_licence_and_are_not_inherited() {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::Preserved,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                EvaluationOrderPreservation::NotPreserved,
                source,
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                AvailabilityPhase::CompileProfile,
            ),
            EvaluationOrderResolution::Preserved
        );
        assert_eq!(
            profile.evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                AvailabilityPhase::CompileProfile,
            ),
            EvaluationOrderResolution::NotPreserved
        );
        for licence in [
            BackendArithmeticLicence::Withheld,
            BackendArithmeticLicence::Granted,
        ] {
            assert_eq!(
                profile.evaluation_order_preservation(
                    &bf16_subject(),
                    licence,
                    AvailabilityPhase::CompileProfile,
                ),
                EvaluationOrderResolution::Unknown,
                "an `f32` row is not evidence about `bf16`"
            );
        }
        assert!(
            profile
                .canonical_descriptor()
                .windows(EVALUATION_ORDER_DOMAIN.len())
                .any(|window| window == EVALUATION_ORDER_DOMAIN)
        );
        assert_ne!(
            profile.canonical_descriptor(),
            TargetProfile::governed().canonical_descriptor()
        );
    }

    #[test]
    fn a_later_phase_evaluation_order_row_defers_rather_than_resolving() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                EvaluationOrderPreservation::NotPreserved,
                device_runtime_source(),
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                AvailabilityPhase::CompileProfile,
            ),
            EvaluationOrderResolution::Deferred {
                available_at: AvailabilityPhase::LiveDevicePreflight,
            }
        );
        assert_eq!(
            profile.evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                AvailabilityPhase::LiveDevicePreflight,
            ),
            EvaluationOrderResolution::NotPreserved
        );
    }

    #[test]
    fn a_second_evaluation_order_verdict_at_one_phase_is_refused_atomically() {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::Preserved,
                source.clone(),
            )
            .unwrap();
        assert_eq!(
            builder.declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::NotPreserved,
                source,
            ),
            Err(
                TargetProfileBuildError::DuplicateEvaluationOrderPreservation {
                    licence: BackendArithmeticLicence::Withheld.key(),
                    phase: AvailabilityPhase::CompileProfile,
                }
            )
        );
        // The refusal inserted nothing, so the first verdict still stands.
        assert_eq!(
            builder.try_build().unwrap().evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                AvailabilityPhase::CompileProfile,
            ),
            EvaluationOrderResolution::Preserved
        );
    }

    #[test]
    fn evaluation_order_subject_licence_and_verdict_participate_in_complete_identity() {
        let descriptor = |subject: ScalarArithmetic, licence, preservation| {
            let mut builder = TargetProfileBuilder::governed();
            builder
                .declare_evaluation_order_preservation(
                    subject,
                    licence,
                    preservation,
                    public_external_source(1),
                )
                .unwrap();
            builder.try_build().unwrap().canonical_descriptor().to_vec()
        };
        let baseline = descriptor(
            ScalarArithmetic::f32(),
            BackendArithmeticLicence::Withheld,
            EvaluationOrderPreservation::Preserved,
        );
        assert_ne!(
            baseline,
            descriptor(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::NotPreserved,
            )
        );
        assert_ne!(
            baseline,
            descriptor(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                EvaluationOrderPreservation::Preserved,
            )
        );
        assert_ne!(
            baseline,
            descriptor(
                bf16_subject(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::Preserved,
            )
        );
    }

    /// Silence is `Unknown` and costs a profile that declares nothing not one
    /// descriptor byte.
    #[test]
    fn a_profile_declaring_no_tree_width_policy_resolves_unknown() {
        for profile in [
            TargetProfile::governed(),
            public_builder("acme.silent-tree-width.v1")
                .try_build()
                .unwrap(),
        ] {
            assert_eq!(
                profile.workgroup_tree_width_policy(AvailabilityPhase::LaunchPreflight),
                WorkgroupTreeWidthPolicyResolution::Unknown,
            );
            assert!(
                !profile
                    .canonical_descriptor()
                    .windows(WORKGROUP_TREE_WIDTH_POLICY_DOMAIN.len())
                    .any(|window| window == WORKGROUP_TREE_WIDTH_POLICY_DOMAIN),
                "an undeclaring profile writes none of the family's bytes, which is \
                 why the complete-declaration domain did not step"
            );
        }
    }

    #[test]
    fn a_declared_tree_width_policy_resolves_and_moves_identity() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                public_external_source(1),
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.workgroup_tree_width_policy(AvailabilityPhase::CompileProfile),
            WorkgroupTreeWidthPolicyResolution::Declared(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1
            )
        );
        assert!(
            profile
                .canonical_descriptor()
                .windows(WORKGROUP_TREE_WIDTH_POLICY_DOMAIN.len())
                .any(|window| window == WORKGROUP_TREE_WIDTH_POLICY_DOMAIN)
        );
        assert_ne!(
            profile.canonical_descriptor(),
            TargetProfile::governed().canonical_descriptor()
        );
    }

    #[test]
    fn a_later_phase_tree_width_policy_defers_rather_than_resolving() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                device_runtime_source(),
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.workgroup_tree_width_policy(AvailabilityPhase::CompileProfile),
            WorkgroupTreeWidthPolicyResolution::Deferred {
                available_at: AvailabilityPhase::LiveDevicePreflight,
            }
        );
        assert_eq!(
            profile.workgroup_tree_width_policy(AvailabilityPhase::LiveDevicePreflight),
            WorkgroupTreeWidthPolicyResolution::Declared(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1
            )
        );
    }

    #[test]
    fn a_second_tree_width_policy_at_one_phase_is_refused_atomically() {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                source.clone(),
            )
            .unwrap();
        assert_eq!(
            builder.declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                source,
            ),
            Err(TargetProfileBuildError::DuplicateWorkgroupTreeWidthPolicy {
                phase: AvailabilityPhase::CompileProfile,
            })
        );
        assert_eq!(
            builder
                .try_build()
                .unwrap()
                .workgroup_tree_width_policy(AvailabilityPhase::CompileProfile),
            WorkgroupTreeWidthPolicyResolution::Declared(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1
            )
        );
    }

    #[test]
    fn changing_the_tree_width_policy_tag_moves_canonical_identity() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                public_external_source(1),
            )
            .unwrap();
        let declared = builder.try_build().unwrap().canonical_descriptor().to_vec();
        assert!(
            declared
                .windows(WORKGROUP_TREE_WIDTH_POLICY_DOMAIN.len())
                .any(|window| window == WORKGROUP_TREE_WIDTH_POLICY_DOMAIN)
        );
        assert_ne!(declared, TargetProfile::governed().canonical_descriptor());
    }

    #[test]
    fn complete_descriptor_is_cached_canonical_and_schema_distinct() {
        let mut left = TargetProfileBuilder::governed();
        left.scalar.reverse();
        let right = TargetProfileBuilder::governed();
        let left = left.try_build().unwrap();
        let right = right.try_build().unwrap();
        assert_eq!(left.canonical_descriptor(), right.canonical_descriptor());
        let cloned = left.clone();
        assert!(
            std::ptr::eq(left.canonical_descriptor(), cloned.canonical_descriptor()),
            "cloning a frozen profile retains its shared immutable allocation"
        );
        assert!(std::ptr::eq(
            left.canonical_descriptor(),
            left.canonical_descriptor()
        ));
        assert_ne!(
            left.canonical_descriptor(),
            left.checked().canonical_descriptor()
        );
        assert!(
            left.canonical_descriptor()
                .windows(COMPLETE_PROFILE_DESCRIPTOR_DOMAIN.len())
                .any(|window| window == COMPLETE_PROFILE_DESCRIPTOR_DOMAIN)
        );
    }

    #[test]
    fn exact_dtype_and_verdict_participate_in_complete_identity() {
        let descriptor = |resolved_type, verdict| {
            let mut builder = TargetProfileBuilder::governed();
            builder.dispatchability = vec![dispatch_fact(resolved_type, verdict)];
            builder.try_build().unwrap().canonical_descriptor().to_vec()
        };
        assert_ne!(
            descriptor(nominal("a"), DTypeDispatchability::Dispatchable),
            descriptor(nominal("b"), DTypeDispatchability::Dispatchable)
        );
        assert_ne!(
            descriptor(nominal("a"), DTypeDispatchability::Dispatchable),
            descriptor(nominal("a"), DTypeDispatchability::Unsupported)
        );
    }

    #[test]
    fn complete_descriptor_obeys_the_artifact_identity_bound() {
        let mut builder = TargetProfileBuilder::governed();
        builder.dispatchability = (0..1_024)
            .map(|index| {
                dispatch_fact(
                    nominal(format!("dtype-{index:02}")),
                    DTypeDispatchability::Dispatchable,
                )
            })
            .collect();
        builder.dispatchability.reverse();
        let failure = builder.build().unwrap_err();
        assert!(matches!(
            failure,
            TargetProfileBuildError::DescriptorTooLong { actual, max }
                if actual > max && max == MAX_TARGET_PROFILE_DESCRIPTOR_BYTES
        ));
    }

    #[test]
    fn target_requests_are_nonempty_unique_and_preserve_order() {
        assert_eq!(TargetRequest::new([]), Err(TargetRequestError::Empty));
        let first = public_builder("test.ordered-a.v1").build().unwrap();
        let second = public_builder("test.ordered-b.v1").build().unwrap();
        assert_eq!(
            TargetRequest::new([first.clone(), first.clone()]),
            Err(TargetRequestError::DuplicateProfile {
                profile: first.profile_key().clone(),
                first: 0,
                duplicate: 1,
            })
        );
        let request = TargetRequest::new([second.clone(), first.clone()]).unwrap();
        assert_eq!(request.profiles(), &[second, first]);
    }

    /// Candidate shapes where all three reduction strategies are expressible, for
    /// one grid-axis bound.
    ///
    /// This helper deliberately reads only the algebraic and launch conditions it
    /// models: `governed_partition` withholds the split,
    /// `capped_tree_partition` withholds the tree — the two choose different
    /// participant counts and are asked separately rather than one standing in
    /// for the other — and the grid-axis bound assesses the prologue's
    /// one-invocation-per-element launch. It does not assess a plan's target
    /// feasibility: local memory and synchronization realization are read later
    /// by the physical feasibility path, which can withhold the tree for every
    /// shape on a profile such as the governed baseline.
    fn three_strategy_domain(grid_axis_bound: u64) -> Vec<(u64, u64)> {
        let mut domain = Vec::new();
        for rows in 1..=grid_axis_bound {
            for contributors in 1..=grid_axis_bound {
                if crate::physical::governed_partition(contributors).is_some()
                    && crate::physical::capped_tree_partition(contributors).is_some()
                    && rows * contributors <= grid_axis_bound
                {
                    domain.push((rows, contributors));
                }
            }
        }
        domain
    }

    /// **The prototype baseline has one three-strategy candidate shape, and it
    /// is not the profile calibration measures against.**
    ///
    /// This test was written as the measured-calibration trigger for
    /// [`calibrate-and-activate-parallel-reduction-selection`], and it could not
    /// have fired. It reads the bound from [`TargetProfileBuilder::governed`] —
    /// the *target-neutral prototype baseline*, keyed
    /// `tiler.prototype-target-neutral-baseline.v1` — while calibration measures
    /// against `tiler_build::BoundMetalCompileDeclaration::first_macos_apple9`.
    /// Both declared four, so the difference was invisible until one of them
    /// moved.
    ///
    /// **On 2026-08-04 the Metal row moved to a measured 268,435,456 and this
    /// one deliberately did not.** A macOS Apple9 device measurement is evidence
    /// about one target; a baseline standing in for every target cannot be
    /// widened by it, and widening it on the compiler's own say-so would be a
    /// number chosen rather than sourced. So the prototype row keeps its
    /// conservative four, and the real trigger lives in the crate that can see
    /// the profile it is about:
    /// `tiler_build::metal_plan::tests::the_measured_grid_axis_admits_more_than_one_three_strategy_shape`.
    ///
    /// What this test still checks is worth keeping and is what its name now
    /// says: for *this helper* the derivation `4 <= contributors <=
    /// rows * contributors <= bound` closes on `(1, 4)`, because both partition
    /// rules — `governed_partition` for the split and `capped_tree_partition`
    /// for the tree — withhold their strategy below four contributors. The two
    /// disagree about *which* participant count to take, never about which
    /// extents admit one, so the floor in that derivation is one number and not
    /// a coincidence. If the prototype baseline is ever widened — which is a
    /// product question about what a target-neutral guarantee should offer, not
    /// an authority question this ticket could answer — this fires.
    ///
    /// The raised-bound case below is not decoration: without it a domain
    /// computation that returned a one-element vector unconditionally would pass
    /// the real assertion, and the check would be indistinguishable from one that
    /// never ran.
    ///
    /// [`calibrate-and-activate-parallel-reduction-selection`]:
    ///     ../../../tickets/calibrate-and-activate-parallel-reduction-selection.md
    #[test]
    fn the_prototype_baseline_has_one_three_strategy_candidate_shape() {
        let bound = TargetProfileBuilder::governed()
            .quantitative
            .iter()
            .find(|declaration| declaration.axis == CapabilityAxis::GridAxisThreads)
            .expect("the governed profile declares the grid-axis limit")
            .bound;

        let domain = three_strategy_domain(bound);
        assert_eq!(
            domain,
            vec![(1, 4)],
            "the prototype baseline's three-strategy domain moved at grid-axis bound {bound}. \
             This is the helper's algebraic-and-launch domain, not a feasibility result. \
             The target-neutral baseline is not the profile calibration measures against: \
             widening it needs an authority covering every target, which no device measurement \
             can supply. The Metal profile's domain is reported by tiler-build's \
             the_measured_grid_axis_admits_more_than_one_three_strategy_shape"
        );

        // The same derivation at a wider bound, so the single point above is a
        // property of this profile rather than of the computation.
        let widened = three_strategy_domain(8);
        assert!(
            widened.len() > 1,
            "raising the grid-axis bound must admit more shapes, or this check cannot \
             distinguish a narrow profile from a broken domain computation: {widened:?}"
        );
        assert!(
            widened.contains(&(1, 4)) && widened.contains(&(2, 4)),
            "the widened domain must extend the narrow one rather than replace it: {widened:?}"
        );
    }

    fn verified_silu_contract() -> tiler_ir::semantic::accuracy::VerifiedAccuracyContract {
        let contract = tiler_ir::semantic::silu_f32_exponential_accuracy_contract();
        let facts = builtin_scalar_value_type_facts(contract.result_type())
            .expect("F32 carries builtin value-type facts");
        contract
            .verify(&facts)
            .expect("the registered SiLU contract verifies")
    }

    fn discharging_evidence(
        scope: &str,
        digest: &[u8],
    ) -> tiler_ir::semantic::accuracy::ConformanceEvidence {
        let reference = |text: &str| {
            tiler_ir::semantic::NormativeDefinitionRef::new(text)
                .expect("a fixture evidence field is canonical")
        };
        tiler_ir::semantic::accuracy::ConformanceEvidence::new(
            tiler_ir::semantic::accuracy::ConformanceEvidenceClass::NormativeGuarantee,
            reference(scope),
            reference("synthetic both-halves fixture, not a Metal specification claim"),
            reference("fixture.elementary.declaration"),
            reference("tiler test fixture, not a toolchain row"),
            None,
            None,
            None,
            digest,
        )
        .expect("the discharging fixture is well formed")
    }

    fn silu_realization(source: &TargetFactSource) -> ElementaryRealization {
        ElementaryRealization::new(
            &verified_silu_contract(),
            discharging_evidence(
                "fixture bound half for tiler::silu-f32@1",
                b"fixture:silu-bound-v1",
            ),
            discharging_evidence(
                "fixture exceptional half for tiler::silu-f32@1",
                b"fixture:silu-exceptional-v1",
            ),
            source,
        )
        .expect("a compile-profile source is accepted")
    }

    #[test]
    fn later_phase_source_is_refused_at_subject_construction() {
        let later = deferred_measurement_source();
        let error = ElementaryRealization::new(
            &verified_silu_contract(),
            discharging_evidence("later-phase bound", b"fixture:later-bound-v1"),
            discharging_evidence("later-phase exceptional", b"fixture:later-exceptional-v1"),
            &later,
        )
        .expect_err("a live-device source cannot speak at compile profile");
        assert_eq!(
            error,
            ElementaryRealizationError::LaterPhaseSource {
                phase: AvailabilityPhase::LiveDevicePreflight,
            }
        );
    }

    fn deferred_measurement_source() -> TargetFactSource {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-runtime-compiler".to_owned(),
            "1.0".to_owned(),
            None,
        )
        .unwrap();
        let environment = TargetExecutionEnvironment::builder()
            .platform("test-platform".to_owned())
            .platform_version("1.0".to_owned())
            .platform_build("build-1".to_owned())
            .architecture("test-architecture".to_owned())
            .hardware("test-hardware".to_owned())
            .build()
            .unwrap();
        let context = TargetMeasurementContext::new([compiler], environment).unwrap();
        TargetFactSource::measured(
            TargetFactProducerIdentity::new("test.runtime-probe.v1".to_owned(), 1).unwrap(),
            MeasuredFactAuthority::DeviceRuntime,
            [context],
        )
        .unwrap()
    }

    #[test]
    fn exact_duplicate_elementary_realization_is_refused() {
        let source = public_external_source(1);
        let realization = silu_realization(&source);
        let mut builder = public_builder("test.elementary-duplicate.v1");
        builder
            .declare_elementary_realization(realization.clone())
            .unwrap();
        assert_eq!(
            builder.declare_elementary_realization(realization),
            Err(TargetProfileBuildError::DuplicateElementaryRealization)
        );
    }

    #[test]
    fn distinct_same_operation_contracts_remain_separate_candidates() {
        let source = public_external_source(1);
        let first = silu_realization(&source);
        let second = ElementaryRealization::new(
            &verified_silu_contract(),
            discharging_evidence("second bound half", b"fixture:silu-bound-v2"),
            discharging_evidence("second exceptional half", b"fixture:silu-exceptional-v2"),
            &source,
        )
        .unwrap();
        let mut builder = public_builder("test.elementary-distinct.v1");
        builder
            .declare_elementary_realization(first.clone())
            .unwrap();
        builder
            .declare_elementary_realization(second.clone())
            .unwrap();
        let profile = builder.build().unwrap();
        let declared = profile.declared_elementary_realizations();
        assert_eq!(declared.len(), 2);
        assert_eq!(declared[0].operation(), first.operation());
        assert_eq!(declared[1].operation(), second.operation());
        assert_ne!(declared[0], declared[1]);
    }

    #[test]
    fn a_profile_declaring_no_elementary_row_encodes_like_a_build_without_the_family() {
        let silent = public_builder("test.elementary-silent.v1").build().unwrap();
        let governed = TargetProfile::governed();
        assert!(silent.declared_elementary_realizations().is_empty());
        assert!(governed.declared_elementary_realizations().is_empty());
        assert!(
            !silent
                .canonical_descriptor()
                .windows(ELEMENTARY_REALIZATION_DOMAIN.len())
                .any(|window| window == ELEMENTARY_REALIZATION_DOMAIN)
        );
        assert!(
            !governed
                .canonical_descriptor()
                .windows(ELEMENTARY_REALIZATION_DOMAIN.len())
                .any(|window| window == ELEMENTARY_REALIZATION_DOMAIN)
        );
    }

    #[test]
    fn declaring_an_elementary_row_appends_the_terminal_family_without_stepping_the_domain() {
        let source = public_external_source(1);
        let mut builder = public_builder("test.elementary-encoded.v1");
        let silent = builder.clone().build().unwrap();
        builder
            .declare_elementary_realization(silu_realization(&source))
            .unwrap();
        let declared = builder.build().unwrap();
        assert_ne!(
            silent.canonical_descriptor(),
            declared.canonical_descriptor()
        );
        assert!(
            declared
                .canonical_descriptor()
                .windows(COMPLETE_PROFILE_DESCRIPTOR_DOMAIN.len())
                .any(|window| window == COMPLETE_PROFILE_DESCRIPTOR_DOMAIN)
        );
        assert!(
            declared
                .canonical_descriptor()
                .windows(ELEMENTARY_REALIZATION_DOMAIN.len())
                .any(|window| window == ELEMENTARY_REALIZATION_DOMAIN)
        );
        assert_eq!(declared.declared_elementary_realizations().len(), 1);
        assert_eq!(
            declared.declared_elementary_realizations()[0].source_producer_key(),
            "test.external-profile-producer.v1"
        );
    }

    #[test]
    fn elementary_declaration_order_is_not_identity() {
        let source = public_external_source(1);
        let first = silu_realization(&source);
        let second = ElementaryRealization::new(
            &verified_silu_contract(),
            discharging_evidence("order bound half", b"fixture:silu-bound-order-v2"),
            discharging_evidence(
                "order exceptional half",
                b"fixture:silu-exceptional-order-v2",
            ),
            &source,
        )
        .unwrap();
        let mut left = public_builder("test.elementary-order.v1");
        left.declare_elementary_realization(first.clone()).unwrap();
        left.declare_elementary_realization(second.clone()).unwrap();
        let mut right = public_builder("test.elementary-order.v1");
        right.declare_elementary_realization(second).unwrap();
        right.declare_elementary_realization(first).unwrap();
        assert_eq!(
            left.build().unwrap().canonical_descriptor(),
            right.build().unwrap().canonical_descriptor()
        );
    }
}
