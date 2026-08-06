---
id: derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold
title: Derive the rewrite-rule declaration and admission shape for the online-softmax fold
status: in-progress
priority: p2
dependencies: []
related: [reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller, decide-whether-to-admit-an-elementary-identity-permission, expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate, derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound, connect-certified-rounding-error-bounds-to-rewrite-permissions, name-the-elementary-identity-rewrite-dimension, derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, optimizer, rewrites]
claimed_from: todo
assignee: agent-rule-object
lease_expires_at: 1785993439
---
## User-visible outcome

The one *rule* object the two accepted numerical decisions name as the thing they are waiting for: a derivation of how an operation declares the functional equation it satisfies, how a rewrite rule declares the dimensions it consumes and carries its own parametric bound, and how the six admission obligations are discharged against a complete scheduled candidate — worked end to end on the online-softmax rescaling fold. **It admits no permission, proposes no ADR, and presumes neither accepted decision's outcome.** Under a continued decline it is what makes the refusal checkable rather than described; under an admission it is the object that would consume the permissions. Both readings are served by the same deliverable, which is why doing this work presumes nothing.

## Why this exists, and why it is the ownerless one

**Fact — ADR 0095's 2026-08-06 reaffirmation added a second reopening condition and spelled its prerequisites concretely.** The two permissions "are considered *together* when a rewrite rule with a derived bound instantiable at a schedulable fold shape is ready to consume them", and "ready to consume" is stated as three things: **a rule in the certified-bounds admission shape**, a retrievable `eps_exp` to instantiate its bound, and the bound derived at the fold shape a parallel schedule would select. [ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) item 5 names the same reassessment as its own first trigger clause, so the two decisions move together or not at all.

**Fact — two of the three prerequisites have owners and one does not.** [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md) is `todo`; [`derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound`](derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound.md) is live. The first prerequisite — the rule itself — has no owner anywhere in the graph, verified by [the flash-class capability record](../docs/research/program-planning/flash-class-capability-set.md)'s axis 1.

**Fact — the shape is derived and the declaration layer is not.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) Part 3 fixes the admission rule completely: a per-rule closed-form bound parametric in shape and target facts, instantiated by exact-rational arithmetic, compared against a caller-stated tolerance, answering `Admit` / `Refuse` / `Undecided` with `Undecided` fail-closed, sitting as a feasibility answer at semantic exploration in the same position as `require_elementary_accuracy`, and never as a cost. Five obligations are stated there and ADR 0101 decision 7 adds a sixth. What has no shape at all is layer 1: `OperationAlgebraicCapabilities` (`crates/tiler-ir/src/semantic/operation.rs:922`) is a struct with one `bool` field, `ordered_associativity`, and a functional-equation capability would be the vocabulary's first **parameterized** law.

**Fact — the three-way decision machinery already exists one layer down.** `ConformanceDecision` (`crates/tiler-reference/src/accuracy.rs:735`) is `Conforms` / `Violates` / `Undecided { reason }` and `decide_predicate` (`:772`) compares an `ExactRational` candidate against a `CertifiedEnclosure`. The rule should reuse it rather than grow a parallel one, and stating why it can or cannot is part of this ticket.

## What this ticket must produce

- **The capability spelling for a declared functional equation**, with its real-domain side condition, derived rather than sketched: what a parameterized law costs the operation-identity encoding, whether it is appends-only at each encoding site by per-tag injectivity reasoning, and what refuses a malformed or unsupported equation. State the identity-domain consequences without executing any of them.
- **The rule object**, worked on R1 (shifted-max rescaling) and R2 (the online normalizer fold) of the flash-class record's five-rule table: which dimensions each declares, what its bound is parametric in, and where the bound's side conditions come from.
- **Each of the six admission obligations discharged or refused explicitly** for the worked rule, including ADR 0101 decision 7's sixth — that the rewritten program's elementary arguments are proved to lie inside every accuracy clause's declared domain. The elementary-identity record's Part 4 already gives that discharge for the online form (the running maximum is non-decreasing and includes `x_j`, so both argument families stay non-positive); restate it as an obligation the rule performs, not as an observation a reader makes.
- **The refusal, emitted rather than described.** ADR 0101 decision 6 requires a rejection to name every missing dimension, the function, and the identity; the elementary-identity record's Part 7 quotes the exact wording as a checkable block. Derive where that string is produced and what carries it into the explain vocabulary, whose `ExplainDisposition::RejectedNumerical` already exists (`crates/tiler-compiler/src/explain.rs:122`).
- **A statement of what remains missing after this ticket**, so the reopening condition's readiness is a checkable fact rather than a judgement call.

## Non-goals

Admitting either permission; proposing an ADR; editing `docs/numerical-semantics.md` or any `docs/decisions/` record; implementing anything under `crates/`; stepping any identity domain; deriving the tree-fold bound (its own ticket owns it); deriving R4's output-accumulator rewrite, whose schedule half is a separate deferral.

## Closes when

The three layers ADR 0101 decision 3 names are each spelled with their obligations and their refusals, the six admission obligations are each discharged or explicitly refused for the worked rule, the refusal wording has a derived production site, and the record states which of ADR 0095's three reopening prerequisites remain open — with the answer traceable to a source read rather than to this ticket's own assertion.
