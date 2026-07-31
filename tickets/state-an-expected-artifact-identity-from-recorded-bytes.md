---
id: state-an-expected-artifact-identity-from-recorded-bytes
title: State an expected artifact identity from recorded bytes
status: todo
priority: p2
dependencies: []
related: [route-the-runtime-loader-through-the-dispatch-record, reconcile-the-empty-domain-proof-member-between-the-two-serial-sum-prototypes]
scopes: [implementation/artifact, implementation/runtime, contracts/artifacts, research/target-profiles, project/tickets]
shared_scopes: []
paths: []
tags: [implementation, artifact, public-boundary]
---
`CanonicalArtifactProgramIdentity` can be read and cannot be stated. Only code that *built* an artifact can hold one, so the cold-consumer half of `DecodedProgram::preflight`'s own documented contract was unrepresentable in its own signature.

## Derived design

Use a distinct immutable, bounded, domain-checked `RecordedArtifactProgramIdentity`. It states producer intent without claiming that the bytes were derived from validated artifact content.

| Option | Enables | Prevents |
| --- | --- | --- |
| **1. Keep `expected: &[u8]`.** The status quo. | Costs nothing, adds no public type, and weakens no comparison — byte equality of canonical identity is exactly what a typed comparison would do. The evidence asymmetry stays visible in `LoadRejection::ProgramMismatch`, where the two sides are already different types on purpose. | Names nothing at the call site. A caller can pass any slice — a hash of the wrong thing, an empty vector, an unrelated key — and the compiler will not object. The second half of `preflight`'s documented contract stays unexpressible in its own signature. |
| **2. A distinct `RecordedArtifactProgramIdentity`.** | Names the concept at the call site while keeping the two evidence classes apart: a derived identity came from validated content, a recorded one is a byte string somebody wrote down. It is the only option under which `preflight` can express both of its documented sources in its signature. | Adds a public type to `tiler_artifact::program`, and the artifact and runtime APIs must adopt the same distinction together — one decision spanning two public boundaries. |
| **3. Broaden `CanonicalArtifactProgramIdentity` with a checked byte constructor.** | — | Eliminated, on the evidence rather than on taste. A byte constructor cannot prove derivation, so the type would stop meaning "encoder-derived" while still being spelled that way, and every existing reader of one would silently widen what it accepts. That is the second-authority shape ADR 0082 names, and it is exactly why `keys.rs:17-19` states the absence of a constructor as a *decision*: the identity "is derived only by this crate's encoder and has no public constructor." |

**Outcome: option 2.** It is the only candidate that lets `DecodedProgram::preflight` and `DecodedProgram::prepare` express the second half of their documented contract—the identity recorded when the producer cached these bytes—and the recording case is a real consumer shape, not a hypothetical: `route-the-runtime-proof-through-the-artifact-envelope` has a producer write the identity to a sidecar and a separate consumer process read it back.

**Ratified by Tom on 2026-07-30.** Implement the distinct `RecordedArtifactProgramIdentity` boundary and do not reopen raw slices or a public constructor on `CanonicalArtifactProgramIdentity`. The exact constructor/error/runtime call-site diff remains acceptance evidence.

**The counterpoint, which the parked version omitted and which is the strongest argument against the recommendation.** `RecordedArtifactProgramIdentity` would be a newtype over `Vec<u8>` whose only possible validation is a length or bound check. It cannot verify that the bytes are a canonical identity of anything, because nothing but the encoder can produce one. So it **names the concept without proving anything the byte slice did not** — the same bytes, the same comparison, the same failure on a wrong input, wrapped in a type that a reader may reasonably mistake for evidence. That risk is not hypothetical; it is the precise reason option 3 is eliminated, arriving one step down. What it buys is call-site intent: a signature that says which of two things it wants, and a wrong argument that fails to compile rather than failing at load.

**Why it still wins.** The comparison is byte equality either way, so nothing about *correctness* changes between options 1 and 2 — this is a decision about whether the API states its own contract. Option 1 leaves a documented contract half-unrepresentable and lets any slice through; option 2 makes the intended argument the easy one to pass. The mistakable-for-evidence risk is real and is answered by naming it in the type's own documentation, in the same terms `LoadRejection::ProgramMismatch` already uses. If Tom judges that documentation insufficient protection against the misreading, option 1 is coherent and nothing is blocked by choosing it.

**Public review boundary.** Implement and test the distinct assertion type as a concrete draft, then present the exact artifact constructor/error, both runtime `preflight` and `prepare` signatures, rejection type, and producer/sidecar/consumer call sites to Tom before acceptance. Do not add a constructor or conversion to `CanonicalArtifactProgramIdentity`.

**Nothing is blocked in the meantime:** `preflight` works today via `expected: &[u8]`, no comparison is weakened, and the asymmetry in `LoadRejection::ProgramMismatch` is already documented as deliberate.

## Fact

`grep -n "impl CanonicalArtifactProgramIdentity" -A 8 crates/tiler-artifact/src/program/model.rs` shows one method, `as_bytes` (the type is declared at `model.rs:692`, the impl block at `:694-700`). The only construction in the workspace is inside the derivation, at `crates/tiler-artifact/src/program/model.rs:1631` — find it with `grep -n "Ok(CanonicalArtifactProgramIdentity(" crates/tiler-artifact/src/program/model.rs`, since this line number has already drifted once. Its sibling opaque identities all have a validating `from_bytes`; find that with `grep -n "fn from_bytes" crates/tiler-artifact/src/program/keys.rs` rather than by line number. This one does not, and `keys.rs:17-19` says why in terms: it "is derived only by this crate's encoder and has no public constructor."

That rule is right for what it was protecting — nobody should mint an artifact identity. It also blocks a consumer from *recording* one, which is a different act.

## Why it matters

`DecodedProgram::preflight` documents two sources for the identity a caller binds against: "the one it obtained by building this artifact, **or recorded when it cached these bytes**." Only the first was expressible. `route-the-runtime-proof-through-the-artifact-envelope` needed the second — a producer writes the identity to a sidecar, a separate consumer process reads it back — and there was no way to turn those bytes into the type.

## What was done instead, and what it costs

**Superseded by this ticket's implementation.** The two paragraphs below record the interim state the ticket existed to replace. `preflight` and `prepare` now take `&RecordedArtifactProgramIdentity` and `ProgramMismatch` carries one.

`preflight` takes `expected: &[u8]` and compares against `identity().as_bytes()`. `LoadRejection::ProgramMismatch` (`crates/tiler-runtime/src/load.rs:820-825`) carries `expected: Vec<u8>` beside `loaded: CanonicalArtifactProgramIdentity`, and the asymmetry is documented as deliberate at `:816-819`: the loaded side was derived from validated content, the expected side is a byte string somebody recorded, and spelling both as one type would suggest they carry equal evidence.

No comparison is weakened — byte equality of canonical identity is exactly what the typed comparison did. What is lost is that the call site no longer names the concept, so a caller can pass any slice and the compiler will not object.

## User-visible outcome

A cold consumer can state the expected identity it recorded without presenting
that assertion as encoder-derived evidence, and runtime preflight rejects a
different loaded artifact with both concepts named clearly.

## Implementation keys

Introduce the distinct assertion type with immutable shared byte storage so repeated attempts and mismatch errors do not copy an identity whose governed bound is large. Its constructor accepts recorded bytes and rejects empty input, input above `MAX_ARTIFACT_IDENTITY_BYTES`, and an identity whose canonical leading frame is not the current `tiler.artifact-program.v11` domain. Domain recognition is syntax/type separation, not proof that the remainder is canonical or corresponds to an artifact.

Adopt it consistently in artifact and runtime APIs, document that it is producer assertion rather than independently derived evidence, and use a dedicated assertion-validation error rather than `ArtifactBuildError`. `ProgramMismatch` carries the recorded assertion beside the encoder-derived loaded identity. Perturb empty, over-bound, wrong-domain, and mismatched-content checks once each before restoration; “wrong length” is not a valid generic case because canonical artifact identities are variable-length.

## What landed

`RecordedArtifactProgramIdentity` (`crates/tiler-artifact/src/program/model.rs`, beside `CanonicalArtifactProgramIdentity`) stores `Arc<[u8]>` and has one constructor, `from_bytes`, checking empty → over-bound → foreign-domain in that order so each refusal is independently reachable and distinguishable. Rejections are a dedicated `#[non_exhaustive] RecordedArtifactIdentityError` in `program/error.rs`, whose `ForeignDomain` message renders the current domain from a new `pub(super) ARTIFACT_DOMAIN_LABEL` — derived from `ARTIFACT_DOMAIN` in a const block rather than written a second time, so a domain bump cannot leave the error naming the previous version. `MAX_ARTIFACT_IDENTITY_BYTES` was already public and unchanged; no encoding, domain, or schema version moved, per the rule below.

`DecodedProgram::preflight`, `DecodedProgram::prepare`, the private `select_route`, and `LoadRejection::ProgramMismatch` all take or carry the new type. The `compile_fail,E0499` doc-test at `crates/tiler-runtime/src/load/route.rs` spells the signature and was updated with it — left stale it would have stopped compiling for the wrong reason, which is the failure mode of compile-fail evidence.

**Every check was perturbed once and observed failing**, then restored: disabling the empty check made `an_empty_recording_is_refused` report `ForeignDomain` instead of `Empty` (proving the two are distinguishable, not just both refusals); disabling the bound check made the over-bound case report `ForeignDomain`; disabling the domain check made both domain tests accept; and disabling the runtime's `identity != expected` comparison made the prototype's `a_foreign_expected_identity_is_a_program_mismatch` route past the mismatch into a deferred-predicate refusal. Mismatched content is covered by that existing probe rather than by a new test — it flips the *trailing* identity byte, which stays valid under a leading-frame domain check, and both the prototype and the spike now say so in a comment so a future edit does not move the perturbation to a leading byte and silently convert a loader probe into an assertion-boundary probe.

A fifth perturbation checks the *evidence* rather than the code: ending the borrow in the `compile_fail,E0499` doc-test (`drop(route)` before the second `preflight`) made it compile, and `cargo test -p tiler-runtime --doc` reported "Test compiled successfully, but it's marked `compile_fail`". So the retyped signature did not quietly turn a borrow-checker proof into a type error that would have passed for the wrong reason.

## The by-hand evidence, and what it turned up

`make full` is green, and it does not reach either prototype binary or the spike. Both were run by hand:

- `spikes/target-profiles/scalar-cpu-vertical`, per its README (`CARGO_TARGET_DIR=./target cargo run`): completes, and its foreign-identity probe still reports `runtime.program-mismatch` through the recorded-assertion path.
- The two-process serial-sum proof (`cargo run -p tiler-prototype-compile -- --out …` then `cargo run -p tiler-prototype-run -- --artifact …`, macOS, Apple M4 Max): the single-member hardware path — the actual cold-consumer case this ticket exists for, a producer writing the identity to a sidecar and a separate process reading it back through `RecordedArtifactProgramIdentity::from_bytes` — routes, commits, and reports bit-for-bit agreement. The proof matrix that follows then fails on its first member with `ForeignProgram` on the `empty-domain` class. **Reproduced unchanged at this branch's base `1e062d9` in a detached worktree**, so it is pre-existing; filed as `reconcile-the-empty-domain-proof-member-between-the-two-serial-sum-prototypes` rather than absorbed.

## Eliminations, recorded so review reads the reasoning

**`From<&CanonicalArtifactProgramIdentity> for RecordedArtifactProgramIdentity` — rejected.** It would have removed an `expect` at the six sites that hold a derived identity and want to state it (the prototype's fixture helper and the spike's `run`). But those six are exactly the tautological case `preflight`'s own documentation warns about — restating an identity read from the artifact about to be loaded checks nothing — and a blanket conversion makes that the *frictionless* path while the honest cold-consumer path keeps its `from_bytes`. The ratified surface stays one constructor; the six sites carry an `expect` whose message says what is being assumed.

**`DecodedProofSidecar::artifact_identity_bytes()` left returning `&[u8]` — deliberate.** The recorded assertion is constructed at the runtime call site instead. Changing the accessor's return type would pull the proof codec's own internal comparisons (`crates/tiler-artifact/src/proof/codec.rs`, the two `artifact.…identity().as_bytes() != self.artifact_identity_bytes()` checks) into scope, and those compare against a *derived* identity — they are integrity checks inside one container, not host assertions, and typing them as assertions would blur precisely the distinction this ticket exists to draw. The producer side needed no change at all: `proof/builder.rs` already derives the sidecar's identity from a `VerifiedArtifactProgram`.

## Graph maintenance

- Keep encoder-derived and recorded assertion identities as distinct evidence classes in documentation, errors, and call sites.
- Advance artifact identity or schema versions only if the encoded artifact changes; a host-side assertion wrapper alone does not justify a version bump. **Nothing was bumped:** the change is entirely host-side, and `docs/artifact-abi.md` records the distinction under "Identity is derived from the canonical envelope" without touching the encoding sections.
- **`research/target-profiles` was added to this ticket's scopes during implementation.** `spikes/target-profiles/scalar-cpu-vertical` is a separate workspace that path-depends on `tiler-runtime` and calls `preflight` at seven sites, so the signature change breaks it — and no `make` target reaches `spikes/`, so the gate would have stayed green over a spike that no longer compiles. Recorded as a dispatch gap acknowledged rather than absorbed: the scope was added explicitly and the spike's call sites, `ProbeSubject`, and `VerticalError` were updated in the same change, verified with `cargo check --all-targets` run from the spike's own directory.
- Preserve the exact public diff for Tom's acceptance review. The consequential surface is: `RecordedArtifactProgramIdentity` with `from_bytes`/`as_bytes`, `RecordedArtifactIdentityError` with its three variants, the two `DecodedProgram` signatures, and `LoadRejection::ProgramMismatch`'s changed field type. **Not self-accepted.**
