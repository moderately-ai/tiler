---
id: implement-the-typed-accuracy-contract-vocabulary
title: Implement the typed transcendental accuracy-contract vocabulary
status: done
priority: p1
dependencies: []
related: [admit-the-silu-activation-family, admit-the-rms-normalization-family, admit-the-softmax-family, record-the-metal-elementary-function-accuracy-guarantee, numerical-policy-contract, own-operation-family-support-matrix, scope-transformer-nonlinear-normalization-and-reductions]
scopes: [implementation/ir, implementation/reference, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, numerics, transcendental, accuracy, boundary]
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

## Outcome

**Landed as one change, not a form split.** The reconsideration trigger authorizes splitting by contract form if the full algebra cannot land at once. It can, and the derivation is that the split would not have divided the work: the four forms are the cheapest part of ADR 0042 — a discriminated enum, a rounding rule name, and a key plus a descriptor digest — while the predicate normalization, the accuracy-domain coverage decision, the metric's dtype-capability check, the refinement relation, the evidence classes, and the certified enclosure are *shared by every form* and are the whole of the volume. Deferring the named-elementary form would have removed about forty lines and left the trigger's forbidden remainder — normalization, coverage, and refinement — carrying the split anyway. So the constant-rational `Ulp` clause the first tuples need and the named-profile form the fast-math tables need both land, and no remainder ticket is filed.

### The public packet, item by item

Every item below is public surface and therefore Tom's to accept; a tested implementation is a concrete draft. Each carries what would refute its shape.

1. **The four contract forms** — `AccuracyContractForm::{CorrectlyRounded, Faithful, BoundedPiecewise, NamedElementary}` in `crates/tiler-ir/src/semantic/accuracy/contract.rs`. Closed, discriminated, and never equated by name; `NamedElementary` carries a profile key, an immutable descriptor digest, and the authority whose descriptor defines the results. *Refuted by* a real specification that is none of the four, or by a case where the named-profile form needs to carry the descriptor's *content* rather than its digest.
2. **The tolerance type** — `ExactTolerance` over `ExactRational` (`rational.rs`): exact, nonnegative by construction, in lowest terms so one number has one encoding, with no host floating-point constructor at all. *Refuted by* a specification whose tolerance is irrational (ADR 0042 sends those to a named profile, so finding one that cannot go there refutes the split).
3. **The metric key** — `AccuracyMetricKey`, `ulp_reference_gap_metric_key()`, and `UlpFormat` (`metric.rs`). The scale is derived from the registered dtype descriptor through an explicit two-row rule table (`ieee-binary`, `bfloat`), each row carrying its normative basis; every other class is refused by name. *Refuted by* a dtype whose adjacent-value behaviour is derivable but whose class this table cannot state a basis for — that is a missing row, not a missing mechanism — or by a reading of ADR 0042 under which deriving IEEE parameters from the class is a "guess" rather than an application of the class's own authority.
4. **The predicate language** — `AccuracyPredicate` with `absolute`, `relative`, `absolute_relative`, `ulp`, `all_of`, `any_of` (`predicate.rs`), normalized by construction and refusing every non-canonical spelling on decode. *Refuted by* a predicate shape a vendor states that none of the six expresses without approximation.
5. **The accuracy-domain language** — `AccuracyDomain`, `AccuracyDomainClause`, `DomainInterval`, `ReferenceResultClass`, `ReferenceResultConstraint` (`domain.rs`), with decided coverage, intersection semantics, and a mandatory operation-specific proof behind any reference-result assertion. *Refuted by* an operation whose ordinary input domain is not a union of exact intervals per operand — a piecewise specification keyed on something other than the operand's value.
6. **The evidence classes** — `ConformanceEvidenceClass` and `ConformanceEvidence` (`evidence.rs`), with `discharge()` as the only route to a hard feasibility conclusion and `Unknown`/`EmpiricalQualification` failing closed. *Refuted by* a required evidence field ADR 0042 lists that the nine-field record omits.
7. **The refinement entry point** — `refines(candidate, required, registry)` returning `RefinementOutcome::{Refines, Unknown}` over an open `RegisteredImplicationRegistry` (`refinement.rs`). *Refuted by* an implication the closed algebra should establish and does not, in a direction that matters.
8. **The reference half** — `CertifiedEnclosure`, `exp_enclosure`, `rsqrt_enclosure`, `decide_predicate`, `decide_contract`, `EnclosurePrecision` (`crates/tiler-reference/src/accuracy.rs`). *Refuted by* a corpus argument whose enclosure the governed halving or term bound cannot produce.

### Every refusal, watched failing

| Rule | Diagnostic code | Perturbation that produced it |
| --- | --- | --- |
| Non-canonical Boolean nesting | `accuracy.predicate.non-canonical-nesting` | a decoded `all-of` whose member is an `all-of` |
| Non-canonical Boolean order | `accuracy.predicate.non-canonical-order` | the same two members encoded in the reverse of canonical-encoding order |
| Duplicate Boolean member | `accuracy.predicate.duplicate-member` | one member encoded twice |
| Non-canonical singleton | `accuracy.predicate.non-canonical-singleton` | a one-member `all-of` |
| Empty collection | `accuracy.predicate.empty-collection` | a zero-member `all-of` and a zero-member `any-of` |
| Undefined relative at a zero reference | `accuracy.predicate.undefined-relative-at-zero-reference` | a `Relative` clause whose reference constraint is `unconstrained`; the same clause with a justified `Nonzero` verifies |
| Empty domain interval | `accuracy.domain.empty-interval` | `(1, 1]`; `[1, 1]` is admitted |
| Empty clause set | `accuracy.domain.empty-clause-set` | a bounded contract with no clauses |
| Unverifiable gap | `accuracy.domain.incomplete-coverage` | two clauses meeting at an open endpoint, witness `0`; closing one endpoint covers |
| Coverage not decidable | `accuracy.domain.coverage-not-verifiable` | four operands × sixteen clauses, past the 4,096-cell budget |
| Unjustified reference class | `accuracy.domain.unjustified-reference-result-class` | asserting `Nonzero` with no proof reference |
| dtype/metric incompatibility | `accuracy.metric.incompatible-dtype` | every non-float governed scalar, counted: 5 compatible, 25 refused, 30 total |
| Metric undefined | `accuracy.metric.reference-out-of-finite-range` | a reference one step above `f32::MAX` |
| Empty composed result set | `accuracy.contract.empty-composed-result-set` | `Ulp(…, 1/4)`, an unbounded `Absolute`, and an `Absolute` below the proved spacing; `Ulp(…, 1/2)` verifies |
| Unregistered cross-metric implication | `accuracy.refinement.unregistered-metric-implication` | `Ulp(apple::msl-ulp@1, 4)` against `Ulp(tiler::ulp-reference-gap@1, 4)`; registering a `ScaledMetric` row decides it, and a factor of two refuses it again |
| Evidence cannot discharge | `accuracy.evidence.class-cannot-discharge` | empirical and unknown records, with the loop counting 3 discharged and 2 refused out of 5 |
| Irreproducible measurement | `accuracy.evidence.{missing-reference-oracle,missing-corpus,malformed-digest}` | an empirical record missing each field in turn |
| Enclosure refusals | `reference.enclosure.{argument-too-large,precision-unreachable,precision-too-coarse,outside-domain}` | `2^40`; a 5,000-bit grid; a four-bit grid on `rsqrt(2^-20)`; `rsqrt(0)` and `rsqrt(-1)` |
| Undecided conformance | `reference.conformance.{enclosure-too-wide,reference-not-provably-nonzero,unsupported-metric,named-profile-not-interpretable,no-applicable-clause}` | see the enclosure proof below |

### The enclosure's failure proof

`a_degraded_enclosure_yields_undecided_rather_than_a_silent_pass` places a candidate exactly on a four-ULP boundary. At `EnclosurePrecision::binary32_corpus()` the decision is definite; at `EnclosurePrecision::new(2)` the same comparison returns `Undecided { EnclosureTooWide }`. It does not resolve toward `Conforms`, which is what a check that cannot fail would do, and it does not resolve toward `Violates`, which would reject correct implementations. Soundness is separately checked by identities the enclosure arithmetic must contain — `exp(x) · exp(-x) ∋ 1` over the whole L3′ corpus, `rsqrt(x)² · x ∋ 1`, strict monotonicity — plus decimal brackets on `e` and `exp(2)` to nineteen digits, which a too-narrow tail bound would miss. The decision also says *no*: `the_decision_says_both_yes_and_no` gets `Conforms` at the rounded value and `Violates` one binade away and at five ULPs against a four-ULP bound, with the same candidate conforming at six.

### Identity, digests, and scope

**The explain digest did not move.** `0b7759de2d9b5756` at `crates/tiler-compiler/src/explain.rs:3739` is unchanged and the full workspace suite is green, which is the traced expectation rather than a lucky pass: the vocabulary registers nothing into the frozen semantic snapshot, so `SemanticIdentity`'s registry-snapshot component cannot move. The contraction precedent moved the digest because it registered an operation; this registers none. **No `implementation/compiler` scope was needed or added.**

### Navigation edits made, and one deliberately not made

- The [support matrix](../docs/roadmap.md#operation-family-support-matrix) transcendentals row: its Fact cell said "no ticket implements one", which this landing makes false. Rewritten to state what exists, that the rung is unmoved because nothing is registered, and that the Milestone 1 precondition is now buildable rather than blocked.
- Absence check 1 in the same file: it claimed "returns no output at all", **which was already false before this landing** — `crates/tiler-ir/src/semantic/broadcast/tests.rs` has named the rotary `cos`/`sin` tables in a comment since `762ba34`. Corrected, with the pre-existing staleness named rather than folded into this change's own effect.
- **Not made, and out of scope:** ADR 0016 and ADR 0042 still carry `implementation_status: "not-started"`, which this landing makes stale — `partial` is what they now describe. `docs/decisions/[0-9]*.md` is `contracts/decisions`, which this ticket does not hold, so the roadmap row *names* the staleness instead of hiding it. **This needs a `contracts/decisions` edit before the next reader takes the frontmatter at face value.**
- `docs/numerical-semantics.md`'s transcendental section is `contracts/numerics`, not held here, and nothing in it became false: it describes the accepted contract, which is unchanged, and its one implementation Fact is about `ApproximationEnvelope`, whose doc comment this change updated in place.

### Unsupported cases and deliberate conservatism

- `AnyOf` nonemptiness requires a *single* disjunct to hold across a whole cell, so a disjunction covered piecewise by two members is `Unknown` rather than accepted. Splitting the clause's domain is the contract-side answer, and the refusal names it.
- The metric's format-rule table interprets two descriptor classes. `ocp-binary-element`, `ocp-exponent-scale`, and `ieee-decimal` are refused with a reason rather than approximated; each needs its own row with its own basis.
- `decide_contract` reports a named-elementary profile as uninterpretable, because this build holds the descriptor's digest and not its content.
- No operation key, no structured-kernel construct, no target honourability fact, and no profile tuple. Q-SEM-004 and `record-the-metal-elementary-function-accuracy-guarantee` remain open, and the cross-metric implication the latter will need is deliberately *unregistered* so it stays `Unknown` until someone derives it.

### Verification

`cargo fmt --all --check`; `cargo check -p tiler-ir -p tiler-reference --all-targets`; `cargo clippy -p tiler-ir -p tiler-reference --all-targets -- -D warnings` (clean, with no new `#[allow]`); `cargo nextest run --workspace` 1,793 → 1,798 passing; `cargo test --doc`; `RUSTDOCFLAGS="-D warnings" cargo doc`; `make full` including the release-profile `tiler-reference` numerical tests, which the enclosure passes by construction because every value in it is an exact integer ratio; `git diff --check`; `tkt lint`; `tkt guard`.

**Provisional boundary acceptance (2026-08-01, overnight mode).** The coordinator provisionally accepted the eight-item public packet under Tom's stated bar — the four contract forms, exact tolerances, the `ulp-reference-gap` metric with rejection-not-guessing dtype compatibility, the six predicates with canonical-spelling refusal at decode, the domain language with witness-proved nonemptiness, the five evidence classes, the conservative refinement relation with the registered-implication registry (the Metal ≤4-ulp case demonstrably expressible and demonstrably NOT name-matched), and the certified-enclosure reference half with its failure-proof. Recorded for Tom's morning review with one-revert isolation.
