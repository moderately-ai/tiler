//! The bounded typed ABI expression domain, its identity, and its evaluator.
//!
//! # Ownership
//!
//! ADR 0068 places the public `AbiExpr` domain type, its admitted roots,
//! validation, canonical identity, and authoritative pure checked evaluation
//! semantics here, with the executable-program representations; ADR 0070 lists
//! `AbiExpr` under `program` for the same reason. `tiler-artifact` owns only
//! the versioned wire encoding, runtime fact binding, phase enforcement,
//! failure classification, and backend-payload mappings.
//!
//! This module lived in `tiler-artifact` until `relocate-abi-expressions-into-tiler-ir`
//! moved it. The rationale the ADR gives is that owning `KernelProgram` here
//! while owning its expression type in `tiler-artifact` "creates a dependency
//! cycle or leaves the program dependent on an external side table for
//! verification and identity", and that a `KernelProgram` must remain
//! "self-contained and independently verifiable before artifact construction".
//! It could not be, while its guards, sizes, and launch geometry were
//! inexpressible in any type it could reach.
//!
//! Binding live facts, and enforcing that a fact could legally be queried at
//! the phase it claims, stay in `tiler_artifact::program::facts` — the half
//! ADR 0068 assigns to that crate either way. The evaluator here consumes an
//! already-resolved [`AbiFacts`] and never reaches for a live device.
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

use crate::identity::push_slice;
use crate::semantic::InputKey;
use crate::shape::Axis;

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
    /// Returns the governed wire tag of this variant.
    ///
    /// The tag is written by an exhaustive match rather than read from the
    /// discriminant, so inserting or reordering a variant is a build error
    /// here instead of a silent re-encoding of every subject ever produced
    /// (ADR 0074 convention 3).
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::CompileProfile => 0x01,
            Self::ArtifactEvidence => 0x02,
            Self::LiveDevicePreflight => 0x03,
            Self::PreparedKernelPreflight => 0x04,
            Self::LaunchPreflight => 0x05,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized phase.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::CompileProfile),
            0x02 => Some(Self::ArtifactEvidence),
            0x03 => Some(Self::LiveDevicePreflight),
            0x04 => Some(Self::PreparedKernelPreflight),
            0x05 => Some(Self::LaunchPreflight),
            _ => None,
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

    /// Encodes this root's canonical byte form.
    ///
    /// The same encoding serves the content key and the artifact envelope's
    /// wire form, so a root can never carry one spelling into identity and a
    /// different one onto disk. `tiler_artifact::program::codec` owns the
    /// inverse and is pinned against this function by an exhaustive round-trip
    /// test.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
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
    /// Returns the governed wire tag of this variant.
    ///
    /// The tag is written by an exhaustive match rather than read from the
    /// discriminant, so inserting or reordering a variant is a build error
    /// here instead of a silent re-encoding of every subject ever produced
    /// (ADR 0074 convention 3).
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Not => 0x01,
            Self::NarrowU16 => 0x02,
            Self::NarrowU32 => 0x03,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized operation.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Not),
            0x02 => Some(Self::NarrowU16),
            0x03 => Some(Self::NarrowU32),
            _ => None,
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
    /// Returns the governed wire tag of this variant.
    ///
    /// The tag is written by an exhaustive match rather than read from the
    /// discriminant, so inserting or reordering a variant is a build error
    /// here instead of a silent re-encoding of every subject ever produced
    /// (ADR 0074 convention 3).
    #[must_use]
    pub const fn tag(self) -> u8 {
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

    /// Resolves a governed wire tag, or `None` for an unrecognized operation.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::CheckedAdd),
            0x02 => Some(Self::CheckedSubtract),
            0x03 => Some(Self::CheckedMultiply),
            0x04 => Some(Self::Minimum),
            0x05 => Some(Self::Maximum),
            0x06 => Some(Self::FloorDivide),
            0x07 => Some(Self::CeilingDivide),
            0x08 => Some(Self::ExactDivide),
            0x09 => Some(Self::IsMultipleOf),
            0x0a => Some(Self::Equal),
            0x0b => Some(Self::LessOrEqual),
            0x0c => Some(Self::And),
            0x0d => Some(Self::Or),
            _ => None,
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
pub enum ExprNode {
    /// A typed root fact naming a program, interface, or target value.
    Root(AbiRoot),
    /// One checked unary application over an earlier arena node.
    Unary {
        /// The admitted unary operation.
        op: AbiUnaryOp,
        /// Arena position of the operand, which always precedes this node.
        operand: u32,
    },
    /// One checked binary application over two earlier arena nodes.
    Binary {
        /// The admitted binary operation.
        op: AbiBinaryOp,
        /// Arena position of the left operand.
        left: u32,
        /// Arena position of the right operand.
        right: u32,
    },
    /// A conditional selection whose branches must agree in type.
    Select {
        /// Arena position of the boolean condition.
        condition: u32,
        /// Arena position of the value taken when the condition holds.
        if_true: u32,
        /// Arena position of the value taken otherwise.
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
/// each could legally be queried at the phase it claims is the binder's job,
/// which ADR 0068 assigns to `tiler_artifact::program::facts::AbiFactBinder` —
/// a downstream crate, so this names it in text rather than linking to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbiFacts {
    pub(super) reached: AvailabilityPhase,
    pub(super) input_extents: Vec<(InputKey, Axis, u64)>,
    pub(super) target_properties: Vec<(TargetPropertyKey, u64)>,
}

impl AbiFacts {
    /// Assembles one resolved fact environment for evaluation.
    ///
    /// ADR 0068 splits this deliberately: binding live facts and proving a fact
    /// could legally be queried at the phase it claims belong to
    /// `tiler-artifact`, while evaluating against an already-resolved
    /// environment belongs here. So this constructor takes resolved values and
    /// asserts nothing about where they came from — the binder is the authority
    /// that a value was legitimately readable at `reached`.
    ///
    /// It is deliberately not a validating constructor. A duplicate extent or
    /// property here resolves to whichever entry the binder recorded first, and
    /// preventing duplicates is the binder's obligation, not a second check
    /// with its own drifting definition of what a duplicate is.
    #[must_use]
    pub fn new(
        reached: AvailabilityPhase,
        input_extents: Vec<(InputKey, Axis, u64)>,
        target_properties: Vec<(TargetPropertyKey, u64)>,
    ) -> Self {
        Self {
            reached,
            input_extents,
            target_properties,
        }
    }

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

/// Derives the canonical content key of one expression node.
///
/// The key names the node's whole subtree by content, never its arena position,
/// so two structurally equal expressions assembled in different orders produce
/// the same bytes and cross-references by key stay injective.
#[must_use]
pub fn expr_key(node: &ExprNode, keys: &[Vec<u8>]) -> Vec<u8> {
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
#[must_use]
pub fn node_type(node: &ExprNode, types: &[AbiType]) -> AbiType {
    match node {
        ExprNode::Root(root) => root.value_type(),
        ExprNode::Unary { op, .. } => op.result_type(),
        ExprNode::Binary { op, .. } => op.result_type(),
        ExprNode::Select { if_true, .. } => types[position(*if_true)],
    }
}

/// Returns the earliest phase at which one node's whole subtree is readable.
///
/// The recurrence is shared by the transactional builder and by the envelope
/// decoder: a use site's phase check must mean the same thing whether the
/// expression was just authored or was just read from bytes.
#[must_use]
pub fn node_phase(node: &ExprNode, phases: &[AvailabilityPhase]) -> AvailabilityPhase {
    match node {
        ExprNode::Root(root) => root.available_at(),
        ExprNode::Unary { operand, .. } => phases[position(*operand)],
        ExprNode::Binary { left, right, .. } => {
            phases[position(*left)].max(phases[position(*right)])
        }
        ExprNode::Select {
            condition,
            if_true,
            if_false,
        } => phases[position(*condition)]
            .max(phases[position(*if_true)])
            .max(phases[position(*if_false)]),
    }
}

/// Returns whether one node's whole subtree reads only bound-interface facts.
///
/// See [`node_phase`] for why the recurrence is shared.
#[must_use]
pub fn node_is_interface_only(node: &ExprNode, interface_only: &[bool]) -> bool {
    match node {
        ExprNode::Root(root) => root.is_interface_fact(),
        ExprNode::Unary { operand, .. } => interface_only[position(*operand)],
        ExprNode::Binary { left, right, .. } => {
            interface_only[position(*left)] && interface_only[position(*right)]
        }
        ExprNode::Select {
            condition,
            if_true,
            if_false,
        } => {
            interface_only[position(*condition)]
                && interface_only[position(*if_true)]
                && interface_only[position(*if_false)]
        }
    }
}

/// Returns the operand type one unary operation requires.
#[must_use]
pub const fn unary_operand_type(op: AbiUnaryOp) -> AbiType {
    op.operand_type()
}

/// Returns the operand type one binary operation requires.
#[must_use]
pub const fn binary_operand_type(op: AbiBinaryOp) -> AbiType {
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
/// # Panics
///
/// Panics when `root` is out of range for `nodes`, or when the arena is not
/// closed under reference. Both are structural properties the verifier proves
/// before an arena reaches here, so a panic means an unverified arena was
/// evaluated rather than a bad value — which is why it is not a returned error.
pub fn evaluate(
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

/// Maximum bytes of one governed target-property key.
pub const MAX_TARGET_PROPERTY_KEY_BYTES: usize = 256;

/// A governed target-property key an ABI expression root may name.
///
/// It moved here with the expression domain: a root that names it is part of
/// that domain, so leaving the key behind would have reintroduced the external
/// side table ADR 0068 rejects.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TargetPropertyKey(String);

/// A rejected governed target-property key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetPropertyKeyError {
    /// The key was empty.
    Empty,
    /// The key exceeded [`MAX_TARGET_PROPERTY_KEY_BYTES`].
    TooLong {
        /// Byte length supplied.
        bytes: usize,
    },
}

impl fmt::Display for TargetPropertyKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a governed target-property key must not be empty"),
            Self::TooLong { bytes } => write!(
                formatter,
                "a governed target-property key of {bytes} bytes exceeds \
                 {MAX_TARGET_PROPERTY_KEY_BYTES}"
            ),
        }
    }
}

impl Error for TargetPropertyKeyError {}

impl TargetPropertyKey {
    /// Creates a validated governed target-property key.
    ///
    /// # Errors
    ///
    /// Returns [`TargetPropertyKeyError`] for an empty or over-long key.
    pub fn new(value: impl AsRef<str>) -> Result<Self, TargetPropertyKeyError> {
        Self::from_owned(value.as_ref().to_owned())
    }

    /// Validates and retains an already-owned key without copying it.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::new`], before retaining the string.
    pub fn from_owned(value: String) -> Result<Self, TargetPropertyKeyError> {
        if value.is_empty() {
            return Err(TargetPropertyKeyError::Empty);
        }
        if value.len() > MAX_TARGET_PROPERTY_KEY_BYTES {
            return Err(TargetPropertyKeyError::TooLong { bytes: value.len() });
        }
        Ok(Self(value))
    }

    /// Returns the exact key text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TargetPropertyKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
