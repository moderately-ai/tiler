---
id: refuse-an-enclosure-precision-the-grid-arithmetic-cannot-express
title: Refuse an enclosure precision the grid arithmetic cannot express
status: in-progress
priority: p2
dependencies: []
related: [bound-the-certified-exponential-s-cost-in-its-admitted-argument-region]
scopes: [implementation/reference]
shared_scopes: []
paths: []
tags: [numerics]
claimed_from: todo
assignee: agent-enclosure-depth
lease_expires_at: 1785971429
---
## The finding

**Fact — a public input panics instead of refusing.** `EnclosurePrecision::new` is a public `const fn` over any `u32`, and `exp_enclosure` turns the grid width into a signed exponent at `crates/tiler-reference/src/accuracy.rs`:

```
let threshold = ExactRational::power_of_two(
    -i32::try_from(precision.fraction_bits().saturating_add(2))
        .expect("a bounded grid width fits i32"),
);
```

A width past `i32::MAX` fails that conversion and panics. Probed at the base of `bound-the-certified-exponential-s-cost-in-its-admitted-argument-region`, one `exp_enclosure(&ExactRational::one(), EnclosurePrecision::new(bits))` per row:

| `bits` | Outcome |
| --- | --- |
| `100_000` | `reference.enclosure.precision-unreachable` |
| `2_147_483_646` | panic — `a bounded grid width fits i32: TryFromIntError(PosOverflow)` |
| `u32::MAX` | panic — same |

**Fact — the doc comment claims the opposite.** `exp_enclosure`'s `# Panics` section says it panics only if "a grid width bounded by the caller's own `EnclosurePrecision` leaves `i32`, which those bounds make unreachable". Nothing bounds `EnclosurePrecision`: `new` accepts any `u32` and returns `Self` infallibly, so the claimed bound does not exist. The source wins and the comment is the defect.

**Inference — same fail-closed family as the argument bound, different axis.** `bound-the-certified-exponential-s-cost-in-its-admitted-argument-region` bounded the *argument* region so every admitted argument has a bounded cost. This is the remaining unbounded axis on the same function, and it is worse in kind: an over-large argument now returns a typed refusal a caller can explain, where an over-large precision aborts the process. A reference oracle whose contract is "fail closed with typed, explainable errors" must not have a public input that panics.

**Fact — no caller in the tree reaches it.** Every construction site passes `EnclosurePrecision::binary32_corpus()` (256) or a small literal in a degradation test; the widest in the tree is `EnclosurePrecision::new(12_000)` in `a_precision_the_series_cannot_reach_is_refused`. The exposure is the public boundary, not a live path.

## What to decide

This is a public-boundary question and belongs to Tom, which is why it is filed rather than absorbed:

1. **Validate at construction.** `EnclosurePrecision::new` becomes fallible, or gains a checked constructor beside an infallible one bounded by a governed maximum. This puts the refusal where the value is written rather than where it is used, which is the shape ADR-adjacent validation elsewhere in this crate prefers — and it changes an accepted public `const fn`'s signature.
2. **Refuse in `exp_enclosure`.** A new `EnclosureError` variant with its own stable diagnostic code, refusing a grid the arithmetic cannot express. Keeps `EnclosurePrecision` a plain newtype and needs a diagnostic code decided rather than invented.
3. **Bound the type's domain silently.** Clamp or saturate. Rejected on inspection: a clamp answers a question the caller did not ask at a precision they did not request, which is the shape this module refuses everywhere else.

## Closes when

`exp_enclosure` has no reachable panic on any `(ExactRational, EnclosurePrecision)` pair its public signature admits; the refusal — wherever it is placed — carries a typed error with a stable diagnostic code and a test that watches it fire *and* watches the admitted neighbour; and `exp_enclosure`'s `# Panics` section states what is actually true rather than a bound that does not exist.

Filed at `awaiting-decision` rather than `todo` because every option above moves a public boundary — a `const fn`'s signature or a new governed diagnostic code — and the board must not offer a ticket whose first step is a decision it cannot make. Tom's answer to "What to decide" is what makes it dispatchable.

## Decided — defence in depth, 2026-08-05

Tom decided at the live review (witnessed first-hand by the coordinator): both layers, not either. (1) `EnclosurePrecision` gains a validated construction bound so the overflowing grid width is unrepresentable — the primary repair. (2) The consumption site's `i32` conversion becomes a checked conversion returning the typed `EnclosureError` refusal rather than a panic — the second layer, kept even though the bound makes it unreachable through the validated constructor, because defence in depth is the stated preference. The second layer's watched-failing evidence comes from perturbing the construction bound (the pattern the exp-bound landing used), not from a wildcard test; a check that cannot be demonstrated failing under a stated perturbation does not land. Both surface changes return for acceptance as one delta. Status moves to `todo`: this is now a decided implementation ticket awaiting dispatch.
