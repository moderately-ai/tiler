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
