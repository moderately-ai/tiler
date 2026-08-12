---
schema: "tiler-doc/v1"
id: "ADR-0110"
kind: "decision"
title: "Split the bounded scalar CPU backend at the production process boundaries"
topics: ["backends", "cpu", "artifacts", "runtime", "packaging", "public-boundary"]
catalog_group: "runtime-integration-placement"
decision_status: "accepted"
implementation_status: "spike-only"
applies_to: ["tiler.contract.architecture", "tiler.contract.cpu-backend"]
evidence: ["tiler.research.extensions.backend-provider-composition", "tiler.research.target-profiles.physical-feasibility-model"]
depends_on: ["ADR-0051", "ADR-0074", "ADR-0075", "ADR-0090"]
ticket: "accept-the-production-boundary-for-the-bounded-scalar-cpu-backend"
---

# 0110: Split the bounded scalar CPU backend at the production process boundaries

**Status:** accepted by Tom on 2026-08-12 in the current Codex session. Tom accepted the three-package boundary, the explicit resource-policy requirement, and physical deletion of the two test simulators after authoritative production evidence replaces them. This record accepts a public ownership and dependency direction; it does not claim that any CPU production crate exists yet.

## Context

The bounded scalar CPU vertical executes one F32 program from a declared CPU target profile through a versioned serialized image, target-neutral artifact bytes, live host qualification, one-way routing commit, decoded-image execution, and bitwise comparison with `tiler-reference`. It proves that CPU is a real sibling backend family rather than a fallback spelling for Metal, but it is a spike: it allocates program storage before routing commit, bypasses the neutral build orchestrator, implements no production `RuntimeAdapter`, and exercises only one pointwise stage.

Two in-memory `KirMachine` test helpers also execute structured kernels. They are useful local test machinery, but neither is a backend: each consumes compiler-owned `VerifiedKernel` values directly and carries no versioned payload, artifact, host qualification, route, or complete launch contract. Sharing those simulators would create a third semantic execution authority without delivering a consumer-nameable CPU device path.

ADR 0090 rejects a monolithic backend-provider object spanning compile, build, artifact, and runtime lifetimes. The same reasoning applies to package ownership. A single CPU package can be correct, but it makes runtime-only consumers acquire compiler/build/KIR dependencies or makes build-time consumers acquire runtime routing and host-execution dependencies. Cargo features do not create a stable substitute for that boundary because feature unification makes the dependency shape ambient.

## Decision

### 1. The production scalar CPU backend is three packages with one-way dependencies

The accepted package responsibilities are:

- `tiler-cpu-image` owns the governed CPU backend and representation vocabulary, the canonical scalar-image grammar, checked encoding and decoding, and the pure decoded-image execution engine. It depends on neither compiler/build code nor runtime routing. Received image values are validated and opaque; producer construction is explicit and fallible.
- `tiler-cpu` owns scalar CPU target-profile declaration, verified-KIR-to-image translation, and payload production through the neutral build seam. It may depend inward on `tiler-ir`, the compiler's target/profile vocabulary, `tiler-cpu-image`, and the build-time interface. It does not decode artifacts for execution and does not implement `RuntimeAdapter`.
- `tiler-cpu-runtime` owns live host observation, the exact CPU `RuntimeAdapter`, placement and allocation, decoded-image execution, and owned completion. It depends on `tiler-cpu-image`, `tiler-artifact`, and `tiler-runtime`, and never on `tiler-compiler`, `tiler-build`, or KIR.

These are concrete responsibilities, not a `BackendProvider` bundle, backend registry, or facade whose Cargo features change its authority. A consumer explicitly selects the CPU adapter and passes it to the neutral routing procedure. Producer and adapter meet only through the artifact's backend family, representation, target profile, and payload bytes.

### 2. The first admitted representation is an explicit scalar correctness baseline

The initial support profile is single-threaded scalar F32 only. Every vector, scalable-vector, packed, threaded, barrier, cooperative-workgroup, BF16, and unimplemented operation or numerical realization remains a typed refusal. The existing `tiler.cpu.scalar-image-v1` identity may be retained only if the production grammar is byte-for-byte identical to the spike grammar; otherwise the production representation receives a new version.

A future native CPU compiler is a distinct versioned representation. It may reuse CPU target and host-qualification vocabularies, but it neither replaces nor silently shadows the scalar image. Callers select one complete representation and routing environment explicitly. There is no inferred Metal-to-CPU, native-to-scalar, or scalar-to-native fallback.

### 3. CPU execution policy is explicit, checked before commit, and never truncates

Constructing the CPU runtime adapter requires an execution-resource policy. There is no default. A liberal alpha caller may explicitly choose an unbounded policy; a bounded caller states limits on checked worst-case scalar operations and allocation bytes. The adapter derives the bound from decoded literal loop ranges, launch geometry, instruction structure, and buffer requirements during reversible preflight. Exceeding it is a typed route refusal before commit, never partial execution, truncation, or fallback after side effects.

Planning may decode, validate, size, and prepare disposable state before routing commit. Program storage and output/temporary allocation occur only after the consuming one-way commit, preserving ADR 0051. Unknown live or prepared property keys return a typed `Unrecognized` observation; numeric sentinels are forbidden even when the first profile carries zero such requirements.

### 4. The oracle and the retired simulators keep separate roles

`tiler-reference` remains the semantic oracle. The CPU image executor implements physical program semantics and is compared independently with that oracle; it does not call or reuse reference evaluation.

The two test `KirMachine` implementations are physically deleted only after production CPU evidence subsumes their named test populations. Tests that need a production CPU result use the production image/executor path. Tests that need a local verifier unit retain narrowly scoped assertions rather than another executable interpreter. No compatibility wrapper, deprecated alias, mock backend, or fake device remains to confuse future readers.

## Consequences

- The repository gains an accepted consumer-nameable CPU backend boundary parallel to the Metal backend family without mirroring Metal's package topology or introducing a backend registry.
- Three package manifests and received-identity joins cost more initially than one leaf crate, but runtime and build dependency closures remain truthful and future native execution does not force compiler dependencies into runtime consumers.
- The scalar interpreter remains maintained production code as a correctness/debug representation. Performance claims require a separately measured native representation; they are not inferred from correctness evidence.
- The current spike is retained as bounded evidence until the production path reproduces and strengthens its refusal and bitwise-agreement population. It is then archived or removed under the evidence-retention ticket rather than becoming a second implementation.
- No artifact, cache, KIR, or semantic identity changes merely because this boundary is accepted. Identities move only when the production packages add new declared profiles, representations, payloads, or selected artifact content.
- The scalar slice does not require a new host-process availability phase because it carries no governed availability row. A phase is added only when a real CPU route predicate needs one.

## Alternatives considered

**One `tiler-cpu` package.** Correct if carefully layered internally and the strongest alternative. Rejected because either runtime execution reaches compiler/build/KIR dependencies or build-time production reaches runtime routing. Cargo features make that closure caller-ambient rather than structurally owned.

**A monolithic `BackendProvider` or backend registry.** Rejected by ADR 0090's responsibility model. Translation, build orchestration, artifact publication, runtime adaptation, and execution are independently selected and fixed at different times.

**Use or merge the two `KirMachine` test interpreters.** Rejected because they execute compiler handles, carry no production payload or host qualification, and would preserve a second executable authority after the real backend exists.

**Start with native AOT CPU code.** Deferred as a later representation. It adds toolchain, code-memory, platform ABI, security, and cache questions before the architecture has one production CPU correctness path.

**An implicit fallback CPU.** Rejected because it hides backend selection and changes failure timing. A caller may attempt another complete equivalent route only through the explicit pre-commit policy that owns that choice.

## Traceability

[The CPU backend contract](../backends/cpu.md) owns the accepted scalar boundary and the still-proposed SIMD/threaded profile. [The system architecture](../architecture.md) owns package responsibility and dependency direction. [The consumer-neutral backend-provider composition record](../research/extensions/backend-provider-composition.md) derives the responsibility split, and [target-profile feasibility research](../research/target-profiles/physical-feasibility-model.md) owns the typed target/profile model. The [bounded scalar CPU vertical](../../spikes/target-profiles/scalar-cpu-vertical/README.md) is the executable evidence and states its measurement limits. [`accept-the-production-boundary-for-the-bounded-scalar-cpu-backend`](../../tickets/accept-the-production-boundary-for-the-bounded-scalar-cpu-backend.md) records the source-first audit and acceptance; [`promote-the-bounded-scalar-cpu-vertical-into-a-production-backend`](../../tickets/promote-the-bounded-scalar-cpu-vertical-into-a-production-backend.md) owns implementation.

## Implementation boundary

Acceptance authorizes the three package responsibilities and their included/excluded public roles, not unrestricted implementation breadth. Production work remains blocked on typed prepared-entry observations, must use the neutral build and runtime seams, must preserve the initial scalar refusal set, and must pass a fresh public-surface review if implementation discovers a consequential API shape not fixed here.
