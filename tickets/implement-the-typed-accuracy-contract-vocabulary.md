---
id: implement-the-typed-accuracy-contract-vocabulary
title: Implement the typed transcendental accuracy-contract vocabulary
status: in-progress
priority: p1
dependencies: []
related: [admit-the-silu-activation-family, admit-the-rms-normalization-family, admit-the-softmax-family, record-the-metal-elementary-function-accuracy-guarantee, numerical-policy-contract, own-operation-family-support-matrix, scope-transformer-nonlinear-normalization-and-reductions]
scopes: [implementation/ir, implementation/reference, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, numerics, transcendental, accuracy, boundary]
claimed_from: todo
assignee: worker-accuracy
lease_expires_at: 1785557885
---
## User-visible outcome

An operation definition can *carry* a resolved accuracy contract, that contract reaches canonical identity, and the reference layer can decide whether a candidate result satisfies it. Until this exists no transcendental key may be registered at all, so this is the gate under all three L3′ verticals rather than a refinement of them.

## Why this is a separate ticket

**Fact — ADRs 0016 and 0042 are accepted and their carrier does not exist.** Both are `implementation_status: not-started`. Exact check, run from the repository root, currently returning nothing: `grep -rn --include='*.rs' -E 'ulp-reference-gap|UlpMetric|AccuracyContract|CorrectlyRounded|NamedElementaryProfile' crates/` and `grep -rln --include='*.rs' -i transcendental crates/`. What *does* exist is the permission dimension that presupposes the carrier: `ApproximationEnvelope::Forbidden` in `crates/tiler-ir/src/schedule/numerics.rs` is documented as "approximate intrinsics are forbidden; every elementary function follows its own resolved accuracy contract", and there is no such contract for it to defer to.

**Fact — Milestone 1 makes this a precondition of registration, not of execution.** The roadmap requires Tiler to "canonically serialize and reference-evaluate every enabled transcendental accuracy contract before admitting such an operation to the vertical slice", and the [support matrix](../docs/roadmap.md#operation-family-support-matrix) transcendentals row repeats it as that row's trigger. ADR 0016 adds the structural reason: "transcendental accuracy participates in semantic, plan, artifact, reference, and explain identity". A key registered without its contract would not be a partial identity, it would be a *wrong* one, and adding the contract later would have to change it.

**Inference — and the vocabulary is not any one family's.** ADR 0042's contract algebra is profile-wide public surface owned by [Numerical semantics](../docs/numerical-semantics.md). Designing it inside a ticket scoped to one activation would let one call site fix a boundary that `Exp` (SiLU and softmax), `Rsqrt` (RMS normalization), and every later elementary function must share, and [Q-SEM-004](../docs/open-questions.md#q-sem-004--first-profile-transcendental-tuples) selects the first *tuples* rather than the first tuple.

## Required delivery

Each bullet is ADR 0042 text, not an invention; the ADR is the specification and this ticket is its implementation.

- **The four discriminated contract forms** — correctly rounded, faithful, bounded piecewise, and named elementary behaviour with an immutable descriptor digest — kept distinct by construction. "Correctly rounded, faithful, and one-ULP contracts are never equated by name."
- **Exact rational tolerances.** "Every tolerance is a canonical exact nonnegative number, initially an integer or rational, never a host floating-point literal." `num-bigint`, `num-integer`, and `num-traits` are already `tiler-ir` dependencies, so this needs no new dependency.
- **The generic bounded predicates** `Absolute`, `Relative` (with its domain excluding `r = 0`), `AbsoluteRelative`, `Ulp(metric_key, t)`, `AllOf`, and `AnyOf`, with the stated normalization: nested same-kind Booleans flattened, sorted by canonical encoding, deduplicated, bounded in depth and cardinality; empty collections invalid; singletons canonicalizing to their member; and the definedness rule applied recursively so `AnyOf` cannot hide an undefined relative predicate at a zero reference.
- **The `tiler::ulp-reference-gap@1` metric** with its exact representable-value rule (the smaller of the predecessor and successor gaps where they differ), its zero rule (the smallest positive finite representable value, minimum subnormal for a gradual-underflow format), its definedness restriction to finite `r` and `z` with `r` in range, and its explicit *non*-inheritance of OpenCL's hypothetical-successor overflow allowance. The dtype/metric compatibility check must **reject rather than guess** a dtype descriptor that does not expose an ordered set of numerically distinct finite values and adjacent-value behaviour.
- **The accuracy-domain predicate language** over exact input operands and typed reference-result classes, with complete coverage of the operation's admitted ordinary input domain, intersection semantics for overlapping clauses (every matching clause applies — no priority, no order-dependent fallback), and rejection of unverifiable gaps or a possibly empty intersection.
- **The five-step observable-result-set composition** in order: input-subnormal contract, exact reference classification, accuracy-conforming candidate selection or the explicit NaN/infinity/domain/overflow contract, result-subnormal and signed-zero mapping, then NaN canonicalization. Verification "requires the final composed result set to be nonempty for every admitted input and rejects the contract when it cannot establish that fact".
- **Canonical serialization into semantic identity**, covering "the operation and dtype signature, reference semantics, complete accuracy contract, domains, exact bounds, metric versions, and the independent exceptional-value contracts".
- **The refinement relation as a conservative proof relation**: identical normalized contracts, identical reference/domain/metric predicates with tighter exact bounds, normalized Boolean implications the closed algebra can establish, and explicitly registered mathematical implications. "Any other implication requires a certificate accepted by a versioned trusted checker; absent such a checker it is `Unknown` and physically infeasible." A distinct metric key is *not* a name to match on — it needs a registered implication.
- **Classified conformance-evidence records** — proof, exhaustive, applicable normative specification or vendor guarantee, empirical qualification, or unknown — each carrying "scope, target, implementation/helper identity, toolchain, device where applicable, reference oracle, corpus, and digest". Only the first three discharge a hard feasibility requirement; `Unknown` "cannot satisfy a hard contract" and must fail closed.
- **Reference evaluation of the predicate, decided rather than approximated.** ADR 0042 requires the inclusive comparison to be "evaluated exactly or with certified bounds rather than by rounded floating-point division". A transcendental reference is irrational at every nonzero representable argument, so `tiler-reference` needs a certified enclosure — an exact-rational series evaluation with a rigorous tail bound is sufficient and needs no new dependency. Size it to a *bounded* corpus: the three L3′ verticals ask for bounded conformance evidence, not an exhaustive sweep of the 2^32 F32 inputs.

## Non-goals

Admitting any operation key, emitting any structured-kernel construct, declaring any target honourability fact, and selecting the first profile tuples. Q-SEM-004's selection consumes this vocabulary and [`record-the-metal-elementary-function-accuracy-guarantee`](record-the-metal-elementary-function-accuracy-guarantee.md) supplies the backend evidence half; neither belongs inside the carrier. Region-level error budgets are a separate accepted layer owned by [`research-region-accuracy-contracts-and-analyzable-error-budgets`](research-region-accuracy-contracts-and-analyzable-error-budgets.md) and must not be folded in here.

## Boundary

The whole deliverable is public surface — the contract forms, the tolerance type, the metric key, the predicate language, the evidence classes, and the refinement entry point. A tested implementation is a concrete draft; acceptance of the boundary is Tom's, and the ticket is not done until that packet has been put to him.

## Reconsideration trigger

Active now: three filed L3′ verticals cannot register a key without it, and the roadmap forbids registering one anyway. If a design pass shows the full ADR 0042 algebra cannot be landed in one change, split by *contract form* — the constant-rational `Ulp` clause the first tuples need before the named-profile form the fast-math tables need — and never by dropping the normalization, coverage, or refinement rules, which are what stop an unsound implication from being accepted.
