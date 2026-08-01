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
---
## Why this exists

**Fact.** `tiler::__private::bind_region` (`crates/tiler/src/expansion.rs`) checks each operand's *rank* and *stored scalar*, and then unifies symbols. It does not compare a declared **literal** extent against the extent the supplied value reports.

**Reproduce.** A region `in a: f32[4], b: f32[4]; out a * b` handed a value whose extents are `[7]` binds without refusal, and `build_result` constructs a `[4]` result — the region's declared shape rather than the operand's. Found while writing `crates/tiler/src/route/tests.rs`, where a rank perturbation had to replace an extent perturbation because the extent one did not fail.

**Inference.** With no dispatch this is a shape claim nobody reads. It stops being harmless the moment an embedded kernel is dispatched against the operand's storage: launch geometry and bounds come from the artifact, which was compiled for `[4]`, while the buffer holds `[7]` elements. `route-an-embedded-artifact-through-a-consumer-storage-seam` is what makes it reachable.

A symbolic axis is already covered, because a symbol's source extent and every obligation are compared. The gap is exactly the literal case, which no obligation names.

## Closes when

`bind_region` refuses a literal extent the supplied value does not report, with a typed refusal naming the operand and axis in the same vocabulary as `BindError::InconsistentExtent`, and a regression test that fails before the fix. The refusal is consumer-visible, so the exact variant and message go to Tom with the change rather than after it.

## Outcome

**Fact.** The facts did not carry literal extents, so the fix spans emission. `OperandFacts::rank: usize` is replaced by `OperandFacts::extents: &'static [OperandExtent]`, where `OperandExtent` is `Literal(u64) | Symbolic`; the rank a value must have is that slice's length, so the two facts cannot disagree. `tiler_macros::binding` lowers each `DeclaredAxis` into it and emits `extents: &[…]` in `facts_source`; `BoundOperand::rank` became `BoundOperand::extents`. `OperandExtent::Symbolic` is payload-free on purpose: `RegionFacts::symbols` stays the single authority for which axes share an extent.

**Fact.** `bind_region` checks literal extents per operand, after rank and stored scalar and before unification, and refuses with the new `BindError::LiteralExtentMismatch { axis: OperandAxis, declared: u64, actual: u64 }`. Rendered: ``tiler.bind.literal-extent-mismatch: `b` axis 1 is declared with extent 4 and the supplied value reports 5``.

**Public surface changed** (`tiler` is a reviewed draft boundary, nothing here self-accepted): the new `__private::OperandExtent` enum and its two variants; the changed `__private::OperandFacts` field; the new `value::BindError::LiteralExtentMismatch` variant, its three fields, and its rendered text. `BindError` is `#[non_exhaustive]`, so the variant lands additively for a matching consumer.

**Measurement (watched failing, at base `c142991`, before the fix).** Two probes phrased against the pre-fix facts both failed, and the route one reproduced the ticket exactly — ``the second operand … : Buffer { scalar: F32, extents: [4] }`` for a `[7]` value. After the fix, neutralizing the comparison in `bind_region` failed all three checks that depend on it: `runtime_value_adapter::a_literal_extent_the_supplied_value_does_not_report_is_refused`, `route::tests::the_region_contract_is_checked_before_the_artifact`, and the `inline_region_executes` trybuild fixture, whose panic showed a `[4]` buffer built from a `[7]` value through the real macro.

**Fact.** `route/tests.rs` now perturbs the operand's literal extent, the perturbation the ticket records as having been unavailable; its rank-perturbation workaround and the note pointing at this ticket are gone. `inline_region_executes` gained the end-to-end case through `tensor!` beside the rank one, and `region::tests` asserts that a declared `f32[4]` reaches the emitted operand facts. The three hand-written compile-pass `FACTS` fixtures were updated and are still byte-compared by the macro crate.

**Fact.** `docs/integration/frontends.md` enumerates this vocabulary and is now incomplete; it is `contracts/integrations`, outside this ticket's scopes, so the correction is `name-the-operand-extent-facts-in-the-frontend-integration-contract`.
