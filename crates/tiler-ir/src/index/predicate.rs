//! Typed vocabulary for residual index-domain predicates.
//!
//! The vocabulary references verified region entities instead of defining a
//! second index-expression or extent language.

use super::{IndexExprClass, VerifiedDimensionId, VerifiedIndexExprId, VerifiedTensorId};

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
    use super::{IndexExprClass, expression_class_is_stateable};

    #[test]
    fn every_admitted_expression_class_is_stateable() {
        assert!(expression_class_is_stateable(IndexExprClass::Affine));
        assert!(expression_class_is_stateable(IndexExprClass::QuasiAffine));
    }
}
