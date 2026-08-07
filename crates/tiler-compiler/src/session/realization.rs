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
//! # What founds the locus an obligation names
//!
//! **Fact.** `StrictF32NumericalContract` is one flat record for one arithmetic
//! type, and [`crate::policy::dimension_requirements`] projects it into one
//! **whole-program** requirement per consumable dimension. That projection is
//! the dtype-wide *ceiling*, and it takes no occurrence.
//!
//! **The obligations are a separate statement, and this is where the position
//! comes from.** ADR 0011's per-operation restrictions attach to a position, so
//! an obligation names one, and it names one this build can found rather than
//! one the enum happens to offer. Two authorities meet here and neither alone is
//! enough: the packaged program's proof-derived coverage says which occurrences
//! exist, and the resolved lowering says which semantic operation realizes each.
//! With the operation in hand, [`crate::policy::OperationNumericalCapability`]
//! decides both whether the freedom acts at that occurrence at all and, through
//! `founded_locus`, where in it — an operand read, the operation's own
//! arithmetic, or a contributor fold's accumulator.
//!
//! So this view states one obligation per `(subject, dimension, occurrence,
//! locus)` that a packaged route relies on. An occurrence whose operation cannot
//! consume a dimension contributes no row for it: a constant retains its bit
//! pattern and a broadcast moves elements, so neither has a position where a
//! rounding or ordering freedom could act, and a row claiming otherwise would be
//! an unfounded assertion carrying real target evidence.
//!
//! **That narrows loci, never dispositions.** The one direction that would be
//! unsafe is dropping a dimension entirely, because the artifact builder derives
//! a dimension with no obligations as `NotRequired` — the single producer
//! assertion the neutral artifact cannot re-check. A dimension some covered
//! occurrence consumes still carries its rows, relocated to the position that
//! founds them; a dimension *no* covered occurrence consumes is genuinely not
//! required by any packaged route, which is the claim `NotRequired` makes.
//!
//! # Two rules the producer enforces rather than assumes
//!
//! A locus obligation must be **at least as strict as the ceiling**, checked row
//! by row through [`crate::policy::is_at_least_as_strict_as`]. A position may
//! demand more than the program-wide contract and may never demand less. Today's
//! resolution rule makes the two equal, so the check cannot fire on this build's
//! own path — which is exactly why it is a check: a change to that rule would
//! otherwise ship a route resting on a freedom the caller never granted.
//!
//! And a dimension an operation consumes must have a **founded** position. When
//! `founded_locus` yields none the producer refuses by name rather than
//! substituting [`tiler_ir::numerics::PolicyLocus::Computation`], so admitting a capability row for
//! a dimension whose position nobody has sited is a typed failure instead of a
//! plausible-looking row.

use tiler_ir::numerics::{
    DIMENSION_COUNT, DimensionBehaviour, FactSourceProvenance, HonouringMeans, NumericalDimension,
    NumericalObligationKey, ScalarArithmeticSubject,
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
    /// resolves no governed scalar subject, when a retained honoured fact names
    /// a subject or a declaring profile other than the assessed one, when a
    /// packaged occurrence has no resolved lowering to name its operation, when
    /// an operation consumes a dimension this build founds no locus for, or when
    /// a derived obligation would be weaker than the dtype-wide ceiling.
    pub(crate) fn materialize(
        contract: &StrictF32NumericalContract,
        plan: &SelectedPlan,
        program: &KernelProgram,
        lowering: &crate::lowering::ResolvedLowering,
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
        // is one position, not two. The *packaged* program stays the authority
        // on which positions exist — the lowering below only says what operation
        // sits at each, and a resolved lowering the program did not package must
        // not add a position to this set.
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

        // The operation realized at each packaged occurrence, read from the one
        // record that holds both halves. A coverage record names a
        // `SemanticOccurrence` and nothing else, and the canonical ordinal
        // cannot be inverted against the graph from here, so the join goes
        // through the refinement receipt that minted the ordinal in the first
        // place — occurrence and operation therefore come from one proof rather
        // than from two structures a caller could pair wrongly.
        let mut operations: Vec<(SemanticOccurrence, &tiler_ir::semantic::OpKey)> = lowering
            .occurrences()
            .iter()
            .map(|occurrence| match occurrence.evidence() {
                crate::lowering::OccurrenceEvidence::Refined(refinement) => (
                    refinement.receipt().occurrence(),
                    refinement.content().operation(),
                ),
            })
            .collect();
        operations.sort_unstable_by_key(|(occurrence, _)| *occurrence);

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
            let dimension = honoured.dimension();
            // The dtype-wide ceiling this dimension resolves to, read from the
            // caller's contract rather than from the fact, because the rule
            // below is precisely a comparison between the two statements.
            let ceiling = contract.behaviour(dimension);
            for occurrence in &occurrences {
                let operation = operations
                    .binary_search_by_key(occurrence, |(covered, _)| *covered)
                    .map(|position| operations[position].1)
                    .map_err(|_| ProgramError::Structure {
                        rule: "numerical-realization-occurrence-unlowered",
                    })?;
                // A capability row is what says this operation can consume the
                // dimension at all. An occurrence whose operation cannot — a
                // constant, a reindex, a broadcast — has no position where the
                // freedom acts, so it contributes no row rather than a
                // `Computation` one asserting a place nothing founds. The
                // dimension itself is still carried by whichever occurrences do
                // consume it, so this narrows a locus and never a disposition.
                let Some(capability) = crate::policy::operation_capability(operation) else {
                    continue;
                };
                if !capability.can_consume(dimension) {
                    continue;
                }
                let Some(locus) = capability.founded_locus(dimension) else {
                    return Err(ProgramError::Structure {
                        rule: "numerical-realization-locus-unfounded",
                    });
                };
                let required = honoured.behaviour();
                // A locus may demand more than the program-wide contract and
                // never less. Checked here rather than argued from the fact that
                // today's resolution rule makes the two equal: the rule is what
                // the record's readers rely on, and a resolution change that
                // broke it would otherwise ship a route resting on a freedom the
                // caller never granted, carrying real evidence for it.
                if !crate::policy::is_at_least_as_strict_as(required, ceiling) {
                    return Err(ProgramError::Structure {
                        rule: "numerical-realization-locus-weaker-than-ceiling",
                    });
                }
                obligations.push(SelectedObligationRow {
                    subject: 0,
                    dimension,
                    locus: NumericalObligationKey::new(*occurrence, locus),
                    required,
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
