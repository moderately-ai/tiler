//! Typed vocabulary for residual index-domain predicates.
//!
//! The vocabulary references verified region entities instead of defining a
//! second index-expression or extent language.

use super::handles::VerifiedRegionOwner;
use super::model::{CompactedAccess, TensorData};
use super::{
    IndexEntityKind, IndexExprClass, ProofResource, VerifiedDimensionId, VerifiedIndexExprId,
    VerifiedIndexHandleError, VerifiedTensorAccessId, VerifiedTensorId,
};

const INDEX_DOMAIN_OBLIGATION_KEY_DOMAIN: &[u8] = b"tiler.index-domain-obligation-key.v1\0";

/// Opaque canonical bytes for one obligation within its owning region.
///
/// This key is region-local because verified handle positions are local. Pair it
/// with the owning region or a refinement occurrence when using it for
/// correlation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalIndexDomainObligationKey(Vec<u8>);

impl CanonicalIndexDomainObligationKey {
    /// Returns canonical region-local key bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A region-owned extent named by a residual index-domain predicate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexExtentRef {
    /// The extent of a verified iteration-domain dimension.
    Dimension(VerifiedDimensionId),
    /// One axis extent of a verified tensor boundary.
    TensorAxis {
        /// Tensor whose shape owns the extent.
        tensor: VerifiedTensorId,
        /// Zero-based axis within the tensor shape.
        axis: u32,
    },
}

/// One atomic predicate over the canonical index-expression graph.
///
/// A region carries a list of these atoms as an implicit conjunction. The
/// vocabulary has no Boolean-expression escape hatch and no physical-guard
/// variant.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexDomainPredicate {
    /// The expression is greater than or equal to zero.
    NonNegative {
        /// Canonical verified expression being constrained.
        expression: VerifiedIndexExprId,
    },
    /// The expression is strictly less than a sourced region extent.
    LessThanExtent {
        /// Canonical verified expression being constrained.
        expression: VerifiedIndexExprId,
        /// Region entity from which the upper bound is sourced.
        extent: IndexExtentRef,
    },
}

/// How a sound proof discharged one index-domain predicate.
///
/// The variants name derivations rather than confidence levels. They have no
/// ordering, and a consumer must match the exact method when it matters.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IndexDomainSoundProof {
    /// The subject access visits no points, so the predicate holds vacuously.
    VacuousEmptyDomain,
    /// Exact interval propagation established the predicate.
    Interval,
    /// The coordinate is a domain dimension whose extent is proved equal to
    /// the boundary axis it addresses.
    ProvedExtentEquality,
}

/// Which facts one discharged index-domain predicate rested on.
///
/// # Why this is beside the evidence rather than inside it
///
/// [`IndexDomainEvidence`] names the *argument* that closed the predicate.
/// This names the *premises* that argument was allowed to read, and the two are
/// independent: interval propagation, the structural equality argument, a
/// vacuous domain, and a finite enumeration can each run over a wholly literal
/// region or over one whose extents, divisors, and coefficients are declared
/// symbols. Folding them into one vocabulary would multiply the variants and
/// force a consumer that cares about only one axis to enumerate the other.
///
/// # One-sided, and in the safe direction
///
/// [`Self::Program`] is the strong claim and is stated only when every extent,
/// divisor, and coefficient the obligation ranges over was written as a
/// literal. [`Self::ShapeEnvironment`] is the weak one: it says a declared
/// symbol participated, not that the environment's facts were *needed*. An
/// access whose axis is spelled `m` in an environment that pins `m == 4` reports
/// [`Self::ShapeEnvironment`] even though the same region spelled `[4]` would
/// have proved the same bound from the program alone — because the two are
/// different programs, and this reports what the proof read rather than what a
/// differently spelled neighbour could have avoided reading.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IndexDomainFactSource {
    /// The region's own literals sufficed.
    ///
    /// No shape environment was consulted, so the proof survives any
    /// environment the region might later be resolved against — including none.
    Program,
    /// This region's shape environment supplied a premise.
    ///
    /// The proof holds under the environment the region's identity names and is
    /// not a claim about the same structure resolved anywhere else. That is
    /// sound because a region folds its environment's identity into its own:
    /// two regions spelled identically over differently constrained
    /// environments are different regions, so a fact read from the environment
    /// is a fact about *this* region.
    ShapeEnvironment,
}

impl IndexDomainFactSource {
    /// Returns the governed tag of this source, exhaustively.
    ///
    /// Written by a match rather than read from the discriminant, for the
    /// reason [`SourcedExtent`](crate::shape::SourcedExtent)'s own tag gives: adding a
    /// source is a build error here instead of a silent re-encoding of every
    /// region identity ever derived (ADR 0074 convention 3).
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::Program => 0x01,
            Self::ShapeEnvironment => 0x02,
        }
    }
}

/// Evidence class attached to one exact index-domain predicate.
///
/// This is an exhaustive maturity vocabulary, not a confidence scale.
/// [`Unknown`](Self::Unknown) is deliberately present beside, rather than
/// inside, the three evidence-bearing classes. An unknown predicate requires a
/// separate structured reason and cannot construct a
/// [`DischargedIndexDomainPredicate`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IndexDomainEvidence {
    /// A sound derivation established the predicate over its complete domain.
    SoundProof(IndexDomainSoundProof),
    /// Every point in a precisely bounded finite domain was checked.
    ExhaustiveFinite {
        /// Number of domain points evaluated.
        points: u64,
    },
    /// Measurement under a named profile, distinct from proof.
    ///
    /// Reserved: the current index verifier has no empirical proof lane and
    /// therefore never emits this variant.
    Empirical,
    /// Neither the predicate nor its negation was established.
    ///
    /// This is not evidence. It remains in the exhaustive vocabulary so a
    /// caller cannot accidentally collapse an unresolved obligation into one
    /// of the three evidence-bearing classes.
    Unknown,
}

/// Why neither an index-domain predicate nor its negation was established.
///
/// These are proof outcomes, not evidence classes, confidence levels, or
/// permission to insert a physical guard.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IndexDomainUnknownReason {
    /// The admitted facts permit models on both sides of the predicate.
    InsufficientFacts,
    /// The current proof engine does not decide this admitted expression
    /// fragment.
    UnsupportedFragment,
    /// A deterministic proof lane stopped at its governed resource limit.
    ResourceLimit {
        /// Exact exhausted proof resource.
        resource: ProofResource,
        /// Amount the proof would have required.
        required: u128,
        /// Configured maximum amount.
        limit: u64,
    },
}

/// One region-minted record that an exact predicate over an exact access was
/// discharged.
///
/// Fields are private because assembling matching handles and an evidence
/// label is not proof. Only the checked verified-region lifecycle mints these
/// records.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DischargedIndexDomainPredicate {
    pub(super) subject: VerifiedTensorAccessId,
    pub(super) predicate: IndexDomainPredicate,
    pub(super) evidence: IndexDomainEvidence,
    pub(super) facts: IndexDomainFactSource,
}

/// Structural authority used while minting region-owned predicate records.
#[derive(Clone, Copy)]
pub(super) struct IndexDomainPredicateContext<'a> {
    owner: VerifiedRegionOwner,
    accesses: &'a [CompactedAccess],
    tensors: &'a [TensorData],
    expression_count: usize,
    dimension_count: usize,
}

impl<'a> IndexDomainPredicateContext<'a> {
    pub(super) const fn new(
        owner: VerifiedRegionOwner,
        accesses: &'a [CompactedAccess],
        tensors: &'a [TensorData],
        expression_count: usize,
        dimension_count: usize,
    ) -> Self {
        Self {
            owner,
            accesses,
            tensors,
            expression_count,
            dimension_count,
        }
    }
}

impl DischargedIndexDomainPredicate {
    /// Returns the verified access whose iteration domain the predicate was
    /// proved over.
    #[must_use]
    pub const fn subject(self) -> VerifiedTensorAccessId {
        self.subject
    }

    /// Returns the exact discharged predicate.
    #[must_use]
    pub const fn predicate(self) -> IndexDomainPredicate {
        self.predicate
    }

    /// Returns how the predicate was established.
    #[must_use]
    pub const fn evidence(self) -> IndexDomainEvidence {
        self.evidence
    }

    /// Returns which facts the argument named by [`Self::evidence`] rested on.
    ///
    /// This is the answer to "would this proof still hold if the shape
    /// environment were not there", and it is recorded rather than left for a
    /// consumer to re-derive. Deriving it means walking every domain extent,
    /// every boundary axis, and every coordinate's transitive divisors and
    /// coefficients looking for a declared symbol; a consumer that forgot one
    /// of those populations would conclude "literal, therefore
    /// environment-independent" and be silently wrong about which proofs a
    /// changed environment invalidates.
    #[must_use]
    pub const fn facts(self) -> IndexDomainFactSource {
        self.facts
    }

    pub(super) fn checked(
        context: IndexDomainPredicateContext<'_>,
        subject: VerifiedTensorAccessId,
        predicate: IndexDomainPredicate,
        evidence: IndexDomainEvidence,
        facts: IndexDomainFactSource,
    ) -> Result<Option<Self>, VerifiedIndexHandleError> {
        check_predicate_handles(context, subject, predicate)?;
        match evidence {
            IndexDomainEvidence::SoundProof(_) | IndexDomainEvidence::ExhaustiveFinite { .. } => {
                Ok(Some(Self {
                    subject,
                    predicate,
                    evidence,
                    facts,
                }))
            }
            IndexDomainEvidence::Empirical | IndexDomainEvidence::Unknown => Ok(None),
        }
    }
}

/// One region-minted residual predicate requiring semantic discharge.
///
/// The record carries no physical-guard or runtime-check representation.
/// Downstream code must either establish the predicate or perform a named
/// semantic discharge before program work.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnknownIndexDomainPredicate {
    pub(super) subject: VerifiedTensorAccessId,
    pub(super) predicate: IndexDomainPredicate,
    pub(super) reason: IndexDomainUnknownReason,
}

impl UnknownIndexDomainPredicate {
    /// Returns the verified access whose iteration domain owns the obligation.
    #[must_use]
    pub const fn subject(self) -> VerifiedTensorAccessId {
        self.subject
    }

    /// Returns the exact unresolved predicate.
    #[must_use]
    pub const fn predicate(self) -> IndexDomainPredicate {
        self.predicate
    }

    /// Returns why the verifier established neither the predicate nor its
    /// negation.
    #[must_use]
    pub const fn reason(self) -> IndexDomainUnknownReason {
        self.reason
    }

    /// Returns a canonical region-local key over the exact predicate and reason.
    #[must_use]
    pub fn canonical_local_key(self) -> CanonicalIndexDomainObligationKey {
        let mut output = INDEX_DOMAIN_OBLIGATION_KEY_DOMAIN.to_vec();
        encode_index_domain_subject_predicate(&mut output, self.subject, self.predicate);
        encode_index_domain_unknown_reason(&mut output, self.reason);
        CanonicalIndexDomainObligationKey(output)
    }

    pub(super) fn checked(
        context: IndexDomainPredicateContext<'_>,
        subject: VerifiedTensorAccessId,
        predicate: IndexDomainPredicate,
        reason: IndexDomainUnknownReason,
    ) -> Result<Self, VerifiedIndexHandleError> {
        check_predicate_handles(context, subject, predicate)?;
        Ok(Self {
            subject,
            predicate,
            reason,
        })
    }
}

pub(super) fn encode_index_domain_subject_predicate(
    output: &mut Vec<u8>,
    subject: VerifiedTensorAccessId,
    predicate: IndexDomainPredicate,
) {
    output.extend_from_slice(&subject.index.to_be_bytes());
    match predicate {
        IndexDomainPredicate::NonNegative { expression } => {
            output.push(1);
            output.extend_from_slice(&expression.index.to_be_bytes());
        }
        IndexDomainPredicate::LessThanExtent { expression, extent } => {
            output.push(2);
            output.extend_from_slice(&expression.index.to_be_bytes());
            match extent {
                IndexExtentRef::Dimension(dimension) => {
                    output.push(1);
                    output.extend_from_slice(&dimension.index.to_be_bytes());
                }
                IndexExtentRef::TensorAxis { tensor, axis } => {
                    output.push(2);
                    output.extend_from_slice(&tensor.index.to_be_bytes());
                    output.extend_from_slice(&axis.to_be_bytes());
                }
            }
        }
    }
}

pub(super) fn encode_index_domain_unknown_reason(
    output: &mut Vec<u8>,
    reason: IndexDomainUnknownReason,
) {
    match reason {
        IndexDomainUnknownReason::InsufficientFacts => output.push(1),
        IndexDomainUnknownReason::UnsupportedFragment => output.push(2),
        IndexDomainUnknownReason::ResourceLimit {
            resource,
            required,
            limit,
        } => {
            output.push(3);
            output.push(match resource {
                ProofResource::Cells => 1,
                ProofResource::IntegerBytes => 2,
            });
            output.extend_from_slice(&required.to_be_bytes());
            output.extend_from_slice(&limit.to_be_bytes());
        }
    }
}

fn check_predicate_handles(
    context: IndexDomainPredicateContext<'_>,
    subject: VerifiedTensorAccessId,
    predicate: IndexDomainPredicate,
) -> Result<(), VerifiedIndexHandleError> {
    let IndexDomainPredicateContext {
        owner,
        accesses,
        tensors,
        expression_count,
        dimension_count,
    } = context;
    if subject.owner != owner {
        return Err(VerifiedIndexHandleError::ForeignRegion {
            entity: IndexEntityKind::TensorAccess,
        });
    }
    let access = accesses
        .get(subject.as_usize())
        .and_then(CompactedAccess::direct)
        .ok_or(VerifiedIndexHandleError::InvalidHandle {
            entity: IndexEntityKind::TensorAccess,
        })?;
    let expression = match predicate {
        IndexDomainPredicate::NonNegative { expression }
        | IndexDomainPredicate::LessThanExtent { expression, .. } => expression,
    };
    if expression.owner != owner {
        return Err(VerifiedIndexHandleError::ForeignRegion {
            entity: IndexEntityKind::IndexExpression,
        });
    }
    if expression.as_usize() >= expression_count
        || !access
            .coordinates
            .contains(&u32::try_from(expression.as_usize()).expect("verified handles fit u32"))
    {
        return Err(VerifiedIndexHandleError::InvalidHandle {
            entity: IndexEntityKind::IndexExpression,
        });
    }
    if let IndexDomainPredicate::LessThanExtent { extent, .. } = predicate {
        match extent {
            IndexExtentRef::Dimension(dimension) => {
                if dimension.owner != owner {
                    return Err(VerifiedIndexHandleError::ForeignRegion {
                        entity: IndexEntityKind::Dimension,
                    });
                }
                let dimension_index =
                    u32::try_from(dimension.as_usize()).expect("verified handles fit u32");
                if dimension.as_usize() >= dimension_count
                    || !access.domain.contains(&dimension_index)
                {
                    return Err(VerifiedIndexHandleError::InvalidHandle {
                        entity: IndexEntityKind::Dimension,
                    });
                }
            }
            IndexExtentRef::TensorAxis { tensor, axis } => {
                if tensor.owner != owner {
                    return Err(VerifiedIndexHandleError::ForeignRegion {
                        entity: IndexEntityKind::Tensor,
                    });
                }
                let tensor_data = tensors.get(tensor.as_usize()).ok_or(
                    VerifiedIndexHandleError::InvalidHandle {
                        entity: IndexEntityKind::Tensor,
                    },
                )?;
                let axis =
                    usize::try_from(axis).map_err(|_| VerifiedIndexHandleError::InvalidHandle {
                        entity: IndexEntityKind::Tensor,
                    })?;
                let expression_index =
                    u32::try_from(expression.as_usize()).expect("verified handles fit u32");
                if tensor.as_usize() != access.tensor as usize
                    || axis >= tensor_data.shape.rank()
                    || access.coordinates.get(axis) != Some(&expression_index)
                {
                    return Err(VerifiedIndexHandleError::InvalidHandle {
                        entity: IndexEntityKind::Tensor,
                    });
                }
            }
        }
    }
    Ok(())
}

/// Confirms that every admitted expression class can be referenced without
/// translating it into a second expression vocabulary.
///
/// This match is intentionally exhaustive. Extending [`IndexExprClass`] must
/// stop here until the new class is deliberately admitted or rejected.
#[allow(
    dead_code,
    reason = "the draft constructor correspondence is exercised only by its unit test until obligations are retained"
)]
const fn expression_class_is_stateable(class: IndexExprClass) -> bool {
    match class {
        // A semi-affine coordinate is referenced by handle exactly as an affine
        // one is. What its class changes is whether a given *analysis* can
        // discharge the predicate — a disposition each pass records for itself,
        // as `IndexDomainEvidence::Unknown` — and not whether the obligation can
        // be stated at all. Answering `false` here would drop the obligation
        // rather than leave it open, which is the one outcome the retained
        // evidence exists to prevent.
        IndexExprClass::Affine | IndexExprClass::QuasiAffine | IndexExprClass::SemiAffine => true,
    }
}

#[cfg(test)]
mod tests {
    use super::super::handles::{VerifiedDimensionId, VerifiedIndexExprId, VerifiedTensorAccessId};
    use super::super::handles::{VerifiedTensorId, next_builder_id};
    use super::super::model::{CompactedAccess, TensorData, VerifiedDirectAccessData};
    use super::{
        DischargedIndexDomainPredicate, IndexDomainEvidence, IndexDomainFactSource,
        IndexDomainPredicate, IndexDomainPredicateContext, IndexDomainSoundProof, IndexExprClass,
        IndexExtentRef, expression_class_is_stateable,
    };
    use crate::index::TensorRole;
    use crate::semantic::{ResolvedValueType, TypeKey};
    use crate::shape::{Shape, SourcedShape};

    fn records_fixture() -> (Vec<CompactedAccess>, Vec<TensorData>) {
        (
            vec![CompactedAccess::Direct(VerifiedDirectAccessData {
                tensor: 0,
                mode: crate::index::AccessMode::Read,
                domain: vec![0],
                coordinates: vec![0],
                bounds_proof: None,
                bounds_facts: IndexDomainFactSource::Program,
                ownership_proof: None,
            })],
            vec![TensorData {
                role: TensorRole::Input,
                value_type: ResolvedValueType::nominal(
                    TypeKey::new("test", "predicate-value", 1).unwrap(),
                ),
                shape: SourcedShape::from_shape(Shape::from_dims([1])),
            }],
        )
    }

    #[test]
    fn every_admitted_expression_class_is_stateable() {
        assert!(expression_class_is_stateable(IndexExprClass::Affine));
        assert!(expression_class_is_stateable(IndexExprClass::QuasiAffine));
        assert!(expression_class_is_stateable(IndexExprClass::SemiAffine));
    }

    #[test]
    fn every_evidence_class_has_an_explicit_discharge_disposition() {
        let owner = next_builder_id().expect("test owner").verified_owner();
        let (accesses, tensors) = records_fixture();
        let subject = VerifiedTensorAccessId::from_verified(owner, 0);
        let expression = VerifiedIndexExprId::from_verified(owner, 0);
        let predicate = IndexDomainPredicate::LessThanExtent {
            expression,
            extent: IndexExtentRef::TensorAxis {
                tensor: VerifiedTensorId::from_verified(owner, 0),
                axis: 0,
            },
        };
        let cases = [
            (
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval),
                true,
            ),
            (IndexDomainEvidence::ExhaustiveFinite { points: 7 }, true),
            (IndexDomainEvidence::Empirical, false),
            (IndexDomainEvidence::Unknown, false),
        ];
        let context = IndexDomainPredicateContext::new(owner, &accesses, &tensors, 1, 1);
        for (evidence, discharged) in cases {
            assert_eq!(
                DischargedIndexDomainPredicate::checked(
                    context,
                    subject,
                    predicate,
                    evidence,
                    IndexDomainFactSource::Program,
                )
                .expect("every handle belongs to the named region")
                .is_some(),
                discharged
            );
        }
    }

    #[test]
    fn evidence_construction_refuses_every_foreign_handle_position() {
        let owner = next_builder_id().expect("test owner").verified_owner();
        let (accesses, tensors) = records_fixture();
        let foreign = next_builder_id()
            .expect("foreign test owner")
            .verified_owner();
        let local_subject = VerifiedTensorAccessId::from_verified(owner, 0);
        let local_expression = VerifiedIndexExprId::from_verified(owner, 0);
        let local_tensor = VerifiedTensorId::from_verified(owner, 0);
        let evidence = IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval);
        let context = IndexDomainPredicateContext::new(owner, &accesses, &tensors, 1, 1);

        let foreign_subject = DischargedIndexDomainPredicate::checked(
            context,
            VerifiedTensorAccessId::from_verified(foreign, 0),
            IndexDomainPredicate::NonNegative {
                expression: local_expression,
            },
            evidence,
            IndexDomainFactSource::Program,
        );
        assert!(foreign_subject.is_err());

        let foreign_expression = DischargedIndexDomainPredicate::checked(
            context,
            local_subject,
            IndexDomainPredicate::NonNegative {
                expression: VerifiedIndexExprId::from_verified(foreign, 0),
            },
            evidence,
            IndexDomainFactSource::Program,
        );
        assert!(foreign_expression.is_err());

        let foreign_tensor = DischargedIndexDomainPredicate::checked(
            context,
            local_subject,
            IndexDomainPredicate::LessThanExtent {
                expression: local_expression,
                extent: IndexExtentRef::TensorAxis {
                    tensor: VerifiedTensorId::from_verified(foreign, 0),
                    axis: 0,
                },
            },
            evidence,
            IndexDomainFactSource::Program,
        );
        assert!(foreign_tensor.is_err());

        let foreign_dimension = DischargedIndexDomainPredicate::checked(
            context,
            local_subject,
            IndexDomainPredicate::LessThanExtent {
                expression: local_expression,
                extent: IndexExtentRef::Dimension(VerifiedDimensionId::from_verified(foreign, 0)),
            },
            evidence,
            IndexDomainFactSource::Program,
        );
        assert!(foreign_dimension.is_err());

        let local = DischargedIndexDomainPredicate::checked(
            context,
            local_subject,
            IndexDomainPredicate::LessThanExtent {
                expression: local_expression,
                extent: IndexExtentRef::TensorAxis {
                    tensor: local_tensor,
                    axis: 0,
                },
            },
            evidence,
            IndexDomainFactSource::Program,
        );
        assert!(local.expect("local handles are accepted").is_some());
    }

    #[test]
    fn evidence_construction_refuses_handles_unrelated_to_the_subject_access() {
        let owner = next_builder_id().expect("test owner").verified_owner();
        let (accesses, mut tensors) = records_fixture();
        tensors.push(TensorData {
            role: TensorRole::Input,
            value_type: ResolvedValueType::nominal(
                TypeKey::new("test", "other-predicate-value", 1).unwrap(),
            ),
            shape: SourcedShape::from_shape(Shape::from_dims([1])),
        });
        let subject = VerifiedTensorAccessId::from_verified(owner, 0);
        let unrelated_expression = VerifiedIndexExprId::from_verified(owner, 1);
        let unrelated_tensor = VerifiedTensorId::from_verified(owner, 1);
        let evidence = IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval);

        assert!(
            DischargedIndexDomainPredicate::checked(
                IndexDomainPredicateContext::new(owner, &accesses, &tensors, 2, 1),
                subject,
                IndexDomainPredicate::NonNegative {
                    expression: unrelated_expression,
                },
                evidence,
                IndexDomainFactSource::Program,
            )
            .is_err()
        );
        assert!(
            DischargedIndexDomainPredicate::checked(
                IndexDomainPredicateContext::new(owner, &accesses, &tensors, 1, 1),
                subject,
                IndexDomainPredicate::LessThanExtent {
                    expression: VerifiedIndexExprId::from_verified(owner, 0),
                    extent: IndexExtentRef::TensorAxis {
                        tensor: unrelated_tensor,
                        axis: 0,
                    },
                },
                evidence,
                IndexDomainFactSource::Program,
            )
            .is_err()
        );
    }
}
