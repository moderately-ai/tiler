---
id: correct-the-artifact-identity-post-compilation-claim
title: Correct the claim that artifact identity needs compiled bytes
status: todo
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
