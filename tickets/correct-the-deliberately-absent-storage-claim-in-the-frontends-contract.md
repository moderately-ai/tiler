---
id: correct-the-deliberately-absent-storage-claim-in-the-frontends-contract
title: Correct the "Deliberately absent" storage claim in the frontends contract
status: done
priority: p2
dependencies: []
related: [correct-two-stale-delivery-spans-in-the-frontends-contract, route-an-embedded-artifact-through-a-consumer-storage-seam]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, frontend, inline-dx, staleness]
---
## Why this exists

Found while correcting the two delivery spans under [`correct-two-stale-delivery-spans-in-the-frontends-contract`](correct-two-stale-delivery-spans-in-the-frontends-contract.md), whose close-condition sweep is scoped to whether a selected macOS family delivers. This span is a different axis, so it was filed rather than absorbed.

[The frontend contract](../docs/integration/frontends.md)'s "Symbol binding and the runtime-value boundary" section ends with **Deliberately absent.**: "No storage access — no pointer, buffer, byte slice, or device object — because nothing dispatches yet and a storage surface with no caller would be an unreviewed boundary."

**Both halves are refuted by the tree at `561dfe0b`.** `crates/tiler/src/value.rs` publishes `RegionOperand`, `RegionRequest`, and `DispatchAdapter`: `RegionRequest::operand(&self, key: &str) -> Option<&[u8]>` is at `value.rs:437` and `RegionRequest::result_mut(&mut self) -> &mut [u8]` at `value.rs:465` — both byte-slice storage access — and `pub trait DispatchAdapter: TensorAdapter` is at `value.rs:517`. The contract's own status paragraph already records why: `route-an-embedded-artifact-through-a-consumer-storage-seam` landed the `tiler::value::DispatchAdapter` boundary on 2026-08-01 and `spikes/runtime/inline-dispatch` reached a completed dispatch on hardware the same day. "Nothing dispatches yet" is therefore false in the same document that asserts it.

**Why this is its own ticket rather than a one-line strike.** The paragraph belongs to a span the contract marks as a reviewed draft — "The exact items below are a reviewed draft under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) and ADR 0074 convention 7 until Tom accepts them". Rewriting it means describing the public boundary `tiler::value` now has: which absences survived, which the dispatch seam replaced, and what the bounded profile still rejects. That needs a full read of `crates/tiler/src/value.rs` and `crates/tiler/src/route.rs`, and the result is a public-boundary description rather than a typo fix.

## Closes when

The **Deliberately absent** paragraph states which absences hold at the commit that lands it, verified by reading `crates/tiler/src/value.rs` in full rather than inferred from the status paragraph; no sentence in that section contradicts the file's own status paragraph on whether a region dispatches; and any claim about a public surface still awaiting Tom's acceptance stays labelled a draft rather than being silently promoted.
