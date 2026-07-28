---
schema: "tiler-doc/v1"
id: "ADR-0085"
kind: "decision"
title: "Admit tiler-build as the build-time orchestrator"
topics: ["rust", "workspace", "dependencies", "build", "artifacts", "cache", "toolchains"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture", "tiler.contract.artifact-abi", "tiler.contract.metal-backend"]
evidence: ["tiler.research.cache.crash-race-protocol", "tiler.research.artifacts.target-neutral-artifact-envelope", "tiler.research.workspace.prototype-crate-layout-and-msrv"]
depends_on: ["ADR-0050", "ADR-0077", "ADR-0082"]
ticket: "bind-the-cache-subject-to-the-carried-payload-provenance"
---

# 0085: Admit tiler-build as the build-time orchestrator

**Status:** accepted. Tom decided this on 2026-07-28 on the derivation below. It admits a ninth reusable library, `tiler-build`, and assigns it the complete build-time publication and acceptance path rather than only one correspondence check.

## Context

**Fact — the cache cannot validate whether a producer's subject describes the artifact stored beside it.** ADR 0082 deliberately limits `tiler-cache` to composing and validating the outer subject frame. Interpreting `tiler-metal-aot`'s private subject encoding would make the cache a second authority over producer identity, while accepting any well-framed subject would permit a writer to pair one compilation subject with an artifact produced from another compilation.

**Fact — the compiler is upstream of publication and acceptance.** The compiler produces checked physical plans. It does not own backend emission, external-tool execution, artifact assembly, cache publication, or cache-hit acceptance, and making it depend on the cache would invert that data flow.

**Fact — the complete check has two supplying authorities.** `tiler-metal-aot` owns the exact request, resolved toolchain observation, derived compilation identity, and compilation provenance. `tiler-artifact::program::PayloadMetadata` owns the carried description of source representation, source bytes, toolchain family, target, artifact family, language, deployment minimum, ordered tool components, SDK identity, and ordered compiler and linker flags.

**Inference — correspondence is an orchestration invariant.** Only the component sequencing producer preparation, artifact assembly, cache lookup, publication, and hit acceptance legitimately sees both authorities. The check must run before publication and before accepting a hit, and a mismatch is a typed producer/protocol defect rather than a cache miss because rebuilding the same mismatched inputs would repeat the defect.

## Decision

`tiler-build` owns the complete build-time flow from a checked compiler plan through backend emission, prepared AOT compilation, artifact assembly, composed cache subject construction, provenance correspondence validation, cache publication, and cache-hit acceptance.

1. The crate is downstream of the authorities it sequences. It may depend on compiler, backend, artifact, AOT-driver, and cache crates; none of those crates depends on it.

2. The crate compares facts through public typed records supplied by their owners. It does not parse or duplicate a private canonical subject encoding, invent a second digest, re-resolve an external toolchain, or re-derive a compiler or artifact identity.

3. A prepared AOT operation binds the request, the one resolved toolchain observation, its compilation identity, and the provenance the compiled artifact will carry. Hit validation borrows those facts without allocation; miss execution consumes the same prepared operation and moves the same provenance record into its result.

4. The Metal correspondence check covers every compilation fact represented in `PayloadMetadata`: source representation, exact source, toolchain family, target, artifact family, language, deployment minimum, ordered tool components, SDK identity, ordered compiler flags, and ordered linker flags. Entry mappings and target obligations remain backend-emission facts and are not attributed to the AOT compiler.

5. The initial implementation may land this correspondence slice before the whole orchestrator is executable, but the crate boundary is not justified as a one-function utility. Later slices complete the already-assigned path rather than creating another integration owner.

## Consequences

- The workspace carries nine reusable libraries. The implemented checked-plan path has the downstream closure `tiler-build -> [tiler-artifact, tiler-cache, tiler-compiler, tiler-ir, tiler-metal, tiler-metal-aot]`: the compiler and shared-IR edges carry the owner-linked plan and semantic graph, while the backend, AOT, artifact, and cache edges carry the authorities the orchestrator sequences.
- `tiler-metal-aot` retains its empty dependency closure. The downstream orchestrator consumes its public facts, so the driver does not acquire artifact, cache, backend, or compiler knowledge.
- `tiler-cache` remains a storage and validation protocol rather than an interpreter for foreign subject encodings.
- A cache hit cannot be accepted solely because its outer bundle, subject digest, and artifact envelope are internally valid. The orchestrator also proves the carried payload agrees with the prepared producer facts.
- Tom separately accepted the initial exact Rust facade on 2026-07-28: borrowed `PreparedCompilation::{request, provenance}` accessors, `validate_prepared_metal_payload`, the exhaustive `MetalPayloadFact` classification, and opaque `MetalPayloadMismatch`. At that point, later orchestration facades still required their own exact-boundary review under ADR 0075.
- Tom accepted the checked-plan facade on 2026-07-28: `PlanAlternative` retains its owning `Compilation`; `Compilation::offered_providers` exposes the complete compiler-minted environment; and `accept_or_publish_metal_plan` returns an opaque `AcceptedMetalPlanArtifact` only after the producer-side verified artifact and accepted decoded envelope agree.

## Alternatives considered

**Put the check in `tiler-cache`.** Eliminated because correctness would require interpreting a producer encoding the cache does not own. Duplicating that vocabulary would permit the producer and cache to drift while each remained internally consistent.

**Put the check in `tiler-compiler`.** Eliminated because publication and hit acceptance are downstream consumers of compiler output. A compiler-to-cache dependency would invert the architecture and give the compiler responsibility for packaging and storage.

**Create a crate for this check alone.** Eliminated on long-term maintainability grounds. The check has no independent lifecycle: its only valid call sites are inside the build-time sequence that prepares compilation, assembles the corresponding artifact, and accepts or publishes a cache result. A one-function crate would leave that sequence ownerless and require another public boundary later.

## Traceability

Implements the cross-authority obligation left explicit by ADR 0082 and preserves ADR 0077's dependency-free AOT driver. The [system architecture](../architecture.md) owns the resulting component and dependency boundary.
