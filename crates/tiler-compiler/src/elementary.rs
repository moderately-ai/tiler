//! The per-point scalar body of each admitted elementary family, stated once.
//!
//! # Why this module exists rather than two spellings of one chain
//!
//! An elementary family such as `tiler::silu-f32@1` is *one* semantic operation
//! whose registered normative definition pins a *composition* — `x / (1 + Exp(-x))`,
//! in that order, with the negation exact and the addition and the division
//! rounding once each. Two independent authorities in this crate have to realize
//! that composition:
//!
//! - the governed index-access lowering ([`crate::governed`]) emits it as
//!   applications of `tiler_ir::index` scalar operations, which
//!   [`crate::legality::refine_index_region`] then proves realizes the
//!   occurrence; and
//! - the request boundary ([`crate::request`]) projects it into the physical
//!   [`tiler_ir::schedule::PointwiseF32Expression`] vocabulary the scheduled
//!   region carries to the backend.
//!
//! Writing the chain twice would make the boundary *restate* a provider's
//! per-point arithmetic as its own — two claims about one meaning, only one of
//! which any authority checks. Stating it once against an abstract sink and
//! driving both realizations from it makes the two spellings one statement: a
//! change to the composition is a change in this file, and neither consumer can
//! drift from the other without the other's build breaking.
//!
//! # What this module is not
//!
//! It is not a scalar IR and it is not a second semantic authority. It carries
//! no types, no shapes, no attributes, and no accuracy claim; the registered
//! semantic definition remains the authority for what the family *means*, the
//! registered accuracy contract for what an implementation may return, and
//! occurrence refinement for whether an emitted region realizes the occurrence.
//! What this module owns is the single ordered spelling of the composition those
//! authorities all describe.

use tiler_ir::schedule::{
    PointwiseF32ExpressionAdmissionError, PointwiseF32ExpressionBuilder, PointwiseF32Value,
};

/// The exact `-1.0f32` bit pattern the activation's negation multiplies by.
pub(crate) const SILU_NEGATIVE_ONE_BITS: u32 = 0xbf80_0000;

/// The exact `1.0f32` bit pattern the activation's divisor adds.
pub(crate) const SILU_ONE_BITS: u32 = 0x3f80_0000;

/// A per-point scalar vocabulary one elementary body can be emitted into.
///
/// Deliberately the smallest set the admitted bodies reach, and deliberately
/// *not* closed over the physical node vocabulary: there is no reciprocal and no
/// negate here, so the two-rounding spellings the pinned references forbid are
/// unstatable rather than merely unselected.
///
/// `rule` names the step for an implementation whose errors carry one. A sink
/// whose error type says nothing about the step ignores it; the parameter exists
/// so the index-region sink can keep the exact diagnostics it had before this
/// module owned the chain.
pub(crate) trait ElementaryPointSink {
    /// The sink's handle to one emitted per-point value.
    type Value: Clone;
    /// Why the sink refused a step.
    type Error;

    /// Emits an exact IEEE-754 binary32 constant.
    fn constant(&mut self, bits: u32, rule: &'static str) -> Result<Self::Value, Self::Error>;

    /// Emits one ordered binary32 addition.
    fn add(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
        rule: &'static str,
    ) -> Result<Self::Value, Self::Error>;

    /// Emits one ordered binary32 multiplication.
    fn multiply(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
        rule: &'static str,
    ) -> Result<Self::Value, Self::Error>;

    /// Emits one ordered binary32 division.
    fn divide(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
        rule: &'static str,
    ) -> Result<Self::Value, Self::Error>;

    /// Emits the precise natural exponential.
    fn exp(
        &mut self,
        argument: Self::Value,
        rule: &'static str,
    ) -> Result<Self::Value, Self::Error>;
}

/// Emits the per-point body of one `tiler::silu-f32@1` occurrence.
///
/// The chain is the pinned reference read left to right: `x * -1.0`, then the
/// precise exponential, then `1.0 + e`, then `x / d`. Three properties of that
/// chain are load-bearing rather than incidental, and they are stated here
/// because this is now the only place the chain is written.
///
/// **The negation is a multiplication by `-1.0`, and it is exact.** IEEE-754
/// multiplication by negative one flips the sign of every operand — both zeros
/// and both infinities included — with no rounding, so it delivers exactly what
/// the reference's "exact sign manipulation" means. There is no negate step to
/// reach for and this does not need one.
///
/// **The divisor is `1.0 + e`, in that order.** Binary32 addition is commutative,
/// so the order is not observable here; it is written this way because the
/// reference is, and a reader comparing the two should not have to reconcile a
/// difference that has no consequence.
///
/// **The result is a division.** `x * (1.0 / d)` rounds twice and would be a
/// different binary32 function — measurably so at `0xc2b00000`, where the two
/// spellings differ by one ULP. [`ElementaryPointSink`] has no reciprocal step,
/// so the substitution is unstatable here rather than merely forbidden.
///
/// # Errors
///
/// Returns the sink's own error, unchanged: this function decides the chain and
/// never the admissibility of a step.
pub(crate) fn silu_point_body<S: ElementaryPointSink>(
    sink: &mut S,
    argument: &S::Value,
) -> Result<S::Value, S::Error> {
    let negative_one = sink.constant(SILU_NEGATIVE_ONE_BITS, "silu-negative-one")?;
    let negated = sink.multiply(argument.clone(), negative_one, "silu-negation")?;
    let exponential = sink.exp(negated, "silu-exponential")?;
    let one = sink.constant(SILU_ONE_BITS, "silu-one")?;
    let divisor = sink.add(one, exponential, "silu-divisor")?;
    sink.divide(argument.clone(), divisor, "silu-division")
}

/// Emits an elementary body into the physical `f32` expression vocabulary.
///
/// The request boundary's realization of an elementary occurrence. It holds the
/// borrow rather than the builder so the caller keeps minting the surrounding
/// expression's nodes after the body is emitted.
pub(crate) struct PointwiseExpressionSink<'a> {
    builder: &'a mut PointwiseF32ExpressionBuilder,
}

impl<'a> PointwiseExpressionSink<'a> {
    /// Borrows one expression builder as an elementary sink.
    pub(crate) const fn new(builder: &'a mut PointwiseF32ExpressionBuilder) -> Self {
        Self { builder }
    }
}

impl ElementaryPointSink for PointwiseExpressionSink<'_> {
    type Value = PointwiseF32Value;
    type Error = PointwiseF32ExpressionAdmissionError;

    fn constant(&mut self, bits: u32, _rule: &'static str) -> Result<Self::Value, Self::Error> {
        self.builder.constant(bits)
    }

    fn add(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
        _rule: &'static str,
    ) -> Result<Self::Value, Self::Error> {
        self.builder.add(lhs, rhs)
    }

    fn multiply(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
        _rule: &'static str,
    ) -> Result<Self::Value, Self::Error> {
        self.builder.multiply(lhs, rhs)
    }

    fn divide(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
        _rule: &'static str,
    ) -> Result<Self::Value, Self::Error> {
        self.builder.divide(lhs, rhs)
    }

    fn exp(
        &mut self,
        argument: Self::Value,
        _rule: &'static str,
    ) -> Result<Self::Value, Self::Error> {
        self.builder.exp(argument)
    }
}

#[cfg(test)]
mod tests {
    use super::{PointwiseExpressionSink, silu_point_body};
    use tiler_ir::schedule::{
        InputOrdinal, PointwiseF32Expression, PointwiseF32ExpressionBuilder, PointwiseF32Node,
        PointwiseF32NodeId,
    };

    /// Renders one verified expression from its root, operands in order.
    ///
    /// Rendering rather than indexing into the node vector: a verified
    /// expression's storage order is a deterministic topological derivation the
    /// builder owns, so an assertion written against node positions would fail
    /// when that derivation changed and pass when the *composition* changed at
    /// the same positions. The rendering is a function of the composition alone.
    fn render(expression: &PointwiseF32Expression, node: PointwiseF32NodeId) -> String {
        let index = usize::try_from(node.index()).expect("a dense node ordinal");
        let operand = |id| render(expression, id);
        match expression.nodes()[index] {
            PointwiseF32Node::Input { ordinal } => format!("input({})", ordinal.get()),
            PointwiseF32Node::Constant { bits } => format!("constant(0x{bits:08x})"),
            PointwiseF32Node::Add { lhs, rhs } => {
                format!("add({}, {})", operand(lhs), operand(rhs))
            }
            PointwiseF32Node::Multiply { lhs, rhs } => {
                format!("multiply({}, {})", operand(lhs), operand(rhs))
            }
            PointwiseF32Node::Divide { lhs, rhs } => {
                format!("divide({}, {})", operand(lhs), operand(rhs))
            }
            PointwiseF32Node::Exp { argument } => format!("exp({})", operand(argument)),
            PointwiseF32Node::Rsqrt { argument } => format!("rsqrt({})", operand(argument)),
        }
    }

    /// The one statement of the chain projects to the pinned composition.
    ///
    /// Spelled out rather than compared against a second call of the same
    /// function, which would assert only that the function is deterministic. The
    /// literal is what a reader checks against the registered normative
    /// definition, and every reordering the composition admits changes it: a
    /// reciprocal-and-multiply spelling, a `e + 1.0` divisor, and a dividend
    /// taken from the negated value rather than from the operand all render
    /// differently.
    #[test]
    fn the_activation_body_projects_to_the_pinned_expression() {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let argument = builder.input(InputOrdinal::FIRST).expect("one input leaf");
        let root = {
            let mut sink = PointwiseExpressionSink::new(&mut builder);
            silu_point_body(&mut sink, &argument).expect("the activation body is admitted")
        };
        let expression = builder
            .build(root)
            .expect("the activation body is a region");
        assert_eq!(
            render(&expression, expression.root()),
            "divide(input(0), add(constant(0x3f800000), \
             exp(multiply(input(0), constant(0xbf800000)))))",
        );
        // Seven nodes, not eight: the operand is read once and shared by the
        // negation and the division, which is what makes the emitted region read
        // its input tensor once per point.
        assert_eq!(expression.nodes().len(), 7);
        assert_eq!(expression.input_count(), 1);
    }
}
