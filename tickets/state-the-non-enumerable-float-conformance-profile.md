---
id: state-the-non-enumerable-float-conformance-profile
title: State the bounded conformance profile for non-enumerable float formats
status: deferred
priority: p3
dependencies: [conform-the-bf16-vertical-end-to-end]
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, evaluate-bf16-reference-semantics, measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, conformance, floats]
---
## User-visible outcome

`f64` and `f128` have a stated conformance standard that says what its bounded universe is, rather than inheriting a claim BF16 could make only because it has 65,536 encodings.

## Why this exists

**Fact.** [The dtype support ledger](../docs/dtype-support.md)'s dry run states the obligation twice in its own words. At rung 4: reference semantics "reuses the exact-rational shape, which needs no widening carrier at all; but F16 alone is exhaustively enumerable, so **F64 and F128 must state a bounded profile** in place of BF16's 65,536-encoding round trip". At rung 13: "F64 and F128 are not exhaustively enumerable and need a stated bounded profile".

**Fact.** Nothing in the corpus says what such a profile must state. [The document metadata contract](../docs/document-metadata.md) fixes that `exhaustive-finite` means "every member of an explicitly named finite universe was checked" and that `bounded-measurement` "holds only for the recorded inputs, environment, and procedure" — so the two classes are not interchangeable, and a sampled corpus that called itself exhaustive would be a false evidence claim rather than a weak one.

**Fact.** `tiler::f16@1`, `tiler::f64@1`, and `tiler::f128@1` are registered with complete structural and special-value descriptors, and no reference, physical ABI, optimizer, kernel, lowering, or runtime-validation vertical exists for any of them. The checked Apple record supplies F16 conformance evidence on its exact host and family rows and supplies no F64 or F128 evidence at all.

## Activation trigger

A named workload selects `f16`, `f64`, or `f128`. **A second target measurement alone does not fire it**, which the ledger's `### Other IEEE binary floats and BF16` trigger states directly.

## What the work would be

Derive, from the BF16 vertical once it is conformed end to end, what transfers and what does not. The reusable half is the exact-rational oracle shape. The non-reusable half is the evidence class: state the sampling population, the boundary corpus (subnormal edges, ties, the overflow midpoint from both sides, the exceptional values), and the named universe, so the claim is `bounded-measurement` with a precise limit rather than an overreaching `exhaustive-finite`. F16 is the control: it is enumerable, so the profile's answer for F16 must agree with the exhaustive result.

## Closes when

The trigger has fired and a bounded conformance profile exists that names its universe, is demonstrated on F16 against the exhaustive result, and is stated in [Correctness and testing](../docs/correctness-and-testing.md) rather than only in a research record.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-3 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
- Depends on [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) because "reuses the BF16 pattern" is a claim about a vertical that has not yet run end to end; deriving from an unconformed pattern would propagate whatever it gets wrong.

## Trigger check log

- 2026-08-04 — **not fired.** Track D-3's trigger is checked in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md):155: no named workload selects `f16`, `f64`, or `f128`, and a second target measurement alone does not fire it. The dependency [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) is also `todo`, so the pattern this would derive from has not run end to end.
- 2026-08-09 — **not fired, although the dependency is now complete.** [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) is `done`, so the pattern this work would study now exists. No named workload selects `f16`, `f64`, or `f128`, however, and the trigger explicitly requires that workload selection rather than BF16 completion or another measurement row.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **Checkable half.** `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u | grep -E '^tiler::(f16|f64|f128)@'` returns exactly `tiler::f128@1`, `tiler::f16@1`, and `tiler::f64@1` — the three catalog identities, with no operation over any of them among the 19 registered operation-key constructors. **Anchor that pattern.** An unanchored `f16` also matches `bf16` and returns seven keys including `tiler::add-bf16@1`, which would read as the very selection this trigger watches for; the `^tiler::(f16|f64|f128)@` form is what distinguishes them. **This condition is not mechanically checkable, and saying so is the repair.** The trigger is *a named workload selecting* `f16`, `f64`, or `f128`, and the entry above is explicit that neither a second target measurement nor the completed BF16 vertical fires it. A human must read `docs/dtype-support.md`'s `### Other IEEE binary floats and BF16` trigger for a workload that has made that selection. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
