//! The bounded typed ABI expression domain, its identity, and its evaluator.
//!
//! # Ownership, and a live contract divergence
//!
//! **Fact.** ADR 0068 places the public `AbiExpr` domain type, its admitted
//! roots, validation, canonical identity, and authoritative pure checked
//! evaluation semantics in `tiler_ir::program`, and assigns only the versioned
//! wire encoding, runtime fact binding, phase enforcement, failure
//! classification, and backend-payload mappings to `tiler-artifact`. ADR 0070
//! lists `AbiExpr` under `program` for the same reason.
//!
//! **Fact.** `tiler_ir::program` as merged carries no ABI, guard, or routing
//! representation. `prototype-kernel-program-ir` scoped those to this
//! artifact-facing projection, which is why this module exists here.
//!
//! **Status.** That divergence is real, is recorded in the ticket
//! `complete-program-identity-with-abi-guards-and-routing`, and is not resolved
//! by this module. This file is therefore written to *move*: it depends on
//! nothing in this crate except its own error type and the governed
//! [`TargetPropertyKey`](super::TargetPropertyKey) newtype, and its evaluator
//! consumes an already-resolved [`AbiFacts`] rather than reaching for a live
//! device. Binding live facts and enforcing that a fact could legally be
//! queried at the phase it claims stays in [`super::facts`], which is the half
//! ADR 0068 assigns to this crate either way.
//!
//! # The bounded profile
//!
//! `docs/artifact-abi.md` describes a larger language than this. Implemented
//! here: literals, input extents, governed target properties, checked add,
//! subtract, multiply, minimum, maximum, floor/ceiling/exact division,
//! divisibility, equality, ordering, boolean composition, conditional select,
//! and explicit checked narrowing. Not implemented and explicitly rejected by
//! absence rather than approximated: element strides, view start elements,
//! remainder, and narrowing widths other than 16 and 32 bits.

use std::error::Error;
use std::fmt;

use tiler_ir::semantic::InputKey;
use tiler_ir::shape::Axis;

use super::keys::TargetPropertyKey;

const EXPR_DOMAIN: &[u8] = b"tiler.artifact-program.abi-expr.v1\0";

/// The value type of one ABI expression.
///
/// The language is deliberately two-typed. A predicate cannot be silently used
/// as a size, and a size cannot be silently used as a guard.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AbiType {
    /// A checked 64-bit unsigned quantity.
    Unsigned,
    /// A predicate.
    Boolean,
}

impl fmt::Display for AbiType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsigned => formatter.write_str("unsigned"),
            Self::Boolean => formatter.write_str("boolean"),
        }
    }
}

/// One evaluated ABI expression value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AbiValue {
    /// A checked 64-bit unsigned quantity.
    Unsigned(u64),
    /// A predicate outcome.
    Boolean(bool),
}

impl AbiValue {
    /// Returns the value type of this value.
    #[must_use]
    pub const fn value_type(self) -> AbiType {
        match self {
            Self::Unsigned(_) => AbiType::Unsigned,
            Self::Boolean(_) => AbiType::Boolean,
        }
    }

    /// Returns the unsigned quantity, or `None` for a predicate outcome.
    #[must_use]
    pub const fn unsigned(self) -> Option<u64> {
        match self {
            Self::Unsigned(value) => Some(value),
            Self::Boolean(_) => None,
        }
    }

    /// Returns the predicate outcome, or `None` for an unsigned quantity.
    #[must_use]
    pub const fn boolean(self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(value),
            Self::Unsigned(_) => None,
        }
    }
}

/// The ordered availability phase at which a fact first becomes readable.
///
/// These are ADR 0043's phases. The order is total and is load-bearing: a use
/// site evaluated at one phase may only name roots available no later than it,
/// and `RoutingCommit` happens after every phase in this list.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AvailabilityPhase {
    /// Known from the governed compile-time target profile alone.
    CompileProfile,
    /// Known from evidence recorded in the artifact.
    ArtifactEvidence,
    /// Known once a live device and context are bound.
    LiveDevicePreflight,
    /// Known once the entry's pipeline or kernel is prepared.
    PreparedKernelPreflight,
    /// Known once one concrete launch instance is being validated.
    LaunchPreflight,
}

impl AvailabilityPhase {
    const fn tag(self) -> u8 {
        match self {
            Self::CompileProfile => 0x01,
            Self::ArtifactEvidence => 0x02,
            Self::LiveDevicePreflight => 0x03,
            Self::PreparedKernelPreflight => 0x04,
            Self::LaunchPreflight => 0x05,
        }
    }
}

impl fmt::Display for AvailabilityPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// One typed root fact an ABI expression may name.
///
/// Roots are not generic variables: each names a typed program-interface or
/// target fact, and each states the phase at which it becomes readable.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AbiRoot {
    /// A constant unsigned quantity.
    UnsignedLiteral(u64),
    /// A constant predicate.
    BooleanLiteral(bool),
    /// The extent of one axis of a named program input tensor.
    InputExtent {
        /// Stable interface key of the bound input.
        key: InputKey,
        /// Axis of that input whose extent is read.
        axis: Axis,
    },
    /// A governed target or device property.
    TargetProperty {
        /// Governed property key.
        key: TargetPropertyKey,
        /// Earliest phase at which the property can be queried.
        phase: AvailabilityPhase,
    },
}

impl AbiRoot {
    /// Returns the value type this root produces.
    #[must_use]
    pub const fn value_type(&self) -> AbiType {
        match self {
            Self::UnsignedLiteral(_) | Self::InputExtent { .. } | Self::TargetProperty { .. } => {
                AbiType::Unsigned
            }
            Self::BooleanLiteral(_) => AbiType::Boolean,
        }
    }

    /// Returns the earliest phase at which this root can be read.
    #[must_use]
    pub const fn available_at(&self) -> AvailabilityPhase {
        match self {
            Self::UnsignedLiteral(_) | Self::BooleanLiteral(_) => AvailabilityPhase::CompileProfile,
            Self::InputExtent { .. } => AvailabilityPhase::LiveDevicePreflight,
            Self::TargetProperty { phase, .. } => *phase,
        }
    }

    /// Returns whether this root is readable from the bound semantic interface.
    ///
    /// Accessible ranges and launch geometry are restricted to interface roots
    /// so they can be computed before any device-dependent query.
    #[must_use]
    pub const fn is_interface_fact(&self) -> bool {
        match self {
            Self::UnsignedLiteral(_) | Self::BooleanLiteral(_) | Self::InputExtent { .. } => true,
            Self::TargetProperty { .. } => false,
        }
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::UnsignedLiteral(value) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            Self::BooleanLiteral(value) => {
                bytes.push(0x02);
                bytes.push(u8::from(*value));
            }
            Self::InputExtent { key, axis } => {
                bytes.push(0x03);
                push_slice(bytes, key.as_str().as_bytes());
                bytes.extend_from_slice(&axis.get().to_be_bytes());
            }
            Self::TargetProperty { key, phase } => {
                bytes.push(0x04);
                push_slice(bytes, key.as_str().as_bytes());
                bytes.push(phase.tag());
            }
        }
    }
}

/// One admitted unary ABI operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AbiUnaryOp {
    /// Predicate negation.
    Not,
    /// Checked narrowing to a 16-bit target field.
    NarrowU16,
    /// Checked narrowing to a 32-bit target field.
    NarrowU32,
}

impl AbiUnaryOp {
    const fn tag(self) -> u8 {
        match self {
            Self::Not => 0x01,
            Self::NarrowU16 => 0x02,
            Self::NarrowU32 => 0x03,
        }
    }

    const fn operand_type(self) -> AbiType {
        match self {
            Self::Not => AbiType::Boolean,
            Self::NarrowU16 | Self::NarrowU32 => AbiType::Unsigned,
        }
    }

    const fn result_type(self) -> AbiType {
        match self {
            Self::Not => AbiType::Boolean,
            Self::NarrowU16 | Self::NarrowU32 => AbiType::Unsigned,
        }
    }
}

/// One admitted binary ABI operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AbiBinaryOp {
    /// Checked unsigned addition.
    CheckedAdd,
    /// Checked unsigned subtraction; underflow is an evaluation failure.
    CheckedSubtract,
    /// Checked unsigned multiplication.
    CheckedMultiply,
    /// Unsigned minimum.
    Minimum,
    /// Unsigned maximum.
    Maximum,
    /// Floor division; a zero divisor is an evaluation failure.
    FloorDivide,
    /// Ceiling division; a zero divisor is an evaluation failure.
    CeilingDivide,
    /// Exact division; a zero divisor or a remainder is an evaluation failure.
    ExactDivide,
    /// Whether the left operand is an exact multiple of the right.
    IsMultipleOf,
    /// Unsigned equality.
    Equal,
    /// Unsigned non-strict ordering.
    LessOrEqual,
    /// Predicate conjunction.
    And,
    /// Predicate disjunction.
    Or,
}

impl AbiBinaryOp {
    const fn tag(self) -> u8 {
        match self {
            Self::CheckedAdd => 0x01,
            Self::CheckedSubtract => 0x02,
            Self::CheckedMultiply => 0x03,
            Self::Minimum => 0x04,
            Self::Maximum => 0x05,
            Self::FloorDivide => 0x06,
            Self::CeilingDivide => 0x07,
            Self::ExactDivide => 0x08,
            Self::IsMultipleOf => 0x09,
            Self::Equal => 0x0a,
            Self::LessOrEqual => 0x0b,
            Self::And => 0x0c,
            Self::Or => 0x0d,
        }
    }

    const fn operand_type(self) -> AbiType {
        match self {
            Self::CheckedAdd
            | Self::CheckedSubtract
            | Self::CheckedMultiply
            | Self::Minimum
            | Self::Maximum
            | Self::FloorDivide
            | Self::CeilingDivide
            | Self::ExactDivide
            | Self::IsMultipleOf
            | Self::Equal
            | Self::LessOrEqual => AbiType::Unsigned,
            Self::And | Self::Or => AbiType::Boolean,
        }
    }

    const fn result_type(self) -> AbiType {
        match self {
            Self::CheckedAdd
            | Self::CheckedSubtract
            | Self::CheckedMultiply
            | Self::Minimum
            | Self::Maximum
            | Self::FloorDivide
            | Self::CeilingDivide
            | Self::ExactDivide => AbiType::Unsigned,
            Self::IsMultipleOf | Self::Equal | Self::LessOrEqual | Self::And | Self::Or => {
                AbiType::Boolean
            }
        }
    }
}

/// One node of the shared expression arena.
///
/// Operands are stored as arena positions and are always strictly smaller than
/// the node's own position, so the arena is acyclic by construction and needs
/// no cycle check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ExprNode {
    Root(AbiRoot),
    Unary {
        op: AbiUnaryOp,
        operand: u32,
    },
    Binary {
        op: AbiBinaryOp,
        left: u32,
        right: u32,
    },
    Select {
        condition: u32,
        if_true: u32,
        if_false: u32,
    },
}

/// A typed failure of pure checked ABI expression evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AbiEvaluationError {
    /// The bound environment supplies no extent for a named input axis.
    UnboundInputExtent {
        /// Interface key whose extent was requested.
        key: InputKey,
        /// Axis whose extent was requested.
        axis: Axis,
    },
    /// The bound environment supplies no value for a named target property.
    UnboundTargetProperty {
        /// Governed property key that was requested.
        key: TargetPropertyKey,
    },
    /// A checked unsigned operation left the 64-bit domain.
    Overflow {
        /// Operation that failed.
        op: AbiBinaryOp,
    },
    /// A division named a zero divisor.
    DivisionByZero {
        /// Operation that failed.
        op: AbiBinaryOp,
    },
    /// An exact division left a nonzero remainder.
    InexactDivision {
        /// Dividend that was not an exact multiple.
        dividend: u64,
        /// Divisor that did not divide it.
        divisor: u64,
    },
    /// A checked narrowing did not fit its target field.
    NarrowingOverflow {
        /// Narrowing that failed.
        op: AbiUnaryOp,
        /// Value that did not fit.
        value: u64,
    },
}

impl fmt::Display for AbiEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for AbiEvaluationError {}

/// A resolved environment of already-bound ABI facts.
///
/// The evaluator is pure over this value. Acquiring live facts and proving that
/// each could legally be queried at the phase it claims is
/// [`super::AbiFactBinder`]'s job, and only a binder produces one of these.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbiFacts {
    pub(super) reached: AvailabilityPhase,
    pub(super) input_extents: Vec<(InputKey, Axis, u64)>,
    pub(super) target_properties: Vec<(TargetPropertyKey, u64)>,
}

impl AbiFacts {
    /// Returns the latest availability phase these facts were bound at.
    #[must_use]
    pub const fn reached_phase(&self) -> AvailabilityPhase {
        self.reached
    }

    fn input_extent(&self, key: &InputKey, axis: Axis) -> Option<u64> {
        self.input_extents
            .iter()
            .find(|(bound, bound_axis, _)| bound == key && *bound_axis == axis)
            .map(|(_, _, extent)| *extent)
    }

    fn target_property(&self, key: &TargetPropertyKey) -> Option<u64> {
        self.target_properties
            .iter()
            .find(|(bound, _)| bound == key)
            .map(|(_, value)| *value)
    }
}

pub(super) fn push_slice(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(
        &u64::try_from(value.len())
            .expect("supported usize fits u64")
            .to_be_bytes(),
    );
    bytes.extend_from_slice(value);
}

/// Derives the canonical content key of one expression node.
///
/// The key names the node's whole subtree by content, never its arena position,
/// so two structurally equal expressions assembled in different orders produce
/// the same bytes and cross-references by key stay injective.
pub(super) fn expr_key(node: &ExprNode, keys: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(EXPR_DOMAIN);
    match node {
        ExprNode::Root(root) => {
            bytes.push(0x01);
            root.encode(&mut bytes);
        }
        ExprNode::Unary { op, operand } => {
            bytes.push(0x02);
            bytes.push(op.tag());
            push_slice(&mut bytes, &keys[position(*operand)]);
        }
        ExprNode::Binary { op, left, right } => {
            bytes.push(0x03);
            bytes.push(op.tag());
            push_slice(&mut bytes, &keys[position(*left)]);
            push_slice(&mut bytes, &keys[position(*right)]);
        }
        ExprNode::Select {
            condition,
            if_true,
            if_false,
        } => {
            bytes.push(0x04);
            push_slice(&mut bytes, &keys[position(*condition)]);
            push_slice(&mut bytes, &keys[position(*if_true)]);
            push_slice(&mut bytes, &keys[position(*if_false)]);
        }
    }
    bytes
}

/// Returns the value type one node produces, given its operands' types.
pub(super) fn node_type(node: &ExprNode, types: &[AbiType]) -> AbiType {
    match node {
        ExprNode::Root(root) => root.value_type(),
        ExprNode::Unary { op, .. } => op.result_type(),
        ExprNode::Binary { op, .. } => op.result_type(),
        ExprNode::Select { if_true, .. } => types[position(*if_true)],
    }
}

/// Returns the operand type one unary operation requires.
pub(super) const fn unary_operand_type(op: AbiUnaryOp) -> AbiType {
    op.operand_type()
}

/// Returns the operand type one binary operation requires.
pub(super) const fn binary_operand_type(op: AbiBinaryOp) -> AbiType {
    op.operand_type()
}

fn position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

/// Evaluates one arena node against already-bound facts.
///
/// Evaluation is lazy through [`ExprNode::Select`]: only the selected branch is
/// evaluated, so a zero-sized bound can guard a branch that would otherwise
/// fail.
///
/// # Errors
///
/// Returns [`AbiEvaluationError`] for an unbound root, a checked-arithmetic
/// overflow or underflow, a zero divisor, an inexact exact division, or a
/// narrowing that does not fit.
pub(super) fn evaluate(
    nodes: &[ExprNode],
    root: u32,
    facts: &AbiFacts,
) -> Result<AbiValue, AbiEvaluationError> {
    enum Step {
        Evaluate(u32),
        Branch(u32),
        Combine(u32),
    }

    let mut memo: Vec<Option<AbiValue>> = vec![None; nodes.len()];
    let mut work = vec![Step::Evaluate(root)];
    while let Some(step) = work.pop() {
        match step {
            Step::Evaluate(index) => {
                if memo[position(index)].is_some() {
                    continue;
                }
                match &nodes[position(index)] {
                    ExprNode::Root(fact) => {
                        memo[position(index)] = Some(evaluate_root(fact, facts)?);
                    }
                    ExprNode::Unary { operand, .. } => {
                        work.push(Step::Combine(index));
                        work.push(Step::Evaluate(*operand));
                    }
                    ExprNode::Binary { left, right, .. } => {
                        work.push(Step::Combine(index));
                        work.push(Step::Evaluate(*left));
                        work.push(Step::Evaluate(*right));
                    }
                    ExprNode::Select { condition, .. } => {
                        work.push(Step::Branch(index));
                        work.push(Step::Evaluate(*condition));
                    }
                }
            }
            Step::Branch(index) => {
                let selected = selected_branch(nodes, index, &memo);
                work.push(Step::Combine(index));
                work.push(Step::Evaluate(selected));
            }
            Step::Combine(index) => {
                if memo[position(index)].is_some() {
                    continue;
                }
                memo[position(index)] = Some(combine(nodes, index, &memo)?);
            }
        }
    }
    Ok(memo[position(root)].expect("the worklist evaluates the requested node"))
}

fn selected_branch(nodes: &[ExprNode], index: u32, memo: &[Option<AbiValue>]) -> u32 {
    let ExprNode::Select {
        condition,
        if_true,
        if_false,
    } = &nodes[position(index)]
    else {
        unreachable!("only a select node schedules a branch step")
    };
    let taken = memo[position(*condition)]
        .expect("a select's condition is evaluated before its branch")
        .boolean()
        .expect("a verified select has a boolean condition");
    if taken { *if_true } else { *if_false }
}

fn combine(
    nodes: &[ExprNode],
    index: u32,
    memo: &[Option<AbiValue>],
) -> Result<AbiValue, AbiEvaluationError> {
    let read = |operand: u32| {
        memo[position(operand)].expect("an operand is evaluated before its operation combines")
    };
    match &nodes[position(index)] {
        ExprNode::Root(_) => unreachable!("a root never schedules a combine step"),
        ExprNode::Unary { op, operand } => apply_unary(*op, read(*operand)),
        ExprNode::Binary { op, left, right } => apply_binary(*op, (read(*left), read(*right))),
        ExprNode::Select { .. } => Ok(read(selected_branch(nodes, index, memo))),
    }
}

fn evaluate_root(root: &AbiRoot, facts: &AbiFacts) -> Result<AbiValue, AbiEvaluationError> {
    match root {
        AbiRoot::UnsignedLiteral(value) => Ok(AbiValue::Unsigned(*value)),
        AbiRoot::BooleanLiteral(value) => Ok(AbiValue::Boolean(*value)),
        AbiRoot::InputExtent { key, axis } => facts.input_extent(key, *axis).map_or_else(
            || {
                Err(AbiEvaluationError::UnboundInputExtent {
                    key: key.clone(),
                    axis: *axis,
                })
            },
            |extent| Ok(AbiValue::Unsigned(extent)),
        ),
        AbiRoot::TargetProperty { key, .. } => facts.target_property(key).map_or_else(
            || Err(AbiEvaluationError::UnboundTargetProperty { key: key.clone() }),
            |value| Ok(AbiValue::Unsigned(value)),
        ),
    }
}

fn apply_unary(op: AbiUnaryOp, operand: AbiValue) -> Result<AbiValue, AbiEvaluationError> {
    let narrow = |value: u64, bound: u64| {
        if value > bound {
            Err(AbiEvaluationError::NarrowingOverflow { op, value })
        } else {
            Ok(AbiValue::Unsigned(value))
        }
    };
    match op {
        AbiUnaryOp::Not => Ok(AbiValue::Boolean(!expect_boolean(operand))),
        AbiUnaryOp::NarrowU16 => narrow(expect_unsigned(operand), u64::from(u16::MAX)),
        AbiUnaryOp::NarrowU32 => narrow(expect_unsigned(operand), u64::from(u32::MAX)),
    }
}

/// Applies one binary operation to two already-evaluated operands.
///
/// The dispatch is a single exhaustive match with no wildcard arm, so a widened
/// operation set is a compile error here rather than a value silently coerced
/// through the wrong operand domain.
fn apply_binary(
    op: AbiBinaryOp,
    operands: (AbiValue, AbiValue),
) -> Result<AbiValue, AbiEvaluationError> {
    let (left, right) = match op.operand_type() {
        AbiType::Boolean => (0, 0),
        AbiType::Unsigned => (expect_unsigned(operands.0), expect_unsigned(operands.1)),
    };
    let checked = |value: Option<u64>| value.ok_or(AbiEvaluationError::Overflow { op });
    let divisor = || {
        if right == 0 {
            Err(AbiEvaluationError::DivisionByZero { op })
        } else {
            Ok(right)
        }
    };
    match op {
        AbiBinaryOp::And => Ok(AbiValue::Boolean(
            expect_boolean(operands.0) && expect_boolean(operands.1),
        )),
        AbiBinaryOp::Or => Ok(AbiValue::Boolean(
            expect_boolean(operands.0) || expect_boolean(operands.1),
        )),
        AbiBinaryOp::CheckedAdd => checked(left.checked_add(right)).map(AbiValue::Unsigned),
        AbiBinaryOp::CheckedSubtract => checked(left.checked_sub(right)).map(AbiValue::Unsigned),
        AbiBinaryOp::CheckedMultiply => checked(left.checked_mul(right)).map(AbiValue::Unsigned),
        AbiBinaryOp::Minimum => Ok(AbiValue::Unsigned(left.min(right))),
        AbiBinaryOp::Maximum => Ok(AbiValue::Unsigned(left.max(right))),
        AbiBinaryOp::FloorDivide => Ok(AbiValue::Unsigned(left / divisor()?)),
        AbiBinaryOp::CeilingDivide => {
            let divisor = divisor()?;
            Ok(AbiValue::Unsigned(
                checked(left.checked_add(divisor - 1))? / divisor,
            ))
        }
        AbiBinaryOp::ExactDivide => {
            let divisor = divisor()?;
            if left % divisor == 0 {
                Ok(AbiValue::Unsigned(left / divisor))
            } else {
                Err(AbiEvaluationError::InexactDivision {
                    dividend: left,
                    divisor,
                })
            }
        }
        AbiBinaryOp::IsMultipleOf => Ok(AbiValue::Boolean(left % divisor()? == 0)),
        AbiBinaryOp::Equal => Ok(AbiValue::Boolean(left == right)),
        AbiBinaryOp::LessOrEqual => Ok(AbiValue::Boolean(left <= right)),
    }
}

/// Reads a verified unsigned operand.
///
/// Insertion-time type checking proves every operand's type before a node
/// enters the arena, so a mismatch here is a broken invariant rather than
/// caller input.
fn expect_unsigned(value: AbiValue) -> u64 {
    value
        .unsigned()
        .expect("a verified unsigned operand holds an unsigned value")
}

/// Reads a verified boolean operand. See [`expect_unsigned`].
fn expect_boolean(value: AbiValue) -> bool {
    value
        .boolean()
        .expect("a verified boolean operand holds a predicate outcome")
}
