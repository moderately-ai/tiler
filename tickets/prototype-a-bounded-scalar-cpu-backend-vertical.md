---
id: prototype-a-bounded-scalar-cpu-backend-vertical
title: Prototype a bounded scalar CPU backend vertical
status: done
priority: p1
dependencies: []
related: [target-profile-feasibility-model, runtime-execution-contract, reference-evaluator-slice]
scopes: [research/target-profiles, research/artifacts, research/runtime, contracts/artifacts]
shared_scopes: [research/program-planning, contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, cpu, spike]
---
## User-visible outcome

A retained executable spike carries one bounded scalar CPU implementation from a declared CPU target profile through verified physical work, an independently identified executable representation and artifact payload, device/context preflight, execution, and bitwise comparison with `tiler-reference`.

## Why this slice exists

The CPU/SIMD contract is proposed and implementation is not started. A second materially different backend is needed before a generic provider interface can be trusted not to encode Metal's execution hierarchy. The reference evaluator executes on a CPU but is not a physical CPU backend and must not be relabelled as one.

## Implementation keys

- Implement the smallest real scalar program already admitted by the semantic and reference layers; do not scaffold a production `tiler-cpu` crate.
- State a bounded CPU target profile with target triple, ABI/data layout, address width, scalar execution model, and exact operation/dtype support; vector and threading claims remain absent and therefore `Unknown`.
- Consume verified structured KIR or record precisely why its current vocabulary cannot express the CPU realization.
- Define a governed backend key and executable representation distinct from Metal and from the reference evaluator.
- Package and decode a real payload, validate it against a device-free environment, then bind it to a live host execution context before dispatch.
- Compare exact results with `tiler-reference`, while keeping reference identity and backend identity separate.
- Perturb target facts, representation, payload identity, and output behavior and watch the corresponding checks fail.
- Retain a reproducible spike harness and result fixture.
- Do not edit production crates in this spike. File any evidence-backed production blocker as a separate ticket with its own scope and public-boundary review where required.

## Closes when

One scalar CPU payload executes through the recorded vertical or the spike identifies a precise architectural blocker, every unsupported vector/thread/dtype feature rejects explicitly, and no production support claim or permanent crate admission is made.

## Outcome

The vertical **executed**. `spikes/target-profiles/scalar-cpu-vertical` carries one bounded scalar CPU implementation from a declared CPU target profile to a bit-for-bit agreement with `tiler-reference`, against `crates/` unmodified. Run it with `cd spikes/target-profiles/scalar-cpu-vertical && CARGO_TARGET_DIR=./target cargo run`; the README records what the run proves and where its evidence stops, and `results/2026-07-31-macos-arm64.json` retains the identities, byte counts, and exact output bit patterns.

**Measurement**, Apple M-series arm64 macOS on the pinned nightly: twelve `f32` elements agreed exactly, including a preserved negative zero, the least positive and least negative subnormals preserved through a multiply, a non-canonical NaN payload canonicalized to `0x7fc00000`, and both infinities. Profile descriptor 797 bytes, payload 265 bytes, envelope 20,327 bytes, artifact identity 9,464 bytes, reference registry identity 80,104 bytes, **zero** deferred prepared-entry predicates.

`ReferenceEvaluator` is not the backend: it evaluates the semantic program through the capability registry, and the backend decodes a serialized image of the *scheduled kernel body* and executes it with no access to `VerifiedKernel`. The two share no code and their identities are reported separately.

## Measured CPU requirements for `specify-the-consumer-neutral-backend-provider-composition-contract`

Each is exercised by this spike; the README's Findings section carries the evidence for each.

- **No production edit is needed to add a backend.** `TargetProfileBuilder`, `CompileRequest`/`TargetRequest`, `ArtifactProgramBuilder`, and `tiler_runtime::load` are together sufficient. The composition contract can assume this surface rather than propose a replacement for it.
- **The prepared-entry deferred-query stage must stay optional.** A profile declaring its workgroup bound as an available compile-time fact mints zero deferred predicates and routes through `DecodedProgram::preflight`; Metal cannot, because only a built pipeline knows its own `maxTotalThreadsPerThreadgroup`. A contract that made `prepare`/`resolve_target_properties` mandatory would encode Metal's pipeline stage as universal.
- **A backend's second stage is not necessarily device-shaped.** The CPU stage is dominated by the *floating-point environment of the running process*, measured rather than declared, and refusing there is what stops a flushing host from executing a preserving contract. The vocabulary ticket should name this "execution-context facts no artifact can assert", not "device facts".
- **Payload validation is a provider obligation, and it must run before the routing commit.** A payload's `code` bytes are opaque to every check `DecodedProgram` performs, so the six payload-level refusals this spike observes are checks only the backend can make. A provider contract that does not require them lets the first careless backend discover a malformed payload after the commit.
- **The transport mapping must stay carried rather than assumed.** A scalar entry's transports are its ABI slots; Metal's are argument-table indices and are not the identity. Either assumption is wrong for the other backend.
- **Governed key namespaces need an owner.** `BackendKey`, `RepresentationKey`, and `TargetProfileKey` validate length and alphabet only, which is what made `tiler.cpu.scalar` expressible without a registry edit and is also why nothing stops two producers minting the same key for different things.
- **Four vocabularies have no CPU referent and are named as seams rather than blockers.** `AvailabilityPhase` has no host-process phase and borrows `LiveDevicePreflight`; `ArtifactExecutionPolicy` is a two-valued `NativeImage`/`RequiresDeviceTranslation` dichotomy with no way to say "interpreted image" or "dynamically linked object"; `PayloadProvenance` requires Apple-shaped `deployment_major`/`deployment_minor` and an SDK identity; and `CapabilityAxis` has no target-triple, ABI, data-layout, vector-width, mask/tail, or scalable-vector axis, so a CPU profile carries the triple only inside its key string and must answer the GPU-only `WorkgroupThreads` and `LocalMemoryBytes` axes with `1` and `0`.

## Graph maintenance

- Feed the measured CPU requirements into `specify-the-consumer-neutral-backend-provider-composition-contract`. **Done**: the section above is that record, and the spike README's Findings carry the evidence per item.
- Keep Q-PLAN-011 and `docs/backends/cpu.md` explicitly proposed until a later accepted implementation plan. **Held**: `docs/backends/cpu.md` gained the spike as evidence and a traceability paragraph stating that it covers the scalar half alone and that the contract stays proposed; `docs/open-questions.md` is unchanged.
- Do not satisfy this ticket by calling `ReferenceEvaluator` the backend implementation. **Held**, see Outcome.
- No blocker ticket was filed. Nothing in `crates/` refused this vertical or behaved incorrectly; the four vocabulary seams above are inputs to the composition contract rather than defects, and filing a ticket per seam before that contract exists would pre-empt its design.
- Remainders this spike did **not** cover, for whoever widens it: the producer and the consumer are one process, so the artifact-identity check is a tautology; the operation-level translation refusals (packed extraction, barriers, dequantization conversions, `I32Subtract`) are guarded by an exhaustive match and not observed refusing; and the admitted program fuses to one stage, so the multi-entry route, the shared-allocation pairing, and the serial-loop interpreter path are implemented and unexercised.
