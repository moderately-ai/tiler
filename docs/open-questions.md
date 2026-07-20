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

### Q-PLAN-017 — First Metal value-proof workload

- Owner/tracking: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  [`research-readiness-gate`](../tickets/research-readiness-gate.md).
- Close when: Tom chooses the proposed strict serial `f32` Sum profile versus
  reduction-free plumbing for the first Metal proof.

### Q-PHASE-001 — Implementation-phase authorization

- Owner/tracking: [`research-readiness-gate`](../tickets/research-readiness-gate.md),
  after Q-PLAN-017.
- Close when: Tom explicitly authorizes, narrows, or declines implementation of
  the selected proof. Authorization creates the crate-layout, MSRV, and
  vertical-slice tickets; it does not silently settle those follow-up choices.

### Q-PKG-001 — Initial workspace crate boundaries

- Owner/tracking: [Architecture](architecture.md), before Roadmap Milestone 0A
  implementation.
- Close when: Tom accepts a concrete crate graph that preserves the documented
  dependency direction without making a consumer or backend part of core.

### Q-PKG-004 — Minimum supported Rust version

- Owner/tracking: [Architecture](architecture.md), before scaffolding.
- Close when: Tom selects an MSRV, thereby choosing standard `File` locking or
  requiring a separately audited compatible lock adapter.

### Q-ART-011 — Apple deployment floors

- Owner/tracking: [Metal backend](backends/metal.md), after its compatibility
  experiment below.
- Close when: old/new macOS and real/simulated iOS library-load and
  pipeline-creation evidence exists and Tom selects the supported floors.

## Milestone-owned implementation contracts

These have a correctness-derived direction. They require implementation and
tests, not a product-level choice unless their evidence exposes a new tradeoff.

### Q-SEM-001 — Numerical-policy presets

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestone 1.
- Close: versioned preset-to-canonical-per-operation expansion table plus
  round-trip and rejection tests.

### Q-SEM-002 — Built-in algebraic capability declarations

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestone 1.
- Close: complete operation/dtype/signature reassociation and commutativity
  matrix with verifier tests.

### Q-SEM-003 — First-profile operation and dtype support

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestones 1 and
  2Q. Dtype recognition itself is settled by ADRs 0026–0038.
- Close: every admitted tuple has explicit reference, optimization, and backend
  support state.

### Q-SEM-004 — First-profile transcendental tuples

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestones 1–2.
- Close: operation/dtype/accuracy allowlist with reference and backend
  conformance evidence.

### Q-SEM-005 — First-profile float-to-integer tuples

- Owner/track: [Numerical semantics](numerical-semantics.md), Milestones 1 and
  2Q.
- Close: family/source/destination/rounding allowlist with exceptional and
  boundary tests.

### Q-SEM-007 — Concrete transactional rewrite API

- Owner/track: [Operation extensions](operation-extensions.md), Milestone 1.
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

- Owner/track: [Optimizer](compiler/optimizer.md), Milestone 2.
- Close: implementation compared with the exhaustive tiny oracle; introduce a
  memo only if measured quality or cost warrants it.

### Q-PLAN-002 — Shared-work duplication

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  Milestone 5.
- Close: legality gate and calibrated cost rule checked against the exhaustive
  oracle.

### Q-PLAN-004 — Coexisting reductions in one kernel

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  Milestone 4.
- Close: topology/order/resource compatibility matrix with positive and
  negative verifier cases.

### Q-PLAN-005 — Physical multi-output kernels

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  Milestone 5. Semantic multi-result programs are already accepted.
- Close: schedule, ABI, runtime profile, and measured value proof.

### Q-PLAN-007 — First Metal capability keys and feasibility rules

- Owner/track: [Metal backend](backends/metal.md), Milestone 2.
- Close: governed profile/schema with boundary tests and stable explain reasons.

### Q-PLAN-009 — First-profile capability providers and phases

- Owner/track: [Architecture](architecture.md), Milestones 2–3. The general
  phases are settled by ADR 0043.
- Close: complete enabled-key/provider allowlist and preflight tests.

### Q-PLAN-013 — Replayable schedule transforms

- Owner/track: [Fusion and scheduling](compiler/fusion-and-scheduling.md),
  Milestone 3.
- Close: versioned transform vocabulary with deterministic replay/golden tests.

### Q-ART-002 — Private lockstep serialization

- Owner/track: [Artifact ABI](artifact-abi.md), Milestones 0A–0B.
- Close: deterministic encoder/decoder plus corruption, canonicality, and
  version-rejection tests. This does not promise a public stable format.

### Q-ART-004 — Expansion-cache root, accounting, and GC policy

- Owner/track: [Frontend integration](integration/frontends.md), Milestones 0B
  and 7.
- Close: private defaults, quotas, GC, durability diagnostics, and race tests.

### Q-ART-008 — Ergonomic artifact-family profiles

- Owner/track: [Frontend integration](integration/frontends.md), Milestone 0B.
- Close: named profiles expand to canonical `ArtifactFamilySelection` with
  generated `cfg` compile-pass/fail tests.

### Q-KIR-001 — Conservative uniformity analysis

- Owner/track: [IR](ir.md), Milestone 4.
- Close: scope-sensitive rules with reduction, barrier, convergence, and
  negative-control tests.

### Q-RUNTIME-002 — Affine-strided Candle layouts

- Owner/track: [Candle integration](integration/candle.md), Milestone 3.
- Close: exact stride/offset/alias predicates and guarded differential tests.

### Q-PKG-002 — Rust data APIs and operation capability traits

- Owner/track: [Operation extensions](operation-extensions.md), Milestone 0A.
  ADRs 0005 and 0044 settle the conceptual split.
- Close: concrete visibility and trait ergonomics with compile/UI tests.

### Q-PKG-003 — Proc-macro to Metal-AOT visibility

- Owner/track: [Frontend integration](integration/frontends.md), Milestone 0B.
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
- Close: exact cold/warm/edit/cache/compiler-invocation matrix when a real
  rust-analyzer binary, rather than only the rustup proxy, is available.

### Q-ART-007 — Apple cross-machine and patch-toolchain evidence

- Owner/track: [Metal backend](backends/metal.md), Milestone 7.
- Close: reproducibility and compatibility matrix across machines and
  toolchain patch versions.

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
