---
schema: "tiler-doc/v1"
id: "tiler.prior-art.ug"
kind: "prior-art"
title: "Lessons from `ug`"
topics: ["compiler", "fusion", "prior-art"]
informs: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.optimizer"]
---

# Lessons from `ug`

**Status:** supporting analysis

This document records lessons from `ug` as prior art. Tiler does not plan to
depend on it or preserve its APIs. Source observations refer to `ug` commit
[`8c6dd50`](https://github.com/LaurentMazare/ug/tree/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7)
and Candle commit
[`71e80f5`](https://github.com/huggingface/candle/tree/71e80f5a50e0464091e17ae0136cb4b7efd95f30).

## What `ug` demonstrates well

`ug` demonstrates a compact end-to-end compiler path:

```text
lazy tensor graph
  -> fused shape-aware expression
  -> index/loop lowering
  -> low kernel IR
  -> CPU/CUDA/Metal source and execution
```

The most valuable technique is reverse coordinate lowering. Its layout
transformations rewrite output coordinates through broadcast, narrow,
transpose, split, and merge before a physical layout constructs
`offset + sum(index * stride)`. See
[`lower_op.rs`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-core/src/lower_op.rs#L119-L227).
This maps directly to compiling einops transformations without materializing
each logical rearrangement.

Other concepts worth retaining are:

- a shape-aware semantic layer above kernel instructions;
- explicit materialization boundaries;
- structural normalization before caching;
- the architectural idea of a low-level interpreter for differential tests;
- an analysis sketch for coalescing dimensions across participating accesses;
- a partial separation between tensor expressions and backend source.

These need qualification. The detailed adjacent-dimension analysis is not wired
into the active lowering path; active compression is a more conservative all-
or-nothing transformation. The interpreter ignores scalar arguments, supports
only part of the dtype/operation space, and cannot serve as a complete semantic
authority. Backend code generation and runtime compilation also remain coupled
through `ug`'s device interface. Tiler copies the ideas, not their completeness.

## Representation lessons

`ug` contains a lazy tensor DAG, a shape-aware tensor AST, an older overlapping
Triton-like language, and a backend representation described in source as
“very untyped almost SSA.” The latter includes mutable accumulators and
assignment rather than SSA invariants. See
[`lang.rs`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-core/src/lang.rs#L607-L709).

Tiler should instead use one durable semantic tensor IR, an interned symbolic
index DAG, a schedule representation, and typed structured kernel code.
Process-global IDs, line-number control-flow links, and semantic AST nodes that
temporarily contain lowering IDs should not cross durable boundaries.

`ug` also notes that recursive index-formula expansion can become exponential.
Tiler's index arena should provide CSE and canonical simplification rather than
relying on recursive trees.

## Shape and ABI lessons

`ug` specializes concrete host `usize` shapes, strides, and offsets into
unchecked `i32` constants during lowering; the limitation is acknowledged in
[`lower_op.rs`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-core/src/lower_op.rs#L87-L113).
Its generic runtime cannot bind the scalar parameters needed for dynamic
metadata.

Tiler instead needs symbolic extents, strides, and offsets from the beginning,
with explicit specialization choices and a typed scalar ABI. Rank, graph,
dtype, and selected schedule variants are good AOT specialization dimensions;
exact shapes usually are not.

## Fusion and scheduling lessons

`ug` recursively fuses elementwise, layout, and reduction nodes, while a use
count above one forces materialization. See
[`schedule.rs`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-core/src/schedule.rs#L327-L463).
That is a useful prototype bound, but fan-out should feed a costed
recompute-versus-materialize decision rather than define legality.

Its automatic GPU schedule primarily sorts output dimensions by size and does
not substantially account for storage layout or reduction structure. Tiler
keeps global fusion decisions separate from local schedules and represents
required/provided physical properties explicitly.

Reshape should not automatically create a kernel boundary. In an einops
compiler, many rank changes remain inverse access-map transformations.

## Reduction lesson

`ug` lowers some reductions to an opaque `ReduceLocal`. The Metal generator
emits calls to `block_reduce_max` and `block_reduce_sum` in
[`code_gen.rs`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-metal/src/code_gen.rs#L167-L174),
but does not provide those helpers; both semantic and explicit-SSA Metal
softmax routes were confirmed to fail compilation.

The broader lesson is not merely to add missing helpers. A portable opaque
block reduction is underspecified across targets. Reduction should remain
semantic until scheduling chooses a serial, subgroup, threadgroup, or
multi-pass implementation with explicit preconditions and numerical behavior.

## Artifact and caching lessons

`ug` usefully normalizes transient argument IDs for a structural kernel key in
[`cache.rs`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-core/src/cache.rs#L14-L70).
However, its lower-IR cache key omits launch configuration, ABI, target,
compiler version, and numerical flags. Tiler artifact identity must cover all
of these and use deterministic symbols rather than process counters.

## Candle integration lessons

Candle's `UgIOp1` is a useful proof that generated Metal pipelines can be
encoded through Candle, but it is a narrow one-buffer, in-place, F32 adapter.
See
[`custom_op.rs`](https://github.com/huggingface/candle/blob/71e80f5a50e0464091e17ae0136cb4b7efd95f30/candle-core/src/custom_op.rs#L379-L487).

It discards `ug` launch configuration, derives dispatch from element count,
and its Metal branch does not validate/apply tensor view layout as required for
a general operation. Tiler needs an output-producing ABI with ordered buffers
and scalars, explicit layout guards, byte offsets, output-shape metadata, and
artifact-owned launch policy.

The part to copy is command-stream integration: obtain Candle's current encoder
and return after encoding rather than committing and synchronously waiting.

## Runtime and testing lessons

`ug`'s own Metal runtime rebuilds pipelines and synchronously waits around
generic dispatch; see
[`runtime.rs`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-metal/src/runtime.rs#L108-L125).
Some global caches do not key device-bound objects by device, and the generic
runtime exposes mutable whole buffers rather than typed view/access roles; see
[`lib.rs`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-core/src/lib.rs#L49-L60).
Tiler therefore uses per-device library/function/pipeline caches, explicit
read/write ABI roles, and asynchronous command-stream integration.

Additional reviewed pitfalls include:

- a CUDA block-reduction helper that is not a general correct implementation
  for all threadgroup sizes
  ([`reduce.cu`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-cuda/src/reduce.cu#L29-L79));
- weak backend test coverage, including backend `cat` tests that accidentally
  exercise CPU code
  ([Metal tests](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-metal/tests/lb_tests.rs#L3-L18));
- apparent element-offset versus byte-offset confusion in a handwritten Metal
  GEMM path
  ([`runtime.rs`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-metal/src/runtime.rs#L510-L517));
- no general scalar metadata ABI despite dynamic kernels requiring one.

These motivate forced testing across reduction regimes, host/MSL ABI layout
tests, checked byte offsets, and adversarial runtime metadata cases.

## Summary

Copy the compiler layering, reverse index mapping, normalization, interpreter,
and explicit boundary concepts. Avoid concrete-only shapes, overlapping public
IRs, global IDs, weak pointer/ABI types, opaque collectives, categorical
fan-out materialization, incomplete cache keys, and runtime compilation as an
architectural requirement.
