use super::support::{admitted_count, semantic, test_root};
use super::*;

#[test]
fn intrinsic_physical_failures_are_invalid_output_not_empty_frontiers() {
    let error = CompileError::from(PhysicalError::Intrinsic {
        rule: "forged",
        region: RegionId::new(0),
    });
    assert!(matches!(
        error,
        CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
            PhysicalError::Intrinsic { .. }
        ))
    ));
}

struct UnregisteredOpaqueProvider {
    identity: tiler_ir::semantic::ProviderIdentity,
    call: crate::call_registry::OpaqueCallIdentity,
    bindings: Vec<(&'static str, AccessOrdinal)>,
}

impl PhysicalImplementationProvider for UnregisteredOpaqueProvider {
    fn provenance(
        &self,
    ) -> Result<
        crate::frontier::PhysicalProviderProvenance,
        crate::frontier::PhysicalProviderProvenanceError,
    > {
        crate::frontier::PhysicalProviderProvenance::new(self.identity.clone())
    }

    fn propose(
        &self,
        context: &crate::frontier::ImplementationContext<'_>,
    ) -> crate::frontier::ProviderOffer {
        crate::frontier::ProviderOffer::proposing(vec![
            crate::frontier::ImplementationProposal::new(
                crate::frontier::ProposalBody::OpaqueCall(Box::new(
                    crate::call_registry::OpaqueCallProposal::new(self.call, self.bindings.clone())
                        .expect("fixture proposal is exactly reportable"),
                )),
                crate::frontier::TargetApplicability::for_targets([context
                    .request()
                    .target_profile()
                    .profile_key()
                    .clone()]),
                crate::frontier::PhysicalCostEstimate::structural(1, 2, 0),
            ),
        ])
    }
}

fn mixed_frontier_trace(
    provider_revision: u32,
    call_revision: u32,
    reverse_providers: bool,
    reverse_bindings: bool,
) -> VerifiedExplainTrace {
    let semantic = semantic(false);
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let subject = FrontierRegionSubject::new(
        "fused",
        request.serial_sum().members.all(),
        crate::physical::RegionWrite::ProgramOutput,
    );
    let governed = GovernedPhysicalProvider;
    let opaque = UnregisteredOpaqueProvider {
        identity: tiler_ir::semantic::ProviderIdentity::new(
            "tiler.test.physical",
            "opaque",
            provider_revision,
        )
        .unwrap(),
        call: crate::call_registry::OpaqueCallIdentity::new("call-owner", "mystery", call_revision)
            .unwrap(),
        bindings: if reverse_bindings {
            vec![
                ("output", AccessOrdinal::new(1)),
                ("input", AccessOrdinal::FIRST),
            ]
        } else {
            vec![
                ("input", AccessOrdinal::FIRST),
                ("output", AccessOrdinal::new(1)),
            ]
        },
    };
    let providers: Vec<&dyn PhysicalImplementationProvider> = if reverse_providers {
        vec![&opaque, &governed]
    } else {
        vec![&governed, &opaque]
    };
    let frontier = enumerate_frontier(
        &request,
        &subject,
        &providers,
        &crate::call_registry::OpaqueCallRegistry::new(),
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .unwrap();
    assert_eq!(frontier.admitted().len(), 1);
    assert_eq!(frontier.rejections().len(), 1);

    let mut explain = ExplainWriter::new(&request).unwrap();
    let root = test_root(&mut explain);
    let cause = record_frontier(&mut explain, "region:fused", "fused", &frontier, root).unwrap();
    let alternative = explain
        .subject(SubjectKind::Alternative, "alternative:test")
        .unwrap();
    explain
        .note_selection(
            alternative,
            SelectionOutcome::Selected,
            Some(TerminalCause::from_record(cause)),
        )
        .unwrap();
    explain
        .finish_success(&["alternative:test"], "alternative:test")
        .unwrap()
}

#[test]
fn mixed_frontier_records_exact_opaque_call_rejection_detail() {
    let trace = mixed_frontier_trace(7, 3, false, false);
    let rejection = trace
        .records()
        .iter()
        .find(|record| record.rule().key().as_str() == "opaque-call.registration.v1")
        .expect("one unregistered-call detail");
    assert!(matches!(
        rejection.event(),
        ExplainEvent::Check {
            stage: ExplainStage::CapabilityResolution,
            assessment,
            rejection: RejectionClass::IntrinsicInvalid,
        } if assessment.predicate().as_str() == "opaque-call.registered"
            && assessment.reason().is_some_and(|reason| reason.as_str() == "opaque-call.unregistered")
    ));
    assert_eq!(
        rejection.event().disposition(),
        ExplainDisposition::RejectedIntrinsic
    );
    let subjects = rejection
        .subjects()
        .iter()
        .map(|subject| (subject.kind(), subject.key().as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        subjects,
        [
            (
                SubjectKind::OpaqueCall,
                "call-owner/mystery@3[input=access#0,output=access#1]",
            ),
            (SubjectKind::Provider, "tiler.test.physical::opaque@7"),
        ]
    );
    assert!(
        trace
            .records()
            .iter()
            .all(|record| !matches!(record.event(), ExplainEvent::CostAssessment { .. })),
        "a local rejection is never cost evidence"
    );
    let rendered = trace.render();
    assert!(rendered.starts_with("tiler-explain-v10 "));
    assert!(rendered.contains("opaque-call:call-owner/mystery@3[input=access#0,output=access#1]"));
    assert!(rendered.contains("provider:tiler.test.physical::opaque@7"));
    assert!(rendered.contains("admitted-count:count=1"));
    assert!(rendered.contains("rejected-count:count=1"));
}

#[test]
fn opaque_call_trace_identity_is_order_independent_and_identity_sensitive() {
    let forward = mixed_frontier_trace(7, 3, false, false);
    let reversed = mixed_frontier_trace(7, 3, true, false);
    assert_eq!(forward.identity(), reversed.identity());
    assert_ne!(
        forward.identity(),
        mixed_frontier_trace(7, 4, false, false).identity()
    );
    assert_ne!(
        forward.identity(),
        mixed_frontier_trace(8, 3, false, false).identity()
    );
    assert_ne!(
        forward.identity(),
        mixed_frontier_trace(7, 3, false, true).identity(),
        "ordered named bindings were absent from explain identity"
    );
}

// ---------------------------------------------------------------------------
// Opaque calls on the compile path
// ---------------------------------------------------------------------------

/// A provider offering one opaque call for the whole-program region only.
///
/// It gates on the region subject rather than proposing everywhere, so a
/// compilation using it differs from the governed one by exactly one
/// implementation: the whole-program region is one the governed provider already
/// implements, which makes the call an *alternative* to a checked scheduled body
/// rather than the only implementation of a region nothing else covers.
struct WholeProgramCallProvider {
    call: crate::call_registry::OpaqueCallIdentity,
}

impl WholeProgramCallProvider {
    /// The call's ABI parameters bound to this region's tensor roles.
    ///
    /// Stated by the provider and never inferred: the ABI says a parameter is
    /// read or written and never which tensor it reads, so the claim is the
    /// provider's and the frontier checks it against the declaration.
    fn bindings() -> Vec<(&'static str, AccessOrdinal)> {
        vec![("x", AccessOrdinal::FIRST), ("y", AccessOrdinal::new(1))]
    }
}

impl PhysicalImplementationProvider for WholeProgramCallProvider {
    fn provenance(
        &self,
    ) -> Result<
        crate::frontier::PhysicalProviderProvenance,
        crate::frontier::PhysicalProviderProvenanceError,
    > {
        crate::frontier::PhysicalProviderProvenance::new(
            tiler_ir::semantic::ProviderIdentity::new(
                "tiler.test.physical",
                "whole-program-call",
                1,
            )
            .expect("the fixture provider identity is valid"),
        )
    }

    fn propose(
        &self,
        context: &crate::frontier::ImplementationContext<'_>,
    ) -> crate::frontier::ProviderOffer {
        if context.subject().role() != "whole-program" {
            return crate::frontier::ProviderOffer::default();
        }
        crate::frontier::ProviderOffer::proposing(vec![
            crate::frontier::ImplementationProposal::new(
                crate::frontier::ProposalBody::OpaqueCall(Box::new(
                    crate::call_registry::OpaqueCallProposal::new(self.call, Self::bindings())
                        .expect("fixture proposal is exactly reportable"),
                )),
                crate::frontier::TargetApplicability::for_targets([context
                    .request()
                    .target_profile()
                    .profile_key()
                    .clone()]),
                crate::frontier::PhysicalCostEstimate::structural(1, 2, 0),
            ),
        ])
    }
}

/// The governed authorities plus one opaque-call provider, with or without the
/// declaration that provider's proposal names.
///
/// The two compositions differ in exactly one registration, which is what makes
/// either case evidence about the registry rather than about the provider.
fn opaque_call_authorities<'a>(
    governed: &'a GovernedPhysicalProvider,
    opaque: &'a WholeProgramCallProvider,
    register: bool,
) -> PhysicalAuthorities<'a> {
    let mut calls = crate::call_registry::OpaqueCallRegistry::new();
    if register {
        calls
            .register(
                opaque.call,
                crate::selection::opaque_call_declaration_fixture(
                    crate::effects::Aliasing::Distinct,
                ),
            )
            .expect("the fixture registers one call");
    }
    PhysicalAuthorities::composed(vec![governed, opaque], calls)
}

/// The fixture call identity both compile-path cases name.
fn fixture_call_identity() -> crate::call_registry::OpaqueCallIdentity {
    crate::call_registry::OpaqueCallIdentity::new("test-owner", "whole-program-call", 1)
        .expect("the fixture call identity is valid")
}

/// A registered opaque call reaches the compile path and is admitted there.
///
/// Admission is the property; the compilation's *refusal* is how a caller of
/// `compile` observes it. Lowering an opaque call is not implemented, so a
/// retained plan that selects one is refused by name at program assembly — and
/// that refusal is reachable only through an admitted opaque body in a retained
/// plan. Before this wiring, no registry any caller could populate reached
/// `enumerate_frontier` at all, so every test of the admission path was a test
/// of an authority nothing could drive.
///
/// The control is in the case: the identical compilation with the registration
/// removed compiles, and its whole-program frontier admits one implementation
/// instead of two. The refusal is therefore caused by the registration and not
/// by the provider merely being installed.
#[test]
fn a_registered_opaque_call_is_admitted_through_the_compile_path() {
    let semantic = semantic(false);
    let governed = GovernedPhysicalProvider;
    let opaque = WholeProgramCallProvider {
        call: fixture_call_identity(),
    };

    // Matched rather than unwrapped: the success value is a whole compilation
    // product, and printing it would bury the one fact a reader of a failure
    // here needs — that the call never reached a plan.
    let Err(refusal) = compile_configured(
        CompilationRequest::governed(&semantic),
        AlgebraicRuleConfiguration::all(),
        &opaque_call_authorities(&governed, &opaque, true),
    ) else {
        panic!("an admitted opaque call has no lowering; the compilation succeeded");
    };
    let CompileError::Explained { source, explain } = refusal else {
        panic!("a refusal after the trace boundary retains its trace");
    };
    assert!(
        matches!(
            *source,
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "unlowerable-opaque-body"
                }
            ))
        ),
        "the registered call reached a retained plan: {source:?}",
    );
    assert_eq!(
        admitted_count(&explain, "whole-program"),
        Some(2),
        "the registered call was admitted beside the governed scheduled body",
    );

    let unregistered = compile_configured(
        CompilationRequest::governed(&semantic),
        AlgebraicRuleConfiguration::all(),
        &opaque_call_authorities(&governed, &opaque, false),
    )
    .expect("the same compilation without the registration has no opaque plan");
    assert_eq!(
        admitted_count(&unregistered.targets[0].explain, "whole-program"),
        Some(1),
        "removing the registration removes the admission, not merely the plan",
    );
}

/// A call a proposal names and no registry holds is refused as unregistered.
///
/// The refusal belongs to the provider rather than to the target: nothing about
/// this profile made the call infeasible, so the compilation keeps its governed
/// alternatives and records the exact proposal it could not resolve.
#[test]
fn an_unregistered_opaque_call_named_on_the_compile_path_is_refused_by_name() {
    let semantic = semantic(false);
    let governed = GovernedPhysicalProvider;
    let opaque = WholeProgramCallProvider {
        call: fixture_call_identity(),
    };

    let product = compile_configured(
        CompilationRequest::governed(&semantic),
        AlgebraicRuleConfiguration::all(),
        &opaque_call_authorities(&governed, &opaque, false),
    )
    .expect("an unregistered call is a provider fault, not a target refusal");
    let trace = &product.targets[0].explain;
    let rejection = trace
        .records()
        .iter()
        .find(|record| record.rule().key().as_str() == "opaque-call.registration.v1")
        .expect("one unregistered-call refusal");
    assert!(matches!(
        rejection.event(),
        ExplainEvent::Check {
            stage: ExplainStage::CapabilityResolution,
            assessment,
            rejection: RejectionClass::IntrinsicInvalid,
        } if assessment.predicate().as_str() == "opaque-call.registered"
            && assessment
                .reason()
                .is_some_and(|reason| reason.as_str() == "opaque-call.unregistered")
    ));
    assert_eq!(
        rejection
            .subjects()
            .iter()
            .map(|subject| (subject.kind(), subject.key().as_str()))
            .collect::<Vec<_>>(),
        [
            (
                SubjectKind::OpaqueCall,
                "test-owner/whole-program-call@1[x=access#0,y=access#1]",
            ),
            (
                SubjectKind::Provider,
                "tiler.test.physical::whole-program-call@1",
            ),
        ],
        "the exact proposal that could not be resolved is retained",
    );
    assert_eq!(
        product.targets[0].portfolio.alternatives.len(),
        compile(CompilationRequest::governed(&semantic))
            .expect("the governed compilation")
            .targets[0]
            .portfolio
            .alternatives
            .len(),
        "an unregistered proposal removes no governed alternative",
    );
}

// ---------------------------------------------------------------------------
// The multi-pass split: enumerated on the frontier, assembled into a program
// ---------------------------------------------------------------------------
//
// **Why these drive the authorities directly, and what now also reaches
// `compile`.** The split consumes reassociation. Under `governed_relaxed` — for
// a long time the only registered contract permitting it — contraction is
// permitted too, and for the recognized serial-sum program, whose members mix
// multiply and add, `derive_fusion_legality` reports `unrealized-contraction`
// for every multi-member candidate, so no legal cover survives and the whole
// compile has no complete plan. That property is unchanged and still pinned by
// `fusion_legality::tests::a_relaxed_mixed_arithmetic_region_still_needs_contraction_evidence`.
//
// `admit-a-reassociating-contract-without-contraction` closed the gap from the
// contract side rather than the proof side:
// `StrictF32NumericalContract::governed_reassociating` permits reassociation and
// forbids contraction, so the prologue's mixed region discharges its contraction
// obligation under the contract's own normative guarantee and the materialized
// cover survives. `the_reassociating_contract_reaches_the_split_through_compile`
// below is the end-to-end half; these keep exercising the exact authorities at
// the relaxed contract, where the split is still enumerable and assemblable
// without being reachable.
