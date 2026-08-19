//! The deterministic budgets one compilation request is bound to.
//!
//! The resource vocabulary every internal stop record maps into, how a refusal
//! on each produced its compared number, the budget set itself, and the
//! comparison every authority reports through. Every field is written into the
//! canonical request subject, so a value here is request identity rather than a
//! tuning knob.

use super::*;

/// How the compared number on a budget refusal was produced.
///
/// A caller acting on an exhausted budget needs the number's provenance, not
/// only its magnitude. An exact completed count, a conservative planning
/// envelope, a truncated-search lower bound, and a stopped-construction lower
/// bound imply different actions, and reading any of them as a uniform
/// "actual" is wrong in the silent direction for the other three.
///
/// # Not `#[non_exhaustive]`
///
/// ADR 0074 convention 5a marks a public enum whose variant set is a
/// bounded-profile placeholder. This is not one: the four provenances are a
/// closed report vocabulary, and `tiler-macros` maps every case totally to
/// caller advice. A genuinely new meaning adds a variant and breaks every
/// total owner until that consumer classifies and renders it. Marking the enum
/// `#[non_exhaustive]` would oblige those owners to carry a wildcard arm over
/// a closed split, which is the cost 5a exists to avoid paying for nothing.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BudgetRefusal {
    /// The compiler finished counting the subject and compared that count.
    ///
    /// The reported value is a submitted program's own values or occurrences,
    /// or a refused region candidate's members, retained outputs, or live
    /// values. Where one resource refused several candidates, it is the
    /// largest of those exact counts. No additional search reaches a plan
    /// under this limit.
    ExactDemand,
    /// The compiler compared a conservative envelope computed before a plan
    /// was chosen.
    ///
    /// The reported value is an upper bound over every plan the request could
    /// reach, not one plan's demand. A particular reachable plan may use less.
    /// No later search runs after the request gate refuses one, but that does
    /// not make the envelope an exact requirement.
    PlanningUpperBound,
    /// The compiler stopped a search before that search finished.
    ///
    /// The reported value is the first demand the limit refused, which is a
    /// lower bound on the space left unexplored rather than the budget a
    /// successful plan requires. A wider limit may reach a plan this
    /// compilation never saw, and may equally find nothing.
    SearchLowerBound,
    /// Explain construction stopped at the first detail it could not retain.
    ///
    /// The reported value is the exact attempted retained prefix including
    /// that refused detail: records retained plus one for the record arm, or
    /// canonical detail bytes retained plus that detail's encoded bytes for
    /// the byte arm. Construction stopped there, so the value is only a lower
    /// bound on the complete trace and never the capacity required for success.
    ConstructionLowerBound,
}

/// Which deterministic budget refused a compilation.
///
/// Five authorities raise a budget refusal. Four own request or planning stop
/// records: `request::check_program_budgets` refuses a submitted program's own
/// size before any target is consulted, and `region::RegionBudgetResource`,
/// `cover::CoverBudgetResource`, and `selection::PlanBudgetResource` bound the
/// three searches that run once one has been. The explain writer owns the two
/// report-only construction resources added at the end of this enumeration.
/// They are existing hard build constants, not fields of
/// `DeterministicBudgets`, and therefore do not enter request or evidence
/// identity. These authorities are named as plain text because each is
/// crate-private and a public doc cannot link a private item.
///
/// The stop records stay distinct because their surrounding data differs — a
/// plan stop also names the cover whose enumeration it stopped. This is the
/// single vocabulary they all name a resource in, so a caller reads one set
/// rather than five, and each request/planning authority maps into it through a
/// total `const fn` that a new internal budget must extend before it compiles.
///
/// [`Self::key`] is the stable diagnostic key, and it is the sole authority for
/// these strings: the per-authority accessors delegate here rather than
/// repeating a table that could drift.
///
/// # Which of these a public caller can actually observe
///
/// The five program-scoped resources and the two report-only explain resources.
/// The other eight are raised by the three searches, which reach a caller only
/// through an empty portfolio, and `crate::session`'s reachability note records
/// why that route is currently unreachable from the public surface: the
/// region-shape bounds are now the same formulas as the program-scoped bounds
/// they derive from, so a program large enough to truncate a search is refused
/// for its size first. Reaching it needs a caller-stated budget set, which the
/// public surface does not admit. The vocabulary is nevertheless complete,
/// because the mapping into it must be total over what the compiler can raise
/// and reachability is a property of the authorities rather than of this type.
///
/// # Why `#[non_exhaustive]`
///
/// ADR 0074 convention 5's clause test asks what an out-of-crate wildcard arm
/// would have to do. No consumer outside this crate maps this vocabulary onto a
/// derived value it must get right per variant, and none matches it to decide
/// what it supports; a consumer renders it, forwards it, or classifies it
/// partially. That is clause 5a, so the attribute applies and a later budget
/// lands additively.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum BudgetResource {
    /// Values a submitted program may declare and produce.
    SemanticValues,
    /// Semantic occurrences a submitted program may declare.
    SemanticOperations,
    /// Dispatch regions the widest plan for a submitted program may assemble.
    Regions,
    /// Host expression nodes the widest plan for a submitted program may spell.
    HostExpressionNodes,
    /// Buffers the widest plan for a submitted program may bind.
    Buffers,
    /// Semantic occurrences admitted in one region candidate.
    RegionMembers,
    /// Retained boundary outputs admitted for one region candidate.
    RegionBoundaryOutputs,
    /// Boundary and member-result values live across one region candidate.
    RegionLiveValues,
    /// Grown candidates admitted for one seed occurrence.
    RegionCandidatesPerSeed,
    /// Candidate expansion attempts admitted for one compilation request.
    RegionExpansions,
    /// Distinct legal complete covers retained for one enumeration request.
    RegionCovers,
    /// Partition-search expansion attempts for one enumeration request.
    RegionCoverExpansions,
    /// Complete-plan combinations admitted for one cover source.
    PhysicalPlanCombinations,
    /// Detail records the explain writer could retain while constructing a trace.
    ///
    /// Report-only: this is a hard build constant, not a caller-stated or
    /// request-identity-bearing `DeterministicBudgets` field.
    ExplainDetailRecords,
    /// Canonical detail bytes the explain writer could retain while constructing a trace.
    ///
    /// Report-only: this is a hard build constant, not a caller-stated or
    /// request-identity-bearing `DeterministicBudgets` field.
    ExplainDetailCanonicalBytes,
}

impl BudgetResource {
    /// Every budget resource, sized from the type.
    ///
    /// `variant_count` is what makes a widened vocabulary a build error here
    /// rather than a census that silently shrinks while still reporting no
    /// duplicate key. A hand-written length would be satisfied by a list that
    /// had stopped covering its own enum.
    ///
    /// Test-only, so the nightly feature it needs stays out of a normal build.
    #[cfg(test)]
    pub(crate) const ALL: [Self; std::mem::variant_count::<Self>()] = [
        Self::SemanticValues,
        Self::SemanticOperations,
        Self::Regions,
        Self::HostExpressionNodes,
        Self::Buffers,
        Self::RegionMembers,
        Self::RegionBoundaryOutputs,
        Self::RegionLiveValues,
        Self::RegionCandidatesPerSeed,
        Self::RegionExpansions,
        Self::RegionCovers,
        Self::RegionCoverExpansions,
        Self::PhysicalPlanCombinations,
        Self::ExplainDetailRecords,
        Self::ExplainDetailCanonicalBytes,
    ];

    /// Returns the stable diagnostic key of this budget.
    ///
    /// The key is meaning rather than presentation: it is the rule key a
    /// request refusal reports, the resource key an explain record carries, and
    /// part of the reason code a failure detail spells, so it is compared. ADR
    /// 0074 convention 2's correction is what decides the spelling — `key` is
    /// reserved for a stable semantic key and `label` for a presentation digest.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::SemanticValues => "semantic-values",
            Self::SemanticOperations => "semantic-operations",
            Self::Regions => "regions",
            Self::HostExpressionNodes => "host-expression-nodes",
            Self::Buffers => "buffers",
            Self::RegionMembers => "region-members",
            Self::RegionBoundaryOutputs => "region-boundary-outputs",
            Self::RegionLiveValues => "region-live-values",
            Self::RegionCandidatesPerSeed => "region-candidates-per-seed",
            Self::RegionExpansions => "region-expansions",
            Self::RegionCovers => "region-covers",
            Self::RegionCoverExpansions => "region-cover-expansions",
            Self::PhysicalPlanCombinations => "physical-plan-combinations",
            Self::ExplainDetailRecords => "explain-detail-records",
            Self::ExplainDetailCanonicalBytes => "explain-detail-canonical-bytes",
        }
    }

    /// Returns how a refusal on this budget produced the compared number.
    ///
    /// This is the sole authority for report kind. Categories are defined by
    /// provenance, not by whether the number can be described abstractly as a
    /// bound: an exact completed count is mathematically both an upper and a
    /// lower bound, so the durable split is completed exact demand, a
    /// conservative planning envelope computed before selection, a lower bound
    /// recorded where enumeration stopped, and an attempted-prefix lower bound
    /// recorded where explain construction stopped.
    ///
    /// The five search bounds stop an enumeration at the first demand they
    /// refuse, and all three stop records say so in their own documentation:
    /// the value is a lower bound on the unexplored space rather than its size.
    /// The three request-gate planning envelopes (`Regions`,
    /// `HostExpressionNodes`, `Buffers`) are computed before a plan is chosen
    /// and may exceed what a particular reachable plan uses. The remaining
    /// five are completed counts of a submitted program or of one refused
    /// region candidate. The two report-only explain resources are exact
    /// attempted prefixes but lower bounds on the complete trace construction
    /// stopped before producing.
    ///
    /// This is the answer a `&'static str` resource could not give. A caller
    /// holding a key has no way to learn the number's provenance without
    /// reading compiler source, which is the reading this whole surface exists
    /// to remove.
    #[must_use]
    pub const fn refusal(self) -> BudgetRefusal {
        match self {
            Self::SemanticValues
            | Self::SemanticOperations
            | Self::RegionMembers
            | Self::RegionBoundaryOutputs
            | Self::RegionLiveValues => BudgetRefusal::ExactDemand,
            Self::Regions | Self::HostExpressionNodes | Self::Buffers => {
                BudgetRefusal::PlanningUpperBound
            }
            Self::RegionCandidatesPerSeed
            | Self::RegionExpansions
            | Self::RegionCovers
            | Self::RegionCoverExpansions
            | Self::PhysicalPlanCombinations => BudgetRefusal::SearchLowerBound,
            Self::ExplainDetailRecords | Self::ExplainDetailCanonicalBytes => {
                BudgetRefusal::ConstructionLowerBound
            }
        }
    }
}

/// Every deterministic budget one compilation request is bound to.
///
/// # The rule: a budget is one of exactly two things, and it must say which
///
/// **An authoring-side derivation names its formula and the declaration the
/// formula is over.** [`Self::governed`] carries both for each of the eight
/// derived fields, so a reader asks that function why a number is what it is and
/// never has to guess. It is a derivation somebody re-runs when the owning
/// declaration moves, *not* a computation this compiler performs: `governed` is
/// a nullary `const fn` returning integer literals, nothing here reads a
/// submitted program, and no field is tracked or recomputed while a request
/// compiles. A derived bound tracks its declaration only for as long as the next
/// author re-derives it.
///
/// **A literal policy cap names the unit of work it counts, what its stop
/// removes, and its evidence status.** All six literal fields are *uncalibrated
/// deterministic policy literals*: the source states what each bounds and what
/// its stop preserves, and no accepted decision, retained measurement, or source
/// derivation establishes why any of the numbers is exactly the one written.
/// That is what they may be called, and it is all they may be called — a literal
/// must never be relabelled as derived or as measured, and choosing or changing
/// one is its own measurement decision with its own identity accounting.
///
/// Both kinds are equally binding at compile time. The distinction is about
/// *provenance*, not about strength.
///
/// # The classification, total over the fourteen fields
///
/// Derived (eight): `semantic_values`, `semantic_operations`, `regions`,
/// `host_expression_nodes`, and `buffers` are formulas over the governed
/// profile's widest admitted program declaration; `region_members`,
/// `region_boundary_outputs`, and `region_live_values` are formulas over those
/// five, because a region is a subset of the program it covers.
///
/// Literal (six): `normalization_rewrites`, `region_candidates_per_seed`,
/// `region_expansions`, `region_covers`, `region_cover_expansions`, and
/// `physical_plan_combinations`.
///
/// # What exhaustion costs, which cuts across the two kinds
///
/// A reader must not infer the stop effect from the provenance. Four classes:
///
/// - **Program bounds** (`semantic_values`, `semantic_operations`, `regions`,
///   `host_expression_nodes`, `buffers`) refuse a *submitted program* at
///   [`check_program_budgets`], before any target is consulted.
/// - **Region-shape bounds** (`region_members`, `region_boundary_outputs`,
///   `region_live_values`) declare the largest region this profile forms at all,
///   so a program whose only implementable cover needs a bigger one has no plan
///   under them however long the search runs.
/// - **Alternative-preserving search caps** (`normalization_rewrites`,
///   `region_candidates_per_seed`, `region_expansions`, `region_covers`,
///   `region_cover_expansions`) cost an alternative and never coverage, because
///   what they bound sits strictly between extremes their stage emits
///   unconditionally: normalization retains the verified input, region formation
///   emits every singleton region and the whole-program region before growth,
///   and cover enumeration retains the fully-materialized and fused covers.
/// - **A truncating cap** (`physical_plan_combinations`) has no such extreme to
///   fall back on and may remove *every* complete plan. Its own field
///   documentation states the accepted contract.
///
/// Whether a refusal's reported number is exact, an envelope, or a lower bound is
/// a separate question again, answered by [`BudgetResource::refusal`].
///
/// # Identity
///
/// Every field is written into the canonical request subject by
/// [`VerifiedRequestSubject::canonical_explain_subject_bytes`], so any value
/// change moves every governed compilation's request and evidence subject —
/// including programs nowhere near the bound — because a budget is a property of
/// the request rather than of the plan chosen for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeterministicBudgets {
    /// Values a submitted program may declare and produce.
    ///
    /// Derived; [`Self::governed`] states the formula and the declaration.
    /// [`check_program_budgets`] refuses the program that exceeds it.
    pub(crate) semantic_values: u32,
    /// Semantic occurrences a submitted program may declare.
    ///
    /// Derived; [`Self::governed`] states the formula and the declaration.
    /// [`check_program_budgets`] refuses the program that exceeds it.
    pub(crate) semantic_operations: u32,
    /// Dispatch regions the widest plan for a submitted program may assemble.
    ///
    /// Derived; [`Self::governed`] states the formula and the declaration. The
    /// compared actual is a planning envelope computed before a plan is chosen,
    /// so a particular reachable plan may use less.
    pub(crate) regions: u32,
    /// Host expression nodes the widest plan for a submitted program may spell.
    ///
    /// Derived; [`Self::governed`] states the formula and the declaration. The
    /// compared actual is a planning envelope, as for `regions`.
    pub(crate) host_expression_nodes: u32,
    /// Buffers the widest plan for a submitted program may bind.
    ///
    /// Derived; [`Self::governed`] states the formula and the declaration. The
    /// compared actual is a planning envelope, as for `regions`.
    pub(crate) buffers: u32,
    /// Rewrites the deterministic normalization stage may commit.
    ///
    /// Normalization visits each verified operation exactly once, so its
    /// traversal is already bounded by `semantic_operations`. This is the
    /// stage's own explicit budget over committed rewrites.
    ///
    /// **Uncalibrated policy literal.** The work unit is one committed rewrite
    /// in one proposed alternative, bounded per alternative rather than summed
    /// across them. A stop abandons the whole rewrite run and returns nothing,
    /// so the verified input survives and only alternatives are lost. Nothing
    /// establishes why the number is exactly what it is.
    pub(crate) normalization_rewrites: u32,
    /// Semantic occurrences admitted in one region candidate.
    ///
    /// **This and the two below bound a region's admissible *shape* rather
    /// than a search, and each of them can refuse a program.** They declare the
    /// largest region this profile will form at all, so a program whose only
    /// implementable cover needs a bigger one has no plan under them however
    /// long the search runs — a refusal reported as `BudgetExhausted` naming
    /// the bound, because the caller's action is to widen it. The two search
    /// bounds below carry the opposite guarantee.
    ///
    /// Because they can refuse, all three are **derivations over the
    /// declaration** rather than stated numbers: a region is a subset of the
    /// program it covers, so [`Self::governed`] derives this one from
    /// `semantic_operations`, `region_live_values` from `semantic_values`, and
    /// `region_boundary_outputs` from the declared output count.
    pub(crate) region_members: u32,
    /// Retained boundary outputs admitted for one region candidate.
    ///
    /// Derived from the declared output count; see `region_members` above for
    /// why all three shape bounds are derivations and what their refusal means.
    pub(crate) region_boundary_outputs: u32,
    /// Boundary and member-result values live across one region candidate.
    ///
    /// Derived from `semantic_values`; see `region_members` above.
    pub(crate) region_live_values: u32,
    /// Grown candidates admitted for one seed occurrence.
    ///
    /// Both coverage extremes — every singleton region and the whole-program
    /// region — are emitted before growth starts and neither is bounded by this
    /// budget, so exhausting it loses the partitions discovered between them
    /// rather than either end.
    ///
    /// **Uncalibrated policy literal.** The work unit is one grown candidate
    /// admitted for one seed. Nothing establishes why the number is exactly
    /// what it is.
    pub(crate) region_candidates_per_seed: u32,
    /// Candidate expansion attempts admitted for one compilation request.
    ///
    /// Bounds the same discovered space as `region_candidates_per_seed` and
    /// carries the same guarantee, for the same reason: coverage precedes
    /// growth. It did not before
    /// `region-expansion-exhaustion-loses-the-only-feasible-plan`, and the
    /// consequence was not academic — growth reaches the whole-program
    /// candidate last, so a twelve-operation chain exhausted this bound before
    /// forming the one region the profile could implement and the compilation
    /// refused.
    ///
    /// **Uncalibrated policy literal.** The work unit is one candidate
    /// expansion attempt across the whole request. Nothing establishes why the
    /// number is exactly what it is.
    pub(crate) region_expansions: u32,
    /// Distinct legal complete covers retained for one enumeration request.
    ///
    /// The fully-materialized and fused covers are retained unconditionally, so
    /// exhausting this bound loses additional discovered partitions rather than
    /// either extreme.
    ///
    /// **Uncalibrated policy literal.** The work unit is one retained distinct
    /// legal complete cover. Nothing establishes why the number is exactly what
    /// it is.
    pub(crate) region_covers: u32,
    /// Partition-search expansion attempts admitted for one cover enumeration.
    ///
    /// Carries the same alternative-preserving guarantee as `region_covers`, and
    /// for the same reason: both cover extremes are retained before the
    /// partition search runs.
    ///
    /// **Uncalibrated policy literal.** The work unit is one partition-search
    /// expansion attempt. Nothing establishes why the number is exactly what it
    /// is.
    pub(crate) region_cover_expansions: u64,
    /// Complete-plan combinations admitted for one cover source.
    ///
    /// **This is the one budget whose exhaustion can remove every complete
    /// plan, and Tom accepted that contract on 2026-08-11.** The value bounds
    /// attempted Cartesian implementation combinations for one cover.
    /// `crate::selection::enumerate_cover_plans` stops before assembling the
    /// combination that would exceed it, records a typed
    /// `crate::selection::PlanBudgetStop`, and retains every valid plan
    /// assembled before that point — which may be none. An empty
    /// `crate::selection::SelectedPortfolio` is the ordinary result of an
    /// explicitly bounded search, not a fallback.
    ///
    /// Unlike region formation and cover enumeration, this layer has no
    /// privileged valid plan to retain outside the bounded search. Region and
    /// cover extremes are verified *coverage* objects; selection instead
    /// receives locally feasible per-region implementation frontiers, and
    /// `crate::selection::assemble_plan` is the first authority that proves
    /// their cross-region boundary compatibility. The first combination in
    /// canonical identity order can therefore disagree while a later one
    /// composes, so a reserved attempt would neither guarantee a plan nor honour
    /// the stated limit. When no harder target refusal exists and no valid
    /// complete plan was retained, compilation returns `BudgetExhausted` naming
    /// `physical-plan-combinations`; it does not claim the program is
    /// infeasible and does not switch backend, target, or strategy family.
    ///
    /// **Uncalibrated policy literal.** The work unit is one attempted complete
    /// combination for one cover source. Nothing establishes why the number is
    /// exactly what it is.
    pub(crate) physical_plan_combinations: u64,
}

impl DeterministicBudgets {
    /// The bounded profile's deterministic budgets.
    ///
    /// **The five program-scoped bounds are sized to the complete decoder-layer
    /// program, which is the largest program shape this profile may be asked to
    /// admit.** Each is derived from that program's own measured counts rather
    /// than from the smallest number that lets it through, which is the rule
    /// [`check_program_budgets`] states and the rule the split reduction's
    /// earlier widenings followed. The counts are the two rows the layer was
    /// verified and reference-evaluated at: eighteen declared inputs and three
    /// ordered named outputs at both, fifty-eight occurrences over seventy-six
    /// values at the C1 prefill row, and sixty-two over eighty at the C1 decode
    /// row. The decode row is the binding one, and it is larger for a reason
    /// that is not the cache: at `T = 1` six position-axis rank pads duplicate
    /// nothing, so the broadcast family refuses a many-to-one relation onto an
    /// extent-one result axis and the layer spells those widenings as further
    /// occurrences.
    ///
    /// - `semantic_values` is `80`: the decode row's eighteen declared inputs
    ///   plus one result per occurrence, because no occurrence in the layer
    ///   produces more than one value. The prefill row is `18 + 58 = 76` by the
    ///   same arithmetic, so eighty bounds both.
    /// - `semantic_operations` is `62`: the decode row's occurrence count.
    /// - `regions` is `12`: [`check_program_budgets`] derives the actual as four
    ///   dispatches per declared output — the widest producer chain one output
    ///   can reach, prologue, partial, final, and epilogue — so three outputs
    ///   reach `3 × 4`.
    /// - `host_expression_nodes` is `51`: the same function derives the actual
    ///   as two nodes per declared input, four per declared output, and three
    ///   program-scoped nodes, so `2 × 18 + 4 × 3 + 3`.
    /// - `buffers` is `30`: the actual is every declared input plus four per
    ///   declared output — the prologue's temporary, a split's staged partial
    ///   tensor, the fold's staged result an epilogue reads, and the output — so
    ///   `18 + 4 × 3`. It was `3`, then `4`, then `6`, then `21` — the one-input
    ///   materialized program's input, temporary and output; the split's staged
    ///   partial tensor; that split over the widest three-input prologue the
    ///   governed target's four buffer bindings admit; and the eighteen-input
    ///   layer under a one-output derivation — and every step, including this
    ///   one, is the same derivation over a wider admitted program.
    ///
    /// **The three region-shape bounds are derived from those five rather than
    /// picked, because a region is a subset of the program it covers.** Its
    /// members are a subset of the program's occurrences, its live values are
    /// disjoint subsets of the program's values, and the largest of them — the
    /// whole-program region — exports exactly what the program declares. So
    /// each is a formula over the same declaration, on exactly the ground
    /// `regions` was corrected on below: a quantity that belongs to a *plan* is
    /// still a function of the declaration the plan covers.
    ///
    /// - `region_members` is `62`: `semantic_operations`, because a region's
    ///   members are a subset of the program's own occurrences and a program
    ///   admitted at all holds no more than that many. It is the program-scoped
    ///   bound itself; see the collapse note below for why the field is still
    ///   encoded.
    /// - `region_boundary_outputs` is `3`: the declared output count, which is
    ///   the same count `regions` multiplies by four. The whole-program region
    ///   exports one value per *named* result and nothing else, because no
    ///   occurrence outside it consumes anything, so the largest region this
    ///   profile forms exports exactly the declaration's ordered named outputs.
    /// - `region_live_values` is `80`: `semantic_values`, because a region's
    ///   live values are its boundary inputs and its members' results, which
    ///   are disjoint subsets of the program's own values. It is tight at the
    ///   whole-program region, whose boundary inputs are the eighteen declared
    ///   inputs — every other value it reads it also produces — and whose
    ///   member results are one per occurrence: the same `18 + 62`
    ///   `semantic_values` is.
    ///
    /// The three program bounds derived through [`check_program_budgets`] —
    /// `regions`, `host_expression_nodes`, and `buffers` — are tight at exactly
    /// eighteen declared inputs and three declared outputs, so their thresholds
    /// coincide along each axis:
    /// a nineteen-input program exceeds two of them at once and the earlier
    /// check, `host-expression-nodes`, is the one that reports, while a
    /// four-output program exceeds all three and `regions` reports.
    ///
    /// **`regions` was `4` and was checked against a constant rather than
    /// derived**, on the ground that a region count is a property of a *plan*
    /// and this profile plans no decoder layer: [`select_supported_strategy`]
    /// refuses it under its own named rules, which is a separate refusal with a
    /// separate remedy. That ground survives and its conclusion did not. A plan
    /// covers every declared output, so the plan-scoped constant is still a
    /// function of the *declaration* — one widest chain per ordered named
    /// output — and while recognition could name only one output the two were
    /// indistinguishable. Since multi-output admission they are not: two
    /// independent chains assemble seven or eight dispatches, so the literal
    /// bounded nothing and the program it refuses is the one it was written to
    /// refuse.
    ///
    /// The four in the per-output derivation is the measured stage count of the
    /// widest chain, taken from
    /// `crate::pipeline::tests::the_widest_assembled_plan_is_the_split_reduction_with_its_epilogue`,
    /// whose reassociation-forbidding neighbour is what attributes the fourth
    /// stage to the split rather than to the epilogue alone.
    ///
    /// The consequence of every one of these moves is the one this comment
    /// already records: every budget is written into the request subject, so
    /// every governed compilation's qualifier moved with them. The one pinned
    /// literal is `explain`'s
    /// `deterministic_trace_is_sealed_and_rendered_separately` request qualifier
    /// — and its ledger comment records the recomputation. No encoding version
    /// moved: the field set, widths, and order are untouched, so a value change
    /// stays injective inside the current `tiler.compiler.request-subject.v6`
    /// domain.
    ///
    /// They move again when the decoder layer becomes plannable, and that is a
    /// second identity move this one cannot honestly absorb. The three
    /// region-shape bounds move *with* them and never on their own account,
    /// which is what the derivation buys: a ceiling somebody has to raise per
    /// program is replaced by a formula that tracks the declaration.
    ///
    /// `normalization_rewrites`, `region_candidates_per_seed`, and
    /// `region_expansions` are unchanged, and the ground for leaving them alone
    /// survives intact: none of the three admits or refuses a program, because
    /// each bounds a *search* whose alternatives sit between two coverage
    /// extremes region formation emits unconditionally, so exhausting one costs
    /// an alternative while the verified input and complete coverage survive.
    ///
    /// That ground was stated for all six `region_*` bounds and was **half
    /// right**: the three shape bounds declare the largest region this profile
    /// forms, so a program whose only implementable cover needs a bigger one is
    /// refused by them however long the search runs. While `region_members` was
    /// the bare constant `32`, that refusal was measurable and measured: a
    /// shared-constant `f32` multiply chain's recognized partition is its whole
    /// program and nothing smaller is implementable, so 33..=62 occurrences
    /// refused `BudgetExhausted` on `region_members` although every bound on the
    /// program's own *size* admitted them. The derivations above dissolve that:
    /// the stated admission envelope and the actual planning envelope are now
    /// the same formulas over one declaration rather than two disagreeing
    /// ceilings.
    ///
    /// **Two of the three collapse onto the program-scoped bound they derive
    /// from, and that is the derivation's answer rather than a defect in it.**
    /// `region_members` *is* `semantic_operations` and `region_live_values`
    /// *is* `semantic_values`, so for a program whose occurrences are each
    /// realized by one region neither can fire: `check_program_budgets` has
    /// already refused anything with more occurrences or values than the region
    /// bound would. They are still encoded, for two reasons. The first is that
    /// region formation's attribution atom is a realization *stage* rather than
    /// an occurrence and its live values include the intermediates a staged law
    /// hands between stages — neither is a value the program's own occurrence
    /// and value counts hold — so both bounds still bind on a program whose
    /// families realize region sequences. The second is that a budget set is a
    /// *request* field: these bound one region's shape for any budget policy,
    /// and the governed profile's coincidence is a property of its declaration
    /// rather than of the fields. Tom accepted on 2026-08-11 that both keep
    /// their slots in the canonical request subject. Omitting them would make
    /// distinct staged-region policies share one request/evidence subject,
    /// while the measured saving is eight bytes. `region_boundary_outputs`
    /// does not collapse: it is the declared output count rather than any
    /// program-scoped bound, and it still refuses a grown candidate that would
    /// export more values than the program names.
    ///
    /// Every value here is a *deliberate* decision and not a test-enabling
    /// edit, because every one of these numbers is inside the canonical request
    /// subject ([`VerifiedRequestSubject::canonical_explain_subject_bytes`]
    /// writes every budget). Every governed compilation's request/evidence
    /// subject moves with such a change — for programs nowhere near any bound
    /// as much as for ones at it — because a budget is a property of the
    /// compilation request rather than of the plan chosen for it. The request
    /// subject is not artifact or cache identity; those move only when the
    /// selected packaged content moves. The one checked-in literal derived from
    /// these bytes is `explain`'s
    /// `deterministic_trace_is_sealed_and_rendered_separately` request
    /// qualifier, whose ledger comment records the recomputation. No encoding
    /// version moved with it — the field set, widths, and order are untouched,
    /// so a value change stays injective inside the current
    /// `tiler.compiler.request-subject.v6` domain.
    ///
    /// A budget is an upper bound, so widening admits program shapes and never
    /// requires them: [`check_program_budgets`] still refuses a program one step
    /// past each of these, and `verify_host_contract` still refuses a built
    /// program whose expression, value, or stage count exceeds
    /// `host_expression_nodes`, `buffers`, or `regions`. The same holds of the
    /// three derived region bounds one layer down:
    /// [`crate::region::RegionBudgetResource`] still stops a candidate one step
    /// past each of `region_members`, `region_boundary_outputs`, and
    /// `region_live_values`, and the stop is still reported as a typed
    /// `BudgetStop` naming the resource rather than dropped. Nor does clearing
    /// the budget gate compile a decoder layer — the recognizer's refusal is
    /// untouched, and what these values remove is only the refusal that was
    /// about *size*.
    pub(crate) const fn governed() -> Self {
        Self {
            semantic_values: 80,
            semantic_operations: 62,
            regions: 12,
            host_expression_nodes: 51,
            buffers: 30,
            normalization_rewrites: 8,
            region_members: 62,
            region_boundary_outputs: 3,
            region_live_values: 80,
            region_candidates_per_seed: 32,
            region_expansions: 10_000,
            region_covers: 1_024,
            region_cover_expansions: 100_000,
            physical_plan_combinations: 4_096,
        }
    }
}

pub(super) fn check_budget(
    resource: BudgetResource,
    limit: u32,
    actual: usize,
) -> Result<(), RequestError> {
    let limit = u64::from(limit);
    // Saturating, on the same ground as the four `count` helpers this crate
    // already carries: no supported target has a `usize` wider than `u64`, and a
    // count that did not fit would exceed every budget this profile declares.
    let reported = u64::try_from(actual).unwrap_or(u64::MAX);
    if reported > limit {
        return Err(RequestError::BudgetExceeded {
            resource,
            limit,
            reported,
        });
    }
    Ok(())
}
