# Roadmap

**Status:** proposed

The roadmap favors narrow end-to-end slices over implementing a broad IR with
no verified runtime contract.

## Milestone 0A: semantic graph and extension feasibility

- Define axes, reindexing, broadcasting, dtype, reduction, empty-domain,
  overflow, alias, and numerical policies.
- Define operation conformance vectors and oracle precedence.
- Prototype the public semantic operation/value graph and deterministic
  identity needed for executable examples.
- Exercise the public experimental extension path through the ordinary compiler
  API with one built-in and one statically linked external operation definition
  using the same capability interfaces. Separately record which providers the
  proposed proc macro can see across its compilation boundary.
- Freeze an explicit per-session registry and prove collision rejection,
  canonical attribute encoding, and separation of semantic keys from provider
  revisions.
- Define the consumer-independent `CompilationRequest`, scoped shape symbols,
  and sourceability of every dynamic output/temporary/guard/launch expression.
- Establish multiple named results, sharing, and multi-result representation
  invariants even if the first runtime profile executes a narrower subset.

**Exit criterion:** tensor meaning and graph invariants have a reviewed
contract, mandatory operation-extension capabilities are explicit, and a small
semantic graph verifies and evaluates without any frontend, backend, or runtime
dependency. Registry, canonical-data, semantic/provider identity, and dynamic
shape-source invariants are tested.

## Milestone 0B: proposed Rust/Metal integration feasibility

- Build a proc-macro spike that compiles fixed deterministic MSL with `xcrun`
  and emits manifest/metallib byte-string literals without consumer setup.
- Implement a minimal content-addressed user cache with cross-process locking,
  validation, and atomic publication.
- Measure cold/warm expansion, rustc memory for representative bundle sizes,
  repeated-literal binary size, rust-analyzer, `cargo check`, cache deletion,
  native macOS, and non-Apple fallback behavior.

**Exit criterion:** cold inline macro AOT produces a loadable bundle, warm
equivalent expansions invoke no external compiler, and the proposed Rust DX
works without build scripts or prebuild commands. Failure does not invalidate
Milestone 0A's consumer-independent compiler boundary.

## Milestone 1: canonical semantic graph and index IR

- Build the typed operation/value semantic graph.
- Lower output coordinates through composed reindexes into access maps.
- Add symbolic extents, strides, and offsets.
- Implement semantic/index verifiers and the slow reference evaluator.
- Canonically serialize and reference-evaluate every enabled transcendental
  accuracy contract before admitting such an operation to the vertical slice.
- Establish randomized differential testing against normative semantics and
  independent compatibility cases; Candle cases belong to the first
  integration suite.
- Add deterministic serialization, hashing, and textual `EXPLAIN`.
- Add the conservative one-allocation-per-output/temporary `BufferPlan` and
  single-device, single-ordered-stream `KernelProgram` verifier.
- Implement constant folding, index CSE, and conservative dimension coalescing.

**Exit criterion:** programs within the implemented view/map normalization
theory produce verified canonical access maps independent of transient IDs and
construction order.

## Milestone 2: conservative Metal vertical slice

- One input, one newly allocated output, F32, statically known rank.
- Contiguous layout with arbitrary valid start offset.
- Reindex plus pointwise fusion.
- Initially limit pointwise operations to fully resolved algebraic semantics;
  any transcendental or GELU enters only with its formula, reference evaluator,
  accuracy contract, and conformance evidence implemented end to end.
- Scalar one-thread-per-output and rank-aware schedules.
- Minimal conservative Metal target profile for correctness and launch limits.
- Deterministic MSL, one metallib bundle, and complete lockstep experimental
  manifest.
- Expansion-time `xcrun`, global content cache, and direct byte embedding.
- Candle custom-op adapter, per-device pipeline cache, and fallback.
- A trivial single-pipeline region builder; general memo/DAG planning is not
  implemented yet.

**Exit criterion:** a useful einops-derived chain executes correctly with fewer
dispatches or intermediates than the reference path.

## Milestone 2Q: quantized-value vertical proof

- Verify and reference-evaluate strict affine `i4/u4/i8/u8` code tensors with
  `f32` expressed, scale, computation, and requantization-intermediate values.
- Cover per-tensor, per-axis, and per-block parameter maps with constant and
  runtime graph operands.
- Implement `AssembleQuantized`, `Quantize`, `Dequantize`, and `Requantize`
  contracts independently of physical packing.
- Lower at least one 8-bit path and one packed 4-bit block path, with complete
  component-role ABI and storage-encoding validation.
- Exercise proof-elided semantic validation; measure runtime enforcement
  separately rather than weakening strict semantics for an integration.

**Exit criterion:** logical code type, quantized interpretation, parameter map,
and packed storage remain independently verified while a representative 8-bit
and 4-bit program agree with the strict reference evaluator.

## Milestone 3: physical properties and alternatives

- Required/provided layout, alignment, vector width, and materialization.
- Scalar, vectorized, collapsed, and general-stride candidates.
- Explicit contiguous/layout enforcers.
- Bounded alternative search and first analytical cost model.
- Add richer device-family profiles and symbolic/guarded routing.
- Structured rejection reasons and plan comparison.

**Exit criterion:** the optimizer chooses among several valid region
implementations and complete `KernelProgram`s and explains the choice.

## Milestone 4: reductions

- Exact serial reduction baseline.
- SIMD-group and threadgroup strategies.
- Fused pointwise prologues and epilogues.
- Explicit accumulator and empty-domain policy.
- Ragged-tail and multi-SIMD-group coverage.
- Multi-pass fallback for large domains.

**Exit criterion:** at least one rearrange/map/reduce chain is safely fused and
outperforms or reduces traffic relative to the reference pipeline.

## Milestone 5: graph partitioning

- Candidate regions across DAGs.
- Costed fuse versus split decisions.
- Fan-out recompute versus materialize.
- Multi-output candidates.
- Live-value/register and intermediate-memory estimates.

**Exit criterion:** fusion is a costed global decision rather than a linear
pipeline heuristic.

## Milestone 6: einsum contractions

- Contraction-order exploration.
- GEMM recognition and library-call alternatives.
- Layout-conversion costing.
- Direct/tiled contractions and fusible epilogues.

**Exit criterion:** contraction planning uses the same properties, enforcers,
and cost framework rather than backend-specific exceptions.

## Milestone 7: artifact maturity

- Stable versioned artifact schema.
- Compatibility policy beyond the earlier lockstep experimental schema.
- Mature macro-local multi-entrypoint packaging and deterministic expansion.
- Concurrent expansion locking and compiler-cache diagnostics.
- Embedded-byte size budgets and, if justified by measurement, linker-level
  deduplication that does not change call-site DX.
- Compile/search/artifact-size budgets.
- Platform and toolchain compatibility policy.

## Milestone 8: calibration

- Device-family microbenchmarks.
- Predicted-versus-observed plan tracking.
- Cost coefficient calibration and candidate pruning.
- Optional offline or profile-guided schedule selection.

The proposed Rust/Metal integration does not require runtime source JIT.

## Deferred until justified

- Generated backward kernels.
- In-place or aliasing kernels.
- Arbitrary user-authored kernel language.
- Cross-threadgroup atomics as a general scheduling tool.
- Runtime autotuning.
- Stable public serialization compatibility before IR boundaries settle.
