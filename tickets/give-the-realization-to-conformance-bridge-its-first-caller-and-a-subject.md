---
id: give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject
title: Give the realization-to-conformance bridge its first caller and a subject
status: in-progress
priority: p2
dependencies: []
related: [accept-the-bf16-subnormal-resolution-carrier, wire-the-bf16-reference-to-the-realization-it-is-told, apply-the-declared-numerical-conformance-on-every-reference-evaluation-path]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, fail-closed]
claimed_from: todo
assignee: w-give-the-r
lease_expires_at: 1786140348
---
## The gap

**Fact — the checked bridge has no caller.** `ReferenceNumericalConformance::from_realization` (`crates/tiler-reference/src/conformance.rs:166`) is the designed, documented path from a region's declared `NumericalRealization` to the conformance a reference evaluation runs under. Three separate module headers cite it as the bridge — `standard.rs:32`, `registry.rs:190`, `evaluate.rs:71`, `oracle.rs:1364`. **Nothing calls it.** Every construction site in `crates/` and `prototypes/` is `ReferenceNumericalConformance::strict()` or a test's `new()`. Verified 2026-08-07 by the coordinator: `grep -rn "from_realization" crates/ prototypes/` returns only doc-comment references.

**Consequence.** Every reference evaluation in the workspace runs under the strict reading whatever a region declared, which is precisely the "silent single-value oracle" `from_realization` was written to refuse. The refusal machinery works and is tested; it is simply never reached, so a region declaring a flushing realization is compared against a preserving oracle today.

**Fact — the bridge discards the format subject.** `from_realization`'s destructuring reads `canonical_arithmetic_nan_bits: _` (`:171`), the one field identifying the region's arithmetic type. The resulting object is structurally format-agnostic while `registry.rs:181` documents it as being for "a capability that performs host binary32 arithmetic". That mismatch is the boundary at which the subject is lost.

## What this owes

- **The first real caller**, so a region's declared realization reaches the evaluation performed under it, and the existing refusals become reachable rather than merely tested.
- **The subject carried across**, drawn from the region rather than from a new field on `NumericalRealization` — the arithmetic type is already a total function of the region's scalar program (`region_arithmetic_type`, `crates/tiler-ir/src/schedule/model.rs:1333`).
- **A capability-side agreement check**: a capability applies a conformance only when its subject matches the capability's own format, and returns a typed refusal otherwise. **This is the obligation Tom's 2026-08-07 arm-A decision deferred to here** — see [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md). It is placed here rather than in the BF16 family because this is where the subject is lost, and because unlike the mixed-width refusal that decision rejected, **this check is reachable**: handing a BF16 capability an `f32`-derived conformance is constructible in a test and can be watched failing.

## Why this matters beyond BF16

`registry.rs:181` divides capabilities into those performing host binary32 arithmetic, which must consult the conformance, and those performing no host arithmetic, which have nothing to read. The BF16 family is neither — it produces arithmetic results over BF16's value set by exact rational arithmetic. That third case is currently undocumented, and the subject is what distinguishes it. Any future non-binary32 family lands in the same gap.

## Required evidence

- A region declaring a non-strict realization is evaluated under it, end to end, rather than under `strict()`.
- At least one existing `from_realization` refusal watched firing through the new caller, proving the path reaches them.
- The subject mismatch watched failing: a capability handed a conformance resolved for another format refuses, with the refusal observed before restoration.
- Populations counted, so a path that stopped being exercised cannot look green.

## Closes when

A declared realization reaches the evaluation performed under it, the conformance carries its arithmetic subject, a mismatch is a typed refusal observed failing, and `registry.rs`'s two-case division is restated to cover a family that performs non-binary32 arithmetic.

## Graph maintenance

Filed 2026-08-07 by the coordinator while assessing the BF16 subnormal carrier fork for Tom. The unused bridge was found by reading rather than reported by any ticket, and it is the reason neither arm of that fork would have delivered its stated outcome alone.
