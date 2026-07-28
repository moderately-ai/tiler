---
id: assemble-prepared-metal-artifacts-in-tiler-build
title: Assemble prepared Metal artifacts in tiler-build
status: todo
priority: p2
dependencies: [bind-the-cache-subject-to-the-carried-payload-provenance]
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/artifact, implementation/metal-aot, implementation/workspace, implementation/build]
shared_scopes: []
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

## Graph maintenance

When artifact assembly is executable and its mismatch tests prove the failure path, close this ticket and release `accept-and-publish-validated-artifacts-through-the-expansion-cache`. If the artifact builder lacks a required typed backend fact, file that authority gap as a dependency rather than spelling the fact locally.
