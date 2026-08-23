---
id: decide-whether-the-refinement-subject-identity-should-carry-its-environment
title: Decide whether the refinement subject identity should carry its environment
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [identity, indexing, decision]
claimed_from: todo
assignee: worker-envid
lease_expires_at: 1787467109
---
## User-visible outcome

Either the refinement subject identity carries the environment a symbolic extent resolves against, or the record states why it deliberately does not — so two subjects that differ only in environment are known to be the same subject or different ones, rather than the question going unasked.

## Why this exists

Filed 2026-08-22 by the coordinator from the sibling sweep of [`destructure-the-gather-bounds-subject-in-its-identity-encoder`](destructure-the-gather-bounds-subject-in-its-identity-encoder.md), which landed as `f197697f`. That lane found it while checking whether a sub-agent's census was right — it was not, and correcting it surfaced this. The lane declined it as a question about *what the identity should encode*, which its non-goals excluded. That judgement was right and is why this is a separate ticket.

**Fact — verified by the coordinator at `3291b105`.** `IndexRefinementSubject` in `crates/tiler-ir/src/index/refinement/subject.rs` declares **fourteen** fields. Thirteen carry `pub(super)`; the fourteenth, `environment: SubjectEnvironment`, is private to the module. `grep -n "environment" crates/tiler-ir/src/index/refinement/identity.rs` returns **nothing** — the subject identity encoder never reads it.

**Fact — two of the fourteen are outside the bytes for different and unequal reasons.** `identity: Box<[u8]>` is correctly self-excluded, because it caches this encoder's own output; including it would be circular. `environment` has no such justification recorded. So the population genuinely in question is **one field, not two** — a distinction worth stating, because a census that lumps them reports a defect twice as large as the one that exists.

**Inference — the tree points both ways, which is what makes this a decision rather than a repair.** `encode_region` in `crates/tiler-ir/src/index/builder/identity.rs` deliberately folds environment identity **in**, and says so: *"The environment a symbolic extent resolves against is part of what this region is."* The refinement subject encoder does the opposite. One of the two is wrong, or the two layers have a stated reason to differ that nobody has written down.

**Fact — `SubjectEnvironment`'s own doc says it is "compared by identity only."** So an identity for it exists or is derivable; the question is not whether the environment *can* be encoded but whether this subject's identity *should* fold it in. Read that comparison path before deciding — it may already answer the question, or may be the thing that has to change.

## Required work

- Re-audit every Fact above at your base with a per-Fact verdict.
- **Decide by reading**, and follow the readiness gate in `AGENTS.md`: enumerate the materially distinct options — fold the environment into the subject identity, record why it is deliberately excluded, or narrow what `SubjectEnvironment` carries so the question dissolves — and eliminate anything that could conflate two subjects that are not the same.
- Determine whether two subjects differing **only** in environment can be constructed today. If they can, that is the decisive evidence and it should be constructed and watched. If they cannot, say what prevents it and whether that wall is a record the program carries or a property of the current frontier — the difference decides whether an exclusion is safe or merely currently unreachable.
- **If the answer is to fold it in, that steps an identity domain.** Stop and report rather than landing it inside this ticket: an identity step needs its own coherent change across the owning version, ledgers, and pins, and this ticket is scoped to the decision.
- If the answer is a documented exclusion, state the reconsideration trigger and pin the reasoning where the next reader will hit it.

## Non-goals

The mechanical destructure sweep, which is [`destructure-the-framed-records-in-the-index-region-identity-encoders`](destructure-the-framed-records-in-the-index-region-identity-encoders.md). Changing `encode_region`'s treatment of environment, which is settled and reasoned. Any public surface change.

## Closes when

The refinement subject identity either carries the environment or records why it does not with a reconsideration trigger, the constructibility of an environment-only difference is settled by reading or construction, and any identity step is split into its own ticket rather than landed here.
