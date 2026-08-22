---
id: admit-a-refutation-only-derived-bound-conformance-oracle
title: Admit a refutation-only derived-bound conformance oracle
status: deferred
priority: p3
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, connect-certified-rounding-error-bounds-to-rewrite-permissions, measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, conformance]
---
## User-visible outcome

A conformance check that can prove a candidate wrong under every legal realization, for the candidates the declared-order oracle can only refuse to judge — and that is typed so it can never be read as an admission.

## Why this is a deferral rather than work

**Fact — [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) eliminated a derived-bound interval as an admission authority and retained it in one strictly weaker role.** A candidate outside a sound worst-case interval is producible by no legal realization, so `Violates` is sound; a candidate inside it has satisfied a bound every legal realization also satisfies, and the interval is a strict superset of the permitted set. Its Part 6 exhibits the failure with exact bits: at four contributors the `gamma_3` interval admits four representable binary32 values where the contract permits two, and one of the two it wrongly admits is what a kernel that dropped two of its four contributors returns.

**Inference — so the retained object has a structurally unreachable `Conforms` arm**, and typing it that way is the point rather than a detail. AGENTS.md requires every check to be proved able to say no; the converse binds here, because a `Conforms` this oracle can technically return is one some caller eventually reads as an admission.

**Inference — it has no caller.** It is useful only for candidates the declared-order oracle refuses, which are exactly the candidates whose executed order is not pinnable. Building a refutation path for a population that does not exist is the caller-less admission this repository declines elsewhere.

## Trigger

A candidate class Tiler emits whose executed evaluation order is not pinnable and which must be qualified rather than refused. The identified route is [`measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order`](measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order.md) returning that the backend compiler does not preserve the emitted order under a permissive contract, *and* a decision that such a candidate is emitted anyway rather than refused.

## What this ticket must produce once fired

- The bound instantiated from the shape, the format, and the target's declared facts in exact rational arithmetic, never a host float, with the five admission obligations [the certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) states each discharged rather than inherited.
- A decision type whose `Conforms` arm is unrepresentable rather than merely unreached.
- A case outside the interval, watched being refused, and a case inside it, watched returning `Undecided` rather than a pass.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md), which retained this object rather than eliminating it outright.

## Trigger check log

- 2026-08-05 — **not fired, on both clauses.** No measurement of backend order preservation exists (`measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order` is `todo` and unstarted), and no decision to emit an unpinnable candidate has been taken. Reproduce the first clause with `tkt show measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order` — a status other than `done` is the not-fired verdict.
- 2026-08-06 — **not fired: the first clause fired and the second did not.** Finding 34 (the Apple record) measures the backend compiler re-serializing an emitted two-by-two split under `relaxed` and `fast` on both compilation paths, which is exactly the NotPreserved answer the first clause names. No decision to emit an unpinnable candidate has been taken — such candidates are still refused, and no workload is asking for qualification — so the caller this oracle needs still does not exist. Reproduce clause 1 with `rg -n '### 34\.|does not survive' docs/research/apple-targets/numerical-behaviour.md` (token `NotPreserved` is in the measure ticket Outcome, not that file); clause 2 with the absence of any decision record admitting an unpinnable candidate.
- 2026-08-09 — **not fired on the remaining clause.** The NotPreserved measurement remains current, but no accepted decision emits a candidate whose evaluation order is unpinnable and no workload asks to qualify one rather than refuse it. The oracle still has no candidate class to evaluate.
