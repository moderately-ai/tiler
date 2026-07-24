---
id: prototype-neutral-artifact-codec
title: Implement the neutral artifact codec
status: in-progress
priority: p0
dependencies: [prototype-artifact-program-model]
related: [prototype-artifact-slice, carry-reconstructable-kernel-programs-in-the-neutral-envelope, own-the-numerical-realization-profile-key, select-the-governed-artifact-digest-implementation, record-the-implemented-artifact-envelope-in-the-contract]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, serialization]
claimed_from: todo
assignee: agent-prototype-neutral-artifact-codec
lease_expires_at: 1784923183
---
Implement a bounded canonical lockstep envelope/program codec independent of compiler internals. Validate schema/version, canonical encoding/order, limits, references, duplicates, section digests and identity, truncation, trailing bytes, corruption, and unsupported features with typed diagnostics.

## Outcome

`crates/tiler-artifact/src/program/codec/` holds a bounded canonical lockstep codec for the target-neutral artifact envelope: framing, canonical manifest encoding, section digests, governed schema and required-feature compatibility, and a decoder that re-proves every artifact-model obligation decidable from the manifest alone. It landed as a **crate-private draft authority** under ADR 0074 convention 7 — a private `mod codec` whose items are `pub(crate)`, with the module-level `#![allow(dead_code, reason = "…")]` naming what it reserves and which slice consumes it. **One decision is Tom's and is recorded below rather than taken.**

### The identity encoder was made a function of the envelope

This is the load-bearing change and it is not a refactor for tidiness. `model::encode_identity` previously read `ArtifactProgramData` — the builder's draft storage — and reached into each variant's `VerifiedKernelProgram` for stage keys, resource requirements, and the numerical realization. A codec cannot reach those (see below), so an identity re-derived at decode time would have had to come from a second encoder that agreed with the first only by inspection. That is exactly the duplicated-authority hazard `AGENTS.md` singles out for artifact identity.

`encode_identity` now takes `&ArtifactEnvelope`, and `ArtifactProgramBuilder::build` reaches it by projecting first. There is **one** identity encoder and **one** subject: the artifact identity a producer stamps and the identity a decoder re-derives are the same bytes from the same code path, structurally rather than coincidentally. The identity byte layout is unchanged; every pre-existing identity test still passes.

The projection also canonicalizes the shared ABI expression arena. Identity was already order-independent because it cross-references by content key; the arena's *positions* were not, so two structurally equal artifacts assembled in different orders would have produced different envelope bytes for one identity. Projection reorders the arena into the unique topological order that always emits the smallest available node by content key, and remaps every reference.

### What the encoding commits to

A fixed 69-byte framing header — magic `TILERART`, envelope format `{1,0}`, canonical encoding `{1,0}`, the governed digest algorithm tag, total byte length, manifest byte length, section count, and the digest of the exact manifest bytes — then one canonical manifest, then length-delimited sections. Total length, manifest length, and section count are all bounded before anything is allocated. The digest over the complete envelope is derived externally and never stored inside the bytes it covers.

The manifest carries: the four governed component schema versions; the routing policy; the derived required-feature set; the three reached semantic subjects; the ordered named interface with each entry's shape and storage element type; the selected capability providers; the backend payload descriptors; the shared ABI expression arena; each plan variant with its program section, guard, declared target profile and feasibility rule set, deferred predicates, and executable entries — each entry carrying its stage subject, proven resource requirements, declared numerical realization, ABI bindings, launch contract, and backend entry; the section descriptors; and the artifact's canonical identity exactly once.

Every variable-length run carries a fixed-width big-endian length before its content. Every encoded enumeration goes through the one governed tag table its vocabulary owns, never a Rust discriminant, and each table is written as an adjacent forward/inverse pair pinned by an exhaustive round-trip test.

**Order is meaning where the model says it is and canonical everywhere else.** Variant order (routing priority), interface order, and ABI binding order are retained. Provider, payload, deferred-predicate, launch-precondition, entry, expression, and section order are replaced by the canonical content order identity already uses. Two artifacts with equal identity therefore encode to equal bytes — which is what makes an envelope digest usable as a cache key — and `payload_and_provider_declaration_order_do_not_change_the_bytes` and `expression_assembly_order_does_not_change_the_bytes` prove it.

### What it deliberately excludes

- **The frozen registry snapshot.** ADR 0072 keeps an unreached provider's provenance out of packaged identity. Carrying it here would put it back into the envelope's bytes and therefore its digest, so an unused provider could invalidate a cache entry. `an_unused_environment_provider_does_not_change_the_bytes` asserts equal bytes *and* equal envelope digests across two genuinely different compilation environments.
- **Backend payload bytes.** A payload is named by governed keys, its own schema version, its opaque content digest, and its execution policy — exactly as the artifact model names it. Whether a bundle's identity is content-addressed over compilation inputs or over emitted payload bytes is `prototype-metal-bundle-assembly`'s decision and this codec does not pre-empt it. The section machinery that ticket needs exists and is exercised; its governed section purposes are its own versioned extension. No size or count stands in for a digest anywhere.
- **A reconstructable kernel program.** A section carries one packaged variant's *canonical kernel-program identity*. `KernelProgramBuilder::new` takes a `&SemanticProgram`, which requires a frozen registry holding `Arc<dyn OperationInferencer>` values; neither is representable as bytes, so no codec can rebuild a `VerifiedKernelProgram` from an envelope alone. Owned by `carry-reconstructable-kernel-programs-in-the-neutral-envelope`.
- **Two builder obligations that tie the ABI to the program**, not to the manifest: a binding's accessible byte range equalling its stage access's exact byte window, and an entry's bindings corresponding to its kernel's buffer parameters. Neither the byte windows nor the kernel signature travel. Carrying the byte window was considered and rejected: it is a value only the program establishes, so a carried copy would let a forged envelope assert a range no verifier examined, and the check would prove agreement between two producer-supplied fields rather than agreement with the plan. Both are folded into identity through the binding's expression content key and the entry's stage key, and the identity is re-derived and compared, so a forgery can restate them only by becoming a different artifact. That boundary is stated in `codec/validate.rs`'s module documentation.

### Required features, and the one this reader refuses

The feature set is **derived from content, never declared**, so a producer cannot understate what a reader must implement; `DeclaredFeatureMismatch` rejects an envelope whose declared set is not the one its content implies. Four keys are governed: `multi-variant-routing`, `deferred-predicates`, and `launch-preconditions` are emitted and supported; `multi-stage-program` is emitted and **not** supported. A multi-stage program's stage execution order is not recoverable from this profile — entries are ordered by canonical stage key, as identity orders them, and the dependency graph does not travel — so refusing to read it is the fail-closed form of that gap and treating declaration order as execution order would be the silent one.

### The governed digest

`docs/artifact-abi.md` requires an explicit governed algorithm and forbids inferring one from a digest width. The header carries a tag; `tiler.digest.sha-256.v1` is the only admitted algorithm; an unrecognized tag is `UnsupportedDigestAlgorithm`. SHA-256 is implemented in-crate (FIPS 180-4) rather than taken from a dependency, because the research contract records the production algorithm choice as an open bounded decision with a measurement attached, and adding the workspace's first cryptographic dependency would have answered it by accident. The implementation is pinned by the four published vectors including the one-million-character case, by every padding branch, and by a chunked-versus-single-shot agreement check; the domains are checked to be prefix-free, because a prefix separator only separates when no domain prefixes another. `select-the-governed-artifact-digest-implementation` owns the audited-crate comparison; swapping the implementation changes no encoded byte.

### Validation, and what each stage is worth

Monotonic and fail-closed, each stage a strictly weaker claim than the next: framing and integrity say only that these are the exact bytes someone wrote; canonical form says the manifest has one byte representation and this is it; structural validity says the tables close; re-proven model obligations say the content still satisfies the builder's rules; identity agreement says the content is the artifact the manifest claims. A decoded `ArtifactEnvelope` is a validated *envelope*, never a `VerifiedArtifactProgram` — nothing in the codec can manufacture a verified value.

Re-proven against decoded content, reporting the model's own typed cause rather than a codec restatement: non-empty portfolio, attributed plan, expression-arena reachability, payload reference closure, backend-entry injectivity, guard and deferred-predicate and accessible-range and launch and precondition use sites (type, availability phase, interface-only), deferred-predicate phase and selected authority, launch agreement with the entry's proven `threads_per_workgroup`, zero-work policy, and duplicate variants. Canonical order and distinctness are proven for features, providers, payloads, expressions, deferred predicates, launch preconditions, entries, and sections; the named interface is proven distinct without an order obligation, because its order is meaning; and a full re-encode-and-compare is the backstop for anything a named check missed.

### Adversarial evidence

The forged-model cases are the strongest: they build a structurally invalid envelope, encode it — which stamps a correct manifest digest, correct section digests, and the canonical identity of whatever it now claims — and require the decoder to reject it anyway. Corrupting bytes and watching a digest reject them proves little, because a forger recomputes digests; every byte-level case therefore reseals the framing before decoding, so what rejects it is the check under test.

Exact typed rejections asserted: `Truncated`, `TotalLengthMismatch`, `TrailingBytes`, `ManifestDigestMismatch`, `SectionDigestMismatch`, `ArtifactIdentityMismatch`, `UnknownTag{RoutingPolicy}`, `UnknownTag{ExpressionNode}`, `UnknownTag{ExpressionRoot}`, `UnsupportedEnvelopeFormat` (major and minor), `UnsupportedManifestSchema`, `UnsupportedComponentSchema{Program}`, `UnsupportedDigestAlgorithm`, `UnsupportedRequiredFeature`, `Limit{Expressions}`, `DeclaredFeatureMismatch`, `UnreferencedSection`, `NonCanonicalOrder` for sections, providers, payloads and expressions, `DuplicateItem{Expression}`, `DuplicateItem{InterfaceKey}`, `MissingReference{Expression}`, `ExpressionOperandOrder`, `ExpressionOperandType`, `ExpressionSelectBranchType`, `IdentityDerivation{AmbiguousCanonicalKey{Provider}}`, and `ModelObligation`/`ModelRule` for `EmptyPortfolio`, `MissingSelectedProvider`, `UnusedExpression`, `UnusedPayload`, `DuplicateBackendEntry`, `ExpressionType{ApplicabilityGuard}`, `LaunchDisagreement`, and `UnselectedDeferredAuthority`. Every truncation of the fixture envelope is rejected. Single-byte corruption is swept exhaustively over the framing header and the framed section stream — the regions read before any digest speaks for them — and sampled at a prime stride of 61 through the manifest interior, whose every byte one digest already covers; the boundary is recorded in the test rather than left as an unexplained loop bound.

`crates/tiler-artifact` now runs 90 unit tests: the 39 pre-existing model tests plus 51 codec tests (5 for the digest, 46 for the envelope), all passing.

### `#[non_exhaustive]` decisions

- `ArtifactCodecError`, `CodecLimitKind`, `ComponentSchemaKind`, `TagSubject`, `OrderedSubject`, `ReferenceSubject` — **marked**. Clause 5a: rejection and diagnostic vocabularies a consumer forwards or partially classifies; no crate maps them totally.
- `SectionKind` — **not marked**. Clause 5c: a section purpose is a recognizer, and `prototype-metal-bundle-assembly` will match it from `tiler-metal` to decide what it can assemble. A wildcard there would silently route a newly governed purpose into reject-unknown.
- `DigestAlgorithm` — **not marked**. Clause 5b: every consumer maps it totally, and a wildcard would have to invent a hash function.
- `NumericalFacts` — **not marked**. A caller-constructed leaf value-data record; 5a's stated asymmetry excludes input records.
- `SubnormalMode`, `NumericalPermission`, `ValueRole` (`tiler-ir`) — **left unmarked**, and this codec is a new out-of-crate consumer that maps each totally in both directions. They were already 5b types; `harden-public-enums-non-exhaustive` must not mark them.
- `KernelType`, `AddressSpace`, `BufferAccess` (`tiler-ir`) — **left as they are**, `#[non_exhaustive]`. The pre-existing cross-crate encoder still rejects an unrecognized variant; the codec's tag-to-variant direction is a closed map over a tag and needs no exhaustiveness.

### Awaiting Tom under ADR 0075

The codec is deliberately `pub(crate)`, so nothing landed that requires his approval. **The atomic decision is which facade it should get when a consumer needs it**; it is stated in the report accompanying this ticket with its alternatives. Until then the surface stands as a reviewed draft and `record-the-implemented-artifact-envelope-in-the-contract` holds the `docs/artifact-abi.md` update, which was deliberately not made here: the contract's "codec unimplemented" language is still accurate for the *public* surface, and `contracts/artifacts` is outside this ticket's scopes.
