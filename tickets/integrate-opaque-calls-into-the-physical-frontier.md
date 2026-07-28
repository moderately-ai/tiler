---
id: integrate-opaque-calls-into-the-physical-frontier
title: Integrate opaque calls into the physical frontier as alternatives
status: todo
priority: p1
dependencies: [implement-opaque-physical-call-providers]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, extensions]
---
Split from `implement-opaque-physical-call-providers`, which delivered the declaration and registration machinery. This is the remainder, and it is different in kind: every piece landed so far was **additive** — new modules beside the existing frontier — while this one must change `frontier.rs` and the surrounding physical-planning path.

## What exists and must not be rebuilt

| Piece | Module |
| --- | --- |
| uncertain pressure estimates, provenance, explicit `Unknown` | `crate::estimate` |
| effects, motion, aliasing, conservative meet | `crate::effects` |
| typed failure stages and the fallback boundary | `crate::failure_stage` |
| named, role-typed ABI | `crate::call_abi` |
| affinity and memory-domain placement | `crate::call_placement` |
| cross-declaration coherence | `crate::call_declaration` |
| identity and registration | `crate::call_registry` |

Applicability is **already solved**: `frontier::TargetApplicability` resolves which providers apply to a target profile, over governed `TargetProfileKey`s with canonical deduplicated ordering. Do not add a second predicate over that question.

## The three remaining items, and why each is here rather than in the parent

**Additive coexistence with scheduled kernels.** An opaque call and a scheduled kernel must be able to be alternatives for one region. `ProposalBody::OpaqueCall` already exists as a variant the bounded frontier rejects explicitly (`frontier.rs`, alongside `KernelSubprogram` and `View`), so this is admitting a rejected variant rather than inventing one. That rejection is a real edit to existing enumeration code, which is why it did not belong with the additive slices.

**Numerical guarantees.** An opaque call's numerical realization has to be stated and checked against the region's contract; nothing landed so far touches numerics. `crate::honourability` and the `NumericalRealization` on `IndexRegion` are the existing authorities — check what they already answer before adding.

**Deterministic rejection and explain behaviour.** The typed errors exist (`PlacementError`, `AbiError`, `IncoherentDeclaration`, `CallRegistrationError`) but nothing emits explain records for them. The `pipeline/tests.rs` rule census is what will catch an unreported rejection, and its `tiler.cost.analytical.v1` entry is the worked example of how a new rule's record count is pinned.

## Structural consequence to expect, not to be surprised by

Admitting `ProposalBody::OpaqueCall` makes `MaterializationForm::OpaqueRuntimeValue` reachable, and that variant is currently one of eight `Reserved` values holding `implement-boundary-property-enforcers` closed. The trigger test `frontier::tests::the_bounded_profile_admits_no_undischarged_boundary` is expected to fire as part of this work. Do not repair it by widening the bounded property sets back into agreement — its firing is the signal that the enforcers ticket has become startable, and its message names the mismatch.

## Closes when

- An opaque call and a scheduled kernel can be alternatives for one region, and the frontier admits both without either being preferred by construction.
- A registered call's declarations are verified against the region and target profile at admission, with a typed rejection naming which declaration failed.
- An unknown or absent numerical realization rejects rather than inheriting the region's, for the same reason an undeclared effect is conservative.
- Every rejection emits a typed explain record; the rule census in `pipeline/tests.rs` is updated in the same change.
- Unknown pressure estimates still cannot establish hard feasibility — the absence of a conversion from `ResourceEstimate` is preserved, not worked around at the integration point.

## Sizing the type change, measured rather than estimated (2026-07-28)

Admitting `ProposalBody::OpaqueCall` is not a one-line change to the rejecting match. `AdmittedImplementation.verified` is a concrete `VerifiedScheduledRegion` (`frontier.rs:802`), and an opaque call is not one — it has no schedule, no index region, and no iteration domain. That field must become a sum over a scheduled region and an opaque call, and **every consumer must then say what it does for a call that has neither**.

There are nine `.verified()` sites, and they fall into three groups rather than one:

*Still answerable for an opaque call* — these read provenance-level facts a call also has:
- `selection.rs:1101`, `selection.rs:1260` — `semantic_members()`, for the identity cross-check.
- `selection.rs:1106` — `target_profile_key()`.

*Not answerable, and must reject or degrade explicitly*:
- `physical.rs:870` — `lower_scheduled_region(scheduled.verified())`. Lowering an opaque call is not lowering a scheduled region; this is where the two paths genuinely diverge.
- `pipeline/planning.rs:509` — collects verified regions for the plan.
- `frontier.rs:2113`, `selection.rs:2404` — test sites.

***Silently wrong if left alone*** — and this is the group worth flagging, because it is code landed earlier in this session and the failure is not a compile error in the obvious place:
- `component_cost.rs:433` (`Indexing`) and `component_cost.rs:513` (`RedundantWork`) both do `.verified().region().index` to read `iteration_shape` and `accesses`. An opaque call has no index region, so both must report `CostValue::Unknown` for any plan containing one — **not zero**. `component_cost::tests::unknown_is_not_a_zero` exists precisely for this substitution, and a plan whose indexing cost silently became zero would be ranked as free.
- `component_cost.rs:479` (`RedundantWork`) additionally reads `semantic_members()`, which *is* answerable — so that arm needs a partial answer rather than a wholesale `Unknown`, and deciding which is a judgement to make deliberately rather than by whichever branch the borrow checker accepts first.

**`MemoryTraffic` is already safe by construction**: it matches on `numerical.profile_key` and falls to `Unknown` on anything unrecognized, so an opaque call reaches the wildcard rather than a wrong number. That was written as a dtype guard and turns out to cover this too — worth noting because the other two arms were written the same day and are not.

*The check that establishes this list, reproducible in one line:* `grep -rn '\.verified()' crates/tiler-compiler/src/` returns nine sites; `grep -n 'struct AdmittedImplementation' -A 12 crates/tiler-compiler/src/frontier.rs` shows the field is concrete.

