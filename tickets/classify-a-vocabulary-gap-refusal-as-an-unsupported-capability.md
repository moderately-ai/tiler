---
id: classify-a-vocabulary-gap-refusal-as-an-unsupported-capability
title: Classify a vocabulary-gap refusal as an unsupported capability
status: in-progress
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics]
claimed_from: todo
assignee: sol-vocabulary-gap-classification
lease_expires_at: 1786409566
---
## User-visible outcome

A caller whose program no *installed vocabulary* can spell is told `CompileFailureClass::UnsupportedCapability`, which names an action — install a provider, or wait for coverage — rather than `NoFeasiblePlan`, which retains hard target refusals and conservative mixed or structural empty portfolios whose causes do not establish a pure capability gap.

## The observation, corrected at the current repository boundary

**False historical subject — corrected 2026-08-09.** `rms_norm(value, weight) * value` now compiles and is bit-checked by source anchor `a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit`; it no longer demonstrates an all-vocabulary-decline portfolio. The live staged-family subject is `rms_norm(matmul(a, b), w)`, retained by `tests/staged_family_over_a_materialized_intermediate.rs` under source anchor `staged_over_an_edge`. Its current class depends on the stated numerical contract: `STRICT_F32` and `FLUSH_SUBNORMALS_TO_ZERO_F32` isolate the vocabulary census and report `UnsupportedCapability { rule: "region-vocabulary" }`; `RELAXED_F32`, `REASSOCIATE_F32`, and `FLUSH_AND_REASSOCIATE_F32` also retain fusion-legality `Unknown` and therefore remain `NoFeasiblePlan`.

**False retention premise — corrected 2026-08-09.** Complete-plan selection does not retain a complete all-declines cause summary. Hard target rejections survive, while frontier strategy declines and fusion-legality rejections are local to enumeration/trace paths. The distinction therefore cannot be derived after the fact from the current empty portfolio alone.
- The trace is complete and correct — every declined region names its wall — but the *class* a caller switches on says the target rejected a plan, when in fact no target could have accepted one.

**Imprecise generalization — corrected 2026-08-10.** The empty-portfolio classifier is not confined to staged families, but `region-partial-coverage` says the cover grouped occurrences no recognized partition owns. It is a structural cover wall, not evidence that an installed schedule vocabulary lacks the right region. A partial-only portfolio therefore remains `NoFeasiblePlan`; the vocabulary-only classification below permits it only as search noise beside positive evidence from a non-partial `StrategyDeclineCause::UnspellableRegion` wall.

## The question

`NoFeasiblePlan`'s current documentation includes hard target rejections and conservative mixed or structural planning failures whose causes do not establish a pure capability gap. `UnsupportedCapability` says the program is valid but no installed capability compiles it, and directs the caller to install a provider or wait for coverage. An empty portfolio whose complete census establishes only supported-but-unspellable region vocabulary matches the second. An empty portfolio caused by a *target* rejection — a region the vocabulary spells and the profile refuses — remains one case of the first, as do mixed and structural empty portfolios. `BudgetExhausted` remains distinct from both because a bounded search proves neither conclusion.

The distinction remains derivable only if planning retains a private, fail-closed cause census while it still has every cover outcome. Classify the no-plan result as a vocabulary gap only when all of these hold:

- enumeration was exhaustive and did not stop on a budget;
- at least one cover was considered;
- at least one complete cover failed solely because every frontier that blocked it declined under a non-partial `StrategyDeclineCause::UnspellableRegion` vocabulary wall;
- every other cover failed only under `UnspellableRegion`; `region-partial-coverage` may occur there as search noise, but a partial-only portfolio is not a vocabulary gap;
- there was no fusion-legality rejection or unknown, boundary disagreement, hard target refusal, silent or mixed frontier, or other structural decline.

**Predicate correction — 2026-08-10.** The earlier wording required every cover to be non-partial, which cannot classify its own live subject. At exact base `b07d269b5ca64605060f7baf70a4d4095be86516`, source anchor `staged_over_an_edge` exhaustively enumerates four covers. One is blocked solely by `region-partial-coverage` over three occurrences; one solely by `region-staged-family-unspellable` over two stages; one by a staged-family wall over one stage plus partial coverage over two occurrences; and one by two staged-family walls over one stage each. The partial walls are alternative groupings the search considered, not the reason the complete staged partition cannot compile. The corrected rule therefore requires at least one cover carrying a non-partial vocabulary wall, permits partial coverage beside or instead of that wall on other all-`UnspellableRegion` covers, and still refuses a portfolio whose only evidence is partial coverage.

**Contract-alignment review correction — 2026-08-10.** Independent exact-hash review of `238030982878f65e082b85adbbb18216e7fdb24c` found that four present-tense passages still repeated the retired hard-target-only definition: optimizer anchor `never as NoFeasiblePlan, which is a hard target rejection`, pipeline-test anchor `rather than NoFeasiblePlan, whose contract is that it`, and the two budget-coverage passages anchored by `public surface documents as`. Current source authority is `CompileFailureClass::NoFeasiblePlan` anchor `This includes hard target rejections and mixed or structural planning`, together with pipeline mappings `portfolio-empty-with-vocabulary-gap`, `portfolio-empty-after-budget-stop`, and `portfolio-empty-without-target-rejection`. The repaired prose now keeps conservative mixed or structural empty portfolios in `NoFeasiblePlan`, hard target refusal as one case, pure supported-but-unspellable populations in `UnsupportedCapability`, and budget exhaustion distinct. Scope `contracts/optimizer` was added because `ticketsplease.toml` maps `docs/compiler/**` there; this is scheduling metadata, not a new contract boundary. No historical quotation needed removal: the repaired passages were live current-surface claims, while dated historical corrections remain explicitly labelled provenance.

**Residual-population review correction — 2026-08-10.** Independent exact-hash review of `dea132879694a9ddd65c863b39c00f2892c25ce7` found three current-surface records that still collapsed the contract-dependent residual: this ticket's anchor `and it currently reports NoFeasiblePlan`, the related done ticket's anchors `recognized then stops without a scheduled-region spelling` and `The vocabulary-gap failure class remains misbucketed`, and the recognized-chain header anchor `still refuses NoFeasiblePlan` beside `the outcome is structural, and no permission`. The authority is the exhaustive five-row assertion under `a_staged_family_over_an_edge_is_recognized_and_stops_at_the_region_vocabulary`: strict and flush-only isolate `UnsupportedCapability { rule: "region-vocabulary" }`, while all three reassociation-permitting contracts add fusion-legality `Unknown` and remain `NoFeasiblePlan`. The related ticket's done outcome and the recognized-chain file's dated 2026-08-08 measurement remain historical evidence; only their live current-state conclusions were repaired.

**Residual-ticket review correction — 2026-08-10.** Independent exact-hash review of `1c387de0b80ac15e2f072e46272a096a4cef095c` found the same collapsed population in two ticket-only current records: the schedule ticket's anchor `the compilation refuses under NoFeasiblePlan` and the deferred depth ticket's Trigger anchor `still refuses NoFeasiblePlan`. Both now state the exhaustive partition: strict and flush-only report `UnsupportedCapability { rule: "region-vocabulary" }`; the three reassociation-permitting contracts add fusion-legality `Unknown` and remain `NoFeasiblePlan`. The two-edge wall is unchanged — one scheduled stage must bind the external operand edge and its law-internal handed value — and the deferred trigger remains not fired because no scheduled region can yet bind both. Its explicitly dated 2026-08-08/09 measurements remain historical provenance rather than current summaries.

**Gate carry — 2026-08-10.** The fresh exact-tip `make full` at `1c387de0b80ac15e2f072e46272a096a4cef095c` carries because this follow-up changes only `tickets/**`, touching none of the gate-invalidating paths named by repository policy. `make citations`, `tkt lint`, `git diff --check`, and the exact-base `tkt guard` are rerun for this ticket-only delta.

Anything else remains `NoFeasiblePlan`. In particular, the existing contraction-permitting mixed-body case must remain a control: its fused candidates resolve fusion legality as `Unknown` under source anchor `unrealized-contraction`, while its surviving covers hit partial coverage, so it is not a pure vocabulary gap.

## Non-goals

Changing `CompileFailureClass`'s variants. Both classes exist and are documented; what is wrong is which one an empty portfolio maps to.

## Closes when

An exhaustive portfolio whose recorded cause census is vocabulary-only reports `UnsupportedCapability`; target, numerical, boundary, and mixed-cause empty portfolios remain `NoFeasiblePlan`, while a budget-stopped search retains its existing `BudgetExceeded` class. Each cause is independently perturbed, and the public class documentation no longer describes `NoFeasiblePlan` as hard-target-only.
