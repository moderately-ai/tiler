---
id: accept-the-partitioned-write-ownership-proof-boundary
title: Accept the partitioned write-ownership proof boundary
status: awaiting-decision
priority: p1
dependencies: []
related: []
scopes: [contracts/decisions]
shared_scopes: []
paths: []
tags: []
---
## The decision

**Only Tom closes this ticket**; it parks at `awaiting-decision` carrying the exact surface. [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) (merged 2026-08-06, worker commit `f7c09423`) landed a public boundary as a labelled draft under ADR 0075, not self-accepted:

- **Added** `WriteOwnershipProofView::PartitionMember { joint: JointPartitionProofView }` — additive on a `#[non_exhaustive]` enum; the retained `trybuild` pass case already carries a wildcard arm.
- **Added** `pub enum JointPartitionProofView { Interval, Exhaustive { points: u64 } }`, `#[non_exhaustive]` — a separate type so the two joint mechanisms are named and a third becomes a build error at every exhaustive site.
- **Removed** `IndexBuildError::DuplicateOutputTensor` — the one non-additive change; removed rather than repurposed because the name states a rule the contract no longer holds (a second root is now the start of a partition, refused at verification when unsound).
- **Added** three `IndexRegionDiagnostic` variants: `OutputPartitionUncovered`, `OutputPartitionRangesOverlap`, `OutputPartitionDoubleWritten` — additive, each observed firing.

The evidence: the joint obligation is decided exactly (separating-axis disjointness, coverage derived from disjoint volumes) or by one shared-bitset walk; all three refusals watched failing under perturbation; no pinned identity moved (proof forms are outside canonical region identity by construction, verified at `encode_region`); full gate green on the branch (2683 tests). A tested implementation is a concrete draft, not approval of its spelling.
