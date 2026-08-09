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

## The observation, corrected at the current repository boundary

**False historical subject — corrected 2026-08-09.** `rms_norm(value, weight) * value` now compiles and is bit-checked by source anchor `a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit`; it no longer demonstrates an all-vocabulary-decline portfolio. The live staged-family subject is `rms_norm(matmul(a, b), a)`, retained by `tests/staged_family_over_a_materialized_intermediate.rs`, and it currently reports `NoFeasiblePlan`.

**False retention premise — corrected 2026-08-09.** Complete-plan selection does not retain a complete all-declines cause summary. Hard target rejections survive, while frontier strategy declines and fusion-legality rejections are local to enumeration/trace paths. The distinction therefore cannot be derived after the fact from the current empty portfolio alone.
- The trace is complete and correct — every declined region names its wall — but the *class* a caller switches on says the target rejected a plan, when in fact no target could have accepted one.

**This is not new behaviour and it is not confined to staged families**; a program whose every cover hits `region-partial-coverage` reaches the same place. What the staged recognizer changed is that the shape became reachable in practice, because such a program used to be refused at the request boundary instead.

## The question

`NoFeasiblePlan`'s own documentation says "This is a hard target rejection, never an exhausted analysis budget", and `UnsupportedCapability`'s says "The program is valid and no installed capability compiles it... The action is to install a provider or wait for coverage". An empty portfolio whose every rejection is a region-vocabulary wall matches the second exactly. An empty portfolio caused by a *target* rejection — a region the vocabulary spells and the profile refuses — matches the first.

The distinction remains derivable only if planning retains a private, fail-closed cause census while it still has every cover outcome. Classify the no-plan result as a vocabulary gap only when all of these hold:

- enumeration was exhaustive and did not stop on a budget;
- at least one cover was considered;
- every cover failed solely because every surviving strategy declined under `StrategyDeclineCause::UnspellableRegion`;
- there was no fusion-legality rejection or unknown, boundary disagreement, hard target refusal, or other structural decline.

Anything else remains `NoFeasiblePlan`. In particular, the existing contraction-permitting mixed-body case must remain a control: its whole cover is numerically illegal and its surviving covers hit partial coverage, so it is not a pure vocabulary gap.

## Non-goals

Changing `CompileFailureClass`'s variants. Both classes exist and are documented; what is wrong is which one an empty portfolio maps to.

## Closes when

An exhaustive portfolio whose recorded cause census is vocabulary-only reports `UnsupportedCapability`; target, numerical, boundary, mixed-cause, and budget-stopped empty portfolios remain `NoFeasiblePlan`. Each cause is independently perturbed, and the public class documentation no longer describes `NoFeasiblePlan` as hard-target-only.
