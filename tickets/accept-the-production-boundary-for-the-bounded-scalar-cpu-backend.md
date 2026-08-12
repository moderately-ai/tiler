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

## Non-goals

- No SIMD, scalable vector, threads, barriers, cooperative workgroups, BF16, packed quantization, or performance claim.
- No automatic Metal-to-CPU fallback. Each routing attempt states one backend environment explicitly.
- No reuse of either test `KirMachine` as production code.
- No semantic evaluation in the CPU backend; `tiler-reference` remains the oracle.

## Closes when

Tom accepts the exact production package/responsibility/API boundary, the CPU backend contract and architecture records can state it without overclaiming vector or threaded support, and the implementation ticket has an unambiguous owner for every translator, codec, host-preflight, build, artifact, runtime, and conformance responsibility.
