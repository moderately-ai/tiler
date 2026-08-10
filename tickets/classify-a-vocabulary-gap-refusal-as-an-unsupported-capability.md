---
id: classify-a-vocabulary-gap-refusal-as-an-unsupported-capability
title: Classify a vocabulary-gap refusal as an unsupported capability
status: in-progress
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics]
claimed_from: todo
assignee: sol-vocabulary-gap-classification
lease_expires_at: 1786409566
---
## User-visible outcome

A caller whose program no *installed vocabulary* can spell is told `CompileFailureClass::UnsupportedCapability`, which names an action — install a provider, or wait for coverage — rather than `NoFeasiblePlan`, which the public documentation defines as "a hard target rejection".

## The observation, corrected at the current repository boundary

**False historical subject — corrected 2026-08-09.** `rms_norm(value, weight) * value` now compiles and is bit-checked by source anchor `a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit`; it no longer demonstrates an all-vocabulary-decline portfolio. The live staged-family subject is `rms_norm(matmul(a, b), w)`, retained by `tests/staged_family_over_a_materialized_intermediate.rs` under source anchor `staged_over_an_edge`, and it currently reports `NoFeasiblePlan`.

**False retention premise — corrected 2026-08-09.** Complete-plan selection does not retain a complete all-declines cause summary. Hard target rejections survive, while frontier strategy declines and fusion-legality rejections are local to enumeration/trace paths. The distinction therefore cannot be derived after the fact from the current empty portfolio alone.
- The trace is complete and correct — every declined region names its wall — but the *class* a caller switches on says the target rejected a plan, when in fact no target could have accepted one.

**Imprecise generalization — corrected 2026-08-10.** The empty-portfolio classifier is not confined to staged families, but `region-partial-coverage` says the cover grouped occurrences no recognized partition owns. It is a structural cover wall, not evidence that an installed schedule vocabulary lacks the right region. A partial-only portfolio therefore remains `NoFeasiblePlan`; the vocabulary-only classification below permits it only as search noise beside positive evidence from a non-partial `StrategyDeclineCause::UnspellableRegion` wall.

## The question

`NoFeasiblePlan`'s own documentation says "This is a hard target rejection, never an exhausted analysis budget", and `UnsupportedCapability`'s says "The program is valid and no installed capability compiles it... The action is to install a provider or wait for coverage". An empty portfolio whose every rejection is a region-vocabulary wall matches the second exactly. An empty portfolio caused by a *target* rejection — a region the vocabulary spells and the profile refuses — matches the first.

The distinction remains derivable only if planning retains a private, fail-closed cause census while it still has every cover outcome. Classify the no-plan result as a vocabulary gap only when all of these hold:

- enumeration was exhaustive and did not stop on a budget;
- at least one cover was considered;
- at least one complete cover failed solely because every frontier that blocked it declined under a non-partial `StrategyDeclineCause::UnspellableRegion` vocabulary wall;
- every other cover failed only under `UnspellableRegion`; `region-partial-coverage` may occur there as search noise, but a partial-only portfolio is not a vocabulary gap;
- there was no fusion-legality rejection or unknown, boundary disagreement, hard target refusal, silent or mixed frontier, or other structural decline.

**Predicate correction — 2026-08-10.** The earlier wording required every cover to be non-partial, which cannot classify its own live subject. At exact base `b07d269b5ca64605060f7baf70a4d4095be86516`, source anchor `staged_over_an_edge` exhaustively enumerates four covers. One is blocked solely by `region-partial-coverage` over three occurrences; one solely by `region-staged-family-unspellable` over two stages; one by a staged-family wall over one stage plus partial coverage over two occurrences; and one by two staged-family walls over one stage each. The partial walls are alternative groupings the search considered, not the reason the complete staged partition cannot compile. The corrected rule therefore requires at least one cover carrying a non-partial vocabulary wall, permits partial coverage beside or instead of that wall on other all-`UnspellableRegion` covers, and still refuses a portfolio whose only evidence is partial coverage.

Anything else remains `NoFeasiblePlan`. In particular, the existing contraction-permitting mixed-body case must remain a control: its fused candidates resolve fusion legality as `Unknown` under source anchor `unrealized-contraction`, while its surviving covers hit partial coverage, so it is not a pure vocabulary gap.

## Non-goals

Changing `CompileFailureClass`'s variants. Both classes exist and are documented; what is wrong is which one an empty portfolio maps to.

## Closes when

An exhaustive portfolio whose recorded cause census is vocabulary-only reports `UnsupportedCapability`; target, numerical, boundary, and mixed-cause empty portfolios remain `NoFeasiblePlan`, while a budget-stopped search retains its existing `BudgetExceeded` class. Each cause is independently perturbed, and the public class documentation no longer describes `NoFeasiblePlan` as hard-target-only.
