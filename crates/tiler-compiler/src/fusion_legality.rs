//! Derived fusion-legality authority for one proposed region occurrence.
//!
//! Region formation proposes candidates; this module answers a different
//! question about one of them: whether implementing that region as a single
//! fused kernel preserves the request's numerical contract exactly. Unlike a
//! graph-shape recognizer or a fixed proof label, legality here is *derived*.
//! For every member operation the derivation resolves a per-operation numerical
//! capability (its fusion role), then discharges each numerical, effect, and
//! materialization obligation against that role, the reached semantic
//! definition, and the effective numerical policy. The result is one of three
//! typed outcomes:
//!
//! - [`FusionLegality::Legal`] carries replayable evidence: every obligation is
//!   discharged with a labelled [`FusionEvidenceClass`];
//! - [`FusionLegality::Rejected`] names the obligation a fused realization is
//!   proved to violate; and
//! - [`FusionLegality::Unknown`] names the obligation the bounded profile cannot
//!   yet establish, failing closed rather than approximating an accept.
//!
//! The proof separates two identities that must never be conflated, mirroring
//! the region and refinement authorities:
//!
//! - [`FusionLegalityContent`] is reusable and site/provider-independent: the
//!   canonical region-content identity, the numerical-contract key, the derived
//!   structural counts, and the ordered discharged obligations with their
//!   evidence classes. It contains no selected provider and no graph site.
//! - [`FusionLegalityProof`] binds that content to one exact occurrence: the
//!   region-occurrence identity, the reached semantic definitions, the selected
//!   fusion-capability provider, and the ordered value/access bindings.
//!
//! The five evidence classes named by the correctness contract — normative
//! guarantee, sound proof, exhaustive-finite, empirical, and unknown — are kept
//! distinct and are never collapsed into one another.
//!
//! # Legality is derived per width, never carried across one
//!
//! Every obligation below is discharged for the arithmetic type the *contract*
//! states, and three of them read that width directly: the conversion-boundary
//! obligation compares each member's operand and result encodings against the
//! region's own dtype, the exceptional-value obligation compares the contract's
//! NaN payload against the one that width canonically produces, and the closed
//! contraction proof is keyed on exact operation keys rather than on roles.
//!
//! That is not fastidiousness. Finding 28 of the Apple numerical behaviour
//! record measures a row on which `f16` fuses a written multiply/add pair under
//! `safe` with `-ffp-contract=fast` and `bf16` does not, so even the *target*
//! side does not agree across widths; and reassociation error is bounded by the
//! significand, which is 8 bits at BF16 against binary32's 24. A row copied from
//! the `f32` table would therefore be a legality claim about another width made
//! without evidence. Each registered family below states the derivation that
//! placed it, and a width with no registered capability resolves to no fusion
//! legality at all rather than to the nearest neighbour's.
//!
//! Scope boundary: this authority derives legality of one candidate. It selects
//! no cover, chooses no physical implementation, schedules nothing, and costs
//! nothing.
//!
//! Nothing here is a compiler API: `fusion_legality` is a private module
//! carrying no `pub` item and no re-export, so this is a crate-internal draft
//! vocabulary. The draft discipline still holds — every shape below is
//! provisional and carries no compatibility story — and the acceptance a public
//! boundary owes Tom is owed at the point any of it is first exported, not here.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::numerics::registered_arithmetic_value_type;
use tiler_ir::schedule::{ArithmeticType, NumericalPermission};
use tiler_ir::semantic::{
    CANONICAL_BF16_ARITHMETIC_NAN_BITS, CANONICAL_F32_ARITHMETIC_NAN_BITS, FrozenSemanticRegistry,
    OpKey, OperationEffect, ProviderIdentity, SemanticProgram, add_bf16_op, add_f32_op,
    broadcast_f32_op, concatenate_f32_op, constant_bf16_op, constant_f32_op, multiply_bf16_op,
    multiply_f32_op, reindex_f32_op, rms_norm_f32_op, silu_f32_op, slice_f32_op, softmax_f32_op,
    strict_serial_sum_f32_op, tensor_contraction_f32_op,
};

use crate::region::{
    MemberOperationFacts, RegionCandidate, RegionContentIdentity, RegionError,
    RegionFormationOutcome, RegionGraph, RegionOccurrenceIdentity, SemanticMemberId,
    verify_candidate,
};
use crate::request::{DeterministicBudgets, StrictF32NumericalContract};

/// Canonical domain-separation tag for reusable fusion-legality content.
const CONTENT_IDENTITY_TAG: &[u8] = b"tiler.compiler.fusion-legality-content.v1\0";
/// Canonical domain-separation tag for one fusion-legality occurrence binding.
const OCCURRENCE_IDENTITY_TAG: &[u8] = b"tiler.compiler.fusion-legality-occurrence.v1\0";
/// Namespace of the governed compiler-owned fusion-capability provider.
const GOVERNED_PROVIDER_NAMESPACE: &str = "tiler";
/// Name of the governed compiler-owned fusion-capability provider.
///
/// **Renamed from `fusion-strict-f32` when the table stopped being one width's.**
/// The old name was a true statement while every registered family was `f32`-keyed
/// and every obligation was discharged against the binary32 constants. It became
/// false the moment this provider declared a role for a BF16 family: an
/// explain record attributing a BF16 region's legality to an authority named
/// `strict-f32` tells a reader the opposite of what happened, and a proof
/// identity binding that name asserts a width the proof is not about.
///
/// The name is part of [`ProviderIdentity`], so this moves every fusion-legality
/// proof identity and every explain record attributed to it. Both are
/// compilation-local — a proof is replayed by equality inside one compilation
/// and is never published — so no artifact identity, cache subject, or golden
/// depends on it.
const GOVERNED_PROVIDER_NAME: &str = "fusion-numerical-capabilities";
/// Output-affecting revision of the governed fusion-capability provider.
const GOVERNED_PROVIDER_REVISION: u32 = 1;

/// The class of evidence that discharged, rejected, or failed to establish one
/// obligation.
///
/// The five classes are deliberately distinct maturity claims and are never
/// collapsed: a sound proof is not empirical, and an unknown is not a normative
/// guarantee. The bounded strict-`f32` profile constructs a subset; the
/// remaining classes are reserved so that a future obligation discharged by
/// finite enumeration or measurement declares itself honestly rather than
/// masquerading as a proof.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(
    dead_code,
    reason = "reserved evidence classes; the bounded profile discharges every obligation by checked invariant, and exhaustive-finite and empirical evidence stay distinct classes a later profile produces"
)]
pub(crate) enum FusionEvidenceClass {
    /// The reached operation's normative definition guarantees the property.
    NormativeGuarantee,
    /// Soundly derived from the verified region structure and numerical policy.
    SoundProof,
    /// Established by exhaustively enumerating a finite domain.
    ///
    /// Reserved: no bounded strict-`f32` obligation discharges this way yet, but
    /// the class is kept distinct so a future finite-domain proof declares
    /// itself honestly rather than masquerading as a sound proof.
    ExhaustiveFinite,
    /// Established only by empirical measurement under a named profile.
    ///
    /// Reserved: kept distinct so a future measured qualification cannot be
    /// mistaken for a proof or a normative guarantee.
    Empirical,
    /// The property could not be established in this bounded profile.
    Unknown,
}

#[allow(
    dead_code,
    reason = "reserved evidence classes; the bounded profile discharges every obligation by checked invariant, and exhaustive-finite and empirical evidence stay distinct classes a later profile produces"
)]
impl FusionEvidenceClass {
    /// Returns the stable identity tag shared by ordering and encoding.
    const fn tag(self) -> u8 {
        match self {
            Self::NormativeGuarantee => 1,
            Self::SoundProof => 2,
            Self::ExhaustiveFinite => 3,
            Self::Empirical => 4,
            Self::Unknown => 5,
        }
    }

    /// Returns the stable presentation name of the evidence class.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::NormativeGuarantee => "normative-guarantee",
            Self::SoundProof => "sound-proof",
            Self::ExhaustiveFinite => "exhaustive-finite",
            Self::Empirical => "empirical",
            Self::Unknown => "unknown",
        }
    }
}

/// The fusion role of one operation family, resolved from its capability.
///
/// The role is the per-operation capability the derivation consults instead of
/// recognizing a whole-graph shape. It fails closed: an operation family with no
/// registered role yields no fusion legality at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FusionOperationRole {
    /// A constant or boundary read: it contributes a value and no reordering,
    /// conversion, or reduction obligation of its own.
    ValueSource,
    /// A separate-rounding elementwise arithmetic operation.
    ElementwiseArithmetic,
    /// A strict lexicographic left-fold reduction with a defined identity.
    ///
    /// The *whole* of the operation is the fold: `tiler::strict-serial-sum-f32@1`
    /// applies no per-point rounding before or after it. That is what makes
    /// [`Self::PrologueCarryingOrderedReduction`] a separate role rather than the
    /// same one — see there.
    OrderedReduction,
    /// An ordered fold wrapped in per-point arithmetic on one or both sides.
    ///
    /// `tiler::rms-norm-f32@1` squares each contributor before the fold and then
    /// divides, adds `eps`, takes a reciprocal square root, and applies two
    /// multiplies after it. `tiler::tensor-contraction-f32@1` carries the
    /// prologue alone: each contributor is the separately rounded binary32
    /// product of two operand elements, and the fold over the canonically
    /// ordered contracted index space is followed by nothing. Every reduction
    /// obligation [`Self::OrderedReduction`] carries is carried here too — the
    /// embedded fold is the same strict left fold over the same canonical
    /// contributor sequence, so [`Self::is_reduction`] answers `true` for both
    /// families and the four reduction obligations are derived identically.
    ///
    /// **Why the prologue's operand arity does not split the role.** A
    /// contraction assembles each contributor from two operands read at
    /// coordinates that drop different free iteration coordinates, where a
    /// normalization reads one operand. That difference lives in the access
    /// relation, and this authority discharges no access obligation: every
    /// obligation here is about a rounding, a fold order, a conversion boundary,
    /// or an exceptional value, and a per-contributor product of two loaded
    /// values commits exactly the same separate rounding a per-contributor
    /// square does. The contributor domain is still *one* domain folded in one
    /// declared order, so one permission still answers for the whole operation,
    /// which is this role's actual contract rather than the operand count its
    /// first holder happened to have.
    ///
    /// **Why it is nevertheless not the same role as [`Self::OrderedReduction`].**
    /// That role's contract is that the operation *is* a fold: nothing else about
    /// it rounds. Classifying a normalization as one would state that fusing it
    /// can only move a fold order, when it also carries seven per-point roundings
    /// whose exceptional-value behaviour a fused realization must preserve;
    /// classifying a contraction as one would state the same falsehood about its
    /// `K` products. The distinction is what keeps
    /// `is_exact_governed_same_family_pointwise` closed: its match is exhaustive,
    /// so a role added later has to decide whether it introduces a
    /// multiply-plus-add adjacency rather than inherit an answer.
    ///
    /// It is counted under the reduction total rather than given a count of its
    /// own, because [`FusionRegionStructure`]'s four role counts sum to the
    /// member count and adding a fifth field would move every previously
    /// encodable region's content identity.
    PrologueCarryingOrderedReduction,
    /// Two folds over one contributor sequence, carrying *different* order
    /// obligations.
    ///
    /// `tiler::softmax-f32@1` reduces each row twice: an order-insensitive
    /// `Maximum` whose result shifts every contributor, and an order-sensitive
    /// sum of the shifted exponentials. Every reduction obligation
    /// [`Self::OrderedReduction`] carries is carried here too — the second fold
    /// is the same strict left fold over the same canonical contributor sequence
    /// — so [`Self::is_reduction`] answers `true` and the four reduction
    /// obligations are derived identically.
    ///
    /// **Why it is nevertheless not [`Self::PrologueCarryingOrderedReduction`].**
    /// That role's contract is that the operation carries *one* fold, wrapped in
    /// per-point arithmetic: one permission therefore answers for the whole
    /// operation. Here it does not. The maximum is associative and commutative on
    /// every binary32 input, so its pass may be reassociated and permuted with no
    /// permission at all, while the sum's pass moves only under the separately
    /// resolved ones. Classifying a softmax as prologue-carrying would state that
    /// a permission covering the sum describes the whole operation, which is
    /// wrong in one direction, and that a permission covering the maximum
    /// licenses the sum, which is wrong in the other.
    ///
    /// It is counted under the reduction total for the same reason the
    /// prologue-carrying role is: [`FusionRegionStructure`]'s four counts sum to
    /// the member count, and a fifth field would move the content identity of
    /// every region this vocabulary can already encode.
    ExtremumShiftedOrderedReduction,
    /// A pure coordinate relation that computes no value.
    ///
    /// A reindex or a broadcast rearranges or replicates which coordinate reads
    /// which element and returns those elements unchanged. It is deliberately
    /// *not* a [`Self::ValueSource`]: a value source contributes a value the
    /// region did not otherwise have, while a coordinate relation contributes an
    /// access map over a value the region already has. Collapsing the two would
    /// make the structural counts below say a region has one more independent
    /// value than it does.
    ///
    /// The role carries no obligation of its own, and that is derived rather
    /// than deferred. Every numerical obligation this authority discharges is
    /// about rounding, order, or an exceptional value produced by arithmetic;
    /// a coordinate relation performs none, so fusing one neither adds nor
    /// removes a rounding, a fold order, a conversion boundary, or a NaN
    /// canonicalization. The one property it does introduce — a broadcast's
    /// aliasing reads — is an index-verifier concern, where the alias contract
    /// already admits aliasing reads and constrains writes, and it is not a
    /// numerical-contract obligation.
    CoordinateRelation,
}

impl FusionOperationRole {
    const fn is_arithmetic(self) -> bool {
        matches!(self, Self::ElementwiseArithmetic)
    }

    const fn is_reduction(self) -> bool {
        matches!(
            self,
            Self::OrderedReduction
                | Self::PrologueCarryingOrderedReduction
                | Self::ExtremumShiftedOrderedReduction
        )
    }

    const fn is_value_source(self) -> bool {
        matches!(self, Self::ValueSource)
    }

    const fn is_coordinate_relation(self) -> bool {
        matches!(self, Self::CoordinateRelation)
    }
}

/// A compiler-owned registry of per-operation fusion numerical capabilities.
///
/// It maps an operation family key to the fusion role the governed provider
/// declares for it. Resolution is a checked lookup, not a graph-shape match, so
/// coverage grows one operation at a time and any unregistered family fails
/// closed to [`FusionLegality::Unknown`].
#[derive(Clone, Debug)]
pub(crate) struct FusionNumericalCapabilities {
    provider: ProviderIdentity,
    revision: u32,
    roles: BTreeMap<OpKey, FusionOperationRole>,
}

impl FusionNumericalCapabilities {
    /// Builds the governed fusion-capability registry.
    ///
    /// The table below is the complete set of families the governed provider
    /// declares a role for; every registered family absent from it resolves to
    /// no fusion legality at all. Each entry states the derivation that placed
    /// it, because resolution is a checked lookup and a role is therefore
    /// decided per family rather than inferred from a neighbour.
    ///
    /// **The table spans two widths and the entries are not each other's.** The
    /// three BF16 families at the end were decided from `tiler-ir`'s own
    /// declared record for them — `arithmetic_bf16_facts` and
    /// `constant_bf16_facts` — and not transferred from the `f32` neighbour that
    /// shares their shape; each states which of its answers is derived from that
    /// record and which is vacuous. What is deliberately **absent** is as
    /// load-bearing as what is present: no BF16 reduction, prologue-carrying
    /// reduction, extremum-shifted reduction, or coordinate relation appears,
    /// because `tiler-ir` registers no such family at that width, and a row for
    /// an operation that does not exist would be a legality claim about nothing.
    #[must_use]
    pub(crate) fn governed() -> Self {
        let provider = ProviderIdentity::new(
            GOVERNED_PROVIDER_NAMESPACE,
            GOVERNED_PROVIDER_NAME,
            GOVERNED_PROVIDER_REVISION,
        )
        .expect("the governed fusion-capability provider identity is valid");
        let mut roles = BTreeMap::new();
        roles.insert(constant_f32_op(), FusionOperationRole::ValueSource);
        roles.insert(
            multiply_f32_op(),
            FusionOperationRole::ElementwiseArithmetic,
        );
        roles.insert(add_f32_op(), FusionOperationRole::ElementwiseArithmetic);
        // The activation is elementwise arithmetic like the multiply and the add,
        // and for the same reason: it applies its own separate roundings per point
        // and introduces no reduction, no conversion boundary, and no order
        // contract. Without a role it would carry no fusion legality at all, which
        // would make every occurrence an optimization boundary in the middle of a
        // program that evaluates it 28 times per forward pass.
        //
        // The role says nothing about its *accuracy*: fusing two regions neither
        // adds nor removes a rounding, so the resolved accuracy contract of the
        // subordinate exponential is unchanged by fusion and is not this
        // authority's to assess.
        roles.insert(silu_f32_op(), FusionOperationRole::ElementwiseArithmetic);
        roles.insert(
            strict_serial_sum_f32_op(),
            FusionOperationRole::OrderedReduction,
        );
        // The normalization embeds a strict ordered fold over an elementwise
        // squaring prologue, which is the shape `OrderedReduction` was defined
        // for — but that role is held by the bare serial sum, and the L3′
        // derivation records that a family without one "resolves to no fusion
        // legality at all". Registering the prologue-carrying role is what gives
        // the 113 occurrences per forward pass any legality to reason about.
        //
        // The role says nothing about the operation's *accuracy*: fusing two
        // regions neither adds nor removes a rounding, so the resolved contract
        // of the subordinate reciprocal square root is unchanged by fusion and is
        // not this authority's to assess.
        roles.insert(
            rms_norm_f32_op(),
            FusionOperationRole::PrologueCarryingOrderedReduction,
        );
        // The softmax embeds *two* folds over one contributor sequence, and the
        // L3′ derivation records that the maximum reduction "resolves to no
        // fusion legality at all" without a role. Registering the
        // extremum-shifted role is what gives the 28 occurrences per forward
        // pass any legality to reason about — and what keeps the two passes'
        // different order obligations from collapsing into one answer.
        //
        // The role says nothing about the operation's *accuracy*: fusing two
        // regions neither adds nor removes a rounding, so the resolved contract
        // of the subordinate exponential is unchanged by fusion and is not this
        // authority's to assess.
        roles.insert(
            softmax_f32_op(),
            FusionOperationRole::ExtremumShiftedOrderedReduction,
        );
        roles.insert(reindex_f32_op(), FusionOperationRole::CoordinateRelation);
        roles.insert(broadcast_f32_op(), FusionOperationRole::CoordinateRelation);
        // The sequence-extension concatenate is a third coordinate relation, and
        // the classification is derived from what the derivation below actually
        // asks rather than from the family's name.
        //
        // *Why a role at all.* Without one, `derive_member` returns `Ok(None)`
        // and every region holding a concatenate resolves to no legality at all
        // — a fail-closed refusal with no premise behind it, because the family
        // is pure, dtype-homogeneous by construction, and reduction-free, so
        // every obligation this authority can ask already has an answer.
        //
        // *Why not `ValueSource`.* Every element of the result is an element of
        // an operand, unchanged. The role doc's distinction decides it: a value
        // source contributes a value the region did not otherwise have, while a
        // coordinate relation contributes an access map over a value the region
        // already has. Counting a concatenate as a value source would make
        // `region_structure` report one more independent value than the region
        // holds.
        //
        // *Why not a seventh role.* A new variant must either derive an
        // obligation differently or fall outside the four structural buckets,
        // and this family does neither: purity is declared, homogeneity is
        // guaranteed by an inferencer that refuses a non-`f32` operand at
        // construction, the join performs no arithmetic and so reaches no
        // result boundary at which a NaN canonicalization could be added or
        // removed, and `is_reduction` is false, so all four reduction
        // obligations discharge exactly as they do for a reindex. A fifth
        // `FusionRegionStructure` count would move the content identity of
        // every region this vocabulary can already encode, paid for no
        // derivational difference.
        //
        // *Why the two-through-eight arity is not an obstacle.* Nothing in the
        // derivation is arity-sensitive: `region_structure` counts members by
        // role and reads `boundary_inputs` from the candidate rather than from
        // any operation's arity, and `member_is_homogeneous` iterates whatever
        // operand encodings the member has.
        roles.insert(
            concatenate_f32_op(),
            FusionOperationRole::CoordinateRelation,
        );
        // The sub-tensor selection is a fourth coordinate relation, and the
        // classification is derived from what the derivation below actually asks
        // rather than from the one property that makes this family unlike the
        // other three.
        //
        // *Why a role at all.* Without one, `derive_member` returns `Ok(None)`
        // and every region holding a selection resolves to no legality at all.
        // That refusal is reachable rather than hypothetical: region formation
        // holds no operation allowlist — `RegionGraph`'s construction admits
        // every occurrence whose key the registry defines and reads only the
        // definition's effect from it — so a program stating a selection does
        // form candidates containing one. And the refusal has no premise behind
        // it: the family declares `OperationEffect::Pure`, its inferencer
        // refuses any operand whose resolved type is not `tiler::f32@1` and
        // builds an `f32` result, its normative definition guarantees every
        // result element is an operand element unchanged with every exceptional
        // payload arriving as it left, and it carries no fold.
        //
        // *Why not `ValueSource`.* `FusionOperationRole::CoordinateRelation`'s
        // own distinction decides it: a value source contributes a value the
        // region did not otherwise have, while a coordinate relation contributes
        // an access map over a value the region already has. Every element of a
        // selection's result is an element of its single operand — which this
        // family states in canonical attribute bytes rather than only in
        // normative prose, `SLICE_FACT_VALUE_BEHAVIOUR` reading
        // "none-every-result-element-is-an-operand-element-unchanged" and
        // `SLICE_FACT_MAPPING_CLASS` reading
        // "total-over-the-result-domain-and-injective-not-surjective-into-the-operand-domain".
        // Counting it as a value source would make `region_structure` report one
        // more independent value than the region holds.
        //
        // *Why not a seventh role, and this is where the family's own difference
        // is answered.* A new variant must derive some obligation differently or
        // fall outside the four structural buckets. The property that separates a
        // selection from a reindex is that its map is *non-surjective*: at least
        // one operand element is never read. That derives nothing differently
        // here, because no obligation below reads a mapping class, an operand
        // count, or how much of a source an access covers — every one is about
        // rounding, order, purity, dtype homogeneity, or an exceptional value
        // produced by arithmetic, and a selection produces none. Non-surjectivity
        // is a semantic admission rule, which is where the family keeps it. A
        // fifth `FusionRegionStructure` count would then move the content
        // identity of every region this vocabulary can already encode, paid for
        // no derivational difference.
        roles.insert(slice_f32_op(), FusionOperationRole::CoordinateRelation);
        // The tensor contraction widens the prologue-carrying reduction rather
        // than taking a role of its own, and the classification is derived from
        // what the derivation below actually asks rather than from the family's
        // arity.
        //
        // *Why a role at all.* Without one, `derive_member` returns `Ok(None)`
        // and every region holding a contraction beside another member resolves
        // to no legality at all. The whole-program shape never asks — planning
        // skips `derive_fusion_legality` below two members and the recognized
        // contraction is one operation — so the refusal only fires for the
        // fused prologue and epilogue shapes Milestone 6 names, which is exactly
        // where it would be a fail-closed refusal with no premise behind it:
        // the family is `OperationEffect::Pure`, `tiler::f32@1` throughout, and
        // its fold is the same strict left fold the serial sum declares.
        //
        // *Why not `OrderedReduction`.* That role's contract is that the whole
        // of the operation is the fold. A contraction rounds `K` products before
        // the fold ever runs, and its declared computation precision says so:
        // "binary32-operands-and-binary32-products". Classifying it there would
        // state that fusing it can only move a fold order.
        //
        // *Why not `ExtremumShiftedOrderedReduction`.* That role exists for two
        // folds over one contributor sequence carrying *different* order
        // obligations. A contraction carries one fold and nothing in it is
        // order-insensitive, so that role would claim a freedom — the softmax
        // maximum's permission-free reassociation — this family's own
        // `reassociation-permitted: false` withholds.
        //
        // *Why not `ElementwiseArithmetic`.* `is_reduction` would answer false
        // and all four reduction obligations would discharge vacuously as a
        // structural fact, so a region containing a contraction would derive
        // `Legal` under a reassociating contract that grants exactly the
        // regrouping this family forbids. That is a silently wrong accept, which
        // is the one outcome this authority may not produce.
        //
        // *Why not a seventh role.* A new variant must derive some obligation
        // differently or fall outside the four structural buckets. This one does
        // neither. The role is consulted in exactly six places — the four
        // `is_*` predicates, the match in
        // `is_exact_governed_same_family_pointwise`, and `region_structure`'s
        // counts — and a contraction and a normalization produce the same answer
        // from all six: neither is arithmetic, a value source, or a coordinate
        // relation; both are reductions; both fall to the closed proof's
        // disqualifying arm; and both count under `reductions`. A variant whose
        // only content is a name would be a distinction with no consumer.
        //
        // *The nine obligations, decided rather than inherited.* Capabilities
        // resolve by this entry. Referential transparency holds because the
        // definition declares `Pure`, and `derive_member` errors rather than
        // guesses when the graph disagrees. Conversion-boundary preservation
        // holds because every operand, product, accumulator, and result is
        // `tiler::f32@1` — the family's declared conversion behaviour is "none"
        // — and `member_is_homogeneous` iterates the member's actual operand
        // encodings, so the two-operand arity is not an obstacle. Exceptional
        // values: the family installs the canonical arithmetic-NaN payload after
        // every combine and at the result boundary, which is a per-combine
        // rewrite the fused body must still apply and which fusion neither adds
        // nor removes, and the obligation's own test is the contract's payload
        // against the governed one. Identity and empty domain are defined by
        // refusal — the fold is unseeded and a zero contracted extent is
        // rejected at construction — which is a stated behaviour rather than an
        // absent one. Contributor order is declared ascending-lexicographic over
        // the canonically ordered contracted index space. Reassociation resolves
        // against the caller's permission exactly as it does for every other
        // reduction, so a permitting contract leaves it unknown rather than
        // discharged. Operand permutation is discharged from the role: the
        // ordered left fold fixes the contributor sequence, and the family's own
        // `permutation-permitted: false` agrees. The ninth — ADR 0015's
        // arithmetic contraction — is decided in
        // `is_exact_governed_same_family_pointwise` below, where this family is
        // the case that closure was written for.
        roles.insert(
            tensor_contraction_f32_op(),
            FusionOperationRole::PrologueCarryingOrderedReduction,
        );
        // The three BF16 families, decided from their own declared record.
        //
        // *Why a role at all.* Without one, `derive_member` returns `Ok(None)`
        // and every BF16 region covering two or more occurrences resolves to no
        // legality at all, so every cover placing it is skipped and the
        // compilation refuses `NoFeasiblePlan`. That refusal had a premise while
        // nothing had decided the width's obligations; deciding them is what
        // this entry is.
        //
        // *Why the `f32` rows could not simply be copied.* Two of the nine
        // obligations are width-sensitive in ways this vocabulary can state.
        // `ReductionReassociation` is bounded by the significand — 8 bits at
        // BF16 against binary32's 24 — so a regrouping permitted under one
        // width's error budget is not the same permission at the other. And
        // finding 28 of `docs/research/apple-targets/numerical-behaviour.md`
        // measures a *target* whose contraction behaviour differs between `f16`
        // and `bf16` under `safe` with `-ffp-contract=fast`. Neither refutes the
        // rows below, and each is why they are derived rather than transferred;
        // the derivations are per obligation and are recorded on the two entries
        // they belong to.
        //
        // *The constant.* A value source for the reason
        // [`FusionOperationRole::ValueSource`] states: it contributes a value the
        // region did not otherwise have and no reordering, conversion, or
        // reduction obligation of its own. `constant_bf16_facts` declares its
        // rounding "none-the-declared-payload-is-already-the-exact-bf16-encoding"
        // and its NaN behaviour "preserved-exactly-the-declared-payload-is-not-
        // canonicalized", so the family performs no arithmetic at all and fusing
        // an occurrence of it neither adds nor removes a rounding. Not a
        // coordinate relation: it computes a value rather than an access map over
        // one the region already holds.
        roles.insert(constant_bf16_op(), FusionOperationRole::ValueSource);
        // *The two arithmetics.* Elementwise arithmetic, and every one of the
        // nine obligations is decided from `arithmetic_bf16_facts` rather than
        // inherited:
        //
        // - **Referential transparency.** `OperationEffect::Pure` is declared on
        //   both registrations, and `derive_member` errors rather than guesses
        //   when the derived graph purity disagrees with it.
        // - **Conversion-boundary preservation.** Computation, accumulator,
        //   intermediate-materialization, and result types all resolve to
        //   `tiler::bf16@1`, and the family's own inferencer refuses a
        //   mixed-precision operand pair and an implicit promotion by name at
        //   application time. So a BF16 member is homogeneous in the *region's*
        //   width, which `derive_fusion_legality` now derives from the contract
        //   rather than from the binary32 constant.
        // - **Arithmetic contraction.** `BF16_FACT_ARITHMETIC_CONTRACTION_PERMITTED`
        //   and `BF16_FACT_FUSED_MULTIPLY_ADD_PERMITTED` are both `false`, and
        //   `docs/research/apple-targets/numerical-behaviour.md`'s boundaries
        //   record that MSL provides no `bfloat` overload of `fma` at all, so
        //   there is no fused primitive at this width to contract into. The
        //   closed proof below is extended to these keys on the same argument it
        //   admits the `f32` add and multiply on — an all-add or all-multiply
        //   region has no multiply-plus-add adjacency — and the contract's own
        //   `Forbidden` resolution discharges the rest by normative guarantee.
        //   **Finding 28 is not a counterexample to that**: it measures what a
        //   target's compiler does under a given flag row, which is the target
        //   profile's authority to declare per subject and is checked before
        //   planning; this obligation asks only whether *fusing* changes what the
        //   contract authorizes, and fusing adds no adjacency the unfused form
        //   lacks.
        // - **Exceptional values.** `BF16_FACT_NAN_BEHAVIOUR` is
        //   "quiet-nan-propagates-and-every-arithmetic-nan-result-is-canonicalized"
        //   and `BF16_FACT_CANONICAL_NAN_BITS` is
        //   `CANONICAL_BF16_ARITHMETIC_NAN_BITS`, so the per-result
        //   canonicalization a fused body must still apply is defined at this
        //   width and is compared against the contract's own payload below.
        // - **The four reduction obligations.** Vacuous, and vacuous by
        //   *derivation* rather than by omission: `is_reduction` is false for this
        //   role, and no BF16 family carrying a fold is registered anywhere in
        //   `tiler-ir`, so there is no BF16 contributor sequence for an identity,
        //   an empty domain, an order, a regrouping, or a permutation to be about.
        //   This is where the significand argument would bite and it has nothing
        //   to bite on: `BF16_FACT_REASSOCIATION_PERMITTED` is `false`, the family
        //   declares no algebraic capability, and the reassociation question at
        //   this width therefore stays open at the *operation vocabulary* rather
        //   than being answered here.
        //
        // Not a coordinate relation and not a value source: both perform
        // per-point arithmetic with their own separate rounding, declared as
        // "bf16-round-to-nearest-ties-to-even-at-every-observable-materialization".
        roles.insert(
            multiply_bf16_op(),
            FusionOperationRole::ElementwiseArithmetic,
        );
        roles.insert(add_bf16_op(), FusionOperationRole::ElementwiseArithmetic);
        Self {
            provider,
            revision: GOVERNED_PROVIDER_REVISION,
            roles,
        }
    }

    /// Returns the provider that declared these capabilities.
    #[must_use]
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the output-affecting revision of the capability source.
    #[must_use]
    pub(crate) const fn revision(&self) -> u32 {
        self.revision
    }

    fn classify(&self, key: &OpKey) -> Option<FusionOperationRole> {
        self.roles.get(key).copied()
    }

    /// Builds the governed registry without one operation family's capability.
    ///
    /// This exercises the fail-closed path where a member operation has no
    /// registered fusion capability.
    #[cfg(test)]
    fn governed_without(excluded: &OpKey) -> Self {
        let mut capabilities = Self::governed();
        capabilities.roles.remove(excluded);
        capabilities
    }
}

/// One numerical, effect, or materialization obligation a fused realization must
/// satisfy.
///
/// The obligations are per-operation-derived rather than a fixed proof label.
/// Reassociation and operand permutation are separate obligations because a
/// permission or capability for one is not evidence for the other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FusionObligation {
    /// Every member operation has a resolved fusion capability.
    OperationCapabilitiesResolved,
    /// Every member operation is referentially transparent.
    ReferentialTransparency,
    /// No observable conversion/materialization boundary is silently removed.
    ConversionBoundaryPreservation,
    /// The separate-rounding contract is preserved; contraction stays authorized.
    ArithmeticContraction,
    /// NaN canonicalization, signed zero, and subnormal handling are preserved.
    ExceptionalValues,
    /// Each reduction's identity and empty-domain result are defined.
    ReductionIdentityAndEmptyDomain,
    /// Each reduction's contributor order satisfies the semantic order contract.
    ReductionContributorOrder,
    /// Reassociation legality is established independently of permutation.
    ReductionReassociation,
    /// Operand-permutation legality is established independently of reassociation.
    ReductionOperandPermutation,
}

impl FusionObligation {
    /// Returns the stable rule key of this obligation.
    pub(crate) const fn rule(self) -> &'static str {
        match self {
            Self::OperationCapabilitiesResolved => "fusion.capabilities-resolved",
            Self::ReferentialTransparency => "fusion.referential-transparency",
            Self::ConversionBoundaryPreservation => "fusion.conversion-boundary",
            Self::ArithmeticContraction => "fusion.arithmetic-contraction",
            Self::ExceptionalValues => "fusion.exceptional-values",
            Self::ReductionIdentityAndEmptyDomain => "fusion.reduction-identity-empty-domain",
            Self::ReductionContributorOrder => "fusion.reduction-contributor-order",
            Self::ReductionReassociation => "fusion.reduction-reassociation",
            Self::ReductionOperandPermutation => "fusion.reduction-operand-permutation",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::OperationCapabilitiesResolved => 1,
            Self::ReferentialTransparency => 2,
            Self::ConversionBoundaryPreservation => 3,
            Self::ArithmeticContraction => 4,
            Self::ExceptionalValues => 5,
            Self::ReductionIdentityAndEmptyDomain => 6,
            Self::ReductionContributorOrder => 7,
            Self::ReductionReassociation => 8,
            Self::ReductionOperandPermutation => 9,
        }
    }
}

/// The assessment of one derived obligation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObligationAssessment {
    /// The obligation holds for a fused realization.
    Discharged,
    /// A fused realization is proved to violate the obligation.
    Rejected {
        /// Stable reason code.
        reason: &'static str,
    },
    /// The obligation cannot be established in this bounded profile.
    Unknown {
        /// Stable reason code.
        reason: &'static str,
    },
}

impl ObligationAssessment {
    const fn tag(self) -> u8 {
        match self {
            Self::Discharged => 1,
            Self::Rejected { .. } => 2,
            Self::Unknown { .. } => 3,
        }
    }

    const fn reason(self) -> Option<&'static str> {
        match self {
            Self::Discharged => None,
            Self::Rejected { reason } | Self::Unknown { reason } => Some(reason),
        }
    }
}

/// One obligation, its assessment, and the class of evidence behind it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DerivedObligation {
    obligation: FusionObligation,
    assessment: ObligationAssessment,
    evidence: FusionEvidenceClass,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl DerivedObligation {
    /// Returns the obligation this record assesses.
    pub(crate) const fn obligation(&self) -> FusionObligation {
        self.obligation
    }

    /// Returns the obligation's assessment.
    pub(crate) const fn assessment(&self) -> ObligationAssessment {
        self.assessment
    }

    /// Returns the class of evidence behind the assessment.
    pub(crate) const fn evidence(&self) -> FusionEvidenceClass {
        self.evidence
    }

    fn discharged(obligation: FusionObligation, evidence: FusionEvidenceClass) -> Self {
        Self {
            obligation,
            assessment: ObligationAssessment::Discharged,
            evidence,
        }
    }

    fn rejected(obligation: FusionObligation, reason: &'static str) -> Self {
        Self {
            obligation,
            assessment: ObligationAssessment::Rejected { reason },
            evidence: FusionEvidenceClass::SoundProof,
        }
    }

    fn unknown(obligation: FusionObligation, reason: &'static str) -> Self {
        Self {
            obligation,
            assessment: ObligationAssessment::Unknown { reason },
            evidence: FusionEvidenceClass::Unknown,
        }
    }
}

/// Site-independent structural counts of one region's derived computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FusionRegionStructure {
    /// Number of member operations.
    members: u32,
    /// Number of value-source members (constants and boundary reads).
    value_sources: u32,
    /// Number of elementwise arithmetic members.
    arithmetic: u32,
    /// Number of members carrying an ordered reduction.
    ///
    /// Every reduction role counts here — the bare fold, the prologue-carrying
    /// one, and the extremum-shifted one — so that the four counts still sum to
    /// `members`. Distinguishing them would need a fifth field, which would move
    /// the content identity of every region this vocabulary can already encode.
    reductions: u32,
    /// Number of pure coordinate-relation members.
    ///
    /// Counted separately rather than folded into any of the three above, so
    /// that the four role counts sum to `members`. A count that did not add up
    /// would make a region containing an uncounted role structurally
    /// indistinguishable from one without it.
    coordinate_relations: u32,
    /// Number of boundary input values.
    boundary_inputs: u32,
    /// Number of retained boundary outputs.
    retained_outputs: u32,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionRegionStructure {
    /// Returns the number of member operations.
    pub(crate) const fn member_count(&self) -> u32 {
        self.members
    }

    /// Returns the number of ordered-reduction members.
    pub(crate) const fn reduction_count(&self) -> u32 {
        self.reductions
    }

    fn encode(&self, output: &mut Vec<u8>) {
        for field in [
            self.members,
            self.value_sources,
            self.arithmetic,
            self.reductions,
            self.coordinate_relations,
            self.boundary_inputs,
            self.retained_outputs,
        ] {
            output.extend_from_slice(&field.to_be_bytes());
        }
    }
}

/// Collision-free identity of reusable fusion-legality content.
///
/// Two occurrences of the same region content, discharged under the same
/// numerical contract, share these bytes. The graph site, selected provider,
/// and reached admission provenance are deliberately absent.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct FusionLegalityContentIdentity(Vec<u8>);

impl FusionLegalityContentIdentity {
    /// Returns the canonical content bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Collision-free identity of one fusion-legality occurrence binding.
///
/// This is reusable content plus the exact graph site, the reached semantic
/// definitions, the selected provider, and the ordered value bindings.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct FusionLegalityIdentity(Vec<u8>);

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionLegalityIdentity {
    /// Returns the canonical occurrence-binding bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Reusable, site- and provider-independent fusion-legality content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionLegalityContent {
    region_content: RegionContentIdentity,
    numerical_contract_key: &'static str,
    structure: FusionRegionStructure,
    obligations: Vec<DerivedObligation>,
    identity: FusionLegalityContentIdentity,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionLegalityContent {
    /// Returns the canonical region-content identity this legality is over.
    pub(crate) const fn region_content(&self) -> &RegionContentIdentity {
        &self.region_content
    }

    /// Returns the numerical-contract key the obligations were discharged under.
    pub(crate) const fn numerical_contract_key(&self) -> &'static str {
        self.numerical_contract_key
    }

    /// Returns the site-independent structural counts.
    pub(crate) const fn structure(&self) -> &FusionRegionStructure {
        &self.structure
    }

    /// Returns the ordered discharged obligations with their evidence classes.
    pub(crate) fn obligations(&self) -> &[DerivedObligation] {
        &self.obligations
    }

    /// Returns the reusable content identity.
    pub(crate) const fn identity(&self) -> &FusionLegalityContentIdentity {
        &self.identity
    }
}

/// One reached semantic definition an occurrence binds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReachedDefinition {
    operation: OpKey,
    normative_definition: String,
    effect_tag: u8,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl ReachedDefinition {
    /// Returns the reached operation family key.
    pub(crate) const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Returns the reached normative-definition reference.
    pub(crate) fn normative_definition(&self) -> &str {
        &self.normative_definition
    }
}

/// One retained boundary output bound to its exact producer occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RetainedOutputBinding {
    value: u32,
    producer: u32,
    result_position: u32,
    named_result: bool,
    external_consumers: bool,
}

/// The ordered value/access mapping of one region occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionValueBindings {
    boundary_inputs: Vec<u32>,
    retained_outputs: Vec<RetainedOutputBinding>,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionValueBindings {
    /// Returns the graph-local boundary input value ordinals.
    pub(crate) fn boundary_inputs(&self) -> &[u32] {
        &self.boundary_inputs
    }

    /// Returns the ordered retained-output bindings.
    pub(crate) fn retained_outputs(&self) -> &[RetainedOutputBinding] {
        &self.retained_outputs
    }
}

/// Replayable evidence that one region occurrence fuses legally.
///
/// It binds reusable [`FusionLegalityContent`] to the exact occurrence: the
/// region-occurrence identity, the reached semantic definitions, the selected
/// fusion-capability provider, and the ordered value bindings. Holding one is
/// evidence that the derivation discharged every obligation for *this* site, not
/// merely that a candidate exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionLegalityProof {
    content: FusionLegalityContent,
    region_occurrence: RegionOccurrenceIdentity,
    registry_snapshot: Box<[u8]>,
    reached_definitions: Vec<ReachedDefinition>,
    provider: ProviderIdentity,
    provider_revision: u32,
    value_bindings: FusionValueBindings,
    identity: FusionLegalityIdentity,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionLegalityProof {
    /// Returns the reusable, site-independent content.
    pub(crate) const fn content(&self) -> &FusionLegalityContent {
        &self.content
    }

    /// Returns the graph-occurrence identity this proof is bound to.
    pub(crate) const fn region_occurrence(&self) -> &RegionOccurrenceIdentity {
        &self.region_occurrence
    }

    /// Returns the reached semantic definitions in region-local order.
    pub(crate) fn reached_definitions(&self) -> &[ReachedDefinition] {
        &self.reached_definitions
    }

    /// Returns the selected fusion-capability provider.
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the ordered value/access bindings of this occurrence.
    pub(crate) const fn value_bindings(&self) -> &FusionValueBindings {
        &self.value_bindings
    }

    /// Returns the occurrence-binding identity that pins this realization.
    pub(crate) const fn identity(&self) -> &FusionLegalityIdentity {
        &self.identity
    }
}

/// A candidate proved to violate an obligation as a fused realization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionRejection {
    obligation: FusionObligation,
    reason: &'static str,
    region: String,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionRejection {
    /// Returns the violated obligation.
    pub(crate) const fn obligation(&self) -> FusionObligation {
        self.obligation
    }

    /// Returns the stable reason code.
    pub(crate) const fn reason(&self) -> &'static str {
        self.reason
    }
}

impl fmt::Display for FusionRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{}: {} rejected",
            self.obligation.rule(),
            self.reason,
            self.region
        )
    }
}

/// A candidate whose fused legality the bounded profile cannot establish.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionUnknown {
    obligation: FusionObligation,
    reason: &'static str,
    region: String,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionUnknown {
    /// Returns the obligation that could not be established.
    pub(crate) const fn obligation(&self) -> FusionObligation {
        self.obligation
    }

    /// Returns the stable reason code.
    pub(crate) const fn reason(&self) -> &'static str {
        self.reason
    }
}

impl fmt::Display for FusionUnknown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{}: {} unknown",
            self.obligation.rule(),
            self.reason,
            self.region
        )
    }
}

/// The typed outcome of deriving fusion legality for one candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FusionLegality {
    /// The candidate fuses legally, with replayable evidence.
    Legal(Box<FusionLegalityProof>),
    /// A fused realization is proved to violate an obligation.
    Rejected(FusionRejection),
    /// The bounded profile cannot establish a required obligation.
    Unknown(FusionUnknown),
}

/// A fault in fusion-legality derivation, distinct from a legality outcome.
///
/// These are invalid compiler input or output — a forged candidate that fails
/// re-derivation, or a verified program whose operation lacks a registry
/// definition — not the legal `Rejected`/`Unknown` outcomes above.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FusionLegalityError {
    /// The candidate failed re-derivation from its own exact contents.
    Region(RegionError),
    /// The derivation observed invalid compiler state.
    Structure {
        /// Stable rule code.
        rule: &'static str,
    },
}

impl FusionLegalityError {
    /// Returns the stable reason code of the fault.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::Region(error) => error.reason(),
            Self::Structure { rule } => rule,
        }
    }
}

impl fmt::Display for FusionLegalityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Region(error) => error.fmt(formatter),
            Self::Structure { rule } => {
                write!(formatter, "fusion.legality.structure.{rule}")
            }
        }
    }
}

impl Error for FusionLegalityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Region(error) => Some(error),
            Self::Structure { .. } => None,
        }
    }
}

impl From<RegionError> for FusionLegalityError {
    fn from(value: RegionError) -> Self {
        Self::Region(value)
    }
}

/// The complete derivation of one member operation.
struct MemberDerivation {
    role: FusionOperationRole,
    reached: ReachedDefinition,
    pure: bool,
    homogeneous: bool,
}

/// Derives fusion legality for one region candidate.
///
/// The candidate is re-derived from the graph before anything else, so a forged
/// or stale candidate fails closed. Each member operation's fusion capability is
/// then resolved and its obligations discharged against the reached semantic
/// definition and the numerical policy. The result is a legal proof, a typed
/// rejection, or a typed unknown; a hard rejection dominates an unknown so the
/// most certain failure is reported.
///
/// # Errors
///
/// Returns a [`FusionLegalityError`] when the candidate does not re-derive or a
/// member operation lacks a semantic-registry definition. A legality outcome
/// (`Rejected`/`Unknown`) is a successful `Ok`, not an error.
///
/// The region formation is **taken rather than derived**, for the same reason
/// [`crate::cover::enumerate_covers`] takes it: it is a pure function of the
/// program this derivation already observes, and every caller holds one.
/// Building a graph here re-ran `canonical_member_order` over the whole
/// program — a colour refinement quadratic in the operation count — once per
/// candidate. A sampling profile of one compile put that single function above
/// every other in the crate at 10.6% of active self time, and the graph was
/// being built fifteen times per compile to produce fifteen equal values.
pub(crate) fn derive_fusion_legality(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    contract: StrictF32NumericalContract,
    capabilities: &FusionNumericalCapabilities,
    formation: &RegionFormationOutcome,
    candidate: &RegionCandidate,
) -> Result<FusionLegality, FusionLegalityError> {
    let graph = formation.graph();
    verify_candidate(graph, budgets, contract, candidate)?;
    let registry = program.semantic_registry();

    let ordered = ordered_members(graph, candidate)?;
    // The region's own dtype, read from the contract rather than from a binary32
    // constant. A contract states its resolutions for exactly one
    // `ArithmeticType` (ADR 0076 item 6) and `recognized_program_arithmetic`
    // refuses a program whose values are two widths at once, so this is the width
    // every member of a compilable region is stated in — and comparing a member's
    // encodings against binary32's would report every BF16 member as carrying an
    // unproven conversion boundary.
    //
    // A width the governed scalar catalog does not describe is invalid compiler
    // state rather than a legality outcome, and is classed with the other two
    // faults this derivation guards: `registered_arithmetic_value_type` is total
    // over the arithmetic vocabulary — `tiler-ir` pins the vocabulary and the
    // catalog to each other, and
    // `every_governed_arithmetic_resolves_a_region_encoding` re-states the
    // population here — so `None` means those two have drifted apart, not that a
    // caller asked something unanswerable. Failing closed with a named rule keeps
    // the drift loud instead of comparing every member against nothing.
    let Some(region_type) = registered_arithmetic_value_type(contract.arithmetic) else {
        return Err(FusionLegalityError::Structure {
            rule: "ungoverned-region-arithmetic",
        });
    };
    let governed_dtype = region_type.canonical_encoding();
    let governed_dtype = governed_dtype.as_bytes();

    // An unresolved capability makes the whole derivation unknown before any
    // role-dependent obligation is evaluated: without a role the reduction and
    // arithmetic obligations cannot be soundly derived.
    let mut members = Vec::with_capacity(ordered.len());
    for member in &ordered {
        match derive_member(graph, registry, capabilities, governed_dtype, *member)? {
            Some(derivation) => members.push(derivation),
            None => {
                return Ok(FusionLegality::Unknown(FusionUnknown {
                    obligation: FusionObligation::OperationCapabilitiesResolved,
                    reason: "unsupported-operation-capability",
                    region: candidate.label().to_owned(),
                }));
            }
        }
    }

    let obligations = derive_obligations(&members, contract);
    if let Some(rejected) = first_rejection(&obligations, candidate) {
        return Ok(FusionLegality::Rejected(rejected));
    }
    if let Some(unknown) = first_unknown(&obligations, candidate) {
        return Ok(FusionLegality::Unknown(unknown));
    }

    let structure = region_structure(candidate, &members);
    let content = assemble_content(candidate, contract, structure, obligations);
    let proof = assemble_proof(candidate, capabilities, registry, &members, content);
    Ok(FusionLegality::Legal(Box::new(proof)))
}

/// Re-derives one legal proof and requires it to equal the retained evidence.
///
/// # Errors
///
/// Returns a [`FusionLegalityError`] when the candidate does not re-derive, or a
/// [`FusionLegalityError::Structure`] when the re-derivation is not a legal proof
/// equal to `proof`.
pub(crate) fn verify_fusion_legality(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    contract: StrictF32NumericalContract,
    capabilities: &FusionNumericalCapabilities,
    formation: &RegionFormationOutcome,
    candidate: &RegionCandidate,
    proof: &FusionLegalityProof,
) -> Result<(), FusionLegalityError> {
    match derive_fusion_legality(
        program,
        budgets,
        contract,
        capabilities,
        formation,
        candidate,
    )? {
        FusionLegality::Legal(expected) if expected.as_ref() == proof => Ok(()),
        _ => Err(FusionLegalityError::Structure {
            rule: "legality-proof-subject",
        }),
    }
}

/// Orders the candidate's members by content-derived canonical position.
///
/// **Projected from attribution atoms onto occurrences.** Fusion legality is
/// derived from operation facts — family key, proven purity, canonical operand
/// and result type encodings — none of which a stage of an occurrence refines,
/// so a candidate covering two stages of one occurrence contributes that
/// occurrence once rather than twice.
fn ordered_members(
    graph: &RegionGraph,
    candidate: &RegionCandidate,
) -> Result<Vec<SemanticMemberId>, FusionLegalityError> {
    let mut keyed = Vec::with_capacity(candidate.members().len());
    for atom in candidate.members() {
        let member = atom.member();
        keyed.push((graph.member_canonical_position(member)?, member));
    }
    keyed.sort_by_key(|(position, _)| *position);
    keyed.dedup_by_key(|(position, _)| *position);
    Ok(keyed.into_iter().map(|(_, member)| member).collect())
}

/// Derives one member's role, reached definition, purity, and type homogeneity.
///
/// Returns `Ok(None)` when the member's operation has no fusion capability.
fn derive_member(
    graph: &RegionGraph,
    registry: &FrozenSemanticRegistry,
    capabilities: &FusionNumericalCapabilities,
    governed_dtype: &[u8],
    member: SemanticMemberId,
) -> Result<Option<MemberDerivation>, FusionLegalityError> {
    let facts = graph.member_operation_facts(member)?;
    let definition =
        registry
            .operation_definition(facts.key())
            .ok_or(FusionLegalityError::Structure {
                rule: "missing-operation-definition",
            })?;
    // The derived graph purity must agree with the reached definition's effect;
    // a disagreement is invalid compiler state, not a legality outcome.
    if facts.is_pure() != matches!(definition.effect(), OperationEffect::Pure) {
        return Err(FusionLegalityError::Structure {
            rule: "effect-disagreement",
        });
    }
    let Some(role) = capabilities.classify(facts.key()) else {
        return Ok(None);
    };
    let reached = ReachedDefinition {
        operation: facts.key().clone(),
        normative_definition: definition.normative_definition().as_str().to_owned(),
        effect_tag: effect_tag(definition.effect()),
    };
    Ok(Some(MemberDerivation {
        role,
        reached,
        pure: facts.is_pure(),
        homogeneous: member_is_homogeneous(&facts, governed_dtype),
    }))
}

/// Returns whether every operand and result type is the governed dtype.
fn member_is_homogeneous(facts: &MemberOperationFacts<'_>, governed_dtype: &[u8]) -> bool {
    facts
        .operand_type_encodings()
        .iter()
        .chain(facts.result_type_encodings())
        .all(|encoding| *encoding == governed_dtype)
}

/// Discharges every obligation for the resolved members under the contract.
fn derive_obligations(
    members: &[MemberDerivation],
    contract: StrictF32NumericalContract,
) -> Vec<DerivedObligation> {
    let mut obligations = Vec::new();

    // Every member resolved a capability, or the caller returned unknown earlier.
    obligations.push(DerivedObligation::discharged(
        FusionObligation::OperationCapabilitiesResolved,
        FusionEvidenceClass::SoundProof,
    ));

    obligations.push(if members.iter().all(|member| member.pure) {
        DerivedObligation::discharged(
            FusionObligation::ReferentialTransparency,
            FusionEvidenceClass::SoundProof,
        )
    } else {
        DerivedObligation::rejected(FusionObligation::ReferentialTransparency, "impure-member")
    });

    obligations.push(if members.iter().all(|member| member.homogeneous) {
        DerivedObligation::discharged(
            FusionObligation::ConversionBoundaryPreservation,
            FusionEvidenceClass::SoundProof,
        )
    } else {
        DerivedObligation::unknown(
            FusionObligation::ConversionBoundaryPreservation,
            "unproven-conversion-preservation",
        )
    });

    // The SoundProof is deliberately closed over the exact governed
    // vocabulary. Merely failing to find both built-in keys would be unsound:
    // a future capability could classify another contraction-capable operation
    // as arithmetic. Only constant-f32 sources plus an add-only or
    // multiply-only family, with no reduction or other member, prove there is
    // no multiply-plus-add contraction opportunity. The governed vocabulary
    // holds one family that is contraction-capable within a single operation —
    // `tiler::tensor-contraction-f32@1`, whose per-contributor step is
    // `accumulator + a * b` — so the closure guards a live case and not only a
    // hypothetical one.
    obligations.push(if is_exact_governed_same_family_pointwise(members) {
        DerivedObligation::discharged(
            FusionObligation::ArithmeticContraction,
            FusionEvidenceClass::SoundProof,
        )
    } else if matches!(contract.contraction, NumericalPermission::Forbidden) {
        DerivedObligation::discharged(
            FusionObligation::ArithmeticContraction,
            FusionEvidenceClass::NormativeGuarantee,
        )
    } else {
        DerivedObligation::unknown(
            FusionObligation::ArithmeticContraction,
            "unrealized-contraction",
        )
    });

    // Exceptional values: NaN canonicalization, signed zero, and subnormal
    // handling must survive fusion.
    //
    // The subnormal dimensions do **not** constrain this, whatever their
    // resolution. `docs/numerical-semantics.md` defines both as per-operation
    // rules — "input flushing treats an existing subnormal operand as zero
    // before arithmetic" and "result flushing replaces a newly produced
    // subnormal result with zero". A materialization boundary is a store and a
    // load: neither is arithmetic and neither produces a newly produced result,
    // so removing one neither adds nor removes a flush. The fused and
    // materialized forms perform the same arithmetic under the same
    // per-operation rule, so their exceptional-value behaviour agrees.
    //
    // Requiring `Preserve` here was the strict contract's assumption rather
    // than this obligation's content, and it deferred every fused candidate
    // under any flush contract — costing the fused alternative for a reason the
    // contract does not state.
    //
    // The canonical NaN pattern *is* constrained, and stays. It is a per-result
    // rewrite the fused body must still apply at every arithmetic boundary.
    // `emit_reduction` and the serial prologue's `emit_scale_bias` realize that
    // for serial sums; standalone exact pointwise trees use `emit_pointwise`.
    //
    // A boundary that genuinely carries semantics is guarded separately:
    // `ConversionBoundaryPreservation` above discharges only when every member
    // is homogeneous, so a removed dtype-conversion boundary is refused there
    // rather than here.
    //
    // **The payload is compared against the contract's own width's**, not
    // against binary32's. A BF16 contract carries
    // `CANONICAL_BF16_ARITHMETIC_NAN_BITS`, which is the leading sixteen bits of
    // the binary32 pattern and therefore a different `u32`; measuring it against
    // the `f32` constant would report every BF16 region's exceptional-value
    // behaviour as unproven while the two governed arithmetics both canonicalize
    // exactly as their registered records declare. A width with no governed
    // payload stays unknown rather than borrowing a neighbour's.
    let exceptional_ok = governed_canonical_arithmetic_nan_bits(contract.arithmetic)
        .is_some_and(|governed| contract.canonical_arithmetic_nan_bits == governed);
    obligations.push(if exceptional_ok {
        DerivedObligation::discharged(
            FusionObligation::ExceptionalValues,
            FusionEvidenceClass::NormativeGuarantee,
        )
    } else {
        DerivedObligation::unknown(
            FusionObligation::ExceptionalValues,
            "unproven-exceptional-values",
        )
    });

    push_reduction_obligations(&mut obligations, members, contract);
    obligations
}

/// The canonical arithmetic-NaN payload one governed width's arithmetic installs.
///
/// Exhaustive over [`ArithmeticType`] with no wildcard arm (ADR 0074 convention
/// 3): a width admitted later must decide its own payload here as a compile
/// error rather than fall through to a neighbour's, which is the exact failure
/// mode the per-width keying exists to prevent. `f16` and `f64` are named by the
/// arithmetic vocabulary and carry no registered canonical payload, so they
/// answer `None` and the obligation stays unknown.
///
/// The two values are the constants `tiler-ir` registers on the arithmetic
/// families themselves — `BF16_FACT_CANONICAL_NAN_BITS` carries the `bf16` one
/// and the binary32 families carry the other — rather than a second statement of
/// them here.
const fn governed_canonical_arithmetic_nan_bits(arithmetic: ArithmeticType) -> Option<u32> {
    match arithmetic {
        ArithmeticType::F32 => Some(CANONICAL_F32_ARITHMETIC_NAN_BITS),
        ArithmeticType::Bf16 => Some(CANONICAL_BF16_ARITHMETIC_NAN_BITS as u32),
        ArithmeticType::F16 | ArithmeticType::F64 => None,
    }
}

/// Whether every arithmetic member is one governed same-family pointwise chain.
///
/// **Closed over exact keys at both widths, and each key's transfer is decided
/// rather than read off its role.** The `f32` derivations are recorded at their
/// arms below; the two BF16 arithmetics are admitted on the identical argument
/// the `f32` add and multiply are admitted on — the conclusion is drawn only
/// when every arithmetic member is an add or every one is a multiply, and a
/// region of adds alone holds no product for an add to absorb — and the BF16
/// constant is admitted on the argument the `f32` constant is, that a value
/// source performing no arithmetic introduces no adjacency.
///
/// **The add/multiply predicates span the two widths and the conclusion still
/// holds**, which is worth stating because a mixed-width region reaching here
/// would otherwise look like an unexamined case. `recognized_program_arithmetic`
/// refuses a program whose values are two widths at once and
/// [`FusionObligation::ConversionBoundaryPreservation`] independently refuses a
/// member that is not the region's own dtype, so no compilable region mixes
/// them; and if one did, a region of an `f32` add beside a `bf16` add still
/// contains no multiply, so "no multiply-plus-add adjacency" is true of it for
/// the same reason.
fn is_exact_governed_same_family_pointwise(members: &[MemberDerivation]) -> bool {
    let constant = constant_f32_op();
    let add = add_f32_op();
    let multiply = multiply_f32_op();
    let reindex = reindex_f32_op();
    let broadcast = broadcast_f32_op();
    let concatenate = concatenate_f32_op();
    let slice = slice_f32_op();
    let bf16_constant = constant_bf16_op();
    let bf16_add = add_bf16_op();
    let bf16_multiply = multiply_bf16_op();
    let mut arithmetic_count = 0_usize;
    let mut all_add = true;
    let mut all_multiply = true;

    for member in members {
        match member.role {
            FusionOperationRole::ValueSource
                if member.reached.operation == constant
                    || member.reached.operation == bf16_constant => {}
            // A governed coordinate relation neither creates nor removes a
            // multiply-plus-add adjacency, so it is passed over rather than
            // disqualifying the region. The soundness rests on what survives
            // below: the conclusion is drawn only when every arithmetic member
            // is an add or every one is a multiply, and inserting a pure data
            // movement between two adds cannot introduce a product to fuse.
            // Closed over the exact governed keys for the same reason the
            // constant arm is: a future capability could classify another
            // contraction-capable family as a coordinate relation.
            //
            // The concatenate is admitted here by deciding that transfer for
            // its key rather than by inheriting it from the role: a join
            // introduces no multiply, no add, and therefore no adjacency
            // between them, so inserting one between two adds cannot create a
            // product to fuse either. Declining to decide it is not free —
            // under a contraction-permitting contract a member that falls
            // through returns `unrealized-contraction` below and `first_unknown`
            // makes the whole candidate unknown, deferring every fused
            // candidate containing a concatenate for a reason the family's own
            // semantics refute.
            //
            // The selection is admitted by deciding the same transfer for its
            // key, and the one thing that could have made the transfer fail is
            // checked rather than passed over. The arm's argument is that
            // "inserting a pure data movement between two adds cannot introduce
            // a product to fuse"; a selection moves strictly *less* data than a
            // reindex, because its map is injective and not surjective, and the
            // argument turns on the movement introducing no operation rather
            // than on its being total. A selection introduces no multiply, no
            // add, and therefore no adjacency between them, so a region of adds
            // with one inserted still holds no product. Declining to decide it
            // costs the same as declining for the concatenate: under a
            // contraction-permitting contract a member falling through returns
            // `unrealized-contraction` below and `first_unknown` makes the whole
            // candidate unknown.
            FusionOperationRole::CoordinateRelation
                if member.reached.operation == reindex
                    || member.reached.operation == broadcast
                    || member.reached.operation == concatenate
                    || member.reached.operation == slice => {}
            FusionOperationRole::ElementwiseArithmetic => {
                arithmetic_count = arithmetic_count.saturating_add(1);
                all_add &= member.reached.operation == add || member.reached.operation == bf16_add;
                all_multiply &= member.reached.operation == multiply
                    || member.reached.operation == bf16_multiply;
            }
            // Every remaining role disqualifies the sound proof, and the
            // prologue-carrying reduction disqualifies it for its own reason
            // rather than the fold's. For `tiler::rms-norm-f32@1` the epilogue
            // puts a multiply next to an add — `u + eps` after `a / N` — so a
            // region containing one has a contraction opportunity that this
            // closed proof cannot rule out. For
            // `tiler::tensor-contraction-f32@1` the adjacency is not in an
            // epilogue at all but in the per-contributor step itself:
            // `accumulator + a * b` is a multiply feeding an add, which is
            // precisely the shape ADR 0015's permission is about. So this family
            // is the case the arm's closure was written for, and it is decided
            // here in the *opposite* direction from the governed coordinate
            // relations above — admitting it would hand a region the very
            // conclusion the proof exists to withhold, and would contradict
            // `policy`'s `TENSOR_CONTRACTION` capability row, the only admitted
            // operation declaring `NumericalDimension::Contraction` and declaring
            // it on exactly this step. Declining is not merely
            // conservative: the family's own facts declare ADR 0015 contraction
            // *forbidden* for it, so under a permitting contract a fused body is
            // free to emit a fused multiply-add its normative definition refuses,
            // and nothing in this bounded profile proves it will not. The honest
            // outcome is the `unrealized-contraction` unknown below, not a
            // rejection — a realization that carries `-ffp-contract=off` through
            // to the emitted text satisfies the obligation, so no violation is
            // proved. Under any contract that forbids contraction — which is
            // every governed contract except the relaxed one — the obligation
            // discharges by normative guarantee and a contraction-bearing region
            // is legal.
            // The extremum-shifted reduction disqualifies it for its own reason
            // as well: its epilogue puts a multiply next to a division —
            // `e_i * (1 / d)` — so a region containing one has an arithmetic
            // adjacency this closed proof cannot rule out.
            FusionOperationRole::ValueSource
            | FusionOperationRole::OrderedReduction
            | FusionOperationRole::PrologueCarryingOrderedReduction
            | FusionOperationRole::ExtremumShiftedOrderedReduction
            | FusionOperationRole::CoordinateRelation => {
                return false;
            }
        }
    }
    arithmetic_count != 0 && (all_add || all_multiply)
}

/// Pushes the four reduction obligations, kept independent per ADR 0014.
fn push_reduction_obligations(
    obligations: &mut Vec<DerivedObligation>,
    members: &[MemberDerivation],
    contract: StrictF32NumericalContract,
) {
    let has_reduction = members.iter().any(|member| member.role.is_reduction());

    // Identity/empty-domain and contributor order rest on the ordered-reduction
    // role's normative definition. With no reduction the obligation is
    // vacuously discharged as a structural fact.
    let reduction_class = if has_reduction {
        FusionEvidenceClass::NormativeGuarantee
    } else {
        FusionEvidenceClass::SoundProof
    };
    obligations.push(DerivedObligation::discharged(
        FusionObligation::ReductionIdentityAndEmptyDomain,
        reduction_class,
    ));
    obligations.push(DerivedObligation::discharged(
        FusionObligation::ReductionContributorOrder,
        reduction_class,
    ));

    // Reassociation is a policy permission over the ordered-reduction role.
    // A pointwise region has no reduction order to preserve here: its exact
    // arithmetic tree is already part of the semantic candidate and the
    // scheduled expression, so this reduction obligation is vacuous.
    obligations.push(
        if !has_reduction || matches!(contract.reassociation, NumericalPermission::Forbidden) {
            DerivedObligation::discharged(
                FusionObligation::ReductionReassociation,
                FusionEvidenceClass::SoundProof,
            )
        } else {
            DerivedObligation::unknown(
                FusionObligation::ReductionReassociation,
                "unproven-reassociation",
            )
        },
    );

    // Operand permutation is independent: the ordered left fold fixes operand
    // order, so no permutation is used. It is derived from the role, not from a
    // separate contract permission field.
    obligations.push(DerivedObligation::discharged(
        FusionObligation::ReductionOperandPermutation,
        FusionEvidenceClass::SoundProof,
    ));
}

/// Returns the first rejected obligation as a typed rejection.
fn first_rejection(
    obligations: &[DerivedObligation],
    candidate: &RegionCandidate,
) -> Option<FusionRejection> {
    obligations
        .iter()
        .find_map(|derived| match derived.assessment {
            ObligationAssessment::Rejected { reason } => Some(FusionRejection {
                obligation: derived.obligation,
                reason,
                region: candidate.label().to_owned(),
            }),
            _ => None,
        })
}

/// Returns the first unknown obligation as a typed unknown.
fn first_unknown(
    obligations: &[DerivedObligation],
    candidate: &RegionCandidate,
) -> Option<FusionUnknown> {
    obligations
        .iter()
        .find_map(|derived| match derived.assessment {
            ObligationAssessment::Unknown { reason } => Some(FusionUnknown {
                obligation: derived.obligation,
                reason,
                region: candidate.label().to_owned(),
            }),
            _ => None,
        })
}

/// Computes the site-independent structural counts of the region.
fn region_structure(
    candidate: &RegionCandidate,
    members: &[MemberDerivation],
) -> FusionRegionStructure {
    let count = |predicate: fn(FusionOperationRole) -> bool| {
        u32::try_from(
            members
                .iter()
                .filter(|member| predicate(member.role))
                .count(),
        )
        .unwrap_or(u32::MAX)
    };
    FusionRegionStructure {
        members: u32::try_from(members.len()).unwrap_or(u32::MAX),
        value_sources: count(FusionOperationRole::is_value_source),
        arithmetic: count(FusionOperationRole::is_arithmetic),
        reductions: count(FusionOperationRole::is_reduction),
        coordinate_relations: count(FusionOperationRole::is_coordinate_relation),
        boundary_inputs: u32::try_from(candidate.boundary_inputs().len()).unwrap_or(u32::MAX),
        retained_outputs: u32::try_from(candidate.retained_outputs().len()).unwrap_or(u32::MAX),
    }
}

/// Assembles reusable content and its canonical identity.
fn assemble_content(
    candidate: &RegionCandidate,
    contract: StrictF32NumericalContract,
    structure: FusionRegionStructure,
    obligations: Vec<DerivedObligation>,
) -> FusionLegalityContent {
    let region_content = candidate.content().clone();
    let identity = encode_content_identity(&region_content, contract.key, &structure, &obligations);
    FusionLegalityContent {
        region_content,
        numerical_contract_key: contract.key,
        structure,
        obligations,
        identity,
    }
}

/// Assembles the occurrence binding and its canonical identity.
fn assemble_proof(
    candidate: &RegionCandidate,
    capabilities: &FusionNumericalCapabilities,
    registry: &FrozenSemanticRegistry,
    members: &[MemberDerivation],
    content: FusionLegalityContent,
) -> FusionLegalityProof {
    let reached_definitions = members
        .iter()
        .map(|member| member.reached.clone())
        .collect::<Vec<_>>();
    let value_bindings = value_bindings(candidate);
    let registry_snapshot = registry
        .snapshot_identity()
        .as_bytes()
        .to_vec()
        .into_boxed_slice();
    let identity = encode_occurrence_identity(
        &content,
        candidate.occurrence(),
        &registry_snapshot,
        &reached_definitions,
        capabilities,
        &value_bindings,
    );
    FusionLegalityProof {
        content,
        region_occurrence: candidate.occurrence().clone(),
        registry_snapshot,
        reached_definitions,
        provider: capabilities.provider().clone(),
        provider_revision: capabilities.revision(),
        value_bindings,
        identity,
    }
}

/// Extracts the ordered value/access mapping from the candidate.
fn value_bindings(candidate: &RegionCandidate) -> FusionValueBindings {
    let boundary_inputs = candidate
        .boundary_inputs()
        .iter()
        .map(|value| value.0)
        .collect();
    let retained_outputs = candidate
        .retained_outputs()
        .iter()
        .map(|output| RetainedOutputBinding {
            value: output.value.0,
            producer: output.producer.0,
            result_position: output.result_position,
            named_result: output.named_result,
            external_consumers: output.external_consumers,
        })
        .collect();
    FusionValueBindings {
        boundary_inputs,
        retained_outputs,
    }
}

fn encode_content_identity(
    region_content: &RegionContentIdentity,
    contract_key: &str,
    structure: &FusionRegionStructure,
    obligations: &[DerivedObligation],
) -> FusionLegalityContentIdentity {
    let mut bytes = CONTENT_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, region_content.as_bytes());
    push_slice(&mut bytes, contract_key.as_bytes());
    structure.encode(&mut bytes);
    push_len(&mut bytes, obligations.len());
    for derived in obligations {
        bytes.push(derived.obligation.tag());
        bytes.push(derived.assessment.tag());
        bytes.push(derived.evidence.tag());
        push_slice(
            &mut bytes,
            derived.assessment.reason().unwrap_or("").as_bytes(),
        );
    }
    FusionLegalityContentIdentity(bytes)
}

fn encode_occurrence_identity(
    content: &FusionLegalityContent,
    occurrence: &RegionOccurrenceIdentity,
    registry_snapshot: &[u8],
    reached_definitions: &[ReachedDefinition],
    capabilities: &FusionNumericalCapabilities,
    value_bindings: &FusionValueBindings,
) -> FusionLegalityIdentity {
    let mut bytes = OCCURRENCE_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, content.identity.as_bytes());
    push_slice(&mut bytes, occurrence.as_bytes());
    push_slice(&mut bytes, registry_snapshot);
    push_len(&mut bytes, reached_definitions.len());
    for reached in reached_definitions {
        encode_op_key(&mut bytes, &reached.operation);
        push_slice(&mut bytes, reached.normative_definition.as_bytes());
        bytes.push(reached.effect_tag);
    }
    encode_provider(&mut bytes, capabilities.provider());
    bytes.extend_from_slice(&capabilities.revision().to_be_bytes());
    push_len(&mut bytes, value_bindings.boundary_inputs.len());
    for input in &value_bindings.boundary_inputs {
        bytes.extend_from_slice(&input.to_be_bytes());
    }
    push_len(&mut bytes, value_bindings.retained_outputs.len());
    for output in &value_bindings.retained_outputs {
        bytes.extend_from_slice(&output.value.to_be_bytes());
        bytes.extend_from_slice(&output.producer.to_be_bytes());
        bytes.extend_from_slice(&output.result_position.to_be_bytes());
        bytes.push(u8::from(output.named_result));
        bytes.push(u8::from(output.external_consumers));
    }
    FusionLegalityIdentity(bytes)
}

/// Encodes one observable effect class into fusion-legality identity.
///
/// Exhaustive with no wildcard arm (ADR 0074 convention 3): a second effect
/// must choose its own tag at this site as a compile error, because a wildcard
/// would give two structurally distinct occurrences the same identity bytes.
/// That is only expressible because `OperationEffect` deliberately carries no
/// `#[non_exhaustive]`, which is what convention 5b decides for a vocabulary an
/// out-of-crate encoder maps totally.
const fn effect_tag(effect: OperationEffect) -> u8 {
    match effect {
        OperationEffect::Pure => 1,
    }
}

fn encode_op_key(output: &mut Vec<u8>, key: &OpKey) {
    push_slice(output, key.namespace().as_bytes());
    push_slice(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}

fn encode_provider(output: &mut Vec<u8>, provider: &ProviderIdentity) {
    push_slice(output, provider.namespace().as_bytes());
    push_slice(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::{
        DerivedObligation, FusionEvidenceClass, FusionLegality, FusionLegalityError,
        FusionNumericalCapabilities, FusionObligation, FusionOperationRole, MemberDerivation,
        ObligationAssessment, ReachedDefinition, derive_fusion_legality, derive_obligations,
        verify_fusion_legality,
    };
    use crate::region::{RegionCandidate, RegionFormationOutcome, form_region_candidates};
    use crate::request::{DeterministicBudgets, NumericalPermission, StrictF32NumericalContract};
    use tiler_ir::semantic::{
        BroadcastAxisMapping, BroadcastAxisSource, F32, F32Add, F32Broadcast, F32Constant,
        F32Multiply, F32Reindex, InputKey, OpKey, OutputKey, ReindexForm, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum, add_f32_op, broadcast_f32_op, constant_f32_op,
        gather_f32_op, reindex_f32_op,
    };
    use tiler_ir::shape::{Axis, Extent, Shape};

    /// Gather has no governed fusion role and therefore derives no legality.
    ///
    /// This names the exact authority ADR 0107 relies on, independently of the
    /// policy inventory and request recognizer. Watched failing under a
    /// deliberate subject perturbation: inserting the Gather key as a coordinate
    /// relation changes this result to `Some(CoordinateRelation)`.
    #[test]
    fn gather_is_absent_from_the_governed_fusion_roles() {
        assert_eq!(
            FusionNumericalCapabilities::governed().classify(&gather_f32_op()),
            None,
        );
    }

    fn serial_sum_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    /// The RMS-normalization weight multiply, plus the head-layout transpose.
    ///
    /// `Multiply(x, Broadcast(w))` is the pinned workload's most frequent
    /// structural occurrence — 113 of its 197 broadcasts — and the reindex is the
    /// head-layout permutation that follows a projection. Both structural
    /// families and one arithmetic family in one connected region is exactly the
    /// shape a fusion role has to classify.
    fn structural_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let activations = builder
            .input::<F32>(
                InputKey::new("activations").unwrap(),
                Shape::from_dims([2, 3]),
            )
            .unwrap();
        let weight = builder
            .input::<F32>(InputKey::new("weight").unwrap(), Shape::from_dims([3]))
            .unwrap();
        let mapping = BroadcastAxisMapping::new(
            [Extent::new(2), Extent::new(3)],
            [
                BroadcastAxisSource::Replicate,
                BroadcastAxisSource::FromOperand(Axis::new(0)),
            ],
        )
        .unwrap();
        let widened = F32Broadcast::apply(&mut builder, &mapping, weight).unwrap();
        let scaled = F32Multiply::apply(&mut builder, activations, widened).unwrap();
        let form = ReindexForm::permute_axes([Axis::new(1), Axis::new(0)]).unwrap();
        let transposed = F32Reindex::apply(&mut builder, &form, scaled).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), transposed)
            .unwrap();
        builder.build().unwrap()
    }

    /// The structural families resolve a role, so a region containing them has
    /// fusion legality instead of failing closed.
    ///
    /// The perturbation is the same region with one role withdrawn: it returns
    /// to `unsupported-operation-capability`, which is what makes the positive
    /// result a property of the registered role rather than of the region.
    #[test]
    fn a_region_containing_both_structural_families_derives_legality() {
        let program = structural_program();
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let (formation, candidate) = whole_program_candidate(&program);

        let outcome = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed(),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Legal(proof) = outcome else {
            panic!("a governed structural region is legal, not {outcome:?}");
        };
        let structure = proof.content().structure();
        assert_eq!(structure.members, 3);
        assert_eq!(structure.arithmetic, 1);
        assert_eq!(structure.coordinate_relations, 2);
        assert_eq!(structure.value_sources, 0);
        assert_eq!(structure.reductions, 0);
        assert_eq!(
            structure.members,
            structure.value_sources
                + structure.arithmetic
                + structure.reductions
                + structure.coordinate_relations,
            "the four role counts account for every member, which is what a \
             separate coordinate-relation count exists to keep true"
        );

        for excluded in [reindex_f32_op(), broadcast_f32_op()] {
            let outcome = derive_fusion_legality(
                &program,
                budgets,
                contract,
                &FusionNumericalCapabilities::governed_without(&excluded),
                &formation,
                &candidate,
            )
            .unwrap();
            let FusionLegality::Unknown(unknown) = outcome else {
                panic!("withdrawing {excluded}'s role must fail closed, not {outcome:?}");
            };
            assert_eq!(
                unknown.obligation(),
                FusionObligation::OperationCapabilitiesResolved
            );
            assert_eq!(unknown.reason(), "unsupported-operation-capability");
        }
    }

    /// A coordinate relation neither creates nor removes a contraction site.
    ///
    /// The same-family pointwise proof passes over a governed reindex or
    /// broadcast rather than being disqualified by it, and the neighbouring case
    /// shows the arm is not simply always true: a region mixing an add and a
    /// multiply still falls through to the contract's own resolution.
    #[test]
    fn a_governed_coordinate_relation_does_not_disqualify_the_same_family_proof() {
        let reindex = MemberDerivation {
            role: FusionOperationRole::CoordinateRelation,
            reached: ReachedDefinition {
                operation: reindex_f32_op(),
                normative_definition: String::new(),
                effect_tag: 1,
            },
            pure: true,
            homogeneous: true,
        };
        let broadcast = MemberDerivation {
            role: FusionOperationRole::CoordinateRelation,
            reached: ReachedDefinition {
                operation: broadcast_f32_op(),
                normative_definition: String::new(),
                effect_tag: 1,
            },
            pure: true,
            homogeneous: true,
        };
        let arithmetic = |operation: OpKey| MemberDerivation {
            role: FusionOperationRole::ElementwiseArithmetic,
            reached: ReachedDefinition {
                operation,
                normative_definition: String::new(),
                effect_tag: 1,
            },
            pure: true,
            homogeneous: true,
        };
        assert!(super::is_exact_governed_same_family_pointwise(&[
            reindex,
            broadcast,
            arithmetic(add_f32_op()),
        ]));
        // An unrecognized coordinate relation is *not* passed over, for the same
        // reason an unrecognized value source is not: a future capability could
        // classify a contraction-capable family as one.
        let foreign = MemberDerivation {
            role: FusionOperationRole::CoordinateRelation,
            reached: ReachedDefinition {
                operation: OpKey::new("example", "unknown-relation", 1).unwrap(),
                normative_definition: String::new(),
                effect_tag: 1,
            },
            pure: true,
            homogeneous: true,
        };
        assert!(!super::is_exact_governed_same_family_pointwise(&[
            foreign,
            arithmetic(add_f32_op()),
        ]));
    }

    fn square_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let product = F32Multiply::apply(&mut builder, input, input).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), product)
            .unwrap();
        builder.build().unwrap()
    }

    /// Returns a program's region formation beside its whole-program candidate.
    ///
    /// Both, because the derivation now takes the formation instead of building
    /// its own graph: a helper that returned only the candidate would leave every
    /// caller re-deriving the formation this one already has.
    fn whole_program_candidate(
        program: &SemanticProgram,
    ) -> (RegionFormationOutcome, RegionCandidate) {
        let outcome = form_region_candidates(
            program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
        )
        .unwrap();
        let candidate = outcome
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();
        (outcome, candidate)
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        !needle.is_empty()
            && haystack
                .windows(needle.len())
                .any(|window| window == needle)
    }

    #[test]
    fn whole_program_serial_sum_is_legal_with_replayable_evidence() {
        let program = serial_sum_program();
        let (formation, candidate) = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Legal(proof) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            &candidate,
        )
        .unwrap() else {
            panic!("the governed serial sum fuses legally");
        };

        // Replay reproduces the exact proof.
        verify_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            &candidate,
            &proof,
        )
        .unwrap();

        // Every obligation is discharged with a labelled evidence class, and the
        // reduction obligations carry a normative guarantee, not a bare label.
        assert!(
            proof
                .content()
                .obligations()
                .iter()
                .all(|derived| matches!(derived.assessment(), ObligationAssessment::Discharged))
        );
        let reduction = proof
            .content()
            .obligations()
            .iter()
            .find(|derived| derived.obligation() == FusionObligation::ReductionContributorOrder)
            .unwrap();
        assert_eq!(
            reduction.evidence(),
            FusionEvidenceClass::NormativeGuarantee
        );
        assert_eq!(proof.content().structure().reduction_count(), 1);

        // The reached definitions cover every member and name the reduction's
        // normative definition.
        assert_eq!(
            proof.reached_definitions().len(),
            usize::try_from(proof.content().structure().member_count()).unwrap()
        );
        assert!(proof.reached_definitions().iter().any(|reached| {
            reached
                .normative_definition()
                .contains("strict-serial-sum-f32")
        }));

        // The occurrence binds the ordered value/access mapping.
        assert_eq!(proof.value_bindings().retained_outputs().len(), 1);
    }

    #[test]
    fn content_identity_excludes_provider_and_occurrence() {
        let program = serial_sum_program();
        let (formation, candidate) = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Legal(proof) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            &candidate,
        )
        .unwrap() else {
            panic!("legal");
        };

        let content_bytes = proof.content().identity().as_bytes();
        let occurrence_bytes = proof.identity().as_bytes();

        // Content and occurrence identities are distinct.
        assert_ne!(content_bytes, occurrence_bytes);
        // Pure content contains neither the selected provider nor the graph site.
        assert!(!contains(content_bytes, proof.provider().name().as_bytes()));
        assert!(!contains(
            content_bytes,
            proof.region_occurrence().as_bytes()
        ));
        // The occurrence binding is content plus the site and the provider.
        assert!(contains(occurrence_bytes, content_bytes));
        assert!(contains(
            occurrence_bytes,
            proof.provider().name().as_bytes()
        ));
        assert!(contains(
            occurrence_bytes,
            proof.region_occurrence().as_bytes()
        ));
    }

    #[test]
    fn a_pure_pointwise_square_is_legal_with_no_reduction() {
        let program = square_program();
        let (formation, candidate) = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Legal(proof) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            &candidate,
        )
        .unwrap() else {
            panic!("a pure square fuses legally");
        };
        assert_eq!(proof.content().structure().reduction_count(), 0);
        // The reduction obligations are vacuously discharged as sound structural
        // facts, distinct from the normative guarantee a real reduction carries.
        let order = proof
            .content()
            .obligations()
            .iter()
            .find(|derived| derived.obligation() == FusionObligation::ReductionContributorOrder)
            .unwrap();
        assert_eq!(order.evidence(), FusionEvidenceClass::SoundProof);
    }

    #[test]
    fn a_relaxed_pointwise_region_preserves_its_exact_tree() {
        let program = square_program();
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed_relaxed();
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region");
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Legal(proof) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            candidate,
        )
        .unwrap() else {
            panic!("permission to transform does not require the transform");
        };
        assert!(
            proof.content().obligations().iter().all(|derived| {
                matches!(derived.assessment(), ObligationAssessment::Discharged)
            })
        );
    }

    #[test]
    fn a_relaxed_mixed_arithmetic_region_still_needs_contraction_evidence() {
        let program = serial_sum_program();
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed_relaxed();
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region");
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Unknown(unknown) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            candidate,
        )
        .unwrap() else {
            panic!("mixed multiply/add remains outside the relaxed capability");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::ArithmeticContraction
        );
        assert_eq!(unknown.reason(), "unrealized-contraction");
    }

    /// The reassociating contract discharges the same region the relaxed one
    /// cannot, and it does so by *forbidding* contraction rather than by proving
    /// anything new about the emission.
    ///
    /// This is the resolution recorded for
    /// `admit-a-reassociating-contract-without-contraction`. The alternative —
    /// widening `is_exact_governed_same_family_pointwise` to state that the
    /// governed emission performs no contraction — was eliminated rather than
    /// deferred: this authority is handed the program, the budgets, the
    /// contract, the capabilities, and the candidate, and none of them names the
    /// realization that will be emitted or the backend that will emit it; and
    /// under a *permitting* realization the claim is false rather than merely
    /// unprovable, because `tiler_metal::emit::realization_requirements` names
    /// `NoFloatingPointContraction` only in the forbidden arm, so the artifact
    /// carries no contraction obligation at all and the measured Apple row fuses
    /// a written multiply/add pair under `-ffp-contract=fast`.
    ///
    /// The perturbation is the second half: the same contract with contraction
    /// permitted returns to `unrealized-contraction`, so the discharge is
    /// reading the contraction resolution and not the contract's key.
    #[test]
    fn a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction() {
        let program = serial_sum_program();
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed_reassociating();
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region");
        let capabilities = FusionNumericalCapabilities::governed();

        // Stated over the whole formed population rather than one hand-picked
        // candidate, and counted, so "no contraction unknown" cannot be
        // satisfied by a filter that matched nothing: **no** multi-member
        // candidate of this program is unknown for contraction under this
        // contract, and at least one mixed multiply/add region is positively
        // legal with the contract's own normative guarantee behind it.
        let mut legal_mixed = 0_usize;
        for formed in formation.candidates() {
            let outcome = derive_fusion_legality(
                &program,
                budgets,
                contract,
                &capabilities,
                &formation,
                formed,
            )
            .unwrap();
            match outcome {
                FusionLegality::Legal(proof) => {
                    let contraction = proof
                        .content()
                        .obligations()
                        .iter()
                        .find(|derived| {
                            derived.obligation() == FusionObligation::ArithmeticContraction
                        })
                        .unwrap();
                    assert_eq!(contraction.assessment(), ObligationAssessment::Discharged);
                    // A same-family region keeps its structural `SoundProof`;
                    // the mixed regions this preset exists for are the ones that
                    // fall through to the contract's own normative guarantee.
                    if contraction.evidence() == FusionEvidenceClass::NormativeGuarantee {
                        legal_mixed += 1;
                    } else {
                        assert_eq!(contraction.evidence(), FusionEvidenceClass::SoundProof);
                    }
                }
                FusionLegality::Unknown(unknown) => assert_ne!(
                    unknown.obligation(),
                    FusionObligation::ArithmeticContraction,
                    "{unknown}"
                ),
                FusionLegality::Rejected(rejection) => {
                    panic!("no governed candidate is rejected: {rejection}")
                }
            }
        }
        assert!(
            legal_mixed > 0,
            "the sweep proved nothing: no mixed-arithmetic candidate was legal at all"
        );

        // The whole-program candidate additionally contains the reduction, whose
        // reassociation this contract permits and this authority does not prove.
        // It stays unknown, on a *different* obligation — recorded rather than
        // hidden, because it is what keeps the fused whole-program alternative
        // out of the reassociating portfolio and it is not this ticket's to
        // change.
        let FusionLegality::Unknown(unknown) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            candidate,
        )
        .unwrap() else {
            panic!("a permitted reduction reassociation is not proved by this authority");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::ReductionReassociation
        );
        assert_eq!(unknown.reason(), "unproven-reassociation");

        // Perturbation: permit contraction on the same contract and the
        // contraction unknown returns, ahead of the reassociation one.
        let mut permitting = contract;
        permitting.contraction = NumericalPermission::Permitted;
        let FusionLegality::Unknown(unknown) = derive_fusion_legality(
            &program,
            budgets,
            permitting,
            &capabilities,
            &formation,
            candidate,
        )
        .unwrap() else {
            panic!("permitting contraction must lose the normative guarantee");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::ArithmeticContraction
        );
        assert_eq!(unknown.reason(), "unrealized-contraction");
    }

    #[test]
    fn a_future_arithmetic_family_cannot_inherit_same_family_contraction_evidence() {
        let member = |role, operation| MemberDerivation {
            role,
            reached: ReachedDefinition {
                operation,
                normative_definition: "test".to_owned(),
                effect_tag: 1,
            },
            pure: true,
            homogeneous: true,
        };
        let members = [
            member(FusionOperationRole::ValueSource, constant_f32_op()),
            member(
                FusionOperationRole::ElementwiseArithmetic,
                OpKey::new("test", "fma-like-f32", 1).unwrap(),
            ),
        ];

        let obligations =
            derive_obligations(&members, StrictF32NumericalContract::governed_relaxed());
        let contraction = obligations
            .iter()
            .find(|derived| derived.obligation() == FusionObligation::ArithmeticContraction)
            .unwrap();
        assert_eq!(
            contraction.assessment(),
            ObligationAssessment::Unknown {
                reason: "unrealized-contraction"
            },
        );
    }

    #[test]
    fn an_unregistered_operation_capability_is_unknown() {
        let program = serial_sum_program();
        let (formation, candidate) = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        // Drop the add capability so a member operation has no fusion role.
        let capabilities = FusionNumericalCapabilities::governed_without(&add_f32_op());

        let FusionLegality::Unknown(unknown) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            &candidate,
        )
        .unwrap() else {
            panic!("a missing capability fails closed to unknown");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::OperationCapabilitiesResolved
        );
        assert_eq!(unknown.reason(), "unsupported-operation-capability");
    }

    #[test]
    fn a_contract_with_foreign_nan_bits_is_unknown() {
        let program = serial_sum_program();
        let (formation, candidate) = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let capabilities = FusionNumericalCapabilities::governed();
        // Keep the governed contract key (so the candidate re-derives) but demand
        // a NaN pattern the governed operations do not produce.
        let mut contract = StrictF32NumericalContract::governed();
        contract.canonical_arithmetic_nan_bits ^= 1;

        let FusionLegality::Unknown(unknown) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            &candidate,
        )
        .unwrap() else {
            panic!("a foreign NaN contract cannot be proved");
        };
        assert_eq!(unknown.obligation(), FusionObligation::ExceptionalValues);
        assert_eq!(unknown.reason(), "unproven-exceptional-values");
    }

    #[test]
    fn a_forged_proof_fails_replay() {
        let program = serial_sum_program();
        let (formation, candidate) = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Legal(mut proof) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            &candidate,
        )
        .unwrap() else {
            panic!("legal");
        };
        // Tamper with the recorded provider revision.
        proof.provider_revision += 1;
        let error = verify_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &formation,
            &candidate,
            &proof,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            FusionLegalityError::Structure {
                rule: "legality-proof-subject"
            }
        ));
    }

    #[test]
    fn a_candidate_from_another_graph_fails_re_derivation() {
        let program = serial_sum_program();
        let (_, candidate) = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        // A structurally different program yields a different graph, so the
        // stored occurrence identity no longer re-derives. The formation passed
        // here is the *other* program's: taking the graph rather than building
        // one does not weaken this check, because `verify_candidate` still runs
        // the candidate against whatever graph it is handed.
        let other = square_program();
        let (other_formation, _) = whole_program_candidate(&other);
        let error = derive_fusion_legality(
            &other,
            budgets,
            contract,
            &capabilities,
            &other_formation,
            &candidate,
        )
        .unwrap_err();
        assert!(matches!(error, FusionLegalityError::Region(_)));
    }

    #[test]
    fn the_five_evidence_classes_stay_distinct() {
        let classes = [
            FusionEvidenceClass::NormativeGuarantee,
            FusionEvidenceClass::SoundProof,
            FusionEvidenceClass::ExhaustiveFinite,
            FusionEvidenceClass::Empirical,
            FusionEvidenceClass::Unknown,
        ];
        let mut names: Vec<&str> = classes.iter().map(|class| class.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), classes.len());
    }

    #[test]
    fn a_rejection_reports_its_exact_obligation_and_reason() {
        // The reject disposition is a fail-closed guard: the governed strict-f32
        // vocabulary (only pure effects, forbidden permissions, preserved
        // subnormals) cannot express an illegal-but-valid program, so this
        // exercises the typed rejection surface directly.
        let rejection = super::FusionRejection {
            obligation: FusionObligation::ReferentialTransparency,
            reason: "impure-member",
            region: "region:0000000000000000".to_owned(),
        };
        assert_eq!(
            rejection.obligation(),
            FusionObligation::ReferentialTransparency
        );
        assert_eq!(rejection.reason(), "impure-member");
        assert_eq!(
            rejection.to_string(),
            "fusion.referential-transparency.impure-member: region:0000000000000000 rejected"
        );
    }

    #[test]
    fn discharged_obligations_are_never_rejected_or_unknown() {
        let discharged = DerivedObligation::discharged(
            FusionObligation::ArithmeticContraction,
            FusionEvidenceClass::NormativeGuarantee,
        );
        assert!(matches!(
            discharged.assessment(),
            ObligationAssessment::Discharged
        ));
    }
}

#[cfg(test)]
mod softmax_role_tests {
    use super::{FusionNumericalCapabilities, FusionOperationRole};
    use tiler_ir::semantic::{
        rms_norm_f32_op, silu_f32_op, softmax_f32_op, strict_serial_sum_f32_op,
    };

    /// The softmax resolves to a role of its own rather than to `Unknown`.
    ///
    /// Before this vertical a region containing a softmax had no registered
    /// capability and therefore no derivable legality at all, which is the state
    /// the L3′ derivation records for the maximum reduction. Asserting the role
    /// by name is what keeps a later change from quietly reclassifying it as the
    /// prologue-carrying one, whose single-fold contract would let one permission
    /// answer for both of the softmax's passes.
    #[test]
    fn the_softmax_resolves_to_the_extremum_shifted_role() {
        let capabilities = FusionNumericalCapabilities::governed();
        assert_eq!(
            capabilities.classify(&softmax_f32_op()),
            Some(FusionOperationRole::ExtremumShiftedOrderedReduction)
        );
        // The three neighbours keep the roles they had, so the insertion widened
        // the map rather than moving an entry.
        assert_eq!(
            capabilities.classify(&rms_norm_f32_op()),
            Some(FusionOperationRole::PrologueCarryingOrderedReduction)
        );
        assert_eq!(
            capabilities.classify(&strict_serial_sum_f32_op()),
            Some(FusionOperationRole::OrderedReduction)
        );
        assert_eq!(
            capabilities.classify(&silu_f32_op()),
            Some(FusionOperationRole::ElementwiseArithmetic)
        );
    }

    /// The role counts as a reduction, and the three reduction roles are distinct.
    ///
    /// `is_reduction` is what derives the four reduction obligations, so a role
    /// that answered `false` would let a softmax region skip them. The
    /// distinctness assertion is the other half: three roles answering `true` to
    /// one predicate must still be three roles, or the order obligations they
    /// carry would be indistinguishable.
    #[test]
    fn the_extremum_shifted_role_is_a_reduction_and_is_not_the_other_two() {
        assert!(FusionOperationRole::ExtremumShiftedOrderedReduction.is_reduction());
        assert!(FusionOperationRole::PrologueCarryingOrderedReduction.is_reduction());
        assert!(FusionOperationRole::OrderedReduction.is_reduction());
        assert_ne!(
            FusionOperationRole::ExtremumShiftedOrderedReduction,
            FusionOperationRole::PrologueCarryingOrderedReduction
        );
        assert_ne!(
            FusionOperationRole::ExtremumShiftedOrderedReduction,
            FusionOperationRole::OrderedReduction
        );
        // And it is not a value source or a coordinate relation, which are the
        // two roles whose obligations a reduction must not inherit.
        assert!(!FusionOperationRole::ExtremumShiftedOrderedReduction.is_value_source());
    }
}

#[cfg(test)]
mod concatenate_role_tests {
    use super::{
        DerivedObligation, FusionEvidenceClass, FusionLegality, FusionNumericalCapabilities,
        FusionObligation, FusionOperationRole, MemberDerivation, ObligationAssessment,
        ReachedDefinition, derive_fusion_legality, derive_obligations,
    };
    use crate::region::form_region_candidates;
    use crate::request::{DeterministicBudgets, NumericalPermission, StrictF32NumericalContract};
    use tiler_ir::semantic::{
        F32, F32Concatenate, F32Multiply, InputKey, OpKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, broadcast_f32_op, concatenate_f32_op, multiply_f32_op,
        reindex_f32_op,
    };
    use tiler_ir::shape::{Axis, Shape};

    /// One decode step appended to a retained cache, beside the arithmetic that
    /// produced the step.
    ///
    /// The join is what the family exists for and the multiply is the "another
    /// operation" the outcome names: a region holding only a concatenate would
    /// leave the interesting half — whether the join disqualifies a neighbour's
    /// evidence — unexercised.
    fn sequence_extension_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let cache = builder
            .input::<F32>(InputKey::new("cache").unwrap(), Shape::from_dims([2, 2, 4]))
            .unwrap();
        let step = builder
            .input::<F32>(InputKey::new("step").unwrap(), Shape::from_dims([2, 1, 4]))
            .unwrap();
        let gain = builder
            .input::<F32>(InputKey::new("gain").unwrap(), Shape::from_dims([2, 1, 4]))
            .unwrap();
        let scaled = F32Multiply::apply(&mut builder, step, gain).unwrap();
        let extended = F32Concatenate::apply(&mut builder, &[cache, scaled], Axis::new(1)).unwrap();
        builder
            .output(OutputKey::new("extended").unwrap(), extended)
            .unwrap();
        builder.build().unwrap()
    }

    /// The concatenate resolves to the coordinate-relation role.
    ///
    /// Asserting the role by name is what keeps a later change from quietly
    /// reclassifying it as a value source, whose contract would make the
    /// structural counts report a region as holding one more independent value
    /// than it does. The two neighbours are asserted with it so the insertion is
    /// shown to have widened the map rather than moved an entry.
    #[test]
    fn the_concatenate_resolves_to_the_coordinate_relation_role() {
        let capabilities = FusionNumericalCapabilities::governed();
        assert_eq!(
            capabilities.classify(&concatenate_f32_op()),
            Some(FusionOperationRole::CoordinateRelation)
        );
        assert_eq!(
            capabilities.classify(&reindex_f32_op()),
            Some(FusionOperationRole::CoordinateRelation)
        );
        assert_eq!(
            capabilities.classify(&broadcast_f32_op()),
            Some(FusionOperationRole::CoordinateRelation)
        );
        assert_eq!(
            capabilities.classify(&multiply_f32_op()),
            Some(FusionOperationRole::ElementwiseArithmetic)
        );
    }

    /// A region holding a concatenate derives legality instead of failing closed.
    ///
    /// The perturbation is the same region with the role withdrawn: it returns
    /// to `unsupported-operation-capability`, which is what makes the positive
    /// result a property of the registered role rather than of the region.
    #[test]
    fn a_region_holding_a_concatenate_derives_legality_instead_of_failing_closed() {
        let program = sequence_extension_program();
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();

        let outcome = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed(),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Legal(proof) = outcome else {
            panic!("a governed sequence-extension region is legal, not {outcome:?}");
        };

        let structure = proof.content().structure();
        assert_eq!(structure.members, 2);
        assert_eq!(structure.arithmetic, 1);
        assert_eq!(structure.coordinate_relations, 1);
        assert_eq!(structure.value_sources, 0);
        assert_eq!(structure.reductions, 0);
        assert_eq!(
            structure.members,
            structure.value_sources
                + structure.arithmetic
                + structure.reductions
                + structure.coordinate_relations,
            "the four role counts account for every member, and the join is \
             counted as the coordinate relation it is"
        );

        // Every one of the nine obligations is discharged, counted rather than
        // filtered: a population assertion that named no obligation would pass
        // over an empty list.
        let obligations = proof.content().obligations();
        assert_eq!(obligations.len(), 9);
        assert!(
            obligations
                .iter()
                .all(|derived| matches!(derived.assessment(), ObligationAssessment::Discharged)),
            "{obligations:?}"
        );

        let perturbed = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed_without(&concatenate_f32_op()),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Unknown(unknown) = perturbed else {
            panic!("withdrawing the concatenate's role must fail closed, not {perturbed:?}");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::OperationCapabilitiesResolved
        );
        assert_eq!(unknown.reason(), "unsupported-operation-capability");
    }

    /// The contraction proof passes over a concatenate by its exact key.
    ///
    /// The arm is closed over keys rather than over the role, so the decision
    /// that a join introduces no multiply-plus-add adjacency has to be stated
    /// for this key. The counterfactual is a coordinate relation the arm does
    /// *not* name, which is what the concatenate would have reached had the arm
    /// been left unextended: under a contraction-permitting contract it returns
    /// `unrealized-contraction`, which `first_unknown` would make the whole
    /// candidate's verdict.
    #[test]
    fn the_contraction_arm_reads_the_concatenate_key_rather_than_its_role() {
        let permitting = StrictF32NumericalContract::governed_relaxed();
        assert!(
            !matches!(permitting.contraction, NumericalPermission::Forbidden),
            "a contract forbidding contraction discharges the obligation on its \
             own, which would make this perturbation vacuous"
        );

        let member = |role, operation| MemberDerivation {
            role,
            reached: ReachedDefinition {
                operation,
                normative_definition: String::new(),
                effect_tag: 1,
            },
            pure: true,
            homogeneous: true,
        };
        let contraction = |members: &[MemberDerivation]| -> DerivedObligation {
            *derive_obligations(members, permitting)
                .iter()
                .find(|derived| derived.obligation() == FusionObligation::ArithmeticContraction)
                .expect("the contraction obligation is always derived")
        };

        let extended = contraction(&[
            member(
                FusionOperationRole::CoordinateRelation,
                concatenate_f32_op(),
            ),
            member(
                FusionOperationRole::ElementwiseArithmetic,
                multiply_f32_op(),
            ),
        ]);
        assert_eq!(extended.assessment(), ObligationAssessment::Discharged);
        assert_eq!(extended.evidence(), FusionEvidenceClass::SoundProof);

        let unextended = contraction(&[
            member(
                FusionOperationRole::CoordinateRelation,
                OpKey::new("example", "unnamed-relation", 1).unwrap(),
            ),
            member(
                FusionOperationRole::ElementwiseArithmetic,
                multiply_f32_op(),
            ),
        ]);
        assert_eq!(
            unextended.assessment(),
            ObligationAssessment::Unknown {
                reason: "unrealized-contraction"
            }
        );

        // End to end under the same permitting contract: the region stays legal
        // and its contraction obligation carries the structural proof rather
        // than the contract's normative guarantee, which is only available when
        // contraction is forbidden.
        let program = sequence_extension_program();
        let budgets = DeterministicBudgets::governed();
        let formation = form_region_candidates(&program, budgets, permitting).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region");
        let outcome = derive_fusion_legality(
            &program,
            budgets,
            permitting,
            &FusionNumericalCapabilities::governed(),
            &formation,
            candidate,
        )
        .unwrap();
        let FusionLegality::Legal(proof) = outcome else {
            panic!("a permitting contract leaves the region legal, not {outcome:?}");
        };
        let derived = proof
            .content()
            .obligations()
            .iter()
            .find(|derived| derived.obligation() == FusionObligation::ArithmeticContraction)
            .unwrap();
        assert_eq!(derived.assessment(), ObligationAssessment::Discharged);
        assert_eq!(derived.evidence(), FusionEvidenceClass::SoundProof);
    }
}

#[cfg(test)]
mod slice_role_tests {
    use super::{
        DerivedObligation, FusionEvidenceClass, FusionLegality, FusionNumericalCapabilities,
        FusionObligation, FusionOperationRole, MemberDerivation, ObligationAssessment,
        ReachedDefinition, derive_fusion_legality, derive_obligations,
    };
    use crate::region::form_region_candidates;
    use crate::request::{DeterministicBudgets, NumericalPermission, StrictF32NumericalContract};
    use tiler_ir::semantic::{
        F32, F32Multiply, F32Slice, InputKey, OpKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, SliceAxisSelection, SliceSelection, broadcast_f32_op,
        concatenate_f32_op, multiply_f32_op, reindex_f32_op, slice_f32_op,
    };
    use tiler_ir::shape::{Extent, Shape};

    /// The final-position projection: a gated logit tensor with one row selected.
    ///
    /// The selection is what the family exists for and the multiply is the
    /// "another operation" the outcome names: `derive_fusion_legality` is skipped
    /// below two members, so a region holding only a selection would leave the
    /// interesting half — whether a non-surjective read disqualifies a
    /// neighbour's evidence — unexercised.
    fn selected_projection_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let logits = builder
            .input::<F32>(InputKey::new("logits").unwrap(), Shape::from_dims([2, 4]))
            .unwrap();
        let gain = builder
            .input::<F32>(InputKey::new("gain").unwrap(), Shape::from_dims([2, 4]))
            .unwrap();
        let scaled = F32Multiply::apply(&mut builder, logits, gain).unwrap();
        let selection = SliceSelection::new([
            SliceAxisSelection::WholeAxis,
            SliceAxisSelection::static_window(3, Extent::new(1)),
        ])
        .unwrap();
        let selected = F32Slice::apply(&mut builder, &selection, scaled).unwrap();
        builder
            .output(OutputKey::new("selected").unwrap(), selected)
            .unwrap();
        builder.build().unwrap()
    }

    /// The selection resolves to the coordinate-relation role.
    ///
    /// Asserting the role by name is what keeps a later change from quietly
    /// reclassifying it as a value source, whose contract would make the
    /// structural counts report a region as holding one more independent value
    /// than it does. The three coordinate relations already registered are
    /// asserted with it so the insertion is shown to have widened the map rather
    /// than moved an entry.
    #[test]
    fn the_selection_resolves_to_the_coordinate_relation_role() {
        let capabilities = FusionNumericalCapabilities::governed();
        assert_eq!(
            capabilities.classify(&slice_f32_op()),
            Some(FusionOperationRole::CoordinateRelation)
        );
        for neighbour in [reindex_f32_op(), broadcast_f32_op(), concatenate_f32_op()] {
            assert_eq!(
                capabilities.classify(&neighbour),
                Some(FusionOperationRole::CoordinateRelation),
                "{neighbour}'s row moved",
            );
        }
        assert_eq!(
            capabilities.classify(&multiply_f32_op()),
            Some(FusionOperationRole::ElementwiseArithmetic)
        );
        // The role is neither a reduction nor arithmetic, which is what makes the
        // four reduction obligations vacuous for a region holding one rather than
        // resting on a fold's normative definition.
        assert!(!FusionOperationRole::CoordinateRelation.is_reduction());
        assert!(!FusionOperationRole::CoordinateRelation.is_arithmetic());
        assert!(!FusionOperationRole::CoordinateRelation.is_value_source());
    }

    /// A region holding a selection derives legality instead of failing closed.
    ///
    /// The perturbation is the same region with the role withdrawn: it returns
    /// to `unsupported-operation-capability`, which is what makes the positive
    /// result a property of the registered role rather than of the region.
    ///
    /// **The scope of the claim.** This is a derived legality for a formed
    /// candidate. It is not a `VerifiedKernel` and not a device-verified result:
    /// the request boundary still refuses a program stating this family under
    /// `operation-set`, because the region vocabulary's `LogicalAccess` cannot
    /// spell a selection's access relation, so this authority is driven directly
    /// rather than through a compile.
    #[test]
    fn a_region_holding_a_selection_derives_legality_instead_of_failing_closed() {
        let program = selected_projection_program();
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();

        let outcome = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed(),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Legal(proof) = outcome else {
            panic!("a governed selection region is legal, not {outcome:?}");
        };

        let structure = proof.content().structure();
        assert_eq!(structure.members, 2);
        assert_eq!(structure.arithmetic, 1);
        assert_eq!(structure.coordinate_relations, 1);
        assert_eq!(structure.value_sources, 0);
        assert_eq!(structure.reductions, 0);
        assert_eq!(
            structure.members,
            structure.value_sources
                + structure.arithmetic
                + structure.reductions
                + structure.coordinate_relations,
            "the four role counts account for every member, and the selection is \
             counted as the coordinate relation it is"
        );

        // Every one of the nine obligations is discharged, counted rather than
        // filtered: a population assertion that named no obligation would pass
        // over an empty list.
        let obligations = proof.content().obligations();
        assert_eq!(obligations.len(), 9);
        assert!(
            obligations
                .iter()
                .all(|derived| matches!(derived.assessment(), ObligationAssessment::Discharged)),
            "{obligations:?}"
        );
        // The four reduction obligations are discharged *vacuously*: no member of
        // this region carries a fold, so nothing in it raised the obligation.
        // That is a different claim from a fold's obligations having been shown
        // to hold, and the evidence class is what separates the two — a region
        // with a real reduction carries `NormativeGuarantee` on the first two.
        // Asserting it is what stops a later reduction-bearing member from
        // silently inheriting the structural answer.
        for obligation in [
            FusionObligation::ReductionIdentityAndEmptyDomain,
            FusionObligation::ReductionContributorOrder,
            FusionObligation::ReductionReassociation,
            FusionObligation::ReductionOperandPermutation,
        ] {
            let derived = obligations
                .iter()
                .find(|derived| derived.obligation() == obligation)
                .unwrap();
            assert_eq!(derived.evidence(), FusionEvidenceClass::SoundProof);
        }
        let reached_slice = proof
            .reached_definitions()
            .iter()
            .find(|reached| reached.operation() == &slice_f32_op())
            .expect("the proof binds the selection's exact reached definition");
        assert!(
            reached_slice
                .normative_definition()
                .contains("offset is a SourcedExtent"),
            "the compiler proof does not carry the source-bearing Slice selection grammar: {reached_slice:?}",
        );
        assert!(
            reached_slice.normative_definition().contains(
                "after its name is decoded and before parsing any relation-specific fields"
            ),
            "the compiler proof does not carry the corrected reserved-relation boundary: {reached_slice:?}",
        );

        let perturbed = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed_without(&slice_f32_op()),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Unknown(unknown) = perturbed else {
            panic!("withdrawing the selection's role must fail closed, not {perturbed:?}");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::OperationCapabilitiesResolved
        );
        assert_eq!(unknown.reason(), "unsupported-operation-capability");
    }

    /// The contraction proof passes over a selection by its exact key.
    ///
    /// The arm is closed over keys rather than over the role, so the decision
    /// that a non-surjective read introduces no multiply-plus-add adjacency has
    /// to be stated for this key. The counterfactual is a coordinate relation the
    /// arm does *not* name, which is what the selection would have reached had
    /// the arm been left unextended: under a contraction-permitting contract it
    /// returns `unrealized-contraction`, which `first_unknown` would make the
    /// whole candidate's verdict.
    #[test]
    fn the_contraction_arm_reads_the_slice_key_rather_than_its_role() {
        let permitting = StrictF32NumericalContract::governed_relaxed();
        assert!(
            !matches!(permitting.contraction, NumericalPermission::Forbidden),
            "a contract forbidding contraction discharges the obligation on its \
             own, which would make this perturbation vacuous"
        );

        let member = |role, operation| MemberDerivation {
            role,
            reached: ReachedDefinition {
                operation,
                normative_definition: String::new(),
                effect_tag: 1,
            },
            pure: true,
            homogeneous: true,
        };
        let contraction = |members: &[MemberDerivation]| -> DerivedObligation {
            *derive_obligations(members, permitting)
                .iter()
                .find(|derived| derived.obligation() == FusionObligation::ArithmeticContraction)
                .expect("the contraction obligation is always derived")
        };

        let extended = contraction(&[
            member(FusionOperationRole::CoordinateRelation, slice_f32_op()),
            member(
                FusionOperationRole::ElementwiseArithmetic,
                multiply_f32_op(),
            ),
        ]);
        assert_eq!(extended.assessment(), ObligationAssessment::Discharged);
        assert_eq!(extended.evidence(), FusionEvidenceClass::SoundProof);

        let unextended = contraction(&[
            member(
                FusionOperationRole::CoordinateRelation,
                OpKey::new("example", "unselected-relation", 1).unwrap(),
            ),
            member(
                FusionOperationRole::ElementwiseArithmetic,
                multiply_f32_op(),
            ),
        ]);
        assert_eq!(
            unextended.assessment(),
            ObligationAssessment::Unknown {
                reason: "unrealized-contraction"
            }
        );

        // End to end under the same permitting contract: the region stays legal
        // and its contraction obligation carries the structural proof rather
        // than the contract's normative guarantee, which is only available when
        // contraction is forbidden.
        let program = selected_projection_program();
        let budgets = DeterministicBudgets::governed();
        let formation = form_region_candidates(&program, budgets, permitting).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region");
        let outcome = derive_fusion_legality(
            &program,
            budgets,
            permitting,
            &FusionNumericalCapabilities::governed(),
            &formation,
            candidate,
        )
        .unwrap();
        let FusionLegality::Legal(proof) = outcome else {
            panic!("a permitting contract leaves the region legal, not {outcome:?}");
        };
        let derived = proof
            .content()
            .obligations()
            .iter()
            .find(|derived| derived.obligation() == FusionObligation::ArithmeticContraction)
            .unwrap();
        assert_eq!(derived.assessment(), ObligationAssessment::Discharged);
        assert_eq!(derived.evidence(), FusionEvidenceClass::SoundProof);
    }
}

#[cfg(test)]
mod contraction_role_tests {
    use super::{
        DerivedObligation, FusionEvidenceClass, FusionLegality, FusionNumericalCapabilities,
        FusionObligation, FusionOperationRole, MemberDerivation, ObligationAssessment,
        ReachedDefinition, derive_fusion_legality, derive_obligations,
    };
    use crate::region::form_region_candidates;
    use crate::request::{DeterministicBudgets, NumericalPermission, StrictF32NumericalContract};
    use tiler_ir::semantic::{
        ContractionIndex, ContractionIndexStructure, F32, F32Multiply, F32Softmax,
        F32TensorContraction, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
        add_f32_op, multiply_f32_op, rms_norm_f32_op, softmax_f32_op, strict_serial_sum_f32_op,
        tensor_contraction_f32_op,
    };
    use tiler_ir::shape::{Axis, Shape};

    /// The projection structure `td,od->to`, spelled with arbitrary labels so the
    /// renaming-invariant canonicalization is exercised rather than assumed.
    fn projection_structure() -> ContractionIndexStructure {
        ContractionIndexStructure::new(
            [
                [ContractionIndex::new(19), ContractionIndex::new(3)],
                [ContractionIndex::new(14), ContractionIndex::new(3)],
            ],
            [ContractionIndex::new(19), ContractionIndex::new(14)],
        )
        .unwrap()
    }

    /// A projection followed by an elementwise gate, in one connected region.
    ///
    /// Two members, one of them the contraction: a region holding the
    /// contraction alone is the shape planning already skips, so it would leave
    /// the whole question — whether the contraction disqualifies a neighbour's
    /// evidence, and whether its own obligations discharge — unexercised. The
    /// extents stay inside the governed baseline profile's launch bound.
    fn gated_projection_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let activations = builder
            .input::<F32>(
                InputKey::new("activations").unwrap(),
                Shape::from_dims([2, 3]),
            )
            .unwrap();
        let weights = builder
            .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let gain = builder
            .input::<F32>(InputKey::new("gain").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let projected = F32TensorContraction::apply(
            &mut builder,
            &projection_structure(),
            activations,
            weights,
        )
        .unwrap();
        let gated = F32Multiply::apply(&mut builder, projected, gain).unwrap();
        builder
            .output(OutputKey::new("gated").unwrap(), gated)
            .unwrap();
        builder.build().unwrap()
    }

    /// The contraction resolves to the prologue-carrying reduction role.
    ///
    /// Asserting the role by name is what keeps a later change from quietly
    /// reclassifying it: as `OrderedReduction` it would claim the whole
    /// operation is the fold, and as `ElementwiseArithmetic` its four reduction
    /// obligations would discharge vacuously and a reassociating contract would
    /// derive `Legal` for a family whose own facts forbid regrouping. The four
    /// neighbours are asserted with it so the insertion is shown to have widened
    /// the map rather than moved an entry.
    #[test]
    fn the_contraction_resolves_to_the_prologue_carrying_role() {
        let capabilities = FusionNumericalCapabilities::governed();
        assert_eq!(
            capabilities.classify(&tensor_contraction_f32_op()),
            Some(FusionOperationRole::PrologueCarryingOrderedReduction)
        );
        assert_eq!(
            capabilities.classify(&rms_norm_f32_op()),
            Some(FusionOperationRole::PrologueCarryingOrderedReduction)
        );
        assert_eq!(
            capabilities.classify(&softmax_f32_op()),
            Some(FusionOperationRole::ExtremumShiftedOrderedReduction)
        );
        assert_eq!(
            capabilities.classify(&strict_serial_sum_f32_op()),
            Some(FusionOperationRole::OrderedReduction)
        );
        assert_eq!(
            capabilities.classify(&multiply_f32_op()),
            Some(FusionOperationRole::ElementwiseArithmetic)
        );
        // Sharing a role is not sharing an identity: the role is a reduction, so
        // the four reduction obligations are derived for it, and it is neither of
        // the two roles whose obligations a contraction must not inherit.
        assert!(FusionOperationRole::PrologueCarryingOrderedReduction.is_reduction());
        assert!(!FusionOperationRole::PrologueCarryingOrderedReduction.is_arithmetic());
        assert!(!FusionOperationRole::PrologueCarryingOrderedReduction.is_coordinate_relation());
    }

    /// A region holding a contraction derives legality instead of failing closed.
    ///
    /// The perturbation is the same region with the role withdrawn: it returns
    /// to `unsupported-operation-capability`, which is what makes the positive
    /// result a property of the registered role rather than of the region.
    #[test]
    fn a_region_holding_a_contraction_derives_legality_instead_of_failing_closed() {
        let program = gated_projection_program();
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();

        let outcome = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed(),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Legal(proof) = outcome else {
            panic!("a governed gated-projection region is legal, not {outcome:?}");
        };

        let structure = proof.content().structure();
        assert_eq!(structure.members, 2);
        assert_eq!(structure.reductions, 1);
        assert_eq!(structure.arithmetic, 1);
        assert_eq!(structure.value_sources, 0);
        assert_eq!(structure.coordinate_relations, 0);
        assert_eq!(
            structure.members,
            structure.value_sources
                + structure.arithmetic
                + structure.reductions
                + structure.coordinate_relations,
            "the four role counts account for every member, and the contraction \
             is counted under the reduction total rather than needing a fifth"
        );

        // Every one of the nine obligations is discharged, counted rather than
        // filtered: a population assertion that named no obligation would pass
        // over an empty list.
        let obligations = proof.content().obligations();
        assert_eq!(obligations.len(), 9);
        assert!(
            obligations
                .iter()
                .all(|derived| matches!(derived.assessment(), ObligationAssessment::Discharged)),
            "{obligations:?}"
        );
        // The reduction obligations rest on the reached normative definition, so
        // they must carry the guarantee class rather than the vacuous structural
        // one a non-reducing region would produce.
        for obligation in [
            FusionObligation::ReductionIdentityAndEmptyDomain,
            FusionObligation::ReductionContributorOrder,
        ] {
            let derived = obligations
                .iter()
                .find(|derived| derived.obligation() == obligation)
                .unwrap();
            assert_eq!(derived.evidence(), FusionEvidenceClass::NormativeGuarantee);
        }
        assert!(proof.reached_definitions().iter().any(|reached| {
            reached
                .normative_definition()
                .contains("tensor-contraction-f32")
        }));

        let perturbed = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed_without(&tensor_contraction_f32_op()),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Unknown(unknown) = perturbed else {
            panic!("withdrawing the contraction's role must fail closed, not {perturbed:?}");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::OperationCapabilitiesResolved
        );
        assert_eq!(unknown.reason(), "unsupported-operation-capability");
    }

    /// The same-family pointwise proof refuses the contraction by its own step.
    ///
    /// This is the arm's decision in the direction opposite to the governed
    /// coordinate relations': a contraction's per-contributor step *is* the
    /// multiply-plus-add adjacency the closed proof exists to rule out, so
    /// admitting its key would hand a region the conclusion the proof withholds.
    /// Under the governed contract the outcome is invisible — contraction is
    /// forbidden, so the obligation discharges by normative guarantee either way
    /// — which is why the refusal is stated against a permitting contract, and
    /// why that contract is asserted to permit before anything rests on it.
    #[test]
    fn the_contraction_disqualifies_the_same_family_proof_by_its_own_step() {
        let permitting = StrictF32NumericalContract::governed_relaxed();
        assert!(
            !matches!(permitting.contraction, NumericalPermission::Forbidden),
            "a contract forbidding contraction discharges the obligation on its \
             own, which would make this perturbation vacuous"
        );

        let member = |role, operation| MemberDerivation {
            role,
            reached: ReachedDefinition {
                operation,
                normative_definition: String::new(),
                effect_tag: 1,
            },
            pure: true,
            homogeneous: true,
        };
        let contraction_obligation = |members: &[MemberDerivation]| -> DerivedObligation {
            *derive_obligations(members, permitting)
                .iter()
                .find(|derived| derived.obligation() == FusionObligation::ArithmeticContraction)
                .expect("the contraction obligation is always derived")
        };

        // The add-only neighbour would carry the sound proof on its own; the
        // contraction removes it, so the assertion is about the contraction and
        // not about the region being uninteresting.
        let alone = contraction_obligation(&[member(
            FusionOperationRole::ElementwiseArithmetic,
            add_f32_op(),
        )]);
        assert_eq!(alone.assessment(), ObligationAssessment::Discharged);
        assert_eq!(alone.evidence(), FusionEvidenceClass::SoundProof);

        let with_contraction = contraction_obligation(&[
            member(
                FusionOperationRole::PrologueCarryingOrderedReduction,
                tensor_contraction_f32_op(),
            ),
            member(FusionOperationRole::ElementwiseArithmetic, add_f32_op()),
        ]);
        assert_eq!(
            with_contraction.assessment(),
            ObligationAssessment::Unknown {
                reason: "unrealized-contraction"
            }
        );

        // End to end: the same region that is legal under the governed contract
        // is unknown under a permitting one, and the obligation named is the
        // contraction rather than any of the other eight.
        let program = gated_projection_program();
        let budgets = DeterministicBudgets::governed();
        let formation = form_region_candidates(&program, budgets, permitting).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region");
        let outcome = derive_fusion_legality(
            &program,
            budgets,
            permitting,
            &FusionNumericalCapabilities::governed(),
            &formation,
            candidate,
        )
        .unwrap();
        let FusionLegality::Unknown(unknown) = outcome else {
            panic!("a permitting contract cannot establish the contraction, not {outcome:?}");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::ArithmeticContraction
        );
        assert_eq!(unknown.reason(), "unrealized-contraction");
    }

    /// The attention shape `Contraction -> Softmax -> Contraction`.
    ///
    /// Three members, all of them reductions, which is the region the flash
    /// capability record names as the case that makes the missing contraction
    /// role fire: `derive_fusion_legality` is skipped below two members, so a
    /// two-member region is enough to observe the refusal but only this shape is
    /// the one the record's axis 2 is about.
    fn attention_scores_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let queries = builder
            .input::<F32>(InputKey::new("queries").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let keys = builder
            .input::<F32>(InputKey::new("keys").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let values = builder
            .input::<F32>(InputKey::new("values").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scores =
            F32TensorContraction::apply(&mut builder, &projection_structure(), queries, keys)
                .unwrap();
        let weights = F32Softmax::apply(&mut builder, scores, Axis::new(1)).unwrap();
        // `ts,sd->td`: the contracted index is the key position, which is the
        // softmax's own reduced axis and the values' leading axis.
        let attended_structure = ContractionIndexStructure::new(
            [
                [ContractionIndex::new(7), ContractionIndex::new(2)],
                [ContractionIndex::new(2), ContractionIndex::new(5)],
            ],
            [ContractionIndex::new(7), ContractionIndex::new(5)],
        )
        .unwrap();
        let attended =
            F32TensorContraction::apply(&mut builder, &attended_structure, weights, values)
                .unwrap();
        builder
            .output(OutputKey::new("attended").unwrap(), attended)
            .unwrap();
        builder.build().unwrap()
    }

    /// The flash-shaped three-member region derives legality end to end.
    ///
    /// Before the role it failed closed at the first contraction with
    /// `unsupported-operation-capability`, which the perturbation reproduces one
    /// role at a time so the positive result belongs to the contraction's entry
    /// and not to the softmax's.
    #[test]
    fn the_flash_shaped_region_derives_legality_rather_than_failing_closed() {
        let program = attention_scores_program();
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();

        let outcome = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed(),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Legal(proof) = outcome else {
            panic!("a governed attention region is legal, not {outcome:?}");
        };
        let structure = proof.content().structure();
        assert_eq!(structure.members, 3);
        assert_eq!(structure.reductions, 3);
        assert_eq!(
            structure.members,
            structure.value_sources
                + structure.arithmetic
                + structure.reductions
                + structure.coordinate_relations
        );

        for excluded in [tensor_contraction_f32_op(), softmax_f32_op()] {
            let perturbed = derive_fusion_legality(
                &program,
                budgets,
                contract,
                &FusionNumericalCapabilities::governed_without(&excluded),
                &formation,
                &candidate,
            )
            .unwrap();
            let FusionLegality::Unknown(unknown) = perturbed else {
                panic!("withdrawing {excluded}'s role must fail closed, not {perturbed:?}");
            };
            assert_eq!(
                unknown.obligation(),
                FusionObligation::OperationCapabilitiesResolved
            );
            assert_eq!(unknown.reason(), "unsupported-operation-capability");
        }
    }
}

/// BF16 legality, established here rather than inherited from the `f32` table.
#[cfg(test)]
mod bf16_role_tests {
    use super::{
        FusionEvidenceClass, FusionLegality, FusionLegalityError, FusionNumericalCapabilities,
        FusionObligation, FusionOperationRole, ObligationAssessment, derive_fusion_legality,
        derive_obligations, governed_canonical_arithmetic_nan_bits,
    };
    use crate::region::form_region_candidates;
    use crate::request::{DeterministicBudgets, NumericalPermission, StrictF32NumericalContract};
    use tiler_ir::numerics::registered_arithmetic_value_type;
    use tiler_ir::schedule::ArithmeticType;
    use tiler_ir::semantic::{
        Bf16, Bf16Add, Bf16Constant, Bf16Multiply, CANONICAL_BF16_ARITHMETIC_NAN_BITS,
        CANONICAL_F32_ARITHMETIC_NAN_BITS, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, add_bf16_op, add_f32_op, constant_bf16_op, constant_f32_op,
        multiply_bf16_op, multiply_f32_op,
    };
    use tiler_ir::shape::Shape;

    /// `out = (x * 1.0) + 2.0`, in BF16: two constants, a multiply, and an add.
    ///
    /// Four occurrences, because `derive_fusion_legality` is skipped below two
    /// members: a one-occurrence fixture would leave every claim here about a
    /// region no authority is asked about. The multiply beside the add is also
    /// what keeps the contraction obligation live rather than discharged by the
    /// closed same-family proof.
    fn bf16_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let one = Bf16Constant::apply(&mut builder, 0x3f80).unwrap();
        let two = Bf16Constant::apply(&mut builder, 0x4000).unwrap();
        let scaled = Bf16Multiply::apply(&mut builder, input, one).unwrap();
        let shifted = Bf16Add::apply(&mut builder, scaled, two).unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), shifted)
            .unwrap();
        builder.build().unwrap()
    }

    /// A BF16 sum of two inputs: two occurrences, both adds, no multiply.
    ///
    /// The shape the closed same-family proof admits, which is what separates the
    /// contraction obligation's two arms at this width.
    fn bf16_add_only_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let left = builder
            .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let right = builder
            .input::<Bf16>(InputKey::new("y").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let bias = Bf16Constant::apply(&mut builder, 0x3f80).unwrap();
        let sum = Bf16Add::apply(&mut builder, left, right).unwrap();
        let biased = Bf16Add::apply(&mut builder, sum, bias).unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), biased)
            .unwrap();
        builder.build().unwrap()
    }

    fn strict_bf16() -> StrictF32NumericalContract {
        crate::session::NumericalContract::STRICT_BF16.resolve()
    }

    /// The three registered BF16 families resolve their own roles.
    ///
    /// Asserting each by name is what keeps a later change from quietly
    /// reclassifying one: the constant as elementwise arithmetic would put a
    /// rounding on a family whose declared rounding is "none", and either
    /// arithmetic as a value source would make the structural counts report the
    /// region as holding an independent value it does not. The `f32` neighbours
    /// are asserted beside them so the insertion is shown to have widened the map
    /// rather than moved an entry.
    ///
    /// The withheld rows are counted, not described: exactly three BF16 families
    /// are registered in `tiler-ir`, so a fourth role appearing here would be a
    /// row for an operation that does not exist.
    #[test]
    fn the_three_bf16_families_resolve_their_own_roles() {
        let capabilities = FusionNumericalCapabilities::governed();
        assert_eq!(
            capabilities.classify(&constant_bf16_op()),
            Some(FusionOperationRole::ValueSource)
        );
        assert_eq!(
            capabilities.classify(&multiply_bf16_op()),
            Some(FusionOperationRole::ElementwiseArithmetic)
        );
        assert_eq!(
            capabilities.classify(&add_bf16_op()),
            Some(FusionOperationRole::ElementwiseArithmetic)
        );
        for f32_key in [constant_f32_op(), multiply_f32_op(), add_f32_op()] {
            assert!(
                capabilities.classify(&f32_key).is_some(),
                "{f32_key}'s row moved",
            );
        }
        // A BF16 key and its `f32` counterpart are different keys, so the two
        // rows are two decisions rather than one entry read twice.
        assert_ne!(add_bf16_op(), add_f32_op());
        assert_ne!(multiply_bf16_op(), multiply_f32_op());
        assert_ne!(constant_bf16_op(), constant_f32_op());
    }

    /// A multi-occurrence BF16 region derives legality instead of failing closed.
    ///
    /// The perturbation is the same region with one role withdrawn, one family at
    /// a time, which is what makes the positive result a property of each
    /// registered row rather than of the region.
    #[test]
    fn a_bf16_region_derives_legality_instead_of_failing_closed() {
        let program = bf16_program();
        let budgets = DeterministicBudgets::governed();
        let contract = strict_bf16();
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();

        let outcome = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed(),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Legal(proof) = outcome else {
            panic!("a governed bf16 region is legal, not {outcome:?}");
        };

        let structure = proof.content().structure();
        assert_eq!(structure.members, 4);
        assert_eq!(structure.value_sources, 2);
        assert_eq!(structure.arithmetic, 2);
        assert_eq!(structure.reductions, 0);
        assert_eq!(structure.coordinate_relations, 0);
        assert_eq!(
            structure.members,
            structure.value_sources
                + structure.arithmetic
                + structure.reductions
                + structure.coordinate_relations,
            "the four role counts account for every member at this width too"
        );

        // Every one of the nine obligations is discharged, counted rather than
        // filtered: a population assertion that named no obligation would pass
        // over an empty list.
        let obligations = proof.content().obligations();
        assert_eq!(obligations.len(), 9);
        assert!(
            obligations
                .iter()
                .all(|derived| matches!(derived.assessment(), ObligationAssessment::Discharged)),
            "{obligations:?}"
        );
        // The four reduction obligations carry the *vacuous* structural proof,
        // not a normative guarantee: no BF16 family carrying a fold is registered,
        // so there is no contributor sequence for them to be about. Asserting the
        // class is what stops a later reduction row from silently inheriting a
        // guarantee this width has no evidence for.
        for obligation in [
            FusionObligation::ReductionIdentityAndEmptyDomain,
            FusionObligation::ReductionContributorOrder,
            FusionObligation::ReductionReassociation,
            FusionObligation::ReductionOperandPermutation,
        ] {
            let derived = obligations
                .iter()
                .find(|derived| derived.obligation() == obligation)
                .unwrap();
            assert_eq!(derived.evidence(), FusionEvidenceClass::SoundProof);
        }
        // The proof carries the width it was stated for: the contract key renders
        // its own domain, so a `bf16` proof and an `f32` one are distinguishable
        // from the content alone.
        assert!(
            proof
                .content()
                .numerical_contract_key()
                .starts_with("tiler.contract.bf16."),
            "the proof does not name the bf16 contract it was derived under",
        );

        for excluded in [constant_bf16_op(), multiply_bf16_op(), add_bf16_op()] {
            let perturbed = derive_fusion_legality(
                &program,
                budgets,
                contract,
                &FusionNumericalCapabilities::governed_without(&excluded),
                &formation,
                &candidate,
            )
            .unwrap();
            let FusionLegality::Unknown(unknown) = perturbed else {
                panic!("withdrawing {excluded}'s role must fail closed, not {perturbed:?}");
            };
            assert_eq!(
                unknown.obligation(),
                FusionObligation::OperationCapabilitiesResolved
            );
            assert_eq!(unknown.reason(), "unsupported-operation-capability");
        }
    }

    /// The `f32` rows do not answer for a BF16 region's conversion boundary.
    ///
    /// **The keying, driven in the direction that would be silent.** The
    /// conversion-boundary obligation compares every member's operand and result
    /// encodings against the *region's* dtype. Derived under the BF16 contract the
    /// members are homogeneous and the obligation discharges; derived under a
    /// binary32 contract — the exact substitution the old constant performed
    /// unconditionally — the same members are refused by name. The request
    /// boundary already refuses this pairing before planning
    /// (`compile.request.numerics.inapplicable`), so this drives the authority
    /// directly rather than through a compile.
    #[test]
    fn an_f32_contract_does_not_discharge_a_bf16_regions_conversion_boundary() {
        let program = bf16_program();
        let budgets = DeterministicBudgets::governed();
        let bf16 = strict_bf16();
        let f32 = StrictF32NumericalContract::governed();
        assert_ne!(bf16.arithmetic, f32.arithmetic);

        let formation = form_region_candidates(&program, budgets, bf16).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();
        assert!(
            matches!(
                derive_fusion_legality(
                    &program,
                    budgets,
                    bf16,
                    &FusionNumericalCapabilities::governed(),
                    &formation,
                    &candidate,
                )
                .unwrap(),
                FusionLegality::Legal(_)
            ),
            "the region's own contract discharges it, so the refusal below is the substitution's",
        );

        let f32_formation = form_region_candidates(&program, budgets, f32).unwrap();
        let f32_candidate = f32_formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();
        let FusionLegality::Unknown(unknown) = derive_fusion_legality(
            &program,
            budgets,
            f32,
            &FusionNumericalCapabilities::governed(),
            &f32_formation,
            &f32_candidate,
        )
        .unwrap() else {
            panic!("a binary32 contract cannot establish a bf16 region's conversion boundary");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::ConversionBoundaryPreservation
        );
        assert_eq!(unknown.reason(), "unproven-conversion-preservation");
    }

    /// The exceptional-value obligation reads the contract's own width's payload.
    ///
    /// Three cases, because the middle one is the whole content: the governed
    /// BF16 payload discharges, the binary32 payload on a BF16 contract does not
    /// — which is exactly what the removed `StrictF32NumericalContract::governed()`
    /// comparison asserted for every BF16 region — and a width with no registered
    /// payload answers `None` rather than borrowing a neighbour's.
    #[test]
    fn the_exceptional_value_obligation_is_keyed_on_the_contracts_own_width() {
        assert_eq!(
            governed_canonical_arithmetic_nan_bits(ArithmeticType::F32),
            Some(CANONICAL_F32_ARITHMETIC_NAN_BITS)
        );
        assert_eq!(
            governed_canonical_arithmetic_nan_bits(ArithmeticType::Bf16),
            Some(u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS))
        );
        assert_ne!(
            CANONICAL_F32_ARITHMETIC_NAN_BITS,
            u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
            "the two payloads coincide, so no substitution could have been observed",
        );
        for unregistered in [ArithmeticType::F16, ArithmeticType::F64] {
            assert_eq!(governed_canonical_arithmetic_nan_bits(unregistered), None);
        }

        let program = bf16_program();
        let budgets = DeterministicBudgets::governed();
        let contract = strict_bf16();
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();

        // The substitution, driven end to end: the same BF16 contract carrying
        // binary32's payload leaves the obligation unknown.
        let mut substituted = contract;
        substituted.canonical_arithmetic_nan_bits = CANONICAL_F32_ARITHMETIC_NAN_BITS;
        let FusionLegality::Unknown(unknown) = derive_fusion_legality(
            &program,
            budgets,
            substituted,
            &FusionNumericalCapabilities::governed(),
            &formation,
            &candidate,
        )
        .unwrap() else {
            panic!("a foreign NaN payload cannot be proved at this width");
        };
        assert_eq!(unknown.obligation(), FusionObligation::ExceptionalValues);
        assert_eq!(unknown.reason(), "unproven-exceptional-values");
    }

    /// The closed contraction proof reads the BF16 keys, and still withholds.
    ///
    /// **The pair is the claim.** An add-only BF16 region carries the closed
    /// same-family sound proof, because the arm names `tiler::add-bf16@1` by key;
    /// a region mixing the BF16 multiply and add does not, and under a
    /// contraction-*permitting* contract it stays `unrealized-contraction`. So
    /// the arm was extended by deciding each key rather than by widening to a
    /// role, and the withholding half is intact at this width.
    ///
    /// The permitting contract is asserted to permit before anything rests on it:
    /// a contract that forbids contraction discharges the obligation on its own
    /// and would make both halves vacuous.
    #[test]
    fn the_contraction_arm_reads_the_bf16_keys_and_still_withholds_the_mixed_region() {
        let budgets = DeterministicBudgets::governed();
        let mut permitting = strict_bf16();
        permitting.contraction = NumericalPermission::Permitted;
        assert!(!matches!(
            permitting.contraction,
            NumericalPermission::Forbidden
        ));

        let add_only = bf16_add_only_program();
        let formation = form_region_candidates(&add_only, budgets, permitting).unwrap();
        let candidate = formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();
        let outcome = derive_fusion_legality(
            &add_only,
            budgets,
            permitting,
            &FusionNumericalCapabilities::governed(),
            &formation,
            &candidate,
        )
        .unwrap();
        let FusionLegality::Legal(proof) = outcome else {
            panic!("an add-only bf16 region carries the closed proof, not {outcome:?}");
        };
        let contraction = proof
            .content()
            .obligations()
            .iter()
            .find(|derived| derived.obligation() == FusionObligation::ArithmeticContraction)
            .unwrap();
        assert_eq!(contraction.assessment(), ObligationAssessment::Discharged);
        assert_eq!(contraction.evidence(), FusionEvidenceClass::SoundProof);

        let mixed = bf16_program();
        let mixed_formation = form_region_candidates(&mixed, budgets, permitting).unwrap();
        let mixed_candidate = mixed_formation
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone();
        let FusionLegality::Unknown(unknown) = derive_fusion_legality(
            &mixed,
            budgets,
            permitting,
            &FusionNumericalCapabilities::governed(),
            &mixed_formation,
            &mixed_candidate,
        )
        .unwrap() else {
            panic!("a permitting contract cannot establish the mixed region's contraction");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::ArithmeticContraction
        );
        assert_eq!(unknown.reason(), "unrealized-contraction");
    }

    /// Every governed arithmetic resolves the encoding the derivation reads.
    ///
    /// This is what makes `ungoverned-region-arithmetic` a drift guard rather
    /// than a reachable outcome, and it names and counts its population so a
    /// vocabulary that stopped resolving one width fails here instead of silently
    /// turning a legality question into a compiler fault.
    #[test]
    fn every_governed_arithmetic_resolves_a_region_encoding() {
        let mut resolved = 0_usize;
        for arithmetic in ArithmeticType::ALL {
            assert!(
                registered_arithmetic_value_type(arithmetic).is_some(),
                "{arithmetic:?} resolves no registered value identity",
            );
            resolved += 1;
        }
        assert_eq!(
            resolved,
            ArithmeticType::ALL.len(),
            "the sweep visited fewer widths than the vocabulary names",
        );
        assert_eq!(
            FusionLegalityError::Structure {
                rule: "ungoverned-region-arithmetic",
            }
            .reason(),
            "ungoverned-region-arithmetic",
        );
    }

    /// A BF16 region's obligations are not the `f32` region's, byte for byte.
    ///
    /// Two structurally identical regions — the same operation shape, the same
    /// counts, the same discharged obligations — derived under the two widths'
    /// contracts carry *different* content identities, because the contract key is
    /// part of the content. That is the property that stops a proof stated at one
    /// width from being replayed as evidence at the other.
    #[test]
    fn a_bf16_proof_and_its_f32_counterpart_do_not_share_a_content_identity() {
        use tiler_ir::semantic::{F32, F32Add, F32Constant};

        let budgets = DeterministicBudgets::governed();
        let bf16_contract = strict_bf16();
        let bf16 = bf16_add_only_program();
        let bf16_formation = form_region_candidates(&bf16, budgets, bf16_contract).unwrap();
        let FusionLegality::Legal(bf16_proof) = derive_fusion_legality(
            &bf16,
            budgets,
            bf16_contract,
            &FusionNumericalCapabilities::governed(),
            &bf16_formation,
            bf16_formation.whole_program_candidate().unwrap(),
        )
        .unwrap() else {
            panic!("the bf16 region is legal");
        };

        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let left = builder
            .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let right = builder
            .input::<F32>(InputKey::new("y").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let sum = F32Add::apply(&mut builder, left, right).unwrap();
        let biased = F32Add::apply(&mut builder, sum, bias).unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), biased)
            .unwrap();
        let f32_program = builder.build().unwrap();
        let f32_contract = StrictF32NumericalContract::governed();
        let f32_formation = form_region_candidates(&f32_program, budgets, f32_contract).unwrap();
        let FusionLegality::Legal(f32_proof) = derive_fusion_legality(
            &f32_program,
            budgets,
            f32_contract,
            &FusionNumericalCapabilities::governed(),
            &f32_formation,
            f32_formation.whole_program_candidate().unwrap(),
        )
        .unwrap() else {
            panic!("the f32 region is legal");
        };

        // Same shape, same counts, same obligations — and different identities.
        assert_eq!(
            bf16_proof.content().structure(),
            f32_proof.content().structure(),
            "the two fixtures are not structurally identical, so the identity \
             difference could be the structure's rather than the width's",
        );
        assert_eq!(
            bf16_proof.content().obligations(),
            f32_proof.content().obligations(),
        );
        assert_ne!(
            bf16_proof.content().identity(),
            f32_proof.content().identity(),
            "a proof stated at one width shares its identity with the other's",
        );
    }

    /// The evidence class an obligation carries is derived, not stamped.
    ///
    /// Driven through `derive_obligations` directly so the assertion is about the
    /// derivation rather than about the fixture reaching it.
    #[test]
    fn a_bf16_contract_forbidding_contraction_discharges_by_normative_guarantee() {
        let program = bf16_program();
        let budgets = DeterministicBudgets::governed();
        let contract = strict_bf16();
        assert!(matches!(
            contract.contraction,
            NumericalPermission::Forbidden
        ));
        let formation = form_region_candidates(&program, budgets, contract).unwrap();
        let FusionLegality::Legal(proof) = derive_fusion_legality(
            &program,
            budgets,
            contract,
            &FusionNumericalCapabilities::governed(),
            &formation,
            formation.whole_program_candidate().unwrap(),
        )
        .unwrap() else {
            panic!("a forbidding bf16 contract discharges the mixed region");
        };
        let contraction = proof
            .content()
            .obligations()
            .iter()
            .find(|derived| derived.obligation() == FusionObligation::ArithmeticContraction)
            .unwrap();
        assert_eq!(
            contraction.evidence(),
            FusionEvidenceClass::NormativeGuarantee,
            "the mixed region carried the closed structural proof, so the \
             contract's own resolution was not what discharged it",
        );

        // And the obligation count is nine at this width, from the derivation
        // rather than from a proof that happened to be assembled.
        let members_only = derive_obligations(&[], contract);
        assert_eq!(members_only.len(), 9);
    }
}
