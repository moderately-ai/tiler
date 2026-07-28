---
id: state-an-expected-artifact-identity-from-recorded-bytes
title: State an expected artifact identity from recorded bytes
status: awaiting-decision
priority: p2
dependencies: []
related: [route-the-runtime-loader-through-the-dispatch-record]
scopes: [implementation/artifact, implementation/runtime]
shared_scopes: []
paths: []
tags: [implementation, artifact, needs-tom]
---
`CanonicalArtifactProgramIdentity` can be read and cannot be stated. Only code that *built* an artifact can hold one, so the cold-consumer half of `DecodedProgram::preflight`'s own documented contract was unrepresentable in its own signature.

## Fact

`grep -n "impl CanonicalArtifactProgramIdentity" -A 8 crates/tiler-artifact/src/program/model.rs` shows one method, `as_bytes`. The only construction in the workspace is `crates/tiler-artifact/src/program/model.rs:1393`, inside the derivation. Its sibling opaque identities all have a validating `from_bytes` (`crates/tiler-artifact/src/program/keys.rs:121`); this one does not, and `keys.rs:17-19` says why in terms: it "is derived only by this crate's encoder and has no public constructor."

That rule is right for what it was protecting — nobody should mint an artifact identity. It also blocks a consumer from *recording* one, which is a different act.

## Why it matters

`DecodedProgram::preflight` documents two sources for the identity a caller binds against: "the one it obtained by building this artifact, **or recorded when it cached these bytes**." Only the first was expressible. `route-the-runtime-proof-through-the-artifact-envelope` needed the second — a producer writes the identity to a sidecar, a separate consumer process reads it back — and there was no way to turn those bytes into the type.

## What was done instead, and what it costs

`preflight` takes `expected: &[u8]` and compares against `identity().as_bytes()`. `LoadRejection::ProgramMismatch` carries `expected: Vec<u8>` beside `loaded: CanonicalArtifactProgramIdentity`, and the asymmetry is documented as deliberate: the loaded side was derived from validated content, the expected side is a byte string somebody recorded, and spelling both as one type would suggest they carry equal evidence.

No comparison is weakened — byte equality of canonical identity is exactly what the typed comparison did. What is lost is that the call site no longer names the concept, so a caller can pass any slice and the compiler will not object.

## User-visible outcome

A cold consumer can state the expected identity it recorded without presenting
that assertion as encoder-derived evidence, and runtime preflight rejects a
different loaded artifact with both concepts named clearly.

## What closes this

Decide whether to keep `expected: &[u8]`, introduce a distinct bounded
`RecordedArtifactProgramIdentity` (or `ExpectedArtifactProgramIdentity`), or
explicitly broaden `CanonicalArtifactProgramIdentity` to represent both
derived and recorded claims. A checked byte constructor cannot prove
derivation, so prefer a distinct type if typed call-site intent is wanted. The
artifact and runtime APIs must agree on the selected evidence distinction.

`needs-tom`: it is a public constructor on a type whose absence of one is a stated decision.

## Parked 2026-07-27 — awaiting Tom

**The question, atomic:** should a consumer be able to *state* an expected artifact identity in the type system, and if so as what?

Three options, with the elimination already run:

1. **Keep `expected: &[u8]`.** Costs nothing and names nothing; a caller can pass any slice and the compiler will not object. This is the status quo.
2. **A distinct `RecordedArtifactProgramIdentity`.** Names the concept at the call site while keeping the evidence classes apart — a derived identity came from validated content, a recorded one is a byte string somebody wrote down.
3. **Broaden `CanonicalArtifactProgramIdentity` with a checked byte constructor.** Rejected on the evidence: a byte constructor cannot prove derivation, so the type would stop meaning "encoder-derived" while still being spelled that way. That is the second-authority shape ADR 0082 names, and it is why `keys.rs` states the absence of a constructor as a decision rather than an omission.

So the live choice is between 1 and 2, and option 2 is the recommendation — it is the only one that lets `DecodedProgram::preflight` express the second half of its own documented contract ("or recorded when it cached these bytes").

**Why it cannot proceed without the decision.** Adding a public constructor to a type whose *absence* of one is a stated decision is ADR 0075's always-ask category twice over. The artifact and runtime APIs must also agree on the same evidence distinction, so this is one decision spanning two public boundaries rather than two independent edits.

Nothing is blocked in the meantime: `preflight` works today via `expected: &[u8]`, no comparison is weakened, and the asymmetry in `LoadRejection::ProgramMismatch` is already documented as deliberate.
