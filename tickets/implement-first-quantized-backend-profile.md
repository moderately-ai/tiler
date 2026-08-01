---
id: implement-first-quantized-backend-profile
title: Implement the first selected quantized backend profile
status: todo
priority: p2
dependencies: [prototype-quantized-value-vertical, scope-first-quantized-lm-profile, admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode, implement-workload-selected-quantized-parameter-maps, widen-the-physical-vocabulary-for-per-axis-quantized-component-access, measure-code-domain-integer-arithmetic-on-the-qualified-apple-row, fuse-quantized-weight-decode-into-the-strict-contraction, implement-first-runtime-semantic-value-precondition-enforcement]
related: [implement-workload-selected-quantized-parameter-maps, own-the-dtype-support-maturity-matrix, admit-a-caller-declared-target-profile, calibrate-device-cost-models, group-internal-compound-materializations-by-logical-value, extend-the-selected-quantized-profile-to-the-tied-embedding-matrix, admit-strict-affine-quantize-physical-candidate]
scopes: [implementation/compiler, implementation/artifact, implementation/reference, implementation/runtime, implementation/metal]
shared_scopes: [project/tickets]
tags: [implementation, quantization, backend, metal, language-model]
---
Activate only after a concrete quantized format, operation set, target backend,
storage layout, numerical contract, and conformance corpus are selected. Then
implement lowering, schedule feasibility, code generation, ABI/runtime binding,
and device comparison without generalizing beyond that measured profile.

Before activation, revise the scopes and dependencies to name the selected
backend, runtime adapter, reference/conformance owner, and measured corpus. For
that selected profile, supported programs compile, execute, and match the
normative reference; every program outside it receives a typed refusal.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## The selection this ticket now consumes (2026-07-31)

[`scope-first-quantized-lm-profile`](scope-first-quantized-lm-profile.md) selected the profile from measured evidence in [the first quantized language-model profile record](../docs/research/numerics/first-quantized-lm-profile.md). This ticket implements exactly it and nothing adjacent.

| Field | Selected value |
| --- | --- |
| Scheme | `tiler::strict-affine@1`, per **output channel** |
| Code type / domain | `tiler::u8@1`, inclusive `[0, 255]` |
| Expressed and compute type | `tiler::f32@1` |
| Parameter map | per-axis over the weight's axis 0, the free index `o` of `td,od->to`; scale and zero point are rank-1 of extent `D_out` |
| Physical storage | unpacked `StorageScalar::U8`; **no packed encoding, no bitstream order, no tail rule** |
| Scale value domain | positive **normal** finite, which is what makes the decode target honourable |
| Covered operands | the 196 weighted projection weights of the pinned `Qwen/Qwen3-0.6B-Base` workload |
| Physical consumption | the decode fused into the contraction's weight operand access, with the materializing plan retained and separately costed |
| Target | `apple9-f32-unified-msl4-macos26`, the qualified row |
| Reference and comparison | Tiler's own reference evaluator on the same quantized program, exact bits, zero tolerance |

Three consequences for the completeness list below. **The packed sub-byte requirement does not apply** — the selected code width is a whole storage unit, so a claim about bit order, shared boundary bits, or canonical tails would be a claim about a path this profile does not have. **No new `StorageScalar`, `StorageEncoding`, or `KernelType` is needed**; the widening that *is* needed is a parameter component addressed by a projection of the iteration domain, which [`widen-the-physical-vocabulary-for-per-axis-quantized-component-access`](widen-the-physical-vocabulary-for-per-axis-quantized-component-access.md) owns as a dependency. And **per-tensor U4, per-tensor U8, every per-block map, MX, and FP8 are each refused by name**, each having been eliminated on stated grounds rather than merely left out.

## Required completeness after activation

- Consume one complete `ResolvedValueType::encoded_numeric` contract and its ordered component declarations. Do not introduce a backend `Q4`, `Q8`, or `quantized integer` dtype that loses the scheme, parameter map, conversion, or metadata association.
- Carry every selected logical component role through lowering, physical storage, artifact identity, role-addressed ABI binding, runtime placement, and resource lifetime. Same-shaped components remain distinguishable by role and can never be inferred from slot order.
- Keep logical scalar type, numerical interpretation, parameter map, physical encoding, and target-native arithmetic as separate capability decisions. An unpacked `u4`, packed `u4`, affine value with `u4` codes, MXFP4 value, and native FP4 operation are five different claims.
- State conversion, physical layout/access, and observable materialization contracts separately. Native instructions, helper code, unpack/dequantize paths, and fused paths must reproduce the same selected semantics or reject.
- Prove both an ordinary byte-addressed path and an actually packed sub-byte path if the selected profile claims both. State bit order, storage-unit order, padding, alignment, partial and unaligned access, ownership of shared boundary bits, and canonical tail behavior.
- Record exact target-family dispatchability and numerical honourability for every dtype the profile consumes. An unmeasured `(target family, dtype)` pair is `Unknown` and cannot produce an executable artifact.
- Compare exact codes and exact reference bits where the contract is exact. Any tolerance or model-level error criterion must be derived from the selected scheme and domain, with saturation and exceptional cases tested separately.
- Refuse every unselected scheme, code dtype, map, storage encoding, operation signature, accumulator, output dtype, and target realization with a typed diagnostic naming the missing capability.
- Keep future boolean, integer, complex, decimal, codebook, hierarchical-scale, MX, sparse, and ragged support outside this profile unless the selected workload makes one a real producer and consumer. Generic seams are not support claims.
- Consume an exact profile-driven physical-vocabulary ticket for every new `StorageScalar`, `StorageEncoding`, and `KernelType` the backend needs. A storage carrier without signature verification, kernel identity, ABI compatibility, target dispatchability, lowering/emission, and typed unsupported-combination tests is not an executable path.
- Update the dtype maturity ledger only for the exact `(logical type, scheme, operation, storage, target, runtime path)` cells this profile implements or tests. Do not promote neighbouring widths or formats from a shared enum arm, helper, intrinsic, or nominal fixture.

## Graph maintenance

- Consume the selection and elimination record from `scope-first-quantized-lm-profile`; do not repeat format selection here. **Done on 2026-07-31**, in the section above.
- **The conditional dependencies are now resolved and structural.** The selected map is non-per-tensor, so `implement-workload-selected-quantized-parameter-maps` is a dependency. The selected valid domain has runtime value predicates on scale and zero point, so `implement-first-runtime-semantic-value-precondition-enforcement` is a dependency. The decode's target honourability rests on the normal-scale precondition, the fused operand access, and a device measurement that has never been taken, so `admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode`, `fuse-quantized-weight-decode-into-the-strict-contraction`, and `measure-code-domain-integer-arithmetic-on-the-qualified-apple-row` are dependencies. All six are edges rather than prose, because a prose list is not a dependency.
- **`group-internal-compound-materializations-by-logical-value` is deliberately not a *direct* dependency, and it is still reached transitively.** The selected profile's compound values are role-addressed interface *inputs*: the executed program contains no `Quantize` and no `Assemble`, so it materializes no compound value internally, and the vertical already proved the interface-input path end to end. No direct edge was added. But `tkt path` shows it on the critical path anyway, through `implement-first-runtime-semantic-value-precondition-enforcement` → `carry-semantic-enforcement-plans-through-program-and-artifact` → `admit-strict-affine-quantize-physical-candidate` → the grouping ticket, because that enforcement vertical is scoped to strict-affine **`Quantize`** and a `Quantize` does produce a compound value internally. **What this profile actually needs enforced is the value domain of an input** — positive normal scale, in-range zero point, parameter extents agreeing with the codes' axis-0 extent — reached through `Dequantize` and binding conformance. Resolving that mismatch is the enforcement ticket's, which carries a dated note; do not route around it here by dropping the dependency.
- Consume `admit-strict-affine-quantize-physical-candidate` for the bounded correctness route. The candidate breaks the dependency cycle by supplying real result work without prematurely exposing dispatch authority.
- Add a dependency on `admit-a-caller-declared-target-profile` before claiming target-family executability.
- Add profile-specific analytical and calibrated cost dependencies before claiming the selected implementation is device-optimal; a correctness-only spike may proceed while unmeasured costs remain `Unknown`, but it cannot select itself as optimal. **The selection record makes no device-optimal claim and its analytical projection is explicitly a hypothesis**, so `calibrate-device-cost-models` and experiment E-2 of that record are prerequisites of any such claim and not of this ticket's closure.
- Update `own-the-dtype-support-maturity-matrix` with construction-site evidence and bounded conformance results when this profile closes.
- Split weight ingestion, packing/repacking, native contraction, runtime binding, and model-level comparison when their scopes or evidence can move independently. Each split ticket must name its exact scheme, target, operation, and corpus.
- Advance versioned identities only for fields the selected producers actually fill, then recompute pins on the merged tree.
