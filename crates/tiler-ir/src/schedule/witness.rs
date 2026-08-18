//! The realization one verified scheduled region pinned, and what it leaves open.
//!
//! Two vocabularies, and the split between them is the whole design. A
//! [`RealizationWitness`] aggregates the *concrete choices* a region recorded at
//! the sites its contract left free; an [`UnpinnedFreedomSite`] names a site the
//! contract granted and the plan records no choice for. Neither can say a plan
//! conforms — that is an oracle's answer over values, and this module never sees
//! one.
//!
//! # Where this comes from
//!
//! [The freedom-sites enumeration](../../../../docs/research/reference/plan-freedom-sites.md)
//! enumerates every (dimension, construct) pair at which a categorically granted
//! numerical permission, if spent, would change a compilable program's observable
//! bits, and sorts them five ways: witness, witness-unevaluable, mirror,
//! undeclared, and unspendable. Its Part 7.2 drafted this surface; Tom accepted
//! items A and B on 2026-08-06 and redirected item C to the plain-scalar form, so
//! the aggregation lives here alone and `tiler-reference` keeps taking plain
//! scalars. A caller destructures a witness at the call rather than handing one
//! across the crate boundary.
//!
//! # Why aggregation is not declaration
//!
//! Every accessor reads a construct the region already carries, and the witness
//! borrows the region's own records rather than copying them, so a witness cannot
//! state a realization the plan does not. That is what separates a witness from
//! the *mirrors* the enumeration names: a field that carries the contract's
//! resolution rather than the plan's choice determines nothing, because two plans
//! choosing differently agree on it. This module never reads
//! `permits_reassociation` or `permits_permutation` off a topology for that
//! reason — they are the contract's resolution, cross-checked against the
//! region's realization by the intrinsic verifier, and the realization is what
//! [`RealizationWitness::realization`] hands back.

use super::cooperative::ContributorArrival;
use super::model::{
    ContributorOrder, ContributorPartition, ReductionPass, ReductionTopology, RegionProgram,
    ScalarProgram, VerifiedScheduledRegion, cooperative_tile, region_arithmetic_type,
};
use super::numerics::{ArithmeticType, NumericalRealization};
use super::pointwise::{PointwiseF32Expression, PointwiseF32Node};
use super::pointwise_bf16::PointwiseBf16Node;
use crate::shape::{Axis, Shape};

/// The concrete realization one verified scheduled region pinned at every site
/// its contract left free.
///
/// Aggregated from the region rather than declared beside it: every accessor
/// below reads a construct the region already carries, so a witness cannot
/// disagree with the plan it describes.
///
/// # What it covers, and what it deliberately does not
///
/// It covers the sites the enumeration classifies as witnesses a reference path
/// can evaluate — the two subnormal dimensions through
/// [`Self::realization`], the serial fold through [`Self::reduced_axes`] and
/// [`Self::order`], the multi-pass split through [`Self::contributor_partition`],
/// [`Self::pass`], and [`Self::accumulation`], the cooperative tile through those
/// plus [`Self::arrival`] and [`Self::rounds`], and the contraction's fold
/// through [`Self::contracted_shape`].
///
/// It does **not** close the sites the enumeration marks undeclared, and no
/// aggregation could: nothing in a plan records whether the emitted body fused an
/// adjacent multiply, which semantic candidate the portfolio selected, or what a
/// backend compiler did with the order it was handed.
/// [`Self::unpinned_freedom_site`] names the first such site rather than leaving
/// a caller to discover it.
///
/// # Equality is deliberately not derived
///
/// What `==` would mean here is exactly the *converse* of the enumeration's
/// determination property: two plans producing identical bits must not disagree
/// on the witness. That converse rests on the pointwise expression builder's
/// canonicalization being a function of the program rather than of its spelling,
/// and the enumeration states that claim untested. It is now tested, and the
/// result is that it holds for the two mitigations the record names and fails for
/// a third spelling it does not — see
/// `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` in
/// this module. A derived `==` would offer callers a comparison stronger than
/// that evidence supports. Every accessor's own return type is comparable, so the
/// pairwise per-site comparison the record's refutation procedure describes stays
/// available.
#[derive(Clone, Copy, Debug)]
pub struct RealizationWitness<'a> {
    program: &'a RegionProgram,
    reduction: &'a ReductionTopology,
}

impl<'a> RealizationWitness<'a> {
    /// Aggregates the witness one verified region determines.
    #[must_use]
    pub const fn of(region: &'a VerifiedScheduledRegion) -> Self {
        let region = region.region();
        Self {
            program: &region.index.program,
            reduction: &region.schedule.reduction,
        }
    }

    /// The declared numerical realization the witness was aggregated under,
    /// when the region's computation class declares one.
    ///
    /// The two subnormal dimensions — the enumeration's sites 1.1 and 2.1, and
    /// the only two the reference realizes exactly — are read from here rather
    /// than restated as accessors of their own, because
    /// `ReferenceNumericalConformance` already takes them in this shape.
    ///
    /// `None` is the partitioned copy's explicit answer: a copy performs no
    /// arithmetic and declares no realization, so there is no realization for
    /// a witness to aggregate — a proved absence, not a missing field.
    #[must_use]
    pub const fn realization(&self) -> Option<&'a NumericalRealization> {
        self.program.numerical()
    }

    /// The scalar program of an arithmetic region, or `None` for the copy.
    ///
    /// The internal read every program-classifying accessor below goes
    /// through, so the copy's "no fold, no expression, no freedom site" answer
    /// is stated once rather than re-derived per accessor.
    const fn scalar(&self) -> Option<&'a ScalarProgram> {
        match self.program {
            RegionProgram::Numerical { scalar, .. } => Some(scalar),
            RegionProgram::PartitionedCopy(_) => None,
        }
    }

    /// The contributor combination order, for a topology that states one.
    ///
    /// Optional for the mirror reason
    /// [the record](../../../../docs/research/reference/plan-freedom-sites.md)
    /// names. A region whose topology is [`ReductionTopology::None`] combines no
    /// contributors, so a total accessor would have to return the vocabulary's
    /// single variant for a sequence that does not exist — a value two plans
    /// agree on for no reason about either. `None` states the absence instead.
    #[must_use]
    pub const fn order(&self) -> Option<ContributorOrder> {
        match self.reduction {
            ReductionTopology::None => None,
            ReductionTopology::Serial { order, .. }
            | ReductionTopology::MultiPass { order, .. }
            | ReductionTopology::Contraction { order, .. }
            | ReductionTopology::LiveContraction { order, .. }
            | ReductionTopology::CooperativeWorkgroup { order, .. }
            | ReductionTopology::CooperativeContraction { order, .. } => Some(*order),
        }
    }

    /// The reduced axes the topology folds, in canonical ascending order.
    ///
    /// The record's Part 2 names `axes` as half of site 4.1's field set while its
    /// Part 7.2 draft has no accessor for it. This accessor carries that named
    /// field, alongside
    /// [`Self::pass`], [`Self::contracted_shape`], [`Self::fold_epilogue`], and
    /// [`Self::unpinned_freedom_site`].
    ///
    /// Empty for a region that folds nothing, and empty for a contraction —
    /// whose contracted space is a shape rather than an axis set, and is
    /// [`Self::contracted_shape`].
    #[must_use]
    pub fn reduced_axes(&self) -> &'a [Axis] {
        match self.reduction {
            ReductionTopology::None
            | ReductionTopology::Contraction { .. }
            | ReductionTopology::CooperativeContraction { .. }
            | ReductionTopology::LiveContraction { .. } => &[],
            ReductionTopology::Serial { axes, .. }
            | ReductionTopology::MultiPass { axes, .. }
            | ReductionTopology::CooperativeWorkgroup { axes, .. } => axes,
        }
    }

    /// The contracted iteration space a contraction folds, if the region is one.
    ///
    /// For the reason [`Self::reduced_axes`] states: the record names
    /// `contracted_shape` as half of site 4.4's field set and drafts no accessor
    /// for it.
    ///
    /// Site 4.4's *spend* population is empty by the variant's own contract — the
    /// fold is the declared contributor sequence itself, so it consumes no
    /// reassociation — but the shape is still what says which sequence that is.
    #[must_use]
    pub const fn contracted_shape(&self) -> Option<&'a Shape> {
        match self.reduction {
            ReductionTopology::Contraction {
                contracted_shape, ..
            }
            | ReductionTopology::CooperativeContraction {
                contracted_shape, ..
            } => Some(contracted_shape),
            ReductionTopology::None
            | ReductionTopology::Serial { .. }
            | ReductionTopology::MultiPass { .. }
            | ReductionTopology::LiveContraction { .. }
            | ReductionTopology::CooperativeWorkgroup { .. } => None,
        }
    }

    /// The declared contributor coverage, for a topology that states one.
    #[must_use]
    pub const fn contributor_coverage(&self) -> Option<crate::schedule::ContributorCoverage> {
        match self.reduction {
            ReductionTopology::MultiPass { coverage, .. }
            | ReductionTopology::CooperativeWorkgroup { coverage, .. } => Some(*coverage),
            ReductionTopology::None
            | ReductionTopology::Serial { .. }
            | ReductionTopology::Contraction { .. }
            | ReductionTopology::CooperativeContraction { .. }
            | ReductionTopology::LiveContraction { .. } => None,
        }
    }

    /// The declared contributor split, for a topology that states one.
    #[must_use]
    pub const fn contributor_partition(&self) -> Option<ContributorPartition> {
        match self.reduction {
            ReductionTopology::MultiPass { coverage, .. }
            | ReductionTopology::CooperativeWorkgroup { coverage, .. } => {
                Some(coverage.partition())
            }
            ReductionTopology::None
            | ReductionTopology::Serial { .. }
            | ReductionTopology::Contraction { .. }
            | ReductionTopology::CooperativeContraction { .. }
            | ReductionTopology::LiveContraction { .. } => None,
        }
    }

    /// Which pass of a multi-dispatch split this region realizes.
    ///
    /// Present for the reason [`Self::reduced_axes`] states.
    ///
    /// The record names `pass` in site 4.2's field set and it is load-bearing
    /// rather than descriptive: a partial pass folds original contributors and a
    /// final pass folds the partials, so two regions of one split agreeing on
    /// every other field still commit different values.
    #[must_use]
    pub const fn pass(&self) -> Option<ReductionPass> {
        match self.reduction {
            ReductionTopology::MultiPass { pass, .. } => Some(*pass),
            ReductionTopology::None
            | ReductionTopology::Serial { .. }
            | ReductionTopology::Contraction { .. }
            | ReductionTopology::LiveContraction { .. }
            | ReductionTopology::CooperativeWorkgroup { .. }
            | ReductionTopology::CooperativeContraction { .. } => None,
        }
    }

    /// The width every combining step is performed at.
    ///
    /// Total rather than optional, and the two arms are different facts. A split
    /// *declares* its accumulation, because a strategy accumulating at a narrower
    /// width than the contract admits is a different computation; every other
    /// topology performs its combining step at the width its scalar program's own
    /// arithmetic is in, which `region_arithmetic_type` derives.
    ///
    /// **The enumeration's site 4.8 has an empty spend population at this base,
    /// and that is a correction to the record rather than a reading of it.** The
    /// record classifies the declared width as a witness no oracle honours,
    /// because `strict_partial_sums_under` takes no width parameter. The
    /// intrinsic verifier now refuses a declared accumulation that differs from
    /// the region's own arithmetic type — at both parallel topologies — so a
    /// verified region's declared width *is* its element width and the reference
    /// answers for exactly it. Widening a fold to a second accumulator width is
    /// what would reopen the site.
    #[must_use]
    pub fn accumulation(&self) -> ArithmeticType {
        match self.reduction {
            ReductionTopology::MultiPass { accumulation, .. }
            | ReductionTopology::CooperativeWorkgroup { accumulation, .. }
            | ReductionTopology::CooperativeContraction { accumulation, .. } => *accumulation,
            ReductionTopology::None
            | ReductionTopology::Serial { .. }
            | ReductionTopology::Contraction { .. }
            | ReductionTopology::LiveContraction { .. } => region_arithmetic_type(self.program),
        }
    }

    /// The staged-partial arrival, for a cooperative tile.
    #[must_use]
    pub const fn arrival(&self) -> Option<ContributorArrival> {
        match self.reduction {
            ReductionTopology::CooperativeWorkgroup { arrival, .. } => Some(*arrival),
            ReductionTopology::None
            | ReductionTopology::Serial { .. }
            | ReductionTopology::MultiPass { .. }
            | ReductionTopology::Contraction { .. }
            | ReductionTopology::CooperativeContraction { .. }
            | ReductionTopology::LiveContraction { .. } => None,
        }
    }

    /// The rounds a cooperative tile executes, for a tile that states one.
    #[must_use]
    pub fn rounds(&self) -> Option<u64> {
        cooperative_tile(self.reduction).map(|tile| tile.rounds)
    }

    /// The pinned per-point expression, when the region's scalar program is one.
    ///
    /// The grouping site 4.5 names. The expression *is* the grouping — its nodes
    /// are explicit binary operations over dense topological ordinals — so no
    /// separate grouping field exists or should.
    ///
    /// Reading it does not make it evaluable: no reference path accepts a
    /// `PointwiseF32Expression`, which is why
    /// [`Self::unpinned_freedom_site`] refuses under a reassociation-permitting
    /// contract rather than letting a caller assume otherwise.
    #[must_use]
    pub const fn pointwise_f32(&self) -> Option<&'a PointwiseF32Expression> {
        // A partitioned copy pins no expression: `scalar` answers `None`
        // explicitly for it, and the match below is exhaustive over the
        // arithmetic vocabulary alone.
        let Some(scalar) = self.scalar() else {
            return None;
        };
        match scalar {
            ScalarProgram::PointwiseF32(expression) => Some(expression),
            ScalarProgram::PointwiseBf16(_)
            | ScalarProgram::StrictAffineU4Dequantize { .. }
            | ScalarProgram::StrictSerialSum { .. }
            | ScalarProgram::FusedMultiplyAddSerialSum { .. }
            | ScalarProgram::SquaredSerialSum { .. }
            | ScalarProgram::SquaredSerialSumThenEpilogue { .. }
            | ScalarProgram::StrictTensorContraction { .. }
            | ScalarProgram::StrictSerialMaximum { .. } => None,
        }
    }

    /// The chain a fold applies to its folded value before committing it.
    ///
    /// This is a *new* site rather than a drafted field the record omitted an
    /// accessor for.
    /// [`ScalarProgram::SquaredSerialSumThenEpilogue`] did not exist when the
    /// enumeration was written at `c335bb5b`, and it pins a second
    /// [`PointwiseF32Expression`] inside a reduction. Its class is site 4.5's —
    /// a witness the plan states exactly and no reference path evaluates — so it
    /// is aggregated here and refused by name rather than folded into
    /// [`Self::pointwise_f32`], whose accessor answers for the region's *scalar
    /// program* and not for every expression a region carries.
    #[must_use]
    pub const fn fold_epilogue(&self) -> Option<&'a PointwiseF32Expression> {
        // A partitioned copy folds nothing, so it carries no epilogue; the
        // explicit `None` from `scalar` states that rather than defaulting it.
        let Some(scalar) = self.scalar() else {
            return None;
        };
        match scalar {
            ScalarProgram::SquaredSerialSumThenEpilogue { epilogue, .. } => Some(epilogue),
            ScalarProgram::PointwiseF32(_)
            | ScalarProgram::PointwiseBf16(_)
            | ScalarProgram::StrictAffineU4Dequantize { .. }
            | ScalarProgram::StrictSerialSum { .. }
            | ScalarProgram::FusedMultiplyAddSerialSum { .. }
            | ScalarProgram::SquaredSerialSum { .. }
            | ScalarProgram::StrictTensorContraction { .. }
            | ScalarProgram::StrictSerialMaximum { .. } => None,
        }
    }

    /// Returns the first freedom site this witness does not pin, if any.
    ///
    /// The record originally drafted
    /// [`UnpinnedFreedomSite`] as the refusal of a `ReferenceNumericalConformance`
    /// constructor; Tom's redirection of that item to the plain-scalar form left
    /// the refusal without a producer, so the decision it encodes is sited here
    /// beside the aggregation it is about.
    ///
    /// `None` is deliberately not a conformance claim, and this returns an
    /// [`Option`] rather than a `Result` for that reason: it says the enumeration
    /// found no site this region's contract grants and this region's plan leaves
    /// open, which is a statement about the *table*, not about values. Nothing
    /// here computes, compares, or admits a result.
    ///
    /// # The order, and why it is not arbitrary
    ///
    /// A region can leave several sites open at once, so "first" needs a rule.
    /// The region-specific refusals lead and the target-wide one trails, because
    /// a refusal that would fire for every region under this contract says
    /// nothing about *this* region and is the least useful thing to report:
    ///
    /// 1. [`UnpinnedFreedomSite::ContractionUnrecorded`] — this region's fold
    ///    step has an adjacent multiply the plan's own vocabulary could record a
    ///    choice for and does not.
    /// 2. [`UnpinnedFreedomSite::RealizationNotEvaluable`] — this region pins
    ///    something exactly and no reference path evaluates it.
    /// 3. [`UnpinnedFreedomSite::BackendOrderUndeclared`] — the plan states the
    ///    adjacency exactly and no target fact declares what a backend compiler
    ///    does with it.
    #[must_use]
    pub fn unpinned_freedom_site(&self) -> Option<UnpinnedFreedomSite> {
        // A partitioned copy has no freedom site, stated explicitly rather
        // than defaulted: it declares no realization, so no permission exists
        // to be spent, and it states no fold or expression a spent permission
        // could reach. Every enumeration site is a (dimension, construct) pair
        // and the copy carries neither half.
        let RegionProgram::Numerical { scalar, numerical } = self.program else {
            return None;
        };
        let contraction_permitted = numerical.permits_contraction();
        if contraction_permitted && let Some(operation) = unrecorded_fold_contraction(scalar) {
            return Some(UnpinnedFreedomSite::ContractionUnrecorded { operation });
        }
        if let Some(reason) = self.unevaluable_realization() {
            return Some(UnpinnedFreedomSite::RealizationNotEvaluable { reason });
        }
        if contraction_permitted && expression_states_contraction_adjacency(scalar) {
            return Some(UnpinnedFreedomSite::BackendOrderUndeclared);
        }
        None
    }

    /// Returns what this region pins that no reference path can evaluate.
    ///
    /// The loop-carried tile is checked before the two expression sites because
    /// it is unevaluable *structurally* — `strict_partial_sums_under`'s
    /// `partition * chunk + within` index arithmetic is a flat blocked split and
    /// a multi-round tile's contributor order is `r * partitions + p` — so it
    /// holds under a strict contract, where the expression sites do not.
    ///
    /// Both expression sites are gated on the reassociation permission, and the
    /// gate is what keeps this from refusing every elementwise region ever
    /// compiled. A freedom site is a place a *granted* permission could be spent;
    /// under a forbidding contract the minted expression is a total deterministic
    /// function of the caller's own program, which the semantic evaluator already
    /// evaluates exactly. What a permitting contract removes is that
    /// correspondence: the rewritten candidate is what was projected, and the
    /// plan retains only the candidate's key string.
    fn unevaluable_realization(&self) -> Option<UnevaluableRealization> {
        if let Some(rounds) = self.rounds()
            && rounds > 1
        {
            return Some(UnevaluableRealization::LoopCarriedCooperativeTile { rounds });
        }
        // A copy pins no expression and grants no permission; the explicit
        // destructure states that rather than defaulting through a helper.
        let RegionProgram::Numerical { scalar, numerical } = self.program else {
            return None;
        };
        if !numerical.permits_reassociation() {
            return None;
        }
        match scalar {
            ScalarProgram::PointwiseF32(_) | ScalarProgram::PointwiseBf16(_) => {
                Some(UnevaluableRealization::PointwiseExpression)
            }
            ScalarProgram::SquaredSerialSumThenEpilogue { .. } => {
                Some(UnevaluableRealization::FoldEpilogueExpression)
            }
            ScalarProgram::StrictAffineU4Dequantize { .. }
            | ScalarProgram::StrictSerialSum { .. }
            | ScalarProgram::FusedMultiplyAddSerialSum { .. }
            | ScalarProgram::SquaredSerialSum { .. }
            | ScalarProgram::StrictTensorContraction { .. }
            | ScalarProgram::StrictSerialMaximum { .. } => None,
        }
    }
}

/// Returns the fold step whose adjacent multiply this program records no fusion
/// choice for.
///
/// Four scalar programs state such a step, at three distinct adjacencies —
/// [`ScalarProgram::SquaredSerialSum`] and
/// [`ScalarProgram::SquaredSerialSumThenEpilogue`] share one, because an epilogue
/// over the folded value does not change what the fold combines.
///
/// Exactly one of the four states a *field* for its adjacency:
/// [`ScalarProgram::FusedMultiplyAddSerialSum`]'s `contraction`. That field is a
/// mirror rather than a witness — the compiler sets it from the resolved
/// contract, so it answers "was I allowed to fuse" and never "did I fuse" — so it
/// is named here beside the three that carry no field at all rather than read.
///
/// Exhaustive, so a scalar program added later states whether its fold has an
/// adjacency instead of inheriting the answer of whichever one it resembles.
fn unrecorded_fold_contraction(program: &ScalarProgram) -> Option<UnrecordedFoldContraction> {
    match program {
        ScalarProgram::StrictTensorContraction { .. } => {
            Some(UnrecordedFoldContraction::ContractedProduct)
        }
        ScalarProgram::SquaredSerialSum { .. }
        | ScalarProgram::SquaredSerialSumThenEpilogue { .. } => {
            Some(UnrecordedFoldContraction::SquaredContributor)
        }
        ScalarProgram::FusedMultiplyAddSerialSum { .. } => {
            Some(UnrecordedFoldContraction::ScaleBiasContributor)
        }
        // A bare sum's step is `accumulator + contributor` and an extrema fold
        // performs no arithmetic at all, so neither has a multiply for a
        // contraction permission to reach. The strict-affine decode multiplies
        // but folds nothing, and its own verification refuses a realization that
        // permits contraction. No pointwise program folds; an adjacency inside
        // one is `expression_states_contraction_adjacency`'s business.
        ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::StrictSerialMaximum { .. }
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::PointwiseF32(_)
        | ScalarProgram::PointwiseBf16(_) => None,
    }
}

/// Returns whether this program's pointwise expression states a multiply whose
/// value an addition consumes.
///
/// The form a floating-point contraction fuses is `a * b + c`, so an expression
/// with no addition over a multiply has nothing for the dropped
/// `-ffp-contract=off` to change. Deciding it on the plan's own nodes is what
/// keeps the claim inside what the plan states: whether a backend's emitted body
/// contains adjacencies of its own is a target fact this vocabulary does not
/// carry, and refusing every region under a permitting contract would report an
/// open site for a fold with no multiply in it.
///
/// Only the two pointwise programs are examined. Every other program that states
/// an adjacency states it in its *fold*, which
/// [`unrecorded_fold_contraction`] names first and more precisely — including
/// [`ScalarProgram::SquaredSerialSumThenEpilogue`], whose epilogue is never
/// reached here because its fold refuses ahead of it.
fn expression_states_contraction_adjacency(program: &ScalarProgram) -> bool {
    match program {
        ScalarProgram::PointwiseF32(expression) => {
            let nodes = expression.nodes();
            nodes.iter().any(|node| match node {
                PointwiseF32Node::Add { lhs, rhs } => [lhs, rhs].into_iter().any(|operand| {
                    matches!(
                        usize::try_from(operand.index())
                            .ok()
                            .and_then(|index| nodes.get(index)),
                        Some(PointwiseF32Node::Multiply { .. })
                    )
                }),
                PointwiseF32Node::Input { .. }
                | PointwiseF32Node::Constant { .. }
                | PointwiseF32Node::Multiply { .. }
                | PointwiseF32Node::Divide { .. }
                | PointwiseF32Node::Exp { .. }
                | PointwiseF32Node::Rsqrt { .. } => false,
            })
        }
        ScalarProgram::PointwiseBf16(expression) => {
            let nodes = expression.nodes();
            nodes.iter().any(|node| match node {
                PointwiseBf16Node::Add { lhs, rhs } => [lhs, rhs].into_iter().any(|operand| {
                    matches!(
                        usize::try_from(operand.index())
                            .ok()
                            .and_then(|index| nodes.get(index)),
                        Some(PointwiseBf16Node::Multiply { .. })
                    )
                }),
                PointwiseBf16Node::Input { .. }
                | PointwiseBf16Node::Constant { .. }
                | PointwiseBf16Node::Multiply { .. } => false,
            })
        }
        ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        | ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | ScalarProgram::StrictTensorContraction { .. }
        | ScalarProgram::StrictSerialMaximum { .. } => false,
    }
}

/// A freedom site the plan grants and the witness does not pin.
///
/// **This enum must never gain a `Conforms`-shaped arm.** It exists to separate
/// "this plan is unqualifiable" from "this plan is wrong", and an arm asserting
/// conformance would make a refusal vocabulary into an admission one — the
/// elimination
/// [the permitted-divergence oracle derivation](../../../../docs/research/reference/permitted-divergence-oracle.md)
/// already ran. Naming the site is the whole content: a caller that cannot
/// proceed can say which freedom it was that stopped it.
///
/// `#[non_exhaustive]` because the population is the enumeration's, and the
/// enumeration is exhaustive over today's vocabulary rather than over the future:
/// a twelfth dimension or a tenth scalar program is classified by applying the
/// record's Part 1 rule, and a new refusal must land additively at every consumer
/// rather than break it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum UnpinnedFreedomSite {
    /// The contract permits contraction and the region's fold has an adjacent
    /// multiply the plan records no choice for.
    ///
    /// The enumeration's sites 3.1, 3.2, and 3.4. A fused step rounds once where
    /// a separate one rounds twice, so two plans agreeing on every field of the
    /// variant can still differ in bits.
    ContractionUnrecorded {
        /// The fold step whose adjacency is unrecorded.
        operation: UnrecordedFoldContraction,
    },
    /// The contract permits contraction and no target fact declares whether the
    /// backend compiler preserves the emitted order.
    ///
    /// The enumeration's site 3.3. `MetalNumericalRequirement::NoFloatingPointContraction`
    /// — which renders `-ffp-contract=off` — is inserted only when the
    /// realization *forbids* contraction, which is correct for a
    /// compiler-selection set and is exactly why it cannot double as a witness:
    /// under a permitting contract the one thing that supplied the pin is dropped
    /// and the executed order becomes a property of a backend compiler no target
    /// profile declares.
    ///
    /// It is the record's second predicted refutation, and the one whose
    /// resolution needs a device measurement rather than a field.
    BackendOrderUndeclared,
    /// The witness pins an order no reference evaluator implements.
    RealizationNotEvaluable {
        /// What the plan pinned that nothing can evaluate.
        reason: UnevaluableRealization,
    },
}

/// The fold step whose adjacent multiply a plan records no fusion choice for.
///
/// Named by the *adjacency* rather than by the scalar program, because that is
/// what a contraction permission reaches: the squaring fold and the squaring fold
/// carrying an epilogue perform the identical combining step, so they name one
/// variant here and differ elsewhere.
///
/// `#[non_exhaustive]` for the reason [`UnpinnedFreedomSite`] is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum UnrecordedFoldContraction {
    /// `accumulator + contributor * contributor`, the squaring prologue's step.
    SquaredContributor,
    /// `accumulator + (scale * contributor + bias)`, the fused affine step.
    ///
    /// The one adjacency with a plan-side field, and the field is a mirror. The
    /// compiler sets `ScalarProgram::FusedMultiplyAddSerialSum`'s `contraction`
    /// from the resolved contract, so it answers "was I allowed to fuse" and
    /// never "did I fuse"; the intrinsic verifier separately requires it `false`
    /// on every admitted region, so it does not even carry the contract's
    /// resolution reliably.
    ScaleBiasContributor,
    /// `accumulator + left * right`, one contraction's per-point product.
    ContractedProduct,
}

/// What a plan pinned exactly that no reference path can evaluate.
///
/// Every variant is a site the enumeration classifies **witness, unevaluable** —
/// the plan records the concrete choice and no reference evaluator accepts it —
/// which is a different failure from an undeclared site and needs a different
/// repair. An undeclared site wants a field; these want an evaluator, a
/// retention decision, or a wider index relation.
///
/// `#[non_exhaustive]` for the reason [`UnpinnedFreedomSite`] is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum UnevaluableRealization {
    /// A pointwise expression the contract permitted the compiler to regroup.
    ///
    /// The enumeration's sites 4.5 and 4.6, at both widths. The grouping is
    /// pinned — the expression's nodes *are* it — and unevaluable for a reason
    /// that is a retention gap rather than an evaluator gap: the reference
    /// already evaluates a `SemanticProgram` exactly, and the rewritten candidate
    /// this expression was projected from is dropped after compilation, leaving
    /// only the candidate key string reachable through explain.
    PointwiseExpression,
    /// A fold's epilogue expression, under the same permission.
    ///
    /// A site the enumeration does not carry, because
    /// [`ScalarProgram::SquaredSerialSumThenEpilogue`] did not exist at its base.
    /// Its class is [`Self::PointwiseExpression`]'s and its repair is the same
    /// one; it is a variant of its own because the two sit at different
    /// constructs, and a caller repairing one has not repaired the other.
    FoldEpilogueExpression,
    /// A cooperative tile whose phases repeat.
    ///
    /// The enumeration's site 4.3 at `rounds > 1`. The tile pins its order
    /// exactly — participant `p` of round `r` owns the contiguous contributor
    /// range at index `r * partitions + p` — and no reference path states it:
    /// `strict_partial_sums_under`'s index arithmetic is the flat blocked
    /// `partition * chunk + within`, which is the single-round order and a
    /// different sequence at every later round.
    ///
    /// Unlike its two siblings this holds under a strict contract, because it is
    /// the *index relation* that is unstatable rather than a permission that was
    /// spent.
    LoopCarriedCooperativeTile {
        /// Rounds the tile's phase sequence executes.
        rounds: u64,
    },
}

#[cfg(test)]
mod tests;
