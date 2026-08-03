//! Compiler-owned discharge of residual logical index-domain predicates.
//!
//! A structurally verified region may retain exact `Unknown` predicates without
//! becoming executable refinement evidence. This stage consumes that pending
//! state before cover enumeration. A trusted rule assesses each exact borrowed
//! obligation once; only an all-`Proved` result seals durable receipts and
//! completes refinement. `Disproved` and unsupported `Unknown` remain distinct
//! typed refusals.
//!
//! The receipts overlay the immutable verified region. They do not rewrite
//! `tiler-ir` verifier evidence, copy its predicate language, or re-drive the
//! lowering provider. The production authority evaluates only exact logical
//! coordinates and extents. Dtype payloads, component layouts, and physical
//! encodings cannot affect an index-domain predicate and are never inspected.

use core::fmt;
use std::collections::{HashMap, HashSet};

use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::Zero;
use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{
    IndexDomainDisproof as IrIndexDomainDisproof, IndexDomainPredicate,
    IndexDomainProofAssessment as IrIndexDomainProofAssessment,
    IndexDomainProofAuthority as IrIndexDomainProofAuthority,
    IndexDomainProofClaim as IrIndexDomainProofClaim,
    IndexDomainProofEvidence as IrIndexDomainProofEvidence,
    IndexDomainProofRefusalKind as IrIndexDomainProofRefusalKind,
    IndexDomainProofVerifier as IrIndexDomainProofVerifier, IndexDomainSoundProof,
    IndexDomainUnknownReason, IndexExprView, IndexExtentRef, IndexInteger, IndexIntegerSign,
    ProofResource, UnknownIndexDomainPredicate, VerifiedDimensionId, VerifiedIndexExprId,
    VerifiedIndexRegion,
};
use tiler_ir::semantic::ProviderIdentity;

use crate::legality::{IndexRefinement, PendingIndexRefinement, complete_pending_index_refinement};

const EXHAUSTIVE_DERIVATION: &[u8] = b"tiler.compiler.exact-index-domain-enumeration.v1\0";
const COUNTEREXAMPLE_TAG: &[u8] = b"tiler.compiler.index-domain-counterexample.v1\0";
const MAX_DISCHARGE_CELLS: u64 = 16 * 1024 * 1024;

/// Versioned semantic identity of one proof or disproof rule.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct IndexDomainProofRuleKey(ProviderIdentity);

impl IndexDomainProofRuleKey {
    /// Creates a compiler-owned rule key.
    fn builtin(name: &str, version: u32) -> Self {
        Self(
            ProviderIdentity::new("tiler", name, version)
                .expect("compiler-owned discharge rule keys are valid"),
        )
    }

    /// Returns the canonical provider-shaped key.
    pub(crate) const fn identity(&self) -> &ProviderIdentity {
        &self.0
    }
}

/// Output-affecting revision of one discharge authority.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct IndexDomainDischargeRevision(u32);

impl IndexDomainDischargeRevision {
    /// Creates a nonzero revision.
    fn new(value: u32) -> Option<Self> {
        (value != 0).then_some(Self(value))
    }

    /// Returns the stored revision.
    pub(crate) const fn get(self) -> u32 {
        self.0
    }
}

/// Complete identity of the trusted rule making one claim.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct IndexDomainDischargeAuthority {
    provider: ProviderIdentity,
    rule: IndexDomainProofRuleKey,
    revision: IndexDomainDischargeRevision,
}

impl IndexDomainDischargeAuthority {
    pub(crate) fn builtin(provider: &str, rule: &str, revision: u32) -> Self {
        Self {
            provider: ProviderIdentity::new("tiler", provider, 1)
                .expect("compiler-owned discharge provider identities are valid"),
            rule: IndexDomainProofRuleKey::builtin(rule, 1),
            revision: IndexDomainDischargeRevision::new(revision)
                .expect("compiler-owned discharge revisions are nonzero"),
        }
    }

    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    pub(crate) const fn rule(&self) -> &IndexDomainProofRuleKey {
        &self.rule
    }

    pub(crate) const fn revision(&self) -> IndexDomainDischargeRevision {
        self.revision
    }
}

/// A proving basis a trusted discharge rule may claim.
///
/// Empirical evidence is absent by construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IndexDomainDischargeProof {
    /// A sound named derivation over the complete predicate domain.
    #[allow(
        dead_code,
        reason = "the production finite authority emits exhaustive evidence, while the complete protocol retains the sound-proof lane for future compiler proofs and exercises it through conformance authorities"
    )]
    Sound {
        proof: IndexDomainSoundProof,
        derivation: Box<[u8]>,
    },
    /// Exact evaluation of every point in a bounded finite domain.
    ExhaustiveFinite { points: u64, derivation: Box<[u8]> },
}

/// A typed semantic disproof claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexDomainDisproof {
    reason: Box<str>,
    point_ordinal: Option<u64>,
    counterexample: Box<[u8]>,
}

impl IndexDomainDisproof {
    pub(crate) fn new(reason: impl Into<Box<str>>, counterexample: impl Into<Box<[u8]>>) -> Self {
        Self {
            reason: reason.into(),
            point_ordinal: None,
            counterexample: counterexample.into(),
        }
    }

    fn with_point_ordinal(mut self, point_ordinal: u64) -> Self {
        self.point_ordinal = Some(point_ordinal);
        self
    }

    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }

    pub(crate) const fn point_ordinal(&self) -> Option<u64> {
        self.point_ordinal
    }
}

/// One trusted rule's total claim about one exact borrowed obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IndexDomainDischargeClaim {
    Proved(IndexDomainDischargeProof),
    Disproved(IndexDomainDisproof),
    Unknown(IndexDomainUnknownReason),
}

/// The private authority callback used by the discharge stage.
///
/// It is deliberately not a public extension seam. The production compiler has
/// one exact finite authority; a public registry belongs with the first
/// independently installable authority and its reviewed resolution contract.
pub(crate) trait IndexDomainDischargeProvider: Send + Sync {
    fn authority(&self) -> &IndexDomainDischargeAuthority;

    fn assess(
        &self,
        region: &VerifiedIndexRegion,
        obligation: UnknownIndexDomainPredicate,
    ) -> IndexDomainDischargeClaim;
}

/// Presents a compiler proof rule through the IR-owned one-pass sealing boundary.
struct IrDischargeAdapter<'a> {
    provider: &'a dyn IndexDomainDischargeProvider,
    authority: IrIndexDomainProofAuthority,
}

impl<'a> IrDischargeAdapter<'a> {
    fn new(provider: &'a dyn IndexDomainDischargeProvider) -> Self {
        let authority = provider.authority();
        Self {
            provider,
            authority: IrIndexDomainProofAuthority::new(
                authority.provider().clone(),
                authority.rule().identity().clone(),
                authority.revision().get(),
            )
            .expect("compiler discharge authorities have nonzero revisions"),
        }
    }
}

impl IrIndexDomainProofVerifier for IrDischargeAdapter<'_> {
    fn authority(&self) -> &IrIndexDomainProofAuthority {
        &self.authority
    }

    fn assess(
        &self,
        region: &VerifiedIndexRegion,
        obligation: UnknownIndexDomainPredicate,
    ) -> IrIndexDomainProofClaim {
        match self.provider.assess(region, obligation) {
            IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::Sound {
                proof,
                derivation,
            }) => IrIndexDomainProofEvidence::sound(proof, derivation).map_or_else(
                |_| IrIndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment),
                IrIndexDomainProofClaim::Proved,
            ),
            IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::ExhaustiveFinite {
                points,
                derivation,
            }) => IrIndexDomainProofEvidence::exhaustive_finite(points, derivation).map_or_else(
                |_| IrIndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment),
                IrIndexDomainProofClaim::Proved,
            ),
            IndexDomainDischargeClaim::Disproved(disproof) => {
                let Ok(mut converted) =
                    IrIndexDomainDisproof::new(disproof.reason(), disproof.counterexample.clone())
                else {
                    return IrIndexDomainProofClaim::Unknown(
                        IndexDomainUnknownReason::UnsupportedFragment,
                    );
                };
                if let Some(point) = disproof.point_ordinal() {
                    converted = converted.with_point_ordinal(point);
                }
                IrIndexDomainProofClaim::Disproved(converted)
            }
            IndexDomainDischargeClaim::Unknown(reason) => IrIndexDomainProofClaim::Unknown(reason),
        }
    }
}

/// One exact assessment retained for explanation on refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexDomainDischargeAssessment {
    obligation: UnknownIndexDomainPredicate,
    authority: IndexDomainDischargeAuthority,
    claim: IndexDomainDischargeClaim,
}

impl IndexDomainDischargeAssessment {
    pub(crate) const fn obligation(&self) -> UnknownIndexDomainPredicate {
        self.obligation
    }

    pub(crate) const fn authority(&self) -> &IndexDomainDischargeAuthority {
        &self.authority
    }

    pub(crate) const fn claim(&self) -> &IndexDomainDischargeClaim {
        &self.claim
    }
}

/// Why semantic discharge refused one otherwise-conforming realization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IndexDomainDischargeRefusalKind {
    Disproved,
    Unknown,
}

/// Atomic refusal retaining every canonical assessment and the exact pending state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexDomainDischargeRefusal {
    pending: Box<PendingIndexRefinement>,
    assessments: Vec<IndexDomainDischargeAssessment>,
    kind: IndexDomainDischargeRefusalKind,
}

impl IndexDomainDischargeRefusal {
    #[allow(
        dead_code,
        reason = "the pending state is retained to prove atomic refusal and inspected by conformance tests; production explanation consumes the exact assessments instead"
    )]
    pub(crate) const fn pending(&self) -> &PendingIndexRefinement {
        &self.pending
    }

    pub(crate) fn assessments(&self) -> &[IndexDomainDischargeAssessment] {
        &self.assessments
    }

    pub(crate) const fn kind(&self) -> IndexDomainDischargeRefusalKind {
        self.kind
    }
}

impl fmt::Display for IndexDomainDischargeRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} index-domain obligation(s) reached semantic discharge as {:?}",
            self.assessments.len(),
            self.kind
        )
    }
}

/// Exact bounded compiler-host evaluator for logical index-domain atoms.
struct ExactFiniteIndexDomainDischarge {
    authority: IndexDomainDischargeAuthority,
}

impl ExactFiniteIndexDomainDischarge {
    fn governed() -> Self {
        Self {
            authority: IndexDomainDischargeAuthority::builtin(
                "compiler.index-domain-discharge",
                "exact-finite-index-domain-enumeration",
                1,
            ),
        }
    }
}

impl IndexDomainDischargeProvider for ExactFiniteIndexDomainDischarge {
    fn authority(&self) -> &IndexDomainDischargeAuthority {
        &self.authority
    }

    fn assess(
        &self,
        region: &VerifiedIndexRegion,
        obligation: UnknownIndexDomainPredicate,
    ) -> IndexDomainDischargeClaim {
        assess_finite_domain(region, obligation)
    }
}

/// Runs the production discharge rule before executable planning.
pub(crate) fn discharge_pending_index_refinement(
    pending: PendingIndexRefinement,
) -> Result<IndexRefinement, IndexDomainDischargeRefusal> {
    discharge_with(&ExactFiniteIndexDomainDischarge::governed(), pending)
}

fn assess_finite_domain(
    region: &VerifiedIndexRegion,
    obligation: UnknownIndexDomainPredicate,
) -> IndexDomainDischargeClaim {
    let access = region
        .access(obligation.subject())
        .expect("a verified residual names an access in its own region");
    let dimensions = access
        .domain()
        .map(|dimension| {
            region
                .dimension(dimension)
                .expect("a verified access domain names its own dimensions")
                .extent()
                .as_static()
                .map(|extent| (dimension, extent.get()))
        })
        .collect::<Option<Vec<_>>>();
    let Some(dimensions) = dimensions else {
        return IndexDomainDischargeClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment);
    };
    let points = dimensions.iter().try_fold(1_u128, |product, (_, extent)| {
        product.checked_mul(u128::from(*extent))
    });
    let Some(points) = points else {
        return resource_limit(u128::MAX);
    };
    let expression = predicate_expression(obligation.predicate());
    let mut plan = HashSet::new();
    if !collect_expression_plan(region, expression, &mut plan) {
        return IndexDomainDischargeClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment);
    }
    let required = points.saturating_mul(plan.len() as u128);
    if required > u128::from(MAX_DISCHARGE_CELLS) {
        return resource_limit(required);
    }
    let Ok(points) = u64::try_from(points) else {
        return resource_limit(required);
    };
    let mut coordinates = vec![0_u64; dimensions.len()];
    let mut environment = HashMap::with_capacity(dimensions.len());
    let mut values = HashMap::with_capacity(plan.len());
    for point_ordinal in 0..points {
        environment.clear();
        environment.extend(
            dimensions
                .iter()
                .zip(&coordinates)
                .map(|((dimension, _), coordinate)| (*dimension, *coordinate)),
        );
        values.clear();
        let Some(value) = evaluate_expression(region, expression, &environment, &mut values) else {
            return IndexDomainDischargeClaim::Unknown(
                IndexDomainUnknownReason::UnsupportedFragment,
            );
        };
        if !predicate_holds(region, obligation.predicate(), &value) {
            let reason = match obligation.predicate() {
                IndexDomainPredicate::NonNegative { .. } => "logical-index-negative",
                IndexDomainPredicate::LessThanExtent { .. } => "logical-index-not-less-than-extent",
            };
            return IndexDomainDischargeClaim::Disproved(
                IndexDomainDisproof::new(reason, encode_counterexample(&coordinates, &value))
                    .with_point_ordinal(point_ordinal),
            );
        }
        increment_coordinates(&mut coordinates, &dimensions);
    }
    IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::ExhaustiveFinite {
        points,
        derivation: EXHAUSTIVE_DERIVATION.into(),
    })
}

fn resource_limit(required: u128) -> IndexDomainDischargeClaim {
    IndexDomainDischargeClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
        resource: ProofResource::Cells,
        required,
        limit: MAX_DISCHARGE_CELLS,
    })
}

const fn predicate_expression(predicate: IndexDomainPredicate) -> VerifiedIndexExprId {
    match predicate {
        IndexDomainPredicate::NonNegative { expression }
        | IndexDomainPredicate::LessThanExtent { expression, .. } => expression,
    }
}

fn collect_expression_plan(
    region: &VerifiedIndexRegion,
    expression: VerifiedIndexExprId,
    reached: &mut HashSet<VerifiedIndexExprId>,
) -> bool {
    if !reached.insert(expression) {
        return true;
    }
    let expression = region
        .index_expression(expression)
        .expect("a verified predicate names an expression in its own region");
    match expression.view() {
        IndexExprView::Constant(_) | IndexExprView::Dimension(_) => true,
        IndexExprView::LinearCombination { terms, .. } => terms
            .map(tiler_ir::index::LinearTermRef::value)
            .all(|child| collect_expression_plan(region, child, reached)),
        // A divisor this region names symbolically has no value here: the
        // enumeration below assigns domain coordinates and nothing else, so a
        // semi-affine expression is declined at the plan stage and reaches the
        // caller as `UnsupportedFragment` rather than as an evaluation that
        // quietly picked a divisor.
        IndexExprView::FloorDiv { dividend, divisor }
        | IndexExprView::Modulo { dividend, divisor } => {
            divisor.as_static().is_some() && collect_expression_plan(region, dividend, reached)
        }
        _ => false,
    }
}

fn evaluate_expression(
    region: &VerifiedIndexRegion,
    expression: VerifiedIndexExprId,
    environment: &HashMap<VerifiedDimensionId, u64>,
    values: &mut HashMap<VerifiedIndexExprId, BigInt>,
) -> Option<BigInt> {
    if let Some(value) = values.get(&expression) {
        return Some(value.clone());
    }
    let view = region
        .index_expression(expression)
        .expect("a verified predicate names an expression in its own region")
        .view();
    let value = match view {
        IndexExprView::Constant(value) => decode_integer(value),
        IndexExprView::Dimension(dimension) => BigInt::from(*environment.get(&dimension)?),
        IndexExprView::LinearCombination { constant, terms } => {
            let mut total = decode_integer(constant);
            for term in terms {
                total += decode_integer(term.coefficient())
                    * evaluate_expression(region, term.value(), environment, values)?;
            }
            total
        }
        // `collect_expression_plan` already declined a symbolic divisor, so a
        // `None` here would mean the two disagreed; it is propagated as "no
        // value" rather than unwrapped, which leaves the obligation open
        // instead of crashing or inventing a coordinate.
        IndexExprView::FloorDiv { dividend, divisor } => {
            evaluate_expression(region, dividend, environment, values)?
                .div_floor(&BigInt::from(divisor.as_static()?.get()))
        }
        IndexExprView::Modulo { dividend, divisor } => {
            evaluate_expression(region, dividend, environment, values)?
                .mod_floor(&BigInt::from(divisor.as_static()?.get()))
        }
        _ => return None,
    };
    values.insert(expression, value.clone());
    Some(value)
}

fn decode_integer(value: &IndexInteger) -> BigInt {
    let (sign, magnitude) = value.to_sign_magnitude();
    BigInt::from_bytes_be(
        match sign {
            IndexIntegerSign::Positive => Sign::Plus,
            IndexIntegerSign::Negative => Sign::Minus,
            IndexIntegerSign::Zero => Sign::NoSign,
        },
        &magnitude,
    )
}

fn predicate_holds(
    region: &VerifiedIndexRegion,
    predicate: IndexDomainPredicate,
    value: &BigInt,
) -> bool {
    match predicate {
        IndexDomainPredicate::NonNegative { .. } => value >= &BigInt::zero(),
        IndexDomainPredicate::LessThanExtent { extent, .. } => {
            value < &BigInt::from(resolve_extent(region, extent))
        }
    }
}

fn resolve_extent(region: &VerifiedIndexRegion, extent: IndexExtentRef) -> u64 {
    match extent {
        IndexExtentRef::Dimension(dimension) => region
            .dimension(dimension)
            .expect("a verified predicate names its own dimension")
            .extent()
            .as_static()
            .expect("the finite discharge rejected symbolic dimensions")
            .get(),
        IndexExtentRef::TensorAxis { tensor, axis } => {
            let shape = region
                .tensor(tensor)
                .expect("a verified predicate names its own tensor")
                .shape()
                .as_static()
                .expect("the finite discharge rejected symbolic boundaries");
            shape.extents()[usize::try_from(axis).expect("a verified tensor axis fits usize")].get()
        }
    }
}

fn increment_coordinates(coordinates: &mut [u64], dimensions: &[(VerifiedDimensionId, u64)]) {
    for (coordinate, (_, extent)) in coordinates.iter_mut().zip(dimensions).rev() {
        *coordinate += 1;
        if *coordinate < *extent {
            return;
        }
        *coordinate = 0;
    }
}

fn encode_counterexample(coordinates: &[u64], value: &BigInt) -> Box<[u8]> {
    let mut output = COUNTEREXAMPLE_TAG.to_vec();
    push_len(&mut output, coordinates.len());
    for coordinate in coordinates {
        output.extend_from_slice(&coordinate.to_be_bytes());
    }
    let (sign, magnitude) = value.to_bytes_be();
    output.push(match sign {
        Sign::Minus => 1,
        Sign::NoSign | Sign::Plus => 0,
    });
    push_slice(&mut output, &magnitude);
    output.into_boxed_slice()
}

pub(crate) fn discharge_with(
    provider: &dyn IndexDomainDischargeProvider,
    pending: PendingIndexRefinement,
) -> Result<IndexRefinement, IndexDomainDischargeRefusal> {
    let adapter = IrDischargeAdapter::new(provider);
    let completed =
        tiler_ir::index::ResolvedIndexRealization::complete(pending.ir_receipt(), &adapter);
    let (ir_receipt, ir_assessments) = match completed {
        Ok(completed) => completed,
        Err(refusal) => {
            let kind = match refusal.kind() {
                IrIndexDomainProofRefusalKind::Disproved => {
                    IndexDomainDischargeRefusalKind::Disproved
                }
                IrIndexDomainProofRefusalKind::Unknown => IndexDomainDischargeRefusalKind::Unknown,
            };
            return Err(IndexDomainDischargeRefusal {
                pending: Box::new(pending),
                assessments: refusal
                    .assessments()
                    .iter()
                    .map(|assessment| convert_ir_assessment(provider.authority(), assessment))
                    .collect(),
                kind,
            });
        }
    };
    debug_assert!(
        ir_assessments
            .iter()
            .all(|assessment| matches!(assessment.claim(), IrIndexDomainProofClaim::Proved(_)))
    );
    Ok(complete_pending_index_refinement(pending, ir_receipt))
}

fn convert_ir_assessment(
    authority: &IndexDomainDischargeAuthority,
    assessment: &IrIndexDomainProofAssessment,
) -> IndexDomainDischargeAssessment {
    let claim = match assessment.claim() {
        IrIndexDomainProofClaim::Proved(IrIndexDomainProofEvidence::Sound {
            proof,
            derivation,
            ..
        }) => IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::Sound {
            proof: *proof,
            derivation: derivation.clone(),
        }),
        IrIndexDomainProofClaim::Proved(IrIndexDomainProofEvidence::ExhaustiveFinite {
            points,
            derivation,
            ..
        }) => IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::ExhaustiveFinite {
            points: *points,
            derivation: derivation.clone(),
        }),
        IrIndexDomainProofClaim::Disproved(disproof) => {
            let mut converted =
                IndexDomainDisproof::new(disproof.reason(), disproof.counterexample());
            if let Some(point) = disproof.point_ordinal() {
                converted = converted.with_point_ordinal(point);
            }
            IndexDomainDischargeClaim::Disproved(converted)
        }
        IrIndexDomainProofClaim::Unknown(reason) => IndexDomainDischargeClaim::Unknown(*reason),
    };
    IndexDomainDischargeAssessment {
        obligation: assessment.obligation(),
        authority: authority.clone(),
        claim,
    }
}

#[cfg(test)]
mod tests {
    use tiler_ir::index::{
        DomainRole, FrozenScalarRegistry, IndexRegionBuilder, ScalarRegistryBuilder, SourcedExtent,
        TensorRole,
    };
    use tiler_ir::semantic::{
        AttributeFieldId, CanonicalField, CanonicalValue, EncodedNumericContract,
        NormativeDefinitionRef, ProviderIdentity, QuantSchemeKey, RegistryError, ResolvedValueType,
        SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
        TypeArguments, TypeDefinitionFacts, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
    };
    use tiler_ir::shape::{Extent, Shape};

    use super::{
        IndexDomainDischargeClaim, IndexDomainDischargeProof, IndexDomainUnknownReason,
        MAX_DISCHARGE_CELLS, ProofResource, assess_finite_domain,
    };

    const LENGTH: u64 = 65_535;

    struct TestTypeFamilies;

    impl SemanticRegistryProvider for TestTypeFamilies {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "index-discharge-types", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            // The nominal and parameterized subjects are the governed
            // `tiler::bool@1` and `tiler::complex@1` the standard registry
            // itself admits, so this fixture no longer mints a second identity
            // under a name the catalog owns. Only the encoded family, which
            // the standard catalog admits no static contract for, is test-owned.
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::EncodedNumeric(
                    QuantSchemeKey::new("test", "encoded", 1).unwrap(),
                ),
                NormativeDefinitionRef::new("test encoded family for index discharge")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ))
        }
    }

    fn scalar_authority() -> FrozenScalarRegistry {
        let mut semantics = SemanticRegistryBuilder::standard().unwrap();
        semantics.register_provider(&TestTypeFamilies).unwrap();
        ScalarRegistryBuilder::new(semantics.freeze().unwrap()).freeze()
    }

    fn complex_type() -> ResolvedValueType {
        ResolvedValueType::parameterized(
            TypeKey::new("tiler", "complex", 1).unwrap(),
            TypeArguments::new([CanonicalValue::value_type(ResolvedValueType::nominal(
                TypeKey::new("tiler", "f32", 1).unwrap(),
            ))])
            .unwrap(),
        )
        .unwrap()
    }

    fn encoded_type() -> ResolvedValueType {
        ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("test", "encoded", 1).unwrap(),
            EncodedNumericContract::new([CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::value_type(ResolvedValueType::nominal(
                    TypeKey::new("tiler", "u4", 1).unwrap(),
                )),
            )])
            .unwrap(),
        )
        .unwrap()
    }

    fn residual_region(
        value_type: ResolvedValueType,
        second_extent: u64,
        rounds: usize,
        offset: i128,
    ) -> tiler_ir::index::VerifiedIndexRegion {
        let mut builder = IndexRegionBuilder::new(scalar_authority()).unwrap();
        let first = builder
            .dimension(DomainRole::Parallel, Extent::new(LENGTH))
            .unwrap();
        let second = builder
            .dimension(DomainRole::Parallel, Extent::new(second_extent))
            .unwrap();
        let shape = Shape::from_dims([LENGTH, second_extent]);
        let input = builder
            .tensor(TensorRole::Input, value_type.clone(), shape.clone())
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, value_type, shape)
            .unwrap();
        let first_coordinate = builder.dimension_expr(first).unwrap();
        let second_coordinate = builder.dimension_expr(second).unwrap();
        let mut conservative = first_coordinate;
        for _ in 0..rounds {
            let two = SourcedExtent::Static(Extent::new(2));
            let modulo = builder.modulo(conservative, two.clone()).unwrap();
            let quotient = builder.floor_div(conservative, two).unwrap();
            conservative = builder
                .linear_combination(
                    0_i128.into(),
                    &[(2_i128.into(), quotient), (1_i128.into(), modulo)],
                )
                .unwrap();
        }
        if offset != 0 {
            conservative = builder
                .linear_combination(offset.into(), &[(1_i128.into(), conservative)])
                .unwrap();
        }
        let value = builder
            .read(input, &[first, second], &[conservative, second_coordinate])
            .unwrap();
        let write = builder
            .write(
                output,
                &[first, second],
                &[first_coordinate, second_coordinate],
            )
            .unwrap();
        builder.output(write, value).unwrap();
        let region = builder.build().unwrap();
        assert_eq!(region.unknown_index_domain_predicates().len(), 1);
        region
    }

    fn claim(region: &tiler_ir::index::VerifiedIndexRegion) -> IndexDomainDischargeClaim {
        let obligation = region
            .unknown_index_domain_predicates()
            .next()
            .expect("the fixture retains one residual");
        assess_finite_domain(region, obligation)
    }

    #[test]
    fn exact_enumeration_proves_the_beyond_verifier_budget_fixture() {
        let region = residual_region(
            ResolvedValueType::nominal(TypeKey::new("tiler", "bool", 1).unwrap()),
            1,
            5,
            0,
        );
        assert!(matches!(
            claim(&region),
            IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::ExhaustiveFinite {
                points: LENGTH,
                ..
            })
        ));
    }

    #[test]
    fn exact_enumeration_returns_a_deterministic_counterexample() {
        let region = residual_region(
            ResolvedValueType::nominal(TypeKey::new("tiler", "u4", 1).unwrap()),
            1,
            5,
            1,
        );
        let first = claim(&region);
        let second = claim(&region);
        assert_eq!(first, second);
        assert!(matches!(
            first,
            IndexDomainDischargeClaim::Disproved(disproof)
                if disproof.reason() == "logical-index-not-less-than-extent"
        ));
    }

    #[test]
    fn the_second_governed_budget_preserves_unknown_without_permission() {
        let region = residual_region(
            ResolvedValueType::nominal(TypeKey::new("tiler", "bool", 1).unwrap()),
            64,
            5,
            0,
        );
        assert!(matches!(
            claim(&region),
            IndexDomainDischargeClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: ProofResource::Cells,
                required,
                limit: MAX_DISCHARGE_CELLS,
            }) if required > u128::from(MAX_DISCHARGE_CELLS)
        ));
    }

    #[test]
    fn dtype_family_does_not_change_a_logical_coordinate_proof() {
        let boolean = ResolvedValueType::nominal(TypeKey::new("tiler", "bool", 1).unwrap());
        let integer = ResolvedValueType::nominal(TypeKey::new("tiler", "u4", 1).unwrap());
        let claims = [boolean, integer, complex_type(), encoded_type()]
            .map(|value_type| claim(&residual_region(value_type, 1, 5, 0)));
        assert_eq!(claims[0], claims[1]);
        assert_eq!(claims[1], claims[2]);
        assert_eq!(claims[2], claims[3]);
    }
}
