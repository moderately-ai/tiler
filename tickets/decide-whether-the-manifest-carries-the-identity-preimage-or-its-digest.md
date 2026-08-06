---
id: decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest
title: Decide whether the manifest carries the identity preimage or its digest
status: review
priority: p2
dependencies: []
related: [attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget, decide-whether-executable-coverage-evidence-folds-as-a-digest]
scopes: [contracts/decisions, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [identity, decision, artifacts, encoding]
claimed_from: todo
assignee: agent-identity-campaign
lease_expires_at: 1786061758
---
## User-visible outcome

The largest single component of every artifact envelope — the canonical-identity preimage the manifest carries at its end, **49.2% of a 114,043-byte fixed-content envelope** — is either kept with a stated reason or replaced by its digest with an accepted identity-domain step behind it. Either way the corpus stops carrying half its wire for a declaration nothing reads.

**This is Tom's decision. It is drafted here rather than taken.**

## Why this exists

**Measurement, 2026-08-06, at `8bd720b8`** ([Where the artifact envelope's fixed content came from](../docs/research/artifacts/manifest-fixed-content-growth.md), reproducible from [`spikes/artifacts/manifest-growth-attribution/`](../spikes/artifacts/manifest-growth-attribution/README.md)). One zero-object envelope of the hot-path fixture is 114,043 bytes: a 69-byte header, an 88,061-byte canonical manifest, a 22,903-byte `KernelProgramSubject` section, a 2,974-byte `BackendPayloadMetadata` section, and 36 bytes of section framing. **56,097 of the manifest's 88,061 bytes are the canonical-identity run it ends with**, and that run grew ×4.2 across the interval the record attributes, against ×3.4 for the rest of the manifest.

**Fact — the run is a complete function of the manifest content above it.** `encode_manifest` (`crates/tiler-artifact/src/program/codec/encode.rs`) ends with `push_slice(&mut bytes, identity.as_bytes())`. `encode_identity` (`crates/tiler-artifact/src/program/model.rs`) reads the *envelope* — schema, routing policy, the three semantic subjects, the interface, providers, payloads, the arena, the variants, the delivered-realization record — and reads nothing from the manifest's own encoding.

**Fact — every decode re-derives it and compares, and no consumer reads the carried copy.** `decode` (`codec/decode.rs`) calls `envelope.canonical_identity()` after `validate` and rejects a disagreement with `ArtifactCodecError::ArtifactIdentityMismatch`. `DecodedArtifact::identity` (`codec/view.rs`) returns the derivation and documents it: "Re-derived, never read from the bytes: `decode_artifact` already proved this equals the identity the encoded manifest carried, so a forged manifest cannot present a chosen identity."

**Fact — what the carried run therefore buys is a declaration check.** It fires when a producer's two derivations of one artifact disagree — the same class of hazard the codec was found carrying for canonical arena order, where "the crate had been carrying two definitions of canonical arena order that only happened to agree" ([the decoder-allocation note](../docs/research/artifacts/decoder-allocation-amplification.md) Section 5). That refusal does not need the preimage: a digest of the derived identity under its own domain refuses the identical set of disagreements in 32 bytes.

## The candidates, with what each enables and prevents

- **Keep the preimage.** The wire stays self-describing: anything holding only the bytes can lift the identity without implementing the derivation. Cost: 56,065 bytes per envelope today, and half of every future manifest addition, for a reader that does not exist in this workspace.
- **Carry the digest instead.** Removes **56,065 of 114,043 bytes — 49.2%** at this fixture, and halves the growth rate, because the doubling the record measures (everything added to the manifest is added again to the identity that folds it) becomes a single copy. Cost: the argument below has to hold, and `MANIFEST_SCHEMA` takes a major step because the trailing run changes width and meaning, so a reader of the earlier schema would frame it wrongly.
- **Carry nothing.** Cheapest bytes and the strongest justification needed: it deletes the producer-disagreement refusal entirely rather than making it cheaper, and that refusal is the reason the run exists.

## The objection the middle option has to answer

[ADR 0074 convention 2](../docs/decisions/0074-use-explicit-public-api-conventions.md) holds that a canonical identity is opaque bytes a receiving crate treats as opaque and never re-derives locally, so a digest standing where canonical bytes stood needs an argument that this site is a fold input rather than an identity a consumer compares. [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md) raises the same objection at the IR layer and defers on it.

**The argument available here, which is not available there.** This run is compared by the crate that is the *authority* for it, against bytes that same crate derives in the same call, and every public reader already reads the derivation. It is a producer's declaration to its own decoder, not an identity crossing a boundary. Whether that distinction is sufficient is exactly what Tom decides.

## What does not move, and it is the part that makes this cheap

**Artifact identity does not move.** `encode_identity` reads the envelope and not the manifest, so `CanonicalArtifactProgramIdentity`, the expansion-cache subject derived from it, and every pinned identity in the workspace are unchanged. Only the envelope's wire bytes move. That is the same shape as the `14.0` arena step, where `MANIFEST_SCHEMA` stepped because the wire was *permitted* to move while artifact identity provably did not — and it is a larger step than that one, because here the wire actually does move.

## Explicit non-goals

Not the IR layer's per-record `SemanticGraphIdentity` — that is [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md)'s, and this ticket does not reopen or duplicate it. Not the `KernelProgramSubject` section, which is a genuine subject rather than a restatement: nothing else in the envelope determines it. Not the manifest body's per-entry stage subjects, which the manifest needs in order to describe its entries at all.

**Not a fix for the growth curve.** The record's Section 5 is explicit: this change moves the point at which the 1 MiB per-invocation embedding ceiling binds from roughly 32 semantic operations to roughly 50, which is still below the ≥ 51-operation decoder-layer program the roadmap contemplates. A 49% constant saving does not buy an order of magnitude against a quadratic. Deciding this does not close that.

## Closes when

Tom has answered, and the answer is an accepted ADR carrying the identity-domain step and its ledger obligations, or a recorded decision to keep the preimage with the reason stated where a later reader of the envelope's size would look.

## Decision — the digest

**Decided by Tom on 2026-08-06 at the live session's decision round (presented by the orchestrator, explain-then-recommend, relay source this ticket): the manifest carries a digest of the derived identity under its own domain, replacing the preimage.** The grounds accepted are the record's: the producer-disagreement refusal survives in 32 bytes, no consumer reads the carried preimage, artifact identity provably does not move, and the self-describing property serves a reader that does not exist — revisitable if one appears. The keep-nothing option was not taken: the refusal is retained.

## Outcome — 2026-08-06, the step is landed and its ADR is drafted for acceptance

**The manifest declares its artifact identity by digest, at manifest schema `15.0`.** `encode_manifest`'s trailing `push_slice(identity.as_bytes())` is now a fixed-width unframed digest under a fourth governed envelope domain, `tiler.artifact-envelope.identity-digest.v1`; `decode` derives the identity, digests it under the same domain, and compares. Unframed rather than framed because that is the codec's existing digest convention — the header's manifest digest and every section descriptor's content digest are already fixed-width and unprefixed — which also saves the eight-byte length prefix the ticket's own estimate kept.

**Measurement, through this ticket's own attribution harness (`spikes/artifacts/manifest-growth-attribution/probe.sh`), at both ends.** Fixed content **114,059 → 57,978 bytes, −56,081, −49.17%**. The manifest itself 88,069 → 31,988. The `KernelProgramSubject` section (22,911), `BackendPayloadMetadata` (2,974), the header (69), and the section framing (36) are all unmoved, so the whole reduction is the identity run plus its prefix less the thirty-two bytes replacing them.

Two figures differ from this ticket's estimate of 56,065 and both are accounted for: the base is `eee734cf` rather than `8bd720b8`, where the staged-realization step had grown the identity run 56,097 → 56,105; and the unframed spelling saves the run's eight-byte length prefix as well.

**Artifact identity did not move, and no pin moved — verified rather than asserted.** `cargo nextest run --workspace` passes at 2,885 tests with no identity constant, golden, or ledger value recomputed, which is exactly the prediction the "What does not move" section above makes. The derived identity is 56,105 bytes on both sides of the change.

**`ArtifactIdentityMismatch` survives intact and gains no sibling.** The refused set is identical: this check has always compared two things the decoder holds rather than the wire against the world, so a "digest disagreed" case and an "identity disagreed" case are one case. The variant's doc now says so.

**Watched failing, three ways.** Deleting the decoder's comparison makes both `a_forged_identity_is_rejected` and `strict_affine_component_order_is_part_of_artifact_identity` fall through to `NonCanonicalManifest`; restoring the preimage beside the digest makes the new `the_manifest_declares_its_identity_by_digest_and_carries_no_preimage` fail on the preimage's presence. A stale prose measurement in `docs/artifact-abi.md` — the carrier-only fixture pair's "forty byte positions" — was silently falsified by the step and is now **68**, pinned as `DIFFERING_CARRIER_POSITIONS` and shown failing at a deliberately wrong value before being set.

**Carried by [ADR 0103](../docs/decisions/0103-declare-the-manifests-artifact-identity-by-digest.md), `decision_status: proposed`.** Tom accepts the record; the substance he decided on 2026-08-06 is stated in it with its relay provenance. The ledger moved with it: `docs/artifact-abi.md`'s identity ledger reads manifest schema 15.0, the governed-digest section carries four domains and the union obligation is now over eight, and the no-layered-digests block records why an identity *declaration* by digest is envelope framing rather than a second identity authority.

**What executes it, and why it waited.** The ADR carrying the identity-domain step (a `MANIFEST_SCHEMA` major step; the ADR 0074 convention-2 argument recorded from the record's own derivation) and the codec implementation land together under one implementation ticket. **Held: [`account-for-a-staged-realization-stage-in-the-kernel-program`](account-for-a-staged-realization-stage-in-the-kernel-program.md) is in flight holding `implementation/artifact` and stepping `MANIFEST_SCHEMA` itself for the staged-realization declaration — two uncoordinated steps of one schema cannot co-run. Release trigger: that ticket integrates into main; the digest step then lands against the post-declaration encoding.** This ticket stays open until the ADR is accepted and the step lands whole.
