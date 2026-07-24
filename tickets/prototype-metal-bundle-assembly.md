---
id: prototype-metal-bundle-assembly
title: Assemble the Metal artifact bundle
status: in-progress
priority: p0
dependencies: [prototype-neutral-artifact-codec, prototype-metal-numerical-realization, prototype-apple-aot-driver]
related: []
scopes: [implementation/artifact, implementation/metal, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, artifact, aot]
claimed_from: todo
assignee: agent-prototype-metal-bundle-assembly
lease_expires_at: 1784932574
---
Assemble deterministic MSL, metallib sections, entry mappings, neutral program metadata, target requirements, provenance and section digests into one bounded self-validating bundle. Validate it without a live device; treat metallib reproducibility as measured evidence, not an assumed guarantee.

## Evidence now available for the reproducibility question

`compile-golden-msl-through-the-aot-driver-in-the-gate` turned that reproducibility from an assumption into an enforced, host-qualified measurement. `crates/tiler-metal/src/golden_compilation.rs::one_golden_compiles_to_identical_bytes_twice_when_a_toolchain_resolves` compiles one golden twice through `Toolchain::compile` and asserts the linked `metallib` bytes and the provenance fingerprint are identical. The driver uses a differently named scratch directory on every call, so a pass shows that per-run host state does not leak into the artifact. **Measurement**, on Metal 32023.883 under macOS 27.0 build 26A5388g: identical. It is one toolchain build on one host, not a portable guarantee.

**The decision this ticket must make explicitly.** That test is a deliberate canary. If a future toolchain embeds a UUID, a timestamp, or a path into the library, it will fail — and the correct response is *not* to relax the assertion. It is to answer the question this ticket owns: does bundle identity depend on `metallib` bytes being reproducible, or is it content-addressed over the inputs (exact source, exact toolchain provenance, exact flags) with the library carried as an opaque payload? The second is robust to a non-reproducible linker and the first is not. Decide it here, state which, and record what the canary failing would then mean.

A related caution from the same work: **a `metallib` byte count is not an identity.** A 14,620-byte link of the four goldens was recorded at commit `59060b5` and stopped being true 47 minutes later when `e24f4c5` changed the emitted source; the same command now yields 14,716. Do not let a size assertion stand in for a digest anywhere in the bundle.

## This ticket inherits the Apple target vocabulary correspondence

`choose-one-owner-for-apple-target-vocabulary` decided that `tiler-metal` and `tiler-metal-aot` each keep their own MSL language version, artifact family, and deployment minimum, and that the two vocabularies are held in step by a total map in `crates/tiler-metal/src/target_correspondence.rs`. That map is a test rather than a conversion function for a structural reason: neither crate may take a normal dependency on the other, so no production `MetalTargetFacts` → `MetalTarget` translation can exist inside either. Whatever assembles a bundle from an emission and a compilation is the first component that needs one, and this ticket is the current candidate.

Two consequences to carry, not rediscover:

- **The translation must be total, and it must live where a dependency permits it.** Do not write a `match` with a wildcard arm that maps an unrecognized emitter family or language standard onto a default; a wildcard there can only invent an `AppleSdk` or a `-std` token, and the resulting artifact's provenance header and its actual compilation would disagree about what it is.
- **Writing that translation makes `tiler_metal::target::{MslLanguageVersion, MetalPlatform}` ADR 0074 convention 5b types**, because the map is then out of crate. Both currently carry `#[non_exhaustive]`, which is correct only while the sole total map is inside `tiler-metal`. This ticket owns removing the attribute from both; per ADR 0074 it is explicitly *not* free to add a wildcard arm instead. The types say so in their own doc comments.
