---
id: carry-the-exhausted-resource-through-the-budget-refusal
title: Accept or revise the exhausted-resource budget-refusal surface
status: awaiting-decision
priority: p3
dependencies: []
related: [measure-executable-coverage-identity-growth-against-the-program-identity-bound]
scopes: [implementation/compiler, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, diagnostics, public-boundary, needs-tom]
---
## User-visible outcome

A caller whose compilation stops on a deterministic budget learns which budget, what its limit was, and what the program asked for — instead of the single word `BudgetExhausted`.

## Why this exists

**Measurement, 2026-08-05, from [`spikes/program-planning/identity-growth/`](../spikes/program-planning/identity-growth/README.md).** The spike compiles one program past the governed `semantic_operations` budget to prove its ladder covers the whole reachable domain. The refusal it can observe is exactly:

```
CompileFailure { class: BudgetExhausted, explain: "absent (refused before a target-qualified trace)" }
```

That is everything a public caller gets. The spike had to attribute the refusal to `semantic_operations` by reading the source, which is a source reading rather than something the refusal said.

**Fact — the information exists and is discarded at the boundary.** `RequestError::BudgetExceeded` carries `resource`, `limit`, and `actual`. `CompileError::BudgetExhausted` wraps that whole error. `class_of` then maps it to the unit variant `CompileFailureClass::BudgetExhausted`, dropping all three fields — while the sibling arms for `InvalidRequest` and `UnsupportedCapability` both preserve a `rule` through `rule_of`, which already has a `BudgetExceeded` arm returning `resource` that this path never reaches.

**Inference — the asymmetry looks like an oversight rather than a decision.** `CompileFailureClass`'s own documentation says the enum exists so "a caller branches on the boundary that refused instead of matching on text", and two of its five variants carry the refusing rule for that reason. A budget refusal is the one class where the actionable detail is a number, and it is the one class that carries nothing. Nothing in `session.rs` argues for the omission, which is what separates this from the deliberate narrowings recorded elsewhere on that surface.

**Fact — a pre-trace refusal has no explain report either.** Request verification runs before a target-qualified trace can be opened, so `CompileFailure::explain` returns `None` here by construction. The typed fields are therefore the only route; there is no fallback a caller could use instead.

## Fact audit, 2026-08-08, at base `acc26984`

Every claim above was re-read at this base before any edit. **Every one of the five line citations the original text carried was stale**, so all five have been removed in favour of the searchable anchors below; the claims they supported were true. Two claims needed correcting outright, and both are corrected in place above and restated here so a reader of the earlier text knows what to un-learn.

**The anchors in this table resolve at `acc26984`, which is this ticket's base and the state the audit describes.** Several of them are the code this ticket then changed, so they will not be found in the tree afterwards — that is the point of them, not a defect. Read them with `git show acc26984:<path> | grep -F '<anchor>'`. Each was run that way before this table was written; the two that did not resolve on the first attempt were a phrase wrapped across two source lines and a struct variant whose field is declared on its own line, and both are now anchored on text that does occur on one line.

| Claim | Verdict | Evidence at this base |
| --- | --- | --- |
| The spike observes `CompileFailure { class: BudgetExhausted, explain: "absent (…)" }` | **Verified** | `session.rs` `"absent (refused before a target-qualified trace)"` inside the hand-written `impl fmt::Debug for CompileFailure` |
| `request.rs:821` attributes the refusal to `semantic_operations` | **Citation false** | Line 821 is inside `DeterministicBudgets`' doc prose. The real site is `check_budget(` … `budgets.semantic_operations,` in `check_program_budgets`. The *key* is `"semantic-operations"` (hyphen); `semantic_operations` (underscore) is the budget field. The ticket used one spelling for both |
| `RequestError::BudgetExceeded` (`request.rs:2169`) carries `resource`, `limit`, `actual` | **Verified, citation false** | Line 2169 is inside `max_input_elements`. Anchor: `BudgetExceeded {` in `pub(crate) enum RequestError`. The field *types* were unstated and load-bearing: they were `&'static str` / `u32` / `usize` |
| `CompileError::BudgetExhausted` wraps the whole error (`pipeline.rs:504`) | **Verified, citation false** | Line 504 is inside `impl Error for CompileError`. Anchor: `RequestError::BudgetExceeded { .. } => Self::BudgetExhausted(value),` |
| `class_of` maps it to the unit variant (`session.rs:1232`) | **Verified, citation false** | Line 1232 is `pub struct AbiEntry`. Anchor: `CompileError::BudgetExhausted(_) => CompileFailureClass::BudgetExhausted,` |
| `rule_of` has a `BudgetExceeded` arm (`session.rs:1255`) this path never reaches | **Verified, citation false** | Line 1255 is in an `ExplainReport` doc comment. Anchor: `RequestError::BudgetExceeded { resource, .. } => resource,`. Unreachability re-derived rather than taken on trust: `rule_of` has exactly two callers, the `InvalidRequest` and `UnsupportedCapability` arms of `class_of`, and no construction site of either wrapper is ever handed a `BudgetExceeded` |
| Two of the five classes carry a `rule` | **Verified** | Two variants of `CompileFailureClass` each declare a field under the comment `Stable diagnostic key of the refusing check.`, which occurs exactly twice in `session.rs` |
| A pre-trace refusal has no explain report | **Imprecise — corrected** | True of the route the spike measured, and **false as a statement about the class**. `BudgetExhausted` has a second producer, `truncating_budget`, which fails through `target_failure(…, ExplainStage::Selection, "portfolio-empty-after-budget-stop", …)` and *does* carry a sealed trace. "There is no fallback a caller could use instead" holds only for the program-scoped budgets |
| Implied: `resource` ranges over the five program budgets | **False — not stated, and the design turns on it** | **Thirteen** distinct keys can reach `RequestError::BudgetExceeded`, from **four** authorities: five string literals in `check_program_budgets`, plus `RegionBudgetResource` (5), `CoverBudgetResource` (2 of 3 — `Refusals` is filtered out), and `PlanBudgetResource` (1). Three of those four were **already typed enumerations** with their own `key()` tables, which the "`&'static str` or a typed enumeration" item does not mention |

**Fact — the demand is not one quantity.** `actual` is exact for the eight bounding resources and a *lower bound* for the five search resources. All three stop records say so, in three different wordings rather than one — `RegionBudgetStop` splits it per budget (`For a per-candidate budget this is the candidate's exact count.` against a growth budget that is `a lower`-`bound on the unexplored space rather than its size`, wrapped across two lines), `CoverBudgetStop` says `This is a lower bound on the unexplored space rather than its size: the`, and `PlanBudgetStop` says `This is a lower bound on the unexplored combinations rather than their` exact count. Publishing `actual` as a bare number would have made one field mean two things, which is why the delivered surface carries `BudgetResource::refusal()` beside it.

**Fact — only the five program-scoped resources are publicly reachable today.** `DeterministicBudgets` is `pub(crate)` and the public surface only ever passes `governed()`, so the truncation route needs a caller-stated budget set to reach; `session.rs`'s own reachability note records the same thing and the reason. The vocabulary is complete anyway, because the mapping into it must be total over what the compiler can raise.

**This repair does not change what the ticket is for.** The outcome, the non-goals, and the closing condition all survive; what changed is that the resource is a thirteen-row vocabulary over four authorities rather than a five-row one over one, and that the refusal has to say which kind of demand it reports.

## Decision boundary — the implementation is built, not accepted

The source now contains the complete draft described below: public `BudgetResource` and `BudgetRefusal` vocabularies, a payload-bearing `CompileFailureClass::BudgetExhausted`, exhaustive mappings from all thirteen internal resources, and the frontend rendering change. That makes this an acceptance ticket now, not an implementation ticket.

Tom decides whether to accept that exact caller-visible surface. **Recommendation: accept it as built.** A closed typed vocabulary is what lets a caller distinguish an exact demand from a lower bound without re-reading compiler source, and the public failure already promises structured classification. **Strongest counterpoint:** the compiler currently exposes only the five program-scoped rows through the public request path, so publishing all thirteen rows commits names for truncation paths a caller cannot yet reach. If that counterpoint wins, revise the public projection rather than silently treating the worker's 2026-08-08 type choice as acceptance.

The phrase **“Decided 2026-08-08” below records the implementation choice made by the worker; it is not Tom's acceptance provenance.** This ticket remains `awaiting-decision` until Tom accepts or revises the exact public enum variants, fields, exhaustiveness posture, and rendering behavior.

## Required work

- Carry `resource`, `limit`, and `actual` through `CompileFailureClass::BudgetExhausted`, matching the shape the two sibling arms already use, and keep the mapping exhaustive with no wildcard arm.
- Decide deliberately whether `resource` is `&'static str` like the sibling `rule` fields or a typed enumeration of budget names, and record the reason on the item. **Worker selection 2026-08-08, pending Tom's acceptance: a typed enumeration, `BudgetResource`.** The item's own reasoning — a closed set a caller may match totally — is not the load-bearing one, and ADR 0074 convention 5's clause test does not decide this: that test picks whether a type is `#[non_exhaustive]`, not whether it is a string. Two things decide it. First, ADR 0074 **convention 1** already names this case in its own words: a variant carries "the exhausted resource with its attempted and permitted quantities". Second, and dispositive, `actual` is an exact quantity for eight resources and a lower bound for five — a `&'static str` cannot tell a caller which, so the same source reading the ticket exists to remove would come straight back. `BudgetResource::refusal()` answers it in one total match. Convention 5's clause test *was* applied, to the attribute: `BudgetResource` is 5a and carries `#[non_exhaustive]`; `BudgetRefusal` is a closed two-way split and deliberately does not.
- Add the regression the bug lacked: a compilation that exceeds one budget must report that budget's name, limit, and actual, and the test must be watched failing before the fix.
- **Added by the work:** the exhausted-resource strings were four separate tables. They are now one, `BudgetResource::key()`, with each authority delegating to it, so the key a refusal reports and the key its explain record carries cannot drift apart. Every one of the thirteen strings is byte-identical to the string it replaced.

## Explicit non-goals

Not changing any budget's value. Not opening budgets to caller configuration — `DeterministicBudgets` staying `pub(crate)` is a separate question with its own argument, and this ticket is about attribution of a refusal rather than control over it.

## Scope note

`implementation/frontend` was added on 2026-08-08 because the work requires it and cannot be finished without it, not to widen the ticket. Any payload on `CompileFailureClass::BudgetExhausted` breaks the unit-variant pattern `CompileFailureClass::BudgetExhausted =>` in `crates/tiler-macros/src/aot.rs`'s `rendered_refusal` with `E0533`, and that crate is `implementation/frontend`. No open ticket held a live claim on that scope when it was added (`tkt list --scope implementation/frontend --status in-progress` and `--status review` were both empty). The edit there is one match arm.

That arm needed more than a `{ .. }` to be correct. Its message said the compiler "stopped searching", which is true of a truncating stop and false of every budget a macro expansion can actually reach, so it now names the resource and splits its advice on `refusal()`.

## Closes when

Tom accepts or revises the exact public failure surface, and a public caller can name the exhausted resource, its limit, and the value that exceeded it from the typed failure alone, with a regression test that was watched failing first.
