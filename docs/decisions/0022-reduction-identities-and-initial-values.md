---
schema: "tiler-doc/v1"
id: "ADR-0022"
kind: "decision"
title: "Define reduction identities and initial values"
topics: ["numerics","reductions","semantics"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality"]
ticket: "reduction-semantics-contract"
---

# 0022: Define reduction identities and initial values

**Status:** accepted

The identity terminology in this decision is refined by ADR 0025: an empty
result is not automatically a bitwise-neutral, replicable padding value.

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md).
- **Work record:** [reduction-semantics-contract](../../tickets/reduction-semantics-contract.md).


## Context

Empty reductions need defined behavior. Some operations have a typed
mathematical identity, such as zero for addition, while others such as the
initial minimum/maximum contracts do not. An explicit initial value could mean
either a real accumulator seed or a value used only when the domain is empty;
overloading both meanings would make logical and parallel execution ambiguous.

NumPy defines `initial` as the value that starts a reduction and requires one
for an empty reduction whose operation has no identity. Rust similarly
distinguishes seeded `fold` from unseeded `reduce`. StableHLO represents init
values explicitly in its reduction operation. Parallel lowering adds a further
constraint: an arbitrary seed is not necessarily neutral and cannot be
duplicated across partial reductions.

## Decision

Each reduction operation declares a typed empty-domain result or explicitly
rejects an empty domain. It separately declares any algebraic-identity and
replicable-padding capabilities under its resolved conformance contract.

An optional explicit `initial` is a true reduction seed. It contributes exactly
once to each logical output reduction domain whether that domain is empty or
non-empty. It is not an empty-only fallback.

A physical schedule may inject or replicate a padding value only when the
resolved operation contract proves it observably neutral. It may not infer that
property from the empty result, nor replicate an arbitrary initial value per
lane, partition, threadgroup, or reduction pass.

An identity-less reduction requires either an explicit initial value or a
proven/runtime-validated non-empty domain. Static violations fail graph
verification; a failed dynamic non-empty semantic precondition produces a
precise invalid-input error before dependent execution.

## Consequences

- Common monoidal reductions have natural empty results.
- `minimum([], initial=10)` and `minimum([20], initial=10)` both produce `10`.
- Parallel reduction topology must track the seed as one logical contributor.
- Identity, initial-value conversion, empty behavior, order permissions, and
  accumulator dtype all participate in semantic and artifact identity.
- Empty-only fallback remains expressible later as a separately named operation
  or explicit conditional without changing `initial` semantics.

## Implementation boundary

Added 2026-08-01 by [`re-audit-adr-implementation-status-after-the-runtime-and-metal-landings`](../../tickets/re-audit-adr-implementation-status-after-the-runtime-and-metal-landings.md), which moved `implementation_status` from `not-started` to `partial`. This section states which clauses that value rests on and which it does not, read at `2aa0824`. It is a status record and adds no decision.

**Realized — a reduction family either states its empty-domain outcome or rejects an empty domain, and the rejection is enforced.** The contraction declares `refused-an-unseeded-fold-has-no-empty-result` as a definition fact entering durable identity (`crates/tiler-ir/src/semantic/contraction.rs:960`), and the schedule verifier refuses a contracted space with no points (`crates/tiler-ir/src/schedule/builder/contraction.rs "contracted space with no points has no result to commit"`). The extrema fold carries no empty-domain field at all — `ScalarProgram::StrictSerialMaximum` states why at `crates/tiler-ir/src/schedule/model.rs:408`: no binary32 value is neutral for `Maximum`, so "a field carrying one would be a value that can never be correct" — and the verifier refuses a reduced domain with no contributors at `crates/tiler-ir/src/schedule/builder/family.rs "EmptyDomainContract::NoIdentity => {"`. This is the decision's "an identity-less reduction requires either an explicit initial value or a proven/runtime-validated non-empty domain", with the static violation failing verification.

**Realized — a typed empty result where one exists, pinned rather than assumed.** The sum families carry `empty_identity_bits` (`crates/tiler-ir/src/schedule/model.rs:330`, `:345`, `:373`) and the verifier requires the value to be `+0.0` bits at every admission site (`crates/tiler-ir/src/schedule/builder/family.rs "EmptyDomainContract::Identity { bits } => bits == 0.0_f32.to_bits()"`, reached from the serial, multi-dispatch, and cooperative fold gates), including for each pass of a split, so a pass cannot inherit or invent an identity its sibling disagrees with.

**Citation repair — 2026-08-19 by [`re-anchor-the-schedule-builder-line-citations`](../../tickets/re-anchor-the-schedule-builder-line-citations.md).** `crates/tiler-ir/src/schedule/builder.rs` became the `crates/tiler-ir/src/schedule/builder/` directory, so the two paragraphs above carry quoted anchors in place of the retired pins `:403` and `:792` and the five-site list `:674`, `:706`, `:741`, `:892`, `:1043`. Both claims were re-read at the split and are unchanged; what did change before the split is the shape of the second one's subject — the five separate `+0.0` comparisons are now one shared authority, `empty_domain_is_satisfied`, which each fold gate calls, so "at every admission site" is discharged by one function rather than by five copies of a constant.

**Realized — the seed is a declared dimension, not an omission.** The contraction declares `none-the-accumulator-starts-at-the-first-product` (`crates/tiler-ir/src/semantic/contraction.rs:956`), so a consumer reads the seed from the contract instead of defaulting one.

**Unrealized — no reduction admits an explicit `initial`.** Every admitted family is unseeded, so "an optional explicit `initial` is a true reduction seed contributing exactly once whether the domain is empty or non-empty" is declared-absent rather than supported, and `minimum([], initial=10)` is not expressible. The parallel-topology consequence that a seed is one logical contributor has nothing to track.

**Partial — replicable padding is a verified schedule statement, not a family capability declaration.** `ContributorCoverage::IdentityPadded` states a `ReductionPaddingIdentity` the intrinsic verifier must prove two-sided-neutral; exact coverage still carries none, and `ContributorPartition::covers` keeps its exact meaning. Families still do not declare an algebraic-identity or padding capability on their conformance contract — the proof is a schedule-verification derivation against the combiner. ADR 0025 owns the empty-result versus padding split and now reads `partial`. No padded schedule is lowered.

**Unrealized — the empty-domain declaration is not uniform across families.** The strict serial sum's definition facts state its fold order, accumulation, and canonical NaN bits and no empty-domain outcome (`crates/tiler-ir/src/semantic/registry.rs:2093`); its `+0.0` identity is a schedule-layer field the verifier pins rather than a semantic declaration. The decision asks every reduction operation to declare one.

## Alternatives considered

Requiring an explicit seed for every reduction is uniform but needlessly
verbose for operations with valid intrinsic identities. Treating `initial` as
an empty-only default is convenient but is not a reduction seed and obscures
its behavior on non-empty input. Letting backends choose empty behavior would
make fusion and cross-target conformance unsound.

## Primary precedents

- [StableHLO `reduce`](https://openxla.org/stablehlo/spec#reduce)
- [NumPy `ufunc.reduce`](https://numpy.org/doc/stable/reference/generated/numpy.ufunc.reduce.html)
- [Rust `Iterator::fold` and `Iterator::reduce`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
