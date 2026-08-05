---
id: route-a-zero-extent-program-through-candle-metal-storage
title: Route a zero-extent program through Candle Metal storage
status: in-progress
priority: p3
dependencies: [prototype-candle-metal-adapter]
related: []
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [candle, runtime]
claimed_from: todo
assignee: agent-zero-extent
lease_expires_at: 1785934520
---
## User-visible outcome

A Candle user can run the `empty-domain` member of the serial-Sum matrix — a reduction over zero contributors, whose result is the reduction's identity element rather than a sum — through the Tiler adapter, or gets a typed refusal that names the limitation instead of a Candle allocator error.

## Why this is open

**Measurement — macOS 27.0 (26A5388g), arm64, Apple M4 Max, Candle 0.11.0, 2026-08-01.** `cargo run -p tiler-prototype-candle -- --artifact /tmp/serial-sum.tiler` reports both `empty-domain` members as `NOT ROUTABLE through Candle Metal storage`. Building the input tensor fails before any Tiler code runs: `Tensor::from_vec(vec![], (1, 0), &metal_device)` returns `Metal error Failed to create metal resource: Buffer`, because Candle's allocator sizes the request as `element_count * dtype.size_in_bytes()` and `newBufferWithLength:options:` returns nil at length zero.

The limitation is therefore upstream of every refusal the adapter owns — there is no Candle tensor of that shape to preflight — and the two `selected` and `materialized` members are excluded from the proof's agreement count and named in its output rather than silently skipped. `docs/integration/candle.md`'s storage-layout contract lists zero-sized views as something the adapter must account for, so this is a gap against the contract and not a case outside it.

## Activation trigger

Either of these, re-checked at the revision the workspace then resolves:

- Candle's Metal allocator admits a zero-length allocation (or rounds one up to a minimum) so a zero-element tensor exists; or
- Tiler decides the adapter should synthesize the storage itself — a one-byte placeholder bound at a zero-length accessible range — which is a decision about whether an adapter may allocate storage a caller's tensor does not have, not a refactor.

## Closes when

- The `empty-domain` members either route and agree with the producer's recorded reference evaluation, or are refused by a typed `TensorRefusal` that names the zero extent, with that refusal watched failing.
- The proof's summary line reports them in the same population as every other member rather than as an excluded count.
- If the placeholder route is taken, the accessible range bound to the kernel is still zero, so a kernel that read the placeholder would be reading outside the range the artifact declares.
