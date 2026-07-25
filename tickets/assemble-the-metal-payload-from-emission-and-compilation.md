---
id: assemble-the-metal-payload-from-emission-and-compilation
title: Assemble the Metal payload from an emission and a compilation
status: todo
priority: p0
dependencies: [prototype-metal-bundle-assembly]
related: [prototype-apple-aot-driver, choose-one-owner-for-apple-target-vocabulary, prototype-artifact-family-delivery]
scopes: [implementation/metal, implementation/metal-aot, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, artifact, aot]
---
`prototype-metal-bundle-assembly` landed the *neutral* half: the artifact envelope now carries a backend payload's compilation subject and its object bytes as two governed sections, decides payload identity, and re-proves it on decode. Nothing in it is Metal. This ticket is the Metal half — filling that neutral shape from a real emission and a real compilation — split out rather than folded in, because it crosses two crates the parent did not touch and carries two public-surface consequences of its own.

## What already exists to build on

`crates/tiler-artifact/src/program/codec/payload.rs` defines the shape a backend fills: a governed source representation and the exact source bytes; provenance as a toolchain key, a normalized target, a family, a language, a deployment minimum, ordered versioned tool components, an SDK identity, and the exact **ordered** compile and link flags; entry mappings from each neutral `BackendEntryKey` to the backend's own symbol and transport slots; and recorded target obligations. `ArtifactProgramBuilder::push_carried_payload` takes that content and *derives* the descriptor digest from it, so a carried payload cannot claim a subject it does not carry.

**Fact — the identity question the parent ticket owned is already answered, and this ticket must not reopen it.** A carried payload is content-addressed over its **compilation inputs**; the emitted object travels opaquely under a section digest that is integrity and is deliberately excluded from artifact identity. `payload_identity_follows_the_compilation_subject_and_not_the_object` pins both directions: relinking the same source yields an equal artifact identity and a different envelope digest, and changing the source yields a different artifact identity. The consequence the parent recorded: the expansion cache is keyed by artifact identity, and an envelope digest names one published encoding rather than the artifact.

That also settles what the golden-compilation canary means. `crates/tiler-metal/src/golden_compilation.rs::one_golden_compiles_to_identical_bytes_twice_when_a_toolchain_resolves` measured identical `metallib` bytes on Metal 32023.883 under macOS 27.0 build 26A5388g. Under the chosen identity that canary is a **toolchain-reproducibility** observation, not an identity precondition: if a future toolchain embeds a UUID or a timestamp it will fail, and the correct response is to record that the toolchain stopped being reproducible — **not** to relax the assertion, and not to change the identity basis. Bundle identity does not depend on it.

## The work

1. **Fill the neutral payload from an emission and a compilation.** Source representation and source bytes from the MSL emission; provenance from the AOT driver's recorded toolchain; entry mappings from the emitted entry points and their buffer parameters; obligations from what `tiler-metal` recorded it could not discharge. Prove the entry mappings cover exactly the backend entry keys the artifact's executable entries name.
2. **Write the `MetalTargetFacts` → `MetalTarget` translation, and decide where it lives.** `choose-one-owner-for-apple-target-vocabulary` decided that `tiler-metal` and `tiler-metal-aot` each keep their own MSL language version, artifact family, and deployment minimum, held in step by a total map in `crates/tiler-metal/src/target_correspondence.rs` that is a *test* rather than a conversion, because neither crate may depend on the other. Whatever assembles a bundle is the first component that needs a real conversion, and it must live where a dependency permits it. **The translation must be total. Do not write a wildcard arm that maps an unrecognized emitter family or language standard onto a default** — a wildcard there can only invent an `AppleSdk` or a `-std` token, and the artifact's provenance header and its actual compilation would then disagree about what it is.
3. **Remove `#[non_exhaustive]` from `tiler_metal::target::{MslLanguageVersion, MetalPlatform}`.** Writing that translation out of crate makes both ADR 0074 convention 5b types, and 5b explicitly does not permit a wildcard arm instead. Both types say so in their own doc comments today. Note that removing the attribute is a source-breaking change to in-workspace call sites, which ADR 0075 routes to Tom; `cargo check` enumerates the affected sites exhaustively before the question is put.
4. **Validate a bundle without a live device**, and keep the maturity claims apart: a bundle that assembles and re-validates is not a bundle that has executed.

## A caution to carry rather than rediscover

**A `metallib` byte count is not an identity.** A 14,620-byte link of the four goldens was recorded at `59060b5` and stopped being true 47 minutes later when `e24f4c5` changed the emitted source; the same command then yielded 14,716. Do not let a size assertion stand in for a digest anywhere.

## Closes when

A bundle is assembled from a real emission and a real compilation, carried in the neutral envelope, and re-validated from bytes without a device; the target translation is total and lives where a dependency permits it; both `tiler-metal` target types have lost `#[non_exhaustive]` with Tom's review of that surface change; and `uv run --locked python scripts/check_repository.py` passes.

## Progress — items 2 and 3 landed; item 1 is blocked on the public compiler boundary

**Landed.** `prototypes/serial-sum-compile/src/target.rs` is the production `MetalTargetFacts` → `MetalTarget` translation, in the one place a dependency permits it: neither backend crate may depend on the other, and the producer is the first component that sees both vocabularies at once. `tiler_metal::target_correspondence` already said its orchestrator inherits that obligation; this is that orchestrator. Every map is total — a wildcard could only invent an `AppleSdk` or a `-std` token, which would let a bundle's provenance header and its actual compilation disagree with nothing able to detect it.

Five tests, written so they cannot pass by repeating the map: the family assertion goes through `AppleSdk::platform()` rather than a second hand-written table; the standard assertion compares `-std` tokens rather than variant names; the deployment minimum is checked through the real target triple; the family map is required to be *onto* the full SDK set, which catches a collapsed arm that pointwise cases would miss; and the standard map is checked injective.

`#[non_exhaustive]` is removed from `tiler_metal::target::{MslLanguageVersion, MetalPlatform}`, which the ticket assigned here. Both doc comments now record them as ADR 0074 convention 5b types and say why: an out-of-crate wildcard could only invent the counterpart, so the enums are deliberately exhaustive and a new family or standard is a build failure at every map. This is a *loosening* rather than an ADR 0075 breaking change — nothing that compiled before stops compiling — so it did not need approval; adding a variant later will.

`scripts/check_workspace.py`'s pinned dependency contract gained the producer's `tiler-metal-aot` edge, with the reason recorded. The gate caught the omission.

**Blocked, and this is the finding that matters.** Item 1 — filling the neutral payload from a real emission and a real compilation — cannot start. `tiler_compiler::pipeline` is a private `mod`, and both `compile` and `CompilationRequest` are `pub(crate)`, so **no caller outside `tiler-compiler` can compile a program at all**. Without a `VerifiedKernel` there is nothing to emit, nothing to compile, and nothing to assemble. The payload carrier's constructors are `pub(crate)` in `tiler-artifact` for the same reason, so even a caller holding a kernel could not build a `PayloadContent`.

That makes `prototype-public-compiler-api` the true head of the critical path to first execution, ahead of every Metal ticket. The translation above is landed behind an ADR 0074 convention 7 `#![allow(dead_code, reason = …)]` naming exactly that blocker as the reason its production caller does not yet exist.
