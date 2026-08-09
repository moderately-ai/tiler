---
id: reconcile-the-published-and-consumed-doc-record
title: Reconcile the published-and-consumed doc record
status: done
priority: p2
dependencies: []
related: []
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

One in-file account of the published-and-consumed capability, with the dead test name retired everywhere it propagated.

## Why this still exists (re-audited 2026-08-09)

The compiler source is already correct. At the source-safe anchors `One spelling of the overlap is now admitted` and `a_published_and_consumed_intermediate_compiles_and_agrees`, `request.rs` describes the admitted narrowing and its live test; no compiler edit remains. The residual drift is durable record state: [the flash capability record](../docs/research/program-planning/flash-class-capability-set.md) still presents the refusal as current, and three completed tickets retain live-looking current-status clauses naming `a_published_and_consumed_intermediate_refuses_by_name`. This ticket now owns only dated reconciliation in those records plus its own corrected scope and outcome. Historical measurements and struck/refuted passages stay intact.

## Closes when

The flash capability row states the admitted current boundary, the three completed tickets carry dated corrections distinguishing their historical refusal from current behavior, and no compiler source or behavior moves.

## Fact repair and current file population, 2026-08-09

**Verified.** `request.rs` and `pipeline/conformance.rs` already agree; the current positive test exists and the old refusing test does not. **False at this base:** the prior claim that compiler prose still needed repair and that six ticket bodies remained live. Full-record reading narrowed the editable population to this ticket, the flash capability record, and three completed ticket records: [`admit-elementwise-epilogues-over-a-materialized-intermediate`](admit-elementwise-epilogues-over-a-materialized-intermediate.md), [`accept-the-public-compiler-facade-boundary`](accept-the-public-compiler-facade-boundary.md), and [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md). Other hits are explicitly struck, refuted, or historical and remain useful evidence.

## Outcome, 2026-08-09

The flash capability table now states the admitted part-boundary overlap and its bounded remainders. Each of the three completed records carries a dated correction that preserves its historical refusal while naming the live positive test. No compiler source, behavior, public surface, identity, or test changed; the ticket's false implementation scope was replaced by the exact `research/program-planning` document scope. The revised closing condition is fully met, so this ticket is `done`.
