---
id: reconcile-optimizer-schematics-with-implemented-identity-and-budgets
title: Reconcile optimizer schematics with implemented identity and budgets
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, optimizer]
---
Two illustrative schematics in the optimizer contracts predate the merged region-formation code and now describe a shape the implementation no longer uses. Found during the region-enumeration contract update, correctly left unfixed there because both are conceptual sketches rather than the two passages that ticket owned, and neither is strictly false today.

- docs/compiler/fusion-and-scheduling.md (the `RegionCandidate` schematic, around lines 61-70) still lists `semantic_region_id` and `numerical_contract_id` as candidate fields, whereas `crates/tiler-compiler/src/region.rs` splits candidate identity into `content` (which folds the numerical-contract key into its bytes) and `occurrence` (the canonical graph site). A reader could take the sketch for the real field set.
- docs/compiler/optimizer.md (the deterministic-budget list, around lines 171-177) names six budgets including "8 nondominated implementations per region", while `DeterministicBudgets` currently has exactly the five `region_*` fields. The sixth is a forward-looking physical-implementation-frontier budget whose stage is not yet implemented.

Neither should be flipped mechanically. For the RegionCandidate sketch, either annotate it as conceptual and point at the content/occurrence identity the code uses, or update it to the real split. For the budget list, keep the forward-looking entry but mark which budgets exist today versus which belong to the not-yet-implemented frontier stage, so the list stops reading as a single implemented set. This is best done when the physical-implementation-frontier stage lands, since that is what makes the sixth budget real and the RegionCandidate downstream fields concrete; until then, the minimal annotation is enough. Run the full documentation gate before completion.

## Outcome

Both schematics reconciled against the merged region-formation code at base commit `a90ccfebb21e9f6b56e1db5da7ddd648f0d0f64f`; edits confined to `docs/compiler/`.

**RegionCandidate sketch (`docs/compiler/fusion-and-scheduling.md`, region-representation section).** Chose to update the two stale identity fields rather than only annotate, because the content/occurrence identity split is merged, implemented code (`crates/tiler-compiler/src/region.rs`, `RegionCandidate` fields `content: RegionContentIdentity` and `occurrence: RegionOccurrenceIdentity`), not forward-looking, and this document's own intro already commits to it. Replaced `semantic_region_id` and `numerical_contract_id` with `region_content_identity` and `region_occurrence_identity`, and added a short paragraph noting the box is a conceptual sketch, that region-content folds the numerical-contract key into its canonical bytes (verified at `encode_content`, which encodes `numerical_contract.key`), that region-occurrence additionally pins the exact graph site, and that no standalone numerical-contract-id field exists. Left the other sketch fields (`member_operations`, etc.) untouched as out of scope.

**Deterministic-budget list (`docs/compiler/optimizer.md`, bounded-hierarchical-search section).** Kept all six budgets but partitioned them. Marked the five implemented today as the `region_*` fields of `DeterministicBudgets`, naming each field: 32 semantic occurrences per region (`region_members`), 8 boundary outputs (`region_boundary_outputs`), 64 live boundary/internal values (`region_live_values`), 32 candidates per seed (`region_candidates_per_seed`), 10,000 candidate expansions per request (`region_expansions`) — all five names and default values verified against `request.rs`. Marked the sixth, "8 nondominated implementations per region", as forward-looking: it bounds the per-region physical-implementation frontier, a stage not yet implemented (no corresponding struct field), and becomes real only when that stage lands. Did not invent a field name or implementation status for it.

Gate, `git diff --check`, and `tkt guard` all run and passed before completion. No `crates/`, `docs/architecture.md`, `docs/ir.md`, or `docs/decisions/**` files were touched; `docs/operation-extensions.md` and `crates/tiler-compiler` belong to other in-flight tickets and were left untouched.
