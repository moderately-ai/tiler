---
id: convert-the-remaining-accuracy-predicate-shapes-to-a-relative-bound
title: Convert the remaining accuracy predicate shapes to a relative bound
status: deferred
priority: p3
dependencies: []
related: [expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate, connect-certified-rounding-error-bounds-to-rewrite-permissions, derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, accuracy, compiler]
---
## User-visible outcome

A registered accuracy requirement stated as an absolute bound, an additive absolute-plus-relative bound, or a Boolean combination yields a numeric relative accuracy instead of a typed refusal, so a parametric rewrite bound is not blocked by the shape a contract happened to be written in.

## Why this exists

**Fact.** `relative_bound_of_predicate` (`crates/tiler-compiler/src/target/accuracy.rs`) converts exactly two of the six shapes `AccuracyPredicateView` admits: `Ulp` under `tiler::ulp-reference-gap@1`, and `Relative`. The other four refuse by name through `RelativeAccuracyRefusalReason::UnconvertiblePredicate`, and `a_predicate_with_no_sound_relative_conversion_refuses_by_name` counts that population at four.

**Fact.** No registered contract states any of the four. `required_elementary_accuracy` has three arms; two are `BoundedPiecewise` with a single `Ulp` clause and one is `Faithful`. So the refusals cost nothing today and the conversions would have no caller.

**Inference — two of the four are blocked on evidence and two on an unforced choice, which is why they are one ticket rather than four.** `Absolute(t)` and `AbsoluteRelative(a, q)` convert only against a proved *lower* bound on the reference magnitude — `t/|r|_min` and `q + a/|r|_min` — which is a clause-level `ReferenceResultConstraint::magnitude` that no registered contract states; the conversion is arithmetic once the proof exists and unsound without it. `AllOf` and `AnyOf` are different: a conjunction admits *any* member as a sound bound, so the choice between the tightest member and a looser member carrying no subnormal precondition is a real trade, and nothing has a caller to decide it. A disjunction has no such freedom — only the weakest member is guaranteed — but its floor is then the union of its members' floors, which is the same representational question seen from the other side.

## What this ticket must produce

- The conversion for `Absolute` and `AbsoluteRelative` from a clause's own proved magnitude lower bound, refusing when no such proof exists rather than inferring one from the input domain.
- The `AllOf` and `AnyOf` composition rule, with the tightness-versus-precondition trade decided against a stated caller rather than by preference, and the resulting `RelativeAccuracyDomain` derived rather than asserted.
- A watched-failing check per added arm, and a population count that moves when the closed predicate vocabulary widens.

## Non-goals

Widening `RelativeAccuracyDomain` into a general value-domain predicate; deciding any numerical permission; changing the requirement-side direction the query already takes.

## Trigger

Fires when a registered accuracy contract states one of the four shapes for an operation a parametric bound must be priced against — that is, when `required_elementary_accuracy` gains an arm whose clause predicate is not a governed `Ulp` or a `Relative`, **or** when a rule consuming the numeric accuracy is refused by `accuracy.elementary.unconvertible-predicate`.

## Trigger check log

- 2026-08-06 — not fired. Every registered requirement converts today. Reproduce with `cargo nextest run -p tiler-compiler -E 'test(/relative_accuracy|unit_roundoff/)'`, whose `the_registered_softmax_accuracy_is_twenty_four_unit_roundoffs`, `the_two_exponentials_yield_one_relative_accuracy`, and `the_faithful_normalization_requirement_gives_two_unit_roundoffs` cover all three registered families and none reaches a refusal; and no consumer exists, because `elementary_relative_accuracy` has no non-test caller.
- 2026-08-09 — **not fired.** `elementary_relative_accuracy` still has no non-test caller, every registered requirement uses the convertible ULP/relative shapes, and no rule reports `accuracy.elementary.unconvertible-predicate`. The additional BF16 and conformance work did not add one of the four predicate shapes this ticket owns.
