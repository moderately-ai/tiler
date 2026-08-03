---
id: own-or-close-the-adr-internal-open-questions
title: Own or close the ADR-internal open questions that assign nobody
status: done
priority: p3
dependencies: []
related: [re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal, resolve-or-retire-the-scalar-lowering-provider-seam, accept-the-public-backend-provider-composition-boundary, derive-the-multi-round-two-level-reduction-composition, reconsider-registered-quantitative-capability-axis-schemas]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, graph-repair]
claimed_from: todo
assignee: agent-adr-questions
lease_expires_at: 1785786696
---
## User-visible outcome

An open question living inside an accepted ADR has an owner, a stated trigger, or a durable answer — the same standard `docs/open-questions.md` is held to — so "No owner is assigned" stops being a durable state.

## Why this exists

**Fact — three questions state it verbatim.**

- [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md):143 — whether the mature per-operation fusion numerical capability is a registered third-party capability or a projection of the semantic definition. It records the real tension (an extension author knows whether their operation is an ordered reduction; a provider *asserting* a numerical permission is a claim the host cannot verify, the class ADR 0021 requires proof or runtime validation for) and offers a recommendation "offered rather than adopted". Ends: "No owner is assigned."
- ADR 0078:144 — whether `ScalarLoweringProvider` should reach the compile path at all. Ends: "No owner is assigned."
- [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):150 — whether the closed quantitative capability-axis set should become extensible. Records that item 1's elimination of pure data "rests partly on the axis set being closed", and that the CPU vertical's missing vector width, mask and tail support, scalable-vector length, cache levels, and thread count are "not merely undeclared but *inexpressible*". Ends: "No owner is assigned", and notes it depends on item 1's answer.

**Inference — the sweep that would catch these is scoped elsewhere.** [`re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`](re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal.md) audits `docs/open-questions.md` under `contracts/navigation` and cannot reach `docs/decisions/`. An ADR-internal question is subject to the same rule AGENTS.md states — remove an open question only when its answer lives in a durable contract or an accepted ADR — and nothing applies it there.

## What each needs, stated so the ticket is writable

- **ADR 0078:144 is discharged by a live ticket.** [`resolve-or-retire-the-scalar-lowering-provider-seam`](resolve-or-retire-the-scalar-lowering-provider-seam.md) runs exactly that elimination and must state which candidate survived. Point the question at it; do not answer it here.
- **ADR 0078:143 needs an owner or a trigger.** The recommendation is written but unadopted, so the honest outcomes are a ticket that adopts or refutes it, or a stated trigger — plausibly the first registered third-party capability claiming a fusion role. Do not adopt a recommendation by moving it out of the open-questions section; that is acceptance by relocation.
- **ADR 0090:150 is Tom's and gated.** It "depends on item 1's answer", and it subsumes the fourth seam ADR 0090:125 names under item 14 — whether a target triple, ABI, and data layout become `CapabilityAxis` values, which today "survive only inside a profile key string and payload provenance". Surface it as a decision with its dependency stated; **do not answer it**, and do not file implementation work behind it. It is a Tom decision, not fileable implementation.

## Boundaries

- Scope is `contracts/decisions`, plus `contracts/navigation` shared if a question is promoted into `docs/open-questions.md`. No code.
- Sweep the accepted ADRs for the same pattern rather than fixing only these three — find one bug, check all siblings. Reproduce with `grep -rn "No owner is assigned" docs/decisions/` and read each hit in full, since a question can be unowned without using that phrase.
- Closing a question requires its answer in a durable contract or an accepted ADR. A recommendation inside the question is not an answer.

## Closes when

`grep -rn "No owner is assigned" docs/decisions/` returns nothing, or returns only questions carrying a stated trigger a reader can evaluate; each of the three above has an owner, a trigger, or a durable answer; and ADR 0090:150 is recorded as Tom's with its gating dependency named rather than resolved.

## Audit outcome (2026-08-03)

**Population.** The tree contains 95 accepted ADRs. Eleven carry an explicit `## Open questions` section. The audit read those eleven sections in full, read ADRs 0078 and 0090 in full, and inspected every accepted ADR's question/deferral/trigger/owner language rather than treating the three literal `No owner is assigned` hits as the population.

**Corrections.** ADR 0078's fusion-capability question now has two evaluable evidence triggers and its scalar-lowering question points at the live elimination ticket. The sibling sweep also closed two stale terminal-owner references into durable answers (ADR 0074's descriptor style and ADR 0094's two-level representation), re-pointed ADR 0091's completed BF16 measurement to its live contract owner, supplied explicit triggers to the otherwise ownerless convention, boundary-policy, and numerical-honourability questions in ADRs 0074–0076, and gave ADR 0096's already-fired multi-round trigger the bounded owner [`derive-the-multi-round-two-level-reduction-composition`](derive-the-multi-round-two-level-reduction-composition.md). None of those corrections changes an accepted decision.

## Remaining atomic decision for Tom — extensible quantitative axes

**Dependency, now satisfied.** ADR 0090 item 1 had to decide whether a target profile carries target facts or provider choices before an extension boundary could be evaluated. Tom accepted the checked split on 2026-07-31: profiles declare facts, providers propose choices, and the host performs every comparison. This question is therefore ripe; no implementation is filed behind either answer.

**Concrete case.** A future CPU profile needs to state a 64 KiB cache-capacity fact and a provider wants to require at least 32 KiB for one tile. The current closed `CapabilityAxis` vocabulary cannot state that row. [ADR 0093](../docs/decisions/0093-bind-vector-lanes-to-the-map-or-the-contributor-partition.md) has already shown why vector width is not the example: width, masks, and tails form one exact atomic realization subject rather than an `AtMost` quantity. The remaining question is whether a genuinely quantitative row such as cache capacity becomes a compiler-owned enum case and builder method, or whether a target can install a governed axis definition that the host validates and compares generically.

**Elimination.** An arbitrary provider-defined key plus opaque comparison callback fails correctness and deterministic identity: the party proposing work could also define what makes it feasible, and two processes could rank the same request under unrecorded code. That candidate is discarded. Two candidates survive:

1. **Keep the set closed and compiler-owned.** Add each admitted quantitative fact as an exhaustive typed axis with a host-owned comparison. This enables the strongest total maps, compiler-enforced review of every consumer, and the smallest identity surface. It prevents a genuinely new quantitative fact from being added by an out-of-tree backend without a compiler change.
2. **Admit registered, host-validated axis schemas.** A frozen per-request registry binds a governed axis key to a host-known quantity, relation, validation, and identity encoding; profiles and requirements may use only registered schemas, and the host still performs every comparison. This enables forkless quantitative target facts while preventing opaque callbacks or provider-authored feasibility verdicts. It costs a new public registration and schema boundary before a second independent backend has shown which parts genuinely generalize.

**Point and counterpoint.** The closed set makes every fact addition compiler work, which sits awkwardly beside the accepted forkless provider model. The registered set removes that fork, but it asks Tiler to stabilize an extension protocol from one bounded CPU vertical whose missing rows do not yet distinguish a general schema from a handful of correct compiler-owned variants. Both can fail closed and preserve the item-1 authority split; the choice is about when to pay for generality and who may extend the fact vocabulary.

**Recommendation.** Keep the quantitative axis set closed for the initial profile and add only a measured quantitative row as a compiler-owned typed variant when a workload actually requires it. Reopen at the first independently authored target profile blocked by a quantitative fact the compiler does not name, or when a second backend's row demonstrates a schema shared with the CPU row. This follows the repository rule that a type-system reservation, an architectural seam, and implemented support are separate maturity claims, while leaving the accepted physical-provider seam fully usable for schedule choices today.

**Question.** Should quantitative capability axes remain compiler-owned and exhaustive for the initial profile, as recommended, or should the first CPU rows establish a registered host-validated axis-schema extension boundary now?

## Integration review — 2026-08-03

Independent fixed-commit review verified the 95-ADR population, all eleven
explicit open-question sections, every sampled owner and trigger, the new
multi-round ticket, and the atomic quantitative-axis packet. It found and the
worker corrected stale current-tense summaries in ADRs 0078 and 0096; a
Decision-span comparison then confirmed that no accepted Decision clause moved.
The corrected endpoint `d6a7657c0557ef4c2401bd73a835517953f77ea0`
passed ticket lint, diff, link, and true-base scope checks before merge commit
`bfe16195c234fd2cc56f4fa60ca01f35af18e2b9` integrated it.

## Tom's quantitative-axis decision — 2026-08-03

Tom accepted the recommendation in the T3 Code orchestration conversation:
quantitative capability axes remain compiler-owned and exhaustive for the
initial profile. ADR 0090 now records the durable answer. The rejected
registered-schema alternative is retained by the explicitly deferred
[`reconsider-registered-quantitative-capability-axis-schemas`](reconsider-registered-quantitative-capability-axis-schemas.md)
ticket with the two evidence triggers from this decision packet. It is related
rather than a dependency, so an un-fired reconsideration cannot park current
target-profile work.
