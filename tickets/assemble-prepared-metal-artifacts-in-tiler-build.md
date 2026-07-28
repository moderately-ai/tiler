---
id: assemble-prepared-metal-artifacts-in-tiler-build
title: Assemble prepared Metal artifacts in tiler-build
status: done
priority: p2
dependencies: [bind-the-cache-subject-to-the-carried-payload-provenance]
related: [implement-the-expansion-cache-protocol, restore-replayable-apple-compatibility-evidence]
scopes: [implementation/artifact, implementation/metal-aot, implementation/workspace, implementation/build, research/apple-targets, implementation/metal, contracts/artifacts, contracts/decisions, contracts/navigation, implementation/cargo-lock, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [build, artifact, correctness]
---
## User-visible outcome

One prepared Metal compilation produces a target-neutral artifact whose payload metadata is derived from the exact source and prepared provenance that determine the compiled object. The build path validates that correspondence before the artifact becomes publishable, and a producer mismatch fails with the typed defect introduced by its dependency rather than emitting bytes.

## Implementation keys

The first missing call site is immediately after `Toolchain::prepare`: `tiler-build` can validate an already-constructed `PayloadMetadata`, but it does not yet own construction of the payload descriptor, compilation of the prepared token, or assembly of the resulting object into `ArtifactProgramBuilder`.

Construct compilation metadata from typed `tiler-metal` and `tiler-metal-aot` facts rather than string copies at the caller. Preserve the decided division: entry mappings and target obligations come from backend emission, while source, toolchain, target, family, language, deployment minimum, components, SDK, and ordered flags come from the prepared compilation. Run correspondence validation on the constructed metadata before consuming the prepared token, and prove the final `CompiledArtifact` carries the same provenance record.

Do not introduce caller-declared ABI facts into `BindingSpec`; the bound program remains the ABI authority. Keep artifact identity at domain v7 and manifest schema 5.0 unless a real encoded-subject change requires the normal versioning procedure.

Every negative assembly check must be fault-injected once. Target `cargo nextest run -p tiler-build` and per-package Clippy while iterating.

## Delivered evidence

`tiler-build` now derives the compiler request from one emitted `MetalTranslationUnit`, rejects an unrealizable unit or typed numerical selection both during request derivation and again at the public prepared-token seam, binds the emitted entry mappings and obligations to the exact prepared source and provenance, validates every carried compilation fact before compiler work, and exposes typestate wrappers for pending and compiled payload publication. The compiled wrapper exposes read-only inspection and checked insertion without releasing mutable owned metadata. The serial-sum producer consumes this path and no longer carries its own target conversion, payload metadata implementation, or backend/representation/schema insertion constants.

The Apple target vocabulary is complete for the pinned toolchain: ten artifact families derive nine SDK selectors and target-triple spellings, twelve semantic MSL revisions derive their platform-specific compiler token, and construction rejects unavailable pairs or deployment minima below their governed floor. The specification supplies the macOS, iOS, tvOS, and visionOS floors; bounded Metal 32023.883 compile-and-link measurements supply the Catalyst and watchOS MSL 4.0 rows without claiming runtime qualification. MSL 4.1 remains vendored future evidence rather than a token accepted by Metal 32023.883.

The source mismatch, request and prepared-token numerical refusals, platform conversion, language conversion, canonical enum inventories, and specification token/floor checks were each perturbed and observed failing before their positive run. The targeted package suite passes 156 tests and production Metal/build Clippy passes with warnings denied.

The primary Apple MSL 4.0 and 4.1 specifications and Metal feature-set chart are vendored under `docs/research/apple-targets/sources/`. The compatibility audit also found that the retained historical compatibility record cannot replay against its named producer digests; `restore-replayable-apple-compatibility-evidence` owns that evidence repair without weakening this ticket's typed target contract.

## Graph maintenance

When artifact assembly is executable and its mismatch tests prove the failure path, close this ticket and release `accept-and-publish-validated-artifacts-through-the-expansion-cache`. If the artifact builder lacks a required typed backend fact, file that authority gap as a dependency rather than spelling the fact locally.
