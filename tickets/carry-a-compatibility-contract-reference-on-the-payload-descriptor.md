---
id: carry-a-compatibility-contract-reference-on-the-payload-descriptor
title: Carry a compatibility-contract reference on the backend payload descriptor
status: done
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec, prototype-metal-bundle-assembly, record-the-implemented-artifact-envelope-in-the-contract]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization]
---
**Fact — the contract lists five things that name a backend payload and the model carries four of them.** `docs/artifact-abi.md`: "The neutral layer references a backend payload through a governed backend key, representation key, payload digest, compatibility-contract reference, and an opaque backend entry key." `BackendPayloadDescriptor` in `crates/tiler-artifact/src/program/model.rs` carries the backend key, the representation key, its own payload schema version, the content digest, and an execution policy. There is no compatibility-contract reference.

**Fact — the declared target contract is carried one level away, on the plan variant.** `VariantRow` carries `TargetProfileRef` (key plus descriptor digest) and `FeasibilityRuleSetRef` (key plus revision). Those are the *plan's* declared target requirements, not the *payload's* compatibility contract.

**Inference — the two coincide only while a payload is not shared.** `MAX_ARTIFACT_PAYLOADS` is 16 and entries cross-reference payloads by index, so nothing in the model stops two variants that declare different target profiles from realizing their entries through one payload. When they do, no field in the envelope states which compatibility contract the payload's own bytes were built against, and the envelope research's rule — "a backend payload is compatible only when the installed backend provider supports its backend key, payload schema, representation, compatibility contract, and execution/translation policy" — has nothing to evaluate. A loader would have to infer the payload's contract from whichever variant it happened to route to, which is the kind of inference this layer exists to forbid.

**What closes this.** Either add the compatibility-contract reference to `BackendPayloadDescriptor` and fold it into the payload canonical key and artifact identity, or decide that a payload is per-variant by construction and state that invariant with the check that enforces it. The first is the contract as written; the second is a narrower model that must be argued rather than assumed. Whichever is chosen, record it in `docs/artifact-abi.md` beside the sentence above.

**Scope note.** Deliberately left unscoped: the fix touches `implementation/artifact` and `contracts/artifacts`. It is adjacent to `prototype-metal-bundle-assembly` but distinct from it — this is the neutral descriptor's field set, not the Metal payload's schema or its identity basis, and it must not pre-empt that ticket's decision on either.

## Outcome

**Tom approved the public-surface change on 2026-07-24 and the contract-as-written option was taken.** `BackendPayloadDescriptor` gains `compatibility: TargetProfileRef`, folded into `canonical_key` and therefore into artifact identity, and carried on the wire between the content digest and the execution policy. The descriptor now carries all five things the contract says name a backend payload.

**Why this rather than the narrower model.** The ticket offered declaring a payload per-variant by construction as the alternative, to be argued rather than assumed. Arguing it defeated it: the rule would make a legitimate program *inexpressible*. Two variants that compile to the same library could not share the payload once sharing is forbidden, and could not declare a second identical descriptor either, because `ArtifactProgramBuilder::push_payload` already refuses that as `ArtifactBuildError::DuplicatePayload`. So the narrower model does not merely lose expressiveness at the margin — it leaves a real program with no encoding at all. That cost is recorded in `docs/artifact-abi.md` beside the decision, as the ticket required.

The field is the *payload's* contract and not the plan's. A variant's `TargetProfileRef` and `FeasibilityRuleSetRef` are the plan's declared requirements; they coincide with the payload's only while a payload is realized by one variant, which nothing in the model requires since entries cross-reference payloads by index. Carrying it per payload is what lets a shared object state what it was built for instead of leaving a loader to infer it from whichever variant it routed to.

**Surface impact, as ADR 0075 frames it.** `cargo check` enumerated the in-workspace construction sites exhaustively before the question was put: exactly two, one in `builder.rs` and one test fixture. `push_carried_payload` takes the reference as a parameter rather than deriving it — a carried payload's provenance names a backend-specific target string, while `TargetProfileRef` is the neutral governed profile, and only an assembler knows which profile it compiled against.

**Two tests.** `the_payload_compatibility_contract_participates_in_its_canonical_key` pins that both halves of the reference — the profile key and the descriptor digest — separate two otherwise-identical payloads, so the digest is part of the contract rather than decoration. `a_changed_payload_compatibility_contract_changes_the_artifact` pins that the reference reaches artifact identity and the envelope bytes, and that the changed artifact still round-trips.

**Contract refresh done in the same pass.** Several statements in `docs/artifact-abi.md` had gone stale against this session's work and were corrected rather than left: the lede's "backend payload bytes still have no section vocabulary"; the "no backend payload bytes in the envelope at all" consumer note; the section vocabulary's "exactly one governed purpose", now three; the descriptor-narrowness note ahead of the narrowing list; the maturity paragraph, which called the payload descriptor a type-system reservation when it is implemented and tested against synthetic content and merely unfilled by a real backend; and the section's anchor commit.

`cargo nextest run --workspace --no-fail-fast` — 614 passed, 0 skipped; clippy, fmt, and `scripts/docs.py render` clean.
