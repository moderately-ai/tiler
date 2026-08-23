---
id: decide-what-prepare-access-does-with-an-unenumerated-access-mode-and-role-pair
title: Decide what prepare_access does with an unenumerated access-mode and role pair
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing]
claimed_from: todo
assignee: worker-prepaccess
lease_expires_at: 1787486699
---
## User-visible outcome

`prepare_access` states what it does with every access-mode and tensor-role pair, so a widened `TensorRole` is a decision someone makes rather than a case silently admitted.

## Why this exists

Filed 2026-08-23 by the coordinator from `worker-drafttrav`'s stop condition on [`make-the-draft-time-index-traversals-outside-compact-rs-exhaustive`](make-the-draft-time-index-traversals-outside-compact-rs-exhaustive.md), which landed as `192153a2` and took every non-test rest pattern in the index builder to zero. That lane reported this site and **declined it deliberately**, because repairing it is not mechanical — which is exactly why it is its own ticket rather than a line in that sweep.

**Fact — reported by that lane, NOT verified by the coordinator.** `crates/tiler-ir/src/index/builder.rs` carries `_ => {}` in `prepare_access`, matching over a `(AccessMode, TensorRole)` pair. Re-derive it at your base before acting; it is a worker report and secondhand.

**Why it is not the same as the sweep's other sites.** Every arm that sweep repaired was a record walk where binding the elided fields to `_` changed nothing and prevented a silent miss. This one is different: the wildcard is over a **pair of enums**, and a new `TensorRole` reaching `prepare_access` would be *silently admitted* rather than merely unvisited. Making it exhaustive therefore means **deciding what a hypothetical new role should do** — admit, refuse, or refuse by name — and that is a policy question about a vocabulary that does not exist yet.

**Inference — so the honest outcomes are not only "make it exhaustive".** Enumerating the current pairs with an explicit arm per role is one. Recording why a wildcard is correct here, with a reconsideration trigger tied to `TensorRole` gaining a variant, is another and may be the better one. A third is a typed refusal for the unenumerated case, which is fail-closed but adds a diagnostic nobody can currently reach — and this repository has repeatedly found that a check whose failing case is unreachable is worse than no check, because it reads as covered.

## Required work

- Re-audit the Fact at your base with a verdict, and read `prepare_access` in full before deciding anything.
- Establish whether a `TensorRole` variant can be added without touching this function — that is the question the whole ticket turns on. `core::mem::variant_count` over `TensorRole`, or an exhaustive match elsewhere that would already break, may settle it.
- **Decide by reading between the three outcomes above.** If a wildcard is genuinely correct, record why at the site with a reconsideration trigger; that is a valid close and better than an enumeration that guesses at admission policy.
- If you enumerate, **state what each new arm does and why** — an arm that silently mirrors the old wildcard has changed nothing except to make the next reader think a decision was taken.
- **Before trusting any refusal you add, state what it would take for it to fire and confirm that case is reachable.** If it is not reachable, say so rather than landing an unreachable diagnostic.
- State whether any identity value moves. Expected: none. Rederive rather than assume.

## Non-goals

The record walks already made exhaustive by `192153a2` and `f5f4cff1`. The identity encoders, done by `a0659d05`. Adding a `TensorRole` variant. Changing what `prepare_access` does for any pair that exists today.

## Closes when

`prepare_access` either enumerates its pairs with a stated reason per arm, or records why a wildcard is correct there with a reconsideration trigger tied to the vocabulary growing, no identity value has moved, and any refusal added has been shown reachable or declared unreachable on purpose.
