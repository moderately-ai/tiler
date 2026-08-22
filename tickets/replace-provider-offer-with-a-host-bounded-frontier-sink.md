---
id: replace-provider-offer-with-a-host-bounded-frontier-sink
title: Replace ProviderOffer vectors with a host-bounded frontier sink
status: todo
priority: p1
dependencies: [decide-whether-the-implementation-frontier-owes-a-retention-budget, calibrate-the-physical-frontier-provider-and-outcome-budgets]
related: [accept-the-installed-physical-provider-public-surface, design-explicit-caller-selected-budget-exhaustion-policies]
scopes: [implementation/compiler, contracts/optimizer, contracts/decisions, research/program-planning, research/extensions, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [optimizer, budgets, backend-providers, public-boundary, identity]
---
## User-visible outcome

An installed physical provider cannot hand Tiler an unbounded proposal/decline collection. It emits through a host-owned bounded channel; exceeding the declared policy returns typed `BudgetExhausted` before proposal verification, selection, or artifact construction, with no partial frontier or silent fallback.

## Required delivery

- Replace `PhysicalImplementationProvider::propose -> ProviderOffer` and the complete `Vec` carrier with an object-safe host-owned bounded sink or equivalent pull protocol. Charge every proposal and every `DeclinedStrategy` before accepting it.
- Preflight the complete provider population, including the governed provider. Bound zero-outcome provider invocations independently from emitted outcomes.
- Latch overflow even if provider code ignores an individual insertion result. Refuse the whole compilation at `limit + 1`; do not retain a prefix, reserve a governed entry, or retry another backend, target, provider set, numerical contract, or budget policy.
- Add the calibrated limits to the complete typed deterministic budget value, `BudgetResource` population, internal mappings, canonical request/evidence encoding, explain records, qualifier pins, and domain/version ledger. Keep artifact/cache consequences separate.
- Preserve additive/provider-order-independent semantics. The same provider set and outputs must either produce the same complete frontier or the same typed exhaustion regardless of installation order.
- Correct every contract and module comment that presently calls the unbounded stored admission set or borrowed Pareto view bounded.

## Public-boundary rule

This deliberately revises a recently accepted public trait and `ProviderOffer` surface. The implementation remains a labelled draft until Tom accepts the exact included/excluded signature at its reviewed commit. Do not retain a deprecated complete-`Vec` alias in this pre-production tree.

## Required negative controls

Perturb independently: provider count, proposal count, decline count, ignored sink refusal, installation order, request-subject field encoding, budget-resource enumeration, and the first-over-limit demand. Each assertion remains unchanged and must fail on its production subject.

## Unsupported guarantee

The provider is trusted in-process native code. This channel bounds compiler-owned accepted outcomes and subsequent verification; it does not sandbox a provider that loops, allocates before emission, or constructs an oversized raw proposal. File isolation or bounded proposal-construction work separately if a real untrusted-provider requirement appears.

## Closes when

The complete compile path uses the bounded channel, calibrated limits are canonical, overflow is an atomic typed refusal, full provider/public-boundary tests and identity censuses pass, and an independent exact-commit review finds no fallback or unbounded retained-output path.

## Preserved draft review — 2026-08-13

Draft `54e272baa525027a6f6f9d982bd3bd7c387597fb` is preserved on `tkt/replace-provider-offer-with-a-host-bounded-frontier-sink`; it is not decision-ready or mergeable. Its first audit named stale base `403db7d`; the exact base is `f3e1efd3b3b4f896976b326e6a3d993147206cd3`. The intervening commits do not change the audited compiler sources, so the Fact verdicts survive, but the base record must be repaired.

Independent full-diff review found these blockers:

1. **High — request exhaustion is downgraded to a target/candidate outcome.** `compile_candidate_target` matches every `CompileError::BudgetExhausted` beside `NoFeasiblePlan`. A `PhysicalFrontierOutcomes` stop can therefore continue to another numerical contract or target and return an earlier compiled slot plus a later rejection. The request-scoped budget must terminate the whole compilation with no partial product or retry. Add independent multi-candidate and multi-target negatives.
2. **High — overflow precedence depends on provider installation order.** `enumerate_frontier_with_outcomes` emits and immediately verifies one provider before asking later providers. A malformed proposal first can return `InvalidCompilerOutput`; the same proposal after a flooding provider returns `BudgetExhausted`. Finish bounded emission for the complete provider population before verifying any accepted proposal so the same set/outputs has one result.
3. **Medium — `u64::MAX` is not bounded.** `FrontierOutcomeBudget::charge` uses `saturating_add(1)`, so once both accepted and limit are `u64::MAX`, every further insertion is accepted. Narrow/reject that policy or represent excess separately and test the boundary.
4. **Low — public documentation still counts four authorities.** `BudgetResource` documentation says `Four authorities`, `rather than four`, and `all three stop records` after adding the frontier authority. Repair the complete vocabulary prose.
5. **Medium — retained provider spikes no longer compile.** All three path-dependent consumers still import `ProviderOffer` and implement the old one-argument `propose`: `spikes/program-planning/physical-frontier-budget-calibration`, `spikes/extensions/forkless-physical-provider`, and `spikes/runtime/backend-provider-portfolio`. Each fails with unresolved import `ProviderOffer` and trait-arity `E0050`. The ticket now owns `research/program-planning`, `research/extensions`, and `research/runtime`; migrate the retained spikes to the exact accepted sink API and run their documented checks before presentation.

The existing order perturbation uses homogeneous decline floods and does not reach blocker 2. The existing overflow integration test uses one target and overflows immediately, so it does not reach blocker 1. Perturb the heterogeneous provider order and a request where an earlier target/candidate succeeds before the shared counter exhausts.

Blocker 1 is reproduced through the public API on this draft: the ordinary five-operation program with governed providers only and 16 distinct valid target profiles returns targets 0 through 12 as `compiled` and targets 13 through 15 as `BudgetExhausted { PhysicalFrontierOutcomes, 256, 257 }`. The temporary harness lived outside the repository; carry this exact subject into the retained calibration spike and the integration suite so the negative is durable.

The branch-local acceptance packet is also incomplete. It omits exact sink method argument/return signatures and the refusal-token traits, gives no per-exclusion reasons despite claiming them, and compares the sink only with a one-sentence pull counterpoint. Rebuild the packet through the full decision-readiness gate after the implementation and spike migrations are complete; do not patch those omissions into a packet that still carries the uncalibrated value.

The claim was released because its calibration dependency was reopened. Resume only after [`calibrate-the-physical-frontier-provider-and-outcome-budgets`](calibrate-the-physical-frontier-provider-and-outcome-budgets.md) supplies a reviewed full-request authority/value. Then rebase the preserved draft, repair every finding above, run exact-commit independent review, and only then update the public-boundary packet for Tom.

## Released to ready — 2026-08-22, its stated resume condition fired

This ticket says **"Resume only after `calibrate-the-physical-frontier-provider-and-outcome-budgets` supplies a reviewed value, then rebase."** That ticket is now **`done`**, and so is the other dependency, `decide-whether-the-implementation-frontier-owes-a-retention-budget` — both verified by the coordinator at `925fdfd8`. The stated condition has fired, so the status moves from `blocked` to `todo`.

**Read this before starting: a preserved draft exists and must not be merged.** Commit `54e272baa525027a6f6f9d982bd3bd7c387597fb` sits on `tkt/replace-provider-offer-with-a-host-bounded-frontier-sink` — 1,031 insertions across 17 files, including a branch-only `tickets/accept-the-host-bounded-physical-frontier-sink.md` that does not exist on `main`. This ticket's own text records it as *"not decision-ready or mergeable"*, and a branch audit confirmed it is **612 commits behind**. Treat it as evidence to read, not a base to build on: the review that parked it named five blockers, one of which is that three retained spikes no longer compile against the revised API. **Rebase or rewrite deliberately; do not merge it, and do not delete the branch.**

**What the calibration actually settled, and what it did not.** It landed the **raw-outcome** axis only. Its own stop condition fired on the **provider-count** axis, which has no accepted value and a different enforcement point — `InstalledPhysicalProviders::installed` runs *before* compile and has no count branch, so a per-request budget field could never enforce it. That axis is [`calibrate-the-physical-provider-count-at-the-installation-seam`](calibrate-the-physical-provider-count-at-the-installation-seam.md), still open. **Do not assume a provider-count limit exists.**

**Scheduling.** Collides with the live gather-vertical lane on `implementation/compiler` and `contracts/decisions`. **Release trigger: that lane merges or stops at a gated boundary.**
