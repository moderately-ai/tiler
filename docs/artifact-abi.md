---
schema: "tiler-doc/v1"
id: "tiler.contract.artifact-abi"
kind: "contract"
title: "Artifact envelope and Metal kernel ABI profile"
topics: ["artifacts", "abi", "metal", "runtime"]
contract_status: "accepted"
implementation_status: "partial"
evidence: ["tiler.research.artifacts.target-neutral-envelope", "tiler.research.cache.crash-race-protocol", "tiler.research.runtime.execution-contract"]
ticket: "synthesize-artifact-contracts"
---

# Artifact envelope and Metal kernel ABI profile

**Status:** accepted research contract; shared IR ownership established, a bounded neutral envelope codec implemented with an accepted capability facade, and a separate proof-case evidence sidecar implemented with an accepted facade

The private compiler proof constructs provisional program portfolios and artifact-construction inputs. ADRs 0070 and 0071 now assign authoritative target-neutral executable meaning and checked construction to shared `tiler-ir` representations; the proof-specific structs remain private until replaced in dependency order.

**Fact — canonical envelope serialization, canonical form, and integrity validation are implemented, and the codec's *capability* is reachable.** `crates/tiler-artifact/src/program/codec/` encodes, decodes, and re-validates the envelope this document specifies, in the bounded lockstep profile recorded under "Implemented envelope profile" below. On Tom's review of 2026-07-25 that capability was promoted: `VerifiedArtifactProgram::encode`, `decode_artifact`, and the `DecodedArtifact` read view are `pub`, and the envelope, encoder, decoder, and section types stay `pub(crate)` behind the private module ADR 0074 convention 7 prescribes. So an out-of-crate consumer obtains bytes and a validated view over them, and does not obtain the codec's internal layout.

**Decided by Tom on 2026-07-25, and implemented: a decoded envelope is a *dispatch record*, never a reconstruction.** It publishes the named interface, each variant's guard and executable entries, each entry's bindings with the interface entry or internal allocation each addresses, each entry's launch contract and backend entry symbol, and each *carried* payload's compilation subject and exact object bytes — and it never rebuilds the shared-IR program it names. A consumer that needs that program holds the one it compiled, and the envelope binds the two by canonical artifact identity. The accurate statement is that a bounded lockstep codec with an accepted capability facade exists, not that the artifact format is stable.

## Ownership boundary

This document owns envelope framing, wire DTOs and encoding, compatibility,
runtime fact binding, the *execution* of the routing commit and a portfolio's
variant priority, digests, failure classification, and backend payload
mappings. The IR contract owns program/portfolio meaning, canonical identity,
ABI-expression semantics, a program's applicability guard and entry ABI, the
ordered routing-commit lifecycle each program declares, and authoritative
verification;
adapters own device-specific loading, binding, and execution. A decoded envelope
is a **dispatch record** and never reconstructs a verified kernel program: a
decoder projects the entries, bindings, launch expressions, and payload
references it has validated, and binds them to the program that produced them
through canonical artifact identity rather than by rebuilding it. A decoder
cannot manufacture a verified value or retain a second editable authority, and
[ADR 0071](decisions/0071-use-checked-builders-for-shared-compiler-ir.md)'s
amended clause governs any future path that does yield an IR value out of an
artifact — such a path must run through `tiler-ir`'s own builders and consuming
verifiers, and must state at its own API boundary the registry dependency that
makes it possible. The implemented profile satisfies all of that. This replaced
a normative requirement that a decoder reconstruct shared IR through its checked
builders; the decision that replaced it, and what it cost, are recorded under
"Implemented envelope profile" below.

This document also owns the **proof-case evidence sidecar**, a second container
produced by the same crate and specified under "Proof-case evidence sidecar"
below. It is owned here for the two reasons that make its boundary with the
envelope a statement about artifacts: the negative claim — an artifact never
names a sidecar and validates and dispatches with none present — constrains what
this document's decoders and runtimes may require, and the governed digest
domain set is shared, so one authority has to hold both. The sidecar's *meaning*
is nonetheless not artifact semantics, and the section below states that
separation as the container's first property rather than as a caveat. Deleting
every sidecar in existence changes nothing this document says an artifact means.

This document describes the accepted first-backend Metal profile of Tiler's
target-neutral artifact concepts. `MetallibBundle`, Metal binding indices, and
direct Rust embedding are profile-specific; the compiler core must also admit
other target payloads and delivery mechanisms.

A metallib alone is not executable safely. The Metal profile pairs compiled code with a
versioned, machine-checkable contract describing executable plans, bindings,
formulas, guards, routing, numerical behavior, and target requirements.

## Target-neutral envelope

The artifact is one bounded, self-verifying envelope with a canonical neutral
manifest and length-delimited typed sections. The neutral layer owns semantic
interfaces, complete program portfolios, routing, guards, checked expressions,
logical ABI roles, feasibility requirements, and execution/failure boundaries.
Backend payload schemas own executable bytes and backend-only transport
metadata.

The neutral layer references a backend payload through a governed backend key,
representation key, payload digest, compatibility-contract reference, and an
opaque backend entry key. It does not contain Metal symbol names, buffer or
function-constant indices, Apple triples, or MSL versions. Those belong to the
Metal payload. A future CUDA payload can use cubin/PTX and CUDA parameter
metadata without changing the neutral program schema.

Every section descriptor contains its required/optional meaning, schema, exact
byte length, and digest. The header bounds total length, manifest length, and
section count before allocation. All executable and required metadata bytes are
hashed. Unknown required meanings fail closed; unknown optional sections may be
skipped only when their schema explicitly permits it. An external
`EnvelopeDigest` covers the exact complete encoding and is not recursively
stored inside itself.

The implemented descriptor carries all four of those fields. The implemented reader still admits no optional section at all, which is named under "Where the implemented profile is narrower than this contract" below rather than weakened here.

Integrity, structural validity, neutral-program validity, backend-payload
validity, declared target compatibility, live applicability, prepared-entry
feasibility, and launch feasibility are distinct monotonic validation stages.
Parse success never implies executable compatibility. See the
[target-neutral envelope research](research/artifacts/target-neutral-artifact-envelope.md).

## Implemented envelope profile

Everything in this section is a fact about `crates/tiler-artifact/src/program/codec/` as of commit `2da6f1d`. It records what one build writes and reads. It does not widen the normative contract above, and where the two differ the difference is named rather than resolved by rewriting either side.

### Maturity of the implementation

**Fact — the codec's capability is accepted and its layout is not.** `codec` is a private `mod` of `tiler_artifact::program`. Promoted on Tom's review of 2026-07-25 are the capability and the read view alone: `VerifiedArtifactProgram::encode`, `decode_artifact`, `DecodedArtifact` with the borrowed projections `DecodedInput`, `DecodedOutput`, `DecodedVariant`, `DecodedDeferredPredicate`, `DecodedEntry`, `DecodedBinding`, `DecodedNumerical` and `DecodedExpr`, `BindingTarget`, `SectionView`, `SectionPurpose`, the payload vocabulary, and `ArtifactCodecFailure`. `ArtifactEnvelope`, the encoder, the decoder, the row types those projections borrow from, and the governed constants — including the three digest domain separators — stay `pub(crate)`. Promoting any further item is named on ADR 0075's always-ask list and requires Tom's review before merge.

**What a consumer can do today.** Build a `VerifiedArtifactProgram` through `tiler_artifact::program` — itself a reviewed *draft* boundary rather than an accepted facade — read its canonical identity, encode it to bytes, decode bytes back into a fully validated `DecodedArtifact`, re-encode that decoded view, observe any typed codec rejection, and carry a backend payload. From the decoded view alone, and holding no program, registry, or producer code, it can also read the whole dispatch record: the named inputs and outputs with their declared shape and element type; each variant's applicability guard, declared target profile, feasibility rule set, and deferred predicates; each executable entry's stage key, proven resource requirements, declared numerical realization, launch contract, and launch preconditions; each binding's target, transport kind, element type, address space, access mode, alignment, and the accessible offset and byte range bounding what it reaches; and, for a payload whose object was carried rather than left pending, its compilation subject, backend entry symbol, transport slots, and exact object bytes. Every carried expression among those evaluates against bound `AbiFacts` through the shared IR's own evaluator.

**What a consumer cannot do today.** Reach an `ArtifactEnvelope`, an encoder, a decoder, one of the manifest row types those projections borrow from, or a governed digest domain; digest a subject under one of this crate's domains; or obtain a `VerifiedArtifactProgram` from bytes, which no decoder can produce for the structural reason recorded under "Deliberate exclusions" and which the dispatch-record decision makes deliberate rather than pending.

Four maturity claims stay distinct here. The framing, canonical manifest, section framing, required-feature mechanism, dispatch-record projection, and rejection vocabulary are **implemented**. The section-purpose vocabulary and the carried-payload entry point are **implemented and exercised from a real emission and a real compilation** — `prototypes/serial-sum-compile` fills a payload from an actual `MetalTranslationUnit` and `metallib` and carries it in an envelope that decodes back — while remaining a **reservation** for every backend other than Metal. The codec's *capability* facade is **accepted**; a facade over the envelope's own layout is an **architectural seam** with no accepted shape. The properties labelled Measurement below are **tested guarantees over the named fixtures**, not universal claims about every artifact.

### Framing header

**Fact — the header is exactly 69 bytes, fixed width, big-endian, with this layout.**

| Offset | Width | Field |
| --- | --- | --- |
| 0 | 8 | magic `TILERART` |
| 8 | 2 + 2 | envelope format `{major, minor}`; `{1, 0}` in this build |
| 12 | 2 + 2 | canonical encoding profile `{major, minor}`; `{1, 0}` in this build |
| 16 | 1 | governed digest algorithm tag; `0x01` is `tiler.digest.sha-256.v1` |
| 17 | 8 | total encoded length |
| 25 | 8 | canonical manifest length |
| 33 | 4 | framed section count |
| 37 | 32 | digest of the exact canonical manifest bytes |

Total length, manifest length, and section count are read and checked against their governed budgets before a byte of the body is copied, so a forged length reports truncation rather than making a reader reserve memory for content that is not there. The total length is derived from the completed encoding rather than declared by a producer, and a supplied byte run whose length disagrees with it rejects as `TotalLengthMismatch` before any digest is computed.

**Fact — no in-band digest covers the header, and every header field is still pinned.** The manifest digest covers the manifest; each section descriptor's digest covers that section's bytes; nothing hashes the header itself. Each header field is nonetheless decided by a named check — magic by `BadMagic`, the two version pairs by `UnsupportedEnvelopeFormat` and `UnsupportedCanonicalEncoding`, the algorithm tag by `UnsupportedDigestAlgorithm`, total length by `TotalLengthMismatch`, section count by `SectionCountMismatch`, and manifest length and manifest digest by `Truncated` or `ManifestDigestMismatch` — with any residue caught by the re-encode equality described below.

**Measurement.** Flipping each of the 69 header bytes, a prime-strided sample of the manifest, and every byte of the framed section stream of the default fixture yields a rejection in every case. Every proper prefix of that encoding is rejected, and one appended byte is rejected both before and after the declared total length is repaired.

### Canonical manifest

**Fact — the manifest opens with the versioned domain tag `tiler.artifact-envelope.manifest.v1\0` and its own `{major, minor}` schema**, then the four governed component schema versions — program, ABI expression, guard-and-routing, target-requirement — and then, in this order: the routing policy tag; the derived required-feature set; the three reached semantic subjects; the named inputs and outputs with declared shape and element type; the selected capability providers; the backend payload descriptors; the shared ABI expression arena; the plan variants, each with its guard, declared target profile and feasibility rule set, deferred predicates, and executable entries; the section descriptors; and the artifact's canonical identity.

Each executable entry carries its stage subject, proven resource requirements, declared numerical realization, ABI bindings, launch contract, and backend entry.

**Fact — a selected capability provider row carries two independent revisions and no third version.** It is the provider's identity and its own output-affecting revision, the governed capability key, and that *capability's* output-affecting revision. [The operation-extension contract](operation-extensions.md) makes the two revisions independent — one provider may register several capabilities that move at different rates — so the capability revision is not derivable from the provider's, and an artifact that folded only the provider's would let a provider change what its lowering emits and produce a byte-identical identity. Both are received from the compiler and neither is re-derived here.

The row deliberately carries **no** capability-API or compiler version, and as of 2026-07-27 that is a settled decision rather than an open gap. The [operation-extension contract](operation-extensions.md#semantic-and-provider-identity) retires the requirement with its derivation: providers are statically linked, so a capability-API mismatch is a compile error rather than something an artifact could record; and a compiler version is discharged by the payload's compilation subject, which folds the exact compiled source, the flags, and the toolchain provenance into artifact identity. A field that existed for the first was once filled by narrowing the capability revision into it — a conflated value rather than a recorded one — and removing it was the fail-closed reading. Nothing needs to be added back.

Replacing that field moved the manifest schema to **4.0**. As with the earlier steps this is a **major** version rather than a minor one, and for a stronger reason than the layout arguments below: the field's *width* changed from two bytes to four, so a `3.0` reader would not merely misread a value — it would lose framing for every row after it. The identity domain moved with it, to `tiler.artifact-program.v3` and the selected-provider record's own to `tiler.artifact-program.provider.v2`, so a pre-existing artifact identity and a current one can never be compared as equal by accident.

**Fact — an ABI binding row carries the *offset* its accessible range starts at, beside the extent.** Both are references into the shared ABI expression arena, both are derived from the packaged program rather than restated by a producer, and both are folded into artifact identity by canonical position. They are derived differently, because the program states them differently: the extent is the program's own accessible-byte *expression*, replayed onto the artifact arena, while the program states where a byte window starts only as a constant, so the builder mints that constant as the offset's literal. The row keeps an expression reference rather than a plain number so a program that one day computes its window offset can carry that formula without a schema step.

Adding it moved the manifest schema to **5.0** and the identity domain to `tiler.artifact-program.v7`. Major again, and additive only in the sense that no field was removed: the offset is inserted ahead of the extent inside a fixed-width record, so a `4.0` reader would consume it *as* the extent and lose framing for every row after it. Every artifact's identity bytes move at this step, which is intended — a `v6` identity described a record with no way to state a placement, so an artifact carrying one is not the same subject as the artifact carrying its `v7` restatement.

**Fact — every variable-length run carries a fixed-width `u64` length before its content**, so no concatenation of fields is ambiguous, and **every encoded enumeration is written through the one governed tag table its vocabulary owns**, never through a Rust discriminant, so inserting a variant cannot silently renumber a value already on disk. Each table is a forward and inverse pair kept in one place and pinned by an exhaustive round-trip test.

**Fact — a well-formed but non-canonical encoding is refused rather than normalized.** Named checks reject an out-of-order or repeated feature, interface key, provider, payload, expression, deferred predicate, launch precondition, executable entry, or section. Because this reader understands every field, the decoder then re-encodes the validated envelope and requires byte equality, rejecting any residual non-canonical spelling as `NonCanonicalManifest`. One artifact therefore has exactly one byte identity.

### What is meaning and what is canonical

**Fact.** Variant order is routing priority, named-interface order is the semantic interface's, and ABI binding order is the kernel signature's; all three are retained. Provider, payload, deferred-predicate, launch-precondition, executable-entry, expression-arena, and section order are replaced by the canonical content order artifact identity already uses. The arena's canonical order is the unique topological order that always emits the smallest available node by canonical content key.

**Measurement.** Declaring the same payloads and providers in reversed order produces byte-identical envelopes, as does minting the same formulas through two different arena assembly orders.

### Identity is derived from the canonical envelope

**Fact — there is one identity encoder and its subject is the envelope.** `encode_identity` is a function of `ArtifactEnvelope`, and the checked builder's terminal projects the verified draft into its envelope before deriving the identity. A decoder re-derives the identity from decoded content through that same function and compares it with the identity the manifest carries, rejecting a mismatch as `ArtifactIdentityMismatch`. There is therefore no second encoder that a decoder would have to agree with by inspection.

**Inference — equal identity implies equal envelope bytes, and three closure checks are what make that true.** Identity replaces arena positions, payload positions, and section positions with canonical content keys, so it does not by itself fix the tables those positions index. An envelope carrying an expression no use site reaches, a payload no entry realizes, or a section no variant references would keep the same identity while changing the bytes, giving one artifact two byte identities. The decoder rejects all three. This closure is what lets an envelope digest serve as a cache key.

**Fact — a re-proven obligation reports the artifact model's own cause.** A decoded envelope is checked again against the rules the transactional builder discharged at construction, and each rejection carries the model's own typed cause rather than a codec-local restatement: one variant wraps an insertion-time build error, another wraps a whole-artifact diagnostic. A rejection therefore reads the same whether an artifact was refused at construction or at load.

**Fact — two builder obligations are not decidable from an envelope and are pinned by identity instead.** A binding's accessible offset and byte range must equal the exact byte window its stage access addresses, and an entry's bindings must correspond one-to-one with its kernel's buffer parameters. Neither the byte windows nor the kernel signature travel in this profile, so a decoder cannot recompute them. Both are folded into the artifact's canonical identity — through the binding's two expression content keys and the entry's stage key — and the identity is re-derived and compared, so a forged envelope can restate them only by becoming a different artifact. Carrying the byte windows so the check could run locally was considered and rejected: the window is a value only the program establishes, so a carried copy would prove agreement between two producer-supplied fields rather than agreement with the plan.

### Required features

**Fact — the required-feature set is derived from content and never declared by a producer**, so it cannot understate what a reader must implement; a declared set the content does not imply rejects as `DeclaredFeatureMismatch`. This build derives five governed keys:

| Governed feature key | Derived when | This reader supports it |
| --- | --- | --- |
| `tiler.artifact.feature.multi-variant-routing` | the portfolio carries more than one variant | yes |
| `tiler.artifact.feature.deferred-predicates` | any variant defers a feasibility predicate | yes |
| `tiler.artifact.feature.launch-preconditions` | any entry declares a launch precondition | yes |
| `tiler.artifact.feature.embedded-payload-code` | any payload carries its object bytes | yes |
| `tiler.artifact.feature.multi-stage-program` | any variant dispatches more than one stage | yes |

`tiler.artifact.feature.embedded-payload-code` is required rather than optional for a reason the mechanism exists to serve: a reader that predates carried payloads would see the descriptors and none of the code and would have no way to notice, because the manifest it understands is complete on its own. Requiring the feature makes such a reader refuse rather than load an artifact whose executable half it silently dropped.

**Fact — this build emits `tiler.artifact.feature.multi-stage-program` and reads it.** Each variant carries its stage execution order and the typed dependency edges that order discharges, both derived by the producer from the packaged program's own `execution_order()` and `dependencies()` rather than stated. A decoder proves the order is a permutation of the variant's entries and that every edge's predecessor precedes its successor in it, so an order contradicting the program's dependency graph is refused rather than executed. Entries themselves remain in canonical stage-key order — identity's order, not execution order — which is why the separate order row exists. `carry-the-stage-execution-order-in-the-envelope` closed this; a *runtime* still dispatches one entry, which is `preflight-every-entry-of-a-multi-stage-route`'s.

### Sections

**Fact — the section vocabulary has three governed purposes in this build**: the canonical kernel-program identity of one packaged variant, and a carried backend payload's compilation subject and its object bytes. Two variants that package the same program share one section, and two payloads carrying the same object share one section, because content is the address — so sharing is a stated property of these purposes rather than an accident of equal bytes. Sections are ordered canonically by content; duplicates, unreferenced sections, a section identifier that is not its canonical position, and an unrecognized purpose tag are each rejected by name.

**Fact — a section carries canonical identity bytes, not a digest of them.** ADR 0074 convention 2 makes a canonical identity an opaque byte encoding and short digests presentation-only, so the governed bound on one section is the shared IR's own identity budget rather than a digest width.

**Fact — the backend metadata and code sections landed, and the identity question this contract left open for them is decided.** `prototype-metal-bundle-assembly` added the two governed purposes and decided that **a carried payload is content-addressed over its compilation inputs, and the emitted object is opaque**. The descriptor's digest is required to equal the identity of the exact canonical payload-metadata bytes — source, target, flags, toolchain provenance, entry mappings, and recorded obligations, and no object byte at all — and `push_carried_payload` derives it rather than accepting one, so a payload cannot claim a subject it does not carry; `PayloadIdentityMismatch` re-proves it on every decode. Content-addressing over the emitted bytes was rejected because it would make artifact identity a function of compiler-output reproducibility, which this document refuses to promise under "Artifact identity" below.

**The cost of that choice is recorded rather than hidden.** *Equal identity implies equal bytes* now holds for the identity-bearing part of an envelope and not for the object sections, so two bundles built from one compilation subject by a non-reproducible linker have equal artifact identity and different envelope digests. The expansion cache is keyed by artifact identity, which this contract already requires, and an envelope digest names one published encoding rather than the artifact. The object section still carries its own content digest, so a substituted library is refused — as a corrupted encoding of this artifact rather than as a different artifact.

### The governed digest

**Fact.** The envelope names its digest algorithm by an explicit header tag and a reader never infers one from a digest width. `0x01` is `tiler.digest.sha-256.v1`, the only admitted value in this build. The envelope governs three domain separators as fixed NUL-terminated crate constants, so `H(domain || bytes)` genuinely separates its subjects rather than colliding a longer domain with a shorter one plus leading content:

```text
manifest_digest = H("tiler.artifact-envelope.manifest-digest.v1\0"
                    || exact canonical manifest bytes)
section_digest  = H("tiler.artifact-envelope.section-digest.v1\0"
                    || section purpose tag || section content schema
                    || exact section bytes)
envelope_digest = H("tiler.artifact-envelope.envelope-digest.v1\0"
                    || exact complete envelope bytes)
```

**Fact — the no-prefix obligation is over the crate's *seven* governed domains, not over these three, and it is normative rather than incidental.** Domain separation by prefix is a property of the whole admitted set: the three above and the proof sidecar's four (recorded under "Proof-case evidence sidecar" below) are hashed by one algorithm in one process, so a domain added to either container could collide with one in the other. A check confined to one container would report a separation it had not established. `crate::proof::tests::no_governed_domain_of_either_container_prefixes_another` checks the union and is the authority for the property; `crate::program::codec::digest`'s own three-domain test is the envelope-local half and names the union test as the authority. **A new governed domain in either container must be added to the union check**, and adding one to the envelope-local check alone does not discharge this obligation.

A section descriptor is derived from its section's position and exact bytes at encode time and re-derived and compared at decode time, never stored beside the bytes it describes, so the two cannot disagree in memory. The envelope digest is computed and never stored in band; a test asserts its bytes occur nowhere in the envelope it covers.

**Fact — the section digest binds the section's purpose and content schema, so it is a standalone content address.** The pre-image is the domain separator, the purpose tag, the content schema, and then the exact section bytes; the qualifiers are fixed width and precede the variable-length content, so no length prefix between them is needed for the pre-image to be unambiguous. Binding the purpose is what makes the digest usable *outside* a complete envelope, which is what content-addressing a backend code section does. Inside an envelope the purpose was already bound one level up, by the manifest descriptor that names it and the manifest digest that covers that descriptor; lifted out, a digest over bytes alone would give two sections of different purposes one address.

**Fact — the algorithm implementation is in-crate and its selection remains an open bounded decision.** SHA-256 is implemented in the crate rather than taken from a dependency, pinned by the FIPS 180-4 published vectors and by every padding branch, because adding the workspace's first cryptographic dependency would answer that decision by accident. The wire contract commits only to the governed tag, so replacing the implementation with an audited crate leaves every encoded envelope byte identical. `select-the-governed-artifact-digest-implementation` owns the comparison.

### Deliberate exclusions

**Fact — the frozen registry snapshot never enters the envelope.** ADR 0072 keeps the provenance of providers a plan never used out of packaged artifact identity, and carrying the snapshot here would put it back into the envelope's bytes and therefore into its digest, letting an unused provider invalidate a cache entry. Only the three reached subjects travel: the semantic graph identity, the reached definitions, and the admission provenance. **Measurement.** An artifact built with an additional available-but-unreached provider encodes to identical bytes and an equal envelope digest, while changing a *reached* provider's revision changes the digest.

**Fact — presentation-only declaration order never enters it**, under the ordering rules above. This is what makes the envelope's bytes a function of the artifact's identity rather than of the order a producer happened to declare things in.

**Fact — no backend spelling enters the neutral manifest, and the emitted object enters no identity.** The manifest names a payload by governed backend and representation keys, its own schema version, the digest of its compilation subject, and its execution policy, and by nothing a backend would spell: no symbol names, binding indices, platform triples, or language versions occur there. Since `prototype-metal-bundle-assembly` those backend spellings and the emitted object *are* carried, each in its own governed section rather than in the manifest — the compilation subject in a `BackendPayloadMetadata` section, which artifact identity folds through the descriptor's digest, and the object bytes in a `BackendPayloadCode` section, which artifact identity excludes entirely. The exclusion that survives is the sharp one: what an artifact *is* does not depend on what a linker emitted.

**Fact — a reconstructable kernel program is not carried, and the blocker is structural rather than an omission.** `tiler_ir::program::KernelProgramBuilder::new` takes a `&SemanticProgram`, which requires a frozen semantic registry holding live inferencer implementations; neither is representable as bytes. A decoded envelope therefore proves *which* program an artifact names and cannot resurrect it, and a consumer that needs a verified kernel program must hold the one it compiled.

**Decision — Tom, 2026-07-25, on `carry-reconstructable-kernel-programs-in-the-neutral-envelope`: a decoded envelope is a dispatch record, never a reconstruction.** It carries entries, bindings, and launch expressions as encoded facts a decoder projects and validates against the packaged program's canonical identity, and it never rebuilds a `VerifiedKernelProgram`. Full IR reconstruction was excluded on evidence rather than preference: the registry the builder needs holds behaviour, not data, so the option was impossible at any encoding cost rather than merely expensive. This is what the ownership boundary above now states, in place of the requirement that a decoder reconstruct shared IR through its checked builders.

**The accepted cost, stated rather than left for a reader to discover.** A binding's target is the one dispatch fact a decoder cannot re-derive: the program that established it reaches the envelope only as identity bytes, so the correspondence is carried rather than recomputed on read. It is not, however, *asserted* by a producer — `ArtifactProgramBuilder::check_bindings` derives it from the program's own stage access, so a producer cannot state a correspondence its plan contradicts, and artifact identity folds it, so a forged envelope restating a target becomes a different artifact. What is weaker than re-derivation is that the proof happened on the writing side, and only a consumer that compares identity has rejected the forgery. What *is* decidable from the manifest alone — that a target names an interface entry the artifact declares — is checked on every decode as `UnknownBindingTargetKey`.

### Governed budgets

**Fact.** Every budget is enforced on both sides by one constant. The encoder checks a projected envelope up front, so a legally built artifact that could not be read back fails to encode rather than producing bytes no reader admits; the decoder checks each count the moment it is read and before anything is allocated for it. The envelope-level bounds are 256 MiB per complete encoding, 64 MiB per manifest, 64 MiB per section, 16 MiB per received opaque identity subject, 4 KiB per encoded text run, 64 required features, 4,096 named interface entries, and 4,096 declared shape rank. Every per-collection bound — variants, entries, bindings, expressions, payloads, providers, deferred predicates, launch preconditions — reuses the artifact model's own constant rather than introducing a second authority for the same limit that would drift from it.

**Fact — a received opaque identity is bounded by the authority that mints it, not by its shape.** A governed key is bounded at 256 UTF-8 bytes because this layer governs what a producer may name. An opaque identity is bytes another authority derived, and the number that admits every value that authority can legally mint is that authority's own; anything else is this layer deciding a question it has just said it does not decide. So a backend entry key — the canonical identity of the structured kernel one executable entry realizes — is bounded by `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES`, 16 MiB, the exact constant the shared IR enforces when it mints one. A payload content digest is fixed-width under the governed digest algorithm and stays under the artifact layer's own 1,024-byte digest bound, and so, provisionally, does a target-profile descriptor identity, whose bytes are a canonical encoding rather than a hash and for which no authority publishes a bound yet.

**Measurement — the three identities never shared a subject, and one shared bound made the artifact contradict itself.** Until this was separated, all three were bounded at 1,024. On this checkout (Apple M4 Max, macOS, the pinned toolchain), the canonical kernel identity of the prototype's serial `f32` sum measures 736 bytes reducing one contributor and 1,121 bytes reducing two, three, four, or eight — it is the reduction *structure* that crosses 1,024, not the data size, so the bound admitted the degenerate reduction and refused every real one, and `prototypes/serial-sum-compile` could not package the program `prototypes/serial-sum-run` dispatches. It grows with program structure without a small ceiling: 1,483 and 1,845 bytes at rank 3 and rank 4 reducing one axis, and 1,700 to 2,279 bytes across ranks 5 to 8 reducing every axis but one, so no round number below the minting authority's own is safe. The contradiction was internal rather than merely inconvenient: an executable entry carries that same identity **twice**, once as its backend entry key and once inside the stage subject `stage_key` derives from it, and the second was already admitted to 16 MiB — so the smaller bound refused values the envelope beside it had accepted and guarded no allocation the stage subject had not already made.

### Rejection vocabulary

**Fact.** Failure is typed and non-erasing. A rejection names the boundary that refused and the subject it refused; a framing, schema, canonical-form, structural, or identity failure is never reinterpreted as a plan-applicability miss; and a rejection never yields a partially validated envelope.

| Boundary | Typed causes |
| --- | --- |
| Framing and integrity | `Truncated`, `TrailingBytes`, `TrailingManifestBytes`, `BadMagic`, `BadManifestDomain`, `BadPayloadMetadataDomain`, `TotalLengthMismatch`, `ManifestDigestMismatch`, `SectionDigestMismatch`, `SectionLengthMismatch`, `SectionCountMismatch`, `NonCanonicalSectionId` |
| Schema and feature compatibility | `UnsupportedEnvelopeFormat`, `UnsupportedCanonicalEncoding`, `UnsupportedManifestSchema`, `UnsupportedComponentSchema`, `UnsupportedSectionSchema`, `UnsupportedPayloadMetadataSchema`, `UnsupportedDigestAlgorithm`, `UnsupportedRequiredFeature` |
| Canonical form | `NonCanonicalOrder`, `DuplicateItem`, `NonCanonicalManifest`, `DeclaredFeatureMismatch`, `UnreferencedSection` |
| Structure and closure | `MissingReference`, `SectionPurposeMismatch`, `SectionDispositionMismatch`, `EmptyBindingTarget`, `UnknownBindingTargetKey`, `UnmappedBackendEntry`, `EntryTransportCardinality`, `ExpressionOperandOrder`, `ExpressionOperandType`, `ExpressionSelectBranchType`, `UnknownTag`, `InvalidText`, `InvalidGovernedKey`, `InvalidInterfaceKey`, `InvalidProviderIdentity`, `InvalidShape` |
| Governed budgets | `Limit` |
| Re-proven model obligations | `ModelRule`, `ModelObligation` |
| Identity | `ArtifactIdentityMismatch`, `PayloadIdentityMismatch`, `IdentityDerivation` |

Each variant carries the structured data a caller reacts to rather than a message: which collection was out of order, which enumeration presented an unimplemented tag with the rejected tag byte, which table a reference missed with the rejected index, which resource was exhausted with its attempted and permitted quantities.

**Fact — four of those decide the dispatch record, and each names a way an envelope could otherwise be wrong without being malformed.** `SectionPurposeMismatch` refuses a reference that resolves to a well-formed section of the wrong governed purpose, which would otherwise load an artifact whose executable half had been replaced by another section of its own envelope with no framing or integrity check failing. `UnknownBindingTargetKey` refuses a binding directed at an interface name the artifact does not declare. `UnmappedBackendEntry` refuses a payload that maps no symbol for an entry it realizes, and `EntryTransportCardinality` refuses a mapping whose transport slots do not correspond one-to-one with that entry's bindings — either of which would move the failure to a loader with less to say about it.

**Measurement — the adversarial cases build a structurally invalid envelope and then encode it**, which stamps a correct manifest digest, correct section digests, and a correct identity for whatever the forgery now says, and require the decoder to reject it anyway by name. Corrupting bytes and watching a digest reject them proves comparatively little, because a forger recomputes digests.

### Where the implemented profile is narrower than this contract

**Fact.** Four normative statements elsewhere in this document once described a format wider than the one this build writes. Items 1, 3, and 4 have since been closed and are retained here as the record of what closed them; item 2 alone is open, with a stated trigger rather than an open-ended gap. Item 3 closed differently from the other two: they were closed by widening the implementation to meet the contract, and it was closed by a decision that narrowed the contract to what the implementation can do.

1. **A section descriptor now carries all four declared fields, and one difference remains.** It is an ordered identifier, a purpose tag, the purpose's required/optional disposition, the purpose's content schema, the exact byte length, and the content digest. The one remaining narrowing is deliberate: the *digest algorithm* is named once in the header rather than per descriptor, because one envelope is digested under one governed algorithm and a per-descriptor spelling would admit an envelope whose sections disagreed about it.

   The disposition and the content schema are properties of the *purpose* rather than of the instance, and are written anyway, because the reader that needs them is precisely the one that does not recognize the purpose and so cannot derive them. A reader that does recognize a purpose owns the answer, and this build therefore requires a descriptor to agree with its own table — a disagreement is a descriptor asserting a schema or a skip permission rather than reporting one, and is rejected by name. Every purpose this build writes is `Required`, and an unrecognized purpose is refused outright, so no skip path exists yet; the field is the mechanism item 2 below will need.

   Adding both fields moved the manifest schema to **2.0** — a major step rather than the minor one it might look like. A minor step would have been wrong, because the reader admits `minor <= implemented` and would have gone on accepting a `1.0` manifest whose descriptors it can no longer parse. A field added inside a fixed-width record is not additive. The envelope format and the canonical encoding profile in the header are unchanged at `{1, 0}`: the manifest's layout moved, not the framing around it.
2. **There are no optional sections, so "unknown optional sections may be skipped only when their schema explicitly permits it" describes no implemented behaviour.** Every unrecognized purpose fails closed. This is the deliberate version-1 posture the envelope research records, and "Loading and validation" below already conditions the skip mechanism on exposing the format outside a lockstep release. It is an explicitly deferred question with that trigger rather than an unrecorded gap.
3. **A decoder does not reconstruct shared IR through its checked builders, and no longer owes it.** The requirement was withdrawn by Tom's decision of 2026-07-25 recorded under "Deliberate exclusions": a decoded envelope is a dispatch record, and reconstruction was excluded as structurally impossible rather than merely unbuilt. What survives of the original requirement is satisfied — nothing in the codec manufactures a verified value, a decoded envelope is a validated envelope rather than a second editable authority, and any future path that does yield an IR value out of an artifact remains bound by ADR 0071's amended clause. The dispatch record itself is implemented and public, as recorded under "Maturity of the implementation" above.

   **Future work under this decision, which the closure does not absorb.** Both facts a complete dispatch record would carry that this record listed as missing have landed. A variant's stage execution order **is** recoverable as of `carry-the-stage-execution-order-in-the-envelope`: the envelope carries the order and the typed dependency edges it discharges, and a decoder refuses an order that contradicts them. Entries still reach a reader in canonical stage-key order, which is identity's order, so a consumer sequences by the order row and not by the entry table. And a binding addressing only *part* of a value **is** packageable as of `carry-the-byte-offset-of-a-partial-binding-view`: the binding row carries the offset its range starts at beside the extent, both derived from the packaged program's own byte window rather than restated by a producer, identity folds them, and a decoder re-proves each at its own use site. What remains narrower is the runtime rather than the ABI, and it fails closed on both counts: a loader dispatches one entry, and it refuses a routed binding whose offset is nonzero rather than publishing an extent the host would silently bind at byte zero. `preflight-every-entry-of-a-multi-stage-route` owns the dispatch gap; `carry-the-binding-offset-through-the-runtime-route` owns honouring the offset at the binding call.
4. **The backend payload descriptor carries its compatibility-contract reference.** It carries the backend key, representation key, payload schema, content digest, the target profile the payload's own bytes were built against, and the execution policy. The reference is folded into the payload's canonical key and therefore into artifact identity, so two payloads that agree on every other field but were built against different profiles are two payloads rather than one.

   The field is the payload's contract, not the plan's. A variant's `TargetProfileRef` and `FeasibilityRuleSetRef` are the *plan's* declared target requirements, and the two coincide only while a payload is realized by one variant — which nothing in this model requires, since entries cross-reference payloads by index. Carrying it per payload is what lets a program share one compiled object across variants declaring different profiles and still state what that object was built for; without it a loader would infer the payload's contract from whichever variant it happened to route to, which is the inference this layer exists to forbid.

   The narrower alternative — declaring a payload per-variant by construction — was rejected on a concrete cost rather than on taste: it makes a legitimate program inexpressible. Two variants compiling to the same library could not share the payload under the new rule, and could not declare a second identical descriptor either, because the builder already refuses that as a duplicate.

## Proof-case evidence sidecar

A producer that compiles an artifact also knows what the artifact is *supposed to compute*, because it can evaluate the same semantic program through the target-independent reference evaluator. The proof-case evidence sidecar is the bounded, separately versioned container that carries that knowledge beside an artifact. Everything in this section is a fact about `crates/tiler-artifact/src/proof/` unless labelled otherwise.

### The separation from artifact semantics, and the two properties a consumer must not confuse

**Normative — a sidecar names an artifact and an artifact never names a sidecar.** No envelope section carries a proof case, no manifest field references one, and an artifact decodes, validates, classifies compatibility, commits routing, and dispatches with no sidecar present. The dependency runs one way, which is what makes proof data deletable without changing what a program means. A runtime that required a sidecar, or that read one to decide routing, a fallback, or a numerical realization, would be violating this contract rather than extending it.

**Normative — a validated sidecar is evidence of integrity and association, and is not evidence of authenticity.** Every digest and every identity in the container is derived from the container's own content, so a forger that rewrites an expected value recomputes all of them and the result validates *and binds to the artifact it names*. The container has no signature, no external trust anchor, and no key. `crate::proof::tests::a_forged_case_is_indistinguishable_from_a_real_one_by_the_container_alone` pins exactly this, so the limit is a checked-in fact rather than a sentence a later reader could mistake for a stronger guarantee.

**Inference — what therefore protects a proof run is the comparison, not the container.** A forged expectation makes a *correct* device fail the bitwise readback comparison, which is a loud result rather than a silent one. A consumer must treat sidecar payloads as **test data** and never as a semantic authority, a fallback value, a reference implementation, or an input to routing. Authenticity, if it is ever required, is a separate mechanism over these bytes and is not a property this container can acquire by adding another digest to itself.

### Facade status

**Fact.** `tiler_artifact::proof` is an **accepted facade**, promoted from the crate-private draft form of ADR 0074 convention 7 on Tom's review of 2026-07-25. Public are the producer's builder and its input records, the case-key and provenance-subject vocabulary, the verified product and the decoded read view with their accessors, the two association checks, the governed budgets, and the four typed rejection vocabularies with the total `ProofCodecError::classification` map. The promotion's reason is structural: the producer and the runner are different crates by construction, so nothing crate-private can let a case written by one be verified by the other.

**Fact — the wire form itself is not public.** The framing magic, the four domain separators, the schema versions, the manifest encoder, and the identity deriver stay private, so an out-of-crate caller cannot digest a subject under one of this container's domains or present bytes the reader did not derive. Broadening the surface to expose them would first require deciding what an out-of-crate producer of these bytes may claim about them, which — given the authenticity limit above — is not a question the container answers today.

### Framing

**Fact — the header is exactly 69 bytes, fixed width, big-endian.** It deliberately mirrors the envelope's discipline without sharing its bytes.

| Offset | Width | Field |
| --- | --- | --- |
| 0 | 8 | magic `TILERPRF` |
| 8 | 2 + 2 | sidecar framing format `{major, minor}`; `{1, 0}` in this build |
| 12 | 2 + 2 | canonical encoding profile `{major, minor}`; `{1, 0}` in this build |
| 16 | 1 | governed digest algorithm tag; `0x01` is `tiler.digest.sha-256.v1` |
| 17 | 8 | total encoded length |
| 25 | 8 | canonical manifest length |
| 33 | 4 | framed payload count |
| 37 | 32 | digest of the exact canonical manifest bytes |

The magic differs from the envelope's `TILERART` in the first differing byte, so a sidecar handed to the artifact reader and an envelope handed to the sidecar reader are each refused at the magic rather than misparsed. As in the envelope, the total length is derived from the completed encoding rather than declared, and every declared count is checked against its governed budget before anything proportional to it is reserved.

The header is followed by one canonical manifest and then a stream of length-delimited payloads.

### Canonical manifest and payload stream

**Fact — the manifest opens with the versioned domain tag `tiler.proof-sidecar.manifest.v1\0` and its own `{major, minor}` schema**, then, in this order: the associated artifact's canonical identity bytes; the digest of the exact encoded envelope bytes; the three provenance subjects; the bound input keys and output keys in the artifact's own interface order; the case table; and the sidecar's canonical identity.

Each case row is its stable key and the payload counts it declares in each direction, followed by one descriptor per payload: the payload's canonical ordinal, its exact byte length, and its content digest.

**Fact — payload position is structural rather than referential.** Payloads are framed in one canonical order — cases by stable key, then that case's inputs in interface order, then its expectations in interface order — and a case's descriptors are aligned with that order positionally. There is no payload index a manifest could point at, so the class of forgery in which one payload's descriptor names another payload's bytes does not exist in this format.

**Fact — the three provenance subjects are separately typed because they answer three different staleness questions.** The semantic-graph identity says which mathematical program was evaluated; the numerical-contract identity says under which contract its result is normative; the reference-implementation identity says which implementation computed it. A sidecar can be stale against any one while agreeing with the other two. Following ADR 0072, the frozen registry snapshot is deliberately absent: a provider that was available and never reached does not change what the program computes, so recording it would let an unused provider invalidate a still-correct expectation.

**Fact — the semantic subject is supplied by the producer and compared, not derived.** The risk the check exists to catch is a producer that reference-evaluated a different program from the one it compiled; deriving the subject from the artifact would make the check tautological. A mismatch is a build failure. The other two subjects are opaque bytes this crate compares and encodes and never re-derives, because it is not the authority for either.

**Fact — payloads are bit-preserving and are never interpreted as numbers.** A signalling NaN, a quiet NaN, a negative zero, and a subnormal all survive the container unchanged, which is the only reason a bitwise readback comparison means anything. A container that parsed floats would be free to canonicalize the first into the second and the comparison would then pass against the wrong value.

**Fact — a well-formed but non-canonical encoding is refused rather than normalized.** Named checks reject an out-of-order or repeated case key or interface key, a non-canonical payload ordinal, and a case whose payloads disagree with the container's own bound interface. The reader then re-derives the canonical identity from the decoded content, requires it to equal the carried one, and re-encodes the whole container and requires byte equality. One sidecar therefore has exactly one byte identity, and the identity a reader reports is always the re-derived one rather than the carried one.

### The sidecar's four governed digest domains

**Fact.** The sidecar's domains are its own; the *algorithm* is the envelope's governed one, because this document requires every digest use in this crate to name one governed algorithm explicitly rather than choose locally, and a sidecar that chose its own would be unverifiable by a reader that knows only the governed tag.

```text
manifest_digest = H("tiler.proof-sidecar.manifest-digest.v1\0"
                    || exact canonical manifest bytes)
payload_digest  = H("tiler.proof-sidecar.payload-digest.v1\0"
                    || payload's canonical ordinal || exact payload bytes)
manifest bytes    open with "tiler.proof-sidecar.manifest.v1\0"
identity bytes    open with "tiler.proof-sidecar.identity.v1\0"
```

**Fact — a payload digest binds the payload's canonical ordinal, so it is a standalone address of that slot's content.** Without the ordinal, two slots holding equal bytes would share one address and a swap between them would be invisible to the manifest.

**Fact — the canonical identity folds payload digests rather than payload bytes.** It covers the association, the three provenance subjects, the bound interface keys, every case key, and a content digest of every payload, so it stays bounded by the case and interface counts while still changing whenever any carried byte changes. That is what keeps it usable as a key for a sidecar carrying megabytes of evidence. It has no constructor: the encoder derives it and nothing else can.

These four and the envelope's three are the seven the union no-prefix obligation under "The governed digest" covers.

### Association is a decision, not a default

**Fact.** A decoded sidecar is fully validated evidence about *nothing* until it is bound. Two checks establish the same association and differ in what they re-prove.

- **`bind_to_envelope(&[u8])`** is for a consumer holding only bytes. It re-derives the envelope digest over the exact bytes supplied, decodes them through the artifact codec, and compares the re-derived artifact identity with the recorded one. Nothing is taken on the producer's word: both values are computed from the caller's own bytes. The digest check runs first, because it is the cheapest and it is the one failure that distinguishes damaged bytes from the wrong artifact entirely.
- **`bind_to_artifact(&VerifiedArtifactProgram)`** is for a consumer holding the program it compiled. It compares the same artifact identity and additionally re-proves every structural obligation locally: that the sidecar binds exactly the artifact's declared inputs and outputs in the artifact's own interface order, that each payload is a whole number of elements of its declared shape, and that all cases agree on each entry's byte length.

**Inference — the second is not a stronger *association*.** Both prove the same artifact identity, which already folds the ordered named interface. The difference is that the second re-proves the obligations rather than inheriting them through an identity comparison, which is what a reader wants when the sidecar was written by an older producer than itself.

**Fact — the obligation has one implementation.** `verify_cases` is called by the builder's terminal and by `bind_to_artifact`. A producer-side copy and a consumer-side copy would agree today, drift later, and each half would still pass its own tests.

**Fact — under the dispatch-record decision, a cold consumer reaches only the weaker check.** A decoded envelope is a dispatch record and never rebuilds a `VerifiedKernelProgram`, so a process that did not compile the artifact cannot obtain a `VerifiedArtifactProgram` and therefore cannot call `bind_to_artifact`. That is not a gap in the sidecar: `bind_to_envelope` establishes the same association from bytes alone, and the obligations `bind_to_artifact` re-proves were already proven by the producer and are folded into the artifact identity both checks compare.

### Governed budgets

**Fact.** Every bound is checked before any allocation proportional to it, in both directions: the encoder refuses to write a container a reader would not admit, and the reader refuses a declared count before reserving for it. The bounds are 256 MiB per complete encoding, 8 MiB per manifest, 8 MiB per derived identity, 16 MiB per case payload, 1,024 bytes per received provenance subject, 4 KiB per encoded text run, 256 proof cases, 256 UTF-8 bytes per stable case key, and 4,096 named interface entries per direction. The framed payload bound is *derived* from the case and interface bounds rather than declared, so the framing bound and the structural bounds cannot disagree.

**Fact — the interface bound deliberately equals the artifact model's own.** A sidecar binds one payload per declared entry, so a looser bound here would admit a container no artifact could ever associate with.

**Fact — no storage width is derived, and the omission is deliberate.** A payload is checked to be a whole, nonzero number of the declared shape's elements; the absolute byte width of a governed element type is a backend fact this crate does not own, and inventing one would assert a byte count no verifier examined. Divisibility plus cross-case length agreement catches the same class of producer error without the invention.

### Rejection vocabulary

**Fact.** Failure is typed, non-erasing, and never partially validated. Construction, encoding and decoding, and association each have their own vocabulary, and `ProofCodecError::classification` is a total map from every reader rejection onto five classes a consumer can act on — `Malformed`, `IntegrityFailure`, `Unsupported`, `Invalid`, `Limit` — so an out-of-crate reader never enumerates the `#[non_exhaustive]` variant set. Collapsing the classes would make a version skew look like corruption.

**Fact — a major version step is refused outright rather than read on a best effort**, for the framing format, the canonical encoding profile, and the manifest schema alike; a minor version this build predates is `Unsupported` rather than ignored. This is the same lockstep posture the envelope takes.

### Maturity, stated apart

The container, its canonical form, its integrity validation, its two association checks, and its facade are **implemented and tested**, including against re-sealed forgeries — the adversarial cases build a structurally invalid sidecar and then encode it, which stamps a correct manifest digest, correct payload digests, and a correct identity for whatever it now says, and require the reader to refuse it by name.

**Reserved and not implemented:** case grouping, per-case tolerance, any comparison policy other than bitwise equality, and execution ordering. Bitwise equality is the only comparison the numerical contract admits and the only one a container that never interprets its payloads can honestly support.

**Not a capability of this container at all:** authenticity, as recorded above. Broadening to it would require an external trust anchor and a key-management contract, neither of which exists in this workspace, and would not be a change to this format's digests.

**No producer or consumer ships one yet.** `prototype-metal-aot-slice` owns generating a sidecar from the real producer and `prototype-metal-runtime-proof` owns validating and comparing against one in the runner. Until both land, this section records a reachable and tested format rather than a delivered evidence path.

## Metal payload hierarchy

```text
Bundle
  = metallib bytes + canonical bundle manifest
Program plan
  = semantic input/output contract + complete physical plan alternatives
Plan variant
  = guards + temporaries + ordered/dependent kernel steps
Kernel entry
  = exactly one Metal symbol, ABI, and dispatch contract
Pipeline specialization
  = kernel entry + function-constant values + Metal device
```

Routing chooses among complete plan variants, not merely individual kernels.
This represents one-kernel fusion, materialized split plans, layout enforcers,
and multi-pass reductions with the same execution model. Every kernel entry has
one symbol and ABI; separately emitted scalar/vector kernels are separate
entries referenced by different plan variants or steps.

## Conceptual Metal payload view

```rust
struct MetalPayload {
    payload_schema: SchemaVersion,
    representation: MetalMetallib,
    compatibility: AppleMetalCompatibility,
    compiler: MetalCompilerProvenance,
    entries: Vec<MetalEntryMapping>,
    code_section: SectionId,
    optional_reflection_section: Option<SectionId>,
}

struct MetalEntryMapping {
    backend_entry_key: BackendEntryKey,
    neutral_entry: ExecutableEntryId,
    symbol: String,
    bindings: Vec<MetalBindingMapping>,
    function_constants: Vec<MetalFunctionConstantMapping>,
    dispatch_api: MetalDispatchConvention,
}

struct MetalBindingMapping {
    neutral_binding: EntryBindingId,
    transport: BufferIndex | InlineBytes | ConstantBufferField,
}
```

This is the Metal payload/profile view, not the neutral envelope schema. It is
illustrative, not a committed Rust API or serialization format. The
full canonical `KernelProgram`, program portfolio, neutral ABI, guards,
routing, checked launch expressions, numerical realizations, resources, and
named outputs occur exactly once in neutral sections. Metal metadata only maps
those stable neutral IDs to Metal transport and executable spellings. Any
duplicated neutral executable authority makes the envelope invalid. The
Milestone 2 one-kernel path remains a neutral program with one variant, no
temporaries, and one step.

The manifest carries governed capability-key and feasibility-rule schema
versions, exact/proven resource requirements, and each deferred predicate's
query contract, availability phase, and provenance authority. A
`target_profile_id` alone is not evidence that an individual variant is legal.
The descriptor hash covers canonical compatibility, compile guarantees, data
layout, execution/memory/vector models, phase schemas, artifact execution
policy, and rule-set/provider revisions. The display key and tuning-model key
do not substitute for that identity.

## ABI expression language

Shapes, metadata values, bounds, constraints, guards, dispatch, temporary
allocation, and routing need an executable representation. Tiler defines one
small, versioned, typed, side-effect-free `AbiExpr` language over:

- literals;
- input dimensions and element strides;
- a view's start element and allocation byte length;
- dtype byte size and admitted target/device properties;
- checked `u64` add, subtract, multiply, min, and max;
- floor, exact, and ceiling division plus remainder/divisibility;
- comparison, boolean composition, and conditional select;
- explicit checked narrowing to target fields.

Subtraction underflow, non-exact division, division by zero, invalid references,
overflow, and failed narrowing are typed evaluation failures. Conditional
evaluation supports zero-sized bounds without evaluating an invalid branch.
Parser expression depth and collection lengths are bounded. Shape formulas,
accessible ranges, metadata, allocation, dispatch, and routing reuse this
evaluator.

The domain type, admitted root vocabulary, validation, canonical identity, and
authoritative pure checked evaluation semantics belong to the executable
program IR in `tiler-ir`, as do the program-level *uses* of that language: the
applicability guard, each stage's launch geometry, and each access's accessible
byte range, all folded into `tiler.kernel-program.v2` program identity. This
artifact contract owns their versioned wire encoding, runtime fact binding and
phase checks, compatibility behavior, and failure classification, plus the two
use sites no single program can carry — a variant's launch preconditions and
its deferred feasibility predicates. It must not recreate a second editable
expression authority.

## Constraint, guard, and error outcomes

The runtime distinguishes three outcomes:

```text
semantic constraint failure
  -> invalid user/input operation; return a semantic error

plan applicability failure
  -> try the next plan variant or a compatible Tensor-level fallback

artifact/launch invariant failure
  -> fail closed; do not reinterpret it as an applicability miss
```

A split-axis factorization is a semantic constraint. Alignment required by a
vectorized plan is an applicability guard. A corrupt binding table or launch
overflow after plan selection is an invariant failure. Their provenance is
encoded and preserved in diagnostics.

Residual tensor-value preconditions are semantic validation obligations. A
plan records whether each is discharged by proof, host validation, device
pre-scan, or a transactional device result, plus its witness dependencies,
temporary/error-record roles, completion observation, and publication boundary.
The validation result is not encoded as an applicability predicate. A runtime
profile that cannot provide the required observability reports the semantic
operation as unsupported before device work begins.

The conceptual target-neutral portion of that manifest is:

```rust
struct ValidationObligationSpec {
    obligation_id: ObligationId,
    predicate_id: SemanticPredicateId,
    witness: WitnessDependency,
    stable_error_codes: Vec<SemanticErrorCode>,
}

struct WitnessDependency {
    witness_id: WitnessId,
    logical_subject: LogicalValueId,
    component_roles: Vec<ComponentRole>,
    logical_view: LogicalViewId,
    value_provenance: ValueProvenance,
    producer_dependencies: Vec<StepId>,
    coherence_requirement: CoherenceRequirement,
}

enum EnforcementPlan {
    ProofElided { proof: ProofRecordId },
    HostScan { evaluator: HostEvaluatorId },
    DevicePreScan { step: StepId, error: ErrorRecordSpec },
    TransactionalDevice {
        steps: Vec<StepId>,
        private_results: Vec<PlanValueId>,
        error: ErrorRecordSpec,
        publication: PublicationMode,
    },
}

struct ErrorRecordSpec {
    schema: SchemaVersion,
    obligation_id: ObligationId,
    logical_index_width: u8,
    stable_code_width: u8,
    reduction_order: ErrorPriorityOrder,
    storage_and_coherence: ErrorStorageContract,
}
```

This remains a schema contract, not a committed Rust representation. Error
priority is the canonical minimum of `(logical_linear_index,
stable_error_code, obligation_ordinal)`. Any backend-specific packed atomic
key must prove those widths lossless. First-writer order is not conforming.

The plan also declares its `CompletionObservation`: terminal completion,
post-completion status/error inspection, error-record coherence, record
validation, and semantic interpretation in that order. A transactional plan's
private-result closure includes all dependent work before publication. Initial
transactional support is out-of-place; mutation requires an explicit shadow or
undo capability. Publication mode distinguishes ownership promotion from a
copy/dispatch, because they have different ABI steps and costs.

## Binding contract

Before evaluating output shapes, semantic constraints, routing, allocation, or
dispatch expressions, the runtime constructs the program's bound semantic
environment from the manifest's `semantic_root_bindings`. Each binding records
the stable extent-symbol identity, binding class, declared value domain, and
source provenance. A target-property source additionally records its versioned
property key, required availability phase, and compatible provider contract.

Semantic root binding is distinct from kernel argument binding. A missing or
invalid semantic binding means the declared program interface cannot be
instantiated; it is not a physical-plan applicability miss. Fallback is legal
only when it consumes the same successfully bound semantic environment. An
artifact cannot reinterpret a target property as an ordinary plan guard when
that property changes observable tensor semantics.

Every kernel binding states:

- stable plan-value identity and Metal buffer index;
- buffer, metadata block, or scalar role;
- storage dtype and scalar width/signedness;
- read, write, or read/write access;
- address space and required alignment;
- alias/access-range constraints;
- explicit metadata layout and minimum accessible byte range.

A first-class semantic tensor may lower to multiple physical bindings. A
quantized tensor, for example, may require code, scale, zero-point, codebook, or
other scheme components. The plan records one logical value-to-component
expansion with stable ordered roles; every kernel ABI binding references the
logical plan value and component role. No backend may infer component meaning
from binding order alone.

Semantic scheme identity, component roles, parameter maps, and numerical
contracts participate in semantic and plan identity. Bit packing, component
interleaving, alignment, padding, and physical scale layout participate in
storage/ABI and artifact identity. Runtime component bindings are validated as
one logical value before any plan dispatch begins.

Every metadata field states its `AbiExpr` source, byte offset, scalar type,
size, alignment, and encoding. Host packing and MSL declarations are generated
from the same layout; Rust `repr(C)` is not the cross-language contract.
Boolean representation and inline-bytes versus constant-buffer transport are
explicit.

The initial buffer convention is:

- bind the Metal allocation buffer at byte offset zero;
- pass logical `start_element` as typed metadata;
- physical address derivation composes each logical tensor access with the
  selected `BufferView`, adds `start_element` exactly once, and produces an
  allocation-relative element offset;
- metadata strides are measured in elements;
- validate the derived allocation-relative range against allocation bytes.

There is no untyped integer “offset,” and the encoder does not also apply the
view start as a byte offset. A future binding-offset variant is a distinct ABI
convention. Negative-stride views are initially unsupported.

## Plan execution and dispatch

A plan variant declares all temporary tensor shapes, dtypes, allocation-size
formulas, allocation identities, value/view bindings, and lifetimes. The
initial profile assigns one allocation per output or temporary and permits no
temporary reuse, suballocation, in-place assignment, or output/input aliasing.
Inputs may alias one another. Steps form an acyclic dependency graph and carry a
canonical topological order. The initial execution profile uses one ordered
device command stream; incomparable DAG nodes are not implicitly concurrent.
Every output is fully initialized before it escapes, and temporary buffers
remain alive through their last GPU use.

Each kernel dispatch formula distinguishes total threads from threadgroup
counts, grid dimensions, threads per threadgroup, dynamic threadgroup memory,
zero-work behavior, and device-limit preconditions. It is evaluated with
`AbiExpr`; launch configuration is never reconstructed from output element
count alone.

## Routing and preparation

When several plan variants are applicable, a canonical routing policy selects
by piecewise cost, constraint region, or stable explicit priority. All variants
in one program have the same semantic and numerical contract. Routing is
versioned, explainable, and independent of manifest serialization order.
The verifier checks this equality per operation; routing never chooses between
different accuracy meanings. Variants may use different realizations only when
each independently refines the same contract.

Before any allocation or encoding, runtime preparation creates or retrieves all
pipelines required by the chosen plan. A pipeline-specific capability failure
may reject that plan and try the next semantically identical preflight-valid
variant. After allocation/encoding begins, the runtime does not retry another
plan or execute fallback.

Preparation refines compile guarantees with live-device and prepared-kernel
facts, then evaluates launch-instance requirements. Live facts are keyed by
device/context; prepared facts additionally key artifact, entry point, and
function constants, canonical pipeline descriptor/configuration, and relevant
archive/runtime mode. Neither becomes portable semantic identity.

`RoutingCommit` occurs only after route-sensitive launch preflight and final
variant selection. Compatibility/capability rejection may route before it;
artifact integrity, schema/ABI inconsistency, dishonest providers, systemic
runtime errors, allocation failure, and all post-commit failures close with an
error.

`EnforcementCommit` occurs when execution of the chosen unresolved semantic
validation begins, including a host scan. No variant or fallback may execute
after it. `PublicationCommit` occurs only after a successful witness and makes
the logical result externally observable. Proof-elided obligations have no
runtime enforcement commit. Device pre-scan places result dispatch after
successful completion observation; transactional enforcement keeps result and
dependent effects private until publication.

## Embedding contract

The proc macro embeds the canonical manifest and metallib as byte-string literal
tokens in its returned Rust expression. Runtime artifact construction borrows
those static byte slices; it does not open source files, compiler-cache paths,
or consumer `OUT_DIR`.

The embedding representation is deterministic and versioned. Artifact identity
is independent of the absolute compiler-cache location. Each manifest or
payload is emitted as one byte-string literal, never one numeric token per
byte. Linker/rustc deduplication is opportunistic and is not part of the storage
or correctness contract.

The initial measured gate is at most 1 MiB of direct bytes per invocation and
at most 32 invocations or 3.2 MiB of logical emitted bytes per consumer package,
whichever comes first. Crossing a gate requires an explicit diagnostic/override
and a measurement case; it is not a claim that Rust or Metal has a hard limit.
Because one proc macro cannot reliably observe a crate-wide total, integration
CI owns the package gate and reports logical bytes separately from actual
linked bytes. See the [embedding measurements](research/embedding/embedded-artifact-costs.md).

One embedded bundle contains all `KernelEntry` values required by that macro
invocation's plan portfolio. It is not required to contain kernels from other
invocations or crates.

## Specialization policy

Good expansion-time specialization dimensions include expression graph, rank,
storage/accumulation dtype, reduction axes, schedule family, a small set of
vector/tile choices, and layout family. Prefer runtime ABI values for extents,
strides, start offsets, and counts to avoid exact-shape artifact explosion.

Function constants are reserved for small choices that materially alter code.
Each specification includes Metal index/name/type, legal values, source
expression, default behavior, and related guards. Values participate in
pipeline-cache identity.

## Artifact identity

Expansion compilation identity includes a domain/schema separator, canonical
semantic, index, scheduled, and structured kernel IR, complete
program plans, semantic root-binding declarations, ABIs, guards, routing,
dispatch, numerical contract, translation-unit membership,
schema/helper/codegen versions, target/profile, compiler, flags, and every
selected conformance-evidence record digest and scope.

For Metal it additionally includes exact generated MSL and helper bytes,
normalized Apple platform family, requested deployment minimum, MSL language
standard, optimization/math/debug/line/include/macro/compiler/linker flags,
canonical SDK version/build and relevant content identities, and the resolved
`metal` and `metallib` component versions or executable digests. Absolute SDK
or temporary paths are provenance rather than portable key material when
equivalent content is otherwise established. Requested deployment minima stay
in identity even when a trivial measured kernel happens to produce equal bytes.

Those resolved component versions identify the offline compiler and only the offline compiler. Apple's runtime source compiler is a separately versioned build belonging to the execution environment rather than to the artifact, so no artifact identity can name it, and widening this list would not change that. The [Metal backend](backends/metal.md) contract records the measurement, the bounded cross-path agreement that accompanies it, and why Tiler's ahead-of-time exclusion is what keeps this provenance complete for every kernel Tiler compiles.

Target requirement predicates, the feasibility-profile descriptor/rule
identity, artifact execution policy, deferred query contracts/phases, and exact
resource requirements are likewise identity. Live fact values and prepared-
pipeline observations scope runtime caches and routing records rather than
portable bundle identity. Tuning-model identity is selection provenance unless
it changes the emitted portfolio or embedded manifest.

Transcendental implementation evidence is explicit artifact provenance rather
than an implied consequence of a compiler flag. Each claim identifies its
class (proof, exhaustive, normative guarantee, empirical, or unknown), scope,
reference oracle, implementation/helper digest, toolchain, target/device where
applicable, and test-corpus digest. Empirical qualification cannot satisfy a
hard worst-case semantic bound.
Evidence is not semantic identity, but changing the evidence record, target or
toolchain scope, or classification changes manifest, bundle, and expansion-
cache identity even when generated code bytes happen to remain equal.

**Decision — 2026-07-27, on `decide-whether-layered-subject-digests-exist-as-hashes`: canonical bytes are the only layered identity Tiler has, and there are no layered digests.** This block previously carried five further derivations — `semantic_digest`, `index_digest`, `schedule_digest`, `refinement_digest`, and `plan_digest` — described as a proposal for deriving a compact key from a subject. They are removed rather than specified.

ADR 0074 convention 2 makes a canonical identity an opaque newtype over its exact canonical byte encoding, with short digests presentation-only, and every layered identity the workspace derives is built that way: the semantic graph, index region, scheduled region, kernel program, and artifact program identities are canonical bytes compared byte for byte, never hashes. Hashing occurs at exactly three sites, all of them envelope framing, and all three are specified verbatim under "The governed digest" above — `manifest_digest`, `section_digest`, and `envelope_digest`, each over a NUL-terminated `tiler.artifact-envelope.*` constant. They are not restated here: this block previously carried its own spellings of the same three, they did not match the crate constants, and only the caveat calling every separator in the block illustrative kept that from being an error. One authority for a governed constant is the point.

**Why specifying them was rejected rather than deferred.** A compact key's only value is being shorter than the canonical bytes, which means trading collision-freedom for width. Applied to a subject that *already* has a canonical-byte identity, that produces a second identity authority over one subject — the shape ADR 0082 names, whose agreement with the real identity could only ever be argued and never checked. Nothing in the tree computes or consumes such a key, no crate exposes one on any of the five types, and the expansion cache keys on a `ComposedSubject` of length-prefixed canonical byte runs rather than on layered digests. Writing a more precise promise that nothing implements would have deepened the divergence this block existed to flag, not closed it.

**What would reopen it.** A consumer that genuinely needs a bounded-width cross-reference to a layer — an external index over layers is the plausible one, and none exists. Such a proposal has to answer the second-authority problem first: which of the two values *is* the identity, what happens when they disagree, and what checks that they do not. Until then the answer is that a consumer wanting a bounded value for presentation uses a presentation label, which is explicitly not an identity.

Section digests are stored only in manifest section descriptors. The manifest
digest is stored only in the framing header and covers the exact manifest bytes,
which contain no `manifest_digest` or `envelope_digest` field. `EnvelopeDigest`
is externally derived and never stored inside the envelope it covers. A layer's identity is
carried as its canonical bytes, so a cross-reference to one is those bytes and
not a digest of them. No field is hashed through a zeroing convention or
recursive definition.

Stable canonical IR, MSL, manifest, and cache keys are required. Tiler promises
deterministic source, manifest, and identity construction; it does not promise
byte-identical Apple output across machines or toolchain builds. A cache hit
validates stored payload bytes and never depends on recompiling to reproduce
them.

## Expansion cache contract

The expansion cache stores one immutable, self-validating bundle per complete
compilation key. The required protocol is:

```text
validate lock-free candidate
  -> on miss, open stable per-key lock file and acquire an OS advisory lock
  -> recheck after acquisition
  -> compile into process-owned state
  -> write a create-new unique temporary bundle on the final filesystem
  -> reopen and fully validate the temporary bundle
  -> atomically rename it over the content-addressed final path
  -> release the lock by closing its descriptor
```

The lock suppresses duplicate compiler work; it is not the correctness
boundary. Correctness comes from complete identity, bounded validation on every
hit, immutable final entries, and atomic publication. A killed process releases
its OS lock. There are no PID leases or stale-lock deletion rules. Internal GC
retains lock files and acquires the same key lock before eviction; lock-free
readers validate their already-open descriptor.

The default durability claim is process-crash safety, not power-loss
durability. A separate `fsync` policy may synchronize the temporary file before
rename and the containing directory afterward, but Darwin does not make that a
universal physical-media guarantee. Cache read/write/lock/publication failures
fall open to validated uncached compilation. Compiler failures, unsupported
targets, and invalid artifacts remain hard expansion errors.

Rust's standard `File::lock` requires an MSRV of at least 1.89. Choosing an
older MSRV requires a separately audited lock adapter. See the
[crash/race protocol and harness](research/cache/crash-and-race-protocol.md).

### Supported filesystems

The protocol requires six properties of the filesystem holding a cache root: the
temporary and final trees share one filesystem; `rename` replaces the final path
atomically; `create_new` refuses an existing path; a descriptor opened before an
unlink keeps reading; an exclusive advisory lock on the per-key file excludes
every contender; and a modification time is reported. A root is supported when
all six hold. `spikes/cache/filesystem_probe.rs` measures them against a
candidate directory and exits non-zero when one is refuted.

**The supported set is local APFS and local exFAT on macOS, both measured**, and
nothing else. `AGENTS.md` states that Tiler develops on macOS only and that
other platforms are unsupported rather than maintained as untested branches, so
a Linux row here would name a platform this product does not support on evidence
that was never gathered. A derivation of ext4, btrfs, and xfs from POSIX and the
Linux manual pages is retained as **inactive research** in
[`docs/research/cache/supported-filesystems.md`](research/cache/supported-filesystems.md);
it becomes a candidate row if and only if Linux is admitted as a supported
platform, and it must be measured before it becomes one.

**Network filesystems are not supported**, because the documented mount modes
put an advisory lock's exclusion outside the local kernel while still reporting
success. That exclusion is a property of the protocol rather than of the host
platform, so it holds whatever the supported set becomes.

Only the lock property can fail invisibly, and it costs duplicate compiler work
rather than correctness: complete identity, immutable final entries, validation
on every hit, and atomic publication do not rest on the filesystem, so every
filesystem failure resolves to a miss, a reported unavailability, an unpublished
result, or repeated work. The cache therefore states this contract and does not
refuse an unrecognized filesystem — a refusal would make an optional accelerator
a correctness dependency, would fail closed on every filesystem nobody
enumerated, and would still not detect the one case that motivates it. Every
locally decidable failure is already reported: a cross-filesystem rename as
`CrossesFilesystems`, an unsupported lock as an `AcquireLock` unavailability.

Access time is **not** used to order eviction. On both measured filesystems it is
maintained under a `relatime`-like predicate or not at all, so an immutable
entry's access time advances at most once — at its first read after publication —
which cannot distinguish an entry used on every build from one used once. See
[the supported-filesystem contract](research/cache/supported-filesystems.md).

## Loading and validation

Before execution, the runtime validates:

1. schema versions and parser resource limits;
2. manifest, metallib, and bundle hashes;
3. target/profile compatibility and semantic root-binding provider support;
4. root-binding declarations and the bound semantic environment;
5. semantic constraints;
6. plan graph, temporary lifetimes, and binding references;
7. symbol availability and compiler-established ABI consistency;
8. storage ranges, plan guards, routing, and launch limits.

For the selected plan, preparation also verifies that every residual semantic
validation obligation has a supported enforcement and that no logical result
or dependent public work can escape before its witness succeeds.

Manifest/schema/hash inspection does not require a device. Symbol existence and
optional pipeline reflection do. Manifest and MSL are generated from the same
verified typed binding table; tests may compare Metal reflection where
available.

Unknown required features fail closed. Compatibility rules for optional fields
and compiler/runtime version skew must be decided before the format is exposed
outside a lockstep release.

**Fact — the implemented reader is exactly that lockstep reader.** It supports only the versions and features this build writes: a major mismatch, a minor beyond what this build implements, an unrecognized digest algorithm, or a required feature it cannot supply is a typed rejection rather than a best-effort read. The optional-field and version-skew rules are consequently still undecided and still unimplemented, which is consistent with the sentence above rather than a gap beneath it. Of the numbered stages above, this profile discharges stage 1, the manifest and section half of stage 2, the binding-reference half of stage 6, and the static half of stage 8 — expression typing, availability phase, the guard's predicate type, and the launch formula's agreement with the entry's proven resource requirements. The rest need a device, a backend payload, or a bound runtime environment that this profile does not carry.

## Traceability

This document owns the neutral artifact envelope and Metal ABI profile. It does
not own backend scheduling or consumer storage. Its governing decisions and
supporting research are declared in frontmatter; unresolved serialization and
compatibility work remains explicit above.
