---
id: implement-first-quantized-backend-profile
title: Implement the first selected quantized backend profile
status: todo
priority: p2
dependencies: [prototype-quantized-value-vertical, scope-first-quantized-lm-profile, admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode, implement-workload-selected-quantized-parameter-maps, widen-the-physical-vocabulary-for-per-axis-quantized-component-access, measure-code-domain-integer-arithmetic-on-the-qualified-apple-row, fuse-quantized-weight-decode-into-the-strict-contraction, implement-first-runtime-semantic-value-precondition-enforcement, reclassify-language-model-work-as-a-conformance-track, admit-a-caller-declared-target-profile]
related: [own-the-dtype-support-maturity-matrix, calibrate-device-cost-models, extend-the-selected-quantized-profile-to-the-tied-embedding-matrix, enforce-resolved-encoded-value-binding-conformance, carry-semantic-enforcement-plans-through-program-and-artifact]
scopes: [implementation/compiler, implementation/artifact, implementation/reference, implementation/runtime, implementation/metal]
shared_scopes: [project/tickets]
tags: [implementation, quantization, backend, metal, language-model, class-generic-capability]
paths: [docs/research/numerics/first-quantized-lm-profile.md, spikes/numerics/qwen3-weight-quantization-profiles/, spikes/apple-targets/code-domain-integer-decode/results/2026-07-31-decode-u8-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/]
---
**Selection is already consumed (2026-07-31).** The concrete quantized format, operation set, target backend, storage layout, numerical contract, and conformance corpus are fixed in the selection section below; this ticket implements exactly that profile. Lowering, schedule feasibility, code generation, ABI/runtime binding, and device comparison stay inside the measured bounds. Supported programs for that profile compile, execute, and match the normative reference; every program outside it receives a typed refusal. Layer scopes name the integration surface (compiler, artifact, reference, runtime, metal); `paths` names the selection record, measured corpus probe tree, and E-1 device-decode results directory.

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
- **The conditional dependencies are now resolved and structural.** The selected map is non-per-tensor, so `implement-workload-selected-quantized-parameter-maps` is a dependency and `widen-the-physical-vocabulary-for-per-axis-quantized-component-access` owns the per-axis component-access vocabulary that map needs. The selected valid domain has runtime value predicates on scale and zero point, so `implement-first-runtime-semantic-value-precondition-enforcement` is a dependency. The decode's target honourability rests on the normal-scale precondition, the fused operand access, and the completed bounded device measurement, so `admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode`, `fuse-quantized-weight-decode-into-the-strict-contraction`, and `measure-code-domain-integer-arithmetic-on-the-qualified-apple-row` are dependencies. All six are edges rather than prose, because a prose list is not a dependency.
- **The selected direct-input enforcement chain is now structural and contains no internal-grouping prerequisite.** [`enforce-resolved-encoded-value-binding-conformance`](enforce-resolved-encoded-value-binding-conformance.md) supplies the type-derived contract, exact proof/evidence identity, and logical scan. [`reconcile-direct-input-conformance-order-with-adr-0033`](reconcile-direct-input-conformance-order-with-adr-0033.md) first aligns the four governing records with ADR 0033. [`carry-semantic-enforcement-plans-through-program-and-artifact`](carry-semantic-enforcement-plans-through-program-and-artifact.md) then carries one plan per selectable alternative, binding the fused contraction access or retained `Dequantize`/materialization stage as that alternative's exact first consumer. [`implement-first-runtime-semantic-value-precondition-enforcement`](implement-first-runtime-semantic-value-precondition-enforcement.md) executes the selected input's conformance after `RoutingCommit` and authorizes only the selected alternative's bound consumer. This ticket depends on that runtime vertical and on the fused/materializing alternative owner directly; feasibility and cost remain separate per alternative, and an unenforceable alternative cannot win by a missing or `Unknown` enforcement cost.
- **`group-internal-compound-materializations-by-logical-value` is neither a direct nor a transitive prerequisite.** The selected profile's compound values are role-addressed interface inputs and the executed program contains no `Quantize` or `Assemble`. [`admit-strict-affine-quantize-physical-candidate`](admit-strict-affine-quantize-physical-candidate.md) closed obsolete under its own trigger; it is not retained as a correctness route or a future consumer.
- **Done in the graph on 2026-08-09:** consume [`admit-a-caller-declared-target-profile`](admit-a-caller-declared-target-profile.md) before claiming target-family executability. Its accepted public `TargetProfile` and full-`ResolvedValueType` dispatchability boundary is the authority this selected Metal declaration uses; do not restate a private dtype list here.
- Add profile-specific analytical and calibrated cost dependencies before claiming the selected implementation is device-optimal; a correctness-only spike may proceed while unmeasured costs remain `Unknown`, but it cannot select itself as optimal. **The selection record makes no device-optimal claim and its analytical projection is explicitly a hypothesis**, so `calibrate-device-cost-models` and experiment E-2 of that record are prerequisites of any such claim and not of this ticket's closure.
- Update `own-the-dtype-support-maturity-matrix` with construction-site evidence and bounded conformance results when this profile closes.
- Split weight ingestion, packing/repacking, native contraction, runtime binding, and model-level comparison when their scopes or evidence can move independently. Each split ticket must name its exact scheme, target, operation, and corpus.
- Advance versioned identities only for fields the selected producers actually fill, then recompute pins on the merged tree.

## Fact audit — 2026-08-10

**Correction — 2026-08-10** (audit base `c99ac54950f2`). (1) Selection and the structural conditional dependencies were already consumed on 2026-07-31 / graph maintenance; the pre-edit opening activation gate ("Activate only after…", "Before activation, revise the scopes and dependencies…") was residual template, not outstanding work — rewritten above into historical/consumed form, and `paths` now names the selection record, Qwen3 corpus probe tree, and E-1 results directory. (2) The graph-maintenance "All six are edges" bullet had named only five backticked tickets; it now also names `widen-the-physical-vocabulary-for-per-axis-quantized-component-access` so the count matches the enumerated structural deps. (3) `related` no longer duplicates `implement-workload-selected-quantized-parameter-maps` or `admit-a-caller-declared-target-profile`, which remain hard `dependencies` only. Status stays `todo`; product implementation is not started.
