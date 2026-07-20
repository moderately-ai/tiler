---
schema: "tiler-doc/v1"
id: "ADR-0042"
kind: "decision"
title: "Use typed transcendental accuracy contracts"
topics: ["numerics","transcendentals","accuracy"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.transcendental-accuracy-precedents"]
ticket: "numerical-policy-contract"
---

# 0042: Use typed transcendental accuracy contracts

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [transcendental accuracy precedents](../research/numerics/transcendental-accuracy-precedents.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

ADR 0016 requires every transcendental operation to carry resolved accuracy,
but left the concrete vocabulary open. A bare `fast`, `approximate`, or
`precise` label is insufficient. Existing specifications use correctly rounded
results, faithful rounding, ULP bounds, absolute bounds, relative bounds, and
piecewise combinations. Their exceptional-value and subnormal rules are often
separate, and even the term ULP has competing definitions.

Backend evidence also differs. A normative platform guarantee, an exhaustive
finite-format test, a proof, and a vendor table of maximum errors observed in
non-exhaustive testing do not establish the same claim. Tiler must not turn an
implementation name or an empirical measurement into an unstated portable
error guarantee.

## Decision

Every transcendental operation identifies immutable, versioned reference
semantics and one discriminated accuracy contract:

- **correctly rounded:** on the ordinary finite in-range reference domain,
  round the infinitely precise reference result once to the result dtype using
  the named rounding rule;
- **faithful:** return the exact representable reference result when one
  exists, otherwise either of the two adjacent result-dtype values that bracket
  the exact finite in-range reference result;
- **bounded piecewise:** satisfy a complete set of typed domain clauses, each
  carrying one or more exact error bounds; or
- **named elementary behavior:** satisfy one immutable, versioned behavior
  profile whose complete result set is defined outside the generic metrics.

The initial generic bounded predicates are:

```text
Absolute(t):       |z - r| <= t
Relative(t):       |z - r| / |r| <= t, with the clause excluding r = 0
AbsoluteRelative(a, q): |z - r| <= a + q * |r|
Ulp(metric_key, t): |z - r| / ulp_result_dtype(r) <= t
AllOf(predicates):  every member predicate is satisfied
AnyOf(predicates):  at least one member predicate is satisfied
```

Here `r` is the infinitely precise reference result and `z` is the
mathematical value of the finite result-dtype candidate selected before
result-subnormal and signed-zero mapping. Every tolerance is a canonical exact
nonnegative number, initially an integer or rational, never a host
floating-point literal.

`AllOf` and `AnyOf` are canonical, nonempty, bounded collections. They are
needed because additive absolute-plus-relative tolerance is not equivalent to
an absolute-or-relative guarantee. Every atomic predicate must be defined over
its enclosing clause domain; in particular, a relative predicate requires that
domain to exclude `r = 0`. Zero can instead use an absolute or
absolute-plus-relative clause. There is no hidden epsilon.

Nested same-kind Boolean predicates are flattened, sorted by canonical
encoding, deduplicated, and bounded in depth and cardinality; empty collections
are invalid and singleton collections canonicalize to their member. The
definedness rule applies recursively, so `AnyOf` cannot hide an undefined
relative predicate at reference zero.

The initial ULP metric key is `tiler::ulp-reference-gap@1`, matching the
definition used by OpenCL. If finite `r` lies strictly between consecutive
numerically distinct finite values `a < b`, `ulp(r) = b - a`. If `r` is
representable, one selected value is `r` and the other is its nearest
numerically unequal finite neighbor; where predecessor and successor gaps
differ, the smaller gap is used. Thus binary `ulp(2^e)` uses the predecessor
gap, while the scale increases immediately above that value. At either zero,
duplicate signed-zero encodings are ignored and the scale is the smallest
positive finite representable value. For a gradual-underflow format this is
the minimum positive subnormal; for a format without subnormals it is the
minimum positive normal.

The metric is defined only when `r` and `z` are finite and `r` lies within the
finite numerical range of a compatible result format. Finite overflow,
infinite references or candidates, and NaNs are outside it. In particular,
`tiler::ulp-reference-gap@1` does not inherit OpenCL's additional hypothetical-
successor overflow allowance.

Metric compatibility requires the dtype descriptor to expose an ordered set of
numerically distinct finite values and adjacent-value behavior. Multiple
encodings of one value, such as decimal cohort members or signed zeros, do not
create zero-width gaps; their representation/quantum behavior remains a
separate operation contract. A dtype/metric pair lacking this capability is
rejected rather than guessed.

The inclusive comparison is evaluated exactly or with certified bounds rather
than by rounded floating-point division. Throughout a gradual subnormal
interval the scale is the result format's minimum positive subnormal; flushing
remains a separate result-subnormal contract.

A bounded contract's clauses use a dedicated versioned accuracy-domain
predicate language over all exact input operands and typed reference-result
classes such as finite and nonzero. An input-domain predicate may justify a
reference-result class only through an operation-specific proof. These
predicates describe semantic cases; they are not automatically runtime guards.
Clauses must cover the complete ordinary input domain admitted by the
operation. Every matching clause applies, so overlap means intersection rather
than priority; unverifiable gaps or a possibly empty intersection reject the
contract. This supports genuinely piecewise specifications without an
order-dependent fallback.

The initial generic predicates have constant exact rational bounds.
Input-dependent tolerance formulas or non-rational constants use a governed,
nominal `NamedElementaryProfileKey` plus its immutable canonical descriptor
digest; they are not approximated into constants. Named behavior is not an
escape hatch for an undocumented approximation: its descriptor completely
defines domains and allowed results, and a key/revision cannot change that
descriptor.

The observable result set is composed in this order:

1. apply the operation's input-subnormal contract;
2. compute and classify the exact reference result;
3. for an ordinary finite in-range reference, select a finite result-dtype
   candidate satisfying the accuracy contract; otherwise apply the explicit
   NaN/infinity/domain/finite-overflow contract;
4. apply the resolved result-subnormal and signed-zero mappings to that
   candidate; and
5. apply any required NaN canonicalization and expose the value-only result.

The output policies transform or restrict the candidate set; they do not erase
the accuracy requirement. For example, a flush policy may map an otherwise
conforming subnormal candidate to its specified zero. Verification requires
the final composed result set to be nonempty for every admitted input and
rejects the contract when it cannot establish that fact. These policies remain
independent identity dimensions under ADRs 0018--0020 rather than being
inferred from an error metric.

The semantic identity contains the operation and dtype signature, reference
semantics, complete accuracy contract, domains, exact bounds, metric versions,
and the independent exceptional-value contracts. An implementation is legal
only when its allowed result set refines that semantic result set.

Conformance evidence is recorded separately as one of:

- formal proof;
- exhaustive testing over the complete admitted finite input space;
- an applicable normative specification or vendor guarantee;
- empirical qualification under a named test corpus and environment; or
- unknown.

Proof, exhaustive evidence, or an applicable normative guarantee may discharge
a hard accuracy feasibility requirement. Empirical results detect regressions
and characterize implementations but do not prove an unmeasured worst-case
bound. Unknown behavior remains unknown and cannot satisfy a hard contract.
Evidence records include their scope, target, implementation/helper identity,
toolchain, device where applicable, reference oracle, corpus, and digest.

Refinement is a conservative proof relation, not an arbitrary provider claim.
Initial host rules cover identical normalized contracts, identical
reference/domain/metric predicates with tighter exact bounds, normalized
`AllOf`/`AnyOf` implications the closed algebra can establish, and explicitly
registered mathematical implications such as a correctly rounded result
satisfying a compatible looser bound. Correctly rounded, faithful, and one-ULP
contracts are never equated by name. Any other implication requires a
certificate accepted by a versioned trusted checker; absent such a checker it
is `Unknown` and physically infeasible.

The approximate-intrinsic permission is therefore not a boolean. Where a
frontend permits a relaxation, it resolves to a maximum admissible accuracy
contract or versioned envelope. Selecting an approximate intrinsic still
requires conformance evidence that it refines the operation's resolved
contract.

Recognition does not enable every contract, function, dtype, or backend in the
first product profile. Initial vertical support is selected separately from
this stable vocabulary.

## Consequences

- Constant-bound piecewise guarantees and portable library guarantees are
  representable without conflating them; input-dependent vendor formulas use
  explicit named profiles initially.
- Faithful rounding is not mislabeled as a one-ULP bound, and relative error at
  a zero reference cannot be interpreted inconsistently.
- Backend labels and compiler flags select candidate implementations; they do
  not define semantics or prove accuracy.
- Bounded and faithful contracts admit result sets, so they do not imply
  cross-target bitwise equality.
- New metrics or named profiles require new versioned keys rather than changing
  the meaning of stored programs.

## Alternatives considered

A scalar maximum-ULP field cannot express absolute-error regions, mixed
absolute/relative tolerances, or functions whose bound depends on the input.
One opaque approximation flag couples unrelated numerical freedoms. Encoding
the chosen backend intrinsic directly as portable semantics confuses meaning
with implementation. Treating empirical maxima as guarantees would turn
incomplete tests into unsound optimizer feasibility evidence.
