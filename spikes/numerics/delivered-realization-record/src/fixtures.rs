//! The adversarial fixtures the ticket's Required-evidence section enumerates.
//!
//! Every subject is built through the **production** validator —
//! `tiler_compiler::target::ScalarArithmetic::new` — so a pair these fixtures use
//! is one the governed built-in scalar catalog already admitted. Nothing here
//! re-implements that check.
//!
//! # Measurement boundary
//!
//! The evidence rows below are **checked synthetic** evidence, which
//! `express-metal-honourability-in-the-shared-form` explicitly admits for this
//! ticket ("Delivered-realization redesign may use checked synthetic evidence").
//! They are structurally valid and they are not measurements.
//!
//! One consequence must not be read past. **Fact — no target profile in this tree
//! declares an `f16` honourability row.** The governed baseline
//! (`tiler.prototype-target-neutral-baseline.v1`) declares twelve rows, every one
//! over `ScalarArithmetic::governed_f32()`; the bound Metal declaration adds `f32`
//! and `bf16` rows and states that "F16 is deliberately absent"; and
//! `declare_metal_f32_subnormal_behaviour` reads only the F32 row. The measured
//! `f16`/`f32` divergence lives one crate away, in `tiler-metal`'s
//! `MetalSubnormalArithmeticFacts`, which states F32 flush, F16 preserve, BF16
//! flush and is never bridged into a compiler profile for F16.
//!
//! So the `f16`/`f32` fixture below proves what the ticket asks it to prove —
//! that the **record** carries two dtypes' subnormal evidence without collision —
//! and it proves nothing about any measured target. A production record for a
//! second dtype waits on a profile that declares one.

use tiler_artifact::program::{TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef};
use tiler_compiler::target::ScalarArithmetic;
use tiler_ir::program::SemanticOccurrence;
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, NumericalRealization, SubnormalMode,
};
use tiler_ir::semantic::{
    ResolvedValueType, StrictAffineU4, TypeKey, complex_value_type, microscaling_scheme_keys,
};

use crate::record::TargetEvidenceDeclaration;
use crate::shared::{
    CompilerBuildIdentity, CompilerBuildRole, DIMENSION_COUNT, DimensionBehaviour,
    ExecutionEnvironmentIdentity, FactAuthority, FactEvidenceBasis, FactSourceProvenance,
    FactValidityScope, HonouringMeans, MeasurementContext, NumericalDimension,
    NumericalObligationKey, PolicyLocus, ProvenanceIdentity, RelaxationRequirement,
    ScalarArithmeticSubject,
};

/// The governed nominal identity of one built-in scalar.
///
/// # Panics
///
/// Panics only if a governed catalog name violates the identity grammar, which a
/// `tiler-ir` test already pins.
#[must_use]
pub fn governed_scalar(name: &str) -> ResolvedValueType {
    ResolvedValueType::nominal(
        TypeKey::new("tiler", name, 1).expect("a governed catalog name is a valid identity"),
    )
}

/// Builds one catalog-validated scalar-arithmetic subject.
///
/// Delegates to the production `ScalarArithmetic::new`, so an admitted pair here
/// is one the governed built-in scalar catalog proved rather than one this spike
/// asserted.
///
/// # Errors
///
/// Returns `None` when the production validator refuses the pair.
#[must_use]
pub fn subject(arithmetic: ArithmeticType, value_type: &str) -> Option<ScalarArithmeticSubject> {
    let resolved = governed_scalar(value_type);
    let checked = ScalarArithmetic::new(arithmetic, resolved).ok()?;
    Some(ScalarArithmeticSubject::new(
        arithmetic,
        checked.resolved_type().clone(),
    ))
}

/// The governed `f32` subject, built through the production constructor.
#[must_use]
pub fn f32_subject() -> ScalarArithmeticSubject {
    let checked = ScalarArithmetic::f32();
    ScalarArithmeticSubject::new(ArithmeticType::F32, checked.resolved_type().clone())
}

/// The governed `f16` subject, built through the production constructor.
///
/// # Panics
///
/// Panics if the catalog stops admitting `f16` arithmetic over `tiler::f16@1`,
/// which would be a catalog regression rather than a fixture one.
#[must_use]
pub fn f16_subject() -> ScalarArithmeticSubject {
    subject(ArithmeticType::F16, "f16").expect("the catalog registers f16 over tiler::f16@1")
}

/// The fixture target profile every record below is attributed to.
///
/// # Panics
///
/// Panics only if the governed key or descriptor grammar rejects a constant.
#[must_use]
pub fn profile() -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new("tiler.spike.delivered-realization.v1")
            .expect("a governed profile key"),
        descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02, 0x03])
            .expect("descriptor bytes"),
    }
}

/// A second profile, differing only in its key.
///
/// # Panics
///
/// Panics only if the governed key or descriptor grammar rejects a constant.
#[must_use]
pub fn other_profile() -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new("tiler.spike.delivered-realization.v2")
            .expect("a governed profile key"),
        descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02, 0x03])
            .expect("descriptor bytes"),
    }
}

/// A complete, structurally valid measured provenance statement.
#[must_use]
pub fn measured_source(compiler_version: &str, platform_build: &str) -> FactSourceProvenance {
    FactSourceProvenance::new(
        AvailabilityPhase::CompileProfile,
        FactAuthority::MeasuredProfile,
        FactValidityScope::MeasuredEnvironment,
        ProvenanceIdentity::new("tiler.spike.measured-authority.v1", 1),
        FactEvidenceBasis::Measurement {
            contexts: vec![MeasurementContext::new(
                vec![CompilerBuildIdentity::new(
                    CompilerBuildRole::CodeGenerator,
                    "spike-offline-compiler",
                    compiler_version,
                    None,
                )],
                ExecutionEnvironmentIdentity::new(
                    "spike-platform",
                    "1.0",
                    platform_build,
                    "spike-architecture",
                    "spike-hardware",
                ),
            )],
        },
    )
}

/// A governed-guarantee provenance statement.
#[must_use]
pub fn governed_source() -> FactSourceProvenance {
    FactSourceProvenance::new(
        AvailabilityPhase::CompileProfile,
        FactAuthority::GovernedProfile,
        FactValidityScope::PortableProfile,
        ProvenanceIdentity::new("tiler.spike.governed-authority.v1", 1),
        FactEvidenceBasis::GovernedGuarantee {
            guarantee: ProvenanceIdentity::new("tiler.spike.governed-guarantee.v1", 1),
        },
    )
}

/// The eleven strict resolutions, in canonical dimension order.
///
/// A dense array literal of exactly [`DIMENSION_COUNT`] entries: widening the
/// vocabulary is a build error here, which is the completeness property the
/// eleven named fields were eliminated in favour of.
#[must_use]
pub fn strict_resolutions() -> [DimensionBehaviour; DIMENSION_COUNT] {
    [
        DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden),
        DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
        DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
        DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven),
    ]
}

/// The eleven resolutions of a flushing contract, in canonical dimension order.
#[must_use]
pub fn flushing_resolutions() -> [DimensionBehaviour; DIMENSION_COUNT] {
    let mut resolutions = strict_resolutions();
    let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    });
    resolutions[NumericalDimension::InputSubnormals.index()] = flush;
    resolutions[NumericalDimension::ResultSubnormals.index()] = flush;
    resolutions
}

/// The scheduled realization matching [`strict_resolutions`] over the eight
/// dimensions a region carries.
#[must_use]
pub const fn strict_realization() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.spike.contract.v1",
        0x7fc0_0000,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

/// One evidence declaration honouring `behaviour` exactly.
#[must_use]
pub fn exact_evidence(behaviour: DimensionBehaviour) -> TargetEvidenceDeclaration {
    TargetEvidenceDeclaration {
        declared: behaviour,
        means: HonouringMeans::SupportedExactly,
        profile: profile(),
        source: measured_source("1.0", "spike-build-a"),
    }
}

/// One evidence declaration honouring `behaviour` only under a named relaxation.
///
/// The `relaxed_on` dimension and `relaxed_to` behaviour are the payload two
/// otherwise identical conditional means differ in — the exact distinction
/// `HonouringMeans::key` erases and this record preserves.
#[must_use]
pub fn relaxation_evidence(
    behaviour: DimensionBehaviour,
    relaxed_on: NumericalDimension,
    relaxed_to: DimensionBehaviour,
) -> TargetEvidenceDeclaration {
    TargetEvidenceDeclaration {
        declared: behaviour,
        means: HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
            relaxation: RelaxationRequirement::new(
                f32_subject().identity(),
                relaxed_on,
                relaxed_to,
            ),
        },
        profile: profile(),
        source: governed_source(),
    }
}

/// One obligation key at the computation locus of an occurrence.
#[must_use]
pub const fn computation(occurrence: u32) -> NumericalObligationKey {
    NumericalObligationKey::new(
        SemanticOccurrence::new(occurrence),
        PolicyLocus::Computation,
    )
}

/// One obligation key at the accumulator locus of an occurrence.
#[must_use]
pub const fn accumulator(occurrence: u32) -> NumericalObligationKey {
    NumericalObligationKey::new(
        SemanticOccurrence::new(occurrence),
        PolicyLocus::Accumulator,
    )
}

/// One obligation key at an ordered component of an occurrence's compound value.
#[must_use]
pub const fn component(occurrence: u32, ordinal: u32) -> NumericalObligationKey {
    NumericalObligationKey::component(SemanticOccurrence::new(occurrence), ordinal)
}

/// One obligation key at the observable materialization locus of an occurrence.
#[must_use]
pub const fn materialization(occurrence: u32) -> NumericalObligationKey {
    NumericalObligationKey::new(
        SemanticOccurrence::new(occurrence),
        PolicyLocus::Materialization,
    )
}

/// Every recognized value type that must **not** manufacture a policy subject.
///
/// A boolean, an integer, a complex value, a decimal, a strict-affine encoded
/// value, and an MX scheme identity. Recognition and arithmetic-subject
/// calibration are separate facts, and the production validator is what
/// separates them: each pairing below is offered to `ScalarArithmetic::new` and
/// each must be refused.
///
/// # Panics
///
/// Panics only if a governed catalog constructor rejects its own constant.
#[must_use]
pub fn non_subject_value_types() -> Vec<(&'static str, ResolvedValueType)> {
    let mut rows = vec![
        ("bool", governed_scalar("bool")),
        ("integer i32", governed_scalar("i32")),
        ("decimal64", governed_scalar("decimal64")),
        (
            "complex over f32",
            complex_value_type(&governed_scalar("f32")).expect("a governed complex identity"),
        ),
        ("strict-affine u4", StrictAffineU4::resolved_type()),
    ];
    // The MX element formats are governed scheme identities rather than nominal
    // scalars, so the row names the element format a scheme is built over: what
    // must be refused is that recognizing an MX identity calibrates an arithmetic
    // subject for it.
    if microscaling_scheme_keys().is_empty() {
        // Named rather than silently skipped: a population that shrank to zero
        // would otherwise make this fixture pass by covering nothing, which is
        // the uniform-pass signature `AGENTS.md` singles out.
        rows.push(("mx element f8e4m3fn", governed_scalar("f8e4m3fn")));
    } else {
        rows.push(("mx element f8e4m3fn", governed_scalar("f8e4m3fn")));
        rows.push(("mx element f6e2m3fn", governed_scalar("f6e2m3fn")));
    }
    rows
}

/// One owner-namespaced type no governed catalog registers.
///
/// # Panics
///
/// Panics only if the identity grammar rejects an owner namespace, which would
/// contradict `validate_component`'s own rule.
#[must_use]
pub fn owner_namespaced_type() -> ResolvedValueType {
    ResolvedValueType::nominal(
        TypeKey::new("acme", "posit16", 1).expect("an owner namespace is a valid identity"),
    )
}
