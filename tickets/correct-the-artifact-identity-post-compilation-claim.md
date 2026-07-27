---
id: correct-the-artifact-identity-post-compilation-claim
title: Correct the claim that artifact identity needs compiled bytes
status: done
priority: p3
dependencies: []
related: [derive-the-pre-compilation-artifact-program-subject, accept-the-tiler-cache-public-boundary]
scopes: [contracts/decisions, implementation/cache]
shared_scopes: []
paths: []
tags: [contract, cache, identity, correction]
---
Three checked-in statements assert something the code disproves, and they were the reason a whole ticket was written to solve a problem that did not exist.

**Fact — the false claim.** `CanonicalArtifactProgramIdentity` is said to fold the compiled payload's bytes, and therefore to exist only after backend compilation.

**Fact — what the code does.** `push_carried_payload` derives `BackendPayloadDescriptor::digest` as `payload_identity(encode_metadata(&metadata))`. `crates/tiler-artifact/src/program/codec/payload.rs:17-19` states those bytes "contain the source, the target, the flags, and the toolchain provenance and **no object byte at all**". `check_payload_identity` (`crates/tiler-artifact/src/program/codec/validate.rs:246`) re-proves it on every decode, and `payload_identity_follows_the_compilation_subject_and_not_the_object` asserts that relinking the same source yields equal artifact identity. The digest is of the compilation *subject*.

**Fact — the three sites, each reproducible in one line.** ADR 0050's traceability section; `crates/tiler-cache/src/expansion/subject.rs:56-64`; `crates/tiler-cache/src/expansion/key.rs:45-49`. Verify each before editing — `derive-the-pre-compilation-artifact-program-subject` landed in between and may have moved line numbers, and `AGENTS.md` requires the construction site to be read rather than the declaration trusted.

**Why this is not merely editorial.** A doc comment is a claim and it is load-bearing. This one made a solved problem look open, and it is cited by ADR 0050 — an accepted decision — so a reader has every reason to believe it. It also understates what the artifact layer already guarantees: content addressing that survives relinking is a stronger property than the documents currently admit to having.

**Do not overcorrect.** The reason the claim was plausible is that a `BackendPayloadDescriptor` *does* accompany compiled content in the carried case, and the digest *is* named a digest. State what the digest is of, rather than only that it is not what was claimed, so the next reader does not re-derive the same wrong conclusion from the same name. Whether `digest` is a misleading field name is a separate question — raise it, do not rename it here.

**Preserve the ADR's status distinctions.** ADR 0050 is accepted; correcting a factual sentence inside it is not a superseding decision and must not be written as one. If the correction changes what the ADR *decided* rather than what it *described*, stop and say so — that would need an explicit superseding record.

## Closes when

All three sites state what the payload digest is derived from, with the check that proves it; no site claims artifact identity requires an object; the ADR's decision status and rationale are untouched; any catalog block quoting the record is updated by hand in the same change; and `make full` passes.

## Outcome — corrected at four sites, not three (2026-07-27)

**Every fact was re-verified against the current source before editing, as the ticket asked.** `crates/tiler-artifact/src/program/codec/payload.rs` states in its own module documentation that the payload-metadata bytes "contain the source, the target, the flags, and the toolchain provenance and **no object byte at all**"; `check_payload_identity` is at `codec/validate.rs:246`; `payload_identity_follows_the_compilation_subject_and_not_the_object` is at `codec/tests.rs:1831`. `PayloadMetadata::identity` (`codec/payload.rs:236`) and `ArtifactProgramBuilder::push_pending_payload` (`program/builder.rs:335`) both exist and are `pub`.

**A fourth site carried the same claim and is corrected too.** The ticket named ADR 0050, `expansion/subject.rs`, and `expansion/key.rs`. `docs/decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md` line 55 repeated it nearly verbatim — "derived from a verified artifact, which needs the payload digest and therefore the compiled bytes". Found by searching for the claim's wording rather than by trusting the ticket's site list.

**The claim was stale in a second way the ticket did not anticipate.** `subject.rs` said no producer existed for `SubjectFacet::ArtifactProgram`. That is now false on both halves: the reason was wrong, *and* `derive-the-pre-compilation-artifact-program-subject` has since landed the producer. The facet that actually blocks an end-to-end key is `BackendCompilations` — `tiler-metal-aot` declares `mod identity;` privately and `CompilationIdentity::as_bytes` is `pub(crate)` (verified at `crates/tiler-metal-aot/src/identity.rs:245`), so no other crate can obtain those bytes. Both corrected sites now name that blocker and `promote-the-metal-aot-compilation-identity`, so the "composable, not yet usable" state stays accurate rather than becoming a second stale claim.

**Each site states what the digest is *of*, per the "do not overcorrect" instruction** — the compilation subject, with the check that re-proves it — rather than only what it is not. The `digest` field is not renamed; whether that name misleads is left as a separate question.

**Both ADRs keep their decision status and rationale.** The corrections are marked in place as corrections to what each record *described*, with the old reasoning quoted and refuted so a reader can see why the sentence changed and cannot re-derive it. Neither ADR's decision changed, so neither is superseded, and `docs/decisions/README.md`'s two catalog entries for 0050 carry only title and status — both unchanged — so no catalog block needed editing.
