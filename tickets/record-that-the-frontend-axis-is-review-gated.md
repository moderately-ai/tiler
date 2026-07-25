---
id: record-that-the-frontend-axis-is-review-gated
title: The frontend axis is gated on a human review, not on engineering
status: done
priority: p1
dependencies: []
related: []
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: [planning, frontend, process]
---
A finding about the parallel-widening plan itself, recorded because the plan's method assumes the axes are independently workable and one of them is not.

**Fact — the frontend axis's dependency structure, read from the tickets.**

- `prototype-inline-proc-macro-frontend` (todo) depends on `prototype-public-compiler-api` and `prototype-neutral-artifact-codec`.
- `prototype-candle-metal-adapter` (todo) depends on `prototype-inline-aot-integration-proof`.
- `prototype-macro-embedding-and-cargo-behavior` (todo) depends on `prototype-inline-proc-macro-frontend`, `prototype-expansion-content-cache`, `prototype-artifact-family-delivery`, and `prototype-metal-aot-slice`.
- `macro-build-environment`, `proc-macro-extension-visibility`, `resolve-macro-environment-alarm-path-dependence`, and `repair-macro-and-embedding-harness-integrity` are already done or closed.

**Fact — `prototype-public-compiler-api`'s closing condition is a review.** That ticket states that "any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit", and its own recorded status names "Tom has not reviewed it" as the first reason it stays open. `pub mod session` exists and works; what is missing is acceptance, not code.

## Re-derived on 2026-07-25, and the finding got sharper rather than weaker

The dependency structure above was accurate when written and three of its edges have since closed. The correction matters, because the original inference is now wrong in the direction that strengthens the conclusion.

**Fact — what closed.** `prototype-neutral-artifact-codec` is done, `prototype-expansion-content-cache` is done, and `prototype-artifact-family-delivery` is done.

**Fact — `prototype-public-compiler-api` is itself dependency-satisfied.** Its only dependency, `prototype-optimizer-conformance-gate`, is done, and `tkt ready` lists it. Nothing in the work graph is upstream of it.

**Inference — the axis now has *zero* dependency-satisfied tickets, not one.** The original text named `prototype-expansion-content-cache` as the single frontend-adjacent ticket that could proceed, and noted it was "a cache rather than a frontend". That ticket is done, so the qualification it carried has been spent: every remaining ticket carrying `implementation/frontend` is blocked.

**Inference — every path through the axis funnels through the same single review.** `prototype-neutral-artifact-codec` closing leaves `prototype-public-compiler-api` as the *sole* remaining dependency of `prototype-inline-proc-macro-frontend`. That ticket in turn is the sole remaining dependency of `generate-cfg-gated-artifact-family-delivery` and one of two remaining for `prototype-macro-embedding-and-cargo-behavior`, which gates `prototype-inline-aot-integration-proof`, which gates `prototype-candle-metal-adapter`. Six tickets, one review.

This is the strongest form the finding can take: the axis is not merely ordered after a decision, it is ordered after *one* decision, and no amount of engineering elsewhere shortens the chain. Closing more upstream tickets cannot help — it has already happened three times and moved the axis no closer.

**Why this matters to the plan rather than only to the schedule.** The parallel-widening method holds that each axis stresses a different seam and that sequencing them would let each be designed around the previous one's answers. The frontend axis is the one that "exercises artifact identity, caching and the public boundary under real reuse" — it is the only axis that tests the public boundary at all, and it is gated on that boundary being accepted. That is not a scheduling accident: an axis whose purpose is to stress a boundary cannot run before the boundary is agreed, so this axis is genuinely ordered after a decision rather than after other engineering.

**The other four axes are unaffected.** Shapes, numerics, operations, and targets each have dependency-satisfied work that touches no public boundary, which is evidence that the axes are otherwise independent as the method assumes.

## Closes when

Either `prototype-public-compiler-api` is reviewed and accepted, unblocking the axis, or the plan records that the frontend axis is deliberately sequenced after that acceptance and does not count toward parallel widening until then. Do not close this by starting frontend work that routes around the unreviewed boundary — that would answer the review question by omission, which is what `prototype-public-compiler-api` exists to prevent.

## Correction — this closes on the FIRST branch, and the coordinator got it wrong

**Fact — the review had already happened when the outcome below was written.** `tickets/prototype-public-compiler-api.md` carries `## Decision — Tom, 2026-07-25`: "Approved: promote the compiler boundary. `pub mod session` in its current shape … together with `pub mod abi` in `tiler-ir`. This closes the ADR 0075 always-ask review that has been gating the entire frontend axis." Commit `a69b0c7`, authored the same day, records it in its message: "Four public promotions approved: the compiler boundary, the proof-sidecar facade, the feasibility rule set, and the numerical preference list."

**Fact — how the error was made, because the mechanism is reusable.** The ticket was read with `tkt show … | head -35`. Line 51 of that file is the Status section's "**Tom has not reviewed it**", written *before* the approval and never revised; the approval is at line 67, below the window. `AGENTS.md` names this exact failure — "a bounded window (`head -N`, a `sed` range, a truncated diff) can split the construct being searched for" — and it was made anyway, then reported twice as a live blocker.

**The axis is therefore engineering-gated, not review-gated.** Every "six tickets, one review" inference above is wrong in its conclusion while remaining right in its structure: the chain is real and it does funnel through `prototype-public-compiler-api`, but what that ticket now waits on is its own remaining work, not an approval. It is `todo`, dependency-satisfied, and dispatchable.

**What is genuinely still owner-reserved is narrower and lives on that ticket**, not on the axis: `CompileFailure` becoming a struct with the four classes moved to `CompileFailureClass` is a change to an existing public signature, which ADR 0075 routes to Tom, and the promotion approved `pub mod session` "in its current shape" rather than that reshaping. That is one signature, not an axis.

## Outcome

**Closed on the first branch: `prototype-public-compiler-api` was reviewed and accepted, which unblocks the axis.** The second-branch reasoning below is preserved because its argument stands on its own and would govern if the approval had not existed — but it is not what closed this ticket, and a reader must not take it as evidence that the axis is still sequenced behind a decision.

**Superseded — the deliberate-sequencing outcome, recorded as originally written.** The frontend axis was to be removed from the width calculation until `prototype-public-compiler-api` was accepted, rather than treated as a scheduling shortfall to be worked around.

The reason this is the correct branch rather than the convenient one is that the axis's *purpose* is to stress the public boundary under real reuse. An axis that exists to test whether a boundary survives a consumer cannot be run before the boundary is agreed without destroying the thing it measures: whatever the frontend then built against would define the boundary by construction, and the review would be ratifying a fait accompli. The four remaining axes — shapes, numerics, operations, targets — each have dependency-satisfied work touching no public boundary, so the plan's independence assumption holds for them and the width is honestly stated as four.

**What this does not license.** It does not license building a frontend against a private path, a duplicated surface, or a `pub(crate)` item promoted without review. `AGENTS.md` and ADR 0075 both place that promotion with Tom, and ADR 0074 convention 7 makes the crate-private draft the accepted staging state precisely so that an unreachable capability is not mistaken for a defect to route around.

**Trigger for reopening.** If the review has not happened by the time a *second* axis becomes review-gated, the finding changes class: one axis waiting on one decision is a sequencing fact, and two axes waiting is evidence that the review cadence, not the dependency graph, is the binding constraint on parallel work. That would be a finding about the method rather than about the frontend, and it deserves its own ticket rather than an amendment here.

The review itself is not closed by this ticket and is not the coordinator's to close. It is surfaced to Tom with the boundary in hand.
