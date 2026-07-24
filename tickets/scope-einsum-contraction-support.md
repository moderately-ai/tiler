---
id: scope-einsum-contraction-support
title: Scope einsum and tensor-contraction support (Milestone 6)
status: in-progress
priority: p1
dependencies: []
related: [own-operation-family-support-matrix]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, breadth, einsum]
claimed_from: todo
assignee: agent-scope-einsum-contraction-support
lease_expires_at: 1784919376
---
Milestone 6 (einsum contractions) has zero tickets and zero open questions. Every
"contraction" currently in the corpus is the FMA numerical-permission sense
(whether multiply-add may fuse into one rounding), never tensor contraction. For a
"DataFusion for tensor compute", general tensor contraction — matmul, batched
matmul, and einsum — is the single most conspicuous missing operation family, and
it is invisible in both the work graph and the durable question index.

Add an owning `docs/open-questions.md` entry (and, if warranted, a `docs/roadmap.md`
Milestone 6 note) that frames the tensor-contraction / einsum question with an
explicit reconsideration trigger: what identity, validation, access-relation, and
lowering consequences a contraction operation family imposes, and what must be true
(a generic compile path, a working backend, the optimizer conformance gate) before
it can be scheduled. This is a deferred question with a trigger, not an
implementation and not a commitment to a specific einsum surface.

Coordinates with `own-operation-family-support-matrix`, which references this as
the contraction line of the broader operation-family matrix.
