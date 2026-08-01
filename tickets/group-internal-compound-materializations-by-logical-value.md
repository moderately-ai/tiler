---
id: group-internal-compound-materializations-by-logical-value
title: Group internal compound materializations by producer-derived logical value
status: todo
priority: p2
dependencies: [prototype-quantized-value-vertical, scope-first-quantized-lm-profile]
related: [implement-first-quantized-backend-profile, implement-workload-selected-quantized-parameter-maps]
scopes: [implementation/ir, implementation/compiler, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, compound-values, artifact]
---
## User-visible outcome

A compound value produced inside a physical program remains one logical value across every internal materialization, stage access, allocation, artifact record, and lifetime decision. Its components can be scheduled and stored separately, but scale, code, zero-point, codebook, mask, or future roles can never be detached, regrouped, or confused with a same-shaped component of another logical value.

## Why this is separate

`prototype-quantized-value-vertical` implements complete role-addressed compound interface inputs and outputs and fails closed on internal compound materializations. The existing program IR has no producer-derived logical identity for internal values, so accepting internal roles there would preserve names while losing which components belong together. The first selected quantized backend profile is the named first consumer that must remove that refusal; synthesizing grouping in the artifact layer would be a second semantic authority and is prohibited.

## Implementation keys

- Add one program-owned `LogicalValueId` or equivalent checked handle minted from a verified semantic producer/value relation, not a caller-supplied integer, resolved type, scheme, or role list.
- Represent one internal logical materialization as its complete `ResolvedValueType`, logical shape, canonical ordered component set, and producer-derived identity. Component allocation, storage encoding, view, and lifetime remain physical facts attached below that group.
- Derive the required roles, component resolved types, and component shapes from the encoded value contract and parameter maps. Reject missing, duplicate, extra, swapped, wrong-type, and wrong-shape components before stage verification.
- Bind stage accesses and definitions to `(logical value, component role)` rather than a bare materialized buffer. A component from one logical value cannot discharge another's role even when every physical fact matches.
- Fold the logical group, complete resolved type, producer relation, ordered roles, component encodings, and accesses into program and artifact identity. The artifact codec validates closure and never reconstructs grouping from slot order.
- Keep `BindingSpec` declaration-only. Artifact and runtime projections derive every logical/component target from the verified program.
- Preserve independent components through allocation/lifetime analysis while retaining the whole logical value through moves, views, outputs, and error reporting. Any alias or reuse across components needs an explicit ownership proof.
- Generalize over ordered component declarations; do not hard-code affine roles. Complex planar/interleaved storage, codebooks, hierarchical scales, masks/outliers, and future compound extensions must fit without modifying a universal three-field struct.
- Continue rejecting unsupported parameter maps and internal compound operations by exact scheme/type/capability until their real producers land.

## Closes when

One workload-selected encoded value is produced internally, consumed by a later stage, and packaged with complete producer-derived grouping; component swaps and cross-value substitutions reject; identity changes for every result-affecting group/role/map/encoding change; decoded bindings retain the logical group and role; no caller-declared ABI fact or slot-position inference is introduced; every new check is perturbed once and observed failing; targeted package tests and Clippy pass; and one `make full` passes.

## Graph maintenance

- Refine this ticket with the exact scheme, producer, consumer, and lowering selected by `scope-first-quantized-lm-profile` before implementation.
- **The expected consumer did not arrive, and this ticket's real one is now `Quantize` rather than the selected profile (2026-07-31).** [The first quantized language-model profile record](../docs/research/numerics/first-quantized-lm-profile.md) selected per-output-channel strict-affine U8 over the workload's *weights*, which reach the program as role-addressed compound **interface inputs**: the executed program contains no `Quantize` and no `Assemble`, so it materializes no compound value internally, and `prototype-quantized-value-vertical` already proved the interface-input path end to end. `implement-first-quantized-backend-profile` therefore gained no *direct* edge to this ticket.
- **It reaches this ticket transitively anyway, and that is the state to reason about.** `admit-strict-affine-quantize-physical-candidate` depends on this ticket and sits under the runtime-enforcement vertical the selected backend does depend on, so `tkt path` puts this ticket on the critical path. That chain is real: its consumer is strict-affine **`Quantize`**, which does produce a compound value internally, and which is therefore the producer this ticket needed. The selected profile is not that consumer and should not be described as one.
- **A profile that quantizes activations or requantizes remains the case that would broaden this beyond `Quantize`'s single producer.** Until one is selected, do not derive grouping from a hypothetical one; that is the producer-less placeholder this repository has repeatedly had to retract.
- Add `implement-workload-selected-quantized-parameter-maps` as a dependency only when the selected internal value uses a non-per-tensor map.
- Advance semantic/program/artifact/cache identity domains exactly once on the merged tree and recompute all pinned values there.
