---
schema: "tiler-doc/v1"
id: "tiler.roadmap"
kind: "roadmap"
title: "Roadmap"
topics: ["roadmap", "implementation"]
roadmap_status: "proposed"
---

# Roadmap

**Status:** proposed

The roadmap favors narrow end-to-end slices over implementing a broad IR with
no verified runtime contract.

ADR 0055 authorizes a bounded, unstable implementation prototype whose first
Metal value proof fuses a resolved `f32` pointwise prologue into a strict serial
`f32` `Sum`. ADR 0065 fixes the current prototype crate layout, and ADR 0067
supersedes the Rust 1.89 floor with the exact `nightly-2026-07-19` toolchain for
dependent static-shape evidence. Broader work below remains proposed
progression rather than blanket implementation authority.

## Milestone 0A: semantic graph and extension feasibility

**Research-contract status:** complete. ADRs 0005, 0006, 0008, 0044, and 0045
fix the graph, shape, registry, and proc-macro visibility boundaries. The
semantic type, operation, typed-handle, and reference boundaries are now
compile-checked, and the retained dependent-array conformance harness passes.
The checked shaped-value layer is implemented with compile-fail and identity
coverage, and the assembled semantic/reference slice passes through a
downstream-style public construction and evaluation test. The bounded
Milestone 0A integration proof is complete; production stabilization and the
broader operation/dtype surface remain future work.

The bullets below are the implementation scope authorized only after the
research-readiness decision; they are not claims that the implementations
already exist.

- Define axes, reindexing, broadcasting, dtype, reduction, empty-domain,
  overflow, alias, and numerical policies.
- Define operation conformance vectors and oracle precedence.
- Implement the public semantic operation/value graph and deterministic
  identity needed for executable examples.
- Implement and exercise the public experimental extension path through the ordinary compiler
  API with one built-in and one statically linked external operation definition
  using the same capability interfaces. Separately record which providers the
  proposed proc macro can see across its compilation boundary.
- Implement an explicit per-session registry and continuously test collision rejection,
  canonical attribute encoding, and separation of semantic keys from provider
  revisions.
- Define the consumer-independent `CompilationRequest`, scoped shape symbols,
  and sourceability of every dynamic output/temporary/guard/launch expression.
- Establish multiple named results, sharing, and multi-result representation
  invariants even if the first runtime profile executes a narrower subset.
- Review and integrate ADR 0067's implemented arbitrary-rank
  `StaticShape<RANK, EXTENTS>` evidence family; reuse the retained conformance
  harness for every compiler-pin migration.

**Exit criterion:** tensor meaning and graph invariants have a reviewed
contract, mandatory operation-extension capabilities are explicit, and a small
semantic graph verifies and evaluates without any frontend, backend, or runtime
dependency. Registry, canonical-data, semantic/provider identity, and dynamic
shape-source invariants are tested.

## Milestone 0B: Rust/Metal integration vertical feasibility

**Research-contract status:** complete. ADRs 0002–0004, 0049–0053 and the
artifact/cache/runtime spikes fix the AOT, inline-DX, family-selection,
publication, and fallback boundaries. The actual Tiler macro-to-dispatch
vertical remains implementation work.

The bullets below are remaining vertical implementation and integration checks,
not completed production capabilities.

- Build a proc-macro spike that compiles fixed deterministic MSL with `xcrun`
  and emits manifest/metallib byte-string literals without consumer setup.
- Implement the accepted immutable self-validating content-addressed cache and
  reproduce the completed process-level crash/race harness against it.
- Retain the completed embedding, Cargo freshness, cache deletion, and Apple
  family/toolchain probes. Measure rust-analyzer cold/warm behavior when the
  component is available, plus the actual native macOS and non-Apple fallback
  paths.

**Exit criterion:** cold inline macro AOT produces a loadable bundle, warm
equivalent expansions invoke no external compiler, and the proposed Rust DX
works without build scripts or prebuild commands. Failure does not invalidate
Milestone 0A's consumer-independent compiler boundary.

The currently authorized Metal AOT and runtime tickets prove backend artifact
and device-execution boundaries but intentionally exclude the proc macro,
generalized cache, and consumer integration. They are prerequisites and
evidence for this milestone, not its complete exit.

## Milestone 1: canonical semantic graph and index IR

- Build the typed operation/value semantic graph.
- Lower output coordinates through composed reindexes into access maps.
- Add symbolic extents, strides, and offsets.
- Implement semantic/index verifiers in `tiler-ir` and the slow executable
  oracle in downstream `tiler-reference`.
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
- Implement the accepted prototype strict serial `f32` `Sum` profile and
  compare one fused map/reduce dispatch with a deliberately materialized
  reference.
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

The strict serial-Sum architectural proof exercises the core compiler and Metal
path before this broader milestone. It intentionally does not claim the Candle
adapter, inline macro, general fallback, or einops-derived workload required by
the milestone exit.

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
- Implement governed capability keys and all `CompileProfile`,
  `ArtifactEvidence`, `LiveDevicePreflight`, `PreparedKernelPreflight`, and
  `LaunchPreflight` fact phases, with aggregate
  `Proven`/`Deferred`/`Rejected`/`Unknown` feasibility and `RoutingCommit`.
- Keep hard resource proofs distinct from register, occupancy, cache, and
  throughput estimates; validate fixed and scalable vector legality.
- Structured rejection reasons and plan comparison.

**Exit criterion:** the optimizer chooses among several valid region
implementations and complete `KernelProgram`s and explains the choice.

## Milestone 4: reductions

- Broaden the exact serial baseline beyond any narrow Milestone 2 proof.
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
