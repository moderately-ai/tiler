---
id: route-a-zero-extent-program-through-candle-metal-storage
title: Route a zero-extent program through Candle Metal storage
status: done
priority: p3
dependencies: [prototype-candle-metal-adapter]
related: [decide-whether-the-candle-adapter-may-synthesize-zero-extent-storage]
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [candle, runtime]
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

## Outcome (2026-08-05) — closed on the typed-refusal branch; no activation trigger fired

**Which close condition was met.** The first bullet's *second* branch: the two `empty-domain` members are refused by a typed `TensorRefusal` that names the zero extent. They do not route, and neither activation trigger above has fired — Candle 0.11.0 still refuses a zero-length allocation, and the placeholder route was deliberately not taken. The third bullet is conditional on a route this ticket did not take and stays untested.

**Measurement — macOS 27.0 (26A5388g), arm64, Apple M4 Max, Candle 0.11.0 (crates.io, `Cargo.lock` checksum `5ecb245…`), 2026-08-05, base `561dfe0`.** The recorded failure reproduces unchanged: `cargo run -p tiler-prototype-candle -- --artifact <base>` reported both `empty-domain` members as `NOT ROUTABLE through Candle Metal storage`, with `Metal error Failed to create metal resource: Buffer`.

**What landed.** `TensorRefusal::ZeroExtentInterface` is decided in `bind_interface` from the artifact's own declared input extents, so `TilerPlan::load` refuses before any Candle tensor is asked for — the refusal is structurally upstream of the allocator rather than a rescue of its error. The output needs no separate check: its element count is proved equal to the declared row extent immediately above, so an empty output implies an empty axis 0 of the input and is named there first. The proof reports the refused members in the same population as every other one:

```text
  empty-domain.selected: REFUSED before any Candle storage is asked for — candle.preflight.zero-extent: the artifact declares "input" with extents [1, 0], whose axis 1 is empty, and this Candle pin's Metal allocator refuses a zero-length buffer, so no Candle tensor of that shape exists to bind
    and Candle still builds no [1, 0] tensor of its own: Metal error Failed to create metal resource: Buffer
candle adapter proof: 6 of 6 published member(s) resolved — 4 routed and agreed with the producer's recorded reference evaluation across 20 case(s), 2 refused by a typed preflight refusal naming a zero extent (empty-domain.selected, empty-domain.materialized)
```

**The refusal is watched failing, and the allocator measurement is retained rather than replaced.** The proof produces the refusal from the two real published envelopes; `wrapper::tests::a_declared_shape_is_admitted_or_names_its_first_empty_axis` puts the zero at each axis in turn and was watched failing under a deliberate perturbation of the position predicate. The corroborating `Tensor::from_vec` attempt stays in the proof after the refusal, so the first activation trigger is *detected* rather than left to a reader: a Candle whose allocator admits a zero-length buffer builds that tensor and the run fails instead of going on reporting a routable member as refused.

**The placeholder decision is untouched and remains Tom's.** Whether the adapter may synthesize a one-byte placeholder for storage a caller's tensor does not have is a decision about the consumer boundary, not a refactor, and implementing half of it would have been worse than neither half. It is filed as `decide-whether-the-candle-adapter-may-synthesize-zero-extent-storage` at `deferred` so it survives this ticket closing. Note the asymmetry with `prototypes/serial-sum-run`, which does allocate at `needed.max(1)`: that prototype owns every buffer it binds, so a placeholder there is its own storage — the Candle case is a placeholder for a *caller's* value, which is the part that needs deciding.
