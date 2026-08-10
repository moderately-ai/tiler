---
id: correct-the-floor-relation-prose-in-the-backend-scoped-route-answer-record
title: Correct residual floor and capacity-comparison relation prose in the backend-scoped route-answer research record
status: done
priority: p2
dependencies: []
related: [correct-the-residual-floor-relation-prose-outside-the-artifact-scopes, correct-the-runtime-route-requirement-relation-prose, correct-the-subgroup-threads-route-dimension-meaning, rename-the-route-resource-floor-vocabulary-for-its-corrected-relation]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, runtime, naming]
---

## Why this exists

[`correct-the-residual-floor-relation-prose-outside-the-artifact-scopes`](correct-the-residual-floor-relation-prose-outside-the-artifact-scopes.md) corrected asserting floor prose in accepted ADR 0090 / 0094 and in `research/scheduling` and `research/extensions`, and filed the `implementation/runtime` residual as [`correct-the-runtime-route-requirement-relation-prose`](correct-the-runtime-route-requirement-relation-prose.md) (now done). Neither ticket held `research/runtime`. A 2026-08-10 ticket audit found live asserting floor / capacity-comparison vocabulary still in `docs/research/runtime/backend-scoped-route-requirement-answers.md`, including a false API shape and stale line pins against `requirement.rs`.

**Fact — the quantitative half is equality over `required`, not a floor.** `crates/tiler-artifact/src/program/requirement.rs` carries `RouteResourceRequirement { dimension, required: u64 }` with no `minimum` field. `is_satisfied_by` reads `RouteResourceDimension::SubgroupThreads => self.required == observed`. The module heading is "Why the relation is an equality and not a floor". The loader owns the comparison; adapters report `Quantity` / `Feature` / `Unrecognized`. Reproduce: `grep -n "is_satisfied_by" -A6 crates/tiler-artifact/src/program/requirement.rs`.

**Fact — the research record still teaches the retired floor model in at least two live sentences.** Read at current tree in `docs/research/runtime/backend-scoped-route-requirement-answers.md`:

- Shape bullet: "a quantity for a qualitative row and a verdict for a floor are `MisansweredRouteRequirement` rather than coerced."
- ADR 0090 item 4 conflict paragraph: "What item 4 forbids is the adapter reversing a *capacity comparison* — the quantitative half, where the loader holds the threshold and the direction (`floor.is_satisfied_by(observed)`, `requirement.rs:186-188`)."

Neither sentence is a *rejecting* use of "floor". Both assert the retired relation or API. `floor.is_satisfied_by` is not an API that exists; the method is on `RouteResourceRequirement` and compares `self.required`. The `requirement.rs:186-188` pin is stale relative to the equality arm. Reproduce: `grep -n 'floor.is_satisfied_by\|verdict for a floor\|capacity comparison' docs/research/runtime/backend-scoped-route-requirement-answers.md`.

**Why this is its own ticket.** The parent residual ticket's scopes and Outcome census never included `research/runtime`. Folding a doc rewrite into a closed ticket's Outcome would pretend that census closed a population it never measured. The runtime crate residual is already done and must not be reopened for a research-record edit.

## What closes this

No live asserting sentence in `docs/research/runtime/backend-scoped-route-requirement-answers.md` describes a route resource requirement as a floor, names a capacity comparison for the quantitative half, or cites `floor.is_satisfied_by` / a stale `minimum` / wrong `requirement.rs` line pin for `SubgroupThreads` satisfaction. Corrected sentences use the landed model: dimension-owned relation, `required`, equality for `SubgroupThreads`, loader-owned satisfaction, `LoadRejection::UnsatisfiedRouteRequirement` / the three-way `Unowned` / `Misanswered` / `Unsatisfied` split — without forking ADR 0090 item 4's decision that an adapter reports facts and never adjudicates them. Every corrected sentence carries a dated marker naming this ticket. *Rejecting* or historical uses of "floor" (including any retained drafted-ADR span material that deliberately preserves pre-landing wording) stay; only asserting live prose is swept. No `decision_status` moves. Prose only; no crate edits.

## Non-goals

- Re-opening or re-editing the parent residual ticket's ADR 0090 / 0094 / scheduling / extensions sites (already corrected with their own markers).
- Runtime crate doc comments (owned and closed by `correct-the-runtime-route-requirement-relation-prose`).
- Changing ADR 0090 item 4's meaning or `decision_status`.
- Renaming public Rust types (type rename already landed under `rename-the-route-resource-floor-vocabulary-for-its-corrected-relation`).

## Outcome

**Fact audit, read at exact base `b3b1652faa6c0060e4958782c2d5d37b563b9f8b` on 2026-08-10.** Both stated Facts verified with no ticket repair: `RouteResourceRequirement` owns `required`, and its `is_satisfied_by` match makes `RouteResourceDimension::SubgroupThreads` an equality; the target record had the two quoted live, asserting statements outside its retained drafted-ADR span. The source/document correction is delivered by `d6749aa56cce2f4e54cf29f480a09a6cfd09c296`: the shape paragraph now rejects a feature verdict for a quantitative row as `LoadRejection::MisansweredRouteRequirement`, and the ADR 0090 item-4 paragraph now states that an adapter reports `Quantity(observed)` while the loader applies the dimension-owned relation to `required`, with equality for `SubgroupThreads` and the `Unowned` / `Misanswered` / `Unsatisfied` `LoadRejection` split. Each corrected sentence has a 2026-08-10 marker naming this ticket.

**Retained-word census.** `rg -n -i 'floor' docs/research/runtime/backend-scoped-route-requirement-answers.md` reports six matches after the correction: a rejecting qualitative-row explanation, a backend-feature ordering counterexample, the two marker links naming this ticket, the valid future CPU-floor/subgroup-equality contrast that demonstrates a relation belongs to its dimension, and the historical `ResourceFloor` transfer note. The deliberately retained drafted-ADR span was not edited. The retired live patterns are absent: `floor.is_satisfied_by`, `verdict for a floor`, `capacity comparison`, `minimum()`, and `requirement.rs:186-188` each have zero matches in the target record.

**Checks.** `make citations`; `tkt lint --format json`; and `git diff --check` passed on the delivered documentation commit. `tkt guard tkt/correct-the-floor-relation-prose-in-the-backend-scoped-route-answer-record --format json` found no under-declared scope or conflict; it reported only a non-gating direct `research/runtime` collision with `reconcile-direct-input-conformance-order-with-adr-0033`. The documentation-only delta carries the published full gate at `b840f8625af3e831519a602afcd9bb8ebe36fc63`, the parent of the exact base: it changes no gate-invalidating path. This outcome records evidence only; the ticket remains `in-progress` and is neither done nor closed.
