//! Checked construction of the bounded physical `f32` pointwise program.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Maximum nodes admitted by one physical `f32` pointwise expression.
pub const MAX_POINTWISE_F32_EXPRESSION_NODES: usize = 4_096;

/// An expression-local node identifier.
///
/// Identifiers are exposed only after the expression has been verified. They
/// are dense ordinals into [`PointwiseF32Expression::nodes`] and cannot be
/// constructed by callers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PointwiseF32NodeId(u32);

impl PointwiseF32NodeId {
    /// Returns the node's canonical topological ordinal.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// A builder-owned handle to one physical `f32` value.
///
/// A value can be cloned and passed back only to the builder that minted it.
/// The builder rejects values from another expression rather than interpreting
/// their coincidentally equal ordinal.
#[derive(Clone, Debug)]
pub struct PointwiseF32Value {
    owner: Arc<()>,
    index: u32,
}

/// One node in a verified physical `f32` pointwise expression.
///
/// This is deliberately a closed physical vocabulary, not a dtype-generic
/// scalar IR. Future conversions, predicates, integer overflow families,
/// mixed-precision operations, and compound values require their own verified
/// physical projections.
///
/// This enum is intentionally exhaustive. Schedule identity and structured
/// kernel lowering are total maps over it, so a new node must stop every such
/// map at compile time until the new physical meaning is encoded and lowered:
///
/// ```
/// use tiler_ir::schedule::{PointwiseF32ExpressionBuilder, PointwiseF32Node};
///
/// fn spelling(node: &PointwiseF32Node) -> &'static str {
///     match node {
///         PointwiseF32Node::Input => "input",
///         PointwiseF32Node::Constant { .. } => "constant",
///         PointwiseF32Node::Add { .. } => "add",
///         PointwiseF32Node::Multiply { .. } => "multiply",
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut builder = PointwiseF32ExpressionBuilder::new();
/// let input = builder.input()?;
/// let expression = builder.build(input)?;
/// assert_eq!(spelling(&expression.nodes()[0]), "input");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PointwiseF32Node {
    /// The single `f32` tensor element read by this scalar invocation.
    Input,
    /// An exact IEEE-754 binary32 constant.
    Constant {
        /// The constant's exact bit pattern.
        bits: u32,
    },
    /// Ordered IEEE-754 binary32 addition.
    Add {
        /// Left operand, defined before this node.
        lhs: PointwiseF32NodeId,
        /// Right operand, defined before this node.
        rhs: PointwiseF32NodeId,
    },
    /// Ordered IEEE-754 binary32 multiplication.
    Multiply {
        /// Left operand, defined before this node.
        lhs: PointwiseF32NodeId,
        /// Right operand, defined before this node.
        rhs: PointwiseF32NodeId,
    },
}

/// An opaque, verified physical `f32` pointwise expression.
///
/// The retained nodes are in a deterministic root-first-derived topological
/// order, preserve operand order and DAG sharing, contain exactly one reachable
/// input, contain no unreachable nodes, and remain within
/// [`MAX_POINTWISE_F32_EXPRESSION_NODES`]. Construction is available only
/// through [`PointwiseF32ExpressionBuilder`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointwiseF32Expression {
    nodes: Box<[PointwiseF32Node]>,
    root: PointwiseF32NodeId,
}

impl PointwiseF32Expression {
    /// Returns the canonically ordered node vocabulary.
    #[must_use]
    pub fn nodes(&self) -> &[PointwiseF32Node] {
        &self.nodes
    }

    /// Returns the expression's explicit root.
    #[must_use]
    pub const fn root(&self) -> PointwiseF32NodeId {
        self.root
    }

    pub(super) fn is_valid(&self) -> bool {
        if self.nodes.is_empty() || self.nodes.len() > MAX_POINTWISE_F32_EXPRESSION_NODES {
            return false;
        }
        let mut inputs = 0_usize;
        for (index, node) in self.nodes.iter().enumerate() {
            match node {
                PointwiseF32Node::Input => inputs += 1,
                PointwiseF32Node::Constant { .. } => {}
                PointwiseF32Node::Add { lhs, rhs } | PointwiseF32Node::Multiply { lhs, rhs } => {
                    let Ok(index) = u32::try_from(index) else {
                        return false;
                    };
                    if lhs.0 >= index || rhs.0 >= index {
                        return false;
                    }
                }
            }
        }
        let Ok(root) = usize::try_from(self.root.0) else {
            return false;
        };
        inputs == 1
            && root < self.nodes.len()
            && reachable_nodes(&self.nodes, self.root)
                .iter()
                .all(|seen| *seen)
    }
}

#[derive(Clone, Debug)]
enum DraftNode {
    Input,
    Constant {
        bits: u32,
    },
    Add {
        lhs: PointwiseF32Value,
        rhs: PointwiseF32Value,
    },
    Multiply {
        lhs: PointwiseF32Value,
        rhs: PointwiseF32Value,
    },
}

/// A checked builder for one bounded physical `f32` pointwise expression.
#[derive(Debug, Default)]
pub struct PointwiseF32ExpressionBuilder {
    owner: Arc<()>,
    nodes: Vec<DraftNode>,
    has_input: bool,
}

impl PointwiseF32ExpressionBuilder {
    /// Opens an empty physical `f32` expression builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the expression's single tensor input.
    ///
    /// # Errors
    ///
    /// Returns [`PointwiseF32ExpressionAdmissionError::DuplicateInput`] for a
    /// second input or
    /// [`PointwiseF32ExpressionAdmissionError::StructuralLimit`] at the node
    /// limit.
    pub fn input(&mut self) -> Result<PointwiseF32Value, PointwiseF32ExpressionAdmissionError> {
        if self.has_input {
            return Err(PointwiseF32ExpressionAdmissionError::DuplicateInput);
        }
        let value = self.push(DraftNode::Input)?;
        self.has_input = true;
        Ok(value)
    }

    /// Adds an exact IEEE-754 binary32 constant.
    ///
    /// # Errors
    ///
    /// Rejects a node beyond the governed expression limit.
    pub fn constant(
        &mut self,
        bits: u32,
    ) -> Result<PointwiseF32Value, PointwiseF32ExpressionAdmissionError> {
        self.push(DraftNode::Constant { bits })
    }

    /// Adds an ordered binary32 `lhs + rhs`.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or structural error when either operand was not
    /// already defined by this builder or the node limit is exhausted.
    pub fn add(
        &mut self,
        lhs: PointwiseF32Value,
        rhs: PointwiseF32Value,
    ) -> Result<PointwiseF32Value, PointwiseF32ExpressionAdmissionError> {
        self.validate_operand(&lhs)?;
        self.validate_operand(&rhs)?;
        self.push(DraftNode::Add { lhs, rhs })
    }

    /// Adds an ordered binary32 `lhs * rhs`.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or structural error when either operand was not
    /// already defined by this builder or the node limit is exhausted.
    pub fn multiply(
        &mut self,
        lhs: PointwiseF32Value,
        rhs: PointwiseF32Value,
    ) -> Result<PointwiseF32Value, PointwiseF32ExpressionAdmissionError> {
        self.validate_operand(&lhs)?;
        self.validate_operand(&rhs)?;
        self.push(DraftNode::Multiply { lhs, rhs })
    }

    /// Verifies, canonically orders, and freezes the expression under `root`.
    ///
    /// # Errors
    ///
    /// Rejects an empty expression, a missing input, a root not minted by this
    /// builder, or any draft node not reachable from the explicit root.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "consuming the builder also consumes its builder-owned root value"
    )]
    pub fn build(
        self,
        root: PointwiseF32Value,
    ) -> Result<PointwiseF32Expression, PointwiseF32ExpressionBuildError> {
        if self.nodes.is_empty() {
            return Err(PointwiseF32ExpressionBuildError::new(
                self,
                PointwiseF32ExpressionDiagnostic::EmptyExpression,
            ));
        }
        if !Arc::ptr_eq(&root.owner, &self.owner)
            || usize::try_from(root.index)
                .ok()
                .is_none_or(|index| index >= self.nodes.len())
        {
            return Err(PointwiseF32ExpressionBuildError::new(
                self,
                PointwiseF32ExpressionDiagnostic::InvalidRoot,
            ));
        }
        if !self.has_input {
            return Err(PointwiseF32ExpressionBuildError::new(
                self,
                PointwiseF32ExpressionDiagnostic::MissingInput,
            ));
        }

        let mut canonical_ids = vec![None; self.nodes.len()];
        let mut nodes = Vec::with_capacity(self.nodes.len());
        canonicalize_nodes(root.index, &self.nodes, &mut canonical_ids, &mut nodes);
        if let Some(index) = canonical_ids.iter().position(Option::is_none) {
            return Err(PointwiseF32ExpressionBuildError::new(
                self,
                PointwiseF32ExpressionDiagnostic::UnreachableNode { index },
            ));
        }
        let root = usize::try_from(root.index)
            .ok()
            .and_then(|index| canonical_ids.get(index))
            .copied()
            .flatten()
            .ok_or_else(|| {
                PointwiseF32ExpressionBuildError::new(
                    self,
                    PointwiseF32ExpressionDiagnostic::InvalidRoot,
                )
            })?;
        Ok(PointwiseF32Expression {
            nodes: nodes.into_boxed_slice(),
            root,
        })
    }

    fn push(
        &mut self,
        node: DraftNode,
    ) -> Result<PointwiseF32Value, PointwiseF32ExpressionAdmissionError> {
        if self.nodes.len() >= MAX_POINTWISE_F32_EXPRESSION_NODES {
            return Err(PointwiseF32ExpressionAdmissionError::StructuralLimit {
                actual: self.nodes.len() + 1,
                limit: MAX_POINTWISE_F32_EXPRESSION_NODES,
            });
        }
        let index = u32::try_from(self.nodes.len()).map_err(|_| {
            PointwiseF32ExpressionAdmissionError::StructuralLimit {
                actual: self.nodes.len() + 1,
                limit: MAX_POINTWISE_F32_EXPRESSION_NODES,
            }
        })?;
        self.nodes.push(node);
        Ok(PointwiseF32Value {
            owner: Arc::clone(&self.owner),
            index,
        })
    }

    fn validate_operand(
        &self,
        operand: &PointwiseF32Value,
    ) -> Result<(), PointwiseF32ExpressionAdmissionError> {
        if !Arc::ptr_eq(&operand.owner, &self.owner) {
            return Err(PointwiseF32ExpressionAdmissionError::ForeignValue);
        }
        if usize::try_from(operand.index)
            .ok()
            .is_none_or(|index| index >= self.nodes.len())
        {
            return Err(PointwiseF32ExpressionAdmissionError::ForwardValue);
        }
        Ok(())
    }
}

fn canonicalize_nodes(
    root: u32,
    draft: &[DraftNode],
    canonical_ids: &mut [Option<PointwiseF32NodeId>],
    nodes: &mut Vec<PointwiseF32Node>,
) {
    let mut pending = vec![(root, false)];
    while let Some((draft_id, operands_visited)) = pending.pop() {
        let draft_index = usize::try_from(draft_id).expect("draft node ordinals are bounded");
        if canonical_ids[draft_index].is_some() {
            continue;
        }
        if !operands_visited {
            pending.push((draft_id, true));
            match &draft[draft_index] {
                DraftNode::Add { lhs, rhs } | DraftNode::Multiply { lhs, rhs } => {
                    // The stack is LIFO: enqueue the right operand first so the
                    // left operand receives the earlier canonical ordinal.
                    pending.push((rhs.index, false));
                    pending.push((lhs.index, false));
                }
                DraftNode::Input | DraftNode::Constant { .. } => {}
            }
            continue;
        }
        let resolve = |value: &PointwiseF32Value| {
            canonical_ids[usize::try_from(value.index).expect("draft node ordinal is bounded")]
                .expect("operands are canonicalized before their user")
        };
        let node = match &draft[draft_index] {
            DraftNode::Input => PointwiseF32Node::Input,
            DraftNode::Constant { bits } => PointwiseF32Node::Constant { bits: *bits },
            DraftNode::Add { lhs, rhs } => PointwiseF32Node::Add {
                lhs: resolve(lhs),
                rhs: resolve(rhs),
            },
            DraftNode::Multiply { lhs, rhs } => PointwiseF32Node::Multiply {
                lhs: resolve(lhs),
                rhs: resolve(rhs),
            },
        };
        let id = PointwiseF32NodeId(u32::try_from(nodes.len()).expect("node count is bounded"));
        nodes.push(node);
        canonical_ids[draft_index] = Some(id);
    }
}

fn reachable_nodes(nodes: &[PointwiseF32Node], root: PointwiseF32NodeId) -> Vec<bool> {
    let mut reachable = vec![false; nodes.len()];
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        let Some(seen) = usize::try_from(node.0)
            .ok()
            .and_then(|index| reachable.get_mut(index))
        else {
            continue;
        };
        if *seen {
            continue;
        }
        *seen = true;
        match &nodes[usize::try_from(node.0).expect("validated node ordinal")] {
            PointwiseF32Node::Add { lhs, rhs } | PointwiseF32Node::Multiply { lhs, rhs } => {
                pending.push(*lhs);
                pending.push(*rhs);
            }
            PointwiseF32Node::Input | PointwiseF32Node::Constant { .. } => {}
        }
    }
    reachable
}

/// Local failure while authoring one physical `f32` pointwise expression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PointwiseF32ExpressionAdmissionError {
    /// A second tensor input was requested.
    DuplicateInput,
    /// An operand value belongs to another builder.
    ForeignValue,
    /// An operand does not name an already-defined value.
    ForwardValue,
    /// The governed node limit was exceeded.
    StructuralLimit {
        /// Attempted node count.
        actual: usize,
        /// Maximum admitted node count.
        limit: usize,
    },
}

impl PointwiseF32ExpressionAdmissionError {
    /// Returns the stable construction-rule identifier.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::DuplicateInput => "pointwise-f32-duplicate-input",
            Self::ForeignValue => "pointwise-f32-foreign-value",
            Self::ForwardValue => "pointwise-f32-forward-value",
            Self::StructuralLimit { .. } => "pointwise-f32-structural-limit",
        }
    }
}

impl fmt::Display for PointwiseF32ExpressionAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}

impl Error for PointwiseF32ExpressionAdmissionError {}

/// Whole-expression verification failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PointwiseF32ExpressionDiagnostic {
    /// The expression has no nodes.
    EmptyExpression,
    /// The expression has no tensor input.
    MissingInput,
    /// The root does not name a value minted by this builder.
    InvalidRoot,
    /// A retained draft node is not reachable from the explicit root.
    UnreachableNode {
        /// Draft ordinal of the first unreachable node.
        index: usize,
    },
}

impl PointwiseF32ExpressionDiagnostic {
    /// Returns the stable whole-expression rule identifier.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::EmptyExpression => "pointwise-f32-empty-expression",
            Self::MissingInput => "pointwise-f32-missing-input",
            Self::InvalidRoot => "pointwise-f32-invalid-root",
            Self::UnreachableNode { .. } => "pointwise-f32-unreachable-node",
        }
    }
}

impl fmt::Display for PointwiseF32ExpressionDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}

impl Error for PointwiseF32ExpressionDiagnostic {}

/// Recoverable failure from consuming whole-expression verification.
///
/// Carries the deterministic diagnostic and returns the intact builder through
/// [`PointwiseF32ExpressionBuildError::into_parts`] so a caller can amend and
/// retry.
#[derive(Debug)]
pub struct PointwiseF32ExpressionBuildError {
    builder: Box<PointwiseF32ExpressionBuilder>,
    diagnostic: PointwiseF32ExpressionDiagnostic,
}

impl PointwiseF32ExpressionBuildError {
    fn new(
        builder: PointwiseF32ExpressionBuilder,
        diagnostic: PointwiseF32ExpressionDiagnostic,
    ) -> Self {
        Self {
            builder: Box::new(builder),
            diagnostic,
        }
    }

    /// Returns the deterministic whole-expression diagnostic.
    #[must_use]
    pub const fn diagnostic(&self) -> PointwiseF32ExpressionDiagnostic {
        self.diagnostic
    }

    /// Recovers the intact builder and its diagnostic.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        PointwiseF32ExpressionBuilder,
        PointwiseF32ExpressionDiagnostic,
    ) {
        (*self.builder, self.diagnostic)
    }
}

impl fmt::Display for PointwiseF32ExpressionBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pointwise-f32 expression verification failed: {}",
            self.diagnostic
        )
    }
}

impl Error for PointwiseF32ExpressionBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.diagnostic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_builder_retains_exact_topological_tree() {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input().unwrap();
        let two = builder.constant(2.0_f32.to_bits()).unwrap();
        let product = builder.multiply(input, two).unwrap();
        let one = builder.constant(1.0_f32.to_bits()).unwrap();
        let root = builder.add(product, one).unwrap();
        let expression = builder.build(root).unwrap();
        assert!(expression.is_valid());
        assert_eq!(expression.root().index(), 4);
        assert_eq!(
            expression.nodes(),
            [
                PointwiseF32Node::Input,
                PointwiseF32Node::Constant {
                    bits: 2.0_f32.to_bits()
                },
                PointwiseF32Node::Multiply {
                    lhs: PointwiseF32NodeId(0),
                    rhs: PointwiseF32NodeId(1),
                },
                PointwiseF32Node::Constant {
                    bits: 1.0_f32.to_bits()
                },
                PointwiseF32Node::Add {
                    lhs: PointwiseF32NodeId(2),
                    rhs: PointwiseF32NodeId(3),
                },
            ]
        );
    }

    #[test]
    fn independent_ready_node_insertion_order_canonicalizes_identically() {
        fn first() -> PointwiseF32Expression {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input().unwrap();
            let two = builder.constant(2.0_f32.to_bits()).unwrap();
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            let add = builder.add(input.clone(), two).unwrap();
            let multiply = builder.multiply(input, three).unwrap();
            let root = builder.add(add, multiply).unwrap();
            builder.build(root).unwrap()
        }
        fn second() -> PointwiseF32Expression {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input().unwrap();
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            let two = builder.constant(2.0_f32.to_bits()).unwrap();
            let multiply = builder.multiply(input.clone(), three).unwrap();
            let add = builder.add(input, two).unwrap();
            let root = builder.add(add, multiply).unwrap();
            builder.build(root).unwrap()
        }
        assert_eq!(first(), second());
    }

    #[test]
    fn canonicalization_preserves_dag_sharing() {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input().unwrap();
        let two = builder.constant(2.0_f32.to_bits()).unwrap();
        let shared = builder.multiply(input, two).unwrap();
        let root = builder.add(shared.clone(), shared).unwrap();
        let expression = builder.build(root).unwrap();
        assert_eq!(expression.nodes().len(), 4);
        assert!(matches!(
            expression.nodes()[3],
            PointwiseF32Node::Add {
                lhs: PointwiseF32NodeId(2),
                rhs: PointwiseF32NodeId(2),
            }
        ));
    }

    #[test]
    fn duplicate_and_missing_inputs_are_typed_errors() {
        let mut duplicate = PointwiseF32ExpressionBuilder::new();
        duplicate.input().unwrap();
        assert!(matches!(
            duplicate.input(),
            Err(PointwiseF32ExpressionAdmissionError::DuplicateInput)
        ));

        let mut missing = PointwiseF32ExpressionBuilder::new();
        let root = missing.constant(0).unwrap();
        let retained_root = root.clone();
        let error = missing.build(root).unwrap_err();
        assert_eq!(
            error.diagnostic(),
            PointwiseF32ExpressionDiagnostic::MissingInput
        );
        let (mut recovered, diagnostic) = error.into_parts();
        assert_eq!(diagnostic, PointwiseF32ExpressionDiagnostic::MissingInput);
        let input = recovered.input().unwrap();
        let repaired_root = recovered.add(input, retained_root).unwrap();
        assert!(recovered.build(repaired_root).is_ok());
    }

    #[test]
    fn foreign_forward_and_invalid_root_values_are_typed_errors() {
        let mut first = PointwiseF32ExpressionBuilder::new();
        let input = first.input().unwrap();
        let mut second = PointwiseF32ExpressionBuilder::new();
        let foreign = second.input().unwrap();
        assert!(matches!(
            first.add(input.clone(), foreign),
            Err(PointwiseF32ExpressionAdmissionError::ForeignValue)
        ));

        let forward = PointwiseF32Value {
            owner: Arc::clone(&first.owner),
            index: 99,
        };
        assert!(matches!(
            first.multiply(input.clone(), forward.clone()),
            Err(PointwiseF32ExpressionAdmissionError::ForwardValue)
        ));
        assert_eq!(
            first.build(forward).unwrap_err().diagnostic(),
            PointwiseF32ExpressionDiagnostic::InvalidRoot
        );
    }

    #[test]
    fn empty_unreachable_and_over_bound_expressions_are_typed_errors() {
        let empty = PointwiseF32ExpressionBuilder::new();
        let forged = PointwiseF32Value {
            owner: Arc::clone(&empty.owner),
            index: 0,
        };
        assert_eq!(
            empty.build(forged).unwrap_err().diagnostic(),
            PointwiseF32ExpressionDiagnostic::EmptyExpression
        );

        let mut unreachable = PointwiseF32ExpressionBuilder::new();
        let input = unreachable.input().unwrap();
        let root = unreachable.constant(0).unwrap();
        let retained_root = root.clone();
        let error = unreachable.build(root).unwrap_err();
        assert_eq!(
            error.diagnostic(),
            PointwiseF32ExpressionDiagnostic::UnreachableNode { index: 0 }
        );
        let (mut recovered, _) = error.into_parts();
        let repaired_root = recovered.add(input, retained_root).unwrap();
        assert!(recovered.build(repaired_root).is_ok());

        let mut bounded = PointwiseF32ExpressionBuilder::new();
        for _ in 0..MAX_POINTWISE_F32_EXPRESSION_NODES {
            bounded.constant(0).unwrap();
        }
        assert!(matches!(
            bounded.constant(0),
            Err(PointwiseF32ExpressionAdmissionError::StructuralLimit {
                actual,
                limit: MAX_POINTWISE_F32_EXPRESSION_NODES,
            }) if actual == MAX_POINTWISE_F32_EXPRESSION_NODES + 1
        ));
    }
}
