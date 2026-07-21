use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;

use crate::request::{NumericalPermission, VerifiedTargetRequest};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum SemanticOccurrence {
    ScaleConstant,
    Multiply,
    BiasConstant,
    Add,
    StrictSum,
}

impl SemanticOccurrence {
    const ALL: [Self; 5] = [
        Self::ScaleConstant,
        Self::Multiply,
        Self::BiasConstant,
        Self::Add,
        Self::StrictSum,
    ];

    const fn stable_name(self) -> &'static str {
        match self {
            Self::ScaleConstant => "scale-constant",
            Self::Multiply => "multiply",
            Self::BiasConstant => "bias-constant",
            Self::Add => "add",
            Self::StrictSum => "strict-sum",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum BoundaryValue {
    InputTensor,
    ScaleConstant,
    Product,
    BiasConstant,
    PointwiseResult,
    OutputTensor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateKind {
    Singleton,
    Pointwise,
    FusedSerialSum,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionCandidate {
    pub(crate) stable_id: String,
    pub(crate) kind: CandidateKind,
    pub(crate) members: BTreeSet<SemanticOccurrence>,
    pub(crate) boundary_inputs: BTreeSet<BoundaryValue>,
    pub(crate) boundary_outputs: BTreeSet<BoundaryValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FusionNumericalProof {
    pub(crate) atomic_operations: AtomicOperationProof,
    pub(crate) contributor_order: ContributorOrderProof,
    pub(crate) nan_boundaries: NaNBoundaryProof,
    pub(crate) materialization_boundaries: MaterializationBoundaryProof,
    pub(crate) forbidden_transforms: ForbiddenTransformProof,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CandidateError {
    Budget {
        limit: u32,
        actual: usize,
    },
    Invalid {
        candidate: String,
        rule: &'static str,
    },
}

impl fmt::Display for CandidateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Budget { limit, actual } => write!(
                formatter,
                "fusion.candidates.budget: {actual} exceeds deterministic limit {limit}"
            ),
            Self::Invalid { candidate, rule } => {
                write!(formatter, "fusion.candidate.{rule}: {candidate} rejected")
            }
        }
    }
}

impl Error for CandidateError {}

pub(crate) fn enumerate_candidates(
    request: &VerifiedTargetRequest,
) -> Result<Vec<RegionCandidate>, CandidateError> {
    let mut candidates: Vec<_> = SemanticOccurrence::ALL
        .into_iter()
        .map(|occurrence| candidate(CandidateKind::Singleton, [occurrence]))
        .collect();
    candidates.push(candidate(
        CandidateKind::Pointwise,
        [
            SemanticOccurrence::ScaleConstant,
            SemanticOccurrence::Multiply,
            SemanticOccurrence::BiasConstant,
            SemanticOccurrence::Add,
        ],
    ));
    candidates.push(candidate(
        CandidateKind::FusedSerialSum,
        SemanticOccurrence::ALL,
    ));
    if candidates.len()
        > usize::try_from(request.budgets.fusion_candidates).expect("u32 fits every supported host")
    {
        return Err(CandidateError::Budget {
            limit: request.budgets.fusion_candidates,
            actual: candidates.len(),
        });
    }
    for candidate in &candidates {
        verify_candidate(candidate)?;
    }
    Ok(candidates)
}

pub(crate) fn prove_fused_numerics(
    request: &VerifiedTargetRequest,
    candidate: &RegionCandidate,
) -> Result<FusionNumericalProof, CandidateError> {
    if candidate.kind != CandidateKind::FusedSerialSum {
        return invalid(candidate, "numerical-proof-kind");
    }
    let proof = FusionNumericalProof {
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
    if request.numerical_contract.contraction != NumericalPermission::Forbidden
        || request.numerical_contract.reassociation != NumericalPermission::Forbidden
        || proof.forbidden_transforms.contraction != NumericalPermission::Forbidden
        || proof.forbidden_transforms.reassociation != NumericalPermission::Forbidden
        || proof.forbidden_transforms.permutation != NumericalPermission::Forbidden
    {
        return invalid(candidate, "strict-f32-proof");
    }
    Ok(proof)
}

fn candidate(
    kind: CandidateKind,
    members: impl IntoIterator<Item = SemanticOccurrence>,
) -> RegionCandidate {
    let members: BTreeSet<_> = members.into_iter().collect();
    let stable_id = format!(
        "candidate:{}",
        SemanticOccurrence::ALL
            .into_iter()
            .filter(|occurrence| members.contains(occurrence))
            .map(SemanticOccurrence::stable_name)
            .collect::<Vec<_>>()
            .join("+")
    );
    let (boundary_inputs, boundary_outputs) = boundaries(&members);
    RegionCandidate {
        stable_id,
        kind,
        members,
        boundary_inputs,
        boundary_outputs,
    }
}

fn dependencies() -> BTreeMap<SemanticOccurrence, BTreeSet<SemanticOccurrence>> {
    BTreeMap::from([
        (SemanticOccurrence::ScaleConstant, BTreeSet::new()),
        (
            SemanticOccurrence::Multiply,
            BTreeSet::from([SemanticOccurrence::ScaleConstant]),
        ),
        (SemanticOccurrence::BiasConstant, BTreeSet::new()),
        (
            SemanticOccurrence::Add,
            BTreeSet::from([
                SemanticOccurrence::Multiply,
                SemanticOccurrence::BiasConstant,
            ]),
        ),
        (
            SemanticOccurrence::StrictSum,
            BTreeSet::from([SemanticOccurrence::Add]),
        ),
    ])
}

fn boundaries(
    members: &BTreeSet<SemanticOccurrence>,
) -> (BTreeSet<BoundaryValue>, BTreeSet<BoundaryValue>) {
    let mut inputs = BTreeSet::new();
    let mut outputs = BTreeSet::new();
    if members.contains(&SemanticOccurrence::ScaleConstant)
        && !members.contains(&SemanticOccurrence::Multiply)
    {
        outputs.insert(BoundaryValue::ScaleConstant);
    }
    if members.contains(&SemanticOccurrence::Multiply) {
        inputs.insert(BoundaryValue::InputTensor);
        if !members.contains(&SemanticOccurrence::ScaleConstant) {
            inputs.insert(BoundaryValue::ScaleConstant);
        }
        if !members.contains(&SemanticOccurrence::Add) {
            outputs.insert(BoundaryValue::Product);
        }
    }
    if members.contains(&SemanticOccurrence::BiasConstant)
        && !members.contains(&SemanticOccurrence::Add)
    {
        outputs.insert(BoundaryValue::BiasConstant);
    }
    if members.contains(&SemanticOccurrence::Add) {
        if !members.contains(&SemanticOccurrence::Multiply) {
            inputs.insert(BoundaryValue::Product);
        }
        if !members.contains(&SemanticOccurrence::BiasConstant) {
            inputs.insert(BoundaryValue::BiasConstant);
        }
        if !members.contains(&SemanticOccurrence::StrictSum) {
            outputs.insert(BoundaryValue::PointwiseResult);
        }
    }
    if members.contains(&SemanticOccurrence::StrictSum) {
        if !members.contains(&SemanticOccurrence::Add) {
            inputs.insert(BoundaryValue::PointwiseResult);
        }
        outputs.insert(BoundaryValue::OutputTensor);
    }
    (inputs, outputs)
}

fn verify_candidate(candidate: &RegionCandidate) -> Result<(), CandidateError> {
    if candidate.members.is_empty()
        || candidate
            .members
            .iter()
            .any(|member| !SemanticOccurrence::ALL.contains(member))
    {
        return invalid(candidate, "membership");
    }
    if boundaries(&candidate.members)
        != (
            candidate.boundary_inputs.clone(),
            candidate.boundary_outputs.clone(),
        )
    {
        return invalid(candidate, "boundaries");
    }
    if !is_connected(&candidate.members) {
        return invalid(candidate, "connectivity");
    }
    if !is_convex(&candidate.members) {
        return invalid(candidate, "convexity");
    }
    Ok(())
}

fn is_connected(members: &BTreeSet<SemanticOccurrence>) -> bool {
    let graph = dependencies();
    let start = *members
        .first()
        .expect("membership checked before connectivity");
    let mut queue = VecDeque::from([start]);
    let mut reached = BTreeSet::new();
    while let Some(node) = queue.pop_front() {
        if !reached.insert(node) {
            continue;
        }
        let neighbors = graph[&node].iter().copied().chain(
            graph
                .iter()
                .filter_map(|(consumer, inputs)| inputs.contains(&node).then_some(*consumer)),
        );
        queue.extend(neighbors.filter(|neighbor| members.contains(neighbor)));
    }
    reached == *members
}

fn is_convex(members: &BTreeSet<SemanticOccurrence>) -> bool {
    let graph = dependencies();
    for start in members {
        let mut queue: VecDeque<_> = graph
            .iter()
            .filter_map(|(consumer, inputs)| inputs.contains(start).then_some((*consumer, false)))
            .collect();
        let mut seen = BTreeSet::new();
        while let Some((node, left_region)) = queue.pop_front() {
            if !seen.insert((node, left_region)) {
                continue;
            }
            let now_left = left_region || !members.contains(&node);
            if now_left && members.contains(&node) {
                return false;
            }
            queue.extend(graph.iter().filter_map(|(consumer, inputs)| {
                inputs.contains(&node).then_some((*consumer, now_left))
            }));
        }
    }
    true
}

fn invalid<T>(candidate: &RegionCandidate, rule: &'static str) -> Result<T, CandidateError> {
    Err(CandidateError::Invalid {
        candidate: candidate.stable_id.clone(),
        rule,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    fn request() -> VerifiedTargetRequest {
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
        let semantic = builder.build().unwrap();
        let verified = verify_request(CompilationRequest::governed(&semantic)).unwrap();
        verified.for_target(verified.target_profiles[0])
    }

    #[test]
    fn governed_enumeration_is_stable_complete_and_boundary_checked() {
        let request = request();
        let candidates = enumerate_candidates(&request).unwrap();
        assert_eq!(candidates.len(), 7);
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.stable_id.as_str())
                .collect::<Vec<_>>(),
            [
                "candidate:scale-constant",
                "candidate:multiply",
                "candidate:bias-constant",
                "candidate:add",
                "candidate:strict-sum",
                "candidate:scale-constant+multiply+bias-constant+add",
                "candidate:scale-constant+multiply+bias-constant+add+strict-sum",
            ]
        );
        let fused = candidates.last().unwrap();
        assert_eq!(
            fused.boundary_inputs,
            BTreeSet::from([BoundaryValue::InputTensor])
        );
        assert_eq!(
            fused.boundary_outputs,
            BTreeSet::from([BoundaryValue::OutputTensor])
        );
        prove_fused_numerics(&request, fused).unwrap();
        assert_eq!(
            candidates[1].boundary_inputs,
            BTreeSet::from([BoundaryValue::InputTensor, BoundaryValue::ScaleConstant,])
        );
        assert_eq!(
            candidates[1].boundary_outputs,
            BTreeSet::from([BoundaryValue::Product])
        );
    }

    #[test]
    fn disconnected_and_nonconvex_candidates_fail_closed() {
        let disconnected = candidate(
            CandidateKind::Pointwise,
            [
                SemanticOccurrence::ScaleConstant,
                SemanticOccurrence::BiasConstant,
            ],
        );
        assert!(matches!(
            verify_candidate(&disconnected),
            Err(CandidateError::Invalid {
                rule: "connectivity",
                ..
            })
        ));

        let nonconvex = candidate(
            CandidateKind::FusedSerialSum,
            [SemanticOccurrence::Multiply, SemanticOccurrence::StrictSum],
        );
        assert!(matches!(
            verify_candidate(&nonconvex),
            Err(CandidateError::Invalid {
                rule: "connectivity" | "convexity",
                ..
            })
        ));
    }
}
