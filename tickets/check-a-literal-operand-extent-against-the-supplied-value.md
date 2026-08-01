---
id: check-a-literal-operand-extent-against-the-supplied-value
title: Check a literal operand extent against the value supplied for it
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, correctness, inline-dx]
claimed_from: todo
assignee: worker-check-a-lite
lease_expires_at: 1785559830
---
## Why this exists

**Fact.** `tiler::__private::bind_region` (`crates/tiler/src/expansion.rs`) checks each operand's *rank* and *stored scalar*, and then unifies symbols. It does not compare a declared **literal** extent against the extent the supplied value reports.

**Reproduce.** A region `in a: f32[4], b: f32[4]; out a * b` handed a value whose extents are `[7]` binds without refusal, and `build_result` constructs a `[4]` result — the region's declared shape rather than the operand's. Found while writing `crates/tiler/src/route/tests.rs`, where a rank perturbation had to replace an extent perturbation because the extent one did not fail.

**Inference.** With no dispatch this is a shape claim nobody reads. It stops being harmless the moment an embedded kernel is dispatched against the operand's storage: launch geometry and bounds come from the artifact, which was compiled for `[4]`, while the buffer holds `[7]` elements. `route-an-embedded-artifact-through-a-consumer-storage-seam` is what makes it reachable.

A symbolic axis is already covered, because a symbol's source extent and every obligation are compared. The gap is exactly the literal case, which no obligation names.

## Closes when

`bind_region` refuses a literal extent the supplied value does not report, with a typed refusal naming the operand and axis in the same vocabulary as `BindError::InconsistentExtent`, and a regression test that fails before the fix. The refusal is consumer-visible, so the exact variant and message go to Tom with the change rather than after it.
