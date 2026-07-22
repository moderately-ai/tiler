---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.region-accuracy-contract"
kind: "research"
title: "Region accuracy contracts and analyzable error budgets"
topics: ["numerics","accuracy","proof"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "partially-adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis","bounded-measurement"]
informs: ["tiler.contract.numerical-semantics","tiler.contract.correctness-and-testing"]
adopted_by: ["ADR-0017","ADR-0021"]
ticket: "research-region-accuracy-contracts-and-analyzable-error-budgets"
---

# Region accuracy contracts and analyzable error budgets

**Status:** contract model established; a narrow trusted-analyzer feasibility
gate passed, with independent certificate checking still open

## Traceability

- **Current disposition:** partially adopted; historical status text below records the report's state when written.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md) and [Correctness and testing](../../correctness-and-testing.md).
- **Adoption:** [ADR 0017](../../decisions/0017-local-vs-region-accuracy.md), [ADR 0021](../../decisions/0021-validated-value-assumptions.md).
- **Work record:** [research-region-accuracy-contracts-and-analyzable-error-budgets](../../../tickets/research-region-accuracy-contracts-and-analyzable-error-budgets.md).


## Outcome

Tiler can add region/output accuracy goals without weakening its current local
numerical semantics. The goal is a typed predicate over a named observable and
a named reference, not a scalar annotation copied onto graph nodes. It is a
hard plan-feasibility constraint and never an optimizer cost.

The initial executable product must not use a region goal to authorize an
otherwise-illegal contraction, reassociation, approximation, precision change,
materialization-boundary removal, or reduction topology. Later delegation is
compatible, but must be explicit: the goal names the operations or numerical
dimensions it governs, and a qualifying candidate carries evidence bound to
the complete candidate and assumptions.

This is the conservative long-term boundary. It preserves room for rigorous
mixed-precision and algebraic optimization without making the first optimizer
depend on a general nonlinear proof engine.

## Why local tolerances do not form a graph budget

An error bound describes a relation between two complete computations over a
domain. It is not generally the sum of per-operation tolerances:

- cancellation can turn a small absolute perturbation into unbounded relative
  error;
- the same uncertain value used twice is correlated, while interval reasoning
  may treat the uses independently;
- casts and materializations add observable rounding points that fusion can
  delete;
- overflow, underflow, signed zero, NaNs, and infinities are discontinuities,
  not ordinary real error;
- reduction error depends on contributor count, values, and topology;
- transcendental approximations have domain-specific errors;
- predicates can change the executed path after a small perturbation.

FPTuner handles a bounded expression class by using symbolic Taylor expansions
and interval functions, then solving a constrained precision-allocation
problem. Daisy assembles sound range and roundoff analyses for a restricted
program language. FPTaylor likewise derives rigorous bounds using symbolic
Taylor expansions. Their success supports an optional analyzer interface; it
does not make arbitrary tensor graphs automatically analyzable.

Herbie is useful for proposing and empirically comparing rewrites, but its
documented random sampling is a different evidence class. Samples can find a
counterexample to a claimed universal bound; passing samples cannot prove that
bound.

## Typed contract

Conceptually:

```text
RegionAccuracyGoal {
  goal_id: AccuracyGoalId,
  observable: ObservableSelector,
  reference: ReferenceSemantics,
  domain: AccuracyDomain,
  metric: ErrorMetric,
  tolerance: ExactTolerance,
  exceptional_relation: ExceptionalRelation,
  assurance: RequiredAssurance,
  delegated_permissions: DelegatedPermissions,
}
```

### Observable

`ObservableSelector` identifies an ordered graph output or a governed
intermediate value and the element set or aggregate being constrained. A
multi-output graph may carry different goals per output. An output norm is not
silently interchangeable with an elementwise maximum; aggregation is part of
the metric identity.

### Reference

`ReferenceSemantics` is one of:

- a digest of canonical Tiler semantic evaluation, including every cast,
  materialization, operation contract, and reduction order;
- a governed real/high-precision lift with a versioned operation interpretation;
- a versioned external oracle contract with a stable identity.

“The mathematically equivalent expression” is insufficient. For example,
strict f32 evaluation of `(a + b) - a` may be zero while the real expression is
`b`. The chosen reference changes which candidate is more accurate.

### Domain and assumptions

`AccuracyDomain` contains typed shape bounds and a value-precondition predicate
over root bindings. It can reuse Tiler's constraint language, but needs
relational predicates rather than only independent intervals to retain
correlations such as `x == y`. Every assumption is proved statically or
validated transactionally before routing commit. An unvalidated assumption
cannot support a certificate.

Dynamic reductions require a finite contributor-count bound. Unbounded rank,
unbounded extents, data-dependent control flow, or unsupported predicates make
the candidate `Unknown`; they do not weaken the requested goal.

### Metrics

Initial discriminated metrics should include:

- maximum elementwise absolute error;
- maximum elementwise relative error with an explicit reference-zero policy;
- maximum elementwise ULP gap for a named result format and versioned ordering;
- mixed absolute-plus-relative acceptance;
- a governed aggregate/norm metric with its reduction semantics.

Tolerance values are exact rationals or integers, never host floats. Relative
error at a zero reference is undefined unless the metric supplies an absolute
floor or a separate zero clause. ULP distance does not define NaN equivalence,
infinity behavior, signed-zero preservation, or subnormal policy; those remain
in `ExceptionalRelation` and the local numerical contract.

### Delegation

`DelegatedPermissions` is empty initially. A later goal may explicitly govern
selected numerical freedoms for selected operations. Anything outside that set
must remain locally legal. This prevents a broad output tolerance from becoming
an ambient `fast` flag.

A candidate using delegation is legal only if qualifying evidence covers the
whole delegated candidate. Local proof fragments may feed that analysis, but
are not independently summed into the output tolerance.

## Evidence and authority

```text
AccuracyEvidence {
  class: SoundProof | ExhaustiveFinite | Empirical | Unknown,
  analyzer_or_test_id,
  version,
  goal_digest,
  candidate_digest,
  target_numerics_digest,
  assumption_digest,
  result,
  provenance,
}
```

- `SoundProof` is accepted only through a governed checker or trusted analyzer
  profile and proves the bound for the complete admitted domain.
- `ExhaustiveFinite` is proof only when every member of a precisely enumerated
  finite domain was evaluated under the named semantics.
- `Empirical` records the generator, adversarial cases, seed, sample count,
  oracle precision, and observed distribution/maxima. It qualifies performance
  experiments but cannot satisfy `SoundUniversal` assurance.
- `Unknown` is an ordinary feasibility rejection with an explanation.

Evidence identity includes the scheduled candidate where topology, contraction,
target intrinsics, intermediate formats, or target subnormal behavior can
affect results. A proof about the logical graph cannot qualify an independently
changed physical plan.

The analyzer returns feasibility evidence, never a cost. Among certified plans,
the normal cost model may still trade execution time, memory, and compilation
cost. Proof time can cap search operationally but cannot be exchanged against a
larger numerical error than the goal permits.

## Compiler flow

```text
mandatory local semantic legality
  -> candidate physical plan with complete numerical identity
  -> bind/prove shape and value domain
  -> obtain applicable accuracy evidence
  -> reject Unknown or insufficient assurance
  -> cost only the qualifying candidates
  -> serialize goal + evidence provenance in explain/artifact identity
  -> validate runtime assumptions before routing commit
```

The database analogy is a check constraint over a physical implementation, not
a cost estimate. The analogy stops where physical schedule choices change the
computed value: reduction trees, contraction, target intrinsics, and storage
rounding can all enter the proof subject itself.

## Bounded empirical witness

[`region_accuracy_probe.py`](../../../spikes/numerics/region_accuracy_probe.py)
uses the repository-locked `mpmath==1.3.0` package with a
100-decimal-digit oracle and adversarial f32/f16 cases. It
demonstrates:

- deleting an f16 materialization changes a later result without any
  reassociation;
- relative error is undefined when the named reference is zero;
- choosing strict finite evaluation versus a real reference reverses the
  apparent accuracy conclusion for a cancellation example;
- left and tree reductions produce different errors for the same contributors.

The probe is deliberately labeled empirical. Its high-precision oracle and
adversarial points help find counterexamples; they are not directed-rounding
interval proofs and do not establish worst-case bounds.

Run:

```sh
uv run --locked python spikes/numerics/region_accuracy_probe.py
uv run --locked python -O spikes/numerics/region_accuracy_probe.py
```

The checked-in [bounded result](../../../spikes/numerics/region_accuracy/results.json)
is the byte-identical output of both modes and retains the exact source,
algorithm, recorded Python implementation/version/cache tag, dependency,
precision, and recorded host fields. It does not identify the Python executable
or complete interpreter build.

## Feasibility gate

Do not enable delegated region budgets until a follow-up experiment integrates
a sound analyzer or small proof checker and measures it on fixed-shape,
branch-free regions. The first proof profile should bound rank and extents and
support a narrow operation set before adding reductions or transcendental
approximations. It must record:

- supported/unsupported graphs and reason codes;
- proof bound versus adversarial observed maximum;
- proof and search time;
- sensitivity to interval subdivision and relational assumptions;
- casts, contraction, reduction topology, overflow, and subnormal coverage;
- independent certificate checking or the exact trusted analyzer boundary.

A useful result is not necessarily a tight proof for every graph. Fast,
non-vacuous certificates for common modest regions justify enabling the layer;
conservative `Unknown` for unsupported regions preserves correctness.

The follow-up [sound analyzer integration spike](sound-region-analyzer-spike.md)
historically found this profile feasible with a pinned Daisy trusted-analyzer
boundary. Its repaired adapter now fails closed, but the historical proof
streams and complete executable closure were not retained and a fresh governed
proof run was unavailable. It does not authorize delegated numerical freedoms:
a fresh evidence envelope, immutable analyzer execution, and independently
checked certificates remain separate gates.

## Primary sources

- [FPTuner: Rigorous Floating-Point Mixed-Precision Tuning](https://soarlab.org/publications/2017_popl_cbbsgr/)
- [Daisy framework paper](https://link.springer.com/chapter/10.1007/978-3-319-89960-2_15)
- [FPTaylor: Rigorous Estimation of Floating-Point Round-off](https://soarlab.org/papers/2018_toplas_sbbjrg.pdf)
- [Herbie sampling documentation](https://herbie.uwplse.org/doc/1.2/faq.html)
- [Herbie command-line sampling controls](https://herbie.uwplse.org/doc/2.0/options.html)
