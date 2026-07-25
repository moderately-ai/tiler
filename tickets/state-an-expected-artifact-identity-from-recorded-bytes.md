---
id: state-an-expected-artifact-identity-from-recorded-bytes
title: State an expected artifact identity from recorded bytes
status: todo
priority: p2
dependencies: []
related: [route-the-runtime-loader-through-the-dispatch-record]
scopes: [implementation/artifact]
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

## What closes this

A decision on whether `CanonicalArtifactProgramIdentity` gains a checked `from_bytes` — making "an identity someone recorded" a first-class value and letting `preflight` be typed again — or whether the byte-slice signature is the honest one because recorded bytes genuinely are not a derived identity and should not wear its type. The second is a defensible answer and would close this ticket by accepting the current state rather than changing it.

`needs-tom`: it is a public constructor on a type whose absence of one is a stated decision.
