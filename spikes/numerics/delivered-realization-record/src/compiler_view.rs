//! **Proposed `tiler_compiler::session`** — the borrowed typed evidence view.
//!
//! # Nothing like this exists today
//!
//! Exact check, reproducible in one line from the repository root:
//! `grep -rln "HonouredNumericalFact" --include="*.rs" .` returns **no files**.
//! `carry-the-honourability-fact-provenance-into-the-artifact-record` drafted the
//! borrowed facade in its own prose and landed the private carry only, so the
//! view below is a proposal, not a description.
//!
//! What the compiler has today, and where: `SelectedPlan::honoured` returns
//! `&[HonouredDimension]`, and both are `pub(crate)` inside the **private** module
//! `mod selection;`. `ProvenEvidence`, `NumericalHonourabilityFact`,
//! `HonouringMeans`, and `DimensionBehaviour` are likewise `pub(crate)` inside the
//! `pub(crate) mod target::{feasibility, honourability}`. So `tiler-build` can
//! read contract keys, the profile descriptor bytes, and the feasibility rule-set
//! identity — and cannot read one honoured dimension.
//!
//! # The gap this view cannot close on its own, stated plainly
//!
//! **Fact.** `grep -rni "locus" --include="*.rs" crates/` returns nothing. There
//! is no per-locus, per-occurrence, or per-accumulator numerical requirement
//! anywhere in the compiler: `StrictF32NumericalContract` is one flat record for
//! one arithmetic type, and `policy::dimension_requirements` projects it into
//! exactly eight **whole-program** `NumericalRequirement`s.
//!
//! **Inference.** The obligation rows this record is shaped to carry cannot be
//! produced by today's compiler. This is a genuine prerequisite rather than a
//! detail of wiring, and it is recorded here rather than papered over: the shape
//! is right — ADR 0011's per-operation restrictions are per-position and a
//! dtype-wide ceiling genuinely cannot express two `f32` loci with different legal
//! requirements — and the producer does not exist yet. Until it does, a
//! conforming producer emits one obligation per consumable dimension at
//! [`PolicyLocus::Computation`] of the occurrence that consumes it, which is
//! exactly as much as the compiler can honestly say. `derive-per-locus-numerical-obligations`
//! is filed for the remainder.
//!
//! **A second observed defect, out of scope and filed.**
//! `policy::dimension_requirements` builds every requirement with a hard-coded
//! `F32::resolved_type()` while reading `contract.arithmetic` from the contract,
//! so an `f16` contract produces requirements whose resolved type is
//! `tiler::f32@1`. The pair never validates, so the outcome is `Unknown` and
//! fails closed rather than admitting anything wrong — which is why this is a
//! correctness-preserving defect and not a live bug — but it means no non-`f32`
//! contract can ever be honoured, whatever a profile declares.

use crate::shared::{
    DIMENSION_COUNT, DimensionBehaviour, FactSourceProvenance, HonouringMeans, NumericalDimension,
    NumericalObligationKey, ScalarArithmeticSubject,
};

/// One compiler-selected obligation, borrowed from the owning `Compilation`.
///
/// Copyable and borrowed rather than owned: the facade exposes no `Arc`, no
/// vector, no constructor, and no canonical encoder, so a consumer can read
/// typed selected evidence without being able to forge a compiler-verified fact.
#[derive(Clone, Copy, Debug)]
pub struct SelectedObligation<'a> {
    subject: &'a ScalarArithmeticSubject,
    dimension: NumericalDimension,
    locus: NumericalObligationKey,
    required: DimensionBehaviour,
    evidence: SelectedEvidence<'a>,
}

impl<'a> SelectedObligation<'a> {
    /// Borrows one selected obligation.
    ///
    /// Crate-visible in the spike because the model has to construct one; the
    /// production spelling has no public constructor at all, which is what makes
    /// the view unforgeable.
    pub(crate) const fn new(
        subject: &'a ScalarArithmeticSubject,
        dimension: NumericalDimension,
        locus: NumericalObligationKey,
        required: DimensionBehaviour,
        evidence: SelectedEvidence<'a>,
    ) -> Self {
        Self {
            subject,
            dimension,
            locus,
            required,
            evidence,
        }
    }

    /// The checked policy subject this obligation is stated for.
    #[must_use]
    pub const fn subject(self) -> &'a ScalarArithmeticSubject {
        self.subject
    }

    /// The dimension this obligation is stated on.
    #[must_use]
    pub const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The program occurrence and policy locus that produced it.
    #[must_use]
    pub const fn locus(self) -> NumericalObligationKey {
        self.locus
    }

    /// The behaviour this locus requires.
    #[must_use]
    pub const fn required(self) -> DimensionBehaviour {
        self.required
    }

    /// The exact checked fact that honours it.
    #[must_use]
    pub const fn evidence(self) -> SelectedEvidence<'a> {
        self.evidence
    }
}

/// One checked target fact, borrowed with its complete structured provenance.
#[derive(Clone, Copy, Debug)]
pub struct SelectedEvidence<'a> {
    declared: DimensionBehaviour,
    means: &'a HonouringMeans,
    profile_key: &'a str,
    profile_descriptor: &'a [u8],
    source: &'a FactSourceProvenance,
}

impl<'a> SelectedEvidence<'a> {
    /// Borrows one checked fact.
    pub(crate) const fn new(
        declared: DimensionBehaviour,
        means: &'a HonouringMeans,
        profile_key: &'a str,
        profile_descriptor: &'a [u8],
        source: &'a FactSourceProvenance,
    ) -> Self {
        Self {
            declared,
            means,
            profile_key,
            profile_descriptor,
            source,
        }
    }

    /// The behaviour the declaring target speaks about.
    #[must_use]
    pub const fn declared(self) -> DimensionBehaviour {
        self.declared
    }

    /// The structured means, relaxation payload included.
    ///
    /// Borrowed structurally rather than rendered to a key, which is the whole
    /// correction: `HonouringMeans::label` collapses every declared relaxation to
    /// one string, so a consumer reading the label cannot tell two conditional
    /// means apart.
    #[must_use]
    pub const fn means(self) -> &'a HonouringMeans {
        self.means
    }

    /// The declaring profile's governed key.
    #[must_use]
    pub const fn profile_key(self) -> &'a str {
        self.profile_key
    }

    /// The declaring profile's exact canonical descriptor bytes.
    #[must_use]
    pub const fn profile_descriptor(self) -> &'a [u8] {
        self.profile_descriptor
    }

    /// The complete structured provenance the declaring authority supplied.
    #[must_use]
    pub const fn source(self) -> &'a FactSourceProvenance {
        self.source
    }
}

/// One compiler-produced scalar-arithmetic policy subject and its contract.
#[derive(Clone, Copy, Debug)]
pub struct SelectedScalarArithmetic<'a> {
    subject: &'a ScalarArithmeticSubject,
    resolutions: &'a [DimensionBehaviour; DIMENSION_COUNT],
}

impl<'a> SelectedScalarArithmetic<'a> {
    /// Borrows one selected contract.
    pub(crate) const fn new(
        subject: &'a ScalarArithmeticSubject,
        resolutions: &'a [DimensionBehaviour; DIMENSION_COUNT],
    ) -> Self {
        Self {
            subject,
            resolutions,
        }
    }

    /// The checked policy subject.
    #[must_use]
    pub const fn subject(self) -> &'a ScalarArithmeticSubject {
        self.subject
    }

    /// The resolved behaviour of one dimension. Total over all eleven.
    #[must_use]
    pub const fn resolution(self, dimension: NumericalDimension) -> DimensionBehaviour {
        self.resolutions[dimension.index()]
    }

    /// The dense resolution array, for a translator that walks it whole.
    #[must_use]
    pub const fn resolutions(self) -> &'a [DimensionBehaviour; DIMENSION_COUNT] {
        self.resolutions
    }
}

/// The complete delivered-realization view of one selected plan.
///
/// Proposed as `PlanAlternative::delivered_realization`, because a realization
/// qualifies one selected plan rather than the compilation as a whole. It is
/// deliberately **one** view rather than three independent iterators: the total
/// boundary must cross-check policy subjects, all-dimension coverage, obligation
/// and locus associations, and the evidence pool *together*, and three iterators
/// a caller zips itself can be zipped wrongly.
#[derive(Clone, Copy, Debug)]
pub struct DeliveredRealizationView<'a> {
    profile_key: &'a str,
    profile_descriptor: &'a [u8],
    subjects: &'a [(
        ScalarArithmeticSubject,
        [DimensionBehaviour; DIMENSION_COUNT],
    )],
    obligations: &'a [SelectedObligation<'a>],
}

impl<'a> DeliveredRealizationView<'a> {
    /// Borrows one plan's complete delivered-realization evidence.
    pub(crate) const fn new(
        profile_key: &'a str,
        profile_descriptor: &'a [u8],
        subjects: &'a [(
            ScalarArithmeticSubject,
            [DimensionBehaviour; DIMENSION_COUNT],
        )],
        obligations: &'a [SelectedObligation<'a>],
    ) -> Self {
        Self {
            profile_key,
            profile_descriptor,
            subjects,
            obligations,
        }
    }

    /// The declaring profile's governed key.
    #[must_use]
    pub const fn profile_key(self) -> &'a str {
        self.profile_key
    }

    /// The declaring profile's exact canonical descriptor bytes.
    #[must_use]
    pub const fn profile_descriptor(self) -> &'a [u8] {
        self.profile_descriptor
    }

    /// Every compiler-produced policy subject with its complete contract.
    ///
    /// A subject exists because the checked request selected a governed scalar
    /// contract, **not** because obligations were found for it: a selected
    /// contract yields one complete eleven-dimension subject even when every
    /// dimension is `NotRequired`, and a recognized semantic type that merely
    /// appears in the program creates no subject at all.
    pub fn scalar_arithmetic(self) -> impl ExactSizeIterator<Item = SelectedScalarArithmetic<'a>> {
        self.subjects
            .iter()
            .map(|(subject, resolutions)| SelectedScalarArithmetic::new(subject, resolutions))
    }

    /// The canonical union of obligations every packaged variant and stage that
    /// routing may select relies on.
    ///
    /// Never "actually exercised": the artifact exists before a route executes,
    /// so the only honest quantifier is over what was packaged.
    pub fn obligations(self) -> impl ExactSizeIterator<Item = SelectedObligation<'a>> {
        self.obligations.iter().copied()
    }
}
