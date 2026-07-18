# Vision and scope

**Status:** proposed

## Problem

Declarative tensor expressions often reach tensor runtimes as a sequence of
generic operations. A rearrangement, broadcast, elementwise expression, and
reduction may therefore become several dispatches and may write intermediate
tensors to device memory even when the complete computation is known during
compilation.

Tiler should compile the combined tensor computation rather than optimizing
individual runtime calls. Given a program such as:

```text
y[b,c] = sum(h,w, gelu(x[b,h,w,c] * scale + bias[c]))
```

it should be able to generate a kernel that reads the original storage,
computes the elementwise expression in registers, performs the selected
reduction, and writes only the final output.

## Product definition

Tiler is a tensor-specific, frontend-, backend-, and runtime-independent
optimizer and kernel compiler toolkit. Its primary public input is a semantic
tensor graph. Frontends lower tensor languages into that graph, and target
integrations consume generated programs without owning compiler semantics.
An initial Metal AOT backend provides the first end-to-end target. Tiler
provides:

- a public experimental semantic tensor operation/value graph;
- symbolic index and access-map lowering;
- semantic rewrites and region-candidate formation;
- target-aware region implementation and complete-program selection;
- typed, structured kernel lowering;
- deterministic source and artifact generation;
- a versioned kernel ABI and runtime guards;
- reference evaluation and verification tools.

It borrows selected database-optimizer techniques for tensor iteration spaces:
semantic expressions are normalized, contract-conforming alternatives are
explored, boundary requirements are enforced, and complete implementations are
costed. The analogy is architectural rather than structural: tensor programs
are shared multi-result DAGs, and GPU schedule feasibility depends on access
maps, numerical contracts, synchronization, and discontinuous resource limits.

## Initial consumers

The first proposed frontend is `candle-einops`, and the first proposed runtime
adapter is Candle Metal. They are validation integrations, not owners of the
logical graph, optimizer, schedule representation, or artifact semantics. Their
Rust macro may compile a self-contained Metal program bundle during expansion,
but that developer experience is an integration contract rather than the
definition of Tiler itself.

Neither einops syntax nor Candle storage types belong in the compiler core.
They are integrations around a reusable compiler.

## Goals

1. Remove avoidable intermediate tensor allocation and global-memory traffic.
2. Fuse compatible reindex, broadcast, map, and reduction operations.
3. Select memory-coalesced, vectorized, and reduction-aware schedules.
4. Support runtime shapes and layouts through symbolic parameters and guards.
5. Produce deterministic target artifacts with explicit delivery policy.
6. Make every specialization assumption visible in the artifact contract.
7. Make fallback and unsupported-operation behavior explicit at integration
   boundaries.
8. Make optimization decisions inspectable with an `EXPLAIN`-style report.
9. Keep numerical contracts and backend capability requirements explicit.
10. Establish reusable boundaries for future frontends and GPU backends.
11. Provide a public experimental semantic graph and operation-extension
    contract so built-in and third-party tensor operations follow the same
    vertical support path.

The proposed Rust integration additionally aims to preserve ordinary inline
macro DX without consumer build scripts, consumer kernel registries, prebuild
commands, or runtime JIT. Its compiler operation-definition registry is an
internal/public compiler API, not an auxiliary consumer kernel registry.

## Non-goals for the first implementation

- Replacing Candle's general execution engine.
- Building a runtime lazy tensor graph.
- Supporting arbitrary user-written GPU programs.
- Guaranteeing a single kernel for every expression.
- Runtime source compilation or autotuning.
- Full CUDA parity in the first milestone.
- Generated backward kernels in the first milestone.
- A core autodiff transformation contract in the first milestone; frontends and
  runtimes own graph eligibility and gradient production initially.
- Graph-level data-dependent control flow, recursion, and semantic loops.
- Cross-device placement, transfers, sharding, distributed collectives, or
  multi-queue scheduling.
- Bitwise equivalence under transformations that deliberately permit floating-
  point reassociation.

## What “optimal” means

The objective is not the largest possible mega-kernel. It is the lowest-cost
valid `KernelProgram` or `ProgramVariant` under a declared numerical contract
and target profile. A program may use multiple kernels when materialization
improves locality, a reduction
requires multiple passes, or fusion harms occupancy enough to outweigh saved
memory traffic.

Optimization should consider:

```text
global-memory traffic
+ intermediate allocation
+ dispatch overhead
+ redundant computation
+ index arithmetic
+ synchronization
+ occupancy loss
+ compilation and artifact cost
```

The initial optimizer will use deterministic heuristics. Its interfaces should
permit empirical calibration and offline autotuning later without adding a
runtime JIT requirement.

## Proposed first-integration success criteria

The first end-to-end milestone succeeds when it can:

- lower a useful einops-derived expression through every documented IR layer;
- generate, compile, and embed a deterministic macro-local metallib bundle;
- execute it through Candle with validated shape, dtype, layout, and offsets;
- fall back correctly when guards fail;
- compare successfully with the unfused Candle result across randomized cases;
- explain its fusion and scheduling decisions;
- demonstrate reduced dispatches or intermediate traffic on at least one
  representative operation chain.
