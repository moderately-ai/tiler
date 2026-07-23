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
