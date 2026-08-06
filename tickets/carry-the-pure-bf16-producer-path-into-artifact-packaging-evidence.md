---
id: carry-the-pure-bf16-producer-path-into-artifact-packaging-evidence
title: Carry the pure-BF16 producer path into artifact packaging evidence
status: in-progress
priority: p1
dependencies: [admit-a-bf16-index-realization-law-and-refinement-contract]
related: [carry-bf16-through-the-artifact-encoding-and-identity, conform-the-bf16-vertical-end-to-end]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, bf16, identity]
claimed_from: todo
assignee: agent-bf16-packaging
lease_expires_at: 1785988695
---
## User-visible outcome

A pure-BF16 semantic program reaches a `VerifiedArtifactProgram` through the ordinary producer path — built, encoded, decoded, and its identity re-derived — rather than only a hand-assembled envelope at the artifact layer.

## Why this is not the producing ticket's own evidence

**Fact.** `admit-a-bf16-index-realization-law-and-refinement-contract` made the composition reachable and proved it as far as its crate can reach: a pure-BF16 constant/multiply/add program obtains verified coverage for all four occurrences and builds a `VerifiedKernelProgram` over a `PointwiseBf16` scheduled region (`crates/tiler-ir/src/program/tests.rs`, `a_pure_bf16_program_covers_every_occurrence_and_builds_a_verified_kernel_program`).

**Fact.** `VerifiedArtifactProgram` lives in `crates/tiler-artifact/src/program/model.rs`, and `crates/tiler-ir/Cargo.toml` declares no workspace crate dependencies — the direction is `tiler-artifact → tiler-ir`. No `tiler-ir` test can reach the artifact layer, so the packaging half of the evidence is `implementation/artifact` work and was recorded as a boundary rather than absorbed.

## What to do

Add the BF16 analogue of the existing `f32` artifact fixture (`crates/tiler-artifact/src/program/tests.rs`, `build_artifact`/`default_artifact`, and the strict-affine variant at `strict_affine_u4_dequantize_artifact`): a pure-BF16 program packaged into a `VerifiedArtifactProgram` that encodes, decodes, and re-derives its identity.

Note that the candidate index region must be hand-built with `IndexRegionBuilder`, as the other artifact fixtures do: `IndexRealizationLaw::realize` and `FrozenSemanticRegistry::index_realization_law` are `pub(crate)` to `tiler-ir`.

## Stale prose to correct in the same change

**Fact.** The doc comment on `bf16_input_envelope` (`crates/tiler-artifact/src/program/codec/tests.rs:2465-2477`) states that "`NumericalContractIdentity` wraps `F32NumericalContractKey` alone, and the standard semantic provider registers index-realization laws for nine `f32` and quantization operations and none for the registered `bf16` family". Both clauses are now false: the identity admits a `bf16` key and twelve laws are registered. The test it justifies (`a_bf16_artifact_round_trips_and_its_carrier_enters_identity`) still passes; only its stated reason for hand-assembling the envelope is stale, and a hand-assembled fixture whose justification has expired is exactly the comment that misleads the next reader.

## Closes when

A pure-BF16 program reaches a `VerifiedArtifactProgram` through the builder, its round trip and identity re-derivation are asserted, and the stale justification is corrected to describe what the code now does.
