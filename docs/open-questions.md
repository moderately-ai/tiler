---
schema: "tiler-doc/v1"
id: "tiler.questions.open"
kind: "questions"
title: "Open design questions"
topics: ["decisions", "research", "roadmap"]
questions_status: "active"
related: ["tiler.roadmap"]
---

# Open design questions

This file contains only unresolved work. Accepted invariants live in contracts
and ADRs; ordinary implementation tasks live in the roadmap. Each question has
one owner and an explicit way to close or reconsider it.

## Genuine product decisions

The initial checked shape-evidence spelling is no longer open: ADR 0067 selects
one pinned-nightly dependent-array family. Its conformance harness and
implementation are tracked work rather than product decisions.

### Q-ART-011 — Apple deployment floors

- Owner/tracking: [Metal backend](backends/metal.md), after its compatibility
  experiment below.
- Close when: old/new macOS and real/simulated iOS library-load and
  pipeline-creation evidence exists and Tom selects the supported floors.

## Milestone-owned implementation contracts

These have a correctness-derived direction. They require implementation and
tests, not a product-level choice unless their evidence exposes a new tradeoff.

Ergonomic artifact-family profiles are no longer open: Tom accepted the
consumer-visible spelling on 2026-07-31 under
[`accept-the-inline-artifact-family-profile-syntax`](../tickets/accept-the-inline-artifact-family-profile-syntax.md),
closing what was Q-ART-008. A region states `deliver <profile>;` or
`deliver <family> <minimum>, …;` in its declaration block, the profile vocabulary
is `fallback-only`, `macos`, `ios`, and `macos-and-ios`, and each spelling
resolves through the one canonical `ArtifactFamilySelection` constructor.
[The frontend contract](integration/frontends.md) states the accepted spelling
and what a stated selected family produces while nothing compiles a payload for
it; how the profiles expand is implementation and tests rather than a remaining
choice.

The implementation graph now maps these contracts to bounded coding tickets:

- semantic/index lowering and fusion search: [capability registration](../tickets/prototype-operation-capability-registry.md),
  [checked refinement](../tickets/prototype-semantic-index-refinement.md),
  [canonical index regions](../tickets/prototype-canonical-index-region-slice.md),
  [generic index oracle](../tickets/prototype-index-region-reference-oracle.md),
  [generic region formation](../tickets/prototype-generic-region-formation.md),
  [legality evidence](../tickets/prototype-fusion-legality-and-numerical-proof.md),
  and [complete cover enumeration](../tickets/prototype-region-cover-enumeration.md);
- mature symbolic indexing: [ShapeEnv-backed index bindings](../tickets/implement-shapeenv-index-bindings.md)
  followed by [typed index-domain predicates and proof exchange](../tickets/implement-index-domain-predicates.md);
- physical/kernel/program layers: [target feasibility](../tickets/prototype-target-feasibility-authority.md),
  [checked schedules](../tickets/prototype-scheduled-region-ir.md),
  [physical implementations](../tickets/prototype-physical-implementation-frontier.md),
  [complete physical-plan selection](../tickets/prototype-complete-physical-plan-selection.md),
  [structured KIR](../tickets/prototype-structured-kir-slice.md), and
  separate [kernel-program](../tickets/prototype-kernel-program-ir.md) and
  [artifact-program](../tickets/prototype-artifact-program-model.md) models;
- artifact and Metal AOT: [neutral codec](../tickets/prototype-neutral-artifact-codec.md),
  [MSL lowering](../tickets/prototype-metal-kir-lowering.md),
  [numerical realization](../tickets/prototype-metal-numerical-realization.md),
  [offline driver](../tickets/prototype-apple-aot-driver.md), and
  [bundle assembly](../tickets/prototype-metal-bundle-assembly.md);
- runtime safety: [artifact validation](../tickets/prototype-runtime-artifact-validation.md),
  [preflight](../tickets/prototype-metal-runtime-preflight.md),
  [routing commit](../tickets/prototype-runtime-routing-commit.md), and
  [execution mechanics](../tickets/prototype-metal-runtime-execution.md); and
- inline delivery: [proc-macro frontend](../tickets/prototype-inline-proc-macro-frontend.md),
  [expansion cache](../tickets/prototype-expansion-content-cache.md),
  [artifact-family selection](../tickets/prototype-artifact-family-delivery.md),
  and the [complete inline proof](../tickets/prototype-inline-aot-integration-proof.md).

### Q-SEM-001 — Numerical-policy presets

- Owner/track: [Numerical semantics](numerical-semantics.md),
  [`implement-first-profile-numerical-policies`](../tickets/implement-first-profile-numerical-policies.md).
- Close: versioned preset-to-canonical-per-operation expansion table plus
  round-trip and rejection tests.

### Q-SEM-002 — Built-in algebraic capability declarations

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestone 1.
- Close: complete operation/dtype/signature reassociation and commutativity
  matrix with verifier tests.

### Q-SEM-003 — First-profile operation and dtype support

- Owner/track: [Numerical semantics](numerical-semantics.md) owns tuple meaning; [dtype support maturity](dtype-support.md) owns delivered state by layer; Milestones 1 and 2Q own profile progression. Built-in recognition policy is settled by ADRs 0026–0038, while registration and execution remain separate implementation claims. The bounded governed-F32 and strict-affine U4/U8 slices do not select a first production profile.
- Close: a named first production consumer has an explicit operation/dtype/signature allowlist, and every tuple that profile requires has delivered reference evaluation, optimizer legality, backend execution, runtime semantic enforcement where required, target dispatchability, and bounded conformance evidence. Recognized but unselected families remain visible in the ledger without blocking closure.

### Q-SEM-004 — First-profile transcendental tuples

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestones 1–2.
- Close: operation/dtype/accuracy allowlist with reference and backend
  conformance evidence.
- Backend half, partially supplied and still open: the [Metal elementary-function accuracy guarantee](research/numerics/metal-elementary-function-accuracy.md) record quotes Apple's normative Table 8.1 for `exp` (≤ 4 ULP under Apple's own ULP definition), `rsqrt`, and division at F32 under the governed compile flags, so the backend evidence for those tuples is a cited vendor guarantee rather than `Unknown`. It does not close this question: adopting the `exp` bound needs a registered cross-metric implication because Apple's ULP definition is a different key, adopting any correctly rounded entry needs the rounding mode Metal's §8.2 declines to fix, and neither the exceptional-value contract nor the reference half is supplied. The reference half remains wholly open.

### Q-SEM-005 — First-profile float-to-integer tuples

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestones 1 and
  2Q.
- Close: family/source/destination/rounding allowlist with exceptional and
  boundary tests.

### Q-SEM-007 — Concrete transactional rewrite API

- Owner/track: [Operation extensions](operation-extensions.md),
  [`implement-transactional-rewrite-engine`](../tickets/implement-transactional-rewrite-engine.md).
- Close: Rust API and deterministic recursion, cycle, transaction, and
  per-rule/global budget tests implementing the settled high-level contract.

### Q-SEM-009 — Decomposition versus direct access lowering

- Owner/track: [Operation extensions](operation-extensions.md), Milestone 1.
- Close: per-built-in capability/decomposition table with equivalence tests.

### Q-SHAPE-001 — Runtime extent specialization policy

- Owner/track: [IR](ir.md), Milestones 2–3. Runtime ABI parameters remain the
  default unless specialization is deliberate.
- Close: first-profile policy with identity, guard, and routing tests.

### Q-SHAPE-002 — First-profile composed-axis factor bindings

- Owner/track: [IR](ir.md), Milestone 2.
- Close: static/runtime binding allowlist and complete sourceability tests.

### Q-PLAN-001 — Initial bounded search representation

- Owner/track: [Optimizer](compiler/optimizer.md),
  [`prototype-generic-region-formation`](../tickets/prototype-generic-region-formation.md).
- Close: implementation compared with the exhaustive tiny oracle; introduce a
  memo only if measured quality or cost warrants it.

### Q-PLAN-002 — Shared-work duplication

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  [`implement-general-dag-partitioning`](../tickets/implement-general-dag-partitioning.md).
- Close: legality plus an uncertainty-bearing analytical cost rule checked
  against the exhaustive oracle. Calibrated device selection becomes
  authoritative only after the separate calibration ticket's activation
  conditions and measurements pass.

### Q-PLAN-004 — Coexisting reductions in one kernel

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  [`implement-parallel-reduction-strategies`](../tickets/implement-parallel-reduction-strategies.md).
- Close: topology/order/resource compatibility matrix with positive and
  negative verifier cases.

### Q-PLAN-005 — Physical multi-output kernels

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  [`implement-general-dag-partitioning`](../tickets/implement-general-dag-partitioning.md).
  Semantic multi-result programs are already accepted.
- Close: schedule, ABI, runtime profile, and measured value proof.

### Q-PLAN-007 — First Metal capability keys and feasibility rules

- Owner/track: [Metal backend](backends/metal.md), Milestone 2. The
  [`target-neutral baseline`](../tickets/prototype-target-neutral-baseline-slice.md)
  and [`Metal AOT proof`](../tickets/prototype-metal-aot-slice.md) implement one
  private named prototype fixture without closing the mature profile.
- Close: governed profile/schema with boundary tests and stable explain reasons.

### Q-PLAN-009 — First-profile capability providers and phases

- Owner/track: [Architecture](architecture.md), Milestones 2–3. The general
  phases are settled by ADR 0043. The target-neutral baseline, Metal AOT, and
  [`runtime proof`](../tickets/prototype-metal-runtime-proof.md) must provide the
  bounded enabled-key/provider/phase subset required by their proof.
- Close: complete enabled-key/provider allowlist and preflight tests.

### Q-PLAN-013 — Replayable schedule transforms

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  Milestone 3.
- Close: versioned transform vocabulary with deterministic replay/golden tests.

### Q-ART-002 — Private lockstep serialization

- Owner/track: [Artifact ABI](artifact-abi.md),
  [`prototype-neutral-artifact-codec`](../tickets/prototype-neutral-artifact-codec.md).
  The [`Metal AOT proof`](../tickets/prototype-metal-aot-slice.md) consumes the
  bounded codec through bundle assembly rather than owning serialization.
- Close: deterministic encoder/decoder plus corruption, canonicality, and
  version-rejection tests. This does not promise a public stable format.

### Q-ART-004 — Expansion-cache root, accounting, and GC policy

- Owner/track: [Frontend integration](integration/frontends.md), with the question split in two and only one half still open. Retargeted 2026-07-31: the previous owner [`prototype-expansion-content-cache`](../tickets/prototype-expansion-content-cache.md) closed `done` with this close condition unmet, which left the question owned by a terminal ticket — unowned in fact, the same way Q-ART-008 was. The retarget happened because the trigger the [build-tool exercise](research/cache/build-tool-exercise.md) set for the root half — the first proc-macro frontend crate — fired when `tiler-macros` was admitted on 2026-07-31 under [ADR 0088](decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md).
- **The root half is closed, 2026-07-31.** [The root policy note](research/cache/root-policy.md) records the derivation (`TILER_EXPANSION_CACHE_DIR`, otherwise `$HOME/Library/Caches/ai.moderately.tiler/expansion`), the `off` disable, a typed refusal for every unusable or non-private root, seven eliminated alternatives, and the measurement boundary; `crates/tiler-macros/src/cache_root.rs` implements it with unit tests over every refusal. Tom accepted the consumer-visible spellings that same day under [ADR 0075](decisions/0075-scope-public-boundary-approval-by-change-category.md), [ADR 0089](decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) records the decision, and [the frontend contract](integration/frontends.md) now states the exact derivation rather than only its shape. What is left is wiring rather than a question: nothing calls the resolver, because no expansion opens a cache.
- **The accounting and collection half is open**, with [`decide-the-expansion-cache-collection-schedule`](../tickets/decide-the-expansion-cache-collection-schedule.md) as its owner. Quotas, when a collection runs, and durability diagnostics are all still unowned by any decision; `tiler-cache` supplies the mechanism (`account`, `collect`, `purge`, `CollectionBound`) and deliberately no schedule.
- Close: the root half closed on 2026-07-31, when Tom accepted the spelling and ADR 0089 recorded the decision; the collection half closes on quotas, GC schedule, durability diagnostics, and race tests. Neither half closes the question alone, so this question stays open on the collection half.

### Q-KIR-001 — Conservative uniformity analysis

- Owner/track: [IR](ir.md), Milestone 4.
- Close: scope-sensitive rules with reduction, barrier, convergence, and
  negative-control tests.

### Q-RUNTIME-002 — Affine-strided Candle layouts

- Owner/track: [Candle integration](integration/candle.md),
  [`prototype-candle-metal-adapter`](../tickets/prototype-candle-metal-adapter.md),
  with affine-strided support remaining beyond its contiguous first profile.
- Close: exact stride/offset/alias predicates and guarded differential tests.

### Q-PKG-002 — Rust data APIs and operation capability traits

- Owner/track: [Operation extensions](operation-extensions.md), Milestone 0A.
  ADRs 0005 and 0044 settle the conceptual split. The
  [`resolved-type registry`](../tickets/prototype-resolved-value-type-registry.md),
  [`typed handles`](../tickets/prototype-typed-value-handles.md), and bounded
  [`shaped-value API`](../tickets/prototype-shaped-value-api.md) now have
  integrated compile/UI proofs. Broader trait ergonomics and stabilization
  remain active rather than blocking the bounded prototype.
- Close: concrete visibility and trait ergonomics with compile/UI tests.

### Q-PKG-003 — Proc-macro to Metal-AOT visibility

- Owner/track: [Frontend integration](integration/frontends.md),
  [`prototype-inline-proc-macro-frontend`](../tickets/prototype-inline-proc-macro-frontend.md).
- Close: private-by-default visibility audit and compile/UI tests while formats
  remain lockstep.

## Bounded evidence gates

### Q-PLAN-008 — Multi-family target-profile compatibility

- Owner/track: [Architecture](architecture.md), Milestone 7.
- Close: versioned capability-intersection rules backed by cross-family,
  device, and OS measurements; unmeasured guarantees remain unknown.

### Q-ART-003 — Additional embedding-platform matrices

- Owner/track: [Artifact ABI](artifact-abi.md), Milestone 7.
- Run when: proposing new delivery platforms or changing the current 1 MiB per
  invocation and 32-invocation/3.2 MiB package gates.

### Q-ART-006 — rust-analyzer cold and warm expansion costs

- Owner/track: [Frontend integration](integration/frontends.md), Milestone 0B.
- Close: exact cold/warm/edit/cache/compiler-invocation matrix.
- The availability blocker is resolved. The
  [build-tool exercise](research/cache/build-tool-exercise.md) drove real
  expansions through the pinned toolchain's own
  `rust-analyzer-proc-macro-srv` — which is the process that expands, and ships
  with the pin even though the LSP binary is not a pinned component — and
  recorded cold and warm cache resolutions under both drivers. What remains is
  the *edit* column: that needs a real language-server session rather than
  `analysis-stats`, which loads a project and expands once.

### Q-ART-007 — Apple cross-machine, patch-toolchain, and runtime-compiler evidence

- Owner/track: [Metal backend](backends/metal.md), Milestone 7.
- Close: a reproducibility and compatibility matrix over four independent axes — the machine and GPU, the Xcode toolchain patch version, the **OS build**, and the **installed simulator runtime version**. The last two are axes in their own right because a host that never changes Xcode can still change two of its three Metal compilers: the offline driver ships with Xcode, the macOS runtime compiler with the OS, and a booted simulator's with that runtime, as [Metal backend](backends/metal.md#compiler-provenance-and-the-runtime-compiler) records. Read without them, this question is satisfied by a matrix that holds the OS constant, and the numerical harness then announces an environment-row difference and declines to compare rather than confirming agreement.
- Closing measurement: re-run [`numerical_probe.py`](../spikes/apple-targets/numerical_probe.py) on a host whose OS build differs while its Xcode build does not, and again against a second installed simulator runtime version, comparing the resulting `environment.family.<name>.runtime_compiler_build` rows with the retained record. A run whose rows are unchanged has not exercised the axis and does not close it.

### Q-ART-011-E — Apple deployment-minimum compatibility experiment

- Owner/track: [Metal backend](backends/metal.md), prerequisite to Q-ART-011.
- Close: record whether incompatibility fails at library load or pipeline
  creation across old/new macOS, iOS devices, and simulators.

## Deferred until an explicit trigger

### Q-SEM-006 — Additional quantization schemes

- Owner: [Numerical semantics](numerical-semantics.md).
- Trigger: strict affine Milestone 2Q is complete and a named workload requires
  another exact scheme.

### Q-SEM-011 — Semantic effects and resource tokens

- Owner: [Operation extensions](operation-extensions.md).
- Trigger: the first stateful, mutating, or hidden-random operation proposal;
  closure requires ordering, liveness, verification, ABI, and failure rules.

### Q-SEM-012 — Semantic modules, calls, and control flow

- Owner: [IR](ir.md).
- Trigger: a workload requires reusable graph functions, interprocedural
  optimization, recursion, or structured control flow.

### Q-SEM-013 — Differentiation ownership

- Owner: [Architecture](architecture.md).
- Trigger: backward-kernel compilation enters the roadmap; closure requires a
  product-layer and semantic/autograd decision.

### Q-SEM-015 — Tensor contraction: matmul, batched matmul, and einsum

- Owner/tracking: the [Milestone 6 framing](roadmap.md#framing-what-a-tensor-contraction-family-would-impose), [`scope-einsum-contraction-support`](../tickets/scope-einsum-contraction-support.md). The [operation-family support matrix](roadmap.md#operation-family-support-matrix) records this family at R6 for a whole-program contraction occurrence since 2026-08-01 — a registered identity with a host reference evaluator, all three of the pinned workload's index structures admitted as structure values, an eighth governed lowering capability, and the `direct` realization's schedule constructs and Metal emission — with no fusion role and no execution row. What remains of the planning half is what this question still owns: contraction-order exploration, GEMM recognition, layout-conversion costing, and the `tiled` schedule. "Contraction" here always means the tensor sense — summation over indices shared by two or more operands — and never ADR 0015's fused-multiply-add permission, which is a separate field of the numerical contract that happens to govern a tensor contraction's own per-contributor step.
- Trigger: a named workload or frontend lowering requires a tensor contraction — fired by the pinned L1 workload. Closure of the semantic half needs an accepted decision fixing what establishes a contraction's identity, what its operation definition rejects at construction, and which access relation it emits; none of those depends on a backend. That decision must settle three choices, and two of the three are now settled. The first: [ADR 0087](decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) accepts one keyed family carrying a renaming-invariant index-structure attribute, on the L2 derivation's three-structure evidence. The third: [ADR 0095](decisions/0095-decline-a-distributivity-permission.md) **declines** a distributivity permission, so [Numerical semantics](numerical-semantics.md#distributivity-is-outside-the-order-contract) continues to define the dimension, admit no permission for it, and reject contraction-chain regrouping — now as a decided position rather than a reserved one, with contraction ordering remaining a planning question within one semantic contraction. Its reopening trigger is the first workload whose natural spelling is a directly regroupable chain, and its dependent question, [`decide-whether-distributivity-directions-share-one-permission`](../tickets/decide-whether-distributivity-directions-share-one-permission.md), does not arise under a decline and stays parked. Still reserved from the framing, and the only one of the three left: whether a semantic contraction node may consume more than two operands. The three are independent: the distributivity derivation, and therefore ADR 0095's decline, holds under either answer to the multi-operand choice.
- Gate: no contraction *planning* work — contraction-order exploration, GEMM recognition, layout-conversion costing, or direct and tiled schedules — may be scheduled until [`prototype-optimizer-conformance-gate`](../tickets/prototype-optimizer-conformance-gate.md) closes and a backend has executed a compiled program, which [`prototype-metal-aot-slice`](../tickets/prototype-metal-aot-slice.md) and [`prototype-metal-runtime-proof`](../tickets/prototype-metal-runtime-proof.md) own. **All three are `done`, so the gate is open, and `realize-the-contraction-through-the-appendable-direct-path` was the first work to pass through it on 2026-08-01.** The two limits below are the evidence the gate rested on, and both have since been lifted deliberately rather than eroded: `normalize_contraction` in `crates/tiler-compiler/src/request.rs` is a third recognized whole-program strategy admitting exactly two inputs, and `governed_index_access_capabilities` registers an eighth capability covering a contraction occurrence. What the gate still holds back is everything the `direct` path did not deliver — contraction-order exploration, GEMM recognition, layout-conversion costing, and the `tiled` schedule, the last of which additionally waits on [`admit-the-first-typed-synchronization-point-and-atomic-target-authority`](../tickets/admit-the-first-typed-synchronization-point-and-atomic-target-authority.md). The original statement of the two limits follows, preserved because it is what the gate's derivation cited: `normalize_serial_sum` rejected any program that did not have exactly one input, so a binary contraction could not reach the compiler at all, and no registered lowering capability covered a contraction occurrence, so resolution would fail closed rather than lower one. **Corrected by `correct-the-surviving-stale-one-contract-claims`.** The second limit previously read that `crates/tiler-compiler/src/capability.rs` and `crates/tiler-compiler/src/legality.rs` are draft authorities with no in-crate production caller. That was true when written and was falsified by `wire-capability-and-refinement-into-compile-path`: `pipeline::compile` calls `resolve_lowering`, which consumes both modules, and governed index-access providers are registered — four then, and six since `admit-the-reindex-and-broadcast-operation-families` added one for each structural family. [The Milestone 6 framing](roadmap.md#framing-what-a-tensor-contraction-family-would-impose) records the same correction and is the surviving statement of it. The gate is unaffected, because what closes it is the absence of a capability for a *contraction* occurrence rather than the absence of a caller — and stating it that way is the invariant, which the fifth and sixth registered providers did not silently falsify. Both limits belong to that gate. [Fusion and scheduling](compiler/fusion-and-scheduling.md) independently requires contraction planning to follow, not precede, the boundary-contract and cost infrastructure.

### Q-SHAPE-004 — Dynamic-rank semantic values

- Owner: [IR](ir.md).
- Trigger: a concrete workload cannot be represented as static-rank variants.

### Q-SHAPE-005 — Device-produced shapes and indirect dispatch

- Owner: [IR](ir.md).
- Trigger: a selected operation requires device-produced extents; closure needs
  a host/device `ShapeProgram`, synchronization, publication, and guard contract.

### Q-SHAPE-006 — Finite piecewise access maps

- Owner: [IR](ir.md).
- Trigger: a named workload is not expressible in the admitted access language.

### Q-SHAPE-007 — Indirect gather/scatter relations

- Owner: [IR](ir.md).
- Trigger: gather/scatter enters an active product profile; closure needs bounds,
  duplicate-write, determinism, and validation rules.

### Q-SHAPE-008 — Negative-stride ABI support

- Owner: [IR](ir.md), after Milestone 3.
- Trigger: signed reachable-range proof and backend/runtime layout support.

### Q-PLAN-011 — CPU execution and vector profile

- Owner: [CPU backend](backends/cpu.md).
- Trigger: the CPU backend enters the active roadmap.

### Q-PLAN-015 — Advanced buffer reuse and in-place execution

- Owner: [Architecture](architecture.md), after Milestones 3/5.
- Trigger: memory/performance data shows the conservative allocation plan is
  insufficient.

### Q-PLAN-016 — Multi-device and sharded planning

- Owner/tracking: [Architecture](architecture.md),
  [`multi-device-and-sharding-scope-gate`](../tickets/multi-device-and-sharding-scope-gate.md).
- Trigger: a selected product workload requires multiple devices or sharding.

### Q-PLAN-018 — External storage and out-of-core orchestration

- Owner/tracking: [Architecture](architecture.md),
  [`external-storage-resource-scope-gate`](../tickets/external-storage-resource-scope-gate.md).
- Trigger: a selected workload requires file-backed, mapped, evicted, or
  out-of-core tensor resources.

### Q-ART-009 — Binary archives and dynamic Metal libraries

- Owner: [Metal backend](backends/metal.md), Milestone 7.
- Trigger: measured startup or bundle-size cost exceeds a documented gate.

### Q-ART-010 — Public serialized-IR compatibility

- Owner: [Artifact ABI](artifact-abi.md), Milestone 7.
- Trigger: a stable external reader/writer use case exists and IR boundaries
  have settled.

### Q-ART-012 — Catalyst artifact support

- Owner: [Metal backend](backends/metal.md).
- Trigger: an integration requires Catalyst; closure needs an explicit family,
  deployment, `cfg`, compile, and runtime compatibility profile.

### Q-KIR-002 — Asynchronous copies and split-phase barriers

- Owner: [IR](ir.md).
- Trigger: a selected pipelined workload needs overlap not expressible by total
  phases.

### Q-KIR-003 — Target-specific lowering IR

- Owner: [IR](ir.md).
- Trigger: a target operation cannot faithfully lower from common structured
  KIR without polluting target-independent semantics.

### Q-KIR-004 — General CFGs, pointers, calls, and aliasing

- Owner: [IR](ir.md).
- Trigger: a demonstrated workload falls outside bounded structured tensor
  kernels and justifies the larger verifier surface.

### Q-RUNTIME-001 — Candle input arity beyond `CustomOp3`

- Owner: [Candle integration](integration/candle.md), Milestone 5.
- Trigger: a profitable selected region exceeds Candle arity and cannot be
  soundly partitioned.

### Q-RUNTIME-004 — Tracked/autograd fusion

- Owner: [Candle integration](integration/candle.md).
- Trigger: backward support enters an explicitly authorized phase.
