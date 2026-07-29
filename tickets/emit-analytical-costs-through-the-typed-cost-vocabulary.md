---
id: emit-analytical-costs-through-the-typed-cost-vocabulary
title: Emit analytical costs through the typed cost vocabulary
status: done
priority: p2
dependencies: []
related: [model-the-eight-unmodelled-cost-components, calibrate-device-cost-models]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, cost-model, explain]
---
## User-visible outcome

A reader of the explain trace can tell a **measured-exact** cost from a **modelled bound** at the type level, and a calibration pass can trust what it compares against: today every analytical cost is stamped `proven`/`checked-invariant`, so a bound that explicitly does not model cache reuse renders with the same evidence class as an exact count. This is the prerequisite `calibrate-device-cost-models` consumes.

Every analytical component cost is written into the explain trace as `PredicateAssessment::proven(..., EvidenceBasis::CheckedInvariant)` via `record_count_step` (`pipeline/trace.rs`, `record_analytical_costs`), so a `Bounded` estimate whose own doc says it does not model cache reuse is stamped a checked invariant, `Exact` and `Bounded` share one evidence basis (distinguishable only by a `.low`/`.high` key suffix), and a Bytes-unit value is carried under `FactValue::Count` while `FactValue::Bytes` exists. The typed cost event `ExplainEvent::CostAssessment { model, basis, terms, disposition }` exists and the structural model already uses it; the analytical model bypasses it.

## Why the cheap fix is impossible (verified, not assumed)

An adversarial verification pass established the fallback \"just change the basis\" cannot be done: `PredicateAssessment::proven` and `disproved` reject any basis outside `NormativeGuarantee | CheckedInvariant | SoundProof | ExhaustiveFinite` with `ExplainError::EvidenceEscalation` (pinned by an existing test), so `Assumption`/`Empirical` are admissible only on `CostAssessment` events. And `ExplainEvent::Check` is rejected outright at `ExplainStage::Costing`, so the stage complaint is a consequence of the event choice rather than independently fixable.

## What the real fix requires

- Extend `Quantity` (currently `Count|Bytes|Threads|Bindings`) with `Operations`, `Registers`, `Nanoseconds` so every `CostComponent::unit` has a typed carrier.
- Extend `CostDisposition` (currently `Retained|Dominated|HigherCost` — all pruning verdicts) with a non-pruning disposition (e.g. `Reported`), because analytical costs never enter dominance and emitting a pruning verdict would assert a Pareto outcome this model never computes — a new false claim, not a fix.
- Emit one `CostAssessment` per plan under `CostModelKey::new(ANALYTICAL_MODEL_KEY)` with `CostTerm` per component, basis `CheckedInvariant` for `Exact` values and `Assumption` for `Bounded` ones, and carry the four frontier rejection reason codes (`UnregisteredCall`, `MalformedBinding` fault, `CallNotAdmissible` reasons) into typed records while in this vocabulary — today only a per-role rejected-count reaches explain and the reasons live solely on the in-memory frontier.
- Move `CostValue::class()` from dead code to the emission (its allow reason names this ticket).

## Scope boundary

No reachable consumer misreads today — `ExplainEvent` is `pub(crate)` and the only public surface is `ExplainReport::render()`, documented as not a parse target — so this is reporting-fidelity debt, not a live wrong answer. It becomes load-bearing the moment `calibrate-device-cost-models` compares device measurements against these records, which is why it must land first.

## Closes when

- Analytical costs appear as `CostAssessment` records with truthful bases and typed quantities; the census moves once, in this change, with the mechanism named.
- Frontier rejection reasons reach explain as typed records.
- No `PredicateAssessment` in the trace claims `CheckedInvariant` for a modelled bound.
- The retained plan set is unchanged: nothing here enters dominance.

## Graph maintenance

- **Read the verified analysis in the body first** — the cheap fix was *proven impossible* (constructor rejects the basis; event rejected at the stage), so if you find yourself changing only `EvidenceBasis` at one call site, you are re-walking a refuted path.
- **When the `Quantity` and `CostDisposition` extensions land**: they touch `explain.rs`'s validation matrix — check `validate` accepts the new disposition at `Costing` and *only* there, and add the rejection test alongside the existing `EvidenceEscalation` pin.
- **The census will move exactly once, in this change.** Update `every_wired_authority_emits_its_typed_explain_records` in the same commit with the mechanism named in its comment; if it moves again later, that is a finding.
- **When done**: update `calibrate-device-cost-models` to state its input now exists (it currently gates on this), remove the `dead_code` allow on `CostValue::class` (its reason names this ticket), and update `model-the-eight-unmodelled-cost-components`' note that evidence classes were collapsed in output.
- **If you discover the frontier rejection reasons cannot ride this vocabulary** (they are refusals, not costs): file a separate ticket for typed rejection records rather than widening `CostAssessment` to carry them — say why on both.

## Implemented outcome

The seven modelled analytical components now emit through `CostAssessment` with typed quantities and the non-pruning `Reported` disposition. Each plan emits the minimum two records because an assessment carries one evidence basis for all its terms: exact values and the exact unmodelled-component count share `CheckedInvariant`, while both endpoints of the memory-traffic bound share `Assumption`. Combining them into one record would either overstate the bound or understate the exact derivations. The component keys carry `CostValue::class()`, so exact and bounded values remain distinguishable without inferring evidence from `.low` and `.high`.

Opaque-call refusals remain separate from costs and are now implemented by `emit-typed-opaque-call-frontier-rejection-records`. That work preserves the feasibility authority's exact quantities, dtype-qualified numerical unhonourability, complete proposal identity, and typed intrinsic/deferred causes in dedicated explain records; `CostAssessment` remains exclusively cost evidence.
