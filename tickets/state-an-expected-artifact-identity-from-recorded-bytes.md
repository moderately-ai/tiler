---
id: state-an-expected-artifact-identity-from-recorded-bytes
title: State an expected artifact identity from recorded bytes
status: todo
priority: p2
dependencies: []
related: [route-the-runtime-loader-through-the-dispatch-record]
scopes: [implementation/artifact, implementation/runtime, contracts/artifacts]
shared_scopes: []
paths: []
tags: [implementation, artifact, public-boundary]
---
`CanonicalArtifactProgramIdentity` can be read and cannot be stated. Only code that *built* an artifact can hold one, so the cold-consumer half of `DecodedProgram::preflight`'s own documented contract was unrepresentable in its own signature.

## Derived design

Use a distinct bounded, domain-checked `RecordedArtifactProgramIdentity` or `ExpectedArtifactProgramIdentity`. It states producer intent without claiming that the bytes were derived from validated artifact content.

| Option | Enables | Prevents |
| --- | --- | --- |
| **1. Keep `expected: &[u8]`.** The status quo. | Costs nothing, adds no public type, and weakens no comparison — byte equality of canonical identity is exactly what a typed comparison would do. The evidence asymmetry stays visible in `LoadRejection::ProgramMismatch`, where the two sides are already different types on purpose. | Names nothing at the call site. A caller can pass any slice — a hash of the wrong thing, an empty vector, an unrelated key — and the compiler will not object. The second half of `preflight`'s documented contract stays unexpressible in its own signature. |
| **2. A distinct `RecordedArtifactProgramIdentity`.** | Names the concept at the call site while keeping the two evidence classes apart: a derived identity came from validated content, a recorded one is a byte string somebody wrote down. It is the only option under which `preflight` can express both of its documented sources in its signature. | Adds a public type to `tiler_artifact::program`, and the artifact and runtime APIs must adopt the same distinction together — one decision spanning two public boundaries. |
| **3. Broaden `CanonicalArtifactProgramIdentity` with a checked byte constructor.** | — | Eliminated, on the evidence rather than on taste. A byte constructor cannot prove derivation, so the type would stop meaning "encoder-derived" while still being spelled that way, and every existing reader of one would silently widen what it accepts. That is the second-authority shape ADR 0082 names, and it is exactly why `keys.rs:17-19` states the absence of a constructor as a *decision*: the identity "is derived only by this crate's encoder and has no public constructor." |

**Outcome: option 2.** It is the only candidate that lets `DecodedProgram::preflight` express the second half of its own documented contract — "the one it obtained by building this artifact, **or recorded when it cached these bytes**" — and the recording case is a real consumer shape, not a hypothetical: `route-the-runtime-proof-through-the-artifact-envelope` has a producer write the identity to a sidecar and a separate consumer process read it back.

**The counterpoint, which the parked version omitted and which is the strongest argument against the recommendation.** `RecordedArtifactProgramIdentity` would be a newtype over `Vec<u8>` whose only possible validation is a length or bound check. It cannot verify that the bytes are a canonical identity of anything, because nothing but the encoder can produce one. So it **names the concept without proving anything the byte slice did not** — the same bytes, the same comparison, the same failure on a wrong input, wrapped in a type that a reader may reasonably mistake for evidence. That risk is not hypothetical; it is the precise reason option 3 is eliminated, arriving one step down. What it buys is call-site intent: a signature that says which of two things it wants, and a wrong argument that fails to compile rather than failing at load.

**Why it still wins.** The comparison is byte equality either way, so nothing about *correctness* changes between options 1 and 2 — this is a decision about whether the API states its own contract. Option 1 leaves a documented contract half-unrepresentable and lets any slice through; option 2 makes the intended argument the easy one to pass. The mistakable-for-evidence risk is real and is answered by naming it in the type's own documentation, in the same terms `LoadRejection::ProgramMismatch` already uses. If Tom judges that documentation insufficient protection against the misreading, option 1 is coherent and nothing is blocked by choosing it.

**Public review boundary.** Implement and test the distinct assertion type as a concrete draft, then present the exact artifact constructor, runtime preflight signature, rejection type, and producer/consumer call sites to Tom before acceptance. Do not add a constructor to `CanonicalArtifactProgramIdentity`.

**Nothing is blocked in the meantime:** `preflight` works today via `expected: &[u8]`, no comparison is weakened, and the asymmetry in `LoadRejection::ProgramMismatch` is already documented as deliberate.

## Fact

`grep -n "impl CanonicalArtifactProgramIdentity" -A 8 crates/tiler-artifact/src/program/model.rs` shows one method, `as_bytes` (the type is declared at `model.rs:692`, the impl block at `:694-700`). The only construction in the workspace is inside the derivation, at `crates/tiler-artifact/src/program/model.rs:1631` — find it with `grep -n "Ok(CanonicalArtifactProgramIdentity(" crates/tiler-artifact/src/program/model.rs`, since this line number has already drifted once. Its sibling opaque identities all have a validating `from_bytes`; find that with `grep -n "fn from_bytes" crates/tiler-artifact/src/program/keys.rs` rather than by line number. This one does not, and `keys.rs:17-19` says why in terms: it "is derived only by this crate's encoder and has no public constructor."

That rule is right for what it was protecting — nobody should mint an artifact identity. It also blocks a consumer from *recording* one, which is a different act.

## Why it matters

`DecodedProgram::preflight` documents two sources for the identity a caller binds against: "the one it obtained by building this artifact, **or recorded when it cached these bytes**." Only the first was expressible. `route-the-runtime-proof-through-the-artifact-envelope` needed the second — a producer writes the identity to a sidecar, a separate consumer process reads it back — and there was no way to turn those bytes into the type.

## What was done instead, and what it costs

`preflight` takes `expected: &[u8]` and compares against `identity().as_bytes()`. `LoadRejection::ProgramMismatch` (`crates/tiler-runtime/src/load.rs:820-825`) carries `expected: Vec<u8>` beside `loaded: CanonicalArtifactProgramIdentity`, and the asymmetry is documented as deliberate at `:816-819`: the loaded side was derived from validated content, the expected side is a byte string somebody recorded, and spelling both as one type would suggest they carry equal evidence.

No comparison is weakened — byte equality of canonical identity is exactly what the typed comparison did. What is lost is that the call site no longer names the concept, so a caller can pass any slice and the compiler will not object.

## User-visible outcome

A cold consumer can state the expected identity it recorded without presenting
that assertion as encoder-derived evidence, and runtime preflight rejects a
different loaded artifact with both concepts named clearly.

## What closes this

Introduce the distinct bounded/domain-checked assertion type, adopt it consistently in artifact and runtime APIs, document that it is producer assertion rather than independently derived evidence, and perturb wrong-length, wrong-domain, and mismatched-content checks once each before restoration.

## Graph maintenance

- Keep encoder-derived and recorded assertion identities as distinct evidence classes in documentation, errors, and call sites.
- Advance artifact identity or schema versions only if the encoded artifact changes; a host-side assertion wrapper alone does not justify a version bump.
- Preserve the exact public diff for Tom's acceptance review.
