---
id: close-remaining-adr-status-drift
title: Close the remaining ADR status drift
status: todo
priority: p2
dependencies: []
related: []
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions]
---
The coherence sweep bumped ADR 0006, ADR 0018, and numerical-semantics.md from `not-started` to `partial` after reading their implementations, and deliberately stopped there rather than silently widening accepted-ADR maturity metadata beyond the audited set. Two further ADRs carry the same class of drift and were reported with evidence:

- ADR 0009 (resolve numerical typing before semantic optimization) remains `not-started` although per-value `ResolvedValueType` with registry-resolved operation signatures is implemented and tested in `tiler-ir`;
- ADR 0024 (round-to-nearest ties-to-even for initial arithmetic) remains `not-started` although `binary32-round-to-nearest-ties-even` is registered as durable identity on the standard f32 arithmetic operations in `crates/tiler-ir/src/semantic/registry.rs`, with reference-evaluator coverage.

Read each implementation before bumping; do not bump on this ticket's say-so. While in this scope, audit every remaining accepted ADR's `implementation_status` against the crates so the corpus stops drifting one audit at a time, and report any ADR whose status cannot be justified either direction.
