---
id: carry-the-exhausted-resource-through-the-budget-refusal
title: Carry the exhausted resource through the budget refusal
status: in-progress
priority: p3
dependencies: []
related: [measure-executable-coverage-identity-growth-against-the-program-identity-bound]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, diagnostics]
claimed_from: todo
assignee: coord
lease_expires_at: 1786182999
---
## User-visible outcome

A caller whose compilation stops on a deterministic budget learns which budget, what its limit was, and what the program asked for — instead of the single word `BudgetExhausted`.

## Why this exists

**Measurement, 2026-08-05, from [`spikes/program-planning/identity-growth/`](../spikes/program-planning/identity-growth/README.md).** The spike compiles one program past the governed `semantic_operations` budget to prove its ladder covers the whole reachable domain. The refusal it can observe is exactly:

```
CompileFailure { class: BudgetExhausted, explain: "absent (refused before a target-qualified trace)" }
```

That is everything a public caller gets. The spike had to attribute the refusal to `semantic_operations` by reading `crates/tiler-compiler/src/request.rs:821`, which is a source reading rather than something the refusal said.

**Fact — the information exists and is discarded at the boundary.** `RequestError::BudgetExceeded` (`crates/tiler-compiler/src/request.rs:2169`) carries `resource`, `limit`, and `actual`. `CompileError::BudgetExhausted` wraps that whole error (`pipeline.rs:504`). `class_of` then maps it to the unit variant `CompileFailureClass::BudgetExhausted` (`session.rs:1232`), dropping all three fields — while the sibling arms for `InvalidRequest` and `UnsupportedCapability` both preserve a `rule` through `rule_of`, which already has a `BudgetExceeded` arm returning `resource` (`session.rs:1255`) that this path never reaches.

**Inference — the asymmetry looks like an oversight rather than a decision.** `CompileFailureClass`'s own documentation says the enum exists so "a caller branches on the boundary that refused instead of matching on text", and two of its five variants carry the refusing rule for that reason. A budget refusal is the one class where the actionable detail is a number, and it is the one class that carries nothing. Nothing in `session.rs` argues for the omission, which is what separates this from the deliberate narrowings recorded elsewhere on that surface.

**Fact — a pre-trace refusal has no explain report either.** Request verification runs before a target-qualified trace can be opened, so `CompileFailure::explain` returns `None` here by construction. The typed fields are therefore the only route; there is no fallback a caller could use instead.

## Required work

- Carry `resource`, `limit`, and `actual` through `CompileFailureClass::BudgetExhausted`, matching the shape the two sibling arms already use, and keep the mapping exhaustive with no wildcard arm.
- Decide deliberately whether `resource` is `&'static str` like the sibling `rule` fields or a typed enumeration of budget names, and record the reason on the item. A typed enumeration is the better fit for a closed set a caller may match totally, and ADR 0074 convention 5's clause test decides it rather than preference.
- Add the regression the bug lacked: a compilation that exceeds one budget must report that budget's name, limit, and actual, and the test must be watched failing before the fix.

## Explicit non-goals

Not changing any budget's value. Not opening budgets to caller configuration — `DeterministicBudgets` staying `pub(crate)` is a separate question with its own argument, and this ticket is about attribution of a refusal rather than control over it.

## Closes when

A public caller can name the exhausted resource, its limit, and the value that exceeded it from the typed failure alone, with a regression test that was watched failing first.
