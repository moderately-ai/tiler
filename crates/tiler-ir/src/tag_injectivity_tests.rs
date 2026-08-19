//! Exhaustive checks for public governed tag tables not already reached by an
//! exhaustive whole-encoder test.

use std::mem::variant_count;

use crate::exhaustive_injectivity::{
    assert_tag_table, assert_tag_table_ref, assert_tag_table_with_inverse,
};
use crate::numerics::{
    CompilerBuildRole, FactAuthority, FactEvidenceBasis, FactValidityScope, HonouringMeans,
    NumericalDimension, PolicyLocus, ProvenanceIdentity, RelaxationRequirement,
    ScalarArithmeticSubject,
};
use crate::program::abi::{AbiBinaryOp, AbiUnaryOp, AvailabilityPhase};
use crate::schedule::{
    ArithmeticType, ContributorArrival, ConvergenceEvidence, LocalCoordinateSource, ReductionPass,
    StagedElement, SynchronizationPlacement,
};

#[test]
fn every_public_governed_tag_table_is_injective_over_its_variant_set() {
    const DIMENSIONS: [NumericalDimension; variant_count::<NumericalDimension>()] =
        crate::numerics::CANONICAL_DIMENSIONS;
    const LOCI: [PolicyLocus; variant_count::<PolicyLocus>()] = [
        PolicyLocus::Input,
        PolicyLocus::Computation,
        PolicyLocus::Accumulator,
        PolicyLocus::Result,
        PolicyLocus::Component,
        PolicyLocus::Materialization,
    ];
    const AUTHORITIES: [FactAuthority; variant_count::<FactAuthority>()] = [
        FactAuthority::GovernedProfile,
        FactAuthority::ExternalProfile,
        FactAuthority::MeasuredProfile,
        FactAuthority::ArtifactEvidence,
        FactAuthority::DeviceRuntime,
        FactAuthority::PreparedKernel,
        FactAuthority::LaunchInstance,
    ];
    const VALIDITY_SCOPES: [FactValidityScope; variant_count::<FactValidityScope>()] = [
        FactValidityScope::PortableProfile,
        FactValidityScope::MeasuredEnvironment,
        FactValidityScope::DeviceInstance,
        FactValidityScope::PreparedArtifact,
        FactValidityScope::LaunchInstance,
    ];
    const PHASES: [AvailabilityPhase; variant_count::<AvailabilityPhase>()] = [
        AvailabilityPhase::CompileProfile,
        AvailabilityPhase::ArtifactEvidence,
        AvailabilityPhase::LiveDevicePreflight,
        AvailabilityPhase::PreparedKernelPreflight,
        AvailabilityPhase::LaunchPreflight,
    ];
    const UNARY_OPS: [AbiUnaryOp; variant_count::<AbiUnaryOp>()] = [
        AbiUnaryOp::Not,
        AbiUnaryOp::NarrowU16,
        AbiUnaryOp::NarrowU32,
    ];
    const BINARY_OPS: [AbiBinaryOp; variant_count::<AbiBinaryOp>()] = [
        AbiBinaryOp::CheckedAdd,
        AbiBinaryOp::CheckedSubtract,
        AbiBinaryOp::CheckedMultiply,
        AbiBinaryOp::Minimum,
        AbiBinaryOp::Maximum,
        AbiBinaryOp::FloorDivide,
        AbiBinaryOp::CeilingDivide,
        AbiBinaryOp::ExactDivide,
        AbiBinaryOp::IsMultipleOf,
        AbiBinaryOp::Equal,
        AbiBinaryOp::LessOrEqual,
        AbiBinaryOp::And,
        AbiBinaryOp::Or,
    ];
    const ARITHMETIC_TYPES: [ArithmeticType; variant_count::<ArithmeticType>()] = [
        ArithmeticType::F16,
        ArithmeticType::Bf16,
        ArithmeticType::F32,
        ArithmeticType::F64,
    ];
    const ARRIVALS: [ContributorArrival; variant_count::<ContributorArrival>()] = [
        ContributorArrival::AscendingParticipant,
        ContributorArrival::NondeterministicArrival,
        ContributorArrival::AtomicAccumulation,
    ];
    const STAGED_ELEMENTS: [StagedElement; variant_count::<StagedElement>()] = [StagedElement::F32];
    const COORDINATES: [LocalCoordinateSource; variant_count::<LocalCoordinateSource>()] = [
        LocalCoordinateSource::LocalLinearInvocation,
        LocalCoordinateSource::LocalWorkgroupPosition,
    ];
    const PASSES: [ReductionPass; variant_count::<ReductionPass>()] =
        [ReductionPass::Partial, ReductionPass::Final];
    const PLACEMENTS: [SynchronizationPlacement; variant_count::<SynchronizationPlacement>()] = [
        SynchronizationPlacement::PhaseBoundary {
            preceding: crate::schedule::PhaseId::new(0),
            following: crate::schedule::PhaseId::new(1),
        },
        SynchronizationPlacement::RoundBoundary,
    ];
    const CONVERGENCE: [ConvergenceEvidence; variant_count::<ConvergenceEvidence>()] = [
        ConvergenceEvidence::EveryParticipantReachesThePoint,
        ConvergenceEvidence::EveryParticipantExecutesEveryRound,
        ConvergenceEvidence::CallerAsserted,
    ];

    assert_tag_table_with_inverse(
        "NumericalDimension::tag",
        &DIMENSIONS,
        NumericalDimension::tag,
        NumericalDimension::from_tag,
    );
    assert_tag_table_with_inverse(
        "PolicyLocus::tag",
        &LOCI,
        PolicyLocus::tag,
        PolicyLocus::from_tag,
    );
    assert_tag_table_with_inverse(
        "FactAuthority::tag",
        &AUTHORITIES,
        FactAuthority::tag,
        FactAuthority::from_tag,
    );
    assert_tag_table_with_inverse(
        "FactValidityScope::tag",
        &VALIDITY_SCOPES,
        FactValidityScope::tag,
        FactValidityScope::from_tag,
    );
    assert_tag_table_with_inverse(
        "AvailabilityPhase::tag",
        &PHASES,
        AvailabilityPhase::tag,
        AvailabilityPhase::from_tag,
    );
    assert_tag_table_with_inverse(
        "AbiUnaryOp::tag",
        &UNARY_OPS,
        AbiUnaryOp::tag,
        AbiUnaryOp::from_tag,
    );
    assert_tag_table_with_inverse(
        "AbiBinaryOp::tag",
        &BINARY_OPS,
        AbiBinaryOp::tag,
        AbiBinaryOp::from_tag,
    );
    assert_tag_table_with_inverse(
        "ArithmeticType::tag",
        &ARITHMETIC_TYPES,
        ArithmeticType::tag,
        ArithmeticType::from_tag,
    );
    assert_tag_table(
        "ContributorArrival::tag",
        &ARRIVALS,
        ContributorArrival::tag,
    );
    assert_tag_table("StagedElement::tag", &STAGED_ELEMENTS, StagedElement::tag);
    assert_tag_table(
        "LocalCoordinateSource::tag",
        &COORDINATES,
        LocalCoordinateSource::tag,
    );
    assert_tag_table("ReductionPass::tag", &PASSES, ReductionPass::tag);
    assert_tag_table(
        "SynchronizationPlacement::tag",
        &PLACEMENTS,
        SynchronizationPlacement::tag,
    );
    assert_tag_table(
        "ConvergenceEvidence::tag",
        &CONVERGENCE,
        ConvergenceEvidence::tag,
    );

    let subject = ScalarArithmeticSubject::f32().identity();
    let means: [HonouringMeans; variant_count::<HonouringMeans>()] = [
        HonouringMeans::SupportedExactly,
        HonouringMeans::SupportedWithExactEmulation,
        HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
            relaxation: RelaxationRequirement::new(
                subject,
                NumericalDimension::Contraction,
                crate::numerics::DimensionBehaviour::Transform(
                    crate::schedule::NumericalPermission::Permitted,
                ),
            ),
        },
        HonouringMeans::Unsupported,
    ];
    assert_tag_table_ref("HonouringMeans::tag", &means, HonouringMeans::tag);

    let roles: [CompilerBuildRole; variant_count::<CompilerBuildRole>()] = [
        CompilerBuildRole::Frontend,
        CompilerBuildRole::Optimizer,
        CompilerBuildRole::IntermediateTranslator,
        CompilerBuildRole::CodeGenerator,
        CompilerBuildRole::Assembler,
        CompilerBuildRole::Linker,
        CompilerBuildRole::RuntimeCompiler,
        CompilerBuildRole::ProviderDefined(ProvenanceIdentity::new("provider", 1)),
    ];
    assert_tag_table_ref("CompilerBuildRole::tag", &roles, CompilerBuildRole::tag);

    let bases: [FactEvidenceBasis; variant_count::<FactEvidenceBasis>()] = [
        FactEvidenceBasis::GovernedGuarantee {
            guarantee: ProvenanceIdentity::new("governed", 1),
        },
        FactEvidenceBasis::ExternalGuarantee {
            reference: ProvenanceIdentity::new("external", 1),
        },
        FactEvidenceBasis::Measurement {
            contexts: Vec::new(),
        },
        FactEvidenceBasis::CompileProfileMeasurement {
            contexts: Vec::new(),
        },
    ];
    assert_tag_table_ref("FactEvidenceBasis::tag", &bases, FactEvidenceBasis::tag);
}
