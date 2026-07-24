---
id: prototype-metal-bundle-assembly
title: Assemble the Metal artifact bundle
status: todo
priority: p0
dependencies: [prototype-neutral-artifact-codec, prototype-metal-numerical-realization, prototype-apple-aot-driver]
related: []
scopes: [implementation/artifact, implementation/metal, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, artifact, aot]
---
Assemble deterministic MSL, metallib sections, entry mappings, neutral program metadata, target requirements, provenance and section digests into one bounded self-validating bundle. Validate it without a live device; treat metallib reproducibility as measured evidence, not an assumed guarantee.

## Evidence now available for the reproducibility question

`compile-golden-msl-through-the-aot-driver-in-the-gate` turned that reproducibility from an assumption into an enforced, host-qualified measurement. `crates/tiler-metal/src/golden_compilation.rs::one_golden_compiles_to_identical_bytes_twice_when_a_toolchain_resolves` compiles one golden twice through `Toolchain::compile` and asserts the linked `metallib` bytes and the provenance fingerprint are identical. The driver uses a differently named scratch directory on every call, so a pass shows that per-run host state does not leak into the artifact. **Measurement**, on Metal 32023.883 under macOS 27.0 build 26A5388g: identical. It is one toolchain build on one host, not a portable guarantee.

**The decision this ticket must make explicitly.** That test is a deliberate canary. If a future toolchain embeds a UUID, a timestamp, or a path into the library, it will fail — and the correct response is *not* to relax the assertion. It is to answer the question this ticket owns: does bundle identity depend on `metallib` bytes being reproducible, or is it content-addressed over the inputs (exact source, exact toolchain provenance, exact flags) with the library carried as an opaque payload? The second is robust to a non-reproducible linker and the first is not. Decide it here, state which, and record what the canary failing would then mean.

A related caution from the same work: **a `metallib` byte count is not an identity.** A 14,620-byte link of the four goldens was recorded at commit `59060b5` and stopped being true 47 minutes later when `e24f4c5` changed the emitted source; the same command now yields 14,716. Do not let a size assertion stand in for a digest anywhere in the bundle.
