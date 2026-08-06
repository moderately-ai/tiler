---
id: widen-the-deterministic-budgets-to-the-decoder-layer-program
title: Widen the deterministic budgets to the decoder-layer program
status: in-progress
priority: p1
dependencies: [assemble-the-decoder-layer-program, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, enumerate-the-split-reduction-on-the-planning-frontier, prototype-public-compiler-api]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, budgets, identity, language-model, class-generic-capability]
claimed_from: todo
assignee: agent-budget-widen
lease_expires_at: 1785983992
---
## User-visible outcome

A compilation request carrying a complete decoder layer is admitted rather than refused for its size, so the refusals a transformer meets are about what it computes rather than about how many operations it has.

## Why this exists

**Fact.** `verify_program` at `crates/tiler-compiler/src/request.rs:1886` checks four resources against `DeterministicBudgets::governed()`: `semantic_values` 16 against `program.value_count()`, `semantic_operations` 8 against `program.operation_count()`, `regions` 3, `host_expression_nodes` 32, and `buffers` 4 against `4.max(program.input_count().saturating_add(1))`.

**Inference.** [The L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) derives that the decoder-layer program exceeds three of the four — eighteen inputs against a buffers actual of nineteen, at least twenty-one values, and at least fifty-one operations — while the embedding and vocabulary-projection programs pass every one.

## What makes this a decision rather than a knob

**Fact.** Every budget is written into `VerifiedRequestSubject::canonical_bytes` (`crates/tiler-compiler/src/request.rs:1193`–`1210`) and therefore into artifact identity. The comment on `DeterministicBudgets::governed` states the consequence directly: widening moves "every governed compilation's request subject, and therefore every artifact identity and cache entry derived from it — for programs that never assemble a split as much as for ones that do, because the budget is a property of the *request* rather than of the plan chosen for it."

So the number of widenings is itself a cost. **This is L6's D-18 and it is Tom's:** take one widening sized to the layer program, as the split reduction's widening from two regions and three buffers to three and four was taken once and stated, or let the budgets grow with the profile and move every identity each time.

## Required content

- The new values, each justified by the largest program shape this profile may assemble rather than by the smallest it might — the same rule the current comment states.
- A budget is an upper bound: widening admits program shapes and never requires them, and `verify_program` must still refuse a request whose shape needs more.
- The artifact-identity movement is stated in the change, not discovered by a reader later. No pinned golden encodes these bytes, so nothing in the suite will report it.

## Closes when

The values are chosen from the layer program's measured counts, Tom has answered D-18, `RequestError::BudgetExceeded` still fires against a program one step larger than the new bound, and the identity movement is recorded.

## D-18 — answered

Answered by Tom on 2026-08-06 at the live decision review in the coordination session, witnessed first-hand by the coordinator: **one widening, sized to the layer program** — values justified by the largest program shape this profile may assemble, per the split-reduction precedent, with the identity movement stated in the change. The ticket is dispatchable once `implementation/compiler` frees (held by the concatenate fusion-role claim at the time of the answer).
