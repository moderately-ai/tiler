---
id: carry-the-exhausted-resource-through-the-budget-refusal
title: Implement the three-way exhausted-resource budget report surface
status: in-progress
priority: p3
dependencies: []
related: [measure-executable-coverage-identity-growth-against-the-program-identity-bound]
scopes: [implementation/compiler, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, diagnostics, public-boundary]
claimed_from: todo
assignee: worker-exhausted-budget
lease_expires_at: 1786666489
---
## User-visible outcome

A caller whose compilation stops on a deterministic budget learns which budget, what its limit was, the value the compiler compared, and whether that value is an exact demand, a conservative planning envelope, or a lower bound from truncated search — instead of the single word `BudgetExhausted` or a falsely uniform `actual`.

## Why this exists

**Measurement, 2026-08-05, from [`spikes/program-planning/identity-growth/`](../spikes/program-planning/identity-growth/README.md).** The spike compiles one program past the governed `semantic_operations` budget to prove its ladder covers the whole reachable domain. The refusal it could observe then was exactly:

```
CompileFailure { class: BudgetExhausted, explain: "absent (refused before a target-qualified trace)" }
```

That was everything a public caller got. The spike had to attribute the refusal to `semantic_operations` by reading the source, which is a source reading rather than something the refusal said. (The Debug string for a missing explain still prints that way; the class itself is no longer a unit variant — see Decision boundary and the 2026-08-10 correction.)

**Historical defect (pre-delivery, motivating this ticket) — the information existed and was discarded at the boundary.** At the 2026-08-08 audit base `acc26984`, `RequestError::BudgetExceeded` already carried `resource`, `limit`, and `actual`, and `CompileError::BudgetExhausted` wrapped that whole error, but `class_of` mapped it to the **unit** variant `CompileFailureClass::BudgetExhausted`, dropping all three fields — while the sibling arms for `InvalidRequest` and `UnsupportedCapability` both preserved a `rule` through `rule_of`, which already had a `BudgetExceeded` arm returning `resource` that the budget path never reached. **That unit-arm drop is no longer true of the tree:** `class_of` now destructures into `CompileFailureClass::BudgetExhausted { resource, limit, actual }`. Do not re-read the unit-variant sentence as current behavior.

**Historical inference (why the work was filed) — the asymmetry looked like an oversight rather than a decision.** `CompileFailureClass`'s documentation says the enum exists so "a caller branches on the boundary that refused instead of matching on text", and sibling variants already carried the refusing rule for that reason. A budget refusal is the class where the actionable detail is a number; before delivery it was the one that carried nothing. Nothing in `session.rs` argued for the omission.

**Fact — a pre-trace program-scoped refusal still has no explain report.** Request verification runs before a target-qualified trace can be opened, so `CompileFailure::explain` returns `None` on that route by construction. The typed fields are therefore the only public route for those refusals. (A second producer via empty portfolio after an analysis budget stop *does* seal a trace — see the 2026-08-08 table.)

## Fact audit, 2026-08-08, at base `acc26984`

Every claim above was re-read at this base before any edit. **Every one of the five line citations the original text carried was stale**, so all five have been removed in favour of the searchable anchors below; the claims they supported were true of that base. Two claims needed correcting outright, and both are corrected in place above and restated here so a reader of the earlier text knows what to un-learn.

**The anchors in this table resolve only at `acc26984` (or via `git show acc26984:<path>`), which is this ticket's pre-change base and the state the 2026-08-08 audit describes.** Several of them are the code this ticket then changed — in particular the unit-arm anchors `CompileError::BudgetExhausted(_) => CompileFailureClass::BudgetExhausted,` and `RequestError::BudgetExceeded { resource, .. } => resource` as a bare `resource` (not `resource.key()`) — and **those strings are absent from the post-change tree** (current `class_of` carries payload fields; current `rule_of` returns `resource.key()`). That absence is expected, not a defect in the table. Read historical anchors with `git show acc26984:<path> | grep -F '<anchor>'`. Each was run that way before this table was written; the two that did not resolve on the first attempt were a phrase wrapped across two source lines and a struct variant whose field is declared on its own line, and both are now anchored on text that does occur on one line at that base.

| Claim | Verdict | Evidence at this base |
| --- | --- | --- |
| The spike observes `CompileFailure { class: BudgetExhausted, explain: "absent (…)" }` | **Verified (historical shape)** | `session.rs` `"absent (refused before a target-qualified trace)"` inside the hand-written `impl fmt::Debug for CompileFailure` (explain-absence string still present; class Debug shape has since grown fields) |
| `request.rs:821` attributes the refusal to `semantic_operations` | **Citation false** | Line 821 is inside `DeterministicBudgets`' doc prose. The real site is `check_budget(` … `budgets.semantic_operations,` in `check_program_budgets`. The *key* is `"semantic-operations"` (hyphen); `semantic_operations` (underscore) is the budget field. The ticket used one spelling for both |
| `RequestError::BudgetExceeded` (`request.rs:2169`) carries `resource`, `limit`, `actual` | **Verified, citation false** | Line 2169 is inside `max_input_elements`. Anchor: `BudgetExceeded {` in `pub(crate) enum RequestError`. The field *types* were unstated and load-bearing: they were `&'static str` / `u32` / `usize` at that base |
| `CompileError::BudgetExhausted` wraps the whole error (`pipeline.rs:504`) | **Verified, citation false** | Line 504 is inside `impl Error for CompileError`. Anchor: `RequestError::BudgetExceeded { .. } => Self::BudgetExhausted(value),` |
| `class_of` maps it to the unit variant (`session.rs:1232`) | **Verified at `acc26984` only — historical** | Line 1232 is `pub struct AbiEntry`. Anchor at that base: `CompileError::BudgetExhausted(_) => CompileFailureClass::BudgetExhausted,`. **Gone from the post-change tree**; current arm is the payload form |
| `rule_of` has a `BudgetExceeded` arm (`session.rs:1255`) this path never reaches | **Verified at `acc26984` (arm still present, now `resource.key()`)** | Line 1255 is in an `ExplainReport` doc comment. Anchor at that base: `RequestError::BudgetExceeded { resource, .. } => resource,`. Post-change: `=> resource.key()`. Unreachability for the public budget path re-derived: `class_of` uses typed fields instead; the arm remains for totality of `rule_of` over `RequestError` |
| Two of the five classes carry a `rule` | **Verified** | Two variants of `CompileFailureClass` each declare a field under the comment `Stable diagnostic key of the refusing check.`, which occurs exactly twice in `session.rs` |
| A pre-trace refusal has no explain report | **Imprecise — corrected** | True of the route the spike measured, and **false as a statement about the class**. `BudgetExhausted` has a second producer, `truncating_budget`, which fails through `target_failure(…, ExplainStage::Selection, "portfolio-empty-after-budget-stop", …)` and *does* carry a sealed trace. "There is no fallback a caller could use instead" holds only for the program-scoped budgets |
| Implied: `resource` ranges over the five program budgets | **False — not stated, and the design turns on it** | **Thirteen** distinct keys can reach `RequestError::BudgetExceeded`, from **four** authorities: five string literals in `check_program_budgets`, plus `RegionBudgetResource` (5), `CoverBudgetResource` (2 of 3 — `Refusals` is filtered out), and `PlanBudgetResource` (1). Three of those four were **already typed enumerations** with their own `key()` tables, which the "`&'static str` or a typed enumeration" item does not mention |

**Fact — the demand is not one quantity.** `actual` is exact for the eight bounding resources and a *lower bound* for the five search resources. All three stop records say so, in three different wordings rather than one — `RegionBudgetStop` splits it per budget (`For a per-candidate budget this is the candidate's exact count.` against a growth budget that is `a lower`-`bound on the unexplored space rather than its size`, wrapped across two lines), `CoverBudgetStop` says `This is a lower bound on the unexplored space rather than its size: the`, and `PlanBudgetStop` says `This is a lower bound on the unexplored combinations rather than their` exact count. Publishing `actual` as a bare number would have made one field mean two things, which is why the delivered surface carries `BudgetResource::refusal()` beside it.

**Fact — only the five program-scoped resources are publicly reachable today.** `DeterministicBudgets` is `pub(crate)` and the public surface only ever passes `governed()`, so the truncation route needs a caller-stated budget set to reach; `session.rs`'s own reachability note records the same thing and the reason. The vocabulary is complete anyway, because the mapping into it must be total over what the compiler can raise.

**This repair does not change what the ticket is for.** The outcome, the non-goals, and the closing condition all survive; what changed is that the resource is a thirteen-row vocabulary over four authorities rather than a five-row one over one, and that the refusal has to say which kind of demand it reports.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** Present-tense "Why this exists" prose that claimed `class_of` maps budget refusals to a unit `CompileFailureClass::BudgetExhausted` and that "the information is discarded at the boundary" is **false of the current tree** and was only true as the pre-delivery defect at base `acc26984`. The draft surface is landed: `CompileFailureClass::BudgetExhausted { resource, limit, actual }`, public `BudgetResource` / `BudgetRefusal`, frontend `rendered_refusal` struct arm, and regression `a_chain_past_the_program_size_bound_names_the_budget_it_exhausted`. The 2026-08-08 table's unit-arm anchors are historical only (`git show acc26984:…`); they do not resolve in HEAD. Ticket remains `awaiting-decision` until Tom accepts or revises the exact public surface (Decision boundary). Optional out-of-scope drift: the identity-growth spike README/WALLS still describe the unit-variant gap.

## Fact audit — 2026-08-12, at base `9167caa9`

The acceptance audit re-read the complete ticket, ADR 0074, all thirteen resource mappings, all four refusal producers, `check_program_budgets`, the public session projection, the macro renderer, and the correctness-bearing tests. Three targeted tests passed: `only_the_search_bounds_report_a_truncated_demand`, `each_widened_budget_refuses_the_program_one_step_past_it`, and `a_chain_past_the_program_size_bound_names_the_budget_it_exhausted`.

**Verified:** carrying a typed `BudgetResource`, the declared limit, and the compared value is the correct public direction. The thirteen-row vocabulary is complete over the four current producers; only the five request-stage rows are publicly reachable under governed budgets today, but keeping the total internal-to-public mapping avoids a second lossy projection.

**False, decision-changing:** the 2026-08-08 statement that `actual` is exact for eight resources and a lower bound for five. It has **three** meanings. `SemanticValues`, `SemanticOperations`, `RegionMembers`, `RegionBoundaryOutputs`, and `RegionLiveValues` report exact completed counts. `Regions`, `HostExpressionNodes`, and `Buffers` report conservative pre-search planning envelopes. `RegionCandidatesPerSeed`, `RegionExpansions`, `RegionCovers`, `RegionCoverExpansions`, and `PhysicalPlanCombinations` report lower bounds from incomplete enumeration. The source states the distinction directly: `check_program_budgets` calls the regions value `The largest shape this profile may assemble`, calls host-expression nodes an `upper bound over every plan`, and calls buffers `The widest buffer count any plan for this request could reach`. It also supplies a counterexample: one input and one output produce the envelope value nine while the widest one-input chain declares seven, because `an upper bound over every reachable plan cannot also be each plan's exact count`.

**False:** `BudgetRefusal`'s claim that there is no third answer and its `Bounding` documentation that every member is a submitted program's own count or an exact candidate count. A completely evaluated envelope is not an exact plan demand. It is true that the current request gate performs no later search after refusing one; it is false that the reported number is therefore the program's exact resource requirement.

**False for the envelope rows:** the macro renderer's statement that every `Bounding` value is `a fact about the region's size`. The value is a fact about the governed admission policy's conservative envelope and may exceed what a particular reachable plan uses.

**Verified:** these meanings are mutually exclusive when classified by their provenance rather than by bare numeric inequalities. An exact value is mathematically both an upper and lower bound, so the durable categories are: completed exact demand; conservative planning envelope computed before selection; and lower bound recorded where enumeration stopped.

**Verified:** this public diagnostic correction changes no deterministic budget value, request-subject bytes, artifact schema, artifact/cache identity, or compiler/runtime algorithm. It changes the pre-alpha Rust output vocabulary and frontend wording only.

## Decision — accepted 2026-08-12 by Tom, relayed in the active Codex session

Retain the typed thirteen-row `BudgetResource` and payload-bearing `CompileFailureClass::BudgetExhausted`, but **revise the landed two-way surface before treating it as accepted**.

- Replace `BudgetRefusal::{Bounding, Truncated}` with a closed three-way report vocabulary whose meanings are `ExactDemand`, `PlanningUpperBound`, and `SearchLowerBound` (exact names may follow repository naming conventions, but those three semantics are fixed).
- Replace the public field name `actual` with the neutral `reported`; its documentation must direct the caller to the resource's report kind.
- Keep one total, wildcard-free `BudgetResource -> report kind` mapping as the authority. Do not store a second independently variable kind beside the resource.
- Keep the report-kind enum closed. `tiler-macros` maps every case totally to caller advice, so a new meaning must stop the build until that consumer defines its rendering; this is ADR 0074's fail-loud total-consumer boundary.
- Render exact demand, conservative envelope, and truncated-search lower bound separately. An envelope message must say that a particular plan may use less; a lower-bound message must not present the value as the budget required for success.
- Keep all thirteen resource variants and keep `BudgetResource` `#[non_exhaustive]`. Public reachability of only five rows today does not justify duplicating or narrowing the compiler's complete refusal vocabulary.

The categories are MECE over the current vocabulary because they are defined by how the value was produced, not by whether the number can be described abstractly as a bound. Future-proofing is fail-loud rather than a claim that a fourth provenance can never exist: a genuinely new meaning adds a deliberate report-kind variant and breaks every total owner until it is classified and rendered.

## Required work

- Preserve the already-landed typed resource and central stable-key authority.
- Implement the accepted three-way report-kind mapping and rename the public numeric field.
- Repair compiler, pipeline, session, optimizer, frontend, spike, and test prose that says eight values are exact or treats every non-search value as a region-size fact.
- Add an exhaustive thirteen-resource classification test sized from `BudgetResource` and perturb each of the three provenance classes.
- Add a regression proving at least one planning envelope differs from the exact demand of a reachable plan; use the source's one-input/one-output host-expression counterexample rather than changing an assertion alone.
- Keep the exact-demand and search-lower-bound public regressions, update the macro rendering tests for all three messages, and watch the new envelope classification fail before the fix.

## Explicit non-goals

Not changing any budget's value. Not opening budgets to caller configuration — `DeterministicBudgets` staying `pub(crate)` is a separate question with its own argument, and this ticket is about attribution of a refusal rather than control over it.

## Scope note

`implementation/frontend` was added on 2026-08-08 because the work requires it and cannot be finished without it, not to widen the ticket. Any payload on `CompileFailureClass::BudgetExhausted` breaks the unit-variant pattern `CompileFailureClass::BudgetExhausted =>` in `crates/tiler-macros/src/aot.rs`'s `rendered_refusal` with `E0533`, and that crate is `implementation/frontend`. No open ticket held a live claim on that scope when it was added (`tkt list --scope implementation/frontend --status in-progress` and `--status review` were both empty). The edit there is one match arm.

That arm needed more than a `{ .. }` to be correct. Its message said the compiler "stopped searching", which is true of a truncating stop and false of every budget a macro expansion can actually reach, so it now names the resource and splits its advice on `refusal()`.

## Closes when

The accepted three-way public failure surface is implemented; a caller can name the exhausted resource, its limit, the reported value, and that value's provenance without reading compiler source; all thirteen resources map exactly once; exact demand, planning envelope, and search lower bound each have a watched-failing regression and distinct frontend rendering; and the stale two-way claims are removed from current contracts and evidence.
