---
id: widen-the-physical-vocabulary-for-per-axis-quantized-component-access
title: Widen the physical vocabulary for per-axis quantized component access
status: todo
priority: p2
dependencies: [implement-workload-selected-quantized-parameter-maps]
related: [implement-first-quantized-backend-profile, prototype-quantized-value-vertical, fuse-quantized-weight-decode-into-the-strict-contraction, scope-first-quantized-lm-profile]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, kernel-ir, abi, physical-vocabulary]
---
## User-visible outcome

A compound value's parameter component can be addressed by a *projection of the iteration domain* — one scale and one zero point per output coordinate — instead of only as a rank-zero scalar, through signature verification, kernel identity, ABI expansion, lowering, and emission. A combination the vocabulary does not support is refused by name rather than addressed with a nearby index.

## The exact widening, and why naming it is the point

**Fact — what already exists.** [`prototype-quantized-value-vertical`](prototype-quantized-value-vertical.md) delivered one shared realized `StorageEncoding` describing unpacked storage and `PackedU4LsbZeroTail`; `StorageScalar` naming truthful `U8` and `F32` carriers; `KernelType` independently naming `U8`, `I32`, `F32`, `Index`, and `Bool`; and role-addressed schedule accesses. **The selected profile needs no new carrier, no new encoding, and no new kernel type** — it is unpacked `U8` throughout, which is why [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md) chose eight bits over four in part.

**Correction — 2026-08-19 (both vocabulary censuses in the Fact above have grown).** The Fact's *delivery* claim and its conclusion both stand — the profile still needs no new carrier, encoding, or kernel type, and this ticket's widening is still about access relations rather than about vocabulary — but neither enumeration is a current census, and a worker sizing an exhaustive match from either would under-cover its domain:

- `pub enum StorageScalar` (`crates/tiler-ir/src/program/model.rs`) names **four** carriers at this base, not two: `U8`, `F32`, `Bf16`, and `U32`. The last was admitted by [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md) as "An unsigned 32-bit integer carrier", explicitly "physical storage and not an integer-arithmetic capability".
- `pub enum KernelType` (`crates/tiler-ir/src/kernel/model.rs`) names **seven** types, not five: `Bool`, `U8`, `Index`, `F32`, `I32`, `Bf16`, and `U32`.

Neither addition is per-axis access vocabulary, so the "what does not exist" Fact below is untouched by them.

**Fact — what does not exist.** Every strict-affine parameter component is rank zero today: `require_scalar_type` in `crates/tiler-ir/src/semantic/quantization.rs` rejects any scale or zero point whose shape is not `Shape::new([])`, and the delivered U4 path addresses its parameters as scalars. A per-axis map makes them rank-1 tensors of extent `D_out`, and the access that reads one is not a scalar read and not the codes' own access either: within the contraction `td,od->to`, the codes are addressed at `(o, d)` while the scale and zero point are addressed at `(o)` alone.

**Inference — that is the widening, and it is a single named thing.** One logical value whose ordered components require *different* access relations over the same iteration domain, one of them a projection that drops a coordinate. Adding an enum variant is not executable support; this ticket exists because the record's own instruction is that the physical-vocabulary widening a profile needs is a separate dependency of backend implementation with its own verification.

## Implementation keys

- Derive each component's access from the value's parameter map, never from role order, slot position, or shape resemblance. Two components of the same rank and extent must not be interchangeable.
- Structured-kernel signature verification must reject a parameter access whose projection does not agree with the map the semantic type declares, and reject a codes access that dropped a coordinate it needs. Both refusals need a test that was watched failing.
- Kernel identity, program identity, and artifact identity must all distinguish two otherwise identical programs whose parameter component is projected onto a different axis. A per-axis map that reaches the ABI without reaching identity is the silently wrong cache hit the vertical already refused once.
- ABI expansion derives the component bindings from the verified logical value and its map. The decoded view must expose the component's semantic type and its map symmetrically with the verified view.
- Metal emission addresses the parameter buffer by the projected index. It must not hoist, broadcast, or reuse a parameter across output coordinates by convention — if a schedule wants that, it must be a stated schedule property.
- Negative tests: a projection onto an absent axis; a projection onto the contracted axis (which is a per-block map and is not this ticket's); a parameter extent disagreeing with the projected axis's extent; a rank-0 parameter offered where the map declares rank 1, and the reverse; a carrier/encoding/access combination the vocabulary does not admit.

## Closes when

A per-axis strict-affine value's parameter components are addressed by a verified projection end to end — schedule, verified kernel, program, artifact, decoded view, Metal emission — every negative case above refuses by name with its refusal observed firing, identity distinguishes the axis, no role is inferred from slot position, targeted package tests and Clippy pass, `tkt lint` and `git diff --check` pass, and one `make full` passes.

## Graph maintenance

- Filed by [`scope-first-quantized-lm-profile`](scope-first-quantized-lm-profile.md) from [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md) as the exact physical-vocabulary ticket that record's analysis requires.
- [`implement-first-quantized-backend-profile`](implement-first-quantized-backend-profile.md) depends on this. It is not a sub-task of it.
- Advance only the physical-carrier, kernel-vocabulary, ABI, and lowering cells of [the dtype support ledger](../docs/dtype-support.md) that this ticket actually tests, and leave backend execution and target dispatchability alone.
