---
id: test-the-cooperative-lowering-shape-refusal
title: Test the cooperative lowering-shape refusal
status: done
priority: p3
dependencies: []
related: [implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5, record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims]
scopes: [implementation/ir, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, testing]
---
## User-visible outcome

`KernelDiagnostic::CooperativeLoweringShape` is a refusal that has been watched refusing, so every documented sentence saying a tile outside the emitted shape is rejected by name rests on a case that must fail rather than on a reading of the destructuring that produces it.

## Why this is a separate ticket

Found on 2026-08-05 while recording [ADR 0097](../docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md) as implemented, read at `92a8a64e`. The exact check is `grep -rn 'CooperativeLoweringShape' crates/`: at this ticket's re-read base it returns the enum variant and rule mapping in `crates/tiler-ir/src/kernel/error.rs`, and the single binding in `cooperative_plan` (`crates/tiler-ir/src/kernel/lower.rs`) — and no test, in the whole workspace. The refusal is therefore implemented support with no tested guarantee, which is two of the four maturity claims [AGENTS.md](../AGENTS.md) keeps apart, and it is out of scope for the recording ticket, which holds `contracts/decisions` and `contracts/navigation` only.

This is not a hypothetical path. Three ticket bodies and the ADR describe what it refuses as a load-bearing boundary — [`admit-the-first-typed-synchronization-point-and-atomic-target-authority`](admit-the-first-typed-synchronization-point-and-atomic-target-authority.md), [`lower-a-loop-carried-cooperative-body`](lower-a-loop-carried-cooperative-body.md), and [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) each enumerate the unsupported cases by name — and since the two-dimensional staging relation landed, a rank-two tile that *verifies* as a schedule is one of them: `a_two_dimensional_cooperative_tile_verifies` in `crates/tiler-ir/src/schedule/builder.rs` builds one, and lowering it is the first case where a schedule the verifier admits has no kernel body at all.

## What the tests must cover

The 13 top-level `cooperative_plan` refusal groups (excluding the non-cooperative `Ok(None)` path) need one unchanged-assertion subject probe per independently mutable clause that can reach its guard. Prefer a re-verified public schedule for a shape the schedule verifier admits; use the real-plan projection only where the verifier correctly rejects the malformed subject first.

- Re-verified public schedules: rank-two spans, a second complete allocation, a third phase, a valid two-slot write, a partial read, and a non-prefix commit.
- Direct real-plan projection: access layout, each staging-ID operand, participant-product failure, each span-rank dimension, zero/nonzero stride, zero visibility edges, zero/two visibility dischargers, unsupported barrier spelling, a missing round anti-dependency discharger, and contributor multiplication overflow.
- The `[staging]`, `[produce, consume]`, and exact staged-access destructures make multiple visibility edges, multiple anti-dependencies, and the rounds/anti-dependency mismatch unreachable *after the earlier guards* at this base. They are defensive source branches, not distinct reached test claims.

`crates/tiler-ir/src/kernel/tests.rs` already builds cooperative regions (`a_cooperative_region_lowers_to_a_staged_fenced_body`, `a_loop_carried_tile_lowers_to_a_peeled_round_body`), so the fixtures to perturb exist and no new harness is needed.

## Closes when

Every independently mutable, reachable `cooperative_plan` refusal guard has a test that fails without its production subject check; public schedules demonstrate the verifier/lowerer boundary and defensive malformed subjects use the real-plan projection. The explicit structural-unreachable classification above is retained rather than silently calling every literal arm tested. `grep -rn 'CooperativeLoweringShape' crates/` returns test sites as well as the definition and the binding, and ADR 0097's implementation-boundary paragraph receives a dated evidence correction in this carrier. `contracts/decisions` is declared for that correction; preserve accepted history rather than rewriting it.

## Graph maintenance

- Filed 2026-08-05 by [`record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims`](record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims.md), which could not reach `crates/` from its scopes.
- It blocks nothing: the refusal is present and correct by reading, and what is missing is the evidence that it can say no. It is `todo` rather than `deferred` because no trigger gates it — the work is reachable now.

## Fact audit — 2026-08-10

**Correction.** The 2026-08-05 census called the `KernelDiagnostic::CooperativeLoweringShape` rule mapping its “diagnostic string.” At `d8f4cfca07c9bc9f8a71a3fa172eb9501bac349f`, `rg -n 'CooperativeLoweringShape' crates/` instead finds the lowerer's binding, the enum variant, and `KernelDiagnostic::rule`'s stable-rule mapping; it still finds no test site. The repair changes no outcome, authority, public boundary, or identity claim.

**Review correction — 2026-08-10, review of `90b5d5db6844ca44ea05e2c6e655d15b2e477b62`.** An initial test probe changed an expected assertion, which shows only that the assertion executes and is not evidence that the guarded subject matters. It is not closing evidence. The retained evidence instead perturbs each separately mutable production check while preserving the corresponding assertion: the unchanged test reports `Ok` where it requires `Err(CooperativeLoweringShape)`. The review also found that the earlier five-plus-rank census understated the 13 top-level refusal groups: access layout, both staging-ID operands, participant-product failure, zero visibility edges, unsupported barrier spelling, contributor multiplication overflow, and independently mutable span ranks were missing. This ticket now distinguishes the six re-verified public schedule variants from direct defensive projection subjects. Multiple visibility edges, multiple anti-dependencies, and the `rounds`/anti-dependency mismatch remain structurally unreachable only *after* the earlier exact destructures; zero visibility is privately reachable by duplicate phase IDs and is now tested. This correction changes neither the refusal's public identity nor its authority.

**Second review correction — 2026-08-10, review of `756a8346b79f499e3b336279b5625993d3257168`.** The public second-allocation and two-slot-write variants did not isolate their owning checks: broadening `[staging]` still met the later access-layout refusal, and removing only `write.span.count != 1` still met the widened read-count refusal. Two direct projection subjects now preserve every downstream condition: an otherwise-unused second staging allocation and a two-slot producing write with the original read. Their production-subject perturbations leave the assertions unchanged and report `Ok(())` where `Err(CooperativeLoweringShape)` is required. This corrects evidence only; the ticket outcome, authority, public identity, and supported lowering shape stay unchanged.
