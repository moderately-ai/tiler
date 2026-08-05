//! The borrowed typed delivered-realization evidence of one selected plan.
//!
//! ADR 0076 item 4 names a consumer that compares generated output against a CPU
//! reference and must know an emulated dimension from a natively honoured one.
//! Before this module nothing outside this crate could read one honoured
//! dimension: [`crate::selection::SelectedPlan::honoured`] is `pub(crate)` inside
//! the private `mod selection`, and `ProvenEvidence`,
//! `NumericalHonourabilityFact`, [`HonouringMeans`], and [`DimensionBehaviour`]
//! are `pub(crate)` inside `pub(crate) mod target::{feasibility, honourability}`.
//! So a producer could read contract keys, profile descriptor bytes, and the
//! feasibility rule-set identity — and not one honoured fact.
//!
//! # Borrowed, `Copy`, and constructor-free
//!
//! Every view here exposes no `Arc`, no vector, no constructor, and no canonical
//! encoder. A consumer reads typed selected evidence and cannot forge a
//! compiler-verified fact, which is what keeps `tiler-build`'s translation a
//! transcription of compiler evidence rather than a second producer of it.
//!
//! It is deliberately **one** view rather than three iterators a caller zips
//! itself, because the total boundary has to cross-check policy subjects,
//! all-dimension coverage, obligation associations, and the evidence pool
//! together, and three iterators can be zipped wrongly.
//!
//! # What today's compiler can honestly say about a locus
//!
//! **Fact.** `StrictF32NumericalContract` is one flat record for one arithmetic
//! type, and [`crate::policy::dimension_requirements`] projects it into one
//! **whole-program** requirement per consumable dimension. There is no
//! per-occurrence, per-accumulator, or per-materialization numerical requirement
//! anywhere on the compile path; the exact check is that
//! `crate::policy::dimension_requirements` takes a contract and no occurrence,
//! and that the honoured facts a plan retains are aggregated across regions and
//! deduplicated by canonical key before any occurrence is known.
//!
//! **Consequence, and why the locus is still keyed.** ADR 0011's per-operation
//! restrictions attach to a position, so the record's shape is right and the
//! producer for it does not exist. Until it does, this view states one obligation
//! per honoured dimension at [`PolicyLocus::Computation`] of **every** occurrence
//! the packaged program covers. That over-states which occurrences consume a
//! dimension and never under-states it, which is the safe direction: an extra
//! obligation carries real evidence and demands more of a comparing consumer,
//! while a missing one would let a dimension's disposition be derived as
//! `NotRequired` — the one producer assertion the neutral artifact cannot check.
//! `derive-per-locus-numerical-obligations` owns narrowing it.

use tiler_ir::numerics::{
    DIMENSION_COUNT, DimensionBehaviour, FactSourceProvenance, HonouringMeans, NumericalDimension,
    NumericalObligationKey, PolicyLocus, ScalarArithmeticSubject,
};
use tiler_ir::program::SemanticOccurrence;

use crate::program::{KernelProgram, ProgramError};
use crate::request::StrictF32NumericalContract;
use crate::selection::SelectedPlan;

/// One compiler-selected obligation, borrowed from the owning `Compilation`.
///
/// Copyable and borrowed rather than owned, and with no public constructor: a
/// consumer can read typed selected evidence without being able to mint a fact
/// no compiler produced.
#[derive(Clone, Copy, Debug)]
pub struct SelectedObligation<'a> {
    subject: &'a ScalarArithmeticSubject,
    dimension: NumericalDimension,
    locus: NumericalObligationKey,
    required: DimensionBehaviour,
    evidence: SelectedEvidence<'a>,
}

impl<'a> SelectedObligation<'a> {
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
    /// The behaviour the declaring target speaks about.
    #[must_use]
    pub const fn declared(self) -> DimensionBehaviour {
        self.declared
    }

    /// The structured means, relaxation payload included.
    ///
    /// Borrowed structurally rather than rendered to a key, which is the whole
    /// correction ADR 0076 records: [`HonouringMeans::label`] collapses every
    /// declared relaxation to one string, so a consumer reading the label cannot
    /// tell two conditional means apart.
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
    ///
    /// The descriptor is the compilation's assessed profile descriptor rather
    /// than a second copy carried per fact, and the two name one profile: the
    /// compiler refuses a retained fact whose declaring key is not the assessed
    /// profile's, so a fact reaching a reader here was declared by the profile
    /// these bytes describe.
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
/// Sited on a plan rather than on the compilation because a realization
/// qualifies one selected plan: two retained alternatives of one compilation can
/// rest on different honoured facts.
#[derive(Clone, Copy, Debug)]
pub struct DeliveredRealizationView<'a> {
    profile_key: &'a str,
    profile_descriptor: &'a [u8],
    evidence: &'a DeliveredRealizationEvidence,
}

impl<'a> DeliveredRealizationView<'a> {
    pub(crate) const fn new(
        profile_key: &'a str,
        profile_descriptor: &'a [u8],
        evidence: &'a DeliveredRealizationEvidence,
    ) -> Self {
        Self {
            profile_key,
            profile_descriptor,
            evidence,
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
    /// dimension is unrequired, and a recognized semantic type that merely
    /// appears in the program creates no subject at all.
    #[must_use]
    pub fn scalar_arithmetic(self) -> impl ExactSizeIterator<Item = SelectedScalarArithmetic<'a>> {
        self.evidence
            .subjects
            .iter()
            .map(|row| SelectedScalarArithmetic {
                subject: &row.subject,
                resolutions: &row.resolutions,
            })
    }

    /// The canonical union of obligations every packaged variant and stage that
    /// routing may select relies on.
    ///
    /// Never "actually exercised": the artifact exists before a route executes,
    /// so the only honest quantifier is over what was packaged.
    #[must_use]
    pub fn obligations(self) -> impl ExactSizeIterator<Item = SelectedObligation<'a>> {
        self.evidence.obligations.iter().map(move |row| {
            SelectedObligation {
                subject: &self.evidence.subjects[row.subject].subject,
                dimension: row.dimension,
                locus: row.locus,
                required: row.required,
                evidence: SelectedEvidence {
                    declared: row.declared,
                    means: &row.means,
                    profile_key: &row.profile_key,
                    // One profile per compilation, proven at materialization
                    // rather than assumed; see `SelectedEvidence::profile_descriptor`.
                    profile_descriptor: self.profile_descriptor,
                    source: &row.source,
                },
            }
        })
    }
}

/// One materialized policy subject and its dense eleven-dimension contract.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectedSubjectRow {
    subject: ScalarArithmeticSubject,
    resolutions: [DimensionBehaviour; DIMENSION_COUNT],
}

/// One materialized obligation row, owned by the retained alternative.
///
/// Owned rather than borrowed because the view a consumer reads borrows *from*
/// this: a slice of `SelectedObligation<'a>` stored beside the facts it points
/// at would be self-referential, and the borrowed view is what makes the
/// evidence unforgeable in the first place.
///
/// `required` and `declared` are kept apart even though today's resolution rule
/// makes them one value — a fact is matched to a requirement *by* the required
/// behaviour, so `HonouredDimension::behaviour` and its fact's behaviour are the
/// same field. They answer different questions, and the artifact record carries
/// both and cross-checks them; collapsing them here would make a change to the
/// resolution rule silently misreport the caller's contract.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectedObligationRow {
    subject: usize,
    dimension: NumericalDimension,
    locus: NumericalObligationKey,
    required: DimensionBehaviour,
    declared: DimensionBehaviour,
    means: HonouringMeans,
    profile_key: String,
    source: FactSourceProvenance,
}

/// The dense subject and sparse obligation tables one retained alternative owns.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeliveredRealizationEvidence {
    subjects: Vec<SelectedSubjectRow>,
    obligations: Vec<SelectedObligationRow>,
}

impl DeliveredRealizationEvidence {
    /// Materializes one retained plan's delivered-realization evidence.
    ///
    /// The subject table is the *contract's* subject rather than the union of
    /// the facts' subjects, and a retained fact naming another subject or another
    /// declaring profile is refused rather than absorbed. Both refusals are the
    /// fail-closed direction for the same reason: an obligation dropped here
    /// would let this dimension's disposition be derived as unrequired by the
    /// artifact builder, and a producer-asserted "no packaged route requires
    /// this" is the one claim the neutral artifact cannot re-check.
    ///
    /// # Errors
    ///
    /// Returns a structural [`ProgramError`] when the contract's arithmetic type
    /// resolves no governed scalar subject, or when a retained honoured fact
    /// names a subject or a declaring profile other than the assessed one.
    pub(crate) fn materialize(
        contract: &StrictF32NumericalContract,
        plan: &SelectedPlan,
        program: &KernelProgram,
        profile_key: &str,
    ) -> Result<Self, ProgramError> {
        let subject = crate::policy::arithmetic_subject(contract.arithmetic).ok_or(
            ProgramError::Structure {
                rule: "numerical-realization-subject-unvalidated",
            },
        )?;
        let mut resolutions =
            [DimensionBehaviour::Transform(tiler_ir::schedule::NumericalPermission::Forbidden);
                DIMENSION_COUNT];
        for dimension in tiler_ir::numerics::CANONICAL_DIMENSIONS {
            resolutions[dimension.index()] = contract.behaviour(dimension);
        }
        let subjects = vec![SelectedSubjectRow {
            subject: subject.clone(),
            resolutions,
        }];

        // Every occurrence the packaged program covers, in canonical ascending
        // order and deduplicated: a graph-local ordinal repeated across stages
        // is one position, not two.
        let mut occurrences: Vec<SemanticOccurrence> = program
            .core()
            .stages()
            .flat_map(|stage| {
                stage
                    .coverage()
                    .iter()
                    .map(tiler_ir::program::CoveredOccurrence::occurrence)
                    .collect::<Vec<_>>()
            })
            .collect();
        occurrences.sort_unstable();
        occurrences.dedup();

        let mut obligations = Vec::new();
        for honoured in plan.honoured() {
            if honoured.arithmetic() != subject.arithmetic()
                || honoured.resolved_type() != subject.resolved_type()
            {
                return Err(ProgramError::Structure {
                    rule: "numerical-realization-subject-unvalidated",
                });
            }
            if honoured.profile().key() != profile_key {
                return Err(ProgramError::Structure {
                    rule: "numerical-realization-profile-unattributed",
                });
            }
            for occurrence in &occurrences {
                obligations.push(SelectedObligationRow {
                    subject: 0,
                    dimension: honoured.dimension(),
                    locus: NumericalObligationKey::new(*occurrence, PolicyLocus::Computation),
                    required: honoured.behaviour(),
                    declared: honoured.fact().behaviour(),
                    means: honoured.means(),
                    profile_key: honoured.profile().key().to_owned(),
                    source: honoured.fact().source().clone(),
                });
            }
        }

        Ok(Self {
            subjects,
            obligations,
        })
    }
}
