---
id: test-the-cooperative-lowering-shape-refusal
title: Test the cooperative lowering-shape refusal
status: in-progress
priority: p3
dependencies: []
related: [implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5, record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims]
scopes: [implementation/ir, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, testing]
claimed_from: todo
assignee: terra-cooperative-refusal
lease_expires_at: 1786418733
---
## User-visible outcome

`KernelDiagnostic::CooperativeLoweringShape` is a refusal that has been watched refusing, so every documented sentence saying a tile outside the emitted shape is rejected by name rests on a case that must fail rather than on a reading of the destructuring that produces it.

## Why this is a separate ticket

Found on 2026-08-05 while recording [ADR 0097](../docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md) as implemented, read at `92a8a64e`. The exact check is `grep -rn 'CooperativeLoweringShape' crates/`: it returns the variant (`crates/tiler-ir/src/kernel/error.rs`), its diagnostic string in the same file, and the single binding in `cooperative_plan` (`crates/tiler-ir/src/kernel/lower.rs`) — and no test, in the whole workspace. The refusal is therefore implemented support with no tested guarantee, which is two of the four maturity claims [AGENTS.md](../AGENTS.md) keeps apart, and it is out of scope for the recording ticket, which holds `contracts/decisions` and `contracts/navigation` only.

This is not a hypothetical path. Three ticket bodies and the ADR describe what it refuses as a load-bearing boundary — [`admit-the-first-typed-synchronization-point-and-atomic-target-authority`](admit-the-first-typed-synchronization-point-and-atomic-target-authority.md), [`lower-a-loop-carried-cooperative-body`](lower-a-loop-carried-cooperative-body.md), and [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) each enumerate the unsupported cases by name — and since the two-dimensional staging relation landed, a rank-two tile that *verifies* as a schedule is one of them: `a_two_dimensional_cooperative_tile_verifies` in `crates/tiler-ir/src/schedule/builder.rs` builds one, and lowering it is the first case where a schedule the verifier admits has no kernel body at all.

## What the tests must cover

One case per refusal `cooperative_plan` states, each watched failing, and each perturbing a fixture that lowers successfully so the rejection names the rule rather than a difference the fixture carried:

- A rank-two participant space with rank-two spans — the destructuring at the stride vectors, which is the newest arm and the one no test reaches today.
- More than one staging allocation, and more than two phases.
- A staged span other than one-slot-per-participant write and whole-set read: a write `count != 1`, a zero produce stride, a nonzero consume stride, and a read `count` that is not the participant count.
- A commit range not starting at participant zero.
- Other than exactly one visibility edge and at most one anti-dependency edge, and the `rounds > 1` versus round-barrier equality.

`crates/tiler-ir/src/kernel/tests.rs` already builds cooperative regions (`a_cooperative_region_lowers_to_a_staged_fenced_body`, `a_loop_carried_tile_lowers_to_a_peeled_round_body`), so the fixtures to perturb exist and no new harness is needed.

## Closes when

Every arm of `cooperative_plan`'s refusal has a test that fails without it, `grep -rn 'CooperativeLoweringShape' crates/` returns test sites as well as the definition and the binding, and the ADR 0097 implementation-boundary paragraph that records this as an untested refusal receives a dated evidence correction in this same carrier. `contracts/decisions` is declared for that correction; preserve the accepted body rather than rewriting it.

## Graph maintenance

- Filed 2026-08-05 by [`record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims`](record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims.md), which could not reach `crates/` from its scopes.
- It blocks nothing: the refusal is present and correct by reading, and what is missing is the evidence that it can say no. It is `todo` rather than `deferred` because no trigger gates it — the work is reachable now.
