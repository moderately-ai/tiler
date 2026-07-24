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

## Progress — the neutral carrier landed; the Metal half did not

**Status: not done.** The envelope can now *carry* a backend payload and decides its identity; nothing yet fills that shape from a real MSL emission and a real `metallib`. The remaining half is `assemble-the-metal-payload-from-emission-and-compilation`, split out rather than hidden because it crosses `implementation/metal` and `implementation/metal-aot`, which this ticket does not hold, and carries a public-surface change ADR 0075 routes to Tom.

### The identity decision this ticket owned is made, and implemented

**A carried payload is content-addressed over its compilation inputs. The emitted object is opaque, under a section digest that is integrity and is deliberately excluded from artifact identity.** The descriptor's digest is required to equal the identity of the exact canonical payload-metadata bytes — source, target, flags, toolchain provenance, entry mappings, obligations, and *no object byte at all* — and `push_carried_payload` derives it rather than accepting it, so a payload cannot claim a subject it does not carry.

The alternative, content-addressing over emitted bytes, was rejected because it makes artifact identity a function of compiler-output reproducibility, which `docs/artifact-abi.md` already refuses to promise: "Tiler promises deterministic source, manifest, and identity construction; it does not promise byte-identical Apple output across machines or toolchain builds."

**What that costs is recorded rather than hidden.** The codec's *equal identity implies equal bytes* property now holds for the identity-bearing part of an envelope and not for the object sections: two bundles built from one compilation subject by a non-reproducible linker have equal artifact identity and different envelope digests. The expansion cache is therefore keyed by artifact identity, which the contract already states, and an envelope digest names one published encoding rather than the artifact.

**And it settles what the golden-compilation canary means.** Under this identity, `one_golden_compiles_to_identical_bytes_twice_when_a_toolchain_resolves` is a toolchain-reproducibility observation, not an identity precondition. If a future toolchain embeds a UUID or a timestamp, the correct response is to record that the toolchain stopped being reproducible — not to relax the assertion, and not to change the identity basis. Bundle identity does not depend on it.

### What landed

- `codec::payload` — a **neutral** compilation subject: governed source representation and exact source bytes, provenance (toolchain key, normalized target, family, language, deployment minimum, ordered versioned tool components, SDK identity, exact ordered compile and link flags), entry mappings from each neutral `BackendEntryKey` to a backend symbol and its transport slots, and recorded target obligations. Nothing here is Metal; a CUDA payload fills the same shape with `nvcc`, `ptxas`, and `sm_90`.
- Two governed section purposes, `BackendPayloadMetadata` and `BackendPayloadCode`, carried in the existing content-addressed table. Two payloads carrying the same object share one section, as a stated property of the purpose.
- The governed feature `tiler.artifact.feature.embedded-payload-code`, derived from content. A reader that predates carried payloads sees the descriptors and none of the code and has no way to notice, because the manifest it understands is complete on its own; requiring the feature makes it refuse instead.
- Decode, with the closure checks that make the carrier safe rather than merely present.

### Three defects found and fixed while completing the interrupted work

1. **A section reference that resolves is not a section reference that is right.** Resolving an index alone would let a forged manifest point a payload's *code* reference at its own *metadata* section: both sections are well formed, both digests verify, the encoder restamps the manifest digest, and the artifact would load with its object bytes silently replaced. `SectionPurposeMismatch` decides it, and the same check now covers a variant's program-section reference. `a_payload_section_reference_of_the_wrong_purpose_is_rejected` pins it.
2. **The section canonical key had to become `(purpose, content)`, not content.** With one purpose the two were the same. With three, ordering by bytes alone would report a legitimate metadata/code pair with equal bytes as a duplicate, and would call a genuine duplicate ordered.
3. **The encoder zipped the descriptor and content tables after declaring a count from the first.** Two tables that disagreed in length would have produced a manifest whose declared payload count outran its rows — a framing desync that pre-empts the `UnusedPayload` and `UnreferencedSection` obligations that should decide such an envelope. Now indexed, so the row count and the declared count are the same number by construction. A pre-existing forgery test caught this.

### Verification

`cargo nextest run --workspace --no-fail-fast` — 607 passed, 0 skipped; `cargo clippy --workspace --all-targets` clean, with the two new lint findings fixed structurally (a named `ProjectedTables` record and an extracted `project_entries`/`decode_provenance`) rather than allowed; `cargo fmt --all --check` clean; `cargo test --doc --workspace` passes.

Nine cases cover the carrier, including the identity direction test above, the wrong-purpose forgery, a descriptor claiming a subject it does not carry, a carried subject that is not payload metadata, a non-canonical collection inside a subject, and the deliberate exception that compiler flag order is meaning and survives unsorted.

### Surface note

`push_carried_payload` and the payload vocabulary are `pub(crate)` behind the codec's private module, under ADR 0074 convention 7. Both carry an `#[allow(dead_code, reason = …)]` naming what they reserve and that their first non-test consumer is the backend assembler that does not exist yet. No public surface changed.
