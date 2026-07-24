---
id: revisit-kernel-body-single-spelling-gate
title: Revisit the single-spelling kernel body refinement gate when the profile widens
status: todo
priority: p2
dependencies: []
related: [prototype-structured-kir-slice, own-operation-family-support-matrix]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, verification]
---
The kernel verifier's final check is **derive-and-compare**: after the specific
rules run (so diagnostics stay precise), it re-derives the canonical body from
the scheduled region and requires structural equality. A kernel that is
semantically equivalent but *differently spelled* is therefore rejected as
`BodyRefinement`. This is deliberate and fail-closed — it never accepts a body on
an unproven equivalence — and the alternative, a symbolic index normalizer, was
designed and rejected as materially more machinery for the same guarantee.

The consequence, which is fine today and will not stay fine: the bounded profile
admits **exactly one spelling** of a legal kernel. That holds because the profile
has one canonical form per scheduled region. It stops holding as soon as the
operation and schedule surface widens enough that two genuinely different
spellings are both legal for one region — at which point derive-and-compare
starts rejecting *valid* kernels, and an external producer cannot supply its own
legal body at all.

**Trigger for reconsideration:** the first time a widened profile admits more
than one legal body for a single scheduled region, or the first time an external
producer needs its own spelling accepted. Both are foreseeable consequences of
the operation-family breadth work, so this ticket is related to that owner.

When that happens, the replacement must still prove equivalence rather than
assume it — either a normalizer with its own correctness argument, or a checked
equivalence relation with stated soundness. **Do not weaken this gate before the
trigger**, and specifically do not replace structural equality with a
looser structural heuristic that admits more bodies without proving they mean the
same thing; a fail-closed rejection of a valid kernel is recoverable, an accepted
wrong kernel is not.
