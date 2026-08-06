---
id: classify-a-vocabulary-gap-refusal-as-an-unsupported-capability
title: Classify a vocabulary-gap refusal as an unsupported capability
status: todo
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics]
---
## User-visible outcome

A caller whose program no *installed vocabulary* can spell is told `CompileFailureClass::UnsupportedCapability`, which names an action — install a provider, or wait for coverage — rather than `NoFeasiblePlan`, which the public documentation defines as "a hard target rejection".

## The observation, and where it came from

Measured while landing the recognizer half of [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md), on `rms_norm(value, weight) * value`:

- Every cover the search enumerates contains at least one region the scheduled-region vocabulary cannot spell, each declining by name under `region-staged-family-unspellable`.
- The portfolio is therefore empty, `enumerate_complete_plans` reports `SelectionError::Structure { rule: "no-complete-plan" }`, and `session::class_of` maps it to `CompileFailureClass::NoFeasiblePlan`.
- The trace is complete and correct — every declined region names its wall — but the *class* a caller switches on says the target rejected a plan, when in fact no target could have accepted one.

**This is not new behaviour and it is not confined to staged families**; a program whose every cover hits `region-partial-coverage` reaches the same place. What the staged recognizer changed is that the shape became reachable in practice, because such a program used to be refused at the request boundary instead.

## The question

`NoFeasiblePlan`'s own documentation says "This is a hard target rejection, never an exhausted analysis budget", and `UnsupportedCapability`'s says "The program is valid and no installed capability compiles it... The action is to install a provider or wait for coverage". An empty portfolio whose every rejection is a region-vocabulary wall matches the second exactly. An empty portfolio caused by a *target* rejection — a region the vocabulary spells and the profile refuses — matches the first.

So the distinction is derivable from the retained rejections rather than needing a new signal, and the work is to derive it rather than to widen a class:

- separate the two causes at `enumerate_complete_plans`, where both the declined strategies and the target rejections are already held;
- keep `InvalidCompilerOutput` unreachable from either — a vocabulary gap is not a defect;
- and make the new mapping fail deliberately before trusting it, since both classes are already produced by other paths.

## Non-goals

Changing `CompileFailureClass`'s variants. Both classes exist and are documented; what is wrong is which one an empty portfolio maps to.

## Closes when

An empty portfolio whose every rejection is a region-vocabulary wall reports `UnsupportedCapability`, one whose cause is a target rejection still reports `NoFeasiblePlan`, both are asserted against programs that produce them, and the public class documentation matches what each is now produced for.
