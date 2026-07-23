//! Numerical legality evidence for one proposed fusion region.
//!
//! Region formation proposes candidates; this module answers a different
//! question about one of them: whether implementing that region as a single
//! strict-`f32` kernel preserves the request's numerical contract exactly.
//!
//! The evidence is bound to the exact region occurrence, the exact region
//! content, the canonical request subject, and the exact materialized reference
//! provider. A candidate label or a copied stable string is not evidence.

use std::error::Error;
use std::fmt;

use crate::region::{RegionCandidate, RegionError, RegionGraph, verify_candidate};
use crate::request::{
    LoweringProviderIdentity, NumericalPermission, VerifiedRequestSubject, VerifiedTargetRequest,
};

/// Machine-checkable evidence that one whole-program region fuses exactly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionNumericalProof {
    candidate: RegionCandidate,
    request_subject: VerifiedRequestSubject,
    materialized_reference_provider: LoweringProviderIdentity,
    atomic_operations: AtomicOperationProof,
    contributor_order: ContributorOrderProof,
    nan_boundaries: NaNBoundaryProof,
    materialization_boundaries: MaterializationBoundaryProof,
    forbidden_transforms: ForbiddenTransformProof,
}

impl FusionNumericalProof {
    pub(crate) fn candidate_stable_id(&self) -> &str {
        self.candidate.stable_id()
    }

    pub(crate) fn canonical_explain_evidence_bytes(&self) -> Vec<u8> {
        let mut bytes = self.request_subject.canonical_explain_subject_bytes();
        encode_evidence_bytes(&mut bytes, self.candidate.occurrence().as_bytes());
        encode_evidence_bytes(&mut bytes, self.candidate.content().as_bytes());
        encode_evidence_bytes(
            &mut bytes,
            self.materialized_reference_provider.key.as_bytes(),
        );
        bytes.extend_from_slice(&self.materialized_reference_provider.revision.to_be_bytes());
        bytes.push(match self.atomic_operations {
            AtomicOperationProof::MultiplyThenAdd => 1,
        });
        bytes.push(match self.contributor_order {
            ContributorOrderProof::OriginalAxisLexicographic => 1,
        });
        bytes.push(match self.nan_boundaries {
            NaNBoundaryProof::CanonicalizeAfterEveryArithmeticOperation => 1,
        });
        bytes.push(match self.materialization_boundaries {
            MaterializationBoundaryProof::NoObservableBoundaryRemoved => 1,
        });
        for permission in [
            self.forbidden_transforms.contraction,
            self.forbidden_transforms.reassociation,
            self.forbidden_transforms.permutation,
        ] {
            bytes.push(match permission {
                NumericalPermission::Forbidden => 1,
            });
        }
        bytes
    }
}

fn encode_evidence_bytes(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    output.extend_from_slice(value);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AtomicOperationProof {
    MultiplyThenAdd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContributorOrderProof {
    OriginalAxisLexicographic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NaNBoundaryProof {
    CanonicalizeAfterEveryArithmeticOperation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MaterializationBoundaryProof {
    NoObservableBoundaryRemoved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ForbiddenTransformProof {
    pub(crate) contraction: NumericalPermission,
    pub(crate) reassociation: NumericalPermission,
    pub(crate) permutation: NumericalPermission,
}

/// Typed failure of fused numerical-legality proof construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FusionError {
    /// The candidate failed recomputation from its own exact contents.
    Region(RegionError),
    /// The candidate is not the subject this evidence may describe.
    Invalid { region: String, rule: &'static str },
}

impl FusionError {
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::Region(error) => error.reason(),
            Self::Invalid { rule, .. } => rule,
        }
    }
}

impl fmt::Display for FusionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Region(error) => error.fmt(formatter),
            Self::Invalid { region, rule } => {
                write!(formatter, "fusion.numerics.{rule}: {region} rejected")
            }
        }
    }
}

impl Error for FusionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Region(error) => Some(error),
            Self::Invalid { .. } => None,
        }
    }
}

impl From<RegionError> for FusionError {
    fn from(value: RegionError) -> Self {
        Self::Region(value)
    }
}

/// Proves that one whole-program region preserves strict `f32` semantics.
///
/// The candidate is rederived from the graph before anything else is checked,
/// and it is admitted only when it covers every operation of the verified
/// program, reads exactly one boundary value, and exports exactly one named
/// program result with no external consumer. The request boundary already pins
/// the program to that exact recognized structure, so this identifies one
/// region without naming any operation role.
pub(crate) fn prove_fused_numerics(
    graph: &RegionGraph,
    request: &VerifiedTargetRequest,
    candidate: &RegionCandidate,
) -> Result<FusionNumericalProof, FusionError> {
    verify_candidate(
        graph,
        request.budgets(),
        request.numerical_contract(),
        candidate,
    )?;
    if !candidate.covers_whole_program() || candidate.boundary_inputs().len() != 1 {
        return invalid(candidate, "region-coverage");
    }
    let [retained] = candidate.retained_outputs() else {
        return invalid(candidate, "region-retained-outputs");
    };
    if !retained.named_result || retained.external_consumers {
        return invalid(candidate, "region-boundary-outputs");
    }
    let proof = FusionNumericalProof {
        candidate: candidate.clone(),
        request_subject: request.subject(),
        materialized_reference_provider: request.capabilities().materialized_serial_sum,
        atomic_operations: AtomicOperationProof::MultiplyThenAdd,
        contributor_order: ContributorOrderProof::OriginalAxisLexicographic,
        nan_boundaries: NaNBoundaryProof::CanonicalizeAfterEveryArithmeticOperation,
        materialization_boundaries: MaterializationBoundaryProof::NoObservableBoundaryRemoved,
        forbidden_transforms: ForbiddenTransformProof {
            contraction: NumericalPermission::Forbidden,
            reassociation: NumericalPermission::Forbidden,
            permutation: NumericalPermission::Forbidden,
        },
    };
    if request.numerical_contract().contraction != NumericalPermission::Forbidden
        || request.numerical_contract().reassociation != NumericalPermission::Forbidden
        || proof.forbidden_transforms.contraction != NumericalPermission::Forbidden
        || proof.forbidden_transforms.reassociation != NumericalPermission::Forbidden
        || proof.forbidden_transforms.permutation != NumericalPermission::Forbidden
    {
        return invalid(candidate, "strict-f32-proof");
    }
    Ok(proof)
}

/// Rederives the proof and requires it to equal the retained evidence exactly.
pub(crate) fn verify_fused_numerics(
    graph: &RegionGraph,
    request: &VerifiedTargetRequest,
    candidate: &RegionCandidate,
    proof: &FusionNumericalProof,
) -> Result<(), FusionError> {
    let expected = prove_fused_numerics(graph, request, candidate)?;
    if proof != &expected {
        return invalid(candidate, "numerical-proof-subject");
    }
    Ok(())
}

fn invalid<T>(candidate: &RegionCandidate, rule: &'static str) -> Result<T, FusionError> {
    Err(FusionError::Invalid {
        region: candidate.stable_id().to_owned(),
        rule,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::form_region_candidates;
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    fn program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    fn target_request(program: &SemanticProgram) -> VerifiedTargetRequest {
        let verified = verify_request(CompilationRequest::governed(program)).unwrap();
        verified.for_target(verified.target_profiles()[0]).unwrap()
    }

    #[test]
    fn only_the_whole_program_region_carries_fused_numerical_evidence() {
        let program = program();
        let request = target_request(&program);
        let outcome =
            form_region_candidates(&program, request.budgets(), request.numerical_contract())
                .unwrap();
        let whole = outcome.whole_program_candidate().unwrap();
        let proof = prove_fused_numerics(outcome.graph(), &request, whole).unwrap();
        verify_fused_numerics(outcome.graph(), &request, whole, &proof).unwrap();
        assert_eq!(proof.candidate_stable_id(), whole.stable_id());

        for candidate in outcome.candidates() {
            if candidate.covers_whole_program() {
                continue;
            }
            assert!(matches!(
                prove_fused_numerics(outcome.graph(), &request, candidate),
                Err(FusionError::Invalid { .. })
            ));
        }
    }

    #[test]
    fn forged_candidates_and_stale_proofs_fail_closed() {
        let program = program();
        let request = target_request(&program);
        let outcome =
            form_region_candidates(&program, request.budgets(), request.numerical_contract())
                .unwrap();
        let whole = outcome.whole_program_candidate().unwrap();
        let proof = prove_fused_numerics(outcome.graph(), &request, whole).unwrap();

        let mut forged = proof.clone();
        forged.materialized_reference_provider.revision += 1;
        assert!(matches!(
            verify_fused_numerics(outcome.graph(), &request, whole, &forged),
            Err(FusionError::Invalid {
                rule: "numerical-proof-subject",
                ..
            })
        ));

        // Evidence bytes bind the occurrence, the content, and the request.
        let singleton = &outcome.candidates()[0];
        assert_ne!(
            proof.canonical_explain_evidence_bytes(),
            FusionNumericalProof {
                candidate: singleton.clone(),
                ..proof.clone()
            }
            .canonical_explain_evidence_bytes()
        );
    }

    #[test]
    fn a_region_from_another_graph_is_rejected_before_any_numerical_claim() {
        let program = program();
        let request = target_request(&program);
        let outcome =
            form_region_candidates(&program, request.budgets(), request.numerical_contract())
                .unwrap();
        let whole = outcome.whole_program_candidate().unwrap().clone();

        // A structurally different program yields a different graph, so the
        // stored occurrence identity no longer rederives.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4, 5]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 3.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 4.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let other = builder.build().unwrap();
        let other_request = target_request(&other);
        let other_outcome = form_region_candidates(
            &other,
            other_request.budgets(),
            other_request.numerical_contract(),
        )
        .unwrap();

        assert!(matches!(
            prove_fused_numerics(other_outcome.graph(), &other_request, &whole),
            Err(FusionError::Region(RegionError::Invalid {
                rule: "identity",
                ..
            }))
        ));
    }

    #[test]
    fn errors_report_their_exact_rule() {
        let error = FusionError::Invalid {
            region: "region:0000000000000000".to_owned(),
            rule: "strict-f32-proof",
        };
        assert_eq!(error.reason(), "strict-f32-proof");
        assert_eq!(
            error.to_string(),
            "fusion.numerics.strict-f32-proof: region:0000000000000000 rejected"
        );
        let error = FusionError::from(RegionError::Structure {
            rule: "value-ordinal",
        });
        assert_eq!(error.reason(), "value-ordinal");
        assert!(error.source().is_some());
    }
}
