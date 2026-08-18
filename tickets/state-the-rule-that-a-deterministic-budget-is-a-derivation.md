---
id: state-the-rule-that-a-deterministic-budget-is-a-derivation
title: Decide and state the deterministic-budget provenance rule
status: in-progress
priority: p3
dependencies: []
related: [derive-the-region-shape-budgets-from-the-declaration, design-explicit-caller-selected-budget-exhaustion-policies]
scopes: [implementation/compiler, contracts/optimizer, research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [budgets, conventions, decision, needs-tom]
claimed_from: todo
assignee: worker-budget-rule
lease_expires_at: 1787066597
---
## Decision accepted — 2026-08-11

Tom accepted the strict truncation contract for `physical_plan_combinations`: the value bounds attempted Cartesian implementation combinations for one cover, and exhaustion may leave the selected portfolio empty. When no harder target refusal exists and no valid complete plan was retained, compilation returns the typed `BudgetExhausted` class naming `physical-plan-combinations`, the declared limit, and the first refused demand. It does not claim the program is infeasible and does not switch backend, target, strategy family, or budget policy.

**Fact — a non-empty truncated portfolio already selects among the best valid plans seen.** `enumerate_cover_plans` retains every valid plan assembled before the stop. The later selection stage continues over that retained population while explain records the budget stop. This is not a fallback: it is the ordinary explicitly bounded search result. The terminal `BudgetExhausted` case is precisely the case where the bounded search retained no valid plan to select.

**Fact — no known physical baseline precedes the join.** Region and cover extremes are verified semantic coverage objects. Selection instead receives locally feasible per-region implementation frontiers, and `assemble_plan` is the first authority that proves their cross-region boundary compatibility. A canonical first combination can disagree while a later combination composes. Searching until the first valid combination would expose the full caller-amplifiable frontier product and defeat the work cap; reserving one arbitrary attempt outside a zero cap would neither guarantee a plan nor honour the stated limit.

**Inference — explicit future policies remain possible without weakening this narrow contract.** A tuned setup may later offer an explicit caller-selected policy such as accepting the best valid plan seen when search truncates or requiring exhaustive search. That future surface must be one required complete typed policy selected before compilation/preflight, carry its exact limits and exhaustion semantics in the request subject, and have no `Default`, ambient platform inference, automatic retry, or cross-backend fallback. [`design-explicit-caller-selected-budget-exhaustion-policies`](design-explicit-caller-selected-budget-exhaustion-policies.md) owns that triggered design rather than smuggling it into this fixed governed policy.

No budget value, request byte, identity domain, public API, or runtime behaviour changes under the decision itself. This ticket remains in progress for the source, contract, and regression work below.

## User-visible outcome

The compiler and its two governing documents tell one exact story for every deterministic budget: what it is derived from or what work-policy it caps, what exhaustion can remove, and whether the observed demand is exact or only a lower bound. No budget value or request identity moves under this ticket.

## Per-Fact audit — 2026-08-09

The original ticket was stale enough to change the work and the state of the node.

- **False — “five of fourteen are derived.”** `DeterministicBudgets::governed` documents **eight** authoring-side derivations: five program-scoped bounds (`semantic_values`, `semantic_operations`, `regions`, `host_expression_nodes`, `buffers`) and three region-shape bounds (`region_members`, `region_boundary_outputs`, `region_live_values`). The derivations are recorded when the governed profile is authored; `governed()` is a nullary `const fn`, so they are not runtime formulas over each request.
- **False — four search constants remain.** Six literal work-policy caps remain: `normalization_rewrites = 8`, `region_candidates_per_seed = 32`, `region_expansions = 10_000`, `region_covers = 1_024`, `region_cover_expansions = 100_000`, and `physical_plan_combinations = 4_096`. The original list omitted `region_candidates_per_seed` and undercounted the population.
- **False — every search-cap exhaustion merely loses an alternative while coverage survives.** Region formation emits the singleton and whole-program extremes before growth, cover enumeration retains the fully materialized and fused extremes, and normalization retains the verified input. `enumerate_cover_plans`, however, checks `produced >= max_combinations` before assembling the next plan; at a limit of zero it records `PlanBudgetStop` and can return an empty `SelectedPortfolio`. `SelectedPortfolio::is_empty` explicitly permits that state. No baseline complete plan is retained by this layer.
- **Verified — exact numeric provenance is absent.** Current source explains what the six literals bound and, for most, what their stop preserves. No accepted decision, retained measurement, or source derivation establishes why the values are exactly 8, 32, 10,000, 1,024, 100,000, and 4,096. They can be described honestly as uncalibrated deterministic policy literals; they cannot be relabelled as derived or measured.
- **False governing prose exists in both directions.** `docs/compiler/optimizer.md`, anchor `Most of the budgets above bound a *search*`, includes `physical_plan_combinations` in a claim that complete coverage survives. `docs/research/region-search/rewrite-search-formalism.md`, anchor `Deterministic budgets.`, calls five `region_*` fields search budgets even though three are shape bounds that can refuse a program.
- **Verified — identity is out of scope.** `canonical_explain_subject_bytes` serializes every field. No value, field, width, order, or request-subject domain may move here.

This audit supersedes the duplicate [`state-the-search-constant-provenance-the-caps-audit-found-bare`](state-the-search-constant-provenance-the-caps-audit-found-bare.md), which is closed as a duplicate carrier rather than left as competing work.

## Per-Fact re-audit at base `c82888d5698af0fad46004f4753969621d108929` — 2026-08-18

The 2026-08-09 audit above was re-verified against source read in full at this base, not against the tree it was written on. Every one of its six bullets survives; two are imprecise, one by omission of a same-class false sentence in the same document, and one new source defect was found at the exact editing site. Counts were recounted from the type rather than carried forward.

- **Verified — eight authoring-side derivations.** `pub(crate) struct DeterministicBudgets` declares fourteen fields, recounted mechanically: `awk '/^pub\(crate\) struct DeterministicBudgets \{/,/^\}/' crates/tiler-compiler/src/request.rs | grep -c 'pub(crate) '` reports `15`, which is the struct line plus fourteen fields, and the `governed()` body carries fourteen `field: value` lines. `governed`'s own prose derives exactly eight: the five program-scoped bounds `semantic_values` `80`, `semantic_operations` `62`, `regions` `12`, `host_expression_nodes` `51`, `buffers` `30` from the decoder layer's C1 decode row, and the three region-shape bounds `region_members` `62`, `region_boundary_outputs` `3`, `region_live_values` `80` from those five. `pub(crate) const fn governed() -> Self` is nullary, so the derivations are authoring-side and nothing is recomputed per request.
- **Verified — six literal work-policy caps, at the stated values.** Read from the `governed()` body: `normalization_rewrites: 8,`, `region_candidates_per_seed: 32,`, `region_expansions: 10_000,`, `region_covers: 1_024,`, `region_cover_expansions: 100_000,`, `physical_plan_combinations: 4_096,`. Eight plus six is the fourteen counted above, so the classification is total over the type.
- **Verified — `physical_plan_combinations` exhaustion can remove every complete plan, unlike the other five caps.** In `crates/tiler-compiler/src/region.rs`, `formation.retain_singleton_coverage()?;` and `formation.retain_whole_program_coverage()?;` both run before growth. In `crates/tiler-compiler/src/cover.rs`, `The fully-materialized (all-singleton) cover is retained unconditionally.` and `The fused (whole-program) cover is retained whenever it exists.` In `crates/tiler-compiler/src/normalize.rs`, `run_rewrite_engine` returns `EngineRun::BudgetStopped` and abandons the whole run, so the verified input survives and only alternatives are lost. `enumerate_cover_plans` instead tests `produced >= max_combinations` at the top of the loop, before assembling the next plan, records a `PlanBudgetStop` with `actual: max_combinations.saturating_add(1)`, and breaks; at `max_combinations = 0` the first iteration takes that branch, so no plan is ever assembled and `SelectedPortfolio::is_empty` is the documented result.
- **Verified — no derivation, decision, or measurement establishes the six exact numbers, with one refinement.** `docs/research/region-search/exhaustive-region-oracle.md` does carry `region_candidates_per_seed` `32` and `region_expansions` `10,000`, but as entries in a list it closes with `These numbers are provisional safety budgets, not performance conclusions` — a proposal this record made, not a derivation or a measurement, which strengthens the Fact rather than contradicting it. `docs/research/program-planning/physical-frontier-budget-calibration.md` measures plan-combination *counts* for census populations but states, at the anchor `Neither budget is yet a`, that the two budgets it calibrates are not `DeterministicBudgets` fields at all; it sets no value here. So all six are honestly describable as uncalibrated deterministic policy literals and none may be relabelled as derived or measured.
- **Imprecise — the false governing prose is real at both named anchors, and a third sentence of the same class sits in the same document unnamed.** `grep -c` at this base: `Most of the budgets above bound a` returns `1` in `docs/compiler/optimizer.md`; `Deterministic budgets.` returns `1` in `docs/research/region-search/rewrite-search-formalism.md`. In the optimizer document the false clause is `The genuine search bounds` … listing `physical_plan_combinations` among bounds that ``cost only alternatives``. But the paragraph two above it says of `region_covers`, `region_cover_expansions`, and `physical_plan_combinations` that hitting any of them stops only that growth path and `never removes either` coverage extreme, which is the same false claim about the same field. That sentence is hard-wrapped in the source, so the rendered-view anchor `never removes either coverage extreme` returns `0` while `never removes either` returns `1` — exactly the failure mode `AGENTS.md` warns reads as absence. Both sentences are corrected.
- **Verified — identity is out of scope and cannot move.** `canonical_explain_subject_bytes` writes the twelve `u32` budget fields in one `for budget in [...]` array and the two `u64` fields (`region_cover_expansions`, `physical_plan_combinations`) in a second, so all fourteen enter `tiler.compiler.request-subject.v6`. No value, field, width, or order moves under this ticket.

**New Fact — a doc-comment paste defect sits inside `DeterministicBudgets::governed`, at the exact site this ticket must edit.** Commit `9ebd6c65` ("Report exhausted budgets with a three-way provenance") replaced the sentence `The four in the per-output derivation is the measured stage count of the` with a copy of the `limit` / `reported` `u64` paragraph that belongs to — and still lives intact at — the `BudgetExceeded` variant. The result is an orphaned fragment beginning `widest chain, taken from` that no longer attaches to any sentence, and a duplicated paragraph explaining public refusal field widths in the middle of a derivation record. `git log -L 1231,1241:crates/tiler-compiler/src/request.rs` shows the replacement. The sentence is restored to the `regions` derivation it documents and the misplaced duplicate is removed; no claim is added or withdrawn by that repair.

## Decision boundary — answered 2026-08-11

Tom decided the contract for `physical_plan_combinations` before the prose is made normative:

1. **Permit an empty portfolio on exhaustion.** Keep the implementation and describe this row as a truncating cap that may remove every complete plan; correct both governing documents so they stop promising retained coverage at this layer.
2. **Require a retained baseline complete plan.** Change selection so a known complete plan is assembled outside the bounded combination search, analogous to region and cover extremes, then state the stronger alternative-loss guarantee.

**Accepted: permit an empty portfolio and preserve the typed stop.** The original retained-baseline recommendation was withdrawn after complete source review. Unlike region and cover enumeration, global selection has no privileged valid plan before cross-region compatibility is assembled. Manufacturing one would either privilege an arbitrary identity-ordered combination that can still fail or search outside the declared cap until a compatible combination appears. The current answer is both stricter and cheaper: honour the work limit exactly, retain and select among valid plans already seen, and return `BudgetExhausted` when that population is empty.

The six exact numeric values are a separate evidence gap, not a hidden third decision. This ticket may record them as uncalibrated policy literals without changing them. Choosing or changing a value requires its own measurement/decision ticket and complete identity accounting.

## Required work after the decision

- State the general rule at `DeterministicBudgets`: an authoring-side derivation names its formula and owning declaration; a literal policy cap names its work unit, stop effect, and evidence status. Do not imply runtime tracking.
- Classify all fourteen fields as the eight derived bounds and six literal policy caps above.
- Correct `docs/compiler/optimizer.md` and `docs/research/region-search/rewrite-search-formalism.md` to distinguish program bounds, region-shape bounds, alternative-preserving search caps, and the decided physical-plan behavior.
- Add a focused `physical_plan_combinations = 0` regression over a known completable cover. It must demonstrate `PlanBudgetResource::Combinations`, `limit = 0`, `actual = 1`, and an empty portfolio. Perturb only the cap to one and show that the known compatible first combination is then retained.
- Add a first-incompatible/later-compatible combination regression so canonical identity order cannot silently be re-described as a valid baseline.
- Preserve every value and every encoded request byte.

## Delivery record — 2026-08-18, from base `c82888d5698af0fad46004f4753969621d108929`

Each required-work bullet, with where it landed and the evidence.

1. **General rule stated at `DeterministicBudgets`.** A new struct-level doc in `crates/tiler-compiler/src/request.rs` opens `The rule: a budget is one of exactly two things, and it must say which`. An authoring-side derivation names its formula and the declaration it is over, and the doc says explicitly that `governed` is a nullary `const fn` returning integer literals and that no field is tracked or recomputed while a request compiles, so nothing implies runtime tracking. A literal policy cap names its work unit, its stop effect, and its evidence status, and the doc states that a literal `must never be relabelled as derived or as measured`.
2. **All fourteen fields classified, twice over.** The struct doc carries the provenance split (eight derived, six literal) and, separately, a four-class stop-effect split — program bounds, region-shape bounds, alternative-preserving search caps, and the one truncating cap — with the explicit warning that a reader must not infer one from the other. Every previously undocumented field (`semantic_values`, `semantic_operations`, `regions`, `host_expression_nodes`, `buffers`) gained a field doc, and each of the six literals now carries an `Uncalibrated policy literal.` paragraph naming its work unit.
3. **Both governing documents corrected.** `docs/compiler/optimizer.md` had **two** false sentences of the same class, not the one the ticket named: the `Most of the budgets above bound a` paragraph counted three exceptions where there are four and listed `physical_plan_combinations` among bounds that cost only alternatives, and the downstream-budget paragraph two above it claimed all three of them `never removes either` coverage extreme. Both are repaired in place and withdrawn in a dated `Correction — 2026-08-18` note, which also fixes a second imprecision found while reading: the `SearchLowerBound` row covers five `BudgetResource` rows, and `normalization_rewrites` has no `BudgetResource` variant at all — its stop is an explain-record disposition. `docs/research/region-search/rewrite-search-formalism.md`'s `Deterministic budgets.` paragraph is rewritten to the four classes over all fourteen fields and carries its own dated correction. Both notes flag that they quote retired wording, so a grep hit on the withdrawn fragment is this note rather than a live claim.
4. **`physical_plan_combinations = 0` regression.** `selection::tests::a_zero_combination_cap_empties_a_completable_cover`, over the two-region `{pointwise, reduction}` cover the existing `a_complete_plan_joins_a_two_region_cover_with_a_boundary_handoff` proves completable. It asserts `PlanBudgetResource::Combinations`, `limit = 0`, `actual = 1`, the stop's cover identity, an empty portfolio, and an empty rejection list; then it moves **only** the cap to one and asserts the same combination is retained with no stop, and re-verifies the portfolio.
5. **First-incompatible/later-compatible regression.** `selection::tests::a_later_combination_composes_where_the_canonically_first_one_disagrees`. The producer region admits two implementations — an opaque call declared `Aliasing::MayAliasInputs`, which the materialized consumer refuses, and a scheduled region, which it accepts. The fixture's precondition is read from `ImplementationFrontier::admitted` rather than assumed, and fails loudly if canonical identity order ever stops placing the incompatible body first. At the governed cap both combinations are reached and exactly one composes; at a cap of one the single admitted attempt is spent on the combination that disagrees, so the portfolio is empty with `limit = 1`, `actual = 2`. Constructing this needed no new fixture machinery: `frontier_with_opaque` already takes an optional scheduled body.
6. **No value and no request byte moved.** `git diff` over `crates/tiler-compiler/src/request.rs` touches no `field: value` line in `governed()` and no line of `canonical_explain_subject_bytes`. The pinned request qualifier `tiler-explain-v9 request=17e0dd47e48b7c18` in `explain::tests::deterministic_trace_is_sealed_and_rendered_separately` is unmodified and green.

**Repaired en route.** The paste defect recorded in the re-audit above: the sentence `The four in the per-output derivation is the measured stage count of the` is restored to `governed`'s `regions` derivation and the misplaced duplicate of the `limit` / `reported` `u64` paragraph is removed. That paragraph still stands, unaltered, where it belongs at `BudgetExceeded`.

### Perturbations — subject changed, assertion untouched

Three perturbations of `enumerate_cover_plans`, each reverted, each run as `cargo nextest run -p tiler-compiler -E 'test(a_zero_combination_cap_empties_a_completable_cover) + test(a_later_combination_composes_where_the_canonically_first_one_disagrees)'`.

- **Stop condition off by one** (`if produced >= max_combinations` → `if produced > max_combinations`). Both reddened. `a zero combination cap must retain no complete plan` and `the single admitted attempt was the combination that does not compose`.
- **Reported demand narrowed** (`actual: max_combinations.saturating_add(1)` → `actual: max_combinations`). Only the demand assertions reddened, which is what separates them from the emptiness assertions: `assertion left == right failed / left: 0 / right: 1` and `assertion left == right failed / left: 1 / right: 2`.
- **Walk order reversed** (`entry.admitted[*index]` → `entry.admitted[entry.admitted.len() - 1 - *index]`). Only the ordering case reddened — `1 passed, 1 failed` — with `the single admitted attempt was the combination that does not compose`. This is the one that shows the canonical-order claim is load-bearing rather than incidental.

### Commands

- `cargo nextest run -p tiler-compiler` — `959 tests run: 959 passed, 1 skipped` (baseline at `c82888d5` was `957 passed`; the delta is exactly the two new cases).
- `cargo test -p tiler-compiler --doc` — `2 passed` plus `14 passed` compile-fail.
- `cargo clippy -p tiler-compiler --all-targets -- -D warnings` — clean.
- `cargo fmt --check` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler` — clean.
- `tkt lint` — `ok: no problems found`. `make citations` — every pinned citation and local link resolves. `git diff --check` — clean.

**Unverified — the rustdoc gate cannot reach the docs this ticket wrote.** `DeterministicBudgets` is `pub(crate)`, so `cargo doc --no-deps` never renders it and could not fail on a broken link in the new text; this is the `AGENTS.md` case about confirming a check reaches its subject. Running `cargo doc --no-deps --document-private-items` does reach it and reports the new links resolving, but it exits `101` at this base on sixteen pre-existing broken intra-doc links in files this ticket does not touch — `cover.rs`, `estimate.rs`, `frontier.rs`, `governed.rs`, `lowering.rs`, `pipeline.rs`, `pipeline/trace.rs`, `program.rs`, `region.rs`, `request.rs` lines far below this edit, and `target/feasibility.rs`. Two of those files are owned by other live lanes, so repairing them is not this ticket's; it wants its own narrow ticket.

## Explicit non-goals

No budget-value change, request-subject encoding change, identity-domain step, new caller-configurable budget set, or silent calibration claim. [`decide-whether-a-derived-budget-belongs-in-the-request-subject`](decide-whether-a-derived-budget-belongs-in-the-request-subject.md) remains the separate public identity decision.

## Closes when

Tom answers the physical-plan stop contract; source and both governing documents classify all fourteen fields truthfully; the physical-plan stop has a subject-perturbation regression; all six literals are labelled with their actual evidence status; and no value or identity moved.
