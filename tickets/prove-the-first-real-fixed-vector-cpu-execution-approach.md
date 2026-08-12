---
id: prove-the-first-real-fixed-vector-cpu-execution-approach
title: Prove the first real fixed-vector CPU execution approach
status: todo
priority: p1
dependencies: [promote-the-bounded-scalar-cpu-vertical-into-a-production-backend, admit-vector-lane-bindings-into-the-schedule-vocabulary, establish-vector-execution-form-numerical-authority, earn-cpu-feature-level-execution-environments-from-host-observation, carry-complete-access-alignment-requirements-on-physical-proposals, derive-artifact-binding-alignment-from-selected-access-requirements, prove-planned-binding-alignment-before-routing-commit]
related: [establish-vector-execution-form-numerical-authority, earn-cpu-feature-level-execution-environments-from-host-observation, separate-vector-operand-alignment-from-target-realization]
scopes: [research/target-profiles, research/runtime, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, simd, experiment, backend-providers, correctness]
---
## User-visible outcome

One explicitly selected CPU approach executes a verified fixed-vector program through real AArch64 Advanced SIMD instructions, versioned artifact payload bytes, independent host qualification, and the accepted CPU runtime boundary. It is evidence for the vector KIR and target subjects, not a simulator, fake device, Candle path, or scalar implementation wearing a vector name.

## Accepted experiment boundary — 2026-08-12

Tom accepted this bounded prerequisite in the live Codex coordination thread after the current-base audit found that the vector-KIR ticket excluded every backend while no real vector CPU representation existed. The accepted direction is to prove the consumer before freezing the public KIR.

The experiment owns a distinct, explicitly selected representation such as `tiler.cpu.fixed-vector-image-v1` under the production responsibilities accepted by ADR 0110:

- `tiler-cpu-image` owns the governed image grammar, checked decoder, and real fixed-vector execution engine;
- `tiler-cpu` owns verified-KIR translation and payload production; and
- `tiler-cpu-runtime` owns host qualification, routing, allocation, execution, and completion.

The experiment may begin under `spikes/` while those production packages are being promoted, but it counts as delivery only when its retained path maps one-to-one onto those owners and carries no compiler object into runtime execution.

The production scalar CPU path is now a hard prerequisite rather than merely related work. A retained vector experiment may use a private candidate KIR spelling, but its codec, artifact publication, runtime adapter, storage planning, and completion must extend the real `tiler-cpu-image` / `tiler-cpu` / `tiler-cpu-runtime` owners. This prevents a second spike architecture from becoming the only vector consumer.

## Exact first population

- AArch64 Advanced SIMD, binary32, four lanes.
- `TailPolicy::Exact` and a literal scalar output count divisible by four.
- One dense pointwise map with contiguous reads and writes.
- Contiguous vector load, scalar broadcast, separate multiply and add, vector NaN canonicalization, and contiguous vector store.
- An explicit `CpuExecutionApproach` and resource policy. There is no feature auto-selection, scalar fallback, architecture preset, or retry under another representation.
- Independent `tiler-reference` comparison over ordinary values, both signed zeros, subnormals, infinities, and NaNs.

## Complete alignment tranche

The first NEON load/store form uses the carrier's natural four-byte requirement; the pinned `stdarch` implementation does not justify inventing a 16-byte instruction requirement. The experiment still exercises the complete accepted alignment architecture:

- the selected physical proposal states one exact requirement for every vector memory access;
- the build layer derives each existing artifact binding field from that selected requirement and its exact view;
- `tiler-cpu-runtime` reports the actual final address guarantee for caller storage and the allocator guarantee for future storage before commit; and
- the same real image runs from an exactly four-byte-aligned address and a stronger 16-byte-aligned address, while insufficient and unknown evidence refuses before the executor.

No aligned-only fake instruction, mock provider, synthetic device, or metadata-only success counts. The requirement remains four in both successful runs; what changes is the real address guarantee, proving the divisibility relation without inventing a backend capability.

Architecture-specific intrinsics are the leading implementation because they state the exact execution form. The pinned Rust `stdarch` source maps `vaddq_f32` to `fadd` and its vector load/store intrinsics to `ldr`/`str`, but that is discovery evidence rather than proof of Tiler's emitted program. The retained experiment must inspect its own optimized object or disassembly and prove vector arithmetic was neither scalarized nor contracted into `fmla`.

Any unsafe load/store wrapper is a separately named ADR 0079 site: bounds and byte ranges are checked before the call, the `SAFETY` argument is local, and the experiment perturbs the bound. No broad unsafe allowance follows from this ticket.

## KIR alternatives the experiment must compare

The experiment must compare, rather than assume, these two internal candidate spellings:

1. Separate scalar storage types from SSA execution shapes, with fixed-vector SSA values, explicit unmasked vector memory operations, and element-wise arithmetic over shape-compatible operands.
2. A structured vector-map block whose scalar values acquire lane meaning from their enclosing context.

The first is the provisional recommendation because it makes lane shape readable on every value and does not require an emitter to reconstruct uniform-versus-lane-varying state from block context. The experiment must falsify that preference if the real consumer shows a material correctness, maintenance, or bounded-cost disadvantage. Neither candidate becomes public merely by appearing in the spike.

## Refusals and perturbations

- Width other than four, nondivisible extent, non-contiguous access, unsupported operation, dtype, tail, reduction, gather, mask, scalable shape, target feature, or numerical realization.
- Artifact backend, representation, target profile, provider/variant, image grammar, and host feature each perturbed independently.
- Access requirement, program-view offset, artifact binding requirement, observed address guarantee, allocator guarantee, and post-commit allocator breach each perturbed independently at their owning layer.
- Multiply/add contraction, scalarized arithmetic, one omitted lane, wrong packet count, one out-of-bounds vector access, and NaN canonicalization removed independently.
- CPU output compared against independent reference bits; no CPU executor code is shared with `tiler-reference`.

## Non-goals

Predicated tails, scalar epilogues, contributor partitions, gathers, scalable vectors, threads, a generic portable-SIMD promise, native object loading, JIT code, or a performance claim. A later native AOT representation remains a distinct approach and never silently replaces this one.

## Strongest counterpoint and reversal trigger

An instruction-image dispatcher may eventually spend more host time decoding and dispatching operations than executing them. Move the first vector tier directly to native AOT only if this retained experiment measures that overhead as dominant or proves the image cannot preserve exact provider, operation, and numerical identity. That reversal must also own object format, relocation, executable-memory, toolchain, ABI, publication-security, and cache consequences; none is assumed here.

## Closes when

The retained run proves actual vector instructions, exact artifact/runtime correspondence, independent bitwise semantics, named refusals, and the winning private KIR shape. The result repairs the dependent public KIR ticket before that boundary returns to Tom.
