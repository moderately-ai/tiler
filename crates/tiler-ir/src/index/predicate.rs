//! Typed vocabulary for residual index-domain predicates.
//!
//! The vocabulary references verified region entities instead of defining a
//! second index-expression or extent language.

use super::handles::VerifiedRegionOwner;
use super::{
    IndexEntityKind, IndexExprClass, VerifiedDimensionId, VerifiedIndexExprId,
    VerifiedIndexHandleError, VerifiedTensorAccessId, VerifiedTensorId,
};

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

    pub(super) fn checked(
        owner: VerifiedRegionOwner,
        subject: VerifiedTensorAccessId,
        predicate: IndexDomainPredicate,
        evidence: IndexDomainEvidence,
    ) -> Result<Option<Self>, VerifiedIndexHandleError> {
        if subject.owner != owner {
            return Err(VerifiedIndexHandleError::ForeignRegion {
                entity: IndexEntityKind::TensorAccess,
            });
        }
        let expression = match predicate {
            IndexDomainPredicate::NonNegative { expression }
            | IndexDomainPredicate::LessThanExtent { expression, .. } => expression,
        };
        if expression.owner != owner {
            return Err(VerifiedIndexHandleError::ForeignRegion {
                entity: IndexEntityKind::IndexExpression,
            });
        }
        if let IndexDomainPredicate::LessThanExtent { extent, .. } = predicate {
            let (extent_owner, entity) = match extent {
                IndexExtentRef::Dimension(dimension) => {
                    (dimension.owner, IndexEntityKind::Dimension)
                }
                IndexExtentRef::TensorAxis { tensor, .. } => {
                    (tensor.owner, IndexEntityKind::Tensor)
                }
            };
            if extent_owner != owner {
                return Err(VerifiedIndexHandleError::ForeignRegion { entity });
            }
        }
        match evidence {
            IndexDomainEvidence::SoundProof(_)
            | IndexDomainEvidence::ExhaustiveFinite { .. }
            | IndexDomainEvidence::Empirical => Ok(Some(Self {
                subject,
                predicate,
                evidence,
            })),
            IndexDomainEvidence::Unknown => Ok(None),
        }
    }
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
        IndexExprClass::Affine | IndexExprClass::QuasiAffine => true,
    }
}

#[cfg(test)]
mod tests {
    use super::super::handles::{VerifiedDimensionId, VerifiedIndexExprId, VerifiedTensorAccessId};
    use super::super::handles::{VerifiedTensorId, next_builder_id};
    use super::{
        DischargedIndexDomainPredicate, IndexDomainEvidence, IndexDomainPredicate,
        IndexDomainSoundProof, IndexExprClass, IndexExtentRef, expression_class_is_stateable,
    };

    #[test]
    fn every_admitted_expression_class_is_stateable() {
        assert!(expression_class_is_stateable(IndexExprClass::Affine));
        assert!(expression_class_is_stateable(IndexExprClass::QuasiAffine));
    }

    #[test]
    fn every_evidence_class_has_an_explicit_discharge_disposition() {
        let owner = next_builder_id().expect("test owner").verified_owner();
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
            (IndexDomainEvidence::Empirical, true),
            (IndexDomainEvidence::Unknown, false),
        ];
        for (evidence, discharged) in cases {
            assert_eq!(
                DischargedIndexDomainPredicate::checked(owner, subject, predicate, evidence)
                    .expect("every handle belongs to the named region")
                    .is_some(),
                discharged
            );
        }
    }

    #[test]
    fn evidence_construction_refuses_every_foreign_handle_position() {
        let owner = next_builder_id().expect("test owner").verified_owner();
        let foreign = next_builder_id()
            .expect("foreign test owner")
            .verified_owner();
        let local_subject = VerifiedTensorAccessId::from_verified(owner, 0);
        let local_expression = VerifiedIndexExprId::from_verified(owner, 0);
        let local_tensor = VerifiedTensorId::from_verified(owner, 0);
        let evidence = IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval);

        let foreign_subject = DischargedIndexDomainPredicate::checked(
            owner,
            VerifiedTensorAccessId::from_verified(foreign, 0),
            IndexDomainPredicate::NonNegative {
                expression: local_expression,
            },
            evidence,
        );
        assert!(foreign_subject.is_err());

        let foreign_expression = DischargedIndexDomainPredicate::checked(
            owner,
            local_subject,
            IndexDomainPredicate::NonNegative {
                expression: VerifiedIndexExprId::from_verified(foreign, 0),
            },
            evidence,
        );
        assert!(foreign_expression.is_err());

        let foreign_tensor = DischargedIndexDomainPredicate::checked(
            owner,
            local_subject,
            IndexDomainPredicate::LessThanExtent {
                expression: local_expression,
                extent: IndexExtentRef::TensorAxis {
                    tensor: VerifiedTensorId::from_verified(foreign, 0),
                    axis: 0,
                },
            },
            evidence,
        );
        assert!(foreign_tensor.is_err());

        let foreign_dimension = DischargedIndexDomainPredicate::checked(
            owner,
            local_subject,
            IndexDomainPredicate::LessThanExtent {
                expression: local_expression,
                extent: IndexExtentRef::Dimension(VerifiedDimensionId::from_verified(foreign, 0)),
            },
            evidence,
        );
        assert!(foreign_dimension.is_err());

        let local = DischargedIndexDomainPredicate::checked(
            owner,
            local_subject,
            IndexDomainPredicate::LessThanExtent {
                expression: local_expression,
                extent: IndexExtentRef::TensorAxis {
                    tensor: local_tensor,
                    axis: 0,
                },
            },
            evidence,
        );
        assert!(local.expect("local handles are accepted").is_some());
    }
}
