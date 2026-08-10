---
id: decide-whether-stage-one-semantic-exploration-adopts-an-e-graph
title: Decide whether stage-one semantic exploration adopts an e-graph
status: deferred
priority: p3
dependencies: [probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary]
related: [survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature]
scopes: [research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [research, search, equality-saturation, deferred, decision]
---
## User-visible outcome

A decision, with evidence, on whether `ExploreLogicalAlternatives` stops being "at most one whole-program proposal per registered rule" and becomes an e-graph over the semantic algebra alone, with enumeration into contract grouping (formalism stage 2) replacing cost-based extraction.

## Why this exists

**Correction — 2026-08-10.** (a) The user-visible outcome previously said "enumeration into region formation"; that names pipeline stage 4 / physical enumeration. The handoff from a component-level stage-1 e-graph is enumeration into contract grouping (formalism stage 2), matching the Inference below, formalism Part 5D, and the optimizer contract. (b) Extraction complexity was written "NP-hard and (relayed) constant-factor-inapproximable"; as of 2026-08-05 both halves are read at their proofs (formalism Part 2 / elimination B reason 2: `stepp-thesis-ucsd-2011` NP-hardness; `sparse-extraction-oopsla-2024` Section 3 inapproximability). (c) The 2026-08-09 trigger entry named `ORDERED_REASSOCIATE_ADD_F32_RULE` and `ORDERED_REASSOCIATE_MULTIPLY_F32_RULE`; the production consts are `ORDERED_REASSOCIATE_ADD_RULE` and `ORDERED_REASSOCIATE_MULTIPLY_RULE` (identity strings still contain `ordered-reassociate-*-f32.v1`). The 2026-08-05 reproduction using `grep -v test` filters lines not modules and over-reports; prefer `RewriteRuleIdentity::new("tiler` for the four production identities.

**Fact.** [The rewrite-search formalism record](../docs/research/region-search/rewrite-search-formalism.md) Part 5D selects a staged, alternative-retaining search and eliminates equality saturation *as the whole search* on four grounds — the published tractability evidence for schedule-shaped rewrite spaces, NP-hard and constant-factor-inapproximable extraction, the tension with typed feasibility refusals, and e-graph cycles against DAG convexity. It also records that stage 1's job in isolation — hold many equivalent semantic forms compactly, apply local rewrites without choosing, answer "is this form present" — is exactly what an e-graph is good at, and is *not* the schedule space that broke RISE.

**Inference.** Adopting it at stage 1 only keeps the four eliminations outside if three conditions hold: no schedule-space concept enters the e-graph, extraction is replaced by enumeration into stage 2 rather than by a cost-based extractor, and no feasibility or numerical-contract fact lives in an e-class. Those are the design constraints, not open questions.

## What the decision owes

- The tractability measurement from the probe this depends on.
- A per-e-node rule-provenance or on-extraction explanation mechanism, because a saturated e-graph discards the derivation and Tiler's typed alternative identity requires the owning rule origin. The record's Part 6 states this as a real obligation on adoption, not a detail.
- A stated budget contract: e-graph budgets are global (nodes, classes, iterations) and not attributable to a subject, so the stage's budget stop must be reported as a stop on the *stage* carrying an `Unknown` evidence class over its whole output — never as a per-candidate account it cannot produce.
- A read of the FMCAD 2024 successor material already closed under [`close-the-conditional-eqsat-search-request-with-the-fmcad-2024-successor`](close-the-conditional-eqsat-search-request-with-the-fmcad-2024-successor.md) (and the formalism Part 5B re-check against it), since it bears on the contract-multiplicity half of elimination reason 3. The acquisition request `conditional-eqsat-successor-unlocated` is closed; the owed action is reading the already-vendored re-check, not locating the paper.

## Trigger check log

- 2026-08-05 — **not fired.** Two of the three triggers are unmet and the third is unmeasured. Stage 3 carries two algebraic rules, well under the ~dozen at which per-rule whole-program proposals stop working; no registered rule is non-terminating under repeated application (`AlgebraicRuleConfiguration` in `crates/tiler-compiler/src/normalize.rs` has exactly two `bool` fields and each rule contributes at most one proposal); and the dependency probe has not run. Reproduce the rule count: `grep -rn 'RewriteRuleIdentity::new("tiler' crates/ --include='*.rs'`.
- 2026-08-09 — **not fired.** The source-acquisition gap named in this decision has closed through `close-the-conditional-eqsat-search-request-with-the-fmcad-2024-successor`, but the required tractability probe remains `deferred`. Production rewrite identities are still the canonical baseline, common-subexpression normalization, and the two ordered reassociation rules; the flash-class additions remain research proposals rather than a registered semantic rewrite set. Without the dependency measurement there is no evidence on which to choose an e-graph. Recheck the dependency status and the production anchors `COMMON_SUBEXPRESSION_RULE`, `ORDERED_REASSOCIATE_ADD_RULE`, and `ORDERED_REASSOCIATE_MULTIPLY_RULE` in `crates/tiler-compiler/src/rewrite.rs`.
