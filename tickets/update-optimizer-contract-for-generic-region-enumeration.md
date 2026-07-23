---
id: update-optimizer-contract-for-generic-region-enumeration
title: Update optimizer contract for generic region enumeration
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, optimizer]
claimed_from: todo
assignee: agent-update-optimizer-contract-for-generic-region-enumeration
lease_expires_at: 1784832272
---
Generic region formation replaced the hardcoded serial-Sum recognizer with a deterministic `EnumerateRegionCandidates` stage over an arbitrary verified DAG (merged as `crates/tiler-compiler/src/region.rs`). Two normative passages in the optimizer contracts still describe the superseded bounded recognizer and now contradict the code:

- docs/compiler/fusion-and-scheduling.md lines ~15-22 say the slice "enumerates five singleton candidates, the four-operation pointwise candidate, and one full-region fused candidate from canonical semantic occurrence roles" and calls it "a bounded recognizer, not evidence that hand-enumeration is complete." That is no longer what the code does — enumeration is general over connected convex regions with content-vs-occurrence identity separation, budgets, and an exhaustive-oracle equality check.
- docs/compiler/optimizer.md line ~339 says "the same interfaces may be backed by a trivial region builder for a narrow semantic graph; this staged shortcut is explicit rather than a second optimizer architecture." The shortcut is gone.

Restate both to describe the implemented generic enumerator: connectivity by minimum-member seeding, convexity decided at emission via forward-closure re-reach, unconditional singleton coverage, separate region-content and region-occurrence identity, the five deterministic budgets with typed explain budget-stops, and validation against the exhaustive subset oracle. Keep the distinction the code preserves: enumeration proposes candidates and does not select covers, choose implementations, lower index regions, plan physically, or cost — those remain later stages. Producer duplication stays disabled in the first profile while the oracle retains it as a completeness witness. Do not overstate: this stage is a candidate enumerator, not a cover selector or a public fusion API. Run the full documentation gate before completion.

## Outcome

Both superseded passages now describe the merged generic `EnumerateRegionCandidates` stage (`crates/tiler-compiler/src/region.rs`), and numerical legality's generic-region consumption in `crates/tiler-compiler/src/fusion.rs` was read to confirm the contract no longer names operation roles.

- `docs/compiler/fusion-and-scheduling.md` (status paragraph): replaced the "five singleton candidates, the four-operation pointwise candidate, and one full-region fused candidate ... bounded recognizer" prose with a description of generic connected-convex enumeration over an arbitrary verified DAG — minimum-member seeding (each connected set generated once), emission-time convexity by forward-closure re-reach, unconditional singleton coverage, separate region-content (members renumbered to region-local positions) and region-occurrence (exact graph site in canonical coordinates) identity, the five deterministic budgets (`region_members`, `region_boundary_outputs`, `region_live_values`, `region_candidates_per_seed`, `region_expansions`) with typed explain budget-stops, and exhaustive-subset-oracle validation. It preserves the code's boundaries: enumeration only proposes candidates and selects no cover, chooses no implementation, lowers no index region, plans nothing physical, and costs nothing — those remain the later `prototype-region-cover-enumeration`, `prototype-physical-implementation-frontier`, and `prototype-complete-physical-plan-selection` stages (all still `todo`, linked); producer duplication stays disabled in the first profile while the exhaustive tiny-DAG oracle retains it as a completeness witness; and the stage is a candidate enumerator, not a cover selector or a public fusion API.
- `docs/compiler/optimizer.md` ("Possible memo contract"): replaced the "trivial region builder for a narrow semantic graph" staged-shortcut sentence with a statement that region enumeration is already general over an arbitrary verified DAG (separate content/occurrence identities, typed budget-stops, oracle-checked), and that goal-directed property search over those candidates — cover enumeration, physical-implementation frontiers, and complete physical-plan selection — is the staged future work, not a second optimizer architecture.

Verification: `uv run --locked python scripts/check_repository.py` passed (documentation validate 175 records, 147 pytest, Rust gate); `git diff --check` clean; `tkt guard` verdict ok. Scope stayed within `docs/compiler/**` plus this ticket file.

Reported, not fixed (outside the two named passages): the illustrative `RegionCandidate` schematic in `fusion-and-scheduling.md` (lines 61-70) still lists `semantic_region_id` and `numerical_contract_id`, whereas the merged code splits identity into `content` (which folds in the numerical-contract key) and `occurrence`; and the `optimizer.md` budget list (lines 171-177) includes "8 nondominated implementations per region", which is a forward-looking physical-frontier budget rather than one of the five `DeterministicBudgets` region-formation fields. Both are illustrative or forward-looking rather than strictly false, so they were left unchanged.
