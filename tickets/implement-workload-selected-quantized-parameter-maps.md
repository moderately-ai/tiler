---
id: implement-workload-selected-quantized-parameter-maps
title: Implement the workload-selected quantized parameter maps
status: todo
priority: p2
dependencies: [prototype-quantized-value-vertical, scope-first-quantized-lm-profile]
related: [implement-first-quantized-backend-profile, implement-first-runtime-semantic-value-precondition-enforcement]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/artifact, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, parameter-maps, dtype]
---
## User-visible outcome

The first workload-selected per-axis or per-block quantized format carries an exact, validated data-coordinate-to-parameter-coordinate map through semantic identity, reference evaluation, transforms, physical planning, artifact identity, and ABI binding. A view that cannot preserve the scheme is repacked, requantized, or rejected explicitly; it is never interpreted with a nearby parameter by convention.

## Activation boundary

`scope-first-quantized-lm-profile` is the named first consumer and must select the exact scheme, parameter granularity, target operation, layout, and conformance corpus before this ticket becomes actionable. If that profile selects per-tensor parameters only, close this ticket as obsolete rather than inventing a map producer.

**The selection landed on 2026-07-31 and it is not per-tensor, so this ticket is actionable rather than obsolete.** [The first quantized language-model profile record](../docs/research/numerics/first-quantized-lm-profile.md) selected **per-output-channel strict-affine U8 to F32**, so the exact map to implement is:

- **Granularity:** per axis, over the weight's axis 0 — the free index `o` of the workload's contraction structure `td,od->to`. Scale is `tiler::f32@1` of shape `[D_out]`; zero point is `tiler::u8@1` of shape `[D_out]`. Both are rank 1, where every strict-affine parameter component is rank 0 today and `require_scalar_type` in `crates/tiler-ir/src/semantic/quantization.rs` enforces exactly that.
- **Not selected, and each refused by name:** per-tensor beyond the two existing proof contracts, and every per-block or per-group map along the contracted axis. The block maps were eliminated on *legality*, not accuracy — a scale that varies inside the reduction makes the fused contraction partition the contracted axis into contiguous intervals merged in order, which consumes the reassociation permission no contract registered for this workload grants — so a later reassociating contract is what would reopen them, and this ticket must not implement one speculatively.
- **Why the axis is load-bearing rather than incidental:** a per-axis map over axis 1 *is* a per-block map with block size `D_in`, which is the inadmissible family. The map must therefore carry which axis it projects onto, and two otherwise identical values whose maps project onto different axes must have different identities.
- **First producer and consumer:** the pinned `Qwen/Qwen3-0.6B-Base` workload's 196 weighted projection weights, consumed through [`widen-the-physical-vocabulary-for-per-axis-quantized-component-access`](widen-the-physical-vocabulary-for-per-axis-quantized-component-access.md) and [`fuse-quantized-weight-decode-into-the-strict-contraction`](fuse-quantized-weight-decode-into-the-strict-contraction.md), both of which are dependents of this ticket rather than part of it.

## Implementation keys

- Extend the typed `ParameterIndexMap` seam delivered by `prototype-quantized-value-vertical`; do not add a second map spelling or a raw `block_size` field.
- Represent the selected coordinate projection canonically and bound its rank, axes, block geometry, arithmetic, and resulting parameter shape. The same verified map must drive semantic validation, reference lookup, component-shape derivation, identity, explanation, and ABI expansion.
- Keep logical code dtype, quantized scheme, parameter map, and physical packing independent. Packed nibbles do not imply block quantization, and block parameters do not imply one storage encoding.
- Prove each shape/view transform preserves map membership and parameter association. An unaligned slice or reshape that crosses groups must select an explicit repack/requantize plan or fail with a typed reason.
- Support only the exact map forms selected by the workload. Per-axis, regular per-block, irregular groups, hierarchical scales, codebooks, masks, and outliers are different contracts; absent forms reject by name.
- Validate component role completeness and parameter tensor shapes before dependent work. Runtime payload values remain operands and do not become static type fields; constant producers and specialization facts still participate in semantic and artifact identity.
- Derive ABI component bindings from the verified logical value and map. Never infer a role from slot position, shape, element width, or resemblance to another component.
- Add exact reference fixtures, transform-preservation fixtures, packed/unaligned access fixtures, unsupported-map refusals, and identity perturbations. Perturb every new check once and observe it fail.

## Closes when

The selected non-per-tensor parameter map is implemented end to end over its first real producer and consumer; component shapes and coordinate selection are derived from one canonical map; legal transforms preserve exact meaning; illegal or unsupported transforms and map families reject by name; reference, compiler, artifact, and ABI identities distinguish every result-affecting map change; targeted package tests and Clippy pass; and one `make full` passes.

## Graph maintenance

- Update `scope-first-quantized-lm-profile` with the selected map and evidence rather than copying its choice here.
- Refine `implement-first-quantized-backend-profile` to depend on this ticket when the selected backend consumes the new map; keep the dependency absent if that backend supports only per-tensor values.
- File separate implementation work for a second independently required map family only when its workload producer and consumer are named. Do not widen this ticket into a universal map language.
- Advance any affected semantic, program, artifact, and cache identity domains exactly once on the merged tree and recompute every pin there.
