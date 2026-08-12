---
id: admit-fixed-vector-ssa-and-unmasked-memory-into-kernel-ir
title: Admit fixed-vector SSA values and exact unmasked memory into kernel IR
status: blocked
priority: p2
dependencies: [admit-vector-lane-bindings-into-the-schedule-vocabulary, prove-the-first-real-fixed-vector-cpu-execution-approach]
related: [design-the-cpu-vector-lane-tier, admit-subgroup-coordinates-and-xor-transfer-into-kernel-ir, decide-how-vector-requirements-cross-the-artifact-boundary]
scopes: [implementation/ir, implementation/artifact, contracts/decisions, implementation/cpu]
shared_scopes: [project/tickets]
paths: []
tags: [kernel-ir, ir, cpu, simd, public-boundary, decision, needs-tom]
---
## User-visible outcome

The first real fixed-vector CPU approach consumes a `VerifiedKernel` whose vector register shape, unmasked memory effects, bounds evidence, and exactly-once packet ownership are explicit. Buffer storage remains scalar and no backend reconstructs lane shape from a schedule or block context.

## Source-first correction and accepted prerequisite — 2026-08-12

The former ticket packet was not decision-ready. It proposed fixed and scalable values, masks, masked memory, lane comparisons, and no backend, while the accepted schedule slice contains only a fixed literal width with `TailPolicy::Exact` and Tom requires a real CPU consumer. Tom accepted [`prove-the-first-real-fixed-vector-cpu-execution-approach`](prove-the-first-real-fixed-vector-cpu-execution-approach.md) as the prerequisite in the live Codex coordination thread.

The current `KernelType` is also two subjects under one name. `ValueData` uses it for SSA values, while `BufferParameter`, `StagingParameter`, artifact `access_type`, and Metal's `msl_type` use it for stored scalar elements. Adding a vector-register variant directly would make a vector register constructible as a tensor-buffer element and force artifact interfaces to answer for an execution shape they do not store.

## Provisional corrected boundary

The real experiment must validate this exact shape before the ticket returns to `awaiting-decision`:

- Split scalar storage from SSA execution shape. A scalar-element vocabulary owns buffer, staging, artifact access, and scalar constant types. A separate SSA value-type vocabulary owns scalar values and fixed-vector values. Preserve every existing scalar canonical byte without a compatibility alias.
- Reuse the accepted `VectorLaneCount`; do not mint another width, an architecture preset, or a backend instruction name in KIR.
- The first vector element population is F32. Other dtypes, fixed index vectors, lane masks, and scalable values remain absent until a real operation requires them.
- Add explicit contiguous unmasked vector load and store operations over scalar F32 buffers. Each carries the existing schedule bounds or ownership evidence, and whole-kernel verification proves the complete `base..base+W` span and exact packet ownership.
- Add a scalar-to-fixed-vector broadcast. Existing F32 add, multiply, and NaN canonicalization may lift element-wise only when all operands have one identical fixed shape; unsupported operation/shape combinations are typed refusals. There is no parallel semantic arithmetic family.
- Keep `Bool` as the whole-block predicate. No mask type, masked memory, mask comparison, predicated tail, scalable form, gather, strided access, horizontal reduction, or implicit scalarization lands here.
- The accepted CPU fixed-vector image is the real emission consumer. Metal gains only exhaustive compile-time/refusal handling required by a widened internal vocabulary; it does not emit this CPU program.

An enclosing `VectorMap` block whose scalar values acquire lane meaning from context remains the strongest challenger. It is rejected unless the real experiment proves a material advantage: without shape on each value, an emitter must recover uniform-versus-lane-varying state from nesting and dataflow, recreating the inference the KIR exists to remove.

## Identity and artifact boundary

Old scalar value tags and buffer element tags remain byte-identical. A fresh vector SSA tag carries its element kind and literal lane count. No kernel-domain step is owed solely for an append-only value spelling; step the domain only if implementation moves an existing field or encoding.

Artifact interface and binding `access_type` remain scalar. The later vector-requirement artifact ticket owns intrinsic vector requirements and selected execution evidence; this ticket must not smuggle register shape into a storage field or provider identity into KIR.

## Required failure evidence

- A vector SSA type used as a buffer or staging element is unconstructible.
- Mismatched lane counts, scalar/vector operand mixing, unsupported vector operations, width other than the selected real approach, and non-contiguous memory refuse by distinct causes.
- A vector load whose last lane exceeds its bounds proof and a vector store whose packet is not exactly owned both refuse.
- The real CPU object contains the expected vector instructions and no scalarized or contracted substitute.
- Every pre-existing scalar kernel identity pin remains byte-identical; lane-count and vector-operation perturbations move the new identity.

## Non-goals

Masks, tails other than `Exact`, scalable vectors, contributor partitions, gathers, strided memory, native object loading, generic portable-SIMD guarantees, Metal vector emission, or performance claims.

## Board release path

This ticket remains blocked on the schedule implementation and the accepted real fixed-vector experiment. After the experiment records the winning private spelling and real consumer evidence, repair any remaining Facts and move this ticket to `awaiting-decision`; do not implement a public KIR from this provisional text alone.

## Closes when

Tom accepts the experiment-backed exact public Rust surface, the kernel verifier proves vector value and effect obligations, and one real CPU artifact reaches bitwise reference agreement without fallback or backend reconstruction.
