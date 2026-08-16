---
schema: "tiler-doc/v1"
id: "tiler.contract.optimizer"
kind: "contract"
title: "Optimizer model"
topics: ["optimizer", "search", "planning"]
contract_status: "accepted"
implementation_status: "partial"
evidence: ["tiler.research.region-search.exhaustive-region-oracle", "tiler.research.reference.normative-reference-slice", "tiler.research.cost-model.bootstrap-cost-model", "tiler.research.program-planning.general-compilation-boundary"]
---

# Optimizer model

**Status:** accepted research contract; bounded prototype implementation

The first bounded compiler slice — reached from outside the crate through the reviewed public `tiler_compiler::session` boundary since [`prototype-public-compiler-api`](../../tickets/prototype-public-compiler-api.md) landed it, not a private one — retains complete materialized and fused program alternatives, carries exact structural metrics, and selects the fused program only when it strictly Pareto-dominates the baseline. Its stable policy key makes no latency claim. A missing per-operation *fusion numerical* capability, candidate-budget exhaustion, or fused target infeasibility rejects only the fused alternative; failure of a compiler-produced verifier remains a hard compiler error. General memo search in the sense [bounded hierarchical search](#bounded-hierarchical-search) reserves the word for, and calibrated cost estimation, remain unimplemented. *Corrected 2026-08-05:* this sentence also named partitioning, which landed 2026-08-04 under [`implement-general-dag-partitioning`](../../tickets/implement-general-dag-partitioning.md) and is contradicted twice below — [what each stage is general over today](#what-each-stage-is-general-over-today) and [the memo contract](#possible-memo-contract) both state the general DAG cover search as done.

**Fact — no installed-provider constant gates the fused alternative.** `crates/tiler-compiler/src/request.rs` used to carry two named lowering-provider constants, one materialized and one optionally installed fused serial-sum provider, and an absent fused constant suppressed the whole-program candidate before its numerical equivalence was ever proved. Both constants are gone. Whether a whole-program candidate is retained is now decided by [fusion legality](fusion-and-scheduling.md#legality) and typed target feasibility alone, and each retained alternative's lowering authority is whatever the request's installed registry resolved for its member occurrences. A missing *lowering* capability is not in this class at all and never rejects a single alternative; [capability resolution](#lowering-capability-resolution-and-index-region-refinement) states what it does instead.

The bounded slice rederives alternative identity, structural cost, KIR,
program, artifact receipt, and selection from the verified semantic/request
subject before returning a portfolio. Selection authority is the verified
alternative identity under a named cost model, not a caller-editable vector
index or stored cost. Explain records retain typed subjects, evidence class,
budget actual/limit pairs, feasibility facts, and provenanced cost values.

## Ownership boundary

This document owns planning phases, rule contracts, alternative retention,
search bounds, costing inputs, and explainability. It consumes verified IR
schemas from the IR contract and does not redefine their fields or backend
resource limits.

Tiler borrows selected techniques from property-aware database optimizers while
using a tensor operation/value DAG, access-aware fusion regions, and explicit
GPU schedules. DataFusion is useful vocabulary for semantic/executable
separation and boundary enforcement, but it is not the structural template for
Tiler's graph or search algorithm.

The contract synthesizes the [region oracle](../research/region-search/exhaustive-region-oracle.md),
[index/access model](../research/indexing/index-access-model.md),
[scheduled-region model](../research/scheduling/scheduled-region-model.md),
[whole-program plan](../research/program-planning/kernel-program-buffer-plan.md),
and [structured-kernel verifier](../research/kernel-ir/structured-kernel-ir-verifier.md).

## Compilation boundary and failure classes

Everything below is reached through one general, consumer-independent
compilation boundary over a verified `SemanticProgram` and explicit request
inputs. Under ADR 0069 there is no graph-specific entry point, `experimental`
namespace, or serial-Sum support profile. A bounded vertical slice remains a
private strategy, conformance fixture, and explain identity; its fixed region,
stage, entry, and buffer cardinalities are not request or result invariants.

The boundary returns either general target-neutral program products or a typed
outcome drawn from five distinct failure classes:

- **invalid request:** the semantic program, resolved numerical contracts,
  shapes, frozen registry, or request inputs are malformed;
- **missing compilation capability:** the program is valid, but no installed
  access, scheduling, lowering, or provider capability covers it;
- **infeasible plan:** every candidate is intrinsically invalid or rejected by
  typed target feasibility;
- **exhausted bounded search:** a declared candidate or expansion budget
  stopped exploration before a complete plan was selected; and
- **compiler IR verification failure:** a compiler-produced index, schedule,
  kernel, or program value failed its authoritative verifier.

These classes are not interchangeable. A valid program that lacks coverage is
never reported as malformed, and an unsupported case fails closed with an
explainable reason rather than being approximated to retain a fast path. A
budget that stops one growth path while complete coverage survives is an
explain reason on the selected plan, not this failure class. A verifier failure
is invalid compiler output and remains a hard error rather than a costed
rejection.

Stable reason keys name the failed relation, not whichever operation happened to expose it. `operation-set` means the recognizer found no admitted operation vocabulary for an occurrence. `reduction-contributor-materialization` instead means the serial-reduction contributor walk reached a recognized value that must cross a materialization boundary, while the retained serial-reduction normal form has no producer relation to carry that boundary. A nested reduction, a contraction, and a registered staged family therefore share this one key: splitting it by producer family would mix the failure cause with its subject and would make every new materializing family widen the public reason vocabulary. The classification changes no admission; the program remains a typed missing compilation capability until that producer relation is representable.

An exhausted index proof budget is not itself any failure class: the structurally verified index region retains the exact residual predicate and typed `ResourceLimit` reason because the subject was not disproved. The current compiler still cannot continue to executable planning without refinement evidence. After independently checking scalar authority and the occurrence interface so a harder provider defect cannot be masked, the named semantic-discharge stage assesses every exact residual and stops before cover or frontier construction unless all are proved. [Index-region refinement](#refinement-requires-discharged-index-domain-evidence) owns that boundary.

## Planning model

```text
SemanticTensorGraph
  -> deterministic normalization
  -> bounded baseline-preserving logical exploration
  -> per-alternative request readmission and contract grouping
  -> independently verified candidate planning
  -> overlapping RegionCandidates
     |-> independent complete-cover enumeration ---------|
     `-> checked schedules + ImplementationFrontier -----|
  -> compatible complete physical-plan selection per semantic candidate
  -> verified global selection across one contract group
  -> structured KIR refinement
  -> KernelProgram or guarded ProgramPortfolio
```

The optimizer must distinguish:

- **logical equivalence:** expressions compute the same tensor under a stated
  numerical policy;
- **fusion legality:** a region can be implemented correctly as one kernel;
- **physical feasibility:** a schedule fits target capabilities and resources;
- **profitability:** the complete plan is preferable to legal alternatives.

## The four surfaces the optimizer may consult

**Fact — Tom set this direction on 2026-08-01.** Physical-plan optimization operates generically over every execution tier and backend, so that no optimizer or selection logic is rewritten when a device family arrives. [ADR 0090](../decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 1 is the accepted authority for where target-specific scheduling knowledge splits, and item 2 for the installable physical-provider seam; this section records the composed consequence here, ahead of the named stages, because the stages below are where a tier-specific special case would be written and every one of them inherits this.

Enumeration, retention, and selection see exactly four surfaces and nothing else. Each names the implementation carrying it today rather than restating it, because a second description of a vocabulary drifts from the first.

1. **Neutral alternatives.** An alternative is a schedule in the execution-axis, tile, and synchronization vocabulary of `crates/tiler-ir/src/schedule/`, never a backend construct. That module is target-neutral by its own declaration — it "owns no target profile, no feasibility decision, no cost model, and no semantic-graph correlation" — and what reaches the frontier is one of its values: `ProposalBody::ScheduledKernel` in `crates/tiler-compiler/src/frontier.rs` carries a `tiler_ir::schedule::ScheduledRegion`, which the host resubmits through the ordinary intrinsic verifier rather than trusting. [Fusion and scheduling](fusion-and-scheduling.md#schedule-representation) owns the vocabulary itself, and [physical implementation](#physical-implementation) owns which bodies the frontier admits.

2. **Typed permissions.** Whether a rewrite or a parallel strategy is legal is answered from the operation's own registered algebraic capabilities and the request's resolved numerical contract. Both are target-independent: no profile, fact, or device participates. `OperationAlgebraicCapabilities` on a frozen semantic definition (`crates/tiler-ir/src/semantic/operation.rs`) declares ordered associativity; `NumericalRealization` (`crates/tiler-ir/src/schedule/numerics.rs`) carries `contraction`, `reassociation`, and `permutation` as typed `NumericalPermission` values; `tiler_compiler::session::NumericalContract` is the composed contract a caller resolves dimension by dimension, from which the request's resolved contract is derived. [Rule authority is operation-owned](#implemented-first-algebraic-portfolio) states the consequence for semantic rewrites, and a physical strategy consumes its permissions the same way — `StrategyDeclineCause::NumericalPermissionRefused` in `crates/tiler-compiler/src/frontier.rs` is a decline decided from the request alone, before any region is constructed and against no target at all.

3. **Feasibility queries.** Whether a target realizes an alternative is answered from typed profile data, never by calling backend code. `crates/tiler-compiler/src/target/feasibility.rs` is that authority, and its own contract is that it "deliberately has no notion of cost". The shape a new tier's fact must take is fixed by the atomic-realization precedent: `TargetProfileBuilder::declare_synchronization_realization` in `crates/tiler-compiler/src/target.rs` takes one whole subject with no per-dimension spelling, so a profile's neighbouring facts cannot compose into a permission for a subject none of them is about. The labelled-draft `declare_subgroup_realization` / `declare_measured_subgroup_realization` pair takes the same whole-subject shape, and unlike synchronization it records silence as absence so standard profiles keep their `v11` declaration bytes. Atomic realization rows are sorted by their complete uniqueness key — `(subject, phase)`, excluding the verdict — before both the checked descriptor (`tiler.target-profile.descriptor.v10`) and the complete declaration (`tiler.target-profile.declaration.v11`) encode, so two profiles stating the same rows in different insertion orders share one identity; an exact duplicate and a same-key contradiction refuse independently, and the sort never chooses a winner. ADR 0043 owns the outcome vocabulary that authority answers in and ADR 0076 the numerical-honourability dimensions it composes alongside. A target lacking a tier therefore starves those alternatives *at feasibility*, with an explainable reason rather than a forked enumeration: `FrontierRejection::Unsynchronizable` carries the fact that refused, and `FrontierRejection::SynchronizationUndeclared` deliberately carries none, which is what keeps a refusal and a silence different answers.

4. **Typed costs.** Preference is expressed only in the typed cost vocabulary, and hard feasibility is never expressed as a cost in either direction. `PhysicalCostEstimate` under the governed `tiler.cost.structural.v1` key is the sole *pruning* input; `crates/tiler-compiler/src/component_cost.rs` reports the analytical vocabulary under its own `tiler.cost.analytical.v1` key and never reaches dominance, because plans carrying different model keys do not dominate each other. The separation is enforced by absent conversions rather than by a rule: the feasibility authority has no cost input, and `crates/tiler-compiler/src/estimate.rs` offers no conversion from an uncertain estimate into anything feasibility consults. [Cost model](cost-model.md) owns ranking objectives and calibration.

   **Since 2026-08-07 a third typed cost vocabulary exists, it is not a pruning input, and the distinction between pruning and preferring is what keeps it inside this surface rather than widening the four.** A target profile may declare a **measured cost row** — a machine quantity carried apart from every `CapabilityAxis`, because an axis is a hard bound whose silence is an `Unknown` that never reaches an executable frontier, while silence about a cost row must mean *no preference* rather than *no plan*. `crates/tiler-compiler/src/measured_cost.rs` turns the one declared row into an ordering over the retained valid plans under its own `tiler.cost.measured-fold-steps.v1` key, with no `dominates` and no path into `aggregate_cost`. The structural relation keeps its four exact dimensions, its single model key, and its full pruning power; `SelectedPortfolio::non_dominated` still computes it and explain still reports it for every alternative. [`activate-measured-reduction-selection-from-a-target-cost-row`](../../tickets/activate-measured-reduction-selection-from-a-target-cost-row.md) is the accepted decision, and Tom accepted the declaration pair's exact public spelling on 2026-08-07 under [`accept-the-measured-cost-row-public-surface`](../../tickets/accept-the-measured-cost-row-public-surface.md).

   **This changes what selection *is*, and the change is stated rather than absorbed.** Selection was a Pareto relation over exact structural counts with a canonical-identity tie break, and a measured term that can prefer a *structurally dominated* plan is not a new dimension in what that relation already does. What licenses it is that structural dominance never claimed to prove a plan faster — this contract's own words are that its policy key "makes no latency claim" — and the retained 2026-08-07 dispatch sweep refutes fewer-resources-is-faster on a named contour: the serial fold issues no more dispatches, launches strictly fewer threads, and allocates no more temporary storage than either parallel reduction strategy, and was measured costing up to **50.7x** the best parallel plan. On that program family the non-dominated view is a singleton, so a term confined to breaking ties inside it could express nothing at all. The measured term therefore ranges over the retained *valid* plans, which is every plan hard feasibility and boundary composition admitted: it can prefer a structurally dominated plan and can never prefer an infeasible one, because none is in the set. **A profile declaring no cost row selects bit-identically and encodes bit-identically**, which is a tested guarantee rather than an intention.

   **The no-cost-pruning invariant below is untouched, and this is the reading that keeps it so.** That invariant forbids pruning a *semantic* alternative on estimated cost. A measured row is consulted where an alternative's meaning is already settled — over complete physical plans of one contract group, after feasibility — which is exactly the stage the invariant already admits cost at. It retains every alternative it does not select, and it is the *selection* that moves, not the retention.

A backend contributes **data** — the facts and realizations its profile declares — and **alternative generators**, whose output re-enters the neutral vocabulary and passes the same intrinsic verification and the same feasibility authority as the compiler's own proposals. It never contributes search logic and never performs a comparison. `PhysicalImplementationProvider` is that seam, and ADR 0090 item 12 enumerates what no component may do.

**Fact — the seam became installable from outside the crate on 2026-08-08, and what a caller can install is narrower than what the frontier admits.** [`drive-an-external-physical-implementation-provider-through-compilation`](../../tickets/drive-an-external-physical-implementation-provider-through-compilation.md) landed `tiler_compiler::physical_provider` as a reviewed draft under ADR 0075, and Tom accepted its exact included and excluded public surface on 2026-08-11: an out-of-crate provider composes an `InstalledPhysicalProviders` and installs it through `CompileRequest::with_physical_providers`, its proposals reach `enumerate_frontier` on the ordinary compile path, and `PlanAlternative::selected_physical_providers` names the provider whose implementation each retained plan selected. Three properties of the *installed* set are narrower than the crate-internal seam and each is a refusal rather than an omission: installation is **additive** and cannot displace the governed provider, `ImplementationProposal::scheduled_kernel` is the **only** body a caller may propose — a subprogram carries graph-local attribution this boundary does not export and an opaque call stays compiler-owned per ADR 0090 item 14 — and `PhysicalCostEstimate::structural` is the only estimate constructor, so an ungoverned cost model has no public spelling to attribute to. What an installed provider builds its body *from* is this host's own baseline spelling of the subject (`ImplementationContext::baseline`), because the request-subject binding compares a proposed region's identity, iteration shape, scalar program, semantic members, and access map against the compiler's own normalization: a provider stating none of those could only guess. That makes *specializing a spelling* the operation the seam supports today and leaves proposing a region the recognizer did not pre-compute out of reach, which is the same wall the stage-8 paragraph below names rather than a second one.

**Fact — one hard question is answered before any of the four is consulted, and it is feasibility rather than cost.** A program carrying a registered elementary family places an accuracy obligation on every target it is compiled against. `request::require_elementary_accuracy` (`crates/tiler-compiler/src/request.rs`) collects the program's operation keys and calls `target::accuracy::assess_program_elementary_accuracy` (`crates/tiler-compiler/src/target/accuracy.rs`), which deduplicates the obligations by operation — a program with one `tiler::silu-f32@1` occurrence and a program with a hundred owe the same contract — and requires some realization the target declares to *provably refine* each one **and** to discharge both its bound evidence and its exceptional-value evidence. Contract refinement is necessary and not sufficient: empirical qualification and `Unknown` remain recorded as evidence and cannot silently become permission to compile. It is asked per target, beside the dtype-dispatch check and before any numerical contract is resolved, because the obligation is the registered operation's and no contract a caller can state widens or waives it; `readmit_candidate` asks it again of every semantic candidate, so a rewrite cannot inherit an admission granted to a program that did not contain the family. Assessment reads the profile's stored declared rows. There is no governed-descriptor shortcut: a profile that declared nothing, including the governed profile until a later evidence ticket can discharge both halves of a Metal row, has an empty view. A caller-built profile states a row through the labelled-draft `TargetProfileBuilder::declare_elementary_realization`, which stores one whole verified subject and refuses only an exact duplicate. A refusal is `RequestError::UnrealizedElementaryAccuracy` and it is target-local — another requested profile may declare the realization this one does not — carrying the operation, the profile key, the refusing authority's own stable key, deterministic candidate-declaration provenance where a row exists, and, when the evidence cannot discharge, the failing half and evidence class: `ElementaryAccuracyRefusal::diagnostic_code` answers `accuracy.elementary.no-installed-realization` when nothing installed speaks about the operation, `accuracy.elementary.unrefined-realization` when something does and the refinement relation could not prove it, and `accuracy.elementary.undischarged-evidence` when a row refines but a half's class cannot discharge a hard requirement. The public structured refusal is `TargetCompileRefusal::ElementaryAccuracy`. Nothing here is tradeable against an estimate. `assess_elementary_accuracy` is conservative in one direction only — an admission is a proof and a refusal may be a limitation of the closed refinement algebra — so it can reject a legal implementation and can never admit an illegal one, which is the asymmetry that makes it a feasibility answer. It is not a fifth surface: no alternative exists when it is asked. It is stated beside them because it is the point at which a target that cannot realize an elementary function stops the compilation, and because the only wrong way to reintroduce it later is as a cost.

**A prohibition is attached to the fourth surface, and it is the invariant the whole search rests on: no semantic alternative is pruned on estimated cost, at any stage.** Typed cost is consulted only where an alternative's *meaning* is already settled. Semantic exploration refuses with semantic-applicability, numerical-permission, and configuration declines carrying distinct typed assessments; contract grouping preference-prunes a whole group without planning it, on the caller's stated order rather than on any estimate; and structural cost and Pareto dominance act only on physical plans, and only inside one contract group. [The rewrite-search formalism record](../research/region-search/rewrite-search-formalism.md) is the authority, and the invariant is a durable commitment rather than an implementation accident: that record's [executable witness](../../spikes/region-search/phase_ordering_witness.py) shows a search that retains alternatives but discards each one a cheaper predecessor dominates failing to reach a mutually-enabled composition *exactly as the greedy rewriter does*, because the enabling intermediate is precisely the alternative a local cost comparison removes. Retention alone is not the property; retention without a cost comparison is.

### The review obligation

Nothing mechanical checks this invariant, and nothing can: no lint or test distinguishes a legitimate generalization of selection logic from a tier-specific special case, because both compile and both pass. The check is a review rule, and it is stated here so a reviewer has a named rule to cite rather than a drift to notice.

**An execution-tier or backend landing whose diff touches enumeration ordering, dominance, retention, or plan selection is treated as a violation of this invariant until its author justifies it.** A tier is expected to arrive as a schedule vocabulary, one or more declared profile facts, a proposal generator, a typed decline, and a cost — and to leave the machinery that compares alternatives untouched. A justified exception exists (generalizing a comparison that was accidentally specific is exactly the repair this invariant wants), but it is argued in the review, not assumed by the landing.

**Fact — the discipline held across the 2026-08-01 tier and strategy landings, verified by reading their diffs rather than their summaries.** The cooperative-workgroup tier landed in two commits: `be8f42e` ([`represent-cooperative-workgroup-reduction-dataflow`](../../tickets/represent-cooperative-workgroup-reduction-dataflow.md)) touched no `crates/tiler-compiler` file at all, adding the tile and its verifier entirely within `crates/tiler-ir/src/schedule/` and `crates/tiler-ir/src/kernel/`; `fece761` ([`admit-the-first-typed-synchronization-point-and-atomic-target-authority`](../../tickets/admit-the-first-typed-synchronization-point-and-atomic-target-authority.md)) added the target fact and its feasibility composition, and its only line in `selection.rs` is `synchronization: None` added to a struct literal inside `mod tests`. `63fde23` ([`implement-the-single-workgroup-synchronized-reduction-strategy`](../../tickets/implement-the-single-workgroup-synchronized-reduction-strategy.md)), which landed the tree strategy hours later, does not touch `selection.rs` at all: it adds a proposal generator, a typed decline, a typed rejection, and a structural cost. None of the three commits edited a dominance, pruning, or plan-selection function — `git show <commit> -- crates/tiler-compiler/src/ | grep '^[+-]' | grep -iE 'dominat|prune|PlanStructuralCost'` returns nothing for each of them.

Two landings are empirical evidence over a bounded population, not a proof that the invariant holds for a tier nobody has built. What they establish is that the surfaces above were sufficient for the one execution tier and the one parallel strategy that have arrived, which is what makes the review rule a check with a precedent rather than an aspiration.

**A second review rule guards the no-cost-pruning invariant, and it is uncheckable for the same reason and with a sharper failure mode.** A profitability check added at semantic exploration compiles, passes every test, makes the candidate set smaller and the compile faster, and is indistinguishable from a legitimate optimization to anything that does not already know what it costs. **A diff that gives semantic exploration, per-alternative readmission, or contract grouping any access to an estimated cost — a `PhysicalCostEstimate` or `PlanStructuralCost` reaching those stages, a comparison of one proposal against a retained sibling, or a rule-`promise` heuristic consulted ahead of applicability — is treated as a violation of this invariant until its author justifies it.** The justification available is narrow, and the phase-ordering witness is what makes it narrow: it must show that the discarded alternative cannot enable a composition, and no cost model can discharge the *legality* half of an enabling relation, because a global fold with no materialized input is not a costly program but not a program at all.

**Fact — what holds the invariant today is an absent dependency rather than a rule, and that absence is what the review rule is spending itself to keep.** `crates/tiler-compiler/src/normalize.rs`, which owns the algebraic portfolio and `group_by_resolved_contract`, names a cost type on exactly one line and that line is a doc comment drawing an analogy to `PlanStructuralCost::dominates` — reproduce with `grep -n 'PhysicalCostEstimate\|PlanStructuralCost\|CoverCost\|ComponentCost' crates/tiler-compiler/src/normalize.rs`, which returns one hit. Over the whole crate the same pattern reaches `component_cost.rs`, `cover.rs`, `frontier.rs`, `selection.rs`, `pipeline/trace.rs`, and `pipeline.rs`'s `ProgramAlternative::structural_cost`, all of which observe complete physical plans. A diff that adds the first real cost dependency to the semantic stages is exactly where this rule is owed.

## Named stages and verifier boundaries

The initial optimizer pipeline has explicit stage names and cannot skip their
verification boundaries:

1. `VerifySemanticRequest` checks the graph, resolved numerical contracts,
   shapes, and frozen operation registry.
2. `NormalizeSemantics` produces one deterministic canonical graph.
3. `ExploreLogicalAlternatives` adds only proved contract-preserving forms.
4. `EnumerateRegionCandidates` forms connected convex semantic regions and
   retains complete singleton coverage.
5. `ResolveLoweringCapabilities` resolves exactly one index/access lowering
   capability per recognized occurrence against the frozen lowering registry the
   request carries. It selects an authority and drives none, so it proves
   nothing about emitted work.
6. `LowerIndexRegions` drives each resolved provider through the canonical index
   builder, derives width-independent domains/access maps, and proves read
   bounds plus exact unique ordinary writes against the occurrence its
   capability was resolved for.
7. `EnumerateCompleteCovers` independently enumerates legal whole-graph covers;
   it does not select schedules or implementations.
8. `ExploreScheduledRegions` intrinsically verifies normalized schedules for
   individual legal regions. Typed target-feasibility assessment then admits
   bounded per-region physical frontiers. This authority does not require a
   previously selected global cover.
9. `SelectCompletePhysicalPlans` joins complete covers with compatible local
   implementations, boundary contracts, proposed materializations,
   dependencies, and guards. It emits a checked selected-plan or portfolio
   receipt for cover/implementation compatibility, not final executable-program
   authority. Buffer requirements remain provisional at this stage.
10. `RefineStructuredKernels` lowers each selected scheduled kernel and proves typed,
    effect-safe refinement of exactly that schedule before backend emission.
11. `AssembleKernelPrograms` constructs verified executable programs from the
    checked physical-plan receipt and verified KIR. Only this post-KIR verifier
    authoritatively checks executable stage coverage, buffers, initialization,
    lifetimes, aliasing, storage handoffs, ABI/launch references, and routing.

Stages 5 and 6 run together, per recognized occurrence, before the first cover is enumerated: grouping occurrences the installed authority cannot lower would enumerate plans nothing could realize. They are two authorities in one pass, not one stage — resolution answers *which* authority lowers an occurrence and refinement answers whether the work that authority emitted realizes it.

The explain vocabulary spells these two stages `CapabilityResolution` and `KernelRefinement`. `KernelRefinement` carries both stage 6 and stage 10, which are different obligations over different subjects: the rule key `kernel.index-region-refinement.v1` names an index region refining a semantic occurrence, and `kernel.plan-refinement` names a structured kernel refining a selected schedule. A trace reader must not read one as evidence for the other.

`ExplainStage::CandidateEnumeration` is overloaded the same way and is the easier of the two to misread, because its name resembles stage 4's. It carries **stage 3 and stage 7**, never stage 4: adopted semantic alternatives, each declined algebraic rule's assessment, and the algebraic-portfolio budget stop are recorded under it (`crates/tiler-compiler/src/pipeline.rs`, `crates/tiler-compiler/src/normalize.rs`), and so are cover-enumeration failures (`crates/tiler-compiler/src/pipeline/planning.rs`). Stage 4 is spelled `RegionFormation` and is the only stage that is. A reader taking a `candidate-enumeration` row as evidence about region candidates has read a semantic rewrite or a whole-graph cover.

**One authority on the compile path is not in the eleven-stage list, and it is not an implementation detail.** Fusion-legality derivation runs between stages 7 and 8, once per multi-occurrence region a retained cover places, and a rejection removes every cover containing that region before any frontier is enumerated for it; a whole-program candidate whose fusion is legal additionally carries the strict-`f32` numerical-equivalence proof the trace cites as a sound proof. It has its own explain stage, `NumericalLegality`, and [fusion and scheduling](fusion-and-scheduling.md#legality) owns its rule content. It is absent from the numbered list because the list was written before it landed; it belongs between 7 and 8, and it is named here rather than renumbered so that no existing citation of a stage number moves.

### What each stage is general over today

**Fact — audited 2026-08-04 by reading the compile path.** The numbered stages above are the contract; this is where implementation currently stops being general, stated per stage so that a reader does not infer one stage's generality from its neighbour's.

**Stages 4 and 7 are general over an arbitrary verified DAG.** Region formation proposes every connected convex region up to the declared budgets, checked set-for-set against an exhaustive subset oracle; cover enumeration is the general DAG partition search that landed 2026-08-04, with fan-out, ordered multi-result outputs, per-edge materialization, memoized budgeted search, and exhaustive small-graph oracle agreement. Its duplication admission is a stated legality contract with typed per-member refusals, and the compile path enumerates under the *exact-partition* admission — [fusion and scheduling](fusion-and-scheduling.md#region-representation) states why, and the reason is stages 8 and 11 below.

**Stages 5, 6, 9, and 10 are general in the sense that matters for them.** They are driven per recognized occurrence, per retained plan, and per selected schedule respectively, with no shape recognizer behind any of them: resolution and refinement are unconditional over whatever occurrences the program contains, selection joins whatever covers and frontiers it is handed, and structured lowering refines whatever schedule was selected.

**Stage 1 is general in mechanism and bounded in vocabulary, and conflating the two is the specific error this section exists to prevent.** The request boundary is no longer a whole-program template match: it checks two program-wide properties — at least one declared input, and one recognized arithmetic width throughout — and then classifies the occurrence producing *each declared output*, walking outward through the occurrences feeding it at any declared input arity and over the pointwise-expression vocabulary of the width it derived, so a program shape nothing taught it is admitted when its occurrences compose. What it refuses is a vocabulary, not a shape: a width this build spells no per-point body in, two recognized widths in one program, an operation no region can spell, an elementwise stage reading a materialized intermediate, and two outputs whose walks share an occurrence. Each has its own rule and its own owning ticket. *Corrected 2026-08-06:* this sentence read "three program-wide properties — at least one declared input, exactly one output" and closed its refusal list with "a reduction reading a declared input directly, and more than one output". Both moved: [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](../../tickets/admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) made recognition one walk per ordered named output, leaving `output-partition-overlap` as the only multi-output refusal, and [`admit-a-reduction-over-a-declared-input-tensor`](../../tickets/admit-a-reduction-over-a-declared-input-tensor.md) widened `tiler-ir`'s serial `StrictSerialSum` arm to the fold's declared contributor domain, so `sum(x)` is realized by one region binding the input directly. *Corrected 2026-08-07 by [`widen-the-strategy-recognizer-past-the-f32-wall`](../../tickets/widen-the-strategy-recognizer-past-the-f32-wall.md), which landed:* the second program-wide property read "`f32` throughout" and the refusal list carried neither width rule, because one `dtype-f32` rule refused every program carrying a non-`f32` value before a subject was normalized. That rule is gone. `recognized_program_arithmetic` (`crates/tiler-compiler/src/request.rs`) derives the program's one arithmetic type from its own values and the same walk mints the vocabulary of that width, so the property is still program-wide and still one width per program; what moved is that the width is *derived* rather than fixed, over the two `recognized_arithmetic` admits. A single-occurrence `bf16` program consequently reaches a selected `PlanAlternative`, and since [`establish-bf16-optimizer-legality`](../../tickets/establish-bf16-optimizer-legality.md) landed hours later a multi-occurrence `bf16` region fuses under a legality proof stated for its own width rather than stopping at the fusion-legality wall. **The refusal moved rather than vanished, and its new authorities are two, neither of them this stage's.** A stated contract whose arithmetic is not the program's is refused by the *contract*, program-scoped and before any target is consulted, as `RequestError::NoApplicableNumericalContract` under the public rule `compile.request.numerics.inapplicable`; a width a profile declares no dispatch fact for is refused per target by the *profile*, as `RequestError::DTypeNotDispatchable` under `compile.request.dtype.dispatch`, where silence about the dtype resolves to disposition `Unknown` rather than to an inherited `f32` verdict. What this stage kept is the two rules now in the list above: `dtype-recognized` for a width this build spells no per-point body in, and `dtype-uniform` for a program carrying two recognized widths at once, which no single scheduled region can hold however well each width is supported alone. Stages 2 and 3 apply to any DAG and carry deliberately small rule sets — one normalization rule, and two operation-owned reassociation rules — which bounds what they *add*, not what they accept.

**Which *registered* families "an operation no region can spell" names is a two-family list, and the two families that used to occupy it no longer do.** `elementwise_family` (`crates/tiler-compiler/src/request.rs`) is keyed by the arithmetic the program derived and classifies exactly `tiler::add-f32@1`, `tiler::multiply-f32@1`, and `tiler::silu-f32@1` under `f32`, and `tiler::add-bf16@1` and `tiler::multiply-bf16@1` under `bf16`; every other operation that walk meets is refused under `operation-set` unless a neighbouring recognizer claims it first. The `bf16` row is shorter for a vocabulary reason rather than an oversight: no `tiler::silu-bf16@1` is registered to classify, and `PointwiseBf16Node` carries no division or exponential for a per-point body to land in. The activation is in that set although no `PointwiseF32Node` spells a sigmoid-weighted linear unit — what the vocabulary has to express is its *per-point body*, and `recognize_elementwise` projects that body rather than restating it, by driving `elementary::silu_point_body` into a `PointwiseF32ExpressionBuilder`. **`tiler::reindex-f32@1` and `tiler::broadcast-f32@1` are admitted**, not refused: [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](../../tickets/admit-the-structural-families-into-the-scheduled-region-vocabulary.md) landed `LogicalAccess::ReindexBijection` and `LogicalAccess::BroadcastReplication` (`crates/tiler-ir/src/schedule/model.rs`), and `recognize_structural_read` (`fn recognize_structural_read` in `crates/tiler-compiler/src/request.rs`) claims each as a *mapped read* contributing addressing and no arithmetic. *Corrected 2026-08-13 by [`admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary`](../../tickets/admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary.md):* a sourced `tiler::broadcast-f32@2` mapping is recognized as the labelled-draft `LogicalAccess::ParametricBroadcast` carrier rather than folded into `BroadcastReplication` or refused under `symbolic-extent`. Concrete literal mappings stay `BroadcastReplication`. The carrier is crate-internal request-subject tag `0x05` under `tiler.compiler.request-subject.v6`; the domain does not step. That enum still also carries `LinearIdentity`, `ScalarBroadcast`, `PackedU4LsbZeroTail`, `ReductionContributor`, and `ContractionOperand`; it has no selection or window map and no partitioned-write map. **`tiler::slice-f32@1` and `tiler::concatenate-f32@1` are what still refuse under `operation-set`, for two different reasons.** Slice holds the accepted `IndexRealizationLaw::Slice` (`slice_f32`, tag 13) and one `GovernedSliceF32` lowering, and `recognize_structural_read` does not name it, so the request boundary refuses the family because `LogicalAccess` cannot spell a literal-offset selection. Concatenate holds the accepted `IndexRealizationLaw::PartitionedConcatenate` (`concatenate_f32`, tag 12) and one lowering per admitted operand arity, and the request boundary refuses it because no scheduled or kernel construct writes a partitioned output — `const UNPLANNED_OPERATIONS` in `crates/tiler-compiler/src/policy.rs` records that reason, and [`admit-the-partitioned-copy-scheduled-region`](../../tickets/admit-the-partitioned-copy-scheduled-region.md) owns the first construct that would retire it. Both nonetheless hold a registered index-access lowering capability, among the twenty-one `governed_index_access_capabilities` returns (`fn governed_index_access_capabilities` in `crates/tiler-compiler/src/governed.rs`) — fourteen fixed-signature families plus one per admitted concatenate arity `MIN_CONCATENATE_OPERANDS..=MAX_CONCATENATE_OPERANDS` (`2..=8` in `crates/tiler-ir/src/semantic/concatenate.rs`), which is what `GOVERNED_INDEX_ACCESS_CAPABILITIES` states and `the_governed_registry_holds_one_capability_per_admitted_concatenate_arity` counts. That is what makes each a wall inside this build's own vocabulary rather than an uninstalled provider. A registered family realized as a region sequence leaves this rule by a different door: `recognize_staged_family` is keyed on `family_realizes_region_sequence`, so `tiler::rms-norm-f32@1` and `tiler::softmax-f32@1` are recognized; softmax then stops at `missing-capability` (`crates/tiler-compiler/tests/softmax_recognizer_boundary.rs`), which is not this rule. A registered family whose values name no arithmetic type this build recognizes does not reach this rule at all — `recognized_program_arithmetic` refuses the program under `dtype-recognized` first, which is the rule that replaced the `dtype-f32` gate here — so `tiler::dequantize-strict-affine@1`'s absence from the elementwise set is still not evidence about the region vocabulary: its `StrictAffineU4` operand names no arithmetic type, and `a_mixed_width_program_and_an_unspelled_width_refuse_by_different_names` (`crates/tiler-compiler/src/request.rs`) is where that attribution is checked against a neighbour of the same shape in a recognized width. `tiler::gather-f32@1` is the same class on the ordinary path: the governed target answers `DTypeNotDispatchable` for its index type before recognition, and a later `dtype-recognized` wall is still not this rule. *Corrected 2026-08-13:* this paragraph read that the set "is a three-family list", named `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` as what still refuse because `LogicalAccess` carries "no reindex map, and a broadcast that is only a rank-zero operand read once", counted "twenty `governed_index_access_capabilities` returns — thirteen named rows", and said "this document names no owner for the concatenation's". **What changed on 2026-08-07 is that `bf16` left this class.** It is recognized, so a registered `bf16` family missing from the elementwise set above *would* be evidence about the region vocabulary — and none is missing, because the three registered `bf16` families are the two classified above plus `tiler::constant-bf16@1`, which `constant_family` answers for.

One stage is neither, and it is where a general program stops today. Stage 8 was the other until 2026-08-04; its paragraph below records what it now guarantees, because the sentence that made it a wall is the sentence this section exists to keep true.

- **Stage 8 answers for every region a cover places, since 2026-08-04.** The governed physical provider spells the subject's exact occurrences against the schedule vocabulary through `crate::physical::spell_region`, and the answer is either a checked scheduled-kernel body — with the split and the workgroup tree additive beside it where the subject admits them — or a `StrategyDeclineCause::UnspellableRegion` naming which of four region-vocabulary walls it hit: a region covering part of a recognized elementwise expression, a region covering the fold together with part of its prologue, the whole program under a prologue the fused vocabulary cannot spell, or a region whose recognized read is the labelled-draft parametric broadcast carrier. The last wall is `parametric-broadcast`; a provider that cannot implement the carrier declines that named rule rather than a static-signature or generic `region-vocabulary` mask. The governed provider's context-to-offer function for previously admitted (non-parametric) subjects is unchanged, so its revision stays 1. Silence is no longer among its answers for a region a cover placed, which mattered because the governed provider was the only one a build could have: an empty offer means "this provider recognizes nothing here", and from the only provider that was indistinguishable from a coverage gap nobody named. **Since 2026-08-08 a caller may install further providers beside it** (see [the four surfaces](#the-four-surfaces-the-optimizer-may-consult)), which does not weaken the guarantee — the governed provider is always installed and still answers for every region a cover places, so an installed provider's silence costs an alternative and never coverage. The written tensor role now comes from the cover as well — a region writes a program output when the cover assigns it one and an intermediate when a materialization edge names it as producer — rather than from which whole-program recognizer matched. **What this does not do is widen the vocabulary.** The walls are still walls, each with its own owning ticket; the change converts each from silence into a typed refusal, and each widening will convert its refusal into an offer with no further change at stage 8. The parametric-broadcast wall is a capability decline, not a scheduled-region spelling. Measured on the governed five-operation program: seventeen distinct region subjects, three answered with implementations and fourteen with a named wall.
- **Stage 11 assembles a cover of any region count, since 2026-08-05.** Program assembly derives every structural quantity from the retained plan's cover and the semantic program: program inputs and keys from the declared interface, one internal value and one program-owned allocation per materialization edge sized by that edge's own element count, one output value per ordered named output, one stage per scheduled region ordered by the cover's edges so producers precede consumers, and one data dependency per edge from the producing stage to each consuming stage. `CoverAssembly::from_plan` is the single derivation and `verify_artifact_refinements` re-derives through it, so the build path and the receipt path no longer carry two independently maintained descriptions of what a cover assembles into. A cover it cannot express is a `RequestError::UnsupportedCapability { phase: "program-assembly" }` refusal naming the region — a **missing compilation capability**, which is the class the retired `"unsupported-plan-shape"` and `"artifact-strategy-cardinality"` rules got wrong by reporting a coverage gap as invalid compiler output. Its output check compares the assembled program's published keys against the semantic program's declared interface in order, for as many outputs as the program declares; the arity condition it used to carry is gone. **What this does not do is widen the reachable set**, and the cap is no longer at stage 11: `verify_region_subject_binding` (`crates/tiler-compiler/src/physical.rs`) admits a verified region only when its `semantic_members` equal one of the partitions the request-level recognizer pre-computed, or are empty for a split's combining pass, so the retained covers of a governed program are still the one- and two-region ones and the longest assembled program is still the three-stage split.

The consequence is the one a reader most needs stated, and it moved with the two walls: **neither stage 8 nor stage 11 is where a general program stops any more — the region vocabulary is.** Both now answer for whatever a cover places, and what no legal cover can yet *contain* is a region outside the recognizer's three pre-computed member sets, because a schedule over any other member set fails the request-subject binding before it is ever proposed. The three widening tickets the stage-8 paragraph names own that boundary, and each converts a refusal into an offer with no further change at either stage. *Corrected 2026-08-05 by [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](../../tickets/admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md), which landed:* this sentence read that the request boundary's `output-arity` refusal remained load-bearing because the cover stated which regions publish *an* output but not which named result each retains. `CoverRegion` now carries the named-result value ordinals its candidate retains, `verify_cover` checks that projection at the same step that binds members, content, and label, and `CoverAssembly::from_plan` attributes each declared output to its publishing region by value. Both `output_count() != 1` guards are gone and ordered multi-output programs compile; what a multi-output program is refused for now is the *shape* of its outputs — two walks sharing an occurrence under `output-partition-overlap` — never their number.

**The two halves now have separate owners, filed 2026-08-04 by the baseline definition rather than by the audit.** [`define-the-minimum-correct-physical-realization-profile`](../../tickets/define-the-minimum-correct-physical-realization-profile.md) defined what each must guarantee — recorded as the [minimum correct physical realization profile](../research/program-planning/minimum-correct-physical-realization-profile.md) — and filed [`derive-physical-proposals-from-the-cover-region-subject`](../../tickets/derive-physical-proposals-from-the-cover-region-subject.md) for the stage-8 half and [`assemble-a-kernel-program-from-an-arbitrary-cover`](../../tickets/assemble-a-kernel-program-from-an-arbitrary-cover.md) for the stage-11 half, the second depending on the first because a plan reaches assembly only when every region of its cover has an admitted implementation. [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](../../tickets/admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) depended on the assembly ticket and relaxed both `output_count() != 1` guards once it landed; it never owned the plan-shape match, which is general-baseline work a single-output four-region program needs just as much.

**The explain consequence the baseline definition corrected, and what discharging it looks like.** The audit read a dropped cover as reaching selection with no attribution at all; the precise version was that a rejection *is* constructed — `select_physical_plans` inserts a `PlanRejection::RegionUnimplemented` for every cover region with an empty admitted set, 38 per governed compile — and that three separate defects kept it from a reader. All three closed 2026-08-04 with the stage-8 generalization. `SelectedPortfolio::rejections()` now has a production reader that emits one record per cover and region, naming both and caused by that region's own frontier enumeration. The frontier record is keyed by the region's canonical occurrence label rather than `region:{role}`, so the fourteen subjects sharing the role `unrecognized` are fourteen subjects in the trace and the role travels beside them as a fact. And the first-sighting deduplication, now over that key, records every enumerated subject instead of one per role. The governed compile's record census moved from 8 frontier records and 2 declines to 34, 16, and 38 coverage gaps.

Semantic, index, schedule, program/buffer, and structured-kernel verifiers have
separate authority. Target feasibility cannot repair intrinsic invalidity;
costing observes only candidates that have passed the applicable gates.
`Intrinsic` and structured-kernel refinement failures therefore remain invalid
compiler output; only a checked target/resource rejection can contribute to a
valid empty physical frontier.

An index-region refinement refusal at stage 6 is the one failure both words could name and neither class covers. It is not invalid compiler output, because the compiler produced nothing wrong: a provider it resolved emitted a region that does not realize the occurrence. It is a missing compilation capability — the installed authority could not lower a program it was handed — and it is reported as one, at the refinement stage and against the exact occurrence. It is never a target rejection.

Search implementations may interleave cover and local-frontier exploration,
feed pruning information in either direction, and lazily schedule only regions
retained by viable covers. Such feedback is implementation freedom: it cannot
make a cover receipt prove schedule feasibility, or a local frontier prove
whole-program coverage.

## Lowering capability resolution and index-region refinement

Stages 5 and 6 answer different questions about one recognized occurrence, and the contract keeps them apart. Resolution selects *which* installed authority lowers the occurrence. Refinement proves that the work that authority emitted *realizes* it. Neither registration nor a successful builder construction is refinement evidence.

### Resolution is unconditional and fails closed

**Fact.** `crates/tiler-compiler/src/lowering.rs` resolves exactly one `LoweringFamily::IndexAccess` capability for every recognized occurrence, against the frozen lowering-capability registry the compilation request carries, and does so for every occurrence before the first cover is enumerated. There is no shape recognizer behind it, no default provider, no approximate provider, and no priority order between candidates.

Two dispositions are distinct, and neither is a preference:

- **absent** — no installed capability matches the occurrence's operation and signature. This is a *deferred* capability: the installed authority was never extended to this occurrence.
- **contended** — more than one matches. This is a *disproved* checked predicate: the authority was extended, and its extensions contradict each other. Reporting it as deferred would suggest a missing registration when the defect is a contradiction between two present ones.

Both stop the compilation with a typed missing-compilation-capability failure attributed to the exact occurrence. Neither narrows the portfolio, because an occurrence nobody can lower has no valid plan at all — retaining a smaller portfolio would return plans for a program the installed authority cannot compile. This is why a missing *lowering* capability behaves unlike a missing per-operation *fusion numerical* capability: the latter leaves every occurrence lowerable and only makes one fused grouping unprovable, so it rejects one alternative.

**Fact — lowering provenance is a registry resolution, not a compile-time table.** An artifact construction plan's `lowering_providers` is the set of `{provider identity, capability revision}` pairs resolution returned, deduplicated in canonical ascending order. `crates/tiler-compiler/src/program.rs` re-derives that set from the request's own installed registry when the plan is built and again when the portfolio is re-verified, and refuses a plan whose recorded provenance differs, so a receipt cannot name a provider the registry never resolved. Several occurrences of one family contribute one entry; one provider owning two capabilities at different revisions contributes two. ADR 0072 is why both halves are retained: a provider revision is the admitting authority's own output-affecting revision, and a capability revision covers the exact lowering that provider registered for one family and signature.

**Fact — an external caller can install that authority on the ordinary compile path.** `session::InstalledCapabilities::installed` carries a caller's `FrozenLoweringCapabilityRegistry` with its exact `FrozenScalarRegistry`; the immutable realization-law snapshot is derived from the exact semantic registry carried by the program, not installer-selected authority. `session::CompileRequest::with_capabilities` installs the pair, and `session::compile` consumes the request. Request preflight verifies full lowering/scalar/program semantic coherence, including the realization-law sidecar, before retaining that program-derived law authority. `session::compile_governed` composes the governed singleton request and calls the same entry point; it is not a privileged capability resolution path.

### Refinement requires discharged index-domain evidence

**Fact.** `legality::refine_index_region` drives the resolved provider through the canonical `tiler-ir` index builder and then proves, independently of the provider, that the emitted region realizes the occurrence: the ordered operand and result interface agrees in type, shape, arity, and aliasing; the reached scalar authority stays inside what the capability declared it may emit; the capability's and the region's semantic type authorities agree; and every ordinary write carries complete unique-ownership evidence. A refined occurrence's explain record carries exhaustive finite evidence, which is the strongest class this stage can produce and is weaker than a sound proof.

A malformed region, or a well-formed region that does not realize its occurrence, is a genuine rejection and fails closed. The artifact plan names the resolved provider as that occurrence's lowering authority, and that claim has to be true.

**A structurally verified residual is neither refinement evidence nor a target rejection.** `tiler_ir::index` retains each unresolved read-bounds atom with an exact `InsufficientFacts`, `UnsupportedFragment`, or `ResourceLimit` reason. The last names the exhausted proof resource, governed limit, and required amount. The region is valid analysis state because nothing disproved the predicate, but no later stage may treat it as refined, insert an unattributed physical guard, or allow it into an executable frontier.

`legality::refine_index_region` revalidates scalar authority and binds the complete operand/result interface before inspecting residuals, so an unresolved bound cannot mask an independent provider defect. If the provider otherwise conforms but any residual remains, refinement returns `IndexRefinementOutcome::Pending`, not an error or refinement evidence. The pending state owns the exact verified region, semantic occurrence, frozen authorities, and checked bindings; it does not copy region-local predicates away from the region that gives their handles meaning, re-run the provider, or mint a refinement identity.

The compiler-owned semantic-discharge policy consumes that pending state before cover enumeration and chooses only a bounded proof-work budget. IR's closed exact-finite evaluator assesses each exact region-owned obligation once as `Proved`, `Disproved`, or `Unknown`; no compiler or provider callback can construct a claim or proof authority. IR alone seals an all-`Proved` result over the canonical region identity, obligation key, fixed proof authority and revision, and exhaustive proof payload, while compiler refinement content retains those same IR proof objects rather than minting a second authority. Any `Disproved` or `Unknown` result refuses the occurrence atomically. `Disproved` establishes that the provider emitted an invalid realization and is therefore invalid compiler/provider output; `Unknown` establishes only missing support and remains an unsupported capability. The failure trace emits one `semantic-discharge` assessment per canonical obligation with its exact key, predicate kind, claim, rule revision, original verifier reason, and current discharge result. Original and current resource stops have separate fields; a `u128` required amount is retained without narrowing as named upper and lower 64-bit halves. Exact exhaustive finite evidence can authorize residual refinement; empirical measurement cannot, and a sound-proof lane remains unsupported until IR admits a closed validated certificate language.

[ADR 0109](../decisions/0109-fail-closed-before-executable-planning-when-index-domain-proof-is-unknown.md) accepts this before-cover refusal as the normative successor to ADR 0078's historical `Ok` and “the plan stands” behavior. It preserves ResourceLimit as Unknown rather than disproof, requires every produced assessment to remain explainable, and gives any assessed Disproved claim overall precedence; it changes no executable or identity surface.

The production authority exactly evaluates every current public index-expression form over the subject access's complete finite logical domain with arbitrary-precision integer arithmetic. It returns `ExhaustiveFinite` only after every point passes, returns the first canonical counterexample when an atom is false, and retains `Unknown` when the expression fragment is unsupported or either governed budget is exceeded. Completion owns one aggregate ledger for the whole call. It resolves each access domain and predicate extent once, groups obligations with the same ordered static domain, constructs one union expression DAG per group, and evaluates that DAG once per point for all predicates in the group. Cells charge semantic planning, every DAG node and edge, coordinate set/advance work, memo clearing, and predicate checks; a separate 64-MiB cumulative integer-byte budget charges literals, operands, conservative arithmetic result widths, and predicates. A group reserves both evaluation resources atomically, and an exhaustion reports the cumulative requirement against the caller's original whole-call limit for that obligation and every later unassessed obligation. Count overflow is unsupported rather than reported as an invented exact requirement. The mandatory assessment vector is bounded by the region's hard structural predicate limit and is not caller-budgeted; caller budgets govern semantic planning and evaluation. These operational charges do not change the mathematical v1 proof identity, which records the exhaustive claim rather than the implementation strategy used to establish it. The completion budgets are intentionally distinct from the structural verifier's budget: they permit a more expensive compiler-time proof without making the fallback unbounded.

This authority is representation-neutral by construction. An `IndexDomainPredicate` describes only logical coordinates and extents; nominal booleans and integers, parameterized complex values, encoded quantized values, packed bits, component buffers, and storage layouts cannot affect its truth. Tensor-value semantic preconditions and reconstruction of physical representations are separate contracts. They require a real semantic producer and complete logical-to-physical component metadata rather than a dense-`f32` callback, and they cannot substitute for coordinate proof.

A proof-resource limit found beside any hard structural, disproval, or write-ownership diagnostic cannot turn the result into a verified region. The independent refusal remains the build result, with the resource stop retained as secondary typed evidence.

**Measurement.** Cells are charged only where the cheaper interval proof fails or a write is not a proved coordinate permutation. Every governed lowering's writes are coordinate permutations and its reads are bounded by its own dimensions, so the governed profile charges nothing at any recognized size — measured at `[70_000, 2]` in `pipeline::conformance::governed_lowerings_never_charge_the_exhaustive_proof_budget`. That is why refinement is attempted for every occurrence rather than gated on a size threshold, and it bounds the claim to the governed lowerings: a registered provider whose emitted access is neither interval-provable nor a proved permutation can and does trip the budget.

### Scalar-authority conformance is containment, not equality

**Fact.** A capability declares the scalar operations it may emit, and refinement requires the region's reached scalar authority to be *contained in* that declaration — the region must reach nothing beyond what was declared. The rule formerly required the two sets to be equal.

Equality is unsatisfiable for a shape-general provider. One capability lowers every occurrence of its operation family and signature, while which of the declared scalar operations a given occurrence actually needs depends on that occurrence's shapes and attributes. A `tiler.strict-serial-sum-f32` occurrence with a single contributor reaches no scalar operation at all, one over an empty reduced domain reaches only the identity constant, and one over many contributors reaches the add: three reached sets, one capability, one declaration that must cover all three. Requiring equality would have forced a provider registration per program shape, which is the opposite of what a declared emitted set is for.

The safety property is unchanged, because equality never carried it. Containment still refuses every region that reached an authority the capability was not admitted to emit, which is what makes the declaration a bound on what a lowering can compute. What equality added was a *completeness* requirement — that every declared operation actually be exercised — and that was the defect: it is a claim about one occurrence rather than about the capability, and no correctness argument rested on it.

### Maturity boundary

Resolution and refinement are implemented and unconditional on the ordinary compile path. An out-of-crate caller has installed an index/access lowering provider written only against the public `capability` surface, driven a recognized occurrence end to end through `session::compile`, and observed the artifact plan recording that provider as the lowering authority. The companion negative test omits one family and fails closed, so a passing installation test cannot be explained by `with_capabilities` ignoring its argument. This is a tested public installation guarantee, and it covers the whole surface rather than part of it: `tiler_compiler::capability` registers exactly one lowering family and it is the one the compile path resolves.

**Fact — a lowering provider has no latitude over the realization it emits, per-point scalar work included.** The registered `tiler_ir::index::IndexRealizationLaw` is what a realization must be: `ResolvedIndexRealization::verify_sequence` builds the expected realization from the law and refuses the candidate unless `expected.identity() == realization.identity()`, and an occurrence whose operation registers no law refuses as `MissingRealizationLaw` before a provider is driven at all. The scalar applications are inside those compared bytes, and the law names them exactly — `PointwiseBinary` carries the applied `ScalarOpKey`, and the staged root-mean-square form fixes its whole fold epilogue rather than parameterizing it. `pipeline::conformance::a_lowering_cannot_replace_the_semantic_providers_realization_law` is that refusal observed on the ordinary compile path against an externally installed provider offering a structurally valid alternate multiply.

**Fact — the scalar-lowering family's absence from the compile path was a classification error rather than a missing evidence row, and the family is gone.** A scalar provider resolved anywhere on this path would have had exactly one admissible output, the per-point sub-expression the law already builds, decided by an authority the provider did not own; and it would have had nowhere checked to return it, since no `refine_scalar_*` authority exists anywhere in the tree — reproduce with `grep -rn "refine_scalar" crates/`, which returns nothing. Under [ADR 0078](../decisions/0078-name-the-intended-public-extension-seams.md) item 1 a surface whose output does not re-enter the ordinary checked path is not a seam, so this contract owed the row's removal rather than an installation test to fill it. [ADR 0105](../decisions/0105-retire-the-scalar-lowering-provider-seam.md), accepted by Tom on 2026-08-06, supersedes the ADR 0078 classification and decides the retirement; [`remove-the-scalar-lowering-family-from-the-compiler`](../../tickets/remove-the-scalar-lowering-family-from-the-compiler.md) executed it. The removal is identity-preserving — the capability key encodes the family's stable tag, index access is tag one, and its `key_token` is unchanged — so no ledger, golden, or identity pin moved for it.

**Fact — since the activation was admitted, refinement's proof is also evidence about what the request boundary projected.** Two authorities in `tiler-compiler` realize one elementary composition: the governed index-access lowering emits it as `tiler_ir::index` scalar applications, and the request boundary projects it into the physical `PointwiseF32Expression` vocabulary the scheduled region carries. Neither writes the chain. `elementary::silu_point_body` (`crates/tiler-compiler/src/elementary.rs`) states `x / (1 + Exp(-x))` once against an abstract per-point sink, and the two realizations are two implementations of that sink — `GovernedElementarySink` in `crates/tiler-compiler/src/governed.rs` and `PointwiseExpressionSink` beside the statement itself. Because the two spellings are one statement, `legality::refine_index_region` proving that the emitted region realizes the occurrence is evidence about the projection as well, and that is the reason a projecting boundary does not create a second unchecked authority: were the boundary to restate the provider's per-point arithmetic independently, there would be two claims about one meaning and only one of them checked.

**Measurement — the composition was observed rather than argued, and the observation's boundary is one perturbation.** Perturbing `silu_point_body`'s division operands was watched failing before any region was scheduled, at the refinement refusal: `LoweringError::Refine` (`crates/tiler-compiler/src/lowering.rs`) carries the stable reason `refinement-refused`, and `lowering_failure` (`crates/tiler-compiler/src/pipeline/planning.rs`) reports it as `RequestError::UnsupportedCapability { phase: "lowering", rule: "refinement-refused" }` — the missing-compilation-capability class [named stages and verifier boundaries](#named-stages-and-verifier-boundaries) assigns a stage-6 refinement refusal, never a target rejection. [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](../../tickets/admit-the-registered-unary-families-at-the-compiler-request-boundary.md) records the perturbation and its observed failure. What this establishes is that the shared statement puts one body's projection under refinement's authority; it is not a mechanical check that a later elementary family inherits, and a family whose projection is written separately would inherit nothing.

## Bounded hierarchical search

**The search formalism is selected. This section previously read that "a Cascades-style memo is one possible implementation technique, not a committed architecture", committing to the search's obligations and deliberately not to its mechanism; [the rewrite-search formalism record](../research/region-search/rewrite-search-formalism.md) closed that gap on 2026-08-05 against the database-optimizer, equality-saturation, tensor-schedule-search, and phase-ordering literatures, and this contract now states the outcome rather than the question.** The selected formalism is a **staged, alternative-retaining** search: every stage retains each legal alternative its rules propose, no stage prunes a semantic alternative on estimated cost, and each stage's retention authority is the one appropriate to what that stage decides — semantic applicability and numerical permission where meanings are proposed, stated contract preference where meanings are chosen between, typed feasibility and then Pareto dominance where implementations are compared. The derivation is not restated here; the record carries it, while [the four surfaces](#the-four-surfaces-the-optimizer-may-consult) states the invariant that makes the staging work and [the review obligation](#the-review-obligation) states the named rule that guards it.

**The record numbers the formalism's four levels one to four, and this contract numbers eleven pipeline stages; the two numberings collide and the mapping is stated here so a reader of both is not misled.** The record's *semantic exploration* is stage 3, `ExploreLogicalAlternatives`. Its *contract grouping* is the per-alternative readmission and grouping [the algebraic portfolio](#implemented-first-algebraic-portfolio) states, which sits between stages 3 and 4 and is not itself numbered. Its *physical enumeration* is stages 4, 7, and 8, whose complete plans stage 9 joins, and it is the memoized level. Its *global selection* is the cross-candidate flattening that follows program assembly, inside the selected contract group. The collision is live rather than hypothetical: the two deferred tickets below say "stage one" in their titles and "stage 3" in their trigger logs about the same stage, and both are right under their own numbering.

The durable concepts are unchanged by the selection: contract-conforming semantic alternatives, explicit region candidates, bounded implementation frontiers, and deterministic complete-program selection. The term `memo` remains reserved for an implementation that actually groups equivalence classes and performs goal-directed property search, and the selection places the memoized level at physical enumeration rather than at semantic exploration. `Partitioner`'s `memo: BTreeMap<CoverageMask, MemoEntry>` in `crates/tiler-compiler/src/cover.rs` is therefore correctly not called one: it memoizes the coverage completions of a covered-set state, has no group/expression separation, and holds no logical alternatives.

**Three formalisms are eliminated, and the third is weaker than the first two; the contract records the difference rather than flattening it.** Destructive cost-pruned rewriting to a fixpoint cannot reach a mutually-enabled composition, and interleaving its phases to a fixed point does not repair that — the obstacle is not the order but that a cost comparison is consulted before the composition is complete. Equality saturation is eliminated *as the whole search*, on four independent grounds: published tractability evidence against unguided saturation over schedule-shaped rewrite spaces, extraction that is NP-hard (read) and constant-factor inapproximable (relayed through one citation the record marks as unread), the tension between an e-class asserting interchangeability and a typed feasibility or numerical refusal that is not a cost, and e-graph cycles against the DAG convexity [region formation](#region-candidate-formation) maintains by construction. A Cascades memo as the whole search is eliminated on this contract's *ordering of three pruning authorities* — numerical-contract preference, then feasibility, then cost — where Cascades has one; the record answers Orca as the strongest objection and then states plainly that this step is an argument from failure to construct such an encoding rather than a proof of impossibility. It is the elimination to attack first, and attacking it means exhibiting a single property-keyed memo that preserves both the ordering and the typed refusal reasons.

**What the selection does not settle is the representation inside semantic exploration, and leaving that open is deliberate rather than unfinished.** Stage 3 retains at most three whole-program proposals today — the preserved baseline plus one per enabled registered rule — and a rewrite vocabulary rich enough for flash-class discovery needs a real alternative structure there. An e-graph over the *semantic algebra alone*, with enumeration into contract grouping replacing cost-based extraction, is the candidate, and adopting it keeps every elimination above outside only under three constraints: no schedule-space concept enters the e-graph, no feasibility or numerical-contract fact lives in an e-class, and per-e-node rule provenance or an on-extraction explanation survives to satisfy the typed alternative identity [the algebraic portfolio](#implemented-first-algebraic-portfolio) requires. [`probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary`](../../tickets/probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary.md) owns the measurement and [`decide-whether-stage-one-semantic-exploration-adopts-an-e-graph`](../../tickets/decide-whether-stage-one-semantic-exploration-adopts-an-e-graph.md) owns the decision; both are `deferred` with stated triggers, and a tractable algebraic e-graph cannot revive the elimination above, which was never about that space.

Examples of equivalent expressions include:

- consecutive reindexes versus one composed access map;
- a pointwise operation before or after a reindex when domains permit;
- alternative associations of a future multi-input einsum contraction, under a numerical policy that permits the distributivity the regrouping consumes.

Logical equivalence is policy-relative, so the third example is a group only where that policy holds; the first two hold unconditionally. No expressible policy holds for the third today, so it names a reserved equivalence group rather than an available one. See [logical exploration](#logical-exploration) for the permission each rewrite consumes.

Recomputation, materialization, fusion, and register residency are physical
implementations of one logical DAG. They do not create new logical equivalence
groups.

The first implementation should use bounded exploration: canonical operation
and value keys, deterministic rule order, small alternative sets, dominance
pruning *over physical implementations*, and explicit search budgets. Tiny
graphs should have an exhaustive oracle in tests so heuristic completeness and
plan quality can be measured. That discipline outlives the formalism question it
was written before: an oracle is what makes a retention claim checkable rather
than a claim about the search that produced it. Dominance is qualified to the
physical level deliberately — applying it to semantic alternatives is the
retain-then-prune-locally strategy the phase-ordering witness watches fail, and
it fails there for the same reason greedy rewriting does.

Five of the first deterministic safety budgets bound region formation, as the
`region_*` fields of `DeterministicBudgets`: 62 semantic occurrences per region
(`region_members`), 3 boundary outputs (`region_boundary_outputs`), 80 live
boundary/internal values (`region_live_values`), 32 candidates per seed
(`region_candidates_per_seed`), and 10,000 candidate expansions per compilation
request (`region_expansions`).

**The first three are authoring-side bounds sized from the governed declaration; they are not runtime derivations from the submitted program.** A single-stage region's members are a subset of the program's occurrences and its live values are subsets of program values, so the governed numbers set `region_members` equal to `semantic_operations`, `region_live_values` equal to `semantic_values`, and `region_boundary_outputs` to the declared-output envelope. Realization sequences make the first two axes independently meaningful: region formation counts stage atoms, and handed intermediates add live values that semantic occurrence/value counts do not contain. Tom therefore accepted on 2026-08-11 that all three remain explicit request-policy fields and that all fourteen budget values stay in the canonical request subject. A future public budget policy must be one required complete typed value with no default or per-field fallback. Budget bytes bind the compiler-internal request/evidence subject and explain qualifier; they do not directly enter artifact or cache identity, which moves only when selected packaged content changes.

Three more bound the stages downstream of it:
1,024 retained complete covers (`region_covers`), 100,000 partition-search
expansions (`region_cover_expansions`), and 4,096 complete-plan combinations per
cover source (`physical_plan_combinations`). Producer duplication is
disabled outside oracle tests in the initial implementation. Hitting any of these
stops only that growth path, emits an explain reason, and never removes either
coverage extreme — the singleton/unfused regions or the whole-program one. These
defaults are calibration inputs, not correctness constants.

**What a budget may be is a contract, not a convention, and two shapes are excluded by name.** Every budget above is a count of *work performed* — occurrences, boundary outputs, live values, candidates, expansions, covers, plan combinations, rewrites — and a new one is added at the stage that owns the growth it bounds, naming which growth path stopped, on which subject, with the exact limit and demand. **A wall-clock time-out and a cost threshold are both forbidden as budget axes**, and the same requirement excludes them: the same request must compile to the same portfolio twice. A time-out makes the output a function of machine load; a cost threshold makes it a function of how quickly a good-enough plan happened to be found, which is a different program on a differently-ordered search over the same candidate set. The comparison is Orca's, whose multi-stage optimization terminates a stage on rule-subset exhaustion, a time-out, *or* a plan beating a threshold — [the rewrite-search formalism record](../research/region-search/rewrite-search-formalism.md) derives which of those three transfer. Rule-subset staging is the one that does, and it is a budget axis Tiler does not yet have: running cheap rules before expensive ones is compatible with deterministic counting, because exhausting a rule subset is itself a count.

**Fact — a ninth budget was named here and never became real, and the condition that made naming it acceptable has expired.** This list previously carried "8 nondominated implementations per region" as forward-looking, on the stated ground that the per-region physical-implementation frontier was not implemented and that the entry would become real when that stage landed. The stage landed: `enumerate_frontier` runs on the ordinary compile path from `pipeline/planning.rs`, and `ImplementationFrontier::non_dominated` retains the frontier. It retains it as a pure Pareto filter with no count bound, and `DeterministicBudgets` in `crates/tiler-compiler/src/request.rs` has no corresponding field, so the retained set is bounded only by how many proposals the installed providers offer — three for a reduction admitting every strategy under the governed provider alone. **The condition that made that bound self-limiting expired on 2026-08-08**, when [ADR 0090](../decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 2's installable seam landed: a caller may now install providers additively, so the retained set is bounded by caller-supplied code rather than by this build's own vocabulary. Whether the frontier owes a retention budget is [an open decision](../../tickets/decide-whether-the-implementation-frontier-owes-a-retention-budget.md) whose stated trigger has therefore fired; this list still states neither answer as a fact.

**Most of the budgets above bound a *search*, so exhausting one costs an alternative while complete coverage survives — and three of them do not, which is a distinction this paragraph used to erase.** `region_members`, `region_boundary_outputs`, and `region_live_values` bound one region's admissible *shape*: they declare the largest region the profile forms at all, and a program whose only implementable cover needs a bigger one has no plan under them however long the search runs. That refusal is reported as `BudgetExhausted` naming the bound, because the caller's action is to widen it, and never as `NoFeasiblePlan`. `NoFeasiblePlan` retains hard target refusals and conservative mixed or structural empty portfolios whose complete causes do not establish a pure capability gap; an exhaustive, non-empty population blocked only by supported-but-unspellable region vocabulary is instead `UnsupportedCapability`. A budget stop establishes neither and remains distinct. The genuine search bounds — `normalization_rewrites`, `region_candidates_per_seed`, `region_expansions`, `region_covers`, `region_cover_expansions`, `physical_plan_combinations` — cost only alternatives, and region formation and cover enumeration each keep that true by emitting **both** extremes of the partition lattice before any search begins: every singleton region and the whole-program region, the fully-materialized cover and the fused one. Charging the whole-program region against the search that discovers the partitions between the extremes is [a defect this contract's earlier unqualified sentence licensed](../../tickets/region-expansion-exhaustion-loses-the-only-feasible-plan.md), because breadth-first growth reaches it last. `tiler_ir::index::MAX_EXHAUSTIVE_PROOF_CELLS` is not one of them: it bounds the structural region verifier's finite fallback, and exhaustion retains an exact residual rather than granting permission. Semantic discharge may still prove that residual under the compiler's separate work budget, which cannot exceed IR's `MAX_FINITE_DOMAIN_PROOF_CELLS` hard bound. Exhausting that completion budget returns `Unknown` and refuses before any alternative containing the occurrence can be formed. [Refinement](#refinement-requires-discharged-index-domain-evidence) states the boundary. A reader must not treat a lost proof as a target rejection, a lost search alternative, or execution permission.

**Correction — 2026-08-13.** The public `BudgetExhausted` payload no longer has a two-way `Bounding`/`Truncated` split or a field named `actual`. [`BudgetResource::refusal`](../../crates/tiler-compiler/src/request.rs) is the sole authority and reports one of three provenances: `ExactDemand` (completed program or candidate counts, including the three shape rows named above), `PlanningUpperBound` (`regions`, `host_expression_nodes`, `buffers` — a conservative envelope computed before a plan is chosen; a particular reachable plan may use less), or `SearchLowerBound` (the genuine search bounds named above; the compared value is not the budget required for success). The compared public field is `reported`.

## Rule classes

### Semantic normalization

Normalization chooses a canonical form and must terminate deterministically:

- resolve axis names and ellipses;
- canonicalize reductions and output-axis policy;
- compose permutations and legal split/merge chains;
- canonicalize explicit broadcast/repeat axis mappings;
- eliminate identity reindexes and no-op casts;
- normalize constants and dtypes;
- remove dead values.

Normalization must not silently change floating-point evaluation order.

### Logical exploration

These rules add alternatives:

- push a view through a pointwise expression;
- add contract-conforming alternatives over named pointwise operations;
- choose alternative associations of a tensor contraction only when the effective distributivity, reassociation, and operand-permutation permissions all authorize the regrouping;
- reassociate arithmetic or reductions only when numerical policy permits.

Each rule above names the effective numerical permission it consumes, as ADR 0011 requires of every semantic rewrite, and a rule that names none consumes none. Pushing a view through a pointwise expression relocates reads without changing which scalar operations compute a value, and initial floating-point operations are value-only under ADR 0020, so adding or removing an evaluation of one is not observable. This stage's guarantee that it adds only proved contract-preserving forms checks each rule's stated precondition; it does not supply a missing one.

### Implemented first algebraic portfolio

**Fact — rule authority is operation-owned.** The frozen semantic definition for an operation declares whether ordered associativity is part of that operation family's algebraic capabilities. The compiler's named add and multiply reassociation rules consume that declaration; matching Rust type shape or recognizing a familiar operation name is not authority to reassociate an extension.

**Fact — the guards are separate and observable.** Each rule first checks its semantic applicability: the operation declares ordered associativity, the program contains a left-associated three-leaf chain with equal operation keys and attributes, and rebuilding the right-associated form through the frozen registry infers the same result type and shape. It then checks the effective numerical contract's reassociation dimension independently. Per-rule configuration is a third guard, so disabling add does not disable multiply. Semantic, numerical, and configuration declines retain distinct typed assessments and stable reasons rather than collapsing into “no proposal.”

**Fact — exploration is baseline-preserving and bounded.** Canonical normalization still produces one deterministic graph. Algebraic exploration runs afterward and retains that graph as the baseline while each enabled registered rule contributes at most one whole-program proposal. The rule registry's canonical identity order makes the set reproducible, the rewrite engine structurally revalidates every proposal through `SemanticProgramBuilder`, and the governed rewrite budget abandons the algebraic proposal set as a whole while preserving the baseline and recording the exact limit and demand.

**Fact — every surviving semantic candidate re-enters the request boundary.** The baseline and every rewrite proposal are independently verified against the caller's original shape environment, stated numerical preferences, budgets, targets, and frozen capabilities. A readmission refusal is invalid compiler output rather than a candidate that may be silently dropped, because a semantics-preserving rewrite of an admitted program must remain admissible.

**Fact — contract preference precedes cost.** Readmitted candidates are grouped by their resolved numerical contract. The compiler evaluates groups in the caller's stated preference order, plans all candidates in a group, and selects the first group containing any feasible complete plan. Later groups are explicitly preference-pruned without physical planning. Structural cost and dominance are compared only inside the selected contract group; no cheaper plan under a different contract can buy a change in meaning.

**Fact — planning and selection preserve candidate ownership.** Each evaluated semantic candidate runs through region formation, capability resolution, index refinement, cover enumeration, physical frontiers, complete-plan selection, KIR refinement, and program assembly under its own verified target request. The compiler then flattens the feasible programs from the selected contract group, globally selects a nondominated program, and re-derives every alternative identity from the owning rule origin, exact semantic program, exact verified request, and physical plan. A mismatched owner key, missing or extra alternative, forged winner, or identity derived from another candidate is invalid compiler output.

**Fact — the first reassociated programs now reach verified physical products.** A one-input, one-output, three-leaf same-family `f32` add or multiply chain that passes the operation-owned associativity guard and an admissible numerical contract can complete ordinary compilation through a bounded verified `PointwiseF32Expression` schedule projection, exact structured-KIR lowering, verified program assembly, and global candidate selection. The physical projection retains ordered operands, exact constant bits, association, DAG sharing, and an explicit root; it is a closed physical vocabulary for the implemented `f32` profile, not a generic scalar IR and not a second semantic operation authority.

**Fact — broader physical reachability remains explicit and unsupported.** Other dtypes, other scalar operations, conversions, predicates, mixed-precision or multi-result expressions, and compound encoded or quantized values do not become executable merely because semantic IR can represent them. Each needs its own complete operation, numerical, schedule, KIR, backend, reference, identity, and ABI vertical; until then the applicable capability or verifier rejects it by name rather than projecting it into `PointwiseF32Expression`. *Amended 2026-08-07:* one dtype has since built that vertical, and naming it is what keeps this a statement of the condition rather than a claim that `f32` is the only width reaching a physical product. `bf16` now has its registered semantic signatures, an exact-rational reference evaluator, a numerical contract of its own key domain, `PointwiseBf16Expression` and the kernel constructs under it, `bfloat` Metal emission, artifact encoding and identity, index-realization laws, three governed index-access lowerings, and — since [`establish-bf16-optimizer-legality`](../../tickets/establish-bf16-optimizer-legality.md) — fusion legality derived at its own significand, so a recognized `bf16` program is projected into `PointwiseBf16Expression` rather than refused. Nothing about the rule changed and nothing was inherited from binary32; the widths, scalar operations, conversions, predicates, and encoded values with no such vertical are still refused by name. The reassociation rules this section states remain `f32`-only for a separate reason that is not reachability: no `bf16` family declares ordered associativity, so no rewrite is proposed for one.

"Contraction" in the third rule is the tensor sense — summation over indices shared by two or more operands — and its association is a numerical question before it is a search question. A reassociation permission is necessary and is never sufficient. Rewriting `(AB)C` to `A(BC)` forms entirely different rounded products rather than regrouping one reduction's contributors: the two programs' contributor sequences share no value and are indexed by different axes, so neither is a grouping of the other. [Numerical semantics](../numerical-semantics.md#distributivity-is-outside-the-order-contract) therefore classifies the rewrite as consuming distributivity — a third dimension, independent of reassociation and operand permutation, that no permission in that contract grants. [ADR 0080](../decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) is the accepted decision behind that classification and behind the rejection wording below. The rule fails closed under every contract Tiler can express, and does so as a settled position rather than pending one.

That rejection must name the missing distributivity dimension. Reporting a forbidden reassociation would be inaccurate and would imply that a contract permitting reassociation would admit the rewrite, which is exactly the inference the numerical contract forbids.

`tiler_compiler::session::NumericalContract` in `crates/tiler-compiler/src/session.rs` is the contract a caller states, composed one dimension at a time from a strict base rather than chosen from a closed set; its five named points are strict, flush-to-zero, relaxed, permit-reassociation, and flush-and-reassociate. A contract that resolves `reassociation` to `Permitted` admits ordered regrouping for operation families that declare ordered associativity, and one that forbids it does not — [Numerical semantics](../numerical-semantics.md) derives why permit-reassociation is a different meaning rather than a relaxation of relaxed, and why omission resolves strict — and **no** statable contract carries a distributivity dimension, because the dimension does not exist in the vocabulary at all. **Corrected 2026-08-04 by the general-pipeline audit; the reachability caveat that stood here is now false and its removal strengthens the paragraph.** This read "`normalize_serial_sum` in the same file independently rejects any program without exactly one input, so no tensor contraction reaches the compiler at all. That reachability limit will lift as the compiler grows." Both halves are wrong at this revision. `normalize_serial_sum` does not exist — `grep -rn 'normalize_serial_sum' crates/` returns only a historical mention in one test's doc comment — and it was never in `session.rs`. A two-operand contraction *does* reach the compiler: `select_supported_strategy` in `crates/tiler-compiler/src/request.rs` classifies the occurrence producing the output and routes a `tiler.strict-tensor-contraction-f32` root through `normalize_contraction`, at any declared input arity. So the association rewrite is now reachable-and-refused rather than unreachable, which is the stronger position: the distributivity gap is the whole reason, not a durable reason standing behind a temporary one.

The distributivity gap is that reason: the rule would still fail closed on a compiler that accepted contractions under a contract that permitted both reassociation and permutation. The same contract's separate `contraction` field is ADR 0015's fused-multiply-add permission, which governs whether a tensor contraction's own `accumulator + a * b` step may round once; no one of these three permissions implies another.

### Region-candidate formation

Region rules propose, but do not automatically select, candidates with explicit
member operations, boundary values, retained results, materialized edges, and
duplication policy:

- pointwise plus pointwise;
- reindex plus pointwise;
- pointwise prologue into a reduction;
- pointwise epilogue after a reduction;
- compatible sibling consumers as a future multi-output kernel;
- supported prologue/epilogue around a semantic operation with an opaque
  library implementation;
- an explicit split/materialize alternative at eligible edges.

Each initial candidate is nonempty, connected, and convex in the operation DAG:
a path between included operations may not leave and re-enter the region.
Explicit duplication creates separately accounted occurrences; it never
silently waives convexity. Values consumed outside the region and graph results
are retained boundary outputs, so one fused region may correctly produce
several ordered values.

Producer duplication, region boundaries, and materialization belong to this
physical exploration phase rather than logical rewrite identity. A hypergraph
may index overlapping candidates internally, but membership alone is not a
complete region identity.

### Physical implementation

Implementation rules produce schedules such as:

- scalar or vectorized flat loops;
- rank-aware strided loops;
- direct or tiled rearrangement;
- serial, subgroup, threadgroup, or multi-pass reduction;
- direct or GEMM-backed contraction.

The bounded frontier admits checked `ScheduledKernel` and `KernelSubprogram`
proposals and rejects the reserved `View` variant explicitly, while `OpaqueCall`
is admitted through its own compiler-owned declaration and registration path.
[Fusion and scheduling](fusion-and-scheduling.md) owns that admission model; this
contract states only that the provider/body representation retains an additive
sum-type seam, so a further body variant lands without weakening scheduled-kernel
verification. [`enumerate-the-split-reduction-on-the-planning-frontier`](../../tickets/enumerate-the-split-reduction-on-the-planning-frontier.md)
promoted `KernelSubprogram` out of the reserved set and
[`integrate-opaque-calls-into-the-physical-frontier`](../../tickets/integrate-opaque-calls-into-the-physical-frontier.md)
did the same for `OpaqueCall`. Opaque-call declaration and registration are
compiler-owned and crate-private, so admission is not an out-of-crate seam: no
caller supplies its own opaque provider today.

Each implementation candidate advertises a machine-checkable numerical
guarantee, realization/provider identity, and scoped evidence. It is admitted
only when that guarantee refines every effective operation contract. A stronger
implementation may satisfy a weaker requested result set, but it does not
rewrite semantic identity.

### Enforcers

An enforcer supplies a missing required property at a cost:

- contiguous materialization;
- layout conversion;
- encoding repacking.

An enforcer may change only how a boundary value is stored, addressed, placed, or delivered, never which values that boundary carries. ADR 0001's separation of semantic planning from physical scheduling holds only because several physical schedules implement one semantic group identically, so a schedule-level step that altered a value would make one semantic program mean different things under different plans. Every entry above is value-preserving in that sense, and so is every property the [boundary-property list](#boundary-requirements-and-guarantees) admits.

A dtype cast is therefore not an enforcer, and resolved value dtype is absent from that list by construction rather than by omission. [Numerical semantics](../numerical-semantics.md#casts) makes casts semantic operations carrying resolved typed conversion contracts, and ADR 0010 forbids a later phase substituting a different conversion or letting fusion erase one that an unfused program happened to realize through a store and reload. A conversion the graph already contains is realized by ordinary lowering of that operation and supplies no missing property; a conversion the graph does not contain may not be introduced by a schedule at all. Admitting dtype to the property list would also break that list's ordering relation, because satisfaction there is subsumption and the dtype analogue of "16-byte alignment satisfies a 4-byte requirement" is a producer keeping `f32` where the boundary calls for `f16` — precisely the erased narrowing ADR 0009 and ADR 0010 forbid.

Choosing wider computation or accumulator precision inside a region is a different mechanism under a different gate: the implementation rules above already require each candidate's machine-checkable numerical guarantee to refine every effective operation contract. That is numerical conformance checked on an implementation, not a missing property supplied at a boundary.

Scalar alignment-safe execution and bounds masking are schedule alternatives or
proof obligations, not enforcers. A partial buffer plus second pass is a
multi-kernel reduction implementation.

### Cleanup

After program selection, local passes perform index-expression CSE,
loop-invariant motion, strength reduction, constant folding, bounds-check
elimination, and dead-code elimination. Schedule-affecting normalization
finishes before `ScheduledRegion` identity is formed. Later structured-kernel
cleanup is independently canonicalized and committed through codegen/artifact
identity; it must not silently mutate the already-hashed schedule.

## Boundary requirements and guarantees

A downstream region implementation requests boundary properties and each
producer implementation advertises what it guarantees. Initial boundary
contracts include:

- storage layout class and contiguous axes;
- storage encoding;
- alignment and vectorizable width;
- materialized buffer, alias/view, or opaque runtime value;
- device and address space;
- **availability** — the dependency after which a produced value may be consumed;
- **visibility** — whether a consumer's reads see the produced value without a further coherence action.

The last two complete the list rather than extending it. `AGENTS.md` names ordering and synchronization as explicit physical contracts rather than implicit node annotations, and a boundary contract that could not express them would leave a plan's ordering and coherence obligations unstated at exactly the boundary they are owed across.

**What each dimension currently establishes, distinguished by maturity.** A dimension appearing here says the optimizer models that property, not that every value in its vocabulary is served. Reading the two new dimensions honestly:

| value | maturity |
| --- | --- |
| availability *after producing dispatch* | **implemented and satisfiable** — a producer guaranteeing availability after its own dispatch discharges it |
| availability *after observed host completion* | **type-system reservation** — ADR 0033 makes host observation a separate boundary (terminal completion, a post-completion status check, error-record visibility, then interpretation) and no guarantee in this vocabulary discharges it, so a requirement naming it rejects explicitly |
| visibility *readable on the requiring affinity* | **implemented and satisfiable** — discharged by a producer coherent on its own affinity |
| visibility *requires an explicit coherence action* | **reserved, and deliberately not satisfiable** — ADR 0047 makes an affinity-to-domain edge declare its own coherence requirements, so a domain owing a flush or invalidate is guaranteed by its producer and unreadable by a consumer until an enforcer supplies the action; treating it as satisfied at a higher cost is the substitution ADR 0043 forbids |

A reserved value is not a weaker form of support. It rejects, and the rejection is the guarantee: the alternative is a plan that silently reads a value no one made visible.

**Admission rule for a new dimension.** The list is extensible, and a dimension is admitted only when all of these are stated: its requirement space, its guarantee space, the satisfaction or subsumption rule between them, how a child boundary derives it, its dominance behaviour, its identity encoding, its maturity by the classes above, and the boundary at which a value-preserving enforcer may discharge it rather than the plan being refused. A dimension without a satisfaction rule is a label, and one without an identity encoding is invisible to every consumer that compares two plans.

**Storage encoding is a distinct property, not part of layout class.** Layout
class answers which logical coordinate maps to which position; encoding answers
how one element is represented at that position. They vary independently: a
blocked layout of bit-packed `u4` and a blocked layout of unpacked `u4` share a
layout class and differ in encoding, so no layout class can express the
difference. Encoding meets the admission test that keeps dtype off this list —
a producer can realize one semantic value either way and the choice is
unobservable in the value — where a narrowing dtype change is observable in it.

Its enforcer is repacking. ADR 0047 already names "materialization/repacking" as
an enforcer family, and the
[transfer taxonomy](../research/transfers/transfer-synchronization-and-resource-lifetime.md)
already separates `MaterializeLayout` ("same logical value and dtype;
addressing/layout may change") from `RepackEncoding` ("explicitly changes
storage encoding"), keeping both distinct from `ConvertDtype`. Its
`TransferStage` also carries an explicit `PreserveStorageEncoding` semantics
field, which a transfer would have no reason to declare unless encoding were a
dimension it could otherwise change. So the enforcer was accepted before the
property it supplies was named here, and this entry closes that gap rather than
adding a new mechanism.

Encoding owes the same treatment every other property owes: canonical keys,
satisfaction and subsumption, child-requirement derivation, and dominance.
Subsumption is not automatic in either direction — an unpacked producer does not
satisfy a packed requirement merely by being cheaper to read, and a packed one
does not satisfy an unpacked requirement merely by being denser — so an encoding
relation is stated per encoding family rather than assumed to be an ordering.

**A quantized value's companion parameters are not a separate property.** Its
component roles are semantic: the [IR contract](../ir.md) makes a quantized
tensor "one first-class semantic tensor value even when its runtime
representation has several components", with the versioned scheme, component
roles, and coordinate maps named in its static type contract, and with scale and
zero-point tensors entering as ordered operands to an explicit assembly or
conversion operation. A schedule may not add, drop, or re-role a component,
because that would change which values the boundary carries. What remains
physical is that "physical packing and addressing remain storage decisions" and
that "artifact lowering may expand one logical quantized argument or result into
several verified physical bindings". Those are encoding and layout applied to
each component, so what this list owes a multi-component value is that its
properties are stated per component — not a further property naming the
companions themselves.

Logical shape, resolved value dtype, accumulation semantics, and numerical
policy are semantic traits or optimization-context constraints, not properties
supplied by a schedule.
Target capabilities, runtime guards, resource use, schedule invariants, and
cost estimates are also distinct concepts rather than entries in one universal
property bag. Iteration order and register residency are region-internal unless
they affect a boundary value.

For example, a vectorized reduction may require a unit-stride reduction axis,
16-byte alignment, and an extent divisible by four. The optimizer compares a
contiguous-materialization enforcer followed by that reduction against a
generic strided reduction.

The boundary-contract system defines canonical keys, satisfaction and
subsumption (for
example, 16-byte alignment satisfies a 4-byte requirement), child requirement
derivation, and dominance. Enforcer insertion is cycle-checked. Interesting
boundary properties such as useful unit-stride axes are retained on a bounded
Pareto frontier even when they are not locally cheapest.

One implementation dominates another only within the same semantic and
constraint region when its applicability covers the other's, its target and
boundary requirements are no stronger, its guarantees are at least as strong,
its hard resources are no worse where relevant, and its symbolic cost is no
worse throughout the compared constraint cell and strictly better somewhere.
Otherwise both remain or the constraint space is partitioned. Cost alone may
not prune the only implementation valid for a runtime region.

Target-requirement implication and evaluation phase participate in dominance.
A candidate needing a stronger or later runtime predicate does not dominate a
generic candidate merely because its estimated cost is lower. Scalar/generic
coverage is retained whenever specialized feasibility is deferred or narrower.

Numerical conformance is checked before this dominance relation. Accuracy is a
hard semantic dimension, not a Pareto cost; incomparable or unknown evidence
cannot be made legal by a lower estimated runtime.

## Possible memo contract

[Bounded hierarchical search](#bounded-hierarchical-search) places the memoized level at physical enumeration, and this is the conceptual key a bounded memo over that level would carry. Two things about it follow from the selected formalism rather than from the sketch, and reading them off the key alone would get both backwards. The `semantic group key` *identifies* which candidate an entry belongs to; it is not a group of alternatives the memo searches over, because a memo keyed on a semantic group is the Cascades-as-the-whole-search shape that section eliminates. And `numerical policy` and `target profile` appear in the optimization key to keep entries from different contract groups and different targets from colliding, not to let the memo compare across them — contract preference is an ordering that runs to completion before physical planning, and a key component is exactly what would turn it into a property a cheaper plan could win on. With those readings fixed, the key is:

```text
semantic group key = canonical semantic expression
optimization key = (group, boundary requirements, target profile,
                    numerical policy, constraint region)
candidate = region implementation + child boundary requirements
            + boundary guarantees
```

It would store a bounded Pareto set, track shared DAG cost without charging a
materialized producer once per parent, detect cycles, and retain structured
rule/candidate provenance. Search-budget exhaustion returns the best complete
plan found under deterministic fallback heuristics.

Region enumeration is already general rather than a trivial builder for a narrow
semantic graph: `EnumerateRegionCandidates` proposes every connected convex
region of an arbitrary verified DAG up to the declared budgets, with separate
content and occurrence identities and typed budget-stops, and is checked against
an exhaustive subset oracle. The staging this paragraph once pointed forward to has landed:
[cover enumeration](../../tickets/prototype-region-cover-enumeration.md),
[physical-implementation frontiers](../../tickets/prototype-physical-implementation-frontier.md),
and [complete physical-plan selection](../../tickets/prototype-complete-physical-plan-selection.md)
are all done, and the general DAG cover search over them (fan-out, ordered
multi-result outputs, checked shared-work duplication, per-edge materialization,
budgeted memoized enumeration against an exhaustive oracle) landed 2026-08-04
under `implement-general-dag-partitioning` — one optimizer architecture
throughout, exactly as this section required.

## Symbolic parameters and routing

The optimizer consumes a constraint environment describing exact/ranged
extents, divisibility, equalities, and optionally common profiled values. Costs
may be symbolic or piecewise over this environment. The selected result can be
a portfolio of AOT variants plus a deterministic routing decision tree or
crossover formula. Guards establish validity; routing chooses profitability
when several variants are valid. Routing policy participates in `EXPLAIN` and
artifact identity.

## Rule interface

Semantic rules conceptually provide:

```text
match(expression) -> bindings
check(bindings, semantic_context) -> proof or rejection
apply(bindings) -> equivalent expression(s)
```

Implementation rules conceptually provide:

```text
implement(group, boundary_requirements) -> candidate {
    implementation,
    child_requirements,
    boundary_guarantees,
    legality_constraints,
    estimated_resources
}
```

Every rule needs a stable name, declared numerical preconditions, positive and
negative tests, deterministic search behavior, and explain-trace output.

## Explainability

An `EXPLAIN` report should show:

```text
logical input
normalization rules fired
equivalent alternatives retained
resolved lowering capability per occurrence
index-region refinement evidence or its typed residual refusal
fusion regions considered
boundary requirements/guarantees
enforcers inserted
schedules considered and rejected
per-operation reference and effective accuracy envelope
candidate numerical guarantee, realization, and evidence class
selected cost and assumptions
runtime guards and fallback
```

Structured rejection reasons are important: “threadgroup reduction rejected:
shared memory exceeds target limit” is actionable; a later MSL compiler error
is not. Numerical reasons are equally concrete, such as “claimed 3 ULP exceeds
required 1 ULP,” “domain uncovered,” or “toolchain evidence unknown,” and are
reported separately from cost rejection.

Every rejection records its stage, stable reason code, rule/provider identity,
affected operation/value or candidate, failed predicate/evidence, and whether
the result is a hard rejection, safe deferral, budget stop, dominance pruning,
or cost disadvantage. Explain output never collapses these into “not fused.”

A budget stop is the one disposition that says nothing about its subject, so it never stands alone. Whatever predicate the stopped analysis was deciding is recorded beside it with an `Unknown` evidence class and the reason its proof stopped. Emitting the stop without that assessment would leave a reader to infer either a pass or a rejection from a record that supports neither, and inferring a pass is the more dangerous of the two.

### Explain authority

Under ADR 0073 the typed explain vocabulary — records, subjects, stages,
dispositions, reason and rule keys, evidence classes, and retention bounds — is a
module of `tiler-compiler`, not a separate `tiler-explain` crate. The compiler
owns record construction, canonical identity, causal integrity, and the versioned
renderer. Emission is compiler-owned: sibling compiler modules obtain record
handles from a writer, and no provider-facing emission trait is published. Module
visibility is a public-facade question rather than a packaging one, and the two
questions have since separated: the compiler boundary is public — `session` is a
`pub mod` and [`prototype-public-compiler-api`](../../tickets/prototype-public-compiler-api.md)
made it so — while `explain` stays a private module. What crosses that boundary is
the read-only result, not the vocabulary: `session` re-exports
`VerifiedCompilationExplain` and hands out an opaque `ExplainReport` whose only
capability is rendering, so no record, subject, stage, disposition, or reason-key
type is nameable outside the crate.

If a second crate must ever read canonical traces, the record, subject, and
disposition vocabulary moves into `tiler-ir` following the `AbiExpr` co-location
precedent of ADRs 0068 and 0070, with emission staying compiler-owned. A new
crate is not the expansion path. Until that trigger fires, a component that
cannot depend on `tiler-compiler` has no explain contract; it is an explicit
unsupported case rather than a licence to copy the vocabulary.

Canonical trace content is data and the renderer is presentation. Nothing in this
contract requires an explain trace to be serialized into an artifact envelope,
and the artifact contract does not carry one.

**Fact — the implemented renderer is `tiler-explain-v9` over trace schema v11, and a compilation's composite renders as `tiler-compilation-explain-v1`.** `crates/tiler-compiler/src/explain.rs` is the single authority for all three numbers and `explain_vocabulary_is_append_only_and_versioned` pins them. **One rule governs both numbers, corrected here on 2026-08-06 — this sentence read "the two move independently, because a record added without changing the rendering advances only the schema", which says that adding a record steps the schema and is what made the appended event tag 13 read as an omission.** A version steps when something a reader already had changes, and does not step when something new merely becomes expressible. The schema version is an encoding domain, so it steps when a change moves or reinterprets a previously encodable record's bytes, and not when a fresh event, subject, disposition, or quantity tag is appended under the existing per-tag framing — the same injectivity rule the schedule, kernel, program, and artifact identity domains state at their own tag sites, and the one `canonical_explain_subject_bytes` states in the negative when it steps to `v5` because the per-tag argument does not close there. The renderer version steps when an existing record's spelling changes, and not when a record type the earlier vocabulary could not produce receives its first spelling. The two therefore still move independently, for the corrected reason: schema v6 advanced alone because a record's encoding changed while its rendering did not, and event tag 13 with its `synchronization:` line advanced neither, because it moved nothing that already existed. **A version does not promise the vocabulary.** Two traces sealed under one schema version by different builds may differ in which tags they can contain, and a reader must never derive a tag set from a version; the ledger comment at the version block is the vocabulary's only authority, and it marks which historical steps the rule forced and which it did not. Nothing depends on the refused promise: a trace is never serialized, embedded, or cached, no decoder exists in the workspace or outside it, and the only comparison of the number is a same-build staleness check. The v3 renderer named below is the historical step that added semantic-selection and numerical-contract-preference events; the versions have advanced since, most recently when [`reconcile-input-ordinal-region-local-and-declared-input-semantics`](../../tickets/reconcile-input-ordinal-region-local-and-declared-input-semantics.md) replaced opaque-call `input#N` bindings with exact `access#N` coordinates. The composite header stayed `tiler-compilation-explain-v1` because it versions only the wrapping spelling and each nested trace announces its own renderer version. The top-level semantic-selection ledger embeds each candidate's exact full canonical compilation subject, and `VerifiedCompilationExplain` accepts the composite only when every keyed nested trace carries that same subject. A key-preserving splice of one candidate trace into another is therefore rejected rather than authenticated by a short label or digest. Each nested trace records only the normalization or algebraic payload adopted by that candidate; declined rules and portfolio budget stops remain in the top-level exploration trace.

### What the public compiler boundary exposes of a trace

*Added 2026-07-25 by `prototype-public-compiler-api`, which settled the seven public-surface questions the typed-explain work deferred. Each statement below is derived from a contract or a measured property rather than chosen, and each names what would reopen it.*

**A trace is complete or absent, never partial, and a failed compilation returns the one it has.** A detail record that would exceed the retained-trace ceiling fails the compilation closed with a typed capacity error rather than being dropped, so a sealed trace is complete by construction and no truncated form exists to describe. A refusal that happens *before* a verified per-target request exists — request verification, semantic output typing, numerical-contract resolution, normalization, target selection — has no trace to seal, and reports that absence as a distinct state. Discarding a sealed trace on the failure path is not an option this contract leaves open: a rejection reported with no stage, reason code, rule, or predicate is the collapse the paragraph above forbids.

**Rendering is deterministic and total; its spelling is not a contract.** One trace renders to one string, and every retained record appears — the renderer has no filter and no bound. The rendered text is a diagnostic for a human reader and is not a parse target, and its leading `tiler-explain-v<N>` names the renderer version so a change to the rendering is visible. Committing to the text would create a second description of a trace that has to be kept in agreement with its canonical bytes, which is the duplicate-derivation hazard the data/presentation split exists to prevent.

**The renderer header's request qualifier is a correlation label, not an identity.** It is a short non-cryptographic fold of the canonical request subject, so two distinct requests may share one. ADR 0074 convention 2 governs it as a presentation label: it is never an equality, dedup, or cache-key input. Redacting it protects nothing — it is derived from the caller's own request — and removing it would leave two rendered traces in one log indistinguishable.

**Nothing in a trace is redacted.** Every provider key and revision a trace attributes is either minted by Tiler or installed by the caller's own request, because the writer refuses a rule attributed to any other provider. There is no third party's detail present to withhold, and withholding one would make a rejection unexplainable, which this contract forbids. Reconsider when a registry the caller does not control can install rules.

**There is no retention control to expose.** The configurable detail budget is gone; exceeding the ceiling is a typed compile failure. Re-introducing a control would re-introduce a trace that is silently incomplete.

**Only the compiler mints an evidence receipt, and only from a proof it derived.** A receipt carries the `SoundProof` evidence class, and this repository keeps `SoundProof`, exhaustive finite evidence, empirical evidence, normative guarantees, and `Unknown` as distinct classes. A receipt supplied by an external provider is a *claim*; recording it as `SoundProof` would convert an assertion into a proof at the boundary, and a fusion legality proof is what admits a rewrite. A provider's contribution is its identity and revision, which the compiler attributes and bounds against the request's installed registry — that is provenance, not evidence. This does not change if a provider can one day ship a machine-checkable proof: the compiler would still mint the receipt, from its own re-check.

**Every identity the boundary emits is canonical bytes, never a digest, and never both.** ADR 0074 convention 2 states the rule; a digest here would be a second identity over the same subject, requiring a stated hash and a collision argument, and the production digest implementation is not yet chosen. Two published values a consumer can disagree about is strictly worse than one.

**Public enums follow ADR 0074 convention 5's clause test, and never a parallel versioned schema view.** Such a view is a second, hand-maintained description of an enum that nothing keeps in agreement, which is convention 3's argument against encoding a projection instead of its source; and it buys compatibility, which ADR 0075 records as a rejected premise while no crate is publishable.
