---
id: accept-adr-0107-indirect-gather-semantic-family
title: Accept or revise ADR 0107 on admitting an indirect gather above the index language
status: done
priority: p1
dependencies: []
related: [admit-an-indirect-gather-family-for-tied-embedding-lookup, admit-the-indirect-access-class-into-the-index-layer, emit-the-indirect-gather-on-metal]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, indexing, semantics, gather, needs-tom]
---
**This ticket is Tom's decision, not an agent's work item.** It exists so the two follow-on tickets have something to depend on rather than being schedulable while the record they extend is still `proposed`.

`docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md` is `decision_status: proposed`, `implementation_status: partial`. The family it describes is registered and reference-evaluated; nothing below the semantic layer changed.

## What the record decides

An indirect gather is admitted as a semantic operation family and as nothing below it. `tiler::gather-f32@1` takes a `tiler::f32@1` source and a `tiler::u32@1` index operand, carries one gathered axis as a typed attribute, and composes the index operand's shape into the position that axis occupied. Bounds are a semantic precondition discharged at a named enforcement boundary, never clamped and never wrapped. Duplicate indices are admitted; the duplicate-write rule is stated and unimplemented so scatter stays additive. A signed index operand is refused by name so the negative-indexing convention is not answered silently.

## The one decision inside it that is not the family

**The index-expression vocabulary is deliberately unchanged, and that is the record's substance rather than a deferral inside it.** ADR 0046 states that "indirect operations remain addable without weakening the verifier for the initial direct-access language", and separately that data-dependent gather "requires later explicit IR contracts". ADR 0107 supplies that contract for the read half while satisfying the non-weakening condition the only way available: by admitting no expression form at all. `AccessData` still carries one tensor ordinal, `IndexNode` still has no variant reading tensor data, and an occurrence therefore reaches no index region, resolves no lowering capability, takes no fusion role, and fails closed at the request boundary.

Accepting this record accepts that a *registered, reference-evaluated, unplannable* family is a legitimate delivered state — not a half-landing to be finished. Rejecting it means either the family should not be registered until the index layer can express it, or the index layer should be widened in the same breath.

## What acceptance does not commit to

Acceptance is not acceptance of the public boundary. Under ADR 0075 the key, the gathered-axis attribute, `GatherAxis`, `GatherError`, and `gather_result_shape` are a **labelled draft** until the exact included and excluded surface is separately accepted. Nor does acceptance decide the index-layer question: `admit-the-indirect-access-class-into-the-index-layer` holds that and is a decision ticket in its own right.

## What closes this ticket

Either set `decision_status: accepted` with an acceptance date and unblock the two follow-ons, or record the requested revisions here and send the record back. If accepted with modifications, amend the record rather than superseding it — it has never been operative.

## Accepted — Tom, 2026-08-07

Accepted in the interactive orchestration session, as a direct answer to the decision presented with its trade-off and counterpoint. Not relayed through any intermediary.

`docs/decisions/0107-…md` moved to `decision_status: accepted` with the acceptance provenance, the exact accepted extent, what acceptance did **not** commit to (the public boundary stays a labelled draft under ADR 0075; the index-layer question stays its own decision), and the counterpoint recorded alongside it — that a registered-but-unplannable family is a trap for a reader who takes registration to imply reachability, accepted on the strength of the boundary being tested rather than asserted.

`admit-the-indirect-access-class-into-the-index-layer` is unblocked and dependency-ready. `emit-the-indirect-gather-on-metal` stays `blocked` on *that* ticket, which is correct — it is a second decision, not a consequence of this one.
