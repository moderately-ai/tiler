---
id: accept-the-production-boundary-for-the-bounded-scalar-cpu-backend
title: Accept the production boundary for the bounded scalar CPU backend
status: awaiting-decision
priority: p1
dependencies: [join-build-time-producers-to-runtime-adapters-through-artifact-identity]
related: [prototype-a-bounded-scalar-cpu-backend-vertical, design-the-cpu-vector-lane-tier, exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio, share-one-structured-kernel-interpreter]
scopes: [contracts/artifacts, contracts/decisions, research/target-profiles, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, backend-providers, decision, needs-tom, public-boundary]
---
## User-visible outcome

Tiler has an accepted, consumer-nameable scalar CPU backend boundary parallel to the Metal backend family: verified structured kernels translate to a versioned CPU payload, an eligible host decodes and executes that payload against real host storage and arithmetic, and the result can be compared independently with `tiler-reference`. The initial profile is deliberately scalar and single-threaded; unsupported vector, threaded, barrier, dtype, and packed paths refuse by name.

## Accepted direction — 2026-08-12

Tom accepted production CPU execution as the destination of the host-KIR work and rejected leaving either test `KirMachine` as the repository's CPU implementation. The bounded scalar vertical is the source evidence and starting point. This ticket does **not** reopen whether CPU is a real sibling backend; it decides the exact production crate/responsibility/API boundary needed to promote it.

## Source-first Facts at `449d54b864b849993692e8bf12117f9064f76b4d`

- **Verified — a real vertical already executes.** `spikes/target-profiles/scalar-cpu-vertical/README.md`, anchor `A second backend, materially different from Metal`, records target-profile declaration, KIR translation, a serialized `tiler.cpu.scalar-image-v1` payload, artifact encoding/decoding, live host qualification, routing commit, execution, and bitwise comparison with `tiler-reference`.
- **Verified — the representation is not a relabelled evaluator.** `src/image.rs`, anchor `A backend payload has to be bytes a host can execute without holding the compiler`, translates `VerifiedKernel` into self-describing bytes. `src/interpret.rs`, anchor `one invocation at a time, in ascending grid index`, executes the decoded image without compiler objects.
- **Verified — the host is an earned execution context.** `src/host.rs`, anchor `binds a live execution context by measuring`, observes architecture, OS, pointer width, byte order, and input/result subnormal behaviour before commit.
- **Verified — the current CPU contract remains proposed.** `docs/backends/cpu.md`, anchor `future contract sketch; not an implementation commitment`, says the spike covers only the scalar half and explicitly leaves vector, scalable-vector, masks/tails, threading, and caches unknown.
- **Verified — backend composition is per responsibility, not one provider bundle.** ADR 0090, anchor `A monolithic Device or BackendProvider trait`, rejects a single object spanning compile, build, artifact, and runtime lifetimes. A CPU crate may own several coherent responsibilities, but must not introduce that bundle or a registry.
- **Verified — an independently nameable CPU owner is now required.** `docs/research/runtime/backend-scoped-route-requirement-answers.md`, anchor `There is no tiler-cpu`, says a CPU backend-scoped fact has no consumer-nameable vocabulary owner until a CPU crate is admitted.
- **Verified — the current host simulators are not this backend.** The two `KirMachine`s execute in-memory `VerifiedKernel` test fixtures; they carry no payload codec, artifact, host qualification, runtime route, or complete launch contract.
- **Verified — the spike is not a production adapter.** `src/vertical.rs`, anchor `fn prepare_route`, allocates program storage while a `Preflight` is still held and calls `preflight.commit()` only afterwards. The accepted `RuntimeAdapter` sequence requires sizing before commit and allocation after it. The spike also assembles `tiler-artifact` directly; `docs/research/extensions/backend-provider-composition.md`, anchor `A production second backend cannot take that route`, requires production publication to use the neutral build/cache/correspondence seam.
- **Verified — the live adapter seam cannot yet state the CPU adapter's honest prepared-property answer.** `RuntimeAdapter::observe_prepared_entry` returns a bare `u64`. This scalar profile carries zero prepared-property rows, but an adapter still implements the total method; returning `0` or `u64::MAX` would let a future unknown key accidentally compare equal. `make-prepared-entry-observations-typed-and-key-dispatched` owns the required `Quantity | Unrecognized` repair.
- **Verified — the retained measurement is narrower than the implementation surface.** The spike executes one F32 pointwise program and one stage. Its multi-entry route, shared-allocation pairing, serial-loop execution, and operation-level refusals are implemented but not exercised by the retained run; promotion must not round them up to tested guarantees.

## Decision required

Choose the narrowest production ownership boundary that preserves four separations:

1. translation from verified KIR versus decoding/executing already-published bytes;
2. CPU backend facts versus target-neutral compiler/build/runtime procedure;
3. backend execution versus the independent semantic oracle;
4. scalar support now versus vector/threaded additions later.

The decision must compare at least:

- one `tiler-cpu` crate containing the versioned image vocabulary, translator, decoder, host qualification, and executor while exposing no monolithic provider object;
- a split producer/runtime package shape that keeps runtime-only consumers from depending on KIR translation dependencies;
- a native AOT CPU representation as a later performance tier rather than a prerequisite for the correctness baseline.

State exact dependency direction, public constructors and refusal types, representation/version ownership, host-preflight phase, build-orchestration join, and whether the scalar interpreter remains a supported baseline when a native tier arrives. Do not decide by mirroring Metal's crate topology: the accepted architecture says responsibilities compose and a second backend may package them differently.

## Revised recommendation after the dependency audit

Use three concrete CPU-owned packages, divided by the process boundary and dependency direction rather than by a generic provider abstraction:

1. **`tiler-cpu-image`** — the governed backend/representation vocabulary, canonical scalar-image grammar, checked encoder/decoder, and pure decoded-image execution engine. It depends on neither compiler/build nor runtime routing. Its received-image types are opaque or validated; any producer-facing construction surface is explicit and fallible rather than raw public fields.
2. **`tiler-cpu`** — target-profile declaration plus verified-KIR-to-image translation and payload production. It depends inward on `tiler-ir`, the compiler target/profile vocabulary, `tiler-cpu-image`, and the neutral build seam. It does not execute artifacts or implement `RuntimeAdapter`.
3. **`tiler-cpu-runtime`** — live host observation, exact CPU `RuntimeAdapter`, placement/allocation, decoded-image execution, and owned completion. It depends on `tiler-cpu-image`, `tiler-artifact`, and `tiler-runtime`, and never on the compiler, build orchestrator, or KIR.

This is not a `BackendProvider` bundle or a registry. A consumer explicitly chooses the CPU adapter and passes it to `route_with_adapter`; producer and adapter join through the artifact's backend/representation/profile bytes. `tiler-conformance` is the first in-tree executed consumer and `exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio` exercises separate explicit Metal and CPU attempts.

The strongest alternative is one leaf `tiler-cpu` crate containing all three responsibility groups. It can be made correct, but its runtime consumer then acquires compiler/build/KIR dependencies or its build consumer acquires runtime dependencies, weakening boundaries the existing loader and AOT-driver splits deliberately enforce. Cargo features do not repair that cleanly: feature unification makes dependency shape ambient, and this workspace currently admits no feature-partition precedent. The three-package form therefore wins on long-term dependency correctness and compile/runtime closure at the cost of two manifests and explicit inter-package received-identity APIs.

A future native CPU compiler is a separate versioned representation and may reuse the target/profile and runtime qualification vocabulary. It does not replace or silently shadow the scalar image. The scalar representation remains an explicitly selected correctness/debug baseline; no cross-representation or cross-family fallback is inferred.

The adapter constructor must take an explicit execution-resource policy with no default. A liberal alpha caller may state an unbounded policy explicitly; a bounded caller may cap statically derived worst-case scalar instructions and allocation bytes. The image carries literal loop ranges and the route carries launch geometry, so the adapter can compute a checked upper bound during `plan_dispatch`, before commit, without executing or allocating. Exhaustion is a typed route refusal, never truncation or partial execution.

The existing `LiveDevicePreflight` naming seam does not block the scalar slice because it carries no live-device or prepared-entry requirement rows: host qualification happens while binding the execution context and validating the exact image/profile. `name-a-host-process-availability-phase` remains deferred until a CPU route fact actually needs a governed availability row. Do not add a phase merely to rename an internal adapter step.

## Non-goals

- No SIMD, scalable vector, threads, barriers, cooperative workgroups, BF16, packed quantization, or performance claim.
- No automatic Metal-to-CPU fallback. Each routing attempt states one backend environment explicitly.
- No reuse of either test `KirMachine` as production code.
- No semantic evaluation in the CPU backend; `tiler-reference` remains the oracle.
- No feature-gated single-crate facade that changes dependency or authority shape according to Cargo feature unification.

## Closes when

Tom accepts the exact production package/responsibility/API boundary, the CPU backend contract and architecture records can state it without overclaiming vector or threaded support, and the implementation ticket has an unambiguous owner for every translator, codec, host-preflight, build, artifact, runtime, and conformance responsibility. Acceptance also decides whether the three-package recommendation survives its strongest one-crate counterpoint and accepts the explicit execution-resource-policy boundary.
