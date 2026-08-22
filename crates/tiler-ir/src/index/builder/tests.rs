//! Resource-ordering tests for the index builder.

use std::cell::Cell;
use std::error::Error as _;
use std::mem::variant_count;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{
    CompactedRegion, ProofBudgetExcess, ReducerBodyBudget, admit_reducer_body_append,
    encode_reducer_body, encode_region, encoded_reducer_body_len,
    encoded_reducer_operation_base_len, encoded_reducer_operation_result_increment,
    encoded_reducer_operation_result_overhead, encoded_reducer_parameter_len, encoded_region_len,
    map_scalar_apply_error, minimum_reducer_body, with_admitted_proof_budget,
};
use crate::index::model::{
    CompactedAccess, CompactedGatherReadAccess, IndexNode, ReducerBodyOperationData,
    ReducerBodyValueData, ReducerBodyValueSource, ScalarReducerBodyData, VerifiedAccessData,
    VerifiedDirectAccessData, WriteOwnershipProof,
};
use crate::index::scalar::{ScalarApplyError, ScalarInferenceHostFailure};
use crate::index::{
    DomainRole, FrozenScalarRegistry, IndexBuildError, IndexDomainEvidence, IndexDomainFactSource,
    IndexDomainPredicate, IndexDomainSoundProof, IndexDomainUnknownReason, IndexExprClass,
    IndexExtentRef, IndexLimitKind, IndexRegionBuilder, MAX_SCALAR_CANONICAL_BYTES, ProofResource,
    ScalarArity, ScalarAttributeSchema, ScalarAttributes, ScalarEffect, ScalarInferenceError,
    ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract,
    ScalarOperationDefinition, ScalarOperationInferencer, ScalarRegistryBuilder, ScalarResultIndex,
    TensorRole, UnknownIndexDomainPredicate, VerifiedIndexRegion,
};
use crate::semantic::{
    CanonicalValue, NormativeDefinitionRef, ProviderIdentity, RegistryError, ResolvedValueType,
    SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
    TypeDefinitionFacts, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
};
use crate::shape::{Extent, Shape};

fn reducer_test_type() -> ResolvedValueType {
    ResolvedValueType::nominal(TypeKey::new("test", "reducer-value", 1).unwrap())
}

struct ReducerTestTypes;
impl SemanticRegistryProvider for ReducerTestTypes {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "reducer-types", 1).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("test", "reducer-value", 1).unwrap()),
            NormativeDefinitionRef::new("urn:test:reducer-value:v1").unwrap(),
            TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
        ))
    }
}

struct FirstOperand {
    calls: Arc<AtomicUsize>,
}
impl ScalarOperationInferencer for FirstOperand {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        outputs.try_push(request.operands()[0].clone())
    }
}

fn reducer_test_registry(calls: Arc<AtomicUsize>) -> FrozenScalarRegistry {
    let mut semantic = SemanticRegistryBuilder::new();
    semantic.register_provider(&ReducerTestTypes).unwrap();
    // Ad-hoc: registers reducer-body scalars sized for the resource-ordering budget
    // under test. The governed profile registers no reducer body at all.
    let mut scalar = ScalarRegistryBuilder::new(semantic.freeze().unwrap());
    let key = ScalarOpKey::new("test", "step", 1).unwrap();
    scalar
        .register(
            ProviderIdentity::new("test", "reducer-scalars", 1).unwrap(),
            ScalarOperationDefinition::new(
                key,
                NormativeDefinitionRef::new("urn:test:step:v1").unwrap(),
                ScalarOperationContract::new(
                    ScalarAttributeSchema::empty(),
                    ScalarArity::exact(1).unwrap(),
                    ScalarArity::exact(1).unwrap(),
                    ScalarEffect::Pure,
                    CanonicalValue::record([]).unwrap(),
                    CanonicalValue::record([]).unwrap(),
                ),
                Arc::new(FirstOperand { calls }),
            ),
        )
        .unwrap();
    scalar.freeze()
}

/// Re-stages verified accesses for a re-encoding test.
///
/// Encoding reads [`CompactedAccess`], which by design cannot carry a gather's
/// bounds resolution — the proof binds the identity these tests recompute. The
/// fixtures here author only direct accesses, so the mapping is total; a gather
/// fixture would have to re-derive its resolution rather than carry one across.
fn compacted_accesses(accesses: &[VerifiedAccessData]) -> Vec<CompactedAccess> {
    accesses
        .iter()
        .map(|access| match access {
            VerifiedAccessData::Direct(direct) => CompactedAccess::Direct(direct.clone()),
            VerifiedAccessData::GatherRead(_) => {
                unreachable!("these identity fixtures author only direct accesses")
            }
        })
        .collect()
}

fn verified_copy() -> VerifiedIndexRegion {
    let mut builder =
        IndexRegionBuilder::new(reducer_test_registry(Arc::new(AtomicUsize::new(0)))).unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(5))
        .unwrap();
    let coordinate = builder.dimension_expr(dimension).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            reducer_test_type(),
            Shape::new([Extent::new(5)]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            reducer_test_type(),
            Shape::new([Extent::new(5)]),
        )
        .unwrap();
    let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
    let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
    builder.output(write, value).unwrap();
    builder.build().unwrap()
}

#[test]
fn proof_materialization_runs_only_after_aggregate_admission() {
    let materializations = Cell::new(0_u32);
    let rejected = with_admitted_proof_budget(11, 8, 10, 10, || {
        materializations.set(materializations.get() + 1);
    });
    assert_eq!(
        rejected,
        Err(ProofBudgetExcess::Cells {
            required: 11,
            limit: 10,
        })
    );
    assert_eq!(materializations.get(), 0);

    with_admitted_proof_budget(10, 10, 10, 10, || {
        materializations.set(materializations.get() + 1);
    })
    .unwrap();
    assert_eq!(materializations.get(), 1);
}

#[test]
fn parent_reducer_budget_rejects_before_retaining_the_append() {
    let commits = Cell::new(0_u32);
    let budget = ReducerBodyBudget {
        parent_bytes_without_body: MAX_SCALAR_CANONICAL_BYTES - 20,
        body_multiplier: 2,
        maximum_encoded_bytes: 10,
    };
    let error =
        admit_reducer_body_append(budget, 11, || commits.set(commits.get() + 1)).unwrap_err();
    assert_eq!(commits.get(), 0);
    assert_eq!(
        error,
        IndexBuildError::StructuralLimit {
            resource: IndexLimitKind::ScalarCanonicalBytes,
            actual: (MAX_SCALAR_CANONICAL_BYTES + 2) as u128,
            limit: MAX_SCALAR_CANONICAL_BYTES as u128,
        }
    );

    admit_reducer_body_append(budget, 10, || commits.set(commits.get() + 1)).unwrap();
    assert_eq!(commits.get(), 1);
}

#[test]
fn enclosing_capacity_errors_have_no_provider_source() {
    for error in [
        ScalarApplyError::Host(ScalarInferenceHostFailure::ResultSlots {
            actual: 65_537,
            limit: 65_536,
        }),
        ScalarApplyError::Host(ScalarInferenceHostFailure::CanonicalBytes {
            actual: MAX_SCALAR_CANONICAL_BYTES + 1,
            limit: MAX_SCALAR_CANONICAL_BYTES,
        }),
    ] {
        let mapped = map_scalar_apply_error(error);
        assert!(matches!(mapped, IndexBuildError::StructuralLimit { .. }));
        assert!(mapped.source().is_none());
    }
}

#[test]
fn incremental_reducer_accounting_matches_the_final_encoder() {
    let value_type = reducer_test_type();
    let key = ScalarOpKey::new("test", "step", 1).unwrap();
    let attributes = ScalarAttributes::empty();
    let body = ScalarReducerBodyData {
        values: vec![
            ReducerBodyValueData {
                source: ReducerBodyValueSource::StateParameter(0),
                value_type: value_type.clone(),
            },
            ReducerBodyValueData {
                source: ReducerBodyValueSource::ContributorParameter(0),
                value_type: value_type.clone(),
            },
            ReducerBodyValueData {
                source: ReducerBodyValueSource::OperationResult {
                    operation: 0,
                    result: ScalarResultIndex::from_usize(0).unwrap(),
                },
                value_type: value_type.clone(),
            },
        ],
        operations: vec![ReducerBodyOperationData {
            key: key.clone(),
            attributes: attributes.clone(),
            operands: vec![0, 1],
            results: vec![2],
        }],
        yields: vec![2],
    };
    let incrementally_accounted = 24_usize
        .saturating_add(encoded_reducer_parameter_len(&value_type).saturating_mul(2))
        .saturating_add(encoded_reducer_operation_base_len(&key, &attributes, 2))
        .saturating_add(encoded_reducer_operation_result_increment(&value_type))
        .saturating_add(4);
    let mut encoded = Vec::new();
    encode_reducer_body(&mut encoded, &body);
    assert_eq!(incrementally_accounted, encoded_reducer_body_len(&body));
    assert_eq!(incrementally_accounted, encoded.len());
}

#[test]
fn near_parent_limit_reducer_failure_leaves_the_outer_builder_unchanged() {
    let inference_calls = Arc::new(AtomicUsize::new(0));
    let mut builder =
        IndexRegionBuilder::new(reducer_test_registry(Arc::clone(&inference_calls))).unwrap();
    let reduction = builder
        .dimension(DomainRole::Reduction, Extent::new(2))
        .unwrap();
    let coordinate = builder.dimension_expr(reduction).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            reducer_test_type(),
            Shape::new([Extent::new(2)]),
        )
        .unwrap();
    let contributor = builder.read(input, &[reduction], &[coordinate]).unwrap();
    let init_input = builder
        .tensor(TensorRole::Input, reducer_test_type(), Shape::new([]))
        .unwrap();
    let init = builder.read(init_input, &[], &[]).unwrap();
    let inputs = builder
        .prepare_reduction_inputs(&[reduction], &[init], &[contributor])
        .unwrap();
    let minimum_body_bytes =
        encoded_reducer_body_len(&minimum_reducer_body(&inputs.init, &inputs.contributors));
    let operation_base = encoded_reducer_operation_base_len(
        &ScalarOpKey::new("test", "step", 1).unwrap(),
        &ScalarAttributes::empty(),
        1,
    );
    let encoded_before_results = minimum_body_bytes
        .checked_sub(4)
        .unwrap()
        .checked_add(operation_base)
        .unwrap();
    let minimum_fixed_results = encoded_before_results
        .checked_add(encoded_reducer_operation_result_overhead())
        .unwrap();
    let target_capacity = minimum_fixed_results - 1;
    assert!(target_capacity >= minimum_body_bytes);
    assert!(target_capacity > encoded_before_results);
    assert!(inputs.body_budget.maximum_encoded_bytes >= target_capacity);
    let removable_headroom = inputs
        .body_budget
        .maximum_encoded_bytes
        .checked_sub(target_capacity)
        .unwrap();
    builder.scalar_bytes = builder
        .scalar_bytes
        .saturating_add(removable_headroom.saturating_mul(inputs.body_budget.body_multiplier));
    let before = (
        builder.operations.len(),
        builder.values.len(),
        builder.scalar_bytes,
    );
    let callback_calls = Cell::new(0_u32);
    let error = builder
        .reduce(&[reduction], &[init], &[contributor], |body| {
            callback_calls.set(callback_calls.get() + 1);
            let state = body.state(0).unwrap();
            body.apply(
                ScalarOpKey::new("test", "step", 1).unwrap(),
                ScalarAttributes::empty(),
                &[state],
            )?;
            unreachable!("the parent budget must reject the nested append")
        })
        .unwrap_err();
    assert!(matches!(
        error,
        IndexBuildError::StructuralLimit {
            resource: IndexLimitKind::ScalarCanonicalBytes,
            ..
        }
    ));
    assert_eq!(callback_calls.get(), 1);
    assert_eq!(inference_calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        before,
        (
            builder.operations.len(),
            builder.values.len(),
            builder.scalar_bytes,
        )
    );
}

#[test]
fn every_coordinate_predicate_retains_exact_inspectable_evidence() {
    let region = verified_copy();
    let accesses = region.accesses().collect::<Vec<_>>();
    let records = region
        .discharged_index_domain_predicates()
        .collect::<Vec<_>>();
    assert_eq!(accesses.len(), 2);
    assert_eq!(records.len(), 4);
    for access in accesses {
        let expression = access
            .view()
            .direct()
            .expect("a direct access")
            .coordinates()
            .next()
            .unwrap();
        let tensor = access.view().direct().expect("a direct access").tensor();
        let expected = [
            IndexDomainPredicate::NonNegative { expression },
            IndexDomainPredicate::LessThanExtent {
                expression,
                extent: IndexExtentRef::TensorAxis { tensor, axis: 0 },
            },
        ];
        for predicate in expected {
            let record = region
                .index_domain_evidence(access.id(), predicate)
                .unwrap()
                .expect("both coordinate obligations were discharged");
            assert_eq!(record.subject(), access.id());
            assert_eq!(record.predicate(), predicate);
            assert_eq!(
                record.evidence(),
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval)
            );
            // No environment exists here, so the record states the strong
            // claim: the region's own literals proved it.
            assert_eq!(record.facts(), IndexDomainFactSource::Program);
        }
    }
}

/// The three sound bounds arguments, sized from their vocabulary.
///
/// Held at module scope rather than inside the identity test below because a
/// `const` after a statement is denied by `clippy::items_after_statements`, and
/// because the length is the point: a fourth argument fails to compile here
/// until someone names it, where a hand-written list would have gone on
/// reporting no identity collision over a population that had quietly shrunk.
const SOUND_PROOFS: [IndexDomainSoundProof; variant_count::<IndexDomainSoundProof>()] = [
    IndexDomainSoundProof::VacuousEmptyDomain,
    IndexDomainSoundProof::Interval,
    IndexDomainSoundProof::ProvedExtentEquality,
];

/// A discharged record can carry every evidence class but one.
///
/// `IndexDomainEvidence::Unknown` is the exclusion and it is structural: an
/// unknown is not evidence and cannot construct a
/// `DischargedIndexDomainPredicate` at all, which is why it sits beside the
/// evidence-bearing classes rather than inside them. The remaining three are
/// `SoundProof`, `ExhaustiveFinite`, and `Empirical`, and the identity
/// enumeration below covers all of them. A fifth class must decide which side
/// it falls on before that enumeration can claim to be complete, so it breaks
/// here first.
const _: () = assert!(
    variant_count::<IndexDomainEvidence>() == 4,
    "a fifth evidence class must decide whether a discharged record can carry it \
before the identity enumeration below can stay complete."
);

/// Both premise sources a bounds proof may rest on, sized from their vocabulary.
const FACT_SOURCE_CASES: [IndexDomainFactSource; variant_count::<IndexDomainFactSource>()] = [
    IndexDomainFactSource::Program,
    IndexDomainFactSource::ShapeEnvironment,
];

#[test]
fn retained_ownership_proof_metadata_does_not_enter_region_identity() {
    let region = verified_copy();
    let data = &region.data;
    let mut compacted = CompactedRegion {
        dimensions: data.dimensions.clone(),
        tensors: data.tensors.clone(),
        expressions: data.expressions.clone(),
        accesses: compacted_accesses(&data.accesses),
        index_domain_evidence: data.index_domain_evidence.clone(),
        unknown_index_domain_predicates: data.unknown_index_domain_predicates.clone(),
        operations: data.operations.clone(),
        values: data.values.clone(),
        outputs: data.outputs.clone(),
    };
    let identity =
        |region: &CompactedRegion| encode_region(region, None, encoded_region_len(region, None));
    let baseline = identity(&compacted);
    assert_eq!(baseline.as_bytes(), region.canonical_identity().as_bytes());

    let write = compacted
        .accesses
        .iter_mut()
        .find_map(|access| match access {
            CompactedAccess::Direct(direct) if direct.mode == crate::index::AccessMode::Write => {
                Some(direct)
            }
            _ => None,
        })
        .expect("the existing copy fixture has one direct write");
    write.ownership_proof = Some(WriteOwnershipProof::CoordinatePermutation {
        facts: IndexDomainFactSource::ShapeEnvironment,
    });
    assert_eq!(
        identity(&compacted),
        baseline,
        "the fact-source tag is retained analysis metadata, not a region byte",
    );

    let write = compacted
        .accesses
        .iter_mut()
        .find_map(|access| match access {
            CompactedAccess::Direct(direct) if direct.mode == crate::index::AccessMode::Write => {
                Some(direct)
            }
            _ => None,
        })
        .expect("the existing copy fixture has one direct write");
    write.ownership_proof = Some(WriteOwnershipProof::Exhaustive {
        points: 987,
        facts: IndexDomainFactSource::ShapeEnvironment,
    });
    assert_eq!(
        identity(&compacted),
        baseline,
        "proof mechanism and point count are retained analysis metadata, not region bytes",
    );
}

#[test]
fn index_domain_subject_predicate_outcome_and_basis_each_enter_region_identity() {
    let region = verified_copy();
    let data = &region.data;
    let compacted = || CompactedRegion {
        dimensions: data.dimensions.clone(),
        tensors: data.tensors.clone(),
        expressions: data.expressions.clone(),
        accesses: compacted_accesses(&data.accesses),
        index_domain_evidence: data.index_domain_evidence.clone(),
        unknown_index_domain_predicates: data.unknown_index_domain_predicates.clone(),
        operations: data.operations.clone(),
        values: data.values.clone(),
        outputs: data.outputs.clone(),
    };
    let identity =
        |region: &CompactedRegion| encode_region(region, None, encoded_region_len(region, None));
    let baseline = compacted();
    assert_eq!(
        identity(&baseline).as_bytes(),
        region.canonical_identity().as_bytes()
    );

    let mut changed_subject = compacted();
    changed_subject.index_domain_evidence[0].subject =
        changed_subject.index_domain_evidence[2].subject;
    assert_ne!(identity(&baseline), identity(&changed_subject));

    let mut changed_predicate = compacted();
    let foreign_tensor = match changed_predicate.index_domain_evidence[3].predicate {
        IndexDomainPredicate::LessThanExtent {
            extent: IndexExtentRef::TensorAxis { tensor, .. },
            ..
        } => tensor,
        IndexDomainPredicate::NonNegative { .. }
        | IndexDomainPredicate::LessThanExtent {
            extent: IndexExtentRef::Dimension(_),
            ..
        } => panic!("the neighbouring upper-bound record names its tensor"),
    };
    let IndexDomainPredicate::LessThanExtent { extent, .. } =
        &mut changed_predicate.index_domain_evidence[1].predicate
    else {
        panic!("the first access's second record is its upper bound")
    };
    *extent = IndexExtentRef::TensorAxis {
        tensor: foreign_tensor,
        axis: 0,
    };
    assert_ne!(identity(&baseline), identity(&changed_predicate));

    // Sized from the vocabularies rather than written out, so a fourth sound
    // proof or a fifth evidence class cannot leave this enumeration silently
    // covering less while still reporting no identity collision. The three
    // arguments come from `SOUND_PROOFS`, whose length is the type's; the two
    // remaining classes a discharged record can carry are appended by hand, and
    // the `variant_count` pin beside that constant is what makes a fifth class a
    // build error rather than an omission.
    let evidence_cases = SOUND_PROOFS
        .into_iter()
        .map(IndexDomainEvidence::SoundProof)
        .chain([
            IndexDomainEvidence::ExhaustiveFinite { points: 7 },
            IndexDomainEvidence::Empirical,
        ])
        .collect::<Vec<_>>();
    let mut evidence_identities = Vec::new();
    for evidence in evidence_cases.iter().copied() {
        let mut changed_basis = compacted();
        changed_basis.index_domain_evidence[0].evidence = evidence;
        evidence_identities.push(identity(&changed_basis));
    }
    evidence_identities.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    evidence_identities.dedup();
    assert_eq!(evidence_identities.len(), evidence_cases.len());

    // The fact source is the fourth thing a discharged record carries into
    // identity, beside its subject, predicate, and evidence. Two regions whose
    // bounds hold for the same reason but rest on different premises are
    // different regions, so the tag must move the bytes.
    let mut fact_source_identities = FACT_SOURCE_CASES
        .into_iter()
        .map(|facts| {
            let mut changed_facts = compacted();
            changed_facts.index_domain_evidence[0].facts = facts;
            identity(&changed_facts)
        })
        .collect::<Vec<_>>();
    fact_source_identities.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    fact_source_identities.dedup();
    assert_eq!(
        fact_source_identities.len(),
        variant_count::<IndexDomainFactSource>()
    );

    let unknown = |reason| {
        let mut changed_outcome = compacted();
        let discharged = changed_outcome.index_domain_evidence.remove(0);
        changed_outcome
            .unknown_index_domain_predicates
            .push(UnknownIndexDomainPredicate {
                subject: discharged.subject,
                predicate: discharged.predicate,
                reason,
            });
        changed_outcome
    };
    // Six cases over three variants, because two of them are the same variant
    // distinguished only by its fields. The population cannot be mirrored from
    // `variant_count` for that reason, so the *variant* census is pinned at
    // `the_index_expression_vocabulary_admits_no_data_dependent_form` instead.
    // Any widening is a build error there before it can be an omission here;
    // this inventory does not claim that a particular fourth reason is needed.
    let unknown_cases = [
        IndexDomainUnknownReason::InsufficientFacts,
        IndexDomainUnknownReason::UnsupportedFragment,
        IndexDomainUnknownReason::ResourceLimit {
            resource: ProofResource::Cells,
            required: 11,
            limit: 7,
        },
        IndexDomainUnknownReason::ResourceLimit {
            resource: ProofResource::IntegerBytes,
            required: 11,
            limit: 7,
        },
        IndexDomainUnknownReason::ResourceLimit {
            resource: ProofResource::Cells,
            required: 12,
            limit: 7,
        },
        IndexDomainUnknownReason::ResourceLimit {
            resource: ProofResource::Cells,
            required: 11,
            limit: 8,
        },
    ];
    let mut unknown_keys = unknown_cases
        .iter()
        .copied()
        .map(|reason| unknown(reason).unknown_index_domain_predicates[0].canonical_local_key())
        .collect::<Vec<_>>();
    unknown_keys.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    unknown_keys.dedup();
    assert_eq!(unknown_keys.len(), unknown_cases.len());
    let mut unknown_identities = unknown_cases
        .into_iter()
        .map(|reason| identity(&unknown(reason)))
        .collect::<Vec<_>>();
    unknown_identities.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    unknown_identities.dedup();
    assert_eq!(unknown_identities.len(), unknown_cases.len());
    assert_ne!(
        identity(&baseline),
        identity(&unknown(IndexDomainUnknownReason::InsufficientFacts))
    );
}

/// The index-expression vocabulary ADR 0107 left unchanged, sized from its
/// types.
///
/// ADR 0107 admitted `tiler::gather-f32@1` as a semantic family, and accepted
/// ADR 0108 then chose *where* a data-dependent coordinate lives: on the
/// **access**, as an append-only tagged form, rather than inside an index
/// expression. The census below holds the second half of that choice. A gather
/// now does reach an index region — through `IndexRegionBuilder::gather_read` —
/// but no `IndexNode` form reads tensor data and no `IndexExprClass` member is
/// data-dependent, and admitting either would move the address out of the
/// access and into the expression vocabulary ADR 0108 deliberately left closed.
///
/// These pins therefore guard a boundary that is now *load-bearing* rather than
/// provisional: the direct-access verifier guarantees ADR 0046 requires rest on
/// every index expression remaining a function of the iteration coordinate.
///
/// A negative decision with no check erodes silently, because the way to break
/// it is to *add* something and nothing is watching the count. These pins are
/// what make the widening loud: each compares a `variant_count` against a
/// hand-written literal, so a sixth node form or a fourth class is a build error
/// here that requires an explicit contract update. Neither is a tautology — the
/// two sides come from different places, the type and this file — which is exactly
/// what an expectation derived from the list it checks would fail to be.
///
/// The unit-variant vocabularies get the stronger form: their arrays are
/// *sized* by `variant_count` and *filled* by hand, so a widened vocabulary
/// fails to compile until someone names the new inhabitant. `IndexNode` and
/// `IndexDomainUnknownReason` carry fields and cannot be mirrored that way, so
/// they take a length pin and state their census.
#[test]
fn the_index_expression_vocabulary_admits_no_data_dependent_form() {
    // Sized by the type, filled by hand: a fourth class breaks this line.
    const CLASSES: [IndexExprClass; variant_count::<IndexExprClass>()] = [
        IndexExprClass::Affine,
        IndexExprClass::QuasiAffine,
        IndexExprClass::SemiAffine,
    ];
    // Five forms, none naming a tensor. A `variant_count`-sized array of values
    // is unavailable because every form but `Dimension` carries a non-const
    // payload, so the census is asserted and printed instead of mirrored.
    const NODE_FORMS: usize = 5;
    const _: () = assert!(
        variant_count::<IndexNode>() == NODE_FORMS,
        "accepted ADR 0108 sites the data-dependent coordinate on the access, not in an \
index expression; amend that decision before widening `IndexNode`."
    );
    // Three current reasons, each carrying its own documented meaning. They do
    // not collectively promise eventual closure, and this census neither
    // reserves nor requires a fourth reason for a future data-dependent form.
    const UNKNOWN_REASONS: usize = 3;
    const _: () = assert!(
        variant_count::<IndexDomainUnknownReason>() == UNKNOWN_REASONS,
        "the index-domain unknown-reason vocabulary changed; update the full census, \
identity cases, exhaustive consumers, and governing decision together."
    );

    assert_eq!(
        CLASSES
            .into_iter()
            .fold(IndexExprClass::Affine, IndexExprClass::join),
        IndexExprClass::SemiAffine,
        "`SemiAffine` is the weakest implemented class, so it absorbs the other two"
    );
    // The property the node census stands in for, named rather than inferred
    // from it: each admitted form's operands are literals, domain dimensions, or
    // declared shape symbols, and no form's name mentions a tensor. Written as a
    // wildcard-free match so a sixth form must be named here rather than inherit
    // an answer from a catch-all.
    let form_name = |node: &IndexNode| match node {
        IndexNode::Constant(_) => "integer-literal",
        IndexNode::Dimension(_) => "domain-dimension",
        IndexNode::LinearCombination { .. } => "linear-combination",
        IndexNode::FloorDiv { .. } => "floor-div-by-extent",
        IndexNode::Modulo { .. } => "modulo-by-extent",
    };
    assert_eq!(form_name(&IndexNode::Dimension(0)), "domain-dimension");
    // The census, printed rather than re-asserted. `assert_eq!(NODE_FORMS, 5)`
    // would compare a literal against itself and pass for any population, which
    // is the failure mode the pins above exist to avoid; the counts are read
    // from the types here so a reader of the output sees the population the
    // decision was made over.
    println!(
        "index-expression census: {} node forms, {} classes, {} unknown reasons, {NODE_FORMS} and \
{UNKNOWN_REASONS} pinned",
        variant_count::<IndexNode>(),
        CLASSES.len(),
        variant_count::<IndexDomainUnknownReason>(),
    );
}

/// The three access-encoding tags are pairwise distinct and append-only.
///
/// Sized from the type: `CompactedAccess` has one variant per admitted access
/// kind and `AccessMode` one per direct mode, so widening either vocabulary
/// without extending this census is an array-length build error rather than a
/// population that silently shrinks while still reporting no collision.
///
/// The direct tags are pinned at their exact historical values because moving
/// either would reinterpret every retained `tiler.index-region.v11` value; the
/// gather tag is pinned at the next free one because a reader that reaches it
/// must be reading an access the earlier vocabulary could not express.
#[test]
fn every_access_kind_has_one_append_only_encoding_tag() {
    const DIRECT_MODES: usize = variant_count::<crate::index::AccessMode>();
    const ACCESS_KINDS: usize = variant_count::<CompactedAccess>();
    // One tag per direct mode, plus one for each non-direct kind.
    const TAGS: usize = DIRECT_MODES + ACCESS_KINDS - 1;

    let direct = |mode| {
        CompactedAccess::Direct(VerifiedDirectAccessData {
            tensor: 0,
            mode,
            domain: Vec::new(),
            coordinates: Vec::new(),
            bounds_proof: None,
            bounds_facts: IndexDomainFactSource::Program,
            ownership_proof: None,
        })
    };
    let accesses: [CompactedAccess; TAGS] = [
        direct(crate::index::AccessMode::Read),
        direct(crate::index::AccessMode::Write),
        CompactedAccess::GatherRead(Box::new(CompactedGatherReadAccess {
            source: 0,
            index: 0,
            axis: 0,
            domain: Vec::new(),
            source_coordinates: Vec::new(),
            index_coordinates: Vec::new(),
        })),
    ];

    let tag = |access: &CompactedAccess| {
        let region = CompactedRegion {
            dimensions: Vec::new(),
            tensors: Vec::new(),
            expressions: Vec::new(),
            accesses: vec![access.clone()],
            index_domain_evidence: Vec::new(),
            unknown_index_domain_predicates: Vec::new(),
            operations: Vec::new(),
            values: Vec::new(),
            outputs: Vec::new(),
        };
        let bytes = encode_region(&region, None, encoded_region_len(&region, None));
        // The access run follows the domain separator, the absent-environment
        // marker, and three empty length frames.
        let start = super::INDEX_REGION_DOMAIN.len() + 1 + 8 * 3 + 8;
        bytes.as_bytes()[start]
    };

    let tags: Vec<u8> = accesses.iter().map(tag).collect();
    assert_eq!(
        tags,
        vec![1, 2, 3],
        "the direct read and write tags are frozen and the gather takes the \
         next free value",
    );
    for (position, left) in tags.iter().enumerate() {
        for right in &tags[position + 1..] {
            assert_ne!(
                left, right,
                "two access kinds encoding to one tag would make the region \
                 encoder non-injective",
            );
        }
    }
}

/// The gather access frame's exact bytes, field by field.
///
/// A behavioural comparison cannot catch a *consistent* reordering of this
/// frame: the source must be F32 and the index U32, so no two constructible
/// regions differ only by which tensor plays which role, and swapping the two
/// writes in the encoder relabels every gather identically. Only an exact byte
/// pin distinguishes "source then index" from "index then source", which is why
/// the field order is pinned here rather than inferred from a pair of regions.
#[test]
fn the_gather_access_frame_pins_its_exact_field_order() {
    let region = CompactedRegion {
        dimensions: Vec::new(),
        tensors: Vec::new(),
        expressions: Vec::new(),
        accesses: vec![CompactedAccess::GatherRead(Box::new(
            CompactedGatherReadAccess {
                source: 0x0000_0007,
                index: 0x0000_0009,
                axis: 0x0000_0002,
                domain: vec![4, 5],
                source_coordinates: vec![6],
                index_coordinates: vec![7, 8],
            },
        ))],
        index_domain_evidence: Vec::new(),
        unknown_index_domain_predicates: Vec::new(),
        operations: Vec::new(),
        values: Vec::new(),
        outputs: Vec::new(),
    };
    let bytes = encode_region(&region, None, encoded_region_len(&region, None));
    let start = super::INDEX_REGION_DOMAIN.len() + 1 + 8 * 3 + 8;
    // Tag, two ordinals, and the axis, then three framed runs. Sliced rather
    // than taken to the end, so the pin describes the access frame alone and
    // not the empty sections that follow it.
    let width = 13 + 8 + 4 * 2 + 8 + 4 + 8 + 4 * 2;
    let access = &bytes.as_bytes()[start..start + width];
    assert_eq!(
        access,
        &[
            // tag
            3, //
            // source tensor ordinal, then index tensor ordinal — distinct
            // values so a swap moves these bytes
            0, 0, 0, 7, //
            0, 0, 0, 9, //
            // gathered axis
            0, 0, 0, 2, //
            // domain: count then members
            0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 4, 0, 0, 0, 5, //
            // source coordinates: count then members
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 6, //
            // index coordinates: count then members
            0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 7, 0, 0, 0, 8,
        ],
        "the gather frame is tag, source, index, axis, domain, source \
         coordinates, index coordinates, in exactly that order",
    );
    // Four empty length frames follow the access run: the index-domain
    // assessments, operations, values, and outputs.
    assert_eq!(
        encoded_region_len(&region, None),
        start + width + 8 * 4,
        "the sizing helper accounts for the gather frame exactly as the encoder \
         writes it; a disagreement would trip `encode_region`'s own capacity \
         assertion before any identity was returned",
    );
}
