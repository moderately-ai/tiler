---
id: reconcile-optimizer-schematics-with-implemented-identity-and-budgets
title: Reconcile optimizer schematics with implemented identity and budgets
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, optimizer]
claimed_from: todo
assignee: agent-reconcile-optimizer-schematics-with-implemented-identity-and-budgets
lease_expires_at: 1784834740
---
Two illustrative schematics in the optimizer contracts predate the merged region-formation code and now describe a shape the implementation no longer uses. Found during the region-enumeration contract update, correctly left unfixed there because both are conceptual sketches rather than the two passages that ticket owned, and neither is strictly false today.

- docs/compiler/fusion-and-scheduling.md (the `RegionCandidate` schematic, around lines 61-70) still lists `semantic_region_id` and `numerical_contract_id` as candidate fields, whereas `crates/tiler-compiler/src/region.rs` splits candidate identity into `content` (which folds the numerical-contract key into its bytes) and `occurrence` (the canonical graph site). A reader could take the sketch for the real field set.
- docs/compiler/optimizer.md (the deterministic-budget list, around lines 171-177) names six budgets including "8 nondominated implementations per region", while `DeterministicBudgets` currently has exactly the five `region_*` fields. The sixth is a forward-looking physical-implementation-frontier budget whose stage is not yet implemented.

Neither should be flipped mechanically. For the RegionCandidate sketch, either annotate it as conceptual and point at the content/occurrence identity the code uses, or update it to the real split. For the budget list, keep the forward-looking entry but mark which budgets exist today versus which belong to the not-yet-implemented frontier stage, so the list stops reading as a single implemented set. This is best done when the physical-implementation-frontier stage lands, since that is what makes the sixth budget real and the RegionCandidate downstream fields concrete; until then, the minimal annotation is enough. Run the full documentation gate before completion.
