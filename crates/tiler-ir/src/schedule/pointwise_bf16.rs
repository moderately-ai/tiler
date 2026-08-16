//! Checked construction of the bounded physical `bf16` pointwise program.
//!
//! # A second expression type, not one parameterized over the width
//!
//! The sibling [`super::pointwise`] module states the same shape for `f32`, and
//! the two are deliberately separate types rather than one type carrying an
//! [`ArithmeticType`](super::ArithmeticType). Three facts decide it, and each of
//! them makes the parameterized form admit a program no registered operation
//! allows:
//!
//! - **The vocabularies are different sets.** `tiler-ir`'s standard registry
//!   registers exactly `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, and
//!   `tiler::add-bf16@1` for this width. It registers no `bf16` division and no
//!   `bf16` elementary function, and an elementary construct is admissible only
//!   once some registered operation's resolved accuracy contract says what it
//!   must deliver. One expression type parameterized over the width would make
//!   `Divide`, `Exp`, and `Rsqrt` spellable at `bf16`, which is an obligation
//!   nothing states.
//! - **The constant payload is the format's own width.** A `bf16` constant is
//!   sixteen bits and an `f32` constant is thirty-two. A shared node would have
//!   to carry the wider payload, which re-admits the over-wide `bf16` constant
//!   this vocabulary makes unrepresentable — see [`PointwiseBf16Node::Constant`].
//! - **Mixing must be unrepresentable rather than unreached.** A
//!   [`PointwiseBf16Value`] cannot be passed to an
//!   [`PointwiseF32ExpressionBuilder`](super::PointwiseF32ExpressionBuilder)
//!   method at all: the two builders take different value types, so a
//!   mixed-width expression is a type error rather than a check some verifier
//!   has to remember to run.
//!
//! What the separation costs is a second canonicalizer and a second validator
//! over four node kinds. That is the same trade the identity encoders already
//! make — `push_tensor_role` is duplicated between the schedule and kernel
//! domains on purpose — and it buys the same thing: each vocabulary's exhaustive
//! matches break on their own widening rather than on the other's.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use super::handles::AccessOrdinal;

/// Maximum nodes admitted by one physical `bf16` pointwise expression.
pub const MAX_POINTWISE_BF16_EXPRESSION_NODES: usize = 4_096;

/// An expression-local node identifier.
///
/// Identifiers are exposed only after the expression has been verified. They
/// are dense ordinals into [`PointwiseBf16Expression::nodes`] and cannot be
/// constructed by callers.
///
/// A distinct type from
/// [`PointwiseF32NodeId`](super::PointwiseF32NodeId), for the reason the module
/// documentation states: two ordinals into two different expressions are not
/// interchangeable, and giving them one type would let a `bf16` operand name an
/// `f32` node by numeric coincidence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PointwiseBf16NodeId(u32);

impl PointwiseBf16NodeId {
    /// Returns the node's canonical topological ordinal.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// A builder-owned handle to one physical `bf16` value.
///
/// A value can be cloned and passed back only to the builder that minted it.
/// The builder rejects values from another expression rather than interpreting
/// their coincidentally equal ordinal.
#[derive(Clone, Debug)]
pub struct PointwiseBf16Value {
    owner: Arc<()>,
    index: u32,
}

/// One node in a verified physical `bf16` pointwise expression.
///
/// This is deliberately a closed physical vocabulary covering exactly the three
/// registered `bf16` operations and nothing else. A conversion to or from
/// binary32, a division, an elementary function, a fused multiply-add, or an
/// accumulating reduction each requires its own registered operation and its own
/// verified physical projection; [ADR
/// 0091](../../../../docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md)
/// owns the first of those and this vocabulary deliberately cannot spell any of
/// them.
///
/// This enum is intentionally exhaustive. Schedule identity and structured
/// kernel lowering are total maps over it, so a new node must stop every such
/// map at compile time until the new physical meaning is encoded and lowered:
///
/// ```
/// use tiler_ir::schedule::{AccessOrdinal, PointwiseBf16ExpressionBuilder, PointwiseBf16Node};
///
/// fn spelling(node: &PointwiseBf16Node) -> &'static str {
///     match node {
///         PointwiseBf16Node::Input { .. } => "input",
///         PointwiseBf16Node::Constant { .. } => "constant",
///         PointwiseBf16Node::Add { .. } => "add",
///         PointwiseBf16Node::Multiply { .. } => "multiply",
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut builder = PointwiseBf16ExpressionBuilder::new();
/// let input = builder.input(AccessOrdinal::FIRST)?;
/// let expression = builder.build(input)?;
/// assert_eq!(spelling(&expression.nodes()[0]), "input");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PointwiseBf16Node {
    /// The `bf16` element one named boundary input contributes to this
    /// invocation.
    ///
    /// The access says *which* of the region's ordered reads is used, so two
    /// leaves reading two different tensors are two different nodes. A verified
    /// expression uses every ordinal in `0..input_count` exactly once.
    Input {
        /// Region-local access this leaf reads.
        access: AccessOrdinal,
    },
    /// An exact `bf16` constant.
    ///
    /// **Sixteen bits, which is what makes an over-wide payload unrepresentable
    /// rather than merely refused.** `tiler::constant-bf16@1` declares its
    /// payload rule as the exact bit pattern of the ratified RISC-V BF16 operand
    /// format, which is two bytes; a `u32` field would admit a thirty-two-bit
    /// payload that no `bf16` value has, and the refusal would then be a runtime
    /// check every producer could forget to reach. The type is the check:
    ///
    /// ```compile_fail,E0308
    /// use tiler_ir::schedule::PointwiseBf16ExpressionBuilder;
    /// let mut builder = PointwiseBf16ExpressionBuilder::new();
    /// // An `f32` bit pattern is not a `bf16` payload, and this does not compile.
    /// let _ = builder.constant(1.0_f32.to_bits());
    /// ```
    Constant {
        /// The constant's exact bit pattern.
        bits: u16,
    },
    /// Ordered `bf16` addition.
    ///
    /// `tiler::add-bf16@1`: one rounding at the observable materialization, no
    /// contraction, no reassociation, and the family's canonical arithmetic NaN
    /// payload at every result boundary.
    Add {
        /// Left operand, defined before this node.
        lhs: PointwiseBf16NodeId,
        /// Right operand, defined before this node.
        rhs: PointwiseBf16NodeId,
    },
    /// Ordered `bf16` multiplication.
    ///
    /// `tiler::multiply-bf16@1`, under the same rounding and permission facts
    /// [`Self::Add`] states. There is deliberately no fused multiply-add node
    /// beside the pair: the registered family declares
    /// `BF16_FACT_FUSED_MULTIPLY_ADD_PERMITTED` false, so a fused construct
    /// would be an operation no registered identity means.
    Multiply {
        /// Left operand, defined before this node.
        lhs: PointwiseBf16NodeId,
        /// Right operand, defined before this node.
        rhs: PointwiseBf16NodeId,
    },
}

/// An opaque, verified physical `bf16` pointwise expression.
///
/// The retained nodes are in a deterministic root-first-derived topological
/// order, preserve operand order and DAG sharing, read every access ordinal in
/// `0..input_count` exactly once, contain no unreachable nodes, and remain
/// within [`MAX_POINTWISE_BF16_EXPRESSION_NODES`]. Construction is available
/// only through [`PointwiseBf16ExpressionBuilder`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointwiseBf16Expression {
    nodes: Box<[PointwiseBf16Node]>,
    root: PointwiseBf16NodeId,
}

impl PointwiseBf16Expression {
    /// Returns the canonically ordered node vocabulary.
    #[must_use]
    pub fn nodes(&self) -> &[PointwiseBf16Node] {
        &self.nodes
    }

    /// Returns the expression's explicit root.
    #[must_use]
    pub const fn root(&self) -> PointwiseBf16NodeId {
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
            .filter(|node| matches!(node, PointwiseBf16Node::Input { .. }))
            .count()
    }

    pub(super) fn is_valid(&self) -> bool {
        if self.nodes.is_empty() || self.nodes.len() > MAX_POINTWISE_BF16_EXPRESSION_NODES {
            return false;
        }
        let mut ordinals = Vec::new();
        for (index, node) in self.nodes.iter().enumerate() {
            match node {
                PointwiseBf16Node::Input { access } => ordinals.push(access.get()),
                PointwiseBf16Node::Constant { .. } => {}
                PointwiseBf16Node::Add { lhs, rhs } | PointwiseBf16Node::Multiply { lhs, rhs } => {
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
        !ordinals.is_empty()
            && access_ordinals_are_dense(&mut ordinals)
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
/// expression never reads and a repeat would leave one access unaddressed.
fn access_ordinals_are_dense(ordinals: &mut [u32]) -> bool {
    ordinals.sort_unstable();
    ordinals
        .iter()
        .enumerate()
        .all(|(position, access)| u32::try_from(position) == Ok(*access))
}

#[derive(Clone, Debug)]
enum DraftNode {
    Input {
        access: AccessOrdinal,
    },
    Constant {
        bits: u16,
    },
    Add {
        lhs: PointwiseBf16Value,
        rhs: PointwiseBf16Value,
    },
    Multiply {
        lhs: PointwiseBf16Value,
        rhs: PointwiseBf16Value,
    },
}

/// A checked builder for one bounded physical `bf16` pointwise expression.
#[derive(Debug, Default)]
pub struct PointwiseBf16ExpressionBuilder {
    owner: Arc<()>,
    nodes: Vec<DraftNode>,
    /// Draft ordinals of the input leaves already minted, keyed by input tensor.
    ///
    /// One leaf per tensor is an invariant of the vocabulary, not an
    /// optimization: two `Input` nodes naming one ordinal would be two distinct
    /// canonical nodes for one read, so an expression reading the same tensor
    /// twice would encode differently depending on how the author spelled it.
    inputs: Vec<(AccessOrdinal, u32)>,
}

impl PointwiseBf16ExpressionBuilder {
    /// Opens an empty physical `bf16` expression builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the leaf reading one named boundary input tensor.
    ///
    /// Asking twice for the same ordinal returns the leaf already minted rather
    /// than a second one, so an expression that reads one tensor at several
    /// places shares that read.
    ///
    /// Ordinals may be requested in any order; whether the set they form is the
    /// dense `0..n` a region can bind is a whole-expression property
    /// [`Self::build`] proves.
    ///
    /// # Errors
    ///
    /// Returns [`PointwiseBf16ExpressionAdmissionError::StructuralLimit`] when a
    /// new leaf would exceed the governed node limit.
    pub fn input(
        &mut self,
        access: AccessOrdinal,
    ) -> Result<PointwiseBf16Value, PointwiseBf16ExpressionAdmissionError> {
        if let Some((_, index)) = self.inputs.iter().find(|(minted, _)| *minted == access) {
            return Ok(PointwiseBf16Value {
                owner: Arc::clone(&self.owner),
                index: *index,
            });
        }
        let value = self.push(DraftNode::Input { access })?;
        self.inputs.push((access, value.index));
        Ok(value)
    }

    /// Adds an exact `bf16` constant given by its sixteen-bit pattern.
    ///
    /// # Errors
    ///
    /// Rejects a node beyond the governed expression limit.
    pub fn constant(
        &mut self,
        bits: u16,
    ) -> Result<PointwiseBf16Value, PointwiseBf16ExpressionAdmissionError> {
        self.push(DraftNode::Constant { bits })
    }

    /// Adds an ordered `bf16` `lhs + rhs`.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or structural error when either operand was not
    /// already defined by this builder or the node limit is exhausted.
    pub fn add(
        &mut self,
        lhs: PointwiseBf16Value,
        rhs: PointwiseBf16Value,
    ) -> Result<PointwiseBf16Value, PointwiseBf16ExpressionAdmissionError> {
        self.validate_operand(&lhs)?;
        self.validate_operand(&rhs)?;
        self.push(DraftNode::Add { lhs, rhs })
    }

    /// Adds an ordered `bf16` `lhs * rhs`.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or structural error when either operand was not
    /// already defined by this builder or the node limit is exhausted.
    pub fn multiply(
        &mut self,
        lhs: PointwiseBf16Value,
        rhs: PointwiseBf16Value,
    ) -> Result<PointwiseBf16Value, PointwiseBf16ExpressionAdmissionError> {
        self.validate_operand(&lhs)?;
        self.validate_operand(&rhs)?;
        self.push(DraftNode::Multiply { lhs, rhs })
    }

    /// Verifies, canonically orders, and freezes the expression under `root`.
    ///
    /// # Errors
    ///
    /// Rejects an empty expression, a missing input, a root not minted by this
    /// builder, any draft node not reachable from the explicit root, or a
    /// reachable access ordinal set that is not the dense `0..n`.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "consuming the builder also consumes its builder-owned root value"
    )]
    pub fn build(
        self,
        root: PointwiseBf16Value,
    ) -> Result<PointwiseBf16Expression, PointwiseBf16ExpressionBuildError> {
        if self.nodes.is_empty() {
            return Err(PointwiseBf16ExpressionBuildError::new(
                self,
                PointwiseBf16ExpressionDiagnostic::EmptyExpression,
            ));
        }
        if !Arc::ptr_eq(&root.owner, &self.owner)
            || usize::try_from(root.index)
                .ok()
                .is_none_or(|index| index >= self.nodes.len())
        {
            return Err(PointwiseBf16ExpressionBuildError::new(
                self,
                PointwiseBf16ExpressionDiagnostic::InvalidRoot,
            ));
        }
        if self.inputs.is_empty() {
            return Err(PointwiseBf16ExpressionBuildError::new(
                self,
                PointwiseBf16ExpressionDiagnostic::MissingInput,
            ));
        }

        let mut canonical_ids = vec![None; self.nodes.len()];
        let mut nodes = Vec::with_capacity(self.nodes.len());
        canonicalize_nodes(root.index, &self.nodes, &mut canonical_ids, &mut nodes);
        if let Some(index) = canonical_ids.iter().position(Option::is_none) {
            return Err(PointwiseBf16ExpressionBuildError::new(
                self,
                PointwiseBf16ExpressionDiagnostic::UnreachableNode { index },
            ));
        }
        // Every retained node is reachable by the check above, so the ordinals
        // gathered here are exactly the ones a region would have to bind.
        let mut ordinals: Vec<u32> = nodes
            .iter()
            .filter_map(|node| match node {
                PointwiseBf16Node::Input { access } => Some(access.get()),
                PointwiseBf16Node::Constant { .. }
                | PointwiseBf16Node::Add { .. }
                | PointwiseBf16Node::Multiply { .. } => None,
            })
            .collect();
        if !access_ordinals_are_dense(&mut ordinals) {
            let missing = ordinals
                .iter()
                .enumerate()
                .find(|(position, access)| u32::try_from(*position) != Ok(**access))
                .map_or(0, |(position, _)| {
                    u32::try_from(position).unwrap_or(u32::MAX)
                });
            return Err(PointwiseBf16ExpressionBuildError::new(
                self,
                PointwiseBf16ExpressionDiagnostic::SparseAccessOrdinals {
                    missing: AccessOrdinal::new(missing),
                },
            ));
        }
        let root = usize::try_from(root.index)
            .ok()
            .and_then(|index| canonical_ids.get(index))
            .copied()
            .flatten()
            .ok_or_else(|| {
                PointwiseBf16ExpressionBuildError::new(
                    self,
                    PointwiseBf16ExpressionDiagnostic::InvalidRoot,
                )
            })?;
        Ok(PointwiseBf16Expression {
            nodes: nodes.into_boxed_slice(),
            root,
        })
    }

    fn push(
        &mut self,
        node: DraftNode,
    ) -> Result<PointwiseBf16Value, PointwiseBf16ExpressionAdmissionError> {
        if self.nodes.len() >= MAX_POINTWISE_BF16_EXPRESSION_NODES {
            return Err(PointwiseBf16ExpressionAdmissionError::StructuralLimit {
                actual: self.nodes.len() + 1,
                limit: MAX_POINTWISE_BF16_EXPRESSION_NODES,
            });
        }
        let index = u32::try_from(self.nodes.len()).map_err(|_| {
            PointwiseBf16ExpressionAdmissionError::StructuralLimit {
                actual: self.nodes.len() + 1,
                limit: MAX_POINTWISE_BF16_EXPRESSION_NODES,
            }
        })?;
        self.nodes.push(node);
        Ok(PointwiseBf16Value {
            owner: Arc::clone(&self.owner),
            index,
        })
    }

    fn validate_operand(
        &self,
        operand: &PointwiseBf16Value,
    ) -> Result<(), PointwiseBf16ExpressionAdmissionError> {
        if !Arc::ptr_eq(&operand.owner, &self.owner) {
            return Err(PointwiseBf16ExpressionAdmissionError::ForeignValue);
        }
        if usize::try_from(operand.index)
            .ok()
            .is_none_or(|index| index >= self.nodes.len())
        {
            return Err(PointwiseBf16ExpressionAdmissionError::ForwardValue);
        }
        Ok(())
    }
}

fn canonicalize_nodes(
    root: u32,
    draft: &[DraftNode],
    canonical_ids: &mut [Option<PointwiseBf16NodeId>],
    nodes: &mut Vec<PointwiseBf16Node>,
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
                DraftNode::Input { .. } | DraftNode::Constant { .. } => {}
            }
            continue;
        }
        let resolve = |value: &PointwiseBf16Value| {
            canonical_ids[usize::try_from(value.index).expect("draft node ordinal is bounded")]
                .expect("operands are canonicalized before their user")
        };
        let node = match &draft[draft_index] {
            DraftNode::Input { access } => PointwiseBf16Node::Input { access: *access },
            DraftNode::Constant { bits } => PointwiseBf16Node::Constant { bits: *bits },
            DraftNode::Add { lhs, rhs } => PointwiseBf16Node::Add {
                lhs: resolve(lhs),
                rhs: resolve(rhs),
            },
            DraftNode::Multiply { lhs, rhs } => PointwiseBf16Node::Multiply {
                lhs: resolve(lhs),
                rhs: resolve(rhs),
            },
        };
        let id = PointwiseBf16NodeId(u32::try_from(nodes.len()).expect("node count is bounded"));
        nodes.push(node);
        canonical_ids[draft_index] = Some(id);
    }
}

fn reachable_nodes(nodes: &[PointwiseBf16Node], root: PointwiseBf16NodeId) -> Vec<bool> {
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
            PointwiseBf16Node::Add { lhs, rhs } | PointwiseBf16Node::Multiply { lhs, rhs } => {
                pending.push(*lhs);
                pending.push(*rhs);
            }
            PointwiseBf16Node::Input { .. } | PointwiseBf16Node::Constant { .. } => {}
        }
    }
    reachable
}

/// Local failure while authoring one physical `bf16` pointwise expression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PointwiseBf16ExpressionAdmissionError {
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

impl PointwiseBf16ExpressionAdmissionError {
    /// Returns the stable construction-rule identifier.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::ForeignValue => "pointwise-bf16-foreign-value",
            Self::ForwardValue => "pointwise-bf16-forward-value",
            Self::StructuralLimit { .. } => "pointwise-bf16-structural-limit",
        }
    }
}

impl fmt::Display for PointwiseBf16ExpressionAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}

impl Error for PointwiseBf16ExpressionAdmissionError {}

/// Whole-expression verification failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PointwiseBf16ExpressionDiagnostic {
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
    /// The reachable access ordinals are not the dense set `0..n`.
    SparseAccessOrdinals {
        /// Smallest access below the largest one the expression never reads.
        missing: AccessOrdinal,
    },
}

impl PointwiseBf16ExpressionDiagnostic {
    /// Returns the stable whole-expression rule identifier.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::EmptyExpression => "pointwise-bf16-empty-expression",
            Self::MissingInput => "pointwise-bf16-missing-input",
            Self::InvalidRoot => "pointwise-bf16-invalid-root",
            Self::UnreachableNode { .. } => "pointwise-bf16-unreachable-node",
            Self::SparseAccessOrdinals { .. } => "pointwise-bf16-sparse-access-ordinals",
        }
    }
}

impl fmt::Display for PointwiseBf16ExpressionDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}

impl Error for PointwiseBf16ExpressionDiagnostic {}

/// Recoverable failure from consuming whole-expression verification.
///
/// Carries the deterministic diagnostic and returns the intact builder through
/// [`PointwiseBf16ExpressionBuildError::into_parts`] so a caller can amend and
/// retry.
#[derive(Debug)]
pub struct PointwiseBf16ExpressionBuildError {
    builder: Box<PointwiseBf16ExpressionBuilder>,
    diagnostic: PointwiseBf16ExpressionDiagnostic,
}

impl PointwiseBf16ExpressionBuildError {
    fn new(
        builder: PointwiseBf16ExpressionBuilder,
        diagnostic: PointwiseBf16ExpressionDiagnostic,
    ) -> Self {
        Self {
            builder: Box::new(builder),
            diagnostic,
        }
    }

    /// Returns the deterministic whole-expression diagnostic.
    #[must_use]
    pub const fn diagnostic(&self) -> PointwiseBf16ExpressionDiagnostic {
        self.diagnostic
    }

    /// Recovers the intact builder and its diagnostic.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        PointwiseBf16ExpressionBuilder,
        PointwiseBf16ExpressionDiagnostic,
    ) {
        (*self.builder, self.diagnostic)
    }
}

impl fmt::Display for PointwiseBf16ExpressionBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pointwise-bf16 expression verification failed: {}",
            self.diagnostic
        )
    }
}

impl Error for PointwiseBf16ExpressionBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.diagnostic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1.0, 2.0, and 3.0 in the `bf16` encoding.
    const ONE: u16 = 0x3f80;
    const TWO: u16 = 0x4000;
    const THREE: u16 = 0x4040;

    #[test]
    fn checked_builder_retains_exact_topological_tree() {
        let mut builder = PointwiseBf16ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let two = builder.constant(TWO).unwrap();
        let product = builder.multiply(input, two).unwrap();
        let one = builder.constant(ONE).unwrap();
        let root = builder.add(product, one).unwrap();
        let expression = builder.build(root).unwrap();
        assert!(expression.is_valid());
        assert_eq!(expression.root().index(), 4);
        assert_eq!(
            expression.nodes(),
            [
                PointwiseBf16Node::Input {
                    access: AccessOrdinal::FIRST
                },
                PointwiseBf16Node::Constant { bits: TWO },
                PointwiseBf16Node::Multiply {
                    lhs: PointwiseBf16NodeId(0),
                    rhs: PointwiseBf16NodeId(1),
                },
                PointwiseBf16Node::Constant { bits: ONE },
                PointwiseBf16Node::Add {
                    lhs: PointwiseBf16NodeId(2),
                    rhs: PointwiseBf16NodeId(3),
                },
            ]
        );
    }

    #[test]
    fn independent_ready_node_insertion_order_canonicalizes_identically() {
        fn first() -> PointwiseBf16Expression {
            let mut builder = PointwiseBf16ExpressionBuilder::new();
            let input = builder.input(AccessOrdinal::FIRST).unwrap();
            let two = builder.constant(TWO).unwrap();
            let three = builder.constant(THREE).unwrap();
            let add = builder.add(input.clone(), two).unwrap();
            let multiply = builder.multiply(input, three).unwrap();
            let root = builder.add(add, multiply).unwrap();
            builder.build(root).unwrap()
        }
        fn second() -> PointwiseBf16Expression {
            let mut builder = PointwiseBf16ExpressionBuilder::new();
            let input = builder.input(AccessOrdinal::FIRST).unwrap();
            let three = builder.constant(THREE).unwrap();
            let two = builder.constant(TWO).unwrap();
            let multiply = builder.multiply(input.clone(), three).unwrap();
            let add = builder.add(input, two).unwrap();
            let root = builder.add(add, multiply).unwrap();
            builder.build(root).unwrap()
        }
        assert_eq!(first(), second());
    }

    #[test]
    fn canonicalization_preserves_dag_sharing() {
        let mut builder = PointwiseBf16ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let two = builder.constant(TWO).unwrap();
        let shared = builder.multiply(input, two).unwrap();
        let root = builder.add(shared.clone(), shared).unwrap();
        let expression = builder.build(root).unwrap();
        assert_eq!(expression.nodes().len(), 4);
        assert!(matches!(
            expression.nodes()[3],
            PointwiseBf16Node::Add {
                lhs: PointwiseBf16NodeId(2),
                rhs: PointwiseBf16NodeId(2),
            }
        ));
    }

    /// Asking twice for one ordinal shares a leaf; two ordinals are two leaves.
    #[test]
    fn repeated_ordinals_share_a_leaf_and_distinct_ordinals_do_not() {
        let mut shared = PointwiseBf16ExpressionBuilder::new();
        let first = shared.input(AccessOrdinal::FIRST).unwrap();
        let again = shared.input(AccessOrdinal::FIRST).unwrap();
        let root = shared.multiply(first, again).unwrap();
        let expression = shared.build(root).unwrap();
        assert_eq!(expression.input_count(), 1);

        let mut distinct = PointwiseBf16ExpressionBuilder::new();
        let a = distinct.input(AccessOrdinal::new(0)).unwrap();
        let b = distinct.input(AccessOrdinal::new(1)).unwrap();
        let root = distinct.add(a, b).unwrap();
        let expression = distinct.build(root).unwrap();
        assert!(expression.is_valid());
        assert_eq!(expression.input_count(), 2);
    }

    /// A gap in the reachable ordinals names a binding nothing reads.
    #[test]
    fn a_sparse_input_ordinal_set_is_refused_with_the_missing_ordinal() {
        let mut sparse = PointwiseBf16ExpressionBuilder::new();
        let first = sparse.input(AccessOrdinal::new(0)).unwrap();
        let third = sparse.input(AccessOrdinal::new(2)).unwrap();
        let root = sparse.add(first, third).unwrap();
        let retained_root = root.clone();
        let error = sparse.build(root).unwrap_err();
        assert_eq!(
            error.diagnostic(),
            PointwiseBf16ExpressionDiagnostic::SparseAccessOrdinals {
                missing: AccessOrdinal::new(1)
            }
        );
        // Amending the recovered builder to read the skipped ordinal is
        // admitted, so the refusal is of the gap and not of the second input.
        let (mut recovered, _) = error.into_parts();
        let second = recovered.input(AccessOrdinal::new(1)).unwrap();
        let repaired_root = recovered.add(retained_root, second).unwrap();
        let expression = recovered.build(repaired_root).unwrap();
        assert_eq!(expression.input_count(), 3);
    }

    #[test]
    fn missing_inputs_are_typed_errors() {
        let mut missing = PointwiseBf16ExpressionBuilder::new();
        let root = missing.constant(0).unwrap();
        let retained_root = root.clone();
        let error = missing.build(root).unwrap_err();
        assert_eq!(
            error.diagnostic(),
            PointwiseBf16ExpressionDiagnostic::MissingInput
        );
        let (mut recovered, diagnostic) = error.into_parts();
        assert_eq!(diagnostic, PointwiseBf16ExpressionDiagnostic::MissingInput);
        let input = recovered.input(AccessOrdinal::FIRST).unwrap();
        let repaired_root = recovered.add(input, retained_root).unwrap();
        assert!(recovered.build(repaired_root).is_ok());
    }

    #[test]
    fn foreign_forward_and_invalid_root_values_are_typed_errors() {
        let mut first = PointwiseBf16ExpressionBuilder::new();
        let input = first.input(AccessOrdinal::FIRST).unwrap();
        let mut second = PointwiseBf16ExpressionBuilder::new();
        let foreign = second.input(AccessOrdinal::FIRST).unwrap();
        assert!(matches!(
            first.add(input.clone(), foreign),
            Err(PointwiseBf16ExpressionAdmissionError::ForeignValue)
        ));

        let forward = PointwiseBf16Value {
            owner: Arc::clone(&first.owner),
            index: 99,
        };
        assert!(matches!(
            first.multiply(input.clone(), forward.clone()),
            Err(PointwiseBf16ExpressionAdmissionError::ForwardValue)
        ));
        assert_eq!(
            first.build(forward).unwrap_err().diagnostic(),
            PointwiseBf16ExpressionDiagnostic::InvalidRoot
        );
    }

    #[test]
    fn empty_unreachable_and_over_bound_expressions_are_typed_errors() {
        let empty = PointwiseBf16ExpressionBuilder::new();
        let forged = PointwiseBf16Value {
            owner: Arc::clone(&empty.owner),
            index: 0,
        };
        assert_eq!(
            empty.build(forged).unwrap_err().diagnostic(),
            PointwiseBf16ExpressionDiagnostic::EmptyExpression
        );

        let mut unreachable = PointwiseBf16ExpressionBuilder::new();
        let input = unreachable.input(AccessOrdinal::FIRST).unwrap();
        let root = unreachable.constant(0).unwrap();
        let retained_root = root.clone();
        let error = unreachable.build(root).unwrap_err();
        assert_eq!(
            error.diagnostic(),
            PointwiseBf16ExpressionDiagnostic::UnreachableNode { index: 0 }
        );
        let (mut recovered, _) = error.into_parts();
        let repaired_root = recovered.add(input, retained_root).unwrap();
        assert!(recovered.build(repaired_root).is_ok());

        let mut bounded = PointwiseBf16ExpressionBuilder::new();
        for _ in 0..MAX_POINTWISE_BF16_EXPRESSION_NODES {
            bounded.constant(0).unwrap();
        }
        assert!(matches!(
            bounded.constant(0),
            Err(PointwiseBf16ExpressionAdmissionError::StructuralLimit {
                actual,
                limit: MAX_POINTWISE_BF16_EXPRESSION_NODES,
            }) if actual == MAX_POINTWISE_BF16_EXPRESSION_NODES + 1
        ));
    }
}
