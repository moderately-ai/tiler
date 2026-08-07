---
id: widen-the-strategy-recognizer-past-the-f32-wall
title: Widen the strategy recognizer past the f32 wall
status: in-progress
priority: p1
dependencies: []
related: [conform-the-bf16-vertical-end-to-end, establish-bf16-optimizer-legality]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [bf16, dtype, blocker]
claimed_from: todo
assignee: agent-recognizer
lease_expires_at: 1786125786
---
## The wall, and why it was unowned

**Fact — `select_supported_strategy` (`crates/tiler-compiler/src/request.rs:4206`) refuses every non-`f32` program under the rule `dtype-f32`, before a subject is normalized.** Nothing downstream can produce the `PlanAlternative` that `compile()`, the artifact envelope, and the runtime routing commit consume, so those three layers are unreachable for BF16 by any route.

Three existing sites assert that wall deliberately — `crates/tiler-compiler/tests/bf16_numerical_contract.rs:399,429,621` and `crates/tiler-compiler/src/pipeline/tests.rs:3922-3927` — so it is a stated boundary rather than an oversight.

**It became load-bearing on 2026-08-07.** [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) crossed BF16 from semantic construction to a real GPU dispatch and a bit comparison, and had to assemble its region through `tiler-ir`'s public builders because of this wall. That closed the ticket on everything else its evidence list demanded, and left its **first bullet** — a program carried through compile, artifact, runtime routing, and dispatch — structurally unreachable. It was the fourth block that ticket hit, and the only one with no owner: `establish-bf16-optimizer-legality` holds legality *keying*, not recognition.

## What this owes

- The recognizer admitting a non-`f32` program whose dtype the profile and contract support, so a `PlanAlternative` exists for it to plan.
- **The refusal kept where it belongs.** Widening recognition must not admit a dtype the target cannot honour — that refusal is the target profile's and the numerical contract's, and it must still fire, from its own authority, with its own typed cause. A program that was refused as `dtype-f32` and is now refused as unhonourable is the *correct* outcome for an unsupported row, and the two must be distinguishable.
- The three deliberate wall assertions **re-founded rather than deleted**: whatever still refuses after this lands is what they should assert. Deleting them would remove the evidence that the boundary is where it is meant to be.
- Every downstream leg the wall was hiding checked rather than assumed. A layer that has never seen a non-`f32` `PlanAlternative` may carry its own `f32` assumption; finding one is a result, not a setback.

## Explicit non-goals

Not legality keying — that is `establish-bf16-optimizer-legality`. Not a new dtype, not a new family, and not a widening of what any target declares it can honour. Not the conformance run itself: closing this lets [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md)'s first bullet be met by a **follow-up run**, which is its own ticket rather than this one.

## Required evidence

- A non-`f32` program reaching a selected `PlanAlternative`, named.
- The unhonourable case still refused, from the profile's or contract's authority, watched failing.
- The three wall assertions re-founded on what actually refuses now.
- Whatever identity moves, enumerated and recomputed on the merged tree.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration of the BF16 vertical, from a block its worker found, refused to edit around because the crate was live-claimed, and reported as unowned. It is p1 because it is the single structural obstacle between a BF16 program and the compiled path, and because three other tickets' evidence lists narrow to it.
