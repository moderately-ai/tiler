//! Checked construction of the bounded physical `f32` pointwise program.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use super::handles::InputOrdinal;

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
/// use tiler_ir::schedule::{InputOrdinal, PointwiseF32ExpressionBuilder, PointwiseF32Node};
///
/// fn spelling(node: &PointwiseF32Node) -> &'static str {
///     match node {
///         PointwiseF32Node::Input { .. } => "input",
///         PointwiseF32Node::Constant { .. } => "constant",
///         PointwiseF32Node::Add { .. } => "add",
///         PointwiseF32Node::Multiply { .. } => "multiply",
///         PointwiseF32Node::Divide { .. } => "divide",
///         PointwiseF32Node::Exp { .. } => "exp",
///         PointwiseF32Node::Rsqrt { .. } => "rsqrt",
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut builder = PointwiseF32ExpressionBuilder::new();
/// let input = builder.input(InputOrdinal::FIRST)?;
/// let expression = builder.build(input)?;
/// assert_eq!(spelling(&expression.nodes()[0]), "input");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PointwiseF32Node {
    /// The `f32` element one named boundary input contributes to this
    /// invocation.
    ///
    /// The ordinal says *which* of the region's input tensors is read, so two
    /// leaves reading two different tensors are two different nodes. A verified
    /// expression uses every ordinal in `0..input_count` exactly once.
    Input {
        /// Region-local ordinal of the input tensor this leaf reads.
        ordinal: InputOrdinal,
    },
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
    /// Ordered IEEE-754 binary32 division.
    ///
    /// One rounding, deliberately. A reciprocal followed by a multiply rounds
    /// twice and is a different binary32 function, so the two are separate nodes
    /// rather than one node with a permission — and this vocabulary has no
    /// reciprocal node at all, which is what makes the substitution unstatable
    /// here rather than merely forbidden.
    Divide {
        /// Dividend, defined before this node.
        lhs: PointwiseF32NodeId,
        /// Divisor, defined before this node.
        rhs: PointwiseF32NodeId,
    },
    /// The natural exponential over IEEE-754 binary32.
    ///
    /// The one node in this vocabulary whose result is not a rational function of
    /// its operands, so it is the one whose admitted result set comes from a
    /// registered accuracy contract rather than from IEEE-754 alone. The node
    /// names the *precise* function; an approximate realization is a different
    /// obligation and this vocabulary cannot spell it.
    Exp {
        /// The function's argument, defined before this node.
        argument: PointwiseF32NodeId,
    },
    /// The reciprocal square root over IEEE-754 binary32.
    ///
    /// One node, deliberately, and there is no `Sqrt` node beside it: `1 / Sqrt(t)`
    /// rounds twice and is a different binary32 function from `Rsqrt(t)`, so the
    /// vocabulary that cannot spell the composition is the one that cannot
    /// exchange them. Like [`Self::Exp`] its admitted result set comes from the
    /// registered accuracy contract of the operation this expression realizes,
    /// never from IEEE-754 alone.
    Rsqrt {
        /// The function's argument, defined before this node.
        argument: PointwiseF32NodeId,
    },
}

/// An opaque, verified physical `f32` pointwise expression.
///
/// The retained nodes are in a deterministic root-first-derived topological
/// order, preserve operand order and DAG sharing, read every input ordinal in
/// `0..input_count` exactly once, contain no unreachable nodes, and remain
/// within [`MAX_POINTWISE_F32_EXPRESSION_NODES`]. Construction is available only
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

    /// Returns how many distinct boundary input tensors this expression reads.
    ///
    /// A verified expression's ordinals are dense from zero, so this is both the
    /// leaf count and one past the largest ordinal — which is what lets a region
    /// verifier pair it with a read-access count without a second derivation.
    #[must_use]
    pub fn input_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| matches!(node, PointwiseF32Node::Input { .. }))
            .count()
    }

    pub(super) fn is_valid(&self) -> bool {
        if self.nodes.is_empty() || self.nodes.len() > MAX_POINTWISE_F32_EXPRESSION_NODES {
            return false;
        }
        let mut ordinals = Vec::new();
        for (index, node) in self.nodes.iter().enumerate() {
            match node {
                PointwiseF32Node::Input { ordinal } => ordinals.push(ordinal.get()),
                PointwiseF32Node::Constant { .. } => {}
                PointwiseF32Node::Add { lhs, rhs }
                | PointwiseF32Node::Multiply { lhs, rhs }
                | PointwiseF32Node::Divide { lhs, rhs } => {
                    let Ok(index) = u32::try_from(index) else {
                        return false;
                    };
                    if lhs.0 >= index || rhs.0 >= index {
                        return false;
                    }
                }
                PointwiseF32Node::Exp { argument } | PointwiseF32Node::Rsqrt { argument } => {
                    let Ok(index) = u32::try_from(index) else {
                        return false;
                    };
                    if argument.0 >= index {
                        return false;
                    }
                }
            }
        }
        let Ok(root) = usize::try_from(self.root.0) else {
            return false;
        };
        !ordinals.is_empty()
            && input_ordinals_are_dense(&mut ordinals)
            && root < self.nodes.len()
            && reachable_nodes(&self.nodes, self.root)
                .iter()
                .all(|seen| *seen)
    }
}

/// Sorts `ordinals` and reports whether they are exactly `0..ordinals.len()`.
///
/// Density is the invariant that makes an ordinal a *position*: a region binds
/// one read access per ordinal in order, so a gap would name a buffer the
/// expression never reads and a repeat would leave one access unaddressed. It is
/// checked over the whole retained set at once rather than per insertion,
/// because a builder is free to author its leaves in any order.
fn input_ordinals_are_dense(ordinals: &mut [u32]) -> bool {
    ordinals.sort_unstable();
    ordinals
        .iter()
        .enumerate()
        .all(|(position, ordinal)| u32::try_from(position) == Ok(*ordinal))
}

#[derive(Clone, Debug)]
enum DraftNode {
    Input {
        ordinal: InputOrdinal,
    },
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
    Divide {
        lhs: PointwiseF32Value,
        rhs: PointwiseF32Value,
    },
    Exp {
        argument: PointwiseF32Value,
    },
    Rsqrt {
        argument: PointwiseF32Value,
    },
}

/// A checked builder for one bounded physical `f32` pointwise expression.
#[derive(Debug, Default)]
pub struct PointwiseF32ExpressionBuilder {
    owner: Arc<()>,
    nodes: Vec<DraftNode>,
    /// Draft ordinals of the input leaves already minted, keyed by input tensor.
    ///
    /// One leaf per tensor is an invariant of the vocabulary, not an
    /// optimization: two `Input` nodes naming one ordinal would be two distinct
    /// canonical nodes for one read, so an expression reading the same tensor
    /// twice would encode differently depending on how the author spelled it.
    inputs: Vec<(InputOrdinal, u32)>,
}

impl PointwiseF32ExpressionBuilder {
    /// Opens an empty physical `f32` expression builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the leaf reading one named boundary input tensor.
    ///
    /// Asking twice for the same ordinal returns the leaf already minted rather
    /// than a second one, so an expression that reads one tensor at several
    /// places shares that read — the same DAG sharing operand reuse gives any
    /// other value, and the reason two spellings of one program cannot encode
    /// differently.
    ///
    /// Ordinals may be requested in any order; whether the set they form is the
    /// dense `0..n` a region can bind is a whole-expression property
    /// [`Self::build`] proves.
    ///
    /// # Errors
    ///
    /// Returns [`PointwiseF32ExpressionAdmissionError::StructuralLimit`] when a
    /// new leaf would exceed the governed node limit.
    pub fn input(
        &mut self,
        ordinal: InputOrdinal,
    ) -> Result<PointwiseF32Value, PointwiseF32ExpressionAdmissionError> {
        if let Some((_, index)) = self.inputs.iter().find(|(minted, _)| *minted == ordinal) {
            return Ok(PointwiseF32Value {
                owner: Arc::clone(&self.owner),
                index: *index,
            });
        }
        let value = self.push(DraftNode::Input { ordinal })?;
        self.inputs.push((ordinal, value.index));
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

    /// Adds an ordered binary32 `lhs / rhs`.
    ///
    /// One rounding. There is deliberately no reciprocal constructor beside it:
    /// `lhs * (1 / rhs)` rounds twice and is a different binary32 function, and an
    /// expression vocabulary that could spell both would let a rewrite exchange
    /// them.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or structural error when either operand was not
    /// already defined by this builder or the node limit is exhausted.
    pub fn divide(
        &mut self,
        lhs: PointwiseF32Value,
        rhs: PointwiseF32Value,
    ) -> Result<PointwiseF32Value, PointwiseF32ExpressionAdmissionError> {
        self.validate_operand(&lhs)?;
        self.validate_operand(&rhs)?;
        self.push(DraftNode::Divide { lhs, rhs })
    }

    /// Adds the binary32 natural exponential of `argument`.
    ///
    /// The *precise* exponential. What it may deliver is the resolved accuracy
    /// contract of the semantic operation this expression realizes, which is a
    /// property of that operation's registered identity rather than of this node.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or structural error when the operand was not
    /// already defined by this builder or the node limit is exhausted.
    pub fn exp(
        &mut self,
        argument: PointwiseF32Value,
    ) -> Result<PointwiseF32Value, PointwiseF32ExpressionAdmissionError> {
        self.validate_operand(&argument)?;
        self.push(DraftNode::Exp { argument })
    }

    /// Adds the binary32 reciprocal square root of `argument`.
    ///
    /// The *precise* reciprocal square root. What it may deliver is the resolved
    /// accuracy contract of the semantic operation this expression realizes,
    /// which is a property of that operation's registered identity rather than of
    /// this node. There is deliberately no `sqrt` constructor beside it: see
    /// [`PointwiseF32Node::Rsqrt`].
    ///
    /// # Errors
    ///
    /// Returns a typed handle or structural error when the operand was not
    /// already defined by this builder or the node limit is exhausted.
    pub fn rsqrt(
        &mut self,
        argument: PointwiseF32Value,
    ) -> Result<PointwiseF32Value, PointwiseF32ExpressionAdmissionError> {
        self.validate_operand(&argument)?;
        self.push(DraftNode::Rsqrt { argument })
    }

    /// Verifies, canonically orders, and freezes the expression under `root`.
    ///
    /// # Errors
    ///
    /// Rejects an empty expression, a missing input, a root not minted by this
    /// builder, any draft node not reachable from the explicit root, or a
    /// reachable input ordinal set that is not the dense `0..n`.
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
        if self.inputs.is_empty() {
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
        // Every retained node is reachable by the check above, so the ordinals
        // gathered here are exactly the ones a region would have to bind.
        let mut ordinals: Vec<u32> = nodes
            .iter()
            .filter_map(|node| match node {
                PointwiseF32Node::Input { ordinal } => Some(ordinal.get()),
                PointwiseF32Node::Constant { .. }
                | PointwiseF32Node::Add { .. }
                | PointwiseF32Node::Multiply { .. }
                | PointwiseF32Node::Divide { .. }
                | PointwiseF32Node::Exp { .. }
                | PointwiseF32Node::Rsqrt { .. } => None,
            })
            .collect();
        if !input_ordinals_are_dense(&mut ordinals) {
            let missing = ordinals
                .iter()
                .enumerate()
                .find(|(position, ordinal)| u32::try_from(*position) != Ok(**ordinal))
                .map_or(0, |(position, _)| {
                    u32::try_from(position).unwrap_or(u32::MAX)
                });
            return Err(PointwiseF32ExpressionBuildError::new(
                self,
                PointwiseF32ExpressionDiagnostic::SparseInputOrdinals { missing },
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
                DraftNode::Add { lhs, rhs }
                | DraftNode::Multiply { lhs, rhs }
                | DraftNode::Divide { lhs, rhs } => {
                    // The stack is LIFO: enqueue the right operand first so the
                    // left operand receives the earlier canonical ordinal.
                    pending.push((rhs.index, false));
                    pending.push((lhs.index, false));
                }
                DraftNode::Exp { argument } | DraftNode::Rsqrt { argument } => {
                    pending.push((argument.index, false));
                }
                DraftNode::Input { .. } | DraftNode::Constant { .. } => {}
            }
            continue;
        }
        let resolve = |value: &PointwiseF32Value| {
            canonical_ids[usize::try_from(value.index).expect("draft node ordinal is bounded")]
                .expect("operands are canonicalized before their user")
        };
        let node = match &draft[draft_index] {
            DraftNode::Input { ordinal } => PointwiseF32Node::Input { ordinal: *ordinal },
            DraftNode::Constant { bits } => PointwiseF32Node::Constant { bits: *bits },
            DraftNode::Add { lhs, rhs } => PointwiseF32Node::Add {
                lhs: resolve(lhs),
                rhs: resolve(rhs),
            },
            DraftNode::Multiply { lhs, rhs } => PointwiseF32Node::Multiply {
                lhs: resolve(lhs),
                rhs: resolve(rhs),
            },
            DraftNode::Divide { lhs, rhs } => PointwiseF32Node::Divide {
                lhs: resolve(lhs),
                rhs: resolve(rhs),
            },
            DraftNode::Rsqrt { argument } => PointwiseF32Node::Rsqrt {
                argument: resolve(argument),
            },
            DraftNode::Exp { argument } => PointwiseF32Node::Exp {
                argument: resolve(argument),
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
            PointwiseF32Node::Add { lhs, rhs }
            | PointwiseF32Node::Multiply { lhs, rhs }
            | PointwiseF32Node::Divide { lhs, rhs } => {
                pending.push(*lhs);
                pending.push(*rhs);
            }
            PointwiseF32Node::Exp { argument } | PointwiseF32Node::Rsqrt { argument } => {
                pending.push(*argument);
            }
            PointwiseF32Node::Input { .. } | PointwiseF32Node::Constant { .. } => {}
        }
    }
    reachable
}

/// Local failure while authoring one physical `f32` pointwise expression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PointwiseF32ExpressionAdmissionError {
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
    /// The reachable input ordinals are not the dense set `0..n`.
    ///
    /// A region binds one read access per ordinal, in order, so an expression
    /// that skips an ordinal names a binding position nothing reads. Rejecting
    /// it here keeps that from becoming a region the schedule verifier has to
    /// discover cannot be bound.
    SparseInputOrdinals {
        /// Smallest ordinal below the largest one the expression never reads.
        missing: u32,
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
            Self::SparseInputOrdinals { .. } => "pointwise-f32-sparse-input-ordinals",
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
        let input = builder.input(InputOrdinal::FIRST).unwrap();
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
                PointwiseF32Node::Input {
                    ordinal: InputOrdinal::FIRST
                },
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
            let input = builder.input(InputOrdinal::FIRST).unwrap();
            let two = builder.constant(2.0_f32.to_bits()).unwrap();
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            let add = builder.add(input.clone(), two).unwrap();
            let multiply = builder.multiply(input, three).unwrap();
            let root = builder.add(add, multiply).unwrap();
            builder.build(root).unwrap()
        }
        fn second() -> PointwiseF32Expression {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input(InputOrdinal::FIRST).unwrap();
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
        let input = builder.input(InputOrdinal::FIRST).unwrap();
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

    /// Asking twice for one ordinal shares a leaf; two ordinals are two leaves.
    ///
    /// The first half is what keeps identity a function of the program rather
    /// than of how it was authored, and the second is the widening itself: the
    /// vocabulary that refused a second input now separates two tensors.
    #[test]
    fn repeated_ordinals_share_a_leaf_and_distinct_ordinals_do_not() {
        let mut shared = PointwiseF32ExpressionBuilder::new();
        let first = shared.input(InputOrdinal::FIRST).unwrap();
        let again = shared.input(InputOrdinal::FIRST).unwrap();
        let root = shared.multiply(first, again).unwrap();
        let expression = shared.build(root).unwrap();
        assert_eq!(expression.input_count(), 1);
        assert_eq!(
            expression.nodes(),
            [
                PointwiseF32Node::Input {
                    ordinal: InputOrdinal::FIRST
                },
                PointwiseF32Node::Multiply {
                    lhs: PointwiseF32NodeId(0),
                    rhs: PointwiseF32NodeId(0),
                },
            ]
        );

        let mut distinct = PointwiseF32ExpressionBuilder::new();
        let a = distinct.input(InputOrdinal::new(0)).unwrap();
        let b = distinct.input(InputOrdinal::new(1)).unwrap();
        let c = distinct.input(InputOrdinal::new(2)).unwrap();
        let product = distinct.multiply(a, b).unwrap();
        let root = distinct.add(product, c).unwrap();
        let expression = distinct.build(root).unwrap();
        assert!(expression.is_valid());
        assert_eq!(expression.input_count(), 3);
        assert_eq!(
            expression.nodes(),
            [
                PointwiseF32Node::Input {
                    ordinal: InputOrdinal::new(0)
                },
                PointwiseF32Node::Input {
                    ordinal: InputOrdinal::new(1)
                },
                PointwiseF32Node::Multiply {
                    lhs: PointwiseF32NodeId(0),
                    rhs: PointwiseF32NodeId(1),
                },
                PointwiseF32Node::Input {
                    ordinal: InputOrdinal::new(2)
                },
                PointwiseF32Node::Add {
                    lhs: PointwiseF32NodeId(2),
                    rhs: PointwiseF32NodeId(3),
                },
            ]
        );
    }

    /// A gap in the reachable ordinals names a binding nothing reads.
    #[test]
    fn a_sparse_input_ordinal_set_is_refused_with_the_missing_ordinal() {
        let mut sparse = PointwiseF32ExpressionBuilder::new();
        let first = sparse.input(InputOrdinal::new(0)).unwrap();
        let third = sparse.input(InputOrdinal::new(2)).unwrap();
        let root = sparse.add(first, third).unwrap();
        let retained_root = root.clone();
        let error = sparse.build(root).unwrap_err();
        assert_eq!(
            error.diagnostic(),
            PointwiseF32ExpressionDiagnostic::SparseInputOrdinals { missing: 1 }
        );
        // Amending the recovered builder to read the skipped ordinal is
        // admitted, so the refusal is of the gap and not of the second input.
        let (mut recovered, _) = error.into_parts();
        let second = recovered.input(InputOrdinal::new(1)).unwrap();
        let repaired_root = recovered.add(retained_root, second).unwrap();
        let expression = recovered.build(repaired_root).unwrap();
        assert_eq!(expression.input_count(), 3);
    }

    #[test]
    fn missing_inputs_are_typed_errors() {
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
        let input = recovered.input(InputOrdinal::FIRST).unwrap();
        let repaired_root = recovered.add(input, retained_root).unwrap();
        assert!(recovered.build(repaired_root).is_ok());
    }

    #[test]
    fn foreign_forward_and_invalid_root_values_are_typed_errors() {
        let mut first = PointwiseF32ExpressionBuilder::new();
        let input = first.input(InputOrdinal::FIRST).unwrap();
        let mut second = PointwiseF32ExpressionBuilder::new();
        let foreign = second.input(InputOrdinal::FIRST).unwrap();
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
        let input = unreachable.input(InputOrdinal::FIRST).unwrap();
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
