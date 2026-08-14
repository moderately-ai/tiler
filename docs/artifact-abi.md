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

The public accepted-but-not-stabilized compiler session constructs verified program portfolios and artifact-construction inputs through the ordinary bounded path. ADRs 0070 and 0071 assign authoritative target-neutral executable meaning and checked construction to shared `tiler-ir` representations; compiler-owned search and explain state remains separate from the artifact boundary.

**Fact — canonical envelope serialization, canonical form, and integrity validation are implemented, and the codec's *capability* is reachable.** `crates/tiler-artifact/src/program/codec/` encodes, decodes, and re-validates the envelope this document specifies, in the bounded lockstep profile recorded under "Implemented envelope profile" below. On Tom's review of 2026-07-25 that capability was promoted: `VerifiedArtifactProgram::encode`, `decode_artifact`, and the `DecodedArtifact` read view are `pub`, and the envelope, encoder, decoder, and section types stay `pub(crate)` behind the private module ADR 0074 convention 7 prescribes. So an out-of-crate consumer obtains bytes and a validated view over them, and does not obtain the codec's internal layout.

**Decided by Tom on 2026-07-25, and implemented: a decoded envelope is a *dispatch record*, never a reconstruction.** It publishes the named interface, each variant's guard and executable entries, each entry's bindings with the interface entry or internal allocation each addresses, each entry's launch contract and backend entry symbol, and each *carried* payload's compilation subject and exact object bytes — and it never rebuilds the shared-IR program it names. A consumer that needs that program holds the one it compiled, and the envelope binds the two by canonical artifact identity. The accurate statement is that a bounded lockstep codec with an accepted capability facade exists, not that the artifact format is stable.

**Fact — one strict-affine u4 dequantization reaches the complete target-neutral structural boundary.** A verified semantic value with ordered code, scale, and zero-point components lowers through a role-addressed schedule and structured kernel, packages as a verified kernel program, projects into the artifact interface and entry bindings, encodes under the neutral envelope, decodes into the public dispatch view, and re-encodes byte-identically. This is structural ABI and identity evidence. The fixture carries a descriptor-only `tiler.test.target-neutral` payload and is not safely runnable: before routing, structural binding, physical canonicality, an authoritative logical-view route, checker support and preparation, observability/coherence feasibility, and hard limits must be established; unresolved logical content conformance begins only after one-way route commitment at `EnforcementCommit`. A residual *operation* precondition — one an occurrence declares about an operand it was applied to — begins at that same boundary when present. No such runtime validation or enforcement is claimed here.

**Four decisions, distinct owners, and two sides of the routing commit.** The distinction is not editorial: moving unresolved content work before `RoutingCommit` forbids a still-legal fallback, while moving preparation or physical validity after it permits selection of a route that cannot check safely.

| subject | what it decides | when | owner |
| --- | --- | --- | --- |
| Logical-conformance preparation | whether the exact `ValueConformanceSubject` has an authoritative reconstructible view, a supported and prepared checker, an observable/coherent route, a hard-limit disposition, and one exact protected first consumer for this alternative | before `RoutingCommit`; a missing item makes this alternative infeasible | the route and selected alternative, using the logical-view and checker authorities |
| Physical representation validity | whether the *carrier* is well formed — including carrier kind and element width, alignment, bit order, and any unused tail bits under `PackedTailRule::Zero` | before `RoutingCommit`, separately | the physical representation owner; the logical scan cannot observe a tail bit and must not be asked to |
| Unresolved logical value conformance | whether the selected bound value implements the complete resolved type it declares — ordered component roles, component types, derived component shapes, parameter maps, inclusive code domain, and positive-**normal** scale domain | after `RoutingCommit`, beginning at `EnforcementCommit` and before the selected alternative's first real consumer or result work | the selected alternative's enforcement plan, driven from the authoritative logical view and the contract derived by `tiler_ir::semantic::check_bound_value` |
| Residual operation precondition | whether the operand of a particular `Quantize` or `Assemble` occurrence satisfies a predicate that occurrence declared | after one-way commitment, at `EnforcementCommit` | the enforcement plan the artifact selects |

The fourth row is the one that most often absorbs the third. A directly bound encoded input has **no** producing occurrence, so no operation precondition can speak about its bytes at all, and its scale domain is a property of the type it declares rather than of an operation that was never applied. Both a bound scale's unresolved conformance and an applied `Quantize` scale operand's residual begin after the commit, but they remain different subjects under different authorities.

**Contract — direct input conformance is planned per retained physical alternative, never once for the route.** The selected quantized profile retains two plans over the same direct `ValueConformanceSubject`. The fused plan binds success to the fused contraction stage that first reads decoded weight elements. The materializing plan binds success to the `Dequantize`/materialization stage that first reads the encoded components and produces the F32 temporary. A witness for one plan authorizes only that named first consumer; no route-global witness may authorize whichever stage happens to run.

Each alternative must prove hard feasibility before costs are compared: complete checker support, authoritative logical-view reconstruction, observability and coherence, hard-limit disposition, and exact evidence-to-first-consumer binding are mandatory. The fused and materializing enforcement costs remain separate; `Unknown` is never priced as zero, and an infeasible alternative cannot win because its omitted enforcement looks cheap. Preparation and all physical representation checks finish before `RoutingCommit`; unresolved content enforcement begins at `EnforcementCommit`; semantic failure is terminal with no result allocation, publication, dependent effect, or fallback; and `PublicationCommit` follows only successful evidence and execution.

**Fact — the current checker scans every logical element.** `check_bound_value` checks presented structure and the governed scan budget, then calls `scan_logical_elements` for every component, including full-domain U8 codes and zero points as well as F32 scales. **Inference — a future selected U8 route may proof-elide code and zero-point content reads, but only after the physical owner has established the U8 carrier and element width and logical-view construction has established the presented type and returned scalar kind.** Carrier and width negatives belong at representation/view construction; presented-type and scalar-kind negatives belong at the conformance boundary. The positive-normal F32 scale still requires content inspection. This inference adds no field to `PresentedComponent`, which currently carries role, resolved type, and shape only.

**Proposal — a future producer derives the plan record; it does not restate it.** The derived subject includes the value origin, stability/version and coherence, route and logical view, complete resolved type and shape, validator identity and revision, selected alternative, enforcement mechanism, exact first consumer, observability/publication boundary, hard-limit disposition, and separate enforcement cost. This contract neither chooses public fields nor pre-requires an artifact identity or schema step. The final producer-filled grammar must be audited for identity and version consequences when it exists; its actual encoding decides whether a version moves.

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

**Accepted compiler-to-assembler boundary — occurrence-bound selected physical provenance.** For every cover-region selection, `PlanAlternative::selected_physical_providers` yields a compiler-constructed borrowed view in canonical whole-occurrence-byte order. A neutral assembler may forward exactly four subjects from that view: the whole opaque canonical region-occurrence identity bytes, the whole opaque compiler-minted implementation-proposal identity bytes, the readable physical `ProviderIdentity`, and the closed proposal-kind code. Occurrence multiplicity is preserved: selecting one provider more than once yields more than one row. The assembler neither parses the canonical bytes nor reconstructs proposal identity from provider and kind.

This is a construction boundary, not a claim about the current manifest layout. The artifact codec and `VerifiedArtifactProgram` builder do not yet package these physical-selection rows; that downstream work owns their manifest membership, schema/version step, validation, and identity placement. The accepted projection exposes no proposal body, constructor, cost, rejected alternative, offered-provider environment, installation order, or selection-policy control, and only the compiler can construct or replace its checked occurrence-to-proposal association.

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

**Draft — live input-extent artifact envelope row, 2026-08-13, not yet accepted.** A compiled payload may consume a program-interface `AbiRoot::InputExtent` in address and loop arithmetic through the accepted structured-kernel `InputExtentParameter`. The envelope now carries that declaration as a per-entry operand row (`DecodedExtentOperand`: interface key, axis, unsigned type) after the backend entry key. Empty lists write nothing, so this envelope row did not itself step `tiler.artifact-program`; the retained-shape-environment landing later took the domain to `v17`. Metal emits the operand as a read-only `constant ulong&` in the next `[[buffer(N)]]` slot after the kernel's buffer table; the payload mapping's extra transports are that placement. The live *value* is frozen from the same `AbiFacts` used for range and launch evaluation and is excluded from artifact, payload, library, and pipeline identity; only the declaration is. `RoutedExtentParameter` is the routed spelling. The kernel/runtime spelling is accepted; this envelope row is a labelled draft: Tom accepts the exact included and excluded surface under [`accept-the-live-extent-artifact-envelope-row`](../tickets/accept-the-live-extent-artifact-envelope-row.md).

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

**Normative — the backend whose representation a payload carries validates that payload from its own bytes, and every backend owes it.** Backend-payload validity is the one stage in that list this layer cannot decide. What the envelope proves about a carried object is that it arrived as its producer wrote it: the section digest is a content address over that section's purpose and its exact bytes, and the descriptor's digest binds the payload's *compilation subject* into artifact identity. Neither says whether the bytes decode into something the named backend can execute — whether their framing and schema are ones it reads, whether they end where they say they do, whether they name the entry symbol the artifact maps, and whether the slots they address are the entry's own in the access modes it declares. The object is opaque here by construction and artifact identity excludes it entirely, as "Deliberate exclusions" below records, so an artifact built over a defective object and an artifact built over a sound one are *the same artifact*: each decodes, each re-validates, and each re-derives one canonical identity. `crates/tiler-runtime/tests/adapter_route/main.rs`'s `every_payload_defect_is_the_backends_refusal_and_the_artifact_layer_accepts_the_bytes` asserts exactly that across eight defects of those kinds, which is what makes this an obligation the artifact layer provably cannot take back rather than a division of labour someone chose. [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 8 is the record.

**Normative — the check runs before the routing commit, and its position is the route's to fix rather than each backend's.** It runs once per routed entry, in execution order, after the route has published that entry's carried object and before the first live-device question, so a payload refusal arrives while another complete variant may still be selected instead of as a terminal failure. [ADR 0051](decisions/0051-make-runtime-routing-commit-one-way.md) is why that position is not negotiable: after `RoutingCommit` no other variant may be routed and every remaining failure is a typed terminal execution error, so a payload first read when it is dispatched is a payload with nowhere left to fail safely. Both ends of the position are derived rather than conventional. The check consumes bytes and no device fact, so nothing about it may wait on a device answer and a defect a producer wrote into an object must never surface as a live-device or prepared-entry outcome. And leaving the moment to each backend would make "before the commit" a promise every implementation keeps its own way, whereas what a reader needs to be able to rely on is that the route asks and the backend answers. Read from the other side, a backend also declares its representation so that a host holding bytes of a representation it does not consume is refused rather than translated. What is normative here is what a backend owes and when it is asked; the surface through which a runtime asks is a reviewed draft that comes to Tom under [ADR 0075](decisions/0075-scope-public-boundary-approval-by-change-category.md), and this contract does not fix it.

**Fact — the record behind those two paragraphs, and what else it decides here.** [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) decides three obligations that touch this contract's subjects: the payload-validation obligation stated above; that `BackendKey` and `RepresentationKey` minting stays open with collision a producer-side responsibility rather than a registry; and that the `prepare` stage is optional under the exact condition the loader already implements rather than universal. It also recorded that the artifact layer's governed-key validator enforced length alone while the compiler's same-named `TargetProfileKey` enforced an alphabet, an asymmetry that record deliberately did not settle; [`reconcile-the-two-target-profile-key-grammars`](../tickets/reconcile-the-two-target-profile-key-grammars.md) has since settled it, and the two now admit the same alphabet while their byte bounds stay deliberately apart — see "Governed budgets" below for what this layer enforces and why the bounds differ. The record was accepted on 2026-07-31, and the first composed backend has since implemented item 8's payload-validation schedule: ADR 0090's status paragraph records that landing, on [`route-a-custom-backend-through-an-independently-selected-adapter`](../tickets/route-a-custom-backend-through-an-independently-selected-adapter.md), which carried an out-of-crate non-Metal adapter fixture.

## Implemented envelope profile

Everything in this section is a fact about the current `crates/tiler-artifact/src/program/codec/` implementation. It records what this build writes and reads. It does not widen the normative contract above, and where the two differ the difference is named rather than resolved by rewriting either side.

### Maturity of the implementation

**Fact — the codec's capability is accepted and its layout is not.** `codec` is a private `mod` of `tiler_artifact::program`. Promoted on Tom's review of 2026-07-25 are the capability and the read view alone: `VerifiedArtifactProgram::encode`, `decode_artifact`, `DecodedArtifact` with the borrowed projections `DecodedInput`, `DecodedOutput`, `DecodedComponent`, `DecodedVariant`, `DecodedDeferredPredicate`, `DecodedEntry`, `DecodedBinding`, `DecodedNumerical` and `DecodedExpr`, `BindingTarget`, `SectionView`, `SectionPurpose`, the payload vocabulary, and `ArtifactCodecFailure`. `ArtifactEnvelope`, the encoder, the decoder, the row types those projections borrow from, and the governed constants — including all seven of the envelope's domain separators — stay `pub(crate)`. Promoting any further item is named on ADR 0075's always-ask list and requires Tom's review before merge.

**What a consumer can do today.** Build a `VerifiedArtifactProgram` through `tiler_artifact::program` — itself a reviewed *draft* boundary rather than an accepted facade — read its canonical identity, encode it to bytes, decode bytes back into a fully validated `DecodedArtifact`, re-encode that decoded view, observe any typed codec rejection, and carry a backend payload. From the decoded view alone, and holding no program, registry, or producer code, it can also read the whole dispatch record: each named input and output's logical shape and canonical resolved-type encoding; its ordered components with semantic role, component shape and resolved type, storage scalar, kernel access type, and storage encoding; each variant's applicability guard, declared target profile, feasibility rule set, deferred predicates, and live-device route requirements; each deferred predicate's exact prepared entry and complete target-property requirement, including the governed query, required quantity, and directional relation; each route requirement's kind, and either its neutral dimension and minimum or its owning backend, governed key, version, and canonical payload; each executable entry's stage key, proven resource requirements, declared numerical realization, launch contract, and launch preconditions; each binding's target, transport kind, component role, storage scalar, kernel access type, storage encoding, address space, access mode, alignment, and accessible offset and extent; and, for a payload whose object was carried rather than left pending, its compilation subject, backend entry symbol, transport slots, and exact object bytes. Every carried expression among those evaluates against bound `AbiFacts` through the shared IR's own evaluator.

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

**Fact — the manifest opens with the versioned domain tag `tiler.artifact-envelope.manifest.v1\0` and its own `{major, minor}` schema**, then the four governed component schema versions — program, ABI expression, guard-and-routing, target-requirement — and then, in this order: the routing policy tag; the derived required-feature set; the three reached semantic subjects; the named inputs and outputs with logical shape, canonical logical resolved type, and ordered physical components; the selected capability providers; the backend payload descriptors; the shared ABI expression arena; the plan variants, each with its guard, declared target profile and feasibility rule set, deferred predicates, live-device route requirements, and executable entries; the section descriptors; and the **digest** of the artifact's canonical identity, unframed and fixed width, under `tiler.artifact-envelope.identity-digest.v1\0`. Each component row carries its optional semantic role and resolved component type, physical shape, storage scalar, kernel access type, and complete storage encoding. Each deferred-predicate row carries the exact entry it queries and the complete prepared-entry target requirement: property key, earliest availability phase, versioned provider identity, required quantity, and directional relation.

Each executable entry carries its stage subject, proven resource requirements, declared numerical realization, ABI bindings, launch contract, and backend entry.

**Fact — a selected capability provider row carries two independent revisions and no third version.** It is the provider's identity and its own output-affecting revision, the governed capability key, and that *capability's* output-affecting revision. [The operation-extension contract](operation-extensions.md) makes the two revisions independent — one provider may register several capabilities that move at different rates — so the capability revision is not derivable from the provider's, and an artifact that folded only the provider's would let a provider change what its lowering emits and produce a byte-identical identity. Both are received from the compiler and neither is re-derived here.

The row deliberately carries **no** capability-API or compiler version, and as of 2026-07-27 that is a settled decision rather than an open gap. The [operation-extension contract](operation-extensions.md#semantic-and-provider-identity) retires the requirement with its derivation: providers are statically linked, so a capability-API mismatch is a compile error rather than something an artifact could record; and a compiler version is discharged by the payload's compilation subject, which folds the exact compiled source, the flags, and the toolchain provenance into artifact identity. A field that existed for the first was once filled by narrowing the capability revision into it — a conflated value rather than a recorded one — and removing it was the fail-closed reading. Nothing needs to be added back.

Replacing that field moved the manifest schema to **4.0**. As with the earlier steps this is a **major** version rather than a minor one, and for a stronger reason than the layout arguments below: the field's *width* changed from two bytes to four, so a `3.0` reader would not merely misread a value — it would lose framing for every row after it. The identity domain moved with it, to `tiler.artifact-program.v3` and the selected-provider record's own to `tiler.artifact-program.provider.v2`, so a pre-existing artifact identity and a current one can never be compared as equal by accident.

**Fact — an ABI binding row carries the *offset* its accessible range starts at, beside the extent.** Both are references into the shared ABI expression arena, both are derived from the packaged program rather than restated by a producer, and both are folded into artifact identity by canonical position. They are derived differently, because the program states them differently: the extent is the program's own accessible-byte *expression*, replayed onto the artifact arena, while the program states where a byte window starts only as a constant, so the builder mints that constant as the offset's literal. The row keeps an expression reference rather than a plain number so a program that one day computes its window offset can carry that formula without a schema step.

Adding it moved the manifest schema to **5.0** and the identity domain to `tiler.artifact-program.v7`. Major again, and additive only in the sense that no field was removed: the offset is inserted ahead of the extent inside a fixed-width record, so a `4.0` reader would consume it *as* the extent and lose framing for every row after it. Every artifact's identity bytes move at this step, which is intended — a `v6` identity described a record with no way to state a placement, so an artifact carrying one is not the same subject as the artifact carrying its `v7` restatement.

**Fact — each entry's resource requirements and numerical realization carry every numerical dimension the bounded operation vocabulary can consume.** In addition to input and result subnormal treatment, contraction, and reassociation, both records carry operand permutation, signed-zero elimination, and separately provenanced NaN-absence and infinity-absence assumptions. The decoded `DecodedNumerical` view publishes the same complete record, so a consumer holding only bytes does not infer a compiler selection from a profile name or silently recover a strict default for a field the producer resolved differently.

**Fact — the entry's canonical arithmetic NaN payload stays thirty-two bits, zero-extended, at every arithmetic width.** `canonical_arithmetic_nan_bits` is a `u32` written big-endian, and a `bf16` entry carries `0x0000_7fc0` — the sixteen-bit `CANONICAL_BF16_ARITHMETIC_NAN_BITS` in the low half with zeros above it — where an `f32` entry carries the whole `0x7fc0_0000`. This is a projection of `NumericalRealization`'s own field and not a second decision: the shared IR fixes the reading there, its scheduled-region verifier refuses a `bf16` region declaring anything else, and the artifact's numerical facts copy the value verbatim. Widening the field or tagging it with a width was rejected on three grounds. It would make the artifact state a payload the record it projects cannot hold. It would move `ARTIFACT_DOMAIN`, the manifest schema, and every schedule and kernel identity in order to carry information no reader needs, since the arithmetic type that fixes how many of these bits are significant is already a total function of the region's scalar program. And zero-extension is injective across the two widths on its own terms, because `0x0000_7fc0` and `0x7fc0_0000` are distinct `u32` values, so no `bf16` entry can collide with an `f32` one here. Moving the field into the delivered-realization record was rejected for that record's own stated reason: `EntryRealization` carries the *behaviours* the two records cross-check, and a NaN bit pattern is a value rather than a behaviour — which is why the record excludes the profile key as well.

**Fact — the delivered-realization record is separate from the entry's numerical facts, and the two are cross-checked.** An executable entry's resource requirements and numerical realization state the eight dimensions the bounded operation vocabulary can consume, dtype-free. The artifact-wide delivered-realization record states, per compiler-produced scalar-arithmetic subject, all eleven governed dimensions plus the means and provenance by which each required one is honoured. The two overlap on eight dimensions and must agree: construction and decode reject a record whose resolution differs from any bound entry's own statement. Because an entry's realization carries no arithmetic type, the record additionally carries one explicit entry-to-subject association per packaged entry; the neutral artifact validates that the association is encoded and references an existing subject, and the compiler and `tiler-build` producer are what prove its semantic meaning. An entry with no association, a record naming a profile other than the artifact's single `TargetProfileRef`, a dangling obligation or evidence reference, and an unknown record-family tag each reject.

**Fact — an incoming fact-source provenance schema is dispatched before the body is read.** Decode admits only schema 3, the sole grammar this build implements. A newer schema, a never-implemented schema, and a retired schema each refuse as a distinct `RealizationCodecError` variant under the rule `unsupported-provenance-schema`; this generation lists no retired number because `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` was introduced at 3. The body is not interpreted, and the value is never reconstructed through `FactSourceProvenance::new`, which stamps the current schema and would silently normalize a foreign one. Current-schema bytes round-trip unchanged.

The record is **required**, on the terms this document already applies to the synchronization realization: there is no absent state, no optional path for decoded bytes, and no `Option` a reader has to rediscover. A builder that declared none is refused with `missing-delivered-realization` before any identity is derived.

An entry ordinal in the record's binding table is the flat **canonical** packaged-entry position — variants in routing priority order, and within each variant its entries in canonical stage-key order. A producer states the flat *declared* position instead, over (variant declaration rank, declared entry ordinal), and the builder remaps it once, exactly as it does for a deferred predicate's entry: a declared ordinal is the transient fact this envelope replaces everywhere else, and two artifacts differing only in it are one artifact.

Adding the record moved the manifest schema to **13.0** and the identity domain to `tiler.artifact-program.v15`. Major, and the asymmetry with the route-requirement family decides it: **every** artifact at this schema writes the record's framed run, so a `12.0` reader would consume its length prefix as the section-descriptor count and lose framing for the rest of the manifest, and no reader that predates the family can read any artifact carrying one — so a required-feature key beside it would mark nothing. The identity domain moves because two artifacts delivering one numerical contract by different **means** — one honouring a dimension exactly, the other only under a declared relaxation — are indistinguishable in `v14` bytes, a means being no part of any `v14` field. They are different artifacts to a consumer comparing generated output against a reference, so a cache holding one must miss for the other.

Adding those fields moved the manifest schema to **6.0** and the identity domain to `tiler.artifact-program.v8`. This is a major schema step because the fields occur inside each executable-entry record ahead of its bindings; a `5.0` reader would lose framing after the old numerical-record boundary. The identity domain moves because `v7` omitted facts that distinguish two executable contracts, so its subject is not interchangeable with the complete `v8` restatement.

**Fact — logical component ABI rows moved the manifest schema to 7.0 and artifact identity to `tiler.artifact-program.v9`.** Interface entries now carry the complete logical resolved-type encoding and ordered component rows, while each binding carries its semantic component role, physical storage scalar, kernel access type, and complete storage encoding. This is a major schema step because the new fields occur inside repeated interface and binding records; a 6.0 reader would lose framing rather than skip an additive tail. A v8 identity could neither distinguish two schemes projected onto different role sets nor distinguish physical encodings and access types that require different runtime bindings, so v8 and v9 subjects are intentionally incomparable.

**Fact — removing the invented barrier-count resource moved the manifest schema to 8.0 and artifact identity to `tiler.artifact-program.v10`.** A barrier-operation count is neither a target capacity nor a proof of synchronization support: legality depends jointly on the schedule synchronization point and phase, operation kind, participants and execution scope, visibility, fenced spaces, ordering, and convergence. The fixed executable-entry resource record therefore no longer carries the four-byte count. This is a major schema step because a 7.0 reader would consume the following resource or numerical bytes at the old offset and lose framing. An entry that performs no synchronization carries no synchronization requirement and needs no target fact; zero is vacuous rather than an asserted capability.

**Fact — the first nonzero synchronization path moved the manifest schema to 12.0 and artifact identity to `tiler.artifact-program.v14`.** The fixed executable-entry resource record now states the synchronization *realization* the entry's schedule requires, or states that it requires none. This is the successor the step above named, and the retired count is not what came back: the record carries the complete subject a target must realize, as one value, never a quantity.

That subject is the operation kind, the invocation scope that must arrive, the invocation scope across which its fenced effects publish, the two memory-domain fence flags, and the ordering — six fields behind one presence byte. A schedule owns the rest of the obligation and never ships it here: the synchronization point's identity, its phase boundary, its participant set, the visibility edges it discharges, and its convergence evidence are proven before a kernel exists and reach artifact identity through the kernel identity the entry's program section already folds. What travels in this record is what a *target* has to attest to.

This is a major manifest step because the field lands inside a record every entry writes, ahead of its numerical fields: an 11.0 reader would consume the presence byte as the input-subnormal tag and lose framing for everything after it. The identity domain moves because a `v13` entry could not state a synchronization obligation at all, so an entry performing a fenced staged handoff and an entry performing none were indistinguishable in these bytes.

**The recorded absence is the load-bearing half.** An entry requiring no realization writes `0x00` rather than nothing. Omitting it would leave "no requirement" recoverable from bytes that never stated it, so an entry that later gained one could share identity with the one that had none — the same defect the retired count had, reached from the other direction. Zero is still not a capability; what changed is that *silence* is no longer a state this record can be in.

There is deliberately **no** required-feature key beside it, and the asymmetry with the route-requirement family is the reason: a variant with zero route rows stays readable by a reader that predates them, so its key marks the artifacts that are not — whereas every artifact at this schema writes the presence byte, so no 11.0 reader can read any of them and a key would mark nothing.

| Wire field | Vocabulary | Who decides it |
| --- | --- | --- |
| Presence | `0x00` absent, `0x01` present; any other byte is `UnknownTag` | Derived from the region's cooperative tile; never producer-declared |
| Operation kind | control barrier, asynchronous copy, split-phase barrier, collective, atomic, inter-dispatch dependency | The schedule's synchronization point; only the control barrier is admitted |
| Arrival scope | subgroup, workgroup, device | Derived from the tile's participant set |
| Publication scope | subgroup, workgroup, device | Derived from the storage the discharged edges cross |
| Fenced domains | workgroup flag, device flag | Derived from those edges; the exact set, never a superset |
| Ordering | relaxed, acquire-release, sequentially consistent | Derived: a producer-to-consumer handoff is a release then an acquire |

Each of the three enumerated vocabularies is this crate's own forward-and-inverse tag pair pinned by an exhaustive round-trip test, exactly as the subnormal and permission tables are — the schedule identity and the artifact identity are different subjects, and a shared table would let one domain's step move the other's bytes.

**Fact — the target side moved with it.** A target profile declares whether it realizes one complete subject, and that declaration is folded into both the checked descriptor (`tiler.target-profile.descriptor.v10`) and the complete declaration (`tiler.target-profile.declaration.v11`); the governed feasibility vocabulary is now `tiler.feasibility.phased-capability-and-numerical-honourability.v6`, a vocabulary widening rather than a revision, because the rules now decide a subgroup-realization predicate `v5` could not express. Synchronization still frames itself unconditionally in the complete declaration; the subgroup family records silence as absence, so a profile that declares no subgroup row keeps the `v11` bytes it encoded before that family existed.

The match is one equality over the whole subject. A profile carrying facts about *neighbouring* realizations — the same barrier at subgroup scope, the same fence including device memory, the same operation under a stronger ordering — resolves the required subject as `Unknown`, however many of its dimensions those facts state. That is the composition this shape exists to refuse: each component is separately true of some machine, and their conjunction is a statement about none of them. A profile that has measured a negative declares the subject unrealizable, which is a typed rejection rather than an unknown; both refuse before executable-frontier admission, and neither is ever expressed as a cost.

A fact admissible only from a later phase is `Unknown`, never `Deferred`. Deferral means a runtime can obtain the value before routing commits, and a synchronization fact carries no query contract that could — no property vocabulary asks a device whether it orders a workgroup-scoped acquire-release fence over threadgroup memory. Deferring one would be a promise nothing can keep.

**Fact — exact prepared-entry requirements moved the manifest schema to 9.0, the target-requirement component schema to 2.0, and artifact identity to `tiler.artifact-program.v11`.** The old deferred row carried an executable predicate, availability phase, and selected compile-time provider but did not name which prepared entry must produce the observation or preserve the query's acquisition provider, threshold, and comparison direction as one checked subject. That representation could not distinguish two entries that expose different values for the same property key. The compiler now exports the exact program-entry ordinal with the whole `PreparedEntryTargetRequirement`; artifact construction mints the executable predicate from that requirement and validation checks its direction rather than trusting an assembler-authored formula. This is a major manifest step because the repeated deferred row changed shape, and the target-requirement component moves independently because its governed vocabulary changed.

**Fact — exact live-device route requirements moved the manifest schema to 10.0, the target-requirement component schema to 3.0, and artifact identity to `tiler.artifact-program.v12`.** A variant could previously state no precondition on the *device* at all: a deferred predicate names a prepared entry, and a target-profile reference is a producer's declaration rather than an executable check against the machine in front of it. The variant record now carries a counted run of route requirements ahead of its entries, so a 9.0 reader would consume that count as the entry count and lose framing for the rest of the manifest — a major step. The target-requirement component moves independently because its governed vocabulary changed rather than merely grew: a requirement is no longer only a prepared-entry quantity. The identity domain moves because a v11 subject cannot distinguish two otherwise identical artifacts that require different device capabilities, and those are not one artifact.

**Fact — one payload per delivery position moved the manifest schema to 11.0 and artifact identity to `tiler.artifact-program.v13`.** An executable entry was realized by exactly one backend payload, so an artifact could be built for exactly one consumer build target. It now names one payload per **delivery position** — the ordered slot a consumer's `#[cfg]`-resolved build target selects — which is what lets one selection produce one envelope carrying one payload per built artifact family, with one identity and no partial delivery. The entry record replaced its single fixed-width payload reference with a counted run of them, so a 10.0 reader would consume the count as the reference and then read the first reference as the entry key's length prefix, losing framing for the rest of the manifest: a major step. The identity domain moves because the count and the order are both folded, in every entry, so a one-family artifact, the two-family artifact carrying its object first, and the two-family artifact carrying it second are three artifacts rather than one subject spelled three ways.

A delivery position is deliberately **not** a plan alternative, a target profile, or a device property. Two positions are one compilation, one plan, one kernel program, and two compiled objects; the artifact layer carries no name for what a position *is*, because "macOS" and "iOS" are consumer-target vocabulary a target-neutral artifact must not hold. Which position a consumer resolves to is stated once, when bytes become a program, and there is no default: an artifact carrying several objects has no "the" payload, and taking the first would load the object built for another target — which `docs/research/apple-targets/artifact-compatibility.md` records as loading and dispatching without error.

Three obligations keep the shape honest, and each is proven at construction and re-proven from bytes. Every entry names at least one payload; every entry names the *same* number, because a consumer resolves one position for the whole artifact and an entry short of it would leave that consumer with no object for a stage its route must dispatch; and no payload is reached from two different positions, which would make the artifact carry fewer objects than the consumer targets it claims to have built for. The neutral layer cannot decide *which* target a payload was built for — that is a backend fact a producer holds — so it refuses the shape that makes the question unanswerable rather than answering it.

**Fact — naming which input tensor a region reads moved the scheduled region to `tiler.schedule.v3` and the structured kernel to `tiler.kernel.v5`.** `TensorRole::Input` classified a boundary tensor without saying which one, and `PointwiseF32Node::Input` named "the" input because a region could read only one, so two reads of two distinct program inputs were indistinguishable in both encodings. Both now carry a region-local `InputOrdinal`, written after the role's own tag byte and after the leaf's. Each lands *inside* a record that repeats — every access, every bounds proof, every buffer parameter, every expression node — so an earlier reader would consume the following field at the old offset and lose framing, which is a major step rather than an appended tag. Every region and kernel ever encoded maps to different bytes now, and that is the point: a cache or artifact holding a `tiler.schedule.v2` or `tiler.kernel.v4` identity must miss rather than match a region whose input binding the earlier vocabulary could not state.

Three domains deliberately do **not** move with them. `tiler.kernel-program.v6` folds the kernel identity by reference and its own record layout is unchanged, so a `v6` program identity built over a `v5` kernel can never be confused with one built over a `v4` kernel — the folded bytes carry their own stepped separator. The artifact domain and the neutral manifest schema encode neither role, and their binding table is keyed by interface *name* (`BindingTarget::ProgramInput`), which already distinguished several inputs before this step. This is the same reasoning that kept the KIR domains still at the v12 step, applied in the other direction.

**Fact — carrying the derived index-arithmetic requirement moved the structured kernel to `tiler.kernel.v7`, the artifact to `tiler.artifact-program.v16`, and the manifest schema to 16.0.** A verified schedule's `ResourceRequirements` gained a nominal `IndexArithmetic`, derived once from the region's own unsigned-64 coordinate space and never restated by a consumer. Both steps are required for framing *and* for meaning. For framing, the tag lands **inside** each fixed resource-requirement run, between the device-memory flag and the synchronization record, so a reader at the earlier version consumes it as the synchronization presence byte and loses framing for everything after it — which is why the manifest step is major rather than minor, a minor bump being admitted by `minor <= implemented`. For meaning, the envelope carries no KIR operations, so a consumer holding earlier bytes could not re-derive the requirement from anything present: two programs differing in what index arithmetic they need were one subject, and a cache holding an earlier identity must miss on the complete one rather than match it.

**The requirement is a derived requirement and mints no route row**, which is the [live-device route requirement](#live-device-route-requirements) family's own admission test applied rather than an exception to it: a row belongs only when the selected route consumes it *and* the verified program does not already state it, and every scheduled region states this one. A row restating it would be a second producer authority that could contradict the dispatch record about one KIR fact. The comparison is made directly against the bound device by the owning backend adapter — `tiler_metal::direct_requirement`, beside the existing direct checks on `local_memory_bytes` and each binding's accessible window — and `the_standard_metal_path_publishes_its_recorded_identities` asserts the published route-requirement population is **zero**.

**Measurement, on the standard Metal path's zero-additional-feature artifact.** Fixed content rises from **65,308 bytes to 65,313**, as manifest 41,113 to 41,116 and non-object sections 24,134 to 24,136. The five bytes are one tag written five times, located by byte-aligning the two envelopes: once in the single entry row's own resource record, and once inside each of the four kernel identities the envelope embeds, every one of which folds the structured kernel that gained the byte at `v7`. Five and not six is the part worth pinning — a sixth would mean one record encodes the requirement twice — and `FIXED_CONTENT_BYTES` holds it. The artifact identity preimage rises from 62,183 to 62,187, and the standard Metal path's artifact identity and expansion-cache subject are rebaselined in the commit that states this.

**Fact — carrying the retained shape environment moved the artifact to `tiler.artifact-program.v17` and the manifest schema to 17.0.** The semantic-subject run now writes the lossless fifth-subject projection — every symbol declaration and root binding, including source, availability phase, and provenance, then every semantic input constraint in canonical order — after admission provenance and before the interface population. Variant guards and solver-derived state stay out, exactly as they stay out of `ShapeEnvIdentity`. The carried bytes are the one authority for both artifact identity and decoded evaluation. A `16.0` reader would consume the new run's length prefix as the input count and lose framing for the rest of the manifest, so the step is major. Two fixed-interface programs that differ only by an unused environment were one `v16` subject; they are two artifacts at `v17`. Invocation values do not enter those bytes. The registry snapshot remains omitted under ADR 0072. The manifest domain, envelope format, canonical encoding profile, component schemas, stage/provider/payload key domains, and shared-IR identity domains do not move.

**Fact — the current identity ledger is source-derived and each step has one owner.** The resolved value type is `tiler.resolved-value-type.v3`, the scheduled region is `tiler.schedule.v5`, the structured kernel is `tiler.kernel.v7`, the verified kernel program is `tiler.kernel-program.v11`, the independently serialized artifact stage key is `tiler.artifact-program.stage.v3`, and the artifact is `tiler.artifact-program.v17`; the neutral manifest schema is 17.0. The numerical-contract key domains are `tiler.contract.f32.v2` and `tiler.contract.bf16.v1`, one per arithmetic type a contract may be stated in; the `bf16` domain was *added* beside the `f32` one rather than widening it, so every `f32` key is byte-identical and no pin moved at that step. The target feasibility profile's checked descriptor encoding is v10, the complete target declaration is v11, and the governed feasibility vocabulary is `tiler.feasibility.phased-capability-and-numerical-honourability.v6` revision 1. The checked descriptor and complete declaration stay at those domains because the subgroup family writes last and only when a row exists; silent profiles keep the bytes they encoded before the family existed. The envelope format and canonical encoding profile remain 1.0; the program, ABI-expression, and guard-and-routing component schemas remain 1.0, while the target-requirement component schema is 3.0; and the selected-provider key domain remains v2. The KIR domains do not move at the v12 or v13 step: a route requirement states what the *emitted payload* needs of a device, and a delivery position states which of several emitted objects a consumer's build target loads, and neither is a fact about the program's own executable meaning. The target-profile domains move because an observed capability fact and an executable later-phase query declaration are now different canonical subjects, and the feasibility vocabulary now carries the query path that makes a deferred quantitative predicate executable. The KIR domains do not move at those steps because the program's launch requirement is unchanged; artifact and manifest versions move separately because their exact-entry target-requirement record changed as described above.

**Fact — canonical semantic stage coverage moved the verified kernel program from `tiler.kernel-program.v6` to `v7`.** The coverage field remains one raw four-byte ordinal, but it now names the semantic graph's canonical traversal rather than builder storage order. Under the v6 separator, identical independent operations with fixed output names admit the same raw bytes while the ordinal denotes different operations across equivalent authoring orders; the new separator makes every prior program identity miss instead of reinterpreting those bytes. The nested artifact and envelope domains do not step: both fold the complete program identity with its separator, so their resulting identity values move without losing injectivity or changing their own grammar.

**Fact — carrying the published output order moved the verified kernel program from `tiler.kernel-program.v7` to `v8`.** A program's published outputs are its ordered output interface, and that order belongs to the unforgeable semantic subject the builder was opened against rather than to the producer: whole-program verification now proves the published records carry the subject's keys in the subject's declared order, each key's records contiguous, and within one key the records in the encoded contract's declared component order, refusing anything else as `misordered-named-output`. Identity therefore folds that list in declaration order instead of sorting the encoded records by content. Each record's own bytes and framing are unchanged, so a one-output program's payload does not move and only the separator distinguishes its `v7` and `v8` spellings; a program publishing several outputs in any order other than ascending record content encodes a different sequence, which is a reinterpretation of retained bytes rather than an append and is why the tag steps. The semantic graph has encoded its outputs in declaration order all along — `tiler.semantic-graph.v2` writes the output list unsorted and seeds its canonical value numbering from it — so this closes an asymmetry in which the artifact layer discarded a permutation the layer below it treats as identity. The nested artifact and envelope domains do not step: both fold the complete program identity with its separator, so their values move without losing injectivity or changing their own grammar, and the standard Metal path's artifact identity and expansion-cache subject are rebaselined in the commit that states this. *(**Superseded 2026-08-08 on the domain name alone**, by [`repair-the-records-the-sourced-semantic-shape-falsifies`](../tickets/repair-the-records-the-sourced-semantic-shape-falsifies.md). The sentence is dated beside rather than rewritten because it was true at this step and the property it asserts is unchanged: the semantic graph domain is now **`tiler.semantic-graph.v3`**, stepped on 2026-08-07 by [`carry-a-sourced-shape-on-semantic-values`](../tickets/carry-a-sourced-shape-on-semantic-values.md) so that every extent carries a source tag, and `v3` writes the output list unsorted exactly as `v2` did. The `bf16` measurement below already names `v3`, which is what made the two readings inconsistent; only this one was stale.)*

**Fact — the independently serialized artifact stage key moves from `tiler.artifact-program.stage.v1` to `v2` for the same coverage correction.** An entry row does not refer to its stage only through the nested kernel-program identity: its stage key separately writes the bound kernel identity followed by the raw coverage ordinals, is compared as a canonical key, and is serialized into the envelope. Its own separator therefore steps. `tiler.artifact-program.v14` and the envelope schema remain unchanged because they fold that complete stepped key with its separator; their identity values move, while their own record grammars remain injective.

**Fact — binding stage coverage to its refinement evidence moved the verified kernel program from `tiler.kernel-program.v8` to `v9`, and the artifact stage key from `tiler.artifact-program.stage.v2` to `v3`.** A covered occurrence is no longer a bare ordinal asserting a claim: it is a record naming the occurrence and, after it, the length-framed reached-only executable-coverage identity of the completed index-refinement receipt that proved it. Two programs alike in every structural respect but resting on different verified index realizations are two programs, and so are the artifacts that carry them. Unlike the four steps above this one is an added run inside a repeated record rather than a reinterpretation, so a `v8` reader handed a `v9` stage section would read the framed identity as the next occurrence; both separators therefore step, the artifact's independently because an entry writes that subject itself.

**Fact — folding the declared publishing-copy contracts moved the verified kernel program from `tiler.kernel-program.v9` to `v10`, and nothing below or beside it stepped.** A program that both publishes a value and feeds it to a later stage cannot express that with one write: `ValueRole` is exclusive, so the producing stage's owning write goes to the temporary its consumer reads across and a second dispatch writes the published value. That second dispatch computes no operation of the bound graph, so — exactly like a split reduction's final pass — it needs a declaration for whole-program verification to admit it, and `tiler_ir::program::PublishingCopy` is that declaration: which stage defines the copied value, which publishes it, and which two values the copy relates. None of it is derivable from the entities already folded, because a dispatch reading one value and writing another is the same stage, value, and edge set whether or not the program declares it to be a publication.

This is the `v6` precedent — a new program-scope declaration section, encoded unconditionally — and it is why the tag steps rather than the section being appended silently: a zero-copy program grows an eight-byte zero count, so every program ever encoded maps to different bytes and a cache or artifact holding a `v9` identity must miss rather than match. An *appended-only conditional* section, written only by programs that declare a copy and leaving every zero-copy program byte-identical, was rejected. It is injective today, because the section it would follow is length-framed, but it leaves the section's presence positionally ambiguous and constrains every future appended section to be read against a grammar whose shape depends on the content before it. Grammar determinacy is the cost that option saves, and it is the part that made the answer correct. The artifact stage key deliberately does **not** step with it, unlike at the v9 step: a publishing copy is a program-scope declaration and no entry writes that subject itself, so `tiler.artifact-program.stage.v3`, `tiler.artifact-program.v15`, and the manifest schema all hold, each framing the complete stepped program identity with its own separator. The standard Metal path's artifact identity and expansion-cache subject are rebaselined in the commit that states this.

**Fact — folding the declared staged-realization contracts moved the verified kernel program from `tiler.kernel-program.v10` to `v11`, and nothing below or beside it stepped.** A registered elementary family whose index-realization law realizes a region *sequence* is computed by several dispatches: the first folds and hands one value on, each later one reads that value and continues the same operation. Only the first claims the occurrence, because whole-program coverage is keyed on `SemanticOccurrence` and refuses one occurrence twice, so every later dispatch computes no operation the program's coverage names and needs a declaration for whole-program verification to admit it. `tiler_ir::program::StagedRealization` is that declaration: which stage hands the value on, which continues from it, which value crosses the dispatch boundary, and which occurrence the chain jointly realizes. It is a third declaration beside the split and the copy rather than a widening of either, because a split partitions a fold's contributors and carries a partition count, a copy publishes what it read and must therefore agree in extent, and a staged realization does neither — the shipped instance hands a `[2]` fold on to a `[2, 2]` pass, so an extent rule would refuse the very case it exists for. Neither the occurrence nor the chain is derivable from the entities already folded: a dispatch reading one value and writing another is the same stage, value, and edge set whether or not it is declared to continue an earlier one, and *which* occurrence it continues is recorded nowhere else, because coverage records only an occurrence's first stage.

The step takes the `v6` and `v10` reasoning unchanged: a fourth program-scope declaration section, encoded unconditionally, so a program with no staged chain grows a second eight-byte zero count and every program ever encoded maps to different bytes. The appended-only conditional alternative is rejected for the reason the paragraph above records, and now with more force rather than less — two adjacent optional sections would make the grammar's shape depend on the content before both of them. Nothing below or beside the program domain steps: a staged realization is a program-scope declaration and no artifact entry writes that subject itself, so `tiler.artifact-program.stage.v3`, `tiler.artifact-program.v15`, and manifest schema 14.0 all hold, each framing the complete stepped program identity with its own separator. **The manifest schema's non-step is a derivation rather than a preference.** A manifest carrying an artifact built over a `v11` program has the identical field widths, positions, and counts it has over a `v10` one — a variant record folds the program identity as one framed section and projects the program's interface, bindings, entries, execution order, and dependency edges, never its declaration sections — so a `14.0` reader frames such an artifact exactly as it frames any other and the reader admission rule `minor <= implemented` neither loses framing nor names the artifact wrongly. What moves is the content of a folded value, which is content moving through a fold rather than a grammar changing. The standard Metal path's artifact identity and expansion-cache subject are rebaselined in the commit that states this.

What is folded is the receipt's **reached-only** projection and never its complete identity. `IndexRefinementReceiptIdentity` restates the complete semantic, scalar, and law registry snapshots, which are compilation-request provenance; folding it would make an unused provider revision invalidate an otherwise identical executable artifact, which ADR 0072 forbids and this crate's `an_unused_semantic_provider_revision_does_not_change_identity` holds. `tiler.ir.index-refinement-executable-coverage.v2` is the projection that retains the selected subject — graph, canonical occurrence, numerical contract, verified region, selected law and provider, reached definitions and admission provenance, exact operand and result bindings, and every residual proof identity — and excludes the rest.

**Fact — the coverage projection names its graph by digest as of `v2`, and that step is what makes kernel-program identity linear rather than quadratic.** Under `v1` each record opened with the bound graph's whole framed `SemanticGraphIdentity`, and there is one record per semantic operation, so the product of a linear encoding with a linear count *was* the quadratic term: kernel-program identity measured exactly `134n² + 3650n + 727` bytes over the ordinary compilation path's domain, whose quadratic coefficient is the graph encoding's own per-operation slope. [ADR 0104](decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) replaces the restatement with a fixed-width governed digest of that identity under `tiler.ir.index-refinement-coverage-graph.v1`, and both coverage tags — the single-region one and its staged sibling — step to `v2`. **Measurement, last re-derived 2026-08-07, taken on the ordinary compilation path over the widened 2..=32 ladder: the curve is exactly `3525n + 727`, quadratic coefficient zero, residual zero at all thirty-one points. Twenty-two of those points were out-of-domain confirmations before they were rows — the eleventh when the explain-ceiling defect fell, then each of 12..=32 when the `region_expansions` truncation fell — every one compiling to the fitted line to the byte; all are now inside the domain, so the extrapolation beyond it has no remaining out-of-domain check. The path refuses this family above thirty-two operations with `BudgetExhausted` on `region_members`, a declared region-shape bound rather than a program-size one.** The 64 MiB program bound moves from 695 operations to 19,038, and twice the identity — the post-[ADR 0103](decisions/0103-declare-the-manifests-artifact-identity-by-digest.md) envelope multiplicity — crosses the 1 MiB per-invocation embedding ceiling between 148 and 149 operations rather than between 50 and 51. At the governed `semantic_operations` budget of 62 that is 219,277 bytes, 41.8% of the ceiling, where the `v1` encoding stood at 283%.

**Corrected 2026-08-07 by [`correct-the-records-the-derived-region-shape-budgets-falsify`](../tickets/correct-the-records-the-derived-region-shape-budgets-falsify.md) — the ladder above is a truncation of the admitted domain, the curve fitted to it has moved, and one of the three figures it carries stops being an extrapolation.** The paragraph's structural account of the `v1`→`v2` step is unaffected and is retained; what is retired is its measurement block and the three derived figures after it. **Fact —** `region_members` was the bare constant `32` when that ladder was measured, so `33..=62` refused as a *region* although every bound on the program's own *size* admitted them. [`derive-the-region-shape-budgets-from-the-declaration`](../tickets/derive-the-region-shape-budgets-from-the-declaration.md) replaced all three region-shape constants with values sized, at authoring time, against the governed profile's own declaration — `region_members` `62` against `semantic_operations`, `region_live_values` `80` against `semantic_values`, `region_boundary_outputs` `3` against the declared output count — while `DeterministicBudgets::governed` stayed a nullary `const fn` returning integer literals, so nothing is computed from a request's declaration at run time. `crates/tiler-compiler/tests/region_search_budget_coverage.rs` compiles every point of `33..=62` through the public `compile_governed` boundary as one whole-program region, and sixty-three refuses `BudgetExhausted` on `semantic_operations` before any target is consulted. **Measurement, re-run 2026-08-07 over the whole admitted domain — sixty-one points, 2..=62 operations, retained at `spikes/program-planning/identity-growth/results/2026-08-07-post-restored-planning-wall-apple-m4-max-macos27.0-26A5388g/growth.tsv`:** the curve is exactly `3530n + 723`, quadratic coefficient zero, residual zero at every point, with `graph_bytes(n) = 134n + 149` unmoved. `3525n + 727` no longer reproduces a single point — every `program_bytes` value is larger by exactly `5n − 4` under an index-refinement encoding step that landed between the two trees, and `(3530n + 723) − (5n − 4) = 3525n + 727` recovers the older ladder by subtraction. **The three derived figures, re-solved on the measured constants.** The 64 MiB program bound moves from 19,038 operations to **19,011**. The 1 MiB per-invocation embedding crossing **does not move**: at the post-[ADR 0103](decisions/0103-declare-the-manifests-artifact-identity-by-digest.md) multiplicity of two it still falls between 148 and 149 operations, `2 × (3530·148 + 723) = 1,046,326` and `2 × (3530·149 + 723) = 1,053,386`, which is the one conclusion the new coefficients could plausibly have flipped and did not. And at the governed `semantic_operations` budget of 62, identity is **219,583 bytes measured rather than 219,277 fitted** — 41.9% of the ceiling at that multiplicity rather than 41.8%, against the `v1` encoding's 283%. **What retires beside the numbers is the out-of-domain claim.** The eleventh point and each of 12..=32 confirmed `3525n + 727` and not this curve, so those confirmations expired with the encoding they were about; the ladder now covers every program size the path admits, so no further out-of-domain check is obtainable along this axis without moving `semantic_operations`. The quadratic-to-linear conclusion this paragraph exists to state is untouched: every run since ADR 0104 reads a quadratic coefficient of exactly zero.

**Corrected 2026-08-08 by [`re-date-the-six-identity-growth-fit-sites-one-displacement-behind`](../tickets/re-date-the-six-identity-growth-fit-sites-one-displacement-behind.md) — the correction above went stale in five days, and what is retired here is this document's practice of restating the coefficients rather than the coefficients themselves.** [`carry-a-sourced-shape-on-semantic-values`](../tickets/carry-a-sourced-shape-on-semantic-values.md) stepped the semantic graph domain to `tiler.semantic-graph.v3` on 2026-08-07, writing every extent through `SourcedShape::encode` with a source tag ahead of it, and the ladder re-run on 2026-08-08 measures every `program_bytes` value larger by exactly `n + 1`, of which `graph_bytes` alone accounts for `n`. So `3530n + 723` and `graph_bytes(n) = 134n + 149` above are statements about bases `cee4fe1a` and `25e76d5d`, and the three figures the block re-solved — 19,011 operations, the 148/149 crossing, and 219,583 bytes at the governed budget — belong to those trees with it. **This contract stops carrying the fit as a live value, and that is the correction.** Three spellings of one curve in four days, each written in the present tense, each pinned by no test, and each falsified by an encoding step no reader of this document would have been watching for — which is the reasoning [`replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin`](../tickets/replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin.md) applied below to the nine byte figures of the two `bf16` Measurement clauses, that replacing digits with newer digits rebuilds the defect one identity step later. The standing authority is [the identity-growth spike](../spikes/program-planning/identity-growth/README.md), whose [results index](../spikes/program-planning/identity-growth/results/README.md) records which compiler tree each retained ladder measured and the exact displacement between consecutive ones, so a reader who needs the current coefficients reads them where they are produced and the next displacement moves one file rather than six. **What this contract states instead, and every one of these survived both measured displacements unchanged.** The quadratic coefficient is exactly zero on every run since ADR 0104, which is the whole of what the `v1`→`v2` step is claimed here to buy. The 64 MiB program bound stays unreachable by orders of magnitude for the sizes this path admits: the fitted refusal point moved 19,038 → 19,011 → 19,006 operations across the two displacements, all of them against a governed `semantic_operations` budget of 62, and the spike records the surviving margin as ×371 in bytes against the roadmap's 51-operation decoder layer. The 1 MiB per-invocation embedding ceiling is still crossed between **148 and 149** operations at the post-[ADR 0103](decisions/0103-declare-the-manifests-artifact-identity-by-digest.md) multiplicity of two — the one conclusion new coefficients could plausibly have flipped, and which neither the `5n − 4` step nor the `n + 1` step moved — where the `v1` encoding crossed it between 50 and 51. And identity at the governed budget stays well under half that ceiling at the same multiplicity, against the `v1` encoding's 283% of it. **Where a coefficient is unavoidable it is dated to its tree rather than stated in the present tense**, the treatment "The lengths behind that argument" below already gives the kernel identity lengths: the run this correction is written from is retained at `spikes/program-planning/identity-growth/results/2026-08-08-post-sourced-semantic-shape-apple-m4-max-macos27.0-26A5388g/growth.tsv`, taken at base `cc667626` on an Apple M4 Max under macOS 27.0 build `26A5388g` and the repository toolchain pin, and reads `3531n + 724` with residual zero at all sixty-one points and a widest measured point of 219,646 bytes at 62 operations, 0.327% of the program bound. That is a reading of one tree and this document does not track it; the spike does.

The record's meaning is unchanged, which is what makes the digest admissible at all: it still says "this occurrence of *this* graph", still refuses two records naming one occurrence ordinal in different graphs, and is still a well-defined standalone value. What it stops doing is carrying bytes the graph identity could be reconstructed from — and nothing reconstructs them: the type has no decoder, no field accessors, and two `compile_fail` doctests holding that it has no byte constructor. The digest is written unframed because it is fixed width, and a length prefix exists to make a variable-length run self-delimiting.

**Nothing above the coverage domain steps, and that is a derivation rather than a preference.** `tiler.kernel-program.v11`, `tiler.kernel-program.stage.v2`, `tiler.artifact-program.stage.v3`, `tiler.artifact-program.v15`, and manifest schema 15.0 each fold the coverage identity with `push_slice` and re-derive no subset of it, so the complete stepped key arrives length-framed with its own separator and no identity taken over a `v1` fold can equal one taken over a `v2` fold. This is the shape of the `tiler.schedule.v4` and `tiler.contract.f32.v2` steps, where content below a fold moved and no domain above it stepped — not the shape of `tiler.kernel-program.v8` to `v9`, which changed the coverage record's own grammar here and had to step with it. Every identity *value* moves for every program ever encoded, which is the cache miss the coverage step exists to produce. The standard Metal path's artifact identity and expansion-cache subject are rebaselined in the commit that states this.

`tiler.artifact-program.v14` and the envelope schema again do not step, by the same per-tag injectivity the coverage correction above records: `push_variant` writes each entry's stage subject with `push_slice`, so the complete stepped key including its own separator arrives length-framed and no `v14` encoding of one artifact can equal a `v14` encoding of another across the change. Their identity *values* move for every artifact ever minted, which is the cache miss a step exists to produce. The standard Metal path's artifact identity and expansion-cache subject are rebaselined in the commit that states this.

Four domains move **together** at the synchronization step, and the shared cause is one field in one record. `tiler.kernel.v5` steps to `v6` and `tiler.artifact-program.v13` to `v14` because both encode the resource record the synchronization realization joined; the manifest steps to 12.0 with the artifact because the same field reframes the entry row; and both target-profile domains step because a profile now declares realizations. The scheduled region deliberately did **not** move with them: the synchronization point lives inside the appended `0x35` cooperative-topology payload, and at that step no cooperative region had ever reached a retained identity — the structured-kernel verifier refused every one before a kernel, program, artifact, or cache entry could hold it. **That premise has since expired**, and what replaced it is the Inference marked *at the `tiler.schedule.v4` step* below; the `v5` block between the two was inserted later, so the replacement is no longer the next paragraph. `tiler.kernel-program.v6` does not move either, for the reason it did not at the v13 step: it folds the kernel identity by reference and its own record layout is unchanged, so a `v6` program identity built over a `v6` kernel can never be confused with one built over a `v5` kernel. These values describe different subjects and must not be collapsed into one global artifact version.

**Fact — composing the numerical contract stepped the contract key domain to `tiler.contract.f32.v2`, and no encoding above it moved.** A caller used to name one of four numerical contracts, so a contract key was one of four hand-written strings (`tiler.strict-f32.v1` and siblings). A caller now resolves the contract's dimensions directly, and the key is the canonical, injective encoding of that dimension vector — its scheme, not its value, is what this domain versions. The step is required because the two schemes describe different vocabularies: a `v1` key named a preset from a closed list, a `v2` key spells a vector, and nothing that holds one may match the other.

**Fact — admitting a second arithmetic type added a domain and stepped nothing.** `tiler.contract.bf16.v1` renders a `bf16` contract's dimension vector under its own domain, with the `bf16` arithmetic tag and that width's sixteen-bit canonical NaN payload as its header. It is a sibling rather than a widening, and the distinction is what makes the change appends-only: the `f32` domain string, header, and dimension rows are untouched, the two parsers each refuse the other's rendering, and the strict-`f32` key is pinned as a literal in `crates/tiler-ir/src/schedule/numerics.rs` so a change to the dimension writer the two now share fails there rather than silently restating every artifact and cache identity minted under it. Nothing above the contract key moved, because no existing key's bytes changed. The domain opens at `v1` because a version counts its own domain's rendering revisions, and this is that domain's first.

**Every encoding above it is unchanged, and every identity derived through one moves anyway.** The scheduled region writes the key length-framed beside the realization fields it names, exactly as it did; the structured kernel and the artifact fold those bytes by reference. So the scheduled region — `tiler.schedule.v4` when that step landed, and `v5` since — together with `tiler.kernel.v6`, `tiler.kernel-program.v6` at that step (`v9` now), `tiler.artifact-program.v14`, and manifest 12.0 all stayed where they were **at that step** — no record layout changed and no reader lost framing — while every region, kernel, program, artifact, and expansion-cache subject ever minted under a `v1` key maps to different bytes. That is the intended consequence: a cache entry published against a contract vocabulary that no longer exists must miss rather than match. `docs/numerical-semantics.md` owns the scheme itself, including what the encoding covers and why injectivity is load-bearing rather than tidy.

**Fact — admitting a loop-carried cooperative tile moved the scheduled region to `tiler.schedule.v4`, and no domain above it stepped.** A `CooperativeTile` carries the number of times its phase sequence executes, so a tile can state that its staging is rewritten between rounds; the field lands inside the `0x35` topology payload, ahead of the staging and phase records it scopes, and every cooperative region's bytes moved with it. A region that stages nothing was untouched except for the eighteen separator bytes, which a retained pair of pinned identities checks byte for byte rather than asserts.

**Fact — widening the cooperative staging relation to two dimensions moved the scheduled region to `tiler.schedule.v5`, and no domain above it stepped.** [ADR 0097](decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md) admits a staged access over a stated participant *space* rather than a contiguous participant range: a `CooperativeTile`'s participants occupy per-dimension extents, slowest-varying first, and a `StagedSpan` carries one stride per participant dimension, so the participant at coordinate `(l_0, .., l_{r-1})` addresses `count` contiguous slots at `offset + sum_d strides[d] * l_d`. That is what makes a blocked operand tile's transposed staged write statable, and it is the relation whose absence — not the round vocabulary, and not the barrier — is what kept a tiled contraction unexpressible.

**Inference — an append was not available, and this time the encoding says so on its own terms.** A stride vector is not a field added beside a stride; it is a different relation in the same position, so there is no position to append to. Both the participant space and the staged span replace an unframed fixed-width run with a length-framed one — the rank leads through the workspace's one length prefix, exactly `rank` eight-byte elements follow, and the span's `offset` and `count` sit at positions the frame determines — and both land inside records that repeat, the coordinates of every tile and every staged write and read of every phase. So every cooperative region's bytes move and no earlier reader keeps framing. Injectivity holds in both directions, which a fixed-rank inline array makes a thing to state rather than assume: two spans differing in rank, strides, offset, or count differ in these bytes, and two spans *equal* in meaning encode identically, because the array's unused tail never reaches the encoding at all. **"This time" contrasts with the `tiler.schedule.v4` step, whose own append argument sits below this block rather than above it.** That argument is the Inference marked *at the `tiler.schedule.v4` step* further down: the `v5` paragraphs here were inserted ahead of two `v4`-step records rather than appended after them, so this chronology reads forward while the page order does not.

**Fact — at the `v5` step every Metal golden's identity moved, including the five that carry no cooperative tile.** `pointwise_scale_bias`, `reduction_single_axis`, `reduction_multi_axis`, `reduction_fused_multiply_add`, and `contraction_strict_tensor` never reach the `0x35` topology payload; their entry symbol, kernel identity digest, and scheduled-region identity digest move for the eighteen separator bytes alone, through the fold. Only `cooperative_workgroup_reduction` stages. The standard Metal path's artifact identity and expansion-cache subject move for the same reason, and both are rebaselined in the commit that states this. That is what a domain separator costs, and it is the intended consequence rather than a regression: every cache entry, artifact, and golden minted under `v4` misses rather than matches.

**Dated 2026-08-08 by [`date-the-artifact-abis-metal-golden-enumeration-to-its-step`](../tickets/date-the-artifact-abis-metal-golden-enumeration-to-its-step.md) — those six names were the whole golden corpus when this step landed, and a reader counting the directory today finds more.** The paragraph is dated beside rather than recounted because it was exactly complete as written: `git ls-tree --name-only a395852a crates/tiler-metal/goldens/` returns `cooperative_workgroup_reduction` and the five named above and nothing else, and the four remaining fixtures arrived afterwards — `pointwise_scale_bias_bf16` at `7a24ed20` on 2026-08-05, then `structural_mirrored_reindex` and `structural_widening_broadcast` at `5f81857c` and `elementary_silu_activation` at `08e6fb35`, both on 2026-08-06. **Restating the sentence over today's directory would be a fresh false claim rather than a refresh**, because it would assert that four goldens' identities moved at a step predating their existence. A domain separator moves the identities that exist when it steps, so the corpus is a fact about one tree here and never a standing quantity this paragraph can carry. **What the corpus is now is stated as its construction rather than as a number**, for the reason the identity-growth correction above names its spike instead of restating coefficients: `GOLDENS` in `crates/tiler-metal/src/golden_compilation.rs` is the table every checked-in fixture is compiled through, and `every_checked_in_golden_is_compiled_by_this_module` reads the `goldens/` directory and requires the sorted `.metal` file names to equal the sorted table names. That is what makes this population unable to shrink quietly, and each way it says *no* is a different mechanism: a fixture added to the directory and not to the table fails that equality by name, a fixture deleted from the directory fails the table's `include_str!` before the test runs, and an entry dropped from the table fails its declared array length. A reader who needs today's count reads it where it is produced.

**Inference, at the `tiler.schedule.v4` step — the append that avoided the earlier steps was no longer available, and the reason generalizes.** Both the `0x35` topology tag and the `arrival` byte after it were justified by "no cooperative region has ever reached a retained identity". The single-workgroup tree strategy expired that: a cooperative region lowers to a verified kernel, emits a checked-in Metal golden, and folds into an artifact identity and an expansion-cache subject. Once bytes are retained, the question stops being *does anything hold the old bytes* and becomes *can an old identity equal a new one* — and adding eight bytes anywhere inside that arm does not answer it. The arm ends in a length-prefixed axis list whose elements are four bytes each, so an old region with axes `[0, 1, 2]` encodes exactly the bytes a new region with axes `[2]` and three rounds does. Only the verifier's requirement that a topology's axes repeat its access's separates them, and an identity encoder that leans on a verifier invariant has stopped being injective on its own terms. A domain separator restores injectivity by construction, and it costs the retained corpus a miss rather than a wrong hit.

Also at the `tiler.schedule.v4` step: the kernel, kernel-program, artifact, and manifest domains deliberately do **not** move with it, for the reason `tiler.kernel-program.v6` did not move at the synchronization step: each folds the identity bytes below it *whole*, separator included, rather than re-deriving a subset of them, so a `v6` kernel identity built over a `v4` region can never be confused with one built over a `v3` region. Their recorded values move — the standard Metal path's artifact identity and cache subject are both rebaselined in the commit that states this — and that is content moving through a fold, not a grammar changing.

**Dated 2026-08-08 by [`date-the-two-v4-step-paragraphs-trailing-the-v5-block`](../tickets/date-the-two-v4-step-paragraphs-trailing-the-v5-block.md) — the two paragraphs above record the `tiler.schedule.v4` step and were written directly beneath it, and the `v5` block was inserted between them and it afterwards.** `git show e4d2aa7d:docs/artifact-abi.md` shows all three `v4` paragraphs adjacent, and `git show a395852a -- docs/artifact-abi.md` shows the `v5` Fact, its Inference, and the golden enumeration added *above* this pair rather than after it. Each therefore names its own step in its opening clause, which is a repair to position and not to content: both were true as written and neither is rewritten here. **Nothing is reordered**, because `git log -S` locates both at `e4d2aa7d` by text that stays byte-identical through this note, and moving them would put that text at a position the search no longer resolves to its own step. **The `v6`/`v4`/`v3` triple above is correct as written and must not be renumbered to `v5`/`v4`**: it is the `v4` step's own argument that a kernel domain folding the region identity whole cannot confuse a `v4` region with the `v3` one it replaced, so restating it at `v5` would record a `v4`-step decision as a `v5`-step one — the substitution the golden enumeration above refused, for the same reason it refused it.

**Fact — every variable-length run carries a fixed-width `u64` length before its content**, so no concatenation of fields is ambiguous, and **every encoded enumeration is written through the one governed tag table its vocabulary owns**, never through a Rust discriminant, so inserting a variant cannot silently renumber a value already on disk. Each table is a forward and inverse pair kept in one place and pinned by an exhaustive round-trip test.

**Fact — the carrier and access-type tables carry `bf16`, appended, and neither version moved.** `StorageScalar::Bf16` is `0x03` in the physical-carrier table and `KernelType::Bf16` is `0x06` in the kernel access-type table; every earlier value is unchanged. Neither vocabulary is `#[non_exhaustive]`, so widening either stops the build at the tag table that has to decide the new value rather than compiling into a run-time rejection. `ARTIFACT_DOMAIN` and the manifest schema both hold: a carrier is one tag byte inside a fixed-width row, so no artifact the earlier vocabulary could encode maps to different bytes, and no such artifact carries either new value. **Measurement, and it is now pinned by a test rather than carried as prose here.** The single-variant fixture artifact encodes to equal lengths at `f32` and at `bf16`, differing at exactly **68** byte positions: the thirty-two-byte manifest digest in the framing header, the thirty-two-byte identity digest the manifest ends with, the interface component's carrier and access tags, and the binding row's pair. It was **40** at manifest schema `14.0`, where the trailing identity *preimage* restated both tag pairs in four bytes instead of being covered by a digest; the `15.0` step traded those four for thirty-two. It then read **67** for the span between `tiler.semantic-graph.v3` and `tiler.artifact-program.v16`, and returning to 68 is coincidence rather than a revert: the four tag positions and the sixty-four digest positions never moved, and what changed each time is only how many digest bytes happen to coincide. Read this number as a measurement of a digest and never as the arithmetic. `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` asserts the count against `DIFFERING_CARRIER_POSITIONS`, so this number can no longer go stale here without a test failing.

**Fact — a decoder that has not assigned a carrier or access tag refuses it before the width it names is used.** The refusal is `UnknownTag` naming the subject and the rejected byte, raised at the tag reader, so the width is never used to frame a row or address a buffer. That ordering is the whole point of refusing: a two-byte carrier read as four addresses twice the bytes the interface provides, and every framing, digest, and identity check passes on the way there. `BindingAccessTypeMismatch` covers the same misread from the other side, refusing a binding whose access type is not the one its carrier stores, with each carrier's pairing named rather than defaulted.

**Fact — the carrier reaches artifact identity, and a producer emits a `bf16` artifact through the ordinary path.** Two artifacts differing only in one interface component's carrier and the binding that addresses it are two artifacts. **Measurement, of the carrier-only forged pair.** That fixture's two canonical identities are equal in length and differ at exactly four byte positions — the component's carrier and access tags, separated by its storage-encoding byte, and the binding row's adjacent pair. A cache that confused them would hand a consumer a kernel addressing twice the bytes it was given. **Corrected 2026-08-06, twice in one day, and the second correction is the current state.** [`admit-a-bf16-index-realization-law-and-refinement-contract`](../tickets/admit-a-bf16-index-realization-law-and-refinement-contract.md) took the refinement wall down — `NumericalContractIdentity` gained a `bf16` route and three `bf16` index-realization laws joined the nine existing rows — and [`carry-the-pure-bf16-producer-path-into-artifact-packaging-evidence`](../tickets/carry-the-pure-bf16-producer-path-into-artifact-packaging-evidence.md) then carried a pure-BF16 program from semantic construction through verified coverage, a `VerifiedKernelProgram`, and a `VerifiedArtifactProgram` that encodes, decodes, re-derives its identity, and re-encodes byte-identically. **Measurement, of the producer-path pair, and the inequalities it evidences are now pinned by tests rather than carried as prose here.** The produced BF16 artifact and its F32 twin from the same parameterized construction are separately derived artifacts of different lengths, so the forged pair's positional byte difference is not defined for them. What each is asserted through is the builder: `a_producer_built_bf16_artifact_round_trips_and_re_derives_its_identity` pins that the pair's two encodings are unequal in **length**, a producer-path width change being structural rather than the tag swap the forged pair is, and `the_bf16_artifact_and_its_f32_twin_are_two_artifacts` (`crates/tiler-artifact/src/program/tests.rs`) pins that their two **identities** differ, which is what makes two widths two artifacts. The direct-assembly `bf16` cases remain the right tool for the unknown-tag and mismatched-binding refusals, which need envelopes no correct producer would emit.

**Corrected 2026-08-08 by [`replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin`](../tickets/replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin.md) — both Measurement clauses above carried absolute byte figures that no test asserted, and the `v15 -> v16` step had falsified every one of them.** The forged clause stated the identity's length and the four exact offsets of its differing bytes; the producer clause stated four artifact and identity lengths. Nine figures, no pin behind any of them, and nothing went red when the derived index-arithmetic requirement moved the artifact to `tiler.artifact-program.v16`. They are retired rather than refreshed, which is the [dtype ledger's](dtype-support.md) conclusion on the overlapping four and the reason it is repeated here rather than re-decided: a number stated in prose and pinned nowhere decays without any gate seeing it, so replacing the digits with newer digits rebuilds the defect one identity step later. **The direction is why this was worth a p1 rather than a refresh.** The envelope did not grow — it **shrank**, against every reader's instinct that an envelope only accretes — so two of the four retired offsets addressed positions past the end of the identity they indexed, and a reader following them landed nowhere at all. An offset is the worst of these to carry: a position inside an identity preimage moves whenever anything ahead of it changes width, and it says nothing the structural description above does not.

**What is left unpinned, stated rather than left for a reader to assume.** The forged pair's *equal identity length* and its *count of four differing positions* survive above as properties, and no test asserts either. Both belong in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity`, which already derives both identities and already pins the **envelope's** differing-position count against `DIFFERING_CARRIER_POSITIONS`: the identity's count belongs beside it as its own constant, and the length equality belongs as the assertion that makes a positional comparison between the two well defined in the first place. The two counts are not one subject and must not be collapsed — the envelope's is a measurement of two digests and can move by coincidence, as that constant's own comment records, while the identity's is the two tag pairs and nothing else. Until that lands, this paragraph's four is a measurement and not a guarantee.

**Corrected 2026-08-10 by [`retire-the-false-unasserted-identity-difference-prose-after-the-pin`](../tickets/retire-the-false-unasserted-identity-difference-prose-after-the-pin.md) — the preceding paragraph is retained as history of a gap that [`pin-the-differing-identity-positions-beside-the-carrier-positions-constant`](../tickets/pin-the-differing-identity-positions-beside-the-carrier-positions-constant.md) closed, not as a live absence claim; the forged-pair Measurement above is now a tested property.** `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` first asserts that the two canonical identity byte runs are equal in length, the precondition that makes their positional comparison total, then asserts the differing-position population against `DIFFERING_IDENTITY_POSITIONS`. That identity constant remains a separate subject from the envelope's `DIFFERING_CARRIER_POSITIONS`: the former pins where the carrier reaches the identity preimage, while the latter measures two digests and may move when digest bytes coincide. Neither assertion pins an absolute identity length or any byte offset.

**Fact — a well-formed but non-canonical encoding is refused rather than normalized.** Named checks reject an out-of-order or repeated feature, interface key, provider, payload, expression, deferred predicate, launch precondition, executable entry, or section. Because this reader understands every field, the decoder then re-encodes the validated envelope and requires byte equality, rejecting any residual non-canonical spelling as `NonCanonicalManifest`. One artifact therefore has exactly one byte identity.

### What is meaning and what is canonical

**Fact.** Variant order is routing priority, named-interface order is the semantic interface's, and ABI binding order is the kernel signature's; all three are retained. Provider, payload, deferred-predicate, launch-precondition, executable-entry, expression-arena, and section order are replaced by the canonical content order artifact identity already uses. The arena's canonical order is the unique topological order that always emits the smallest available node under the shared IR's `compare_expr_nodes` — a total, content-derived order over expression *structure*, needing no numbering and exactly injective. Launch-precondition order is that same relation, reached through the same function identity folds, so the order a producer stores and the order a decoder re-proves are one definition rather than two that agree by inspection.

**Measurement.** Declaring the same payloads and providers in reversed order produces byte-identical envelopes, as does minting the same formulas through two different arena assembly orders.

**Fact — replacing the arena's order relation moved the manifest schema to 14.0 and moved no identity at all.** The order was previously derived from a table of canonical content keys, one per arena node. That is a different relation and not a different implementation of one: `expr_key` frames each operand's whole key behind an eight-byte length prefix, so comparing two keys compares operand *lengths* before operand content, while the comparator compares structure directly. An arena carrying an input-extent root with a long key and a target-property root with a short one orders the two nodes that read them oppositely under the two relations, so it encodes to different bytes at `13.0` and at `14.0`.

This is the first manifest step whose framing is untouched — every field keeps its width and its position — and it is **major** anyway, because a manifest schema names one canonical byte spelling of an artifact and this changes which spelling that is. A `13.0` reader, admitted at `minor <= implemented`, would refuse a `14.0` artifact as a non-canonical spelling: a rejection naming the wrong thing about it.

**No identity domain moves with it, and that asymmetry is the point.** `encode_identity` writes the arena exactly once in a canonical *numbering* derived from the use sites and the DAG beneath them, never from arena position, and it already ordered both expression-bearing sets with the comparator. So the artifact identity of every artifact is invariant to arena permutation and did not move here; `tiler.artifact-program.v15` and the standard Metal path's pinned artifact identity and expansion-cache subject all hold unchanged. This is the reverse of the usual step — the wire moved and the subject did not.

**Fact — replacing the manifest's trailing identity preimage with its digest moved the manifest schema to 15.0, and moved no identity at all.** The manifest ended with the artifact's canonical identity in full. It now ends with a thirty-two-byte digest of that identity under its own governed digest domain, `tiler.artifact-envelope.identity-digest.v1`, written unframed exactly as the header's manifest digest and each section descriptor's content digest are. **Measurement, on the zero-object hot-path fixture at `eee734cf`:** fixed content falls from **114,059 bytes to 57,978 — a 56,081-byte reduction, 49.17%** — of which 56,105 is the identity run and 8 its length prefix, against the 32 bytes that replace them. The manifest itself falls from 88,069 to 31,988. Nothing else moves: the `KernelProgramSubject` section stays 22,911 bytes, `BackendPayloadMetadata` stays 2,974, and the derived identity is still 56,105 bytes.

The step is **major** and is the first to need both of the reasons the steps above give separately. A `15.0` reader admitted at `minor <= implemented` would otherwise go on accepting a `14.0` manifest and read that manifest's eight-byte length prefix as the head of a digest, refusing as `TrailingManifestBytes` an artifact that is well formed at its own schema — a rejection naming the wrong thing about it, which is the `14.0` step's own argument. And a manifest schema names one canonical byte spelling of an artifact, which this changes for **every** artifact rather than for the arenas the `14.0` step reached.

**No identity domain moves with it, and it is the `14.0` asymmetry again.** `encode_identity` reads the envelope and never the manifest, so the trailing run was always a pure function of the content above it. `tiler.artifact-program.v15`, the expansion-cache subject derived from it, and the standard Metal path's two pinned values are unchanged — **verified rather than argued**: the whole workspace suite passes at this step with no pin recomputed. The wire moved and the subject did not, which is the reverse of the usual step and the same shape the `14.0` step had — except that there the wire was merely *permitted* to move and here it moves for every artifact.

**What the run buys survives, and what it cost does not.** The run was never read as an identity: `DecodedArtifact::identity` returns the decoder's own re-derivation and documents that it never reads the carried copy, so no consumer in the workspace read the preimage. What the run buys is a *declaration* check, firing when a producer's two derivations of one artifact disagree — and a digest of the derived identity refuses the identical set of disagreements in thirty-two bytes, so `ArtifactIdentityMismatch` keeps its exact meaning and no sibling rejection is warranted. What is given up is that a reader holding only the wire can no longer lift the identity without running the derivation. **The ADR 0074 convention-2 objection is answered rather than waived:** a canonical identity is opaque bytes a receiving crate never re-derives locally, and a digest standing where canonical bytes stood needs an argument that the site is a fold input rather than an identity a consumer compares. At this site it is — the run is compared by the crate that is the *authority* for it, against bytes that same crate derives in the same call. It is a producer's declaration to its own decoder, not an identity crossing a boundary. Recorded in [ADR 0103](decisions/0103-declare-the-manifests-artifact-identity-by-digest.md) and measured in [the manifest-growth research note](research/artifacts/manifest-fixed-content-growth.md).

**The reason for the `14.0` change is a bound rather than a preference.** A content key names its node's whole subtree, so a table of them over an arena of `d` chained nodes costs bytes quadratic in `d`, and `parse_expressions` built that table out of manifest bytes before any identity check ran. **Measurement.** A 226,214-byte envelope carrying a 4,000-node chain — `MAX_ABI_EXPRESSIONS` less what the compiled program and the chain's literals occupy — made a decode allocate a peak of 1,569,620,906 live bytes at `13.0` and 670,658 at `14.0`, a 2,340-fold reduction, and the quadratic growth is gone: peak live now runs between 2.48× and 3.23× the envelope across a 31-fold change in arena size, where it previously ran from 15.0× to 6,938.7×. The producer side fell with it, because the builder's own key table went at the same time: encoding that artifact peaked at 1,569,451,274 bytes and now peaks at 768,193. A forged manifest carrying such a chain, with only its manifest digest repaired, reaches the arena parser and is refused only afterwards, so the cost was reachable from attacker-chosen bytes rather than merely imposable by a producer. Recorded in [the decoder-allocation research note](research/artifacts/decoder-allocation-amplification.md).

### Identity is derived from the canonical envelope

**Fact — there is one identity encoder and its subject is the envelope.** `encode_identity` is a function of `ArtifactEnvelope`, and the checked builder's terminal projects the verified draft into its envelope before deriving the identity. A decoder re-derives the identity from decoded content through that same function and compares it with the identity the manifest carries, rejecting a mismatch as `ArtifactIdentityMismatch`. There is therefore no second encoder that a decoder would have to agree with by inspection.

**Inference — equal identity implies equal envelope bytes, and three closure checks are what make that true.** Identity replaces arena positions with a canonical numbering of the arena it writes exactly once, and payload and section positions with canonical content keys, so it does not by itself fix the tables those positions index. An envelope carrying an expression no use site reaches, a payload no entry realizes *at any delivery position*, or a section no variant references would keep the same identity while changing the bytes, giving one artifact two byte identities. The decoder rejects all three. This closure is what lets an envelope digest serve as a cache key.

**Fact — an identity a consumer recorded is a distinct type from an identity this crate derived.** `CanonicalArtifactProgramIdentity` has no public constructor and is produced only by the encoder above, so holding one is evidence that content was validated. A cold consumer holds no such thing: it reads bytes a producer wrote beside the cached artifact, which is an assertion about which artifact it wants rather than a derivation. `RecordedArtifactProgramIdentity::from_bytes` is where those bytes become statable, rejecting empty input, input above `MAX_ARTIFACT_IDENTITY_BYTES`, and bytes whose leading frame is not the current artifact-identity domain separator, each as a `RecordedArtifactIdentityError`. Recognizing the domain separates an artifact identity from a kernel identity, a content digest, a cache key, or an identity recorded under a superseded domain; it proves nothing about the remainder. The two types deliberately do not convert in either direction, and a runtime's program-mismatch rejection carries one of each, because comparing equal bytes does not make the warrants behind them equal.

**Fact — a re-proven obligation reports the artifact model's own cause.** A decoded envelope is checked again against the rules the transactional builder discharged at construction, and each rejection carries the model's own typed cause rather than a codec-local restatement: one variant wraps an insertion-time build error, another wraps a whole-artifact diagnostic. A rejection therefore reads the same whether an artifact was refused at construction or at load.

**Fact — two builder obligations are not decidable from an envelope and are pinned by identity instead.** A binding's accessible offset and byte range must equal the exact byte window its stage access addresses, and an entry's bindings must correspond one-to-one with its kernel's buffer parameters. Neither the byte windows nor the kernel signature travel in this profile, so a decoder cannot recompute them. Both are folded into the artifact's canonical identity — through the binding's two canonical positions in the once-written arena and the entry's stage key — and the identity is re-derived and compared, so a forged envelope can restate them only by becoming a different artifact. Carrying the byte windows so the check could run locally was considered and rejected: the window is a value only the program establishes, so a carried copy would prove agreement between two producer-supplied fields rather than agreement with the plan.

### Required features

**Fact — the required-feature set is derived from content and never declared by a producer**, so it cannot understate what a reader must implement; a declared set the content does not imply rejects as `DeclaredFeatureMismatch`. This build derives seven governed keys:

| Governed feature key | Derived when | This reader supports it |
| --- | --- | --- |
| `tiler.artifact.feature.multi-variant-routing` | the portfolio carries more than one variant | yes |
| `tiler.artifact.feature.deferred-predicates` | any variant defers a feasibility predicate | yes |
| `tiler.artifact.feature.launch-preconditions` | any entry declares a launch precondition | yes |
| `tiler.artifact.feature.embedded-payload-code` | any payload carries its object bytes | yes |
| `tiler.artifact.feature.multi-stage-program` | any variant dispatches more than one stage | yes |
| `tiler.artifact.feature.route-requirements` | any variant declares a live-device route requirement | yes |
| `tiler.artifact.feature.multi-payload-delivery` | the artifact declares more than one delivery position | yes |

`tiler.artifact.feature.multi-payload-delivery` is required for the same reason and one more: a reader that resolves no delivery position would take whichever object came first, which is correct for the one-position artifact and silently wrong for any other. A one-position artifact emits no key, so the ordinary single-target artifact stays readable by a reader that predates the family.

`tiler.artifact.feature.embedded-payload-code` is required rather than optional for a reason the mechanism exists to serve: a reader that predates carried payloads would see the descriptors and none of the code and would have no way to notice, because the manifest it understands is complete on its own. Requiring the feature makes such a reader refuse rather than load an artifact whose executable half it silently dropped.

**Fact — this build emits `tiler.artifact.feature.multi-stage-program` and reads it.** Each variant carries its stage execution order and the typed dependency edges that order discharges, both derived by the producer from the packaged program's own `execution_order()` and `dependencies()` rather than stated. A decoder proves the order is a permutation of the variant's entries and that every edge's predecessor precedes its successor in it, so an order contradicting the program's dependency graph is refused rather than executed. Entries themselves remain in canonical stage-key order — identity's order, not execution order — which is why the separate order row exists. `carry-the-stage-execution-order-in-the-envelope` closed the envelope half; `preflight-every-entry-of-a-multi-stage-route` subsequently made the bounded runtime prepare, route, and dispatch every entry, and the retained proof measures the materialized route executing two stages through one shared allocation.

### Live-device route requirements

**Fact — a variant carries the additional requirements its route places on a live device, and the row family is backend-neutral.** "Family" here is this row vocabulary itself and neither a *backend family* nor an *artifact family*; the [glossary](glossary.md#backend-device-and-execution-context-vocabulary) separates the senses, and one of the two kinds below is owned by a backend precisely because the other is not. Two row kinds:

| Kind | Wire fields | Who decides it |
| --- | --- | --- |
| Core quantitative row | governed dimension tag, `u64` required quantity | the neutral runtime, by the relation the dimension fixes |
| Backend-scoped qualitative row | owner `BackendKey`, governed `RouteFeatureKey`, nonzero `u32` version, canonical payload | the adapter the owner names |

The kind tag leads each row so a reader frames the rest of it before reading a field, which is what lets an unrecognized kind be refused by name instead of mis-framed into the row after it. Rows are ordered by canonical content and are distinct **by subject** — the dimension, or the (owner, key, version) triple — because two rows on one subject state two answers to one question and nothing in the envelope can say which the producer meant.

**Fact — what belongs here is decided by derivability, and the elimination is exhaustive rather than illustrative.** A row belongs only when the selected route consumes it *and* the verified program does not already state it. Enumerating the quantitative live-device properties `MTLDevice.h` declares in the macOS 26.5 SDK — `maxThreadsPerThreadgroup`, `maxThreadgroupMemoryLength`, `maxBufferLength`, `recommendedMaxWorkingSetSize`, `currentAllocatedSize`, `maxTransferRate`, `peerCount`, `maximumConcurrentCompilationTaskCount` — the requirement side of the first four is already carried: threads per workgroup by the entry's launch geometry and its proven resource requirements, threadgroup memory by that record's `local_memory_bytes`, and buffer length and resident bytes by each binding's evaluated accessible window. Those stay **derived requirements**, checked directly against the device by an adapter, and `prototypes/serial-sum-run` already compares its evaluated windows against `max_buffer_length`. Copying any of them into a row would mint a second authority a producer could contradict. The remaining four are not correctness predicates on a route.

So the core quantitative vocabulary is narrow by derivation: `RouteResourceDimension` carries the one dimension that survives, a subgroup width, which the neutral kernel IR cannot state because it admits only whole-grid invocation binding and has no subgroup to describe. **Measurement boundary:** it is a live-device property in general — Vulkan publishes `subgroupSize` on the physical device and CUDA publishes `warpSize` — and Metal publishes no device-scoped equivalent, so the first Metal adapter answers it `Unrecognized`, which refuses the route. A Metal route that genuinely needs that width must state it as a `PreparedEntryTargetRequirement` against the prepared pipeline, which is the authority that has it. This is a typed reservation with one implemented backend answer of "cannot decide", not a tested guarantee.

**Fact — the relation is the dimension's and the subgroup width's is an equality, not a floor.** A required quantity is on the wire; the comparison that decides it is not, so correcting a relation moves no artifact identity. `RouteResourceDimension::SubgroupThreads` is compared by equality because a width-`W` combine tree's steps are its content rather than a consequence of it — a butterfly at width 32 has five steps and at width 64 it has six — so a device executing more threads per subgroup than the route was verified at satisfies a floor while running lane arithmetic nothing checked. The row also carries no claim that a subgroup executes in lockstep: CUDA's independent thread scheduling withdraws that guarantee from compute capability 7.0 onward, and a row over a quantity no device provides is one no adapter could soundly observe under any relation. The remaining obligation of a subgroup route — that every lane of `0..W` be active at each combine step — is a property of the program that no target declares and a schedule discharges intrinsically, so only the width crosses this boundary. Accepted at Tom's review of 2026-08-01 on the subgroup execution tier's §3 derivation, and that derivation now carries a decision number: [ADR 0094](decisions/0094-bind-a-subgroup-combine-to-a-register-transfer-tree.md) item 7 decides that a subgroup width is a literal in the schedule, an equality against an atomic target subject, and never a floor. The correction landed here ahead of that record and on its own authority, because a floor over lockstep threads was wrong about a quantity no device provides whatever became of the design that found it.

**Fact — the qualitative half exists because equal numbers cannot distinguish two devices.** The two hosts this workspace reaches are the evidence: an Apple M4 Max and an Apple M3 Pro report the same highest GPU family and identical threadgroup limits, differing only in buffer and working-set size — quantities that track installed memory rather than capability. A requirement that a device support a named feature is therefore not expressible as a number.

**Fact — the neutral layer never interprets a backend payload, and still decides everything decidable without a device.** The payload is bytes the emitting backend minted; reading them here would put a backend's vocabulary — an Apple GPU family, say — inside the neutral core. What the neutral layer does own is the owner's governed-key grammar, the key and version, a non-empty payload bounded at `MAX_ROUTE_FEATURE_PAYLOAD_BYTES`, subject distinctness, and canonical order. An empty payload is refused rather than admitted as "no argument": at this layer an empty payload and a truncated one are the same bytes, and a capability taking no argument can spell that explicitly.

**Fact — zero rows is a state and a missing row is undetectable here.** A route consuming no additional requirement declares none, and no feature key is emitted, so a reader that predates this row family still loads it. Whether a row was *omitted* is decidable only against a producer-owned exhaustive declaration of what the selected payload uses, and no such declaration reaches the artifact. That is why the feature key is required rather than optional for an artifact that does carry rows: a reader that predates the row family would otherwise parse a manifest that looks complete and route without evaluating a precondition the producer declared.

**Fact — a route requirement is attached after the variant rather than inside `VariantSpec`.** A deferred predicate is minted with the plan, by the compiler that chose it. A route requirement states what the *emitted payload* consumes, which is known only after backend emission and to a different producer stage. `ArtifactProgramBuilder::require_route` takes the variant handle and the row; the exact call-site boundary is a reviewed draft under ADR 0074 convention 7.

### Sections

**Fact — the section vocabulary has three governed purposes in this build**: the canonical kernel-program identity of one packaged variant, and a carried backend payload's compilation subject and its object bytes. Two variants that package the same program share one section, and two payloads carrying the same object share one section, because content is the address — so sharing is a stated property of these purposes rather than an accident of equal bytes. Sections are ordered canonically by content; duplicates, unreferenced sections, a section identifier that is not its canonical position, and an unrecognized purpose tag are each rejected by name.

**Fact — a section carries canonical identity bytes, not a digest of them.** ADR 0074 convention 2 makes a canonical identity an opaque byte encoding and short digests presentation-only, so the governed bound on one section is the shared IR's own identity budget rather than a digest width.

**Fact — the backend metadata and code sections landed, and the identity question this contract left open for them is decided.** `prototype-metal-bundle-assembly` added the two governed purposes and decided that **a carried payload is content-addressed over its compilation inputs, and the emitted object is opaque**. The descriptor's digest is required to equal the identity of the exact canonical payload-metadata bytes — source, target, flags, toolchain provenance, entry mappings, and recorded obligations, and no object byte at all — and `push_carried_payload` derives it rather than accepting one, so a payload cannot claim a subject it does not carry; `PayloadIdentityMismatch` re-proves it on every decode. Content-addressing over the emitted bytes was rejected because it would make artifact identity a function of compiler-output reproducibility, which this document refuses to promise under "Artifact identity" below.

**Fact — which provenance fields a payload owes follows the shape it declares, and the platform block is a backend's statement rather than a required Apple one.** As of [`generalize-payload-provenance-beyond-the-apple-shape`](../tickets/generalize-payload-provenance-beyond-the-apple-shape.md), every carried payload owes a toolchain, a target, a family, a language, and a role and a version for each tool component it lists. Beyond that it declares a platform shape: `VersionedSdk` says its toolchain resolved against a versioned platform SDK and additionally owes a non-zero deployment-minimum major and all three SDK fields, while `Unversioned` says it resolved against none and owes — and may state — no platform field at all. An owed field left empty is refused by that field's name (`IncompletePayloadProvenance`) where the payload's identity is derived, and again on decode because an artifact's bytes arrive from a producer the reading process never ran. Nothing became optional: a Metal payload owes exactly what it owed before, and now owes it to a check rather than to a convention. This closes the third of the four vocabulary gaps [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 14 named — a backend with no SDK previously had to mint one, and an approximated field that enters durable identity is worse than an absent one because it makes two unlike artifacts comparable.

**Fact — that widening moved no already-encodable payload's bytes, and the mechanism is an appended tag.** The versioned-SDK shape encodes to exactly the bytes this record encoded before it had a platform block: the deployment minimum in its two `u16` positions ahead of the component list, the three SDK runs after it, and nothing appended. The unversioned shape writes those same positions as two zeroes and three empty runs and appends one tag byte after the obligation list. Injectivity is argued per tag and rests on the grammar being self-delimiting — every run carries a fixed-width length and every collection a fixed-width count, so the number of bytes one record occupies is a function of the bytes themselves, and a versioned encoding is that grammar followed by nothing while an unversioned one is that same grammar followed by exactly one byte. No tag value is admitted for the versioned shape, and a tagged encoding that filled a platform position is refused as `PlatformFieldWithoutPlatform` rather than normalized, because one record with two spellings is two payload identities. A later platform shape is another appended tag and moves nothing either. The cost, stated rather than hidden, is 28 pinned bytes plus a tag in every unversioned payload's identity subject; the alternatives — a leading discriminator, or moving the block to the end under a new schema minor — would each have moved every already-published Metal payload's identity and every expansion-cache key that folds one.

**The cost of that choice is recorded rather than hidden.** *Equal identity implies equal bytes* now holds for the identity-bearing part of an envelope and not for the object sections, so two bundles built from one compilation subject by a non-reproducible linker have equal artifact identity and different envelope digests. The expansion cache is keyed by artifact identity, which this contract already requires, and an envelope digest names one published encoding rather than the artifact. The object section still carries its own content digest, so a substituted library is refused — as a corrupted encoding of this artifact rather than as a different artifact.

### The governed digest

**Accepted boundary, not yet implemented — 2026-08-12.** [ADR 0111](decisions/0111-separate-externally-specified-raw-hashes-from-governed-tiler-digests.md) separates an externally specified raw digest record from every governed subject below. The accepted external path names the exact algorithm from the external record and returns an opaque result type that cannot convert to or from the governed `Digest`; it adds no domain, tag, wire field, artifact identity, or alternate spelling of any pre-image in this contract. The envelope and sidecar continue to use only the governed domain-separated path described here.

**Fact.** The envelope names its digest algorithm by an explicit header tag and a reader never infers one from a digest width. `0x01` is `tiler.digest.sha-256.v1`, the only admitted value in this build. The envelope governs **seven** domain separators as fixed NUL-terminated crate constants, so `H(domain || bytes)` genuinely separates its subjects rather than colliding a longer domain with a shorter one plus leading content. Four are digest arguments:

```text
manifest_digest = H("tiler.artifact-envelope.manifest-digest.v1\0"
                    || exact canonical manifest bytes)
section_digest  = H("tiler.artifact-envelope.section-digest.v1\0"
                    || section purpose tag || section content schema
                    || exact section bytes)
envelope_digest = H("tiler.artifact-envelope.envelope-digest.v1\0"
                    || exact complete envelope bytes)
identity_digest = H("tiler.artifact-envelope.identity-digest.v1\0"
                    || exact canonical artifact identity bytes)
```

A fifth is a digest argument reached through a carried payload, and two are framing tags rather than digest arguments:

```text
payload_identity = H("tiler.artifact-envelope.payload-identity.v1\0"
                    || exact canonical payload metadata bytes)
manifest bytes         open with "tiler.artifact-envelope.manifest.v1\0"
payload metadata bytes open with "tiler.artifact-envelope.payload-metadata.v1\0"
```

**Fact — a framing tag is governed by the same obligation as a digest argument, and for the same reason.** A tag that opens a canonical byte run is the leading content of a pre-image that is digested, compared, or recognised, so a domain that prefixed it would merge those bytes with another subject exactly as a colliding digest domain would. The recognition case is the sharpest: `RecordedArtifactProgramIdentity::from_bytes` admits bytes by `starts_with` on the artifact-identity separator, so a governed domain that prefixed that separator would let another subject's bytes be accepted as an artifact identity with no digest involved at all. The sidecar's list below has always counted its two framing tags for this reason; the envelope's two were the ones missing.

**Fact — the identity digest's pre-image opens with the artifact identity's own separator, so the identity domain's steps are inside the digested bytes.** A digest taken over a `tiler.artifact-program.v14` identity can therefore never equal one taken over a `v15` identity, and the identity-digest domain does not have to restate the identity version to stay injective across it. It is a separate domain rather than a reuse of `manifest-digest` because that one covers the manifest bytes this digest is written *into*.

**Fact — the no-prefix obligation is over the crate's *eighteen* governed domains, not over the envelope's seven, and it is normative rather than incidental.** Domain separation by prefix is a property of the whole admitted set: the envelope's seven above, the proof sidecar's four (recorded under "Proof-case evidence sidecar" below), and the artifact program's seven identity and key domains are hashed by one algorithm in one process, so a domain added to any of them could collide with one in another. A check confined to one container would report a separation it had not established. `tiler_artifact::domains::no_governed_domain_of_this_crate_prefixes_another` checks the union and is the authority for the property; the envelope-local test beside the codec is the other half, derives its population from the same enumeration, and names the union test as the authority. **A new governed domain anywhere in this crate must be added to that enumeration**, and adding one to the envelope-local check alone does not discharge this obligation.

**Fact — the population is enumerated from a type, because a hand-written count is what let it go stale.** `tiler_artifact::domains::GovernedDomain` names every governed domain the crate admits; `ALL` declares its length as `core::mem::variant_count`, and `bytes` and `container` are wildcard-free matches, so a widened vocabulary is three build errors rather than a list that quietly stops covering. The per-container counts this document states are asserted against `variant_count` in a `const` block, so a domain admitted without a documented count moving fails to compile. A third check reads the crate's own sources and requires every declared domain constant to appear in the enumeration — the one mechanism that catches a constant added to a module and enumerated nowhere, which is precisely how the envelope's manifest framing tag and both payload domains came to be admitted while the union check still reported success over eight of them.

**Correction — 2026-08-10.** The paragraph below used to say that the cross-crate obligation was "discharged by construction rather than by a check" and that "every domain the shared IR admits opens `tiler.ir.`", so "the two sets diverge at the first byte after the shared `tiler.`". It also said "Neither crate can hold a check over the union: `tiler-artifact` depends on `tiler-ir` and not the reverse". None was a sound premise: `EXPR_DOMAIN` has lived at `tiler.artifact-program.abi-expr.v1` in `tiler-ir` since before this contract made the namespace claim, and the dependency points in the direction that would let `tiler-artifact` consume an IR enumeration. The retired wording is quoted only here so a later search lands in this correction rather than mistaking it for a live premise.

**Fact — the cross-crate no-prefix obligation is discharged by a spelling and terminator argument over the observed IR population, not by namespace construction.** Since [ADR 0104](decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) the shared IR has admitted governed domains under the same algorithm and in the same process. No crate currently owns a check over the union. `tiler-ir` cannot see this crate's domains because it does not depend on this crate. This crate does depend on `tiler-ir`, but the complete IR pin population is private and test-only, with no exported enumeration to range over. `tiler-digest`, which owns the algorithm, deliberately knows no subject domain at all because a domain belongs to the authority that decides what it names.

Read at this commit, the separation follows from the spellings and their terminators. Every domain this crate admits ends in one NUL that occurs nowhere else in it, and every terminated spelling in the IR pin population likewise carries its only NUL at the end. A strict prefix in either direction between two such spellings would therefore require one of them to carry a NUL at an interior position. Every IR spelling without a terminator carries no NUL at all; those spellings open `tiler.contract.` or are exactly `tiler.scalar`, neither of which any domain admitted here extends. Exact equality is the remaining cross-crate possibility, and inspection of the complete private IR pin population against this crate's complete enumeration finds none.

**A crate admitting a governed domain owes the check over its own set and this argument against every other set.** Only this crate's half is mechanically checkable here: `no_governed_domain_of_this_crate_prefixes_another` checks its complete enumeration, requires every member to open with an established prefix, and requires the terminator to be its only NUL. A newly admitted domain in either crate must reopen the cross-crate argument; a domain here spelled outside the established prefixes or carrying any other NUL breaks the local test rather than silently weakening the contract.

The established local prefix is `tiler.artifact` rather than `tiler.artifact-` because the core route requirement's `tiler.artifact.route-requirement.v1` is spelled with a `.` where the envelope and program families use a `-`. This is a fact about the local population the test checks, not a claim that the IR occupies a separate namespace.

A section descriptor is derived from its section's position and exact bytes at encode time and re-derived and compared at decode time, never stored beside the bytes it describes, so the two cannot disagree in memory. The envelope digest is computed and never stored in band; a test asserts its bytes occur nowhere in the envelope it covers.

**Fact — the section digest binds the section's purpose and content schema, so it is a standalone content address.** The pre-image is the domain separator, the purpose tag, the content schema, and then the exact section bytes; the qualifiers are fixed width and precede the variable-length content, so no length prefix between them is needed for the pre-image to be unambiguous. Binding the purpose is what makes the digest usable *outside* a complete envelope, which is what content-addressing a backend code section does. Inside an envelope the purpose was already bound one level up, by the manifest descriptor that names it and the manifest digest that covers that descriptor; lifted out, a digest over bytes alone would give two sections of different purposes one address.

**Fact — the wire algorithm and its implementation are both selected, but remain separate authorities.** The governed wire value is SHA-256 under `tiler.digest.sha-256.v1` and tag `0x01`; `select-the-governed-artifact-digest-implementation` subsequently measured the implementation choice and adopted `sha2` 0.11.0. The unchanged FIPS 180-4 vectors, padding cases, and independent message-length sweep pin the output bytes across that replacement, so a future implementation change remains internal only if it preserves every governed digest byte.

**Fact — the algorithm is owned by `tiler-digest`, the workspace's bottom crate, and this contract governs its use rather than its home.** `DigestAlgorithm`, the opaque `Digest`, `DIGEST_BYTES`, and the tag table lived in a private module of `tiler-artifact` until [ADR 0104](decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md), which needed the algorithm in `tiler-ir` — the crate every other member depends on, so the reachability problem could not be resolved by moving the *consumer* the way [ADR 0082](decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md) moved the expansion cache. Tom decided on 2026-08-06 that hashing is its own crate below both. `tiler-artifact` re-exports the same three names from `tiler_artifact::program`, so every path this contract's consumers already used still resolves, and `tiler-cache` reaches it exactly where ADR 0082 says it does. **The property being preserved is unchanged and is the reason the crate exists**: being the only place that maps the governed tag to an implementation is what a second component reaching for a hash function would destroy, and it now has a structural home rather than riding in whichever crate needed it first.

**Fact — the public surface admits exactly two pre-image shapes, and refuses to express a third.** A caller supplies one domain and one variable-length run, or one domain, fixed-width qualifiers, and one trailing variable-length run — the two shapes the digests above actually take. There is no entry point that hashes an arbitrary sequence of parts: such a call puts unambiguity entirely on the caller, and a concatenation of variable-length runs has no unambiguous reading. This is what makes the section digest's qualifier discipline, stated below, a property of the signature rather than of a convention.

### Deliberate exclusions

**Fact — the frozen registry snapshot never enters the envelope.** ADR 0072 keeps the provenance of providers a plan never used out of packaged artifact identity, and carrying the snapshot here would put it back into the envelope's bytes and therefore into its digest, letting an unused provider invalidate a cache entry. Only the three reached subjects travel: the semantic graph identity, the reached definitions, and the admission provenance. **Measurement.** An artifact built with an additional available-but-unreached provider encodes to identical bytes and an equal envelope digest, while changing a *reached* provider's revision changes the digest.

**Fact — four of five now: the shape environment travels, the registry snapshot does not.** *Corrected 2026-08-13 by [`evaluate-retained-shape-relations-before-routing-commit`](../tickets/evaluate-retained-shape-relations-before-routing-commit.md).* The 2026-08-08 sentence that the fifth subject could be dropped because no two artifacts can differ by it was already false at that date: `SemanticProgramBuilder::try_standard_with_shape_environment` accepts ordinary fixed-shape inputs, and `project_semantic` then dropped the unused environment. Two fixed-interface programs could differ only by a retained environment and collapse to one envelope. The subject now travels as the private lossless `RetainedShapeEnvironment` inside the semantic-subject run. The registry snapshot remains omitted under ADR 0072. A symbolic *interface* is still refused as `ArtifactBuildError::SymbolicSemanticInterface`; that refusal no longer has to carry the fifth subject's injectivity.

**Fact — presentation-only declaration order never enters it**, under the ordering rules above. This is what makes the envelope's bytes a function of the artifact's identity rather than of the order a producer happened to declare things in.

**Fact — no backend spelling enters the neutral manifest, and the emitted object enters no identity.** The manifest names a payload by governed backend and representation keys, its own schema version, the digest of its compilation subject, and its execution policy, and by nothing a backend would spell: no symbol names, binding indices, platform triples, or language versions occur there. Since `prototype-metal-bundle-assembly` those backend spellings and the emitted object *are* carried, each in its own governed section rather than in the manifest — the compilation subject in a `BackendPayloadMetadata` section, which artifact identity folds through the descriptor's digest, and the object bytes in a `BackendPayloadCode` section, which artifact identity excludes entirely. The exclusion that survives is the sharp one: what an artifact *is* does not depend on what a linker emitted.

**Fact — a reconstructable kernel program is not carried, and the blocker is structural rather than an omission.** `tiler_ir::program::KernelProgramBuilder::new` takes a `&SemanticProgram`, which requires a frozen semantic registry holding live inferencer implementations; neither is representable as bytes. A decoded envelope therefore proves *which* program an artifact names and cannot resurrect it, and a consumer that needs a verified kernel program must hold the one it compiled.

**Decision — Tom, 2026-07-25, on `carry-reconstructable-kernel-programs-in-the-neutral-envelope`: a decoded envelope is a dispatch record, never a reconstruction.** It carries entries, bindings, and launch expressions as encoded facts a decoder structurally validates and folds together with the opaque packaged-program identity when re-deriving artifact identity; it does not prove those facts against a program it cannot reconstruct. Full IR reconstruction was excluded on evidence rather than preference: the registry the builder needs holds behaviour, not data, so the option was impossible at any encoding cost rather than merely expensive. This is what the ownership boundary above now states, in place of the requirement that a decoder reconstruct shared IR through its checked builders.

**The accepted cost, stated rather than left for a reader to discover.** A binding's target is the one dispatch fact a decoder cannot re-derive: the program that established it reaches the envelope only as identity bytes, so the correspondence is carried rather than recomputed on read. It is not, however, *asserted* by a producer — `ArtifactProgramBuilder::check_bindings` derives it from the program's own stage access, so a producer cannot state a correspondence its plan contradicts, and artifact identity folds it, so a forged envelope restating a target becomes a different artifact. What is weaker than re-derivation is that the proof happened on the writing side, and only a consumer that compares identity has rejected the forgery. What *is* decidable from the manifest alone — that a target names an interface entry the artifact declares — is checked on every decode as `UnknownBindingTargetKey`.

### Governed budgets

**Fact.** Every budget is enforced on both sides by one constant. The encoder checks a projected envelope up front, so a legally built artifact that could not be read back fails to encode rather than producing bytes no reader admits; the decoder checks each count the moment it is read and before anything is allocated for it. The envelope-level bounds are 256 MiB per complete encoding, 64 MiB per manifest, 64 MiB per section, 16 MiB per received opaque identity subject, 4 KiB per encoded text run, 64 required features, 4,096 named interface entries, and 4,096 declared shape rank. Every per-collection bound — variants, entries, bindings, expressions, payloads, providers, deferred predicates, launch preconditions — reuses the artifact model's own constant rather than introducing a second authority for the same limit that would drift from it.

**Fact — a received opaque identity is bounded by the authority that mints it, not by its shape.** A governed key is bounded at 256 UTF-8 bytes and spelled in ASCII lowercase, ASCII digits, `.`, `-`, and `_`, because this layer governs what a producer may name — the spelling as much as the length, since every one of these keys exists to be compared byte for byte against one minted by a producer that never met this one, and a key carrying case, whitespace, or a control byte would leave two keys a reader sees as one comparing unequal and could not be copied back out of the rejection that prints it. `tiler_compiler::target::TargetProfileKey` admits exactly that alphabet, deliberately, so every profile key that compiler mints is packageable here; the byte bounds deliberately do *not* agree, because 128 there is that compiler's *minting* bound and 256 here is this layer's *admission* bound, and the smaller-mints-into-larger direction is what makes the difference safe rather than a gap. An opaque identity is bytes another authority derived, and the number that admits every value that authority can legally mint is that authority's own; anything else is this layer deciding a question it has just said it does not decide. So a backend entry key — the canonical identity of the structured kernel one executable entry realizes — is bounded by `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES`, 16 MiB, the exact constant the shared IR enforces when it mints one. A payload content digest is fixed-width under the governed digest algorithm and stays under the artifact layer's own 1,024-byte digest bound. A target-profile descriptor identity does not: its bytes are a canonical encoding rather than a hash, so it carries its own 64 KiB ceiling, the same number `tiler-compiler` refuses past where a descriptor is minted. Neither crate depends on the other, so that equality is held by review rather than by a check, and changing either side requires checking both.

**Measurement — the three identities never shared a subject, and one shared bound made the artifact contradict itself.** Until this was separated, all three were bounded at 1,024. The canonical kernel identity of the prototype's serial `f32` sum crosses that bound at the *second* contributor and not at any data size: reducing one contributor it fits, and reducing two, three, four, or eight it does not, so the bound admitted the degenerate reduction and refused every real one, and `prototypes/serial-sum-compile` could not package the program `prototypes/serial-sum-run` dispatches. It grows with program structure at a fixed slope per rank and without a small ceiling, so no round number below the minting authority's own is safe. The contradiction was internal rather than merely inconvenient: an executable entry carries that same identity **twice**, once as its backend entry key and once inside the stage subject `stage_key` derives from it, and the second was already admitted to 16 MiB — so the smaller bound refused values the envelope beside it had accepted and guarded no allocation the stage subject had not already made.

**The lengths behind that argument, dated to the trees they were taken on rather than stated in the present tense.** They are a record of why the bound moved, so both the reading the decision was made on and a reading at a later tree belong here; neither is a claim about any other tree. Host for both columns: Apple M4 Max, macOS, the toolchain `rust-toolchain.toml` pins. The left column is the sweep recorded in [`bound-the-backend-entry-key-by-the-identity-it-carries`](../tickets/bound-the-backend-entry-key-by-the-identity-it-carries.md) on 2026-07-25, which is the evidence the bound was changed on. The right column re-runs the identical shapes on 2026-08-08 at commit `68ba010a`, macOS 27.0 (26A5388g), toolchain `nightly-2026-07-19`, recorded by [`date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi`](../tickets/date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi.md).

| input shape | reduced axes | 2026-07-25 | 2026-08-08 at `68ba010a` |
| --- | --- | --- | --- |
| `[4, 1]` | `[1]` | 736 | **924** |
| `[4, 2]`, `[4, 3]`, `[4, 4]`, `[4, 8]` | `[1]` | 1,121 | **1,309** |
| `[4, 3, 3]` | `[2]` | 1,483 | **1,671** |
| `[4, 3, 3, 3]` | `[3]` | 1,845 | **2,033** |
| `[4, 3, 3, 3, 3]` | `[1, 2, 3, 4]` | 1,700 | **1,888** |
| `[4, 3, 3, 3, 3, 3, 3, 3]` | `[1, 2, 3, 4, 5, 6, 7]` | 2,279 | **2,467** |

**Every figure moved, and the conclusion did not.** 924 is still under 1,024 and 1,309 still over, so the old bound would still have admitted exactly the degenerate reduction and refused every real one. Both slopes reproduce to the byte: +362 per rank reducing one axis, and +193 per rank reducing all but one, re-measured at every intermediate rank of the second family (1,888 / 2,081 / 2,274 / 2,467 across ranks 5 to 8). Only the constant offset moved. **The uniform +188 across all six is a property of this one step and must not be used to correct a later reading.** The retirement correction above records the neighbouring `v15 -> v16` step going the other way entirely — the envelope shrank rather than grew, and two of the offsets it retired ended up past the end of the identity they indexed — so a delta measured at one step is not a portable correction for another. A reader who needs a current number regenerates it from the construction below rather than adding one to these.

**Reproducing it, including the part that no longer reproduces the way it was first taken.** Build the scale-then-bias-then-`StrictSerialF32Sum` program at the shape and reduced-axis set in the table, compile it, and read `VerifiedKernel::canonical_identity().as_bytes().len()` off the selected plan's kernels. The 2026-07-25 sweep compiled through `tiler_compiler::session::compile_governed` under `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32`; at `68ba010a` that route refuses `[4, 3, 3]` reducing `[2]`, `[64, 3]`, and `[4096, 3]` as `NoFeasiblePlan` before a plan composes, so it can no longer reach the rank-3 and rank-4 single-axis rows at all. The right column was taken under `BoundMetalCompileDeclaration::first_macos_apple9()`, the declaration `prototypes/serial-sum-compile` itself compiles under, which reaches every row; where both routes admit a shape they agree on the length exactly, so the route is a reachability difference and not an identity difference.

**What is pinned here and what is not, stated rather than left for a reader to assume.** No test asserts any length in the table above, and that is deliberate rather than a gap: a length pinned in a test decays the moment the constant offset moves, which is what "Every figure moved, and the conclusion did not" above records happening to every figure in that table at one identity step. What *is* asserted, as of 2026-08-08, is the two-sided inequality the Measurement headed "the three identities never shared a subject" turns on. `crates/tiler-conformance/src/serial_sum/tests.rs`'s `the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor` builds the same scale-then-bias-then-`StrictSerialF32Sum` program at one contributor and at two, compiles each under `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` against `BoundMetalCompileDeclaration::first_macos_apple9()` — the right column's route, not the left's — takes the widest canonical kernel identity the selected plan's kernels carry, and asserts from both sides that the one-contributor reduction's is **under** `MAX_OPAQUE_IDENTITY_BYTES` and the two-contributor one's is **over** it. It pins no length: the two readings reach a reader only through its diagnostic output and its failure messages, never through an assertion. It reads the widest rather than the first because an artifact carries each entry's identity as its own `BackendEntryKey`, so the bound is crossed as soon as any one of them crosses it; and it asserts both directions because either half alone is satisfied by an identity that stopped growing with the program.

**The structural argument is unchanged, and it is why that assertion lives one crate over rather than beside the constant.** `crates/tiler-artifact/Cargo.toml` deliberately carries no `tiler-compiler` edge in either position. `tiler-runtime`'s `the_consumer_links_no_compiler_emitter_or_build_provider` walks `Cargo.lock`, which merges normal, build, and development edges into one list per package, and refuses `tiler-compiler` anywhere in the consumer's transitive closure — the closure [ADR 0081](decisions/0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md) item 2 decides. So the crate that owns the bound can never compile a real reduction to check against it, and the assertion belongs where both crates are already reachable, which `tiler-conformance` is.

**What the `tiler-artifact` case does assert, restated.** `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` (`crates/tiler-artifact/src/program/tests.rs`) constructs a `BackendEntryKey` over a **fabricated** vector whose length is *derived* from the bound rather than measured — `MAX_OPAQUE_IDENTITY_BYTES + 1`, the smallest length that bound refuses — so it states that a backend entry key is admitted past the shared bound, and nothing at all about a kernel. Its sibling `an_artifact_encodes_an_entry_key_longer_than_the_digest_bound` derives its long key the same way. Against a derived length the assertion that had stood beside the first — that the vector's own length exceeded `MAX_OPAQUE_IDENTITY_BYTES` — is a tautology, and it was deleted rather than re-derived. `MAX_OPAQUE_IDENTITY_BYTES`'s own documentation in `crates/tiler-artifact/src/program/keys.rs` names the conformance case as the evidence for the measured half, so the crate that cannot check the claim at least says where it is checked.

**Dated 2026-08-08 by [`correct-the-artifact-abis-claim-that-nothing-asserts-the-kernel-identity-crossing`](../tickets/correct-the-artifact-abis-claim-that-nothing-asserts-the-kernel-identity-crossing.md), and the two retired sentences are quoted rather than deleted so they stay greppable — a later search for either lands in this note and not in a live claim.** They read "No test asserts any of these lengths, and none asserts the inequality the paragraph turns on" and, of the `tiler-artifact` case, "it builds `vec![0x5a; 1_121]` and asserts that length exceeds `MAX_OPAQUE_IDENTITY_BYTES`". Both were true of the tree they were written at and false forty-seven minutes later. They landed at `0f8b0c32`, where `git show 0f8b0c32:crates/tiler-artifact/src/program/tests.rs` reads `let measured_kernel_identity = vec![0x5a; 1_121];` and `git show 0f8b0c32:crates/tiler-conformance/src/serial_sum/tests.rs` contains no crossing case at all; `fe282f1e` then landed [`pin-the-serial-sum-kernel-identitys-crossing-of-the-opaque-identity-bound`](../tickets/pin-the-serial-sum-kernel-identitys-crossing-of-the-opaque-identity-bound.md) and falsified both. **Dating a claim beside rather than substituting it is what this document does for a claim that was true at its own tree**, and the two treatments are separated by that test rather than by how wrong a sentence reads now: "The lengths behind that argument" above dates the table's two columns instead of refreshing them, and "those six names were the whole golden corpus when this step landed" dates a fixture enumeration for the same reason, while item 4 under "Where the implemented profile is narrower than this contract" *substitutes* its withdrawn "a program share one compiled object across variants declaring different profiles" because no artifact could ever have exercised it. Neither treatment is decided by an ADR; both are read off the practice, and the way to tell them apart is `git show <commit>:<path>` at the commit that wrote the sentence. **What the retired pair got right, and this note keeps, is that a length is the wrong subject to pin**: that case stayed green across the whole drift the table's two columns record, while its own comment calling its literal "the measured one" became false.

### Rejection vocabulary

**Fact.** Failure is typed and non-erasing. A rejection names the boundary that refused and the subject it refused; a framing, schema, canonical-form, structural, or identity failure is never reinterpreted as a plan-applicability miss; and a rejection never yields a partially validated envelope.

| Boundary | Typed causes |
| --- | --- |
| Framing and integrity | `Truncated`, `TrailingBytes`, `TrailingManifestBytes`, `BadMagic`, `BadManifestDomain`, `BadPayloadMetadataDomain`, `TotalLengthMismatch`, `ManifestDigestMismatch`, `SectionDigestMismatch`, `SectionLengthMismatch`, `SectionCountMismatch`, `NonCanonicalSectionId` |
| Schema and feature compatibility | `UnsupportedEnvelopeFormat`, `UnsupportedCanonicalEncoding`, `UnsupportedManifestSchema`, `UnsupportedComponentSchema`, `UnsupportedSectionSchema`, `UnsupportedPayloadMetadataSchema`, `UnsupportedDigestAlgorithm`, `UnsupportedRequiredFeature` |
| Canonical form | `NonCanonicalOrder`, `DuplicateItem`, `NonCanonicalManifest`, `DeclaredFeatureMismatch`, `UnreferencedSection` |
| Structure and closure | `MissingReference`, `SectionPurposeMismatch`, `SectionDispositionMismatch`, `EmptyBindingTarget`, `UnknownBindingTargetKey`, `MalformedInterfaceComponents`, `UnknownBindingTargetComponent`, `BindingComponentMismatch`, `BindingAccessTypeMismatch`, `UnmappedBackendEntry`, `EntryTransportCardinality`, `ExpressionOperandOrder`, `ExpressionOperandType`, `ExpressionSelectBranchType`, `UnknownTag`, `InvalidText`, `InvalidGovernedKey`, `InvalidInterfaceKey`, `InvalidProviderIdentity`, `InvalidShape` |
| Governed budgets | `Limit` |
| Re-proven model obligations | `ModelRule`, `ModelObligation` |
| Identity | `ArtifactIdentityMismatch`, `PayloadIdentityMismatch`, `IdentityDerivation` |

Each variant carries the structured data a caller reacts to rather than a message: which collection was out of order, which enumeration presented an unimplemented tag with the rejected tag byte, which table a reference missed with the rejected index, which resource was exhausted with its attempted and permitted quantities.

**Fact — eight of those decide the dispatch record, and each names a way an envelope could otherwise be wrong without being malformed.** `SectionPurposeMismatch` refuses a reference that resolves to a well-formed section of the wrong governed purpose, which would otherwise load an artifact whose executable half had been replaced by another section of its own envelope with no framing or integrity check failing. `UnknownBindingTargetKey` refuses a binding directed at an interface name the artifact does not declare. `MalformedInterfaceComponents` refuses an empty component set or repeated semantic role; `UnknownBindingTargetComponent` refuses a role absent from the named interface value; `BindingComponentMismatch` refuses a binding whose carrier, encoding, or access type disagrees with that component; and `BindingAccessTypeMismatch` independently refuses a kernel access type incompatible with the physical storage. `UnmappedBackendEntry` refuses a payload that maps no symbol for an entry it realizes, and `EntryTransportCardinality` refuses a mapping whose transport slots do not correspond one-to-one with that entry's bindings — either of which would move the failure to a loader with less to say about it.

**Fact — a route requirement is refused through the vocabulary it broke rather than through a class of its own.** An unrecognized kind or dimension tag is `UnknownTag` carrying the rejected byte under the `RouteRequirementKind` or `RouteResourceDimension` subject; a stored order that is not canonical is `NonCanonicalOrder` under the `RouteRequirement` subject; a zero required quantity, a zero version, an empty payload, and two rows on one subject are each `ModelRule` carrying the artifact model's own cause; an owner or key that fails its grammar is `InvalidGovernedKey`; and a count or payload beyond its bound is `Limit` under `RouteRequirements` or `RouteFeaturePayloadBytes`. Adding a class here would have drawn a distinction the codec already draws.

**Measurement — the adversarial cases build a structurally invalid envelope and then encode it**, which stamps a correct manifest digest, correct section digests, and a correct identity for whatever the forgery now says, and require the decoder to reject it anyway by name. Corrupting bytes and watching a digest reject them proves comparatively little, because a forger recomputes digests.

### Where the implemented profile is narrower than this contract

**Fact.** Four normative statements elsewhere in this document once described a format wider than the one this build writes. Items 1, 3, and 4 have since been closed and are retained here as the record of what closed them; item 2 alone is open, with a stated trigger rather than an open-ended gap. Item 3 closed differently from the other two: they were closed by widening the implementation to meet the contract, and it was closed by a decision that narrowed the contract to what the implementation can do.

1. **A section descriptor now carries all four declared fields, and one difference remains.** It is an ordered identifier, a purpose tag, the purpose's required/optional disposition, the purpose's content schema, the exact byte length, and the content digest. The one remaining narrowing is deliberate: the *digest algorithm* is named once in the header rather than per descriptor, because one envelope is digested under one governed algorithm and a per-descriptor spelling would admit an envelope whose sections disagreed about it.

   The disposition and the content schema are properties of the *purpose* rather than of the instance, and are written anyway, because the reader that needs them is precisely the one that does not recognize the purpose and so cannot derive them. A reader that does recognize a purpose owns the answer, and this build therefore requires a descriptor to agree with its own table — a disagreement is a descriptor asserting a schema or a skip permission rather than reporting one, and is rejected by name. Every purpose this build writes is `Required`, and an unrecognized purpose is refused outright, so no skip path exists yet; the field is the mechanism item 2 below will need.

   Adding both fields moved the manifest schema to **2.0** — a major step rather than the minor one it might look like. A minor step would have been wrong, because the reader admits `minor <= implemented` and would have gone on accepting a `1.0` manifest whose descriptors it can no longer parse. A field added inside a fixed-width record is not additive. The envelope format and the canonical encoding profile in the header are unchanged at `{1, 0}`: the manifest's layout moved, not the framing around it.
2. **There are no optional sections, so "unknown optional sections may be skipped only when their schema explicitly permits it" describes no implemented behaviour.** Every unrecognized purpose fails closed. This is the deliberate version-1 posture the envelope research records, and "Loading and validation" below already conditions the skip mechanism on exposing the format outside a lockstep release. It is an explicitly deferred question with that trigger rather than an unrecorded gap.
3. **A decoder does not reconstruct shared IR through its checked builders, and no longer owes it.** The requirement was withdrawn by Tom's decision of 2026-07-25 recorded under "Deliberate exclusions": a decoded envelope is a dispatch record, and reconstruction was excluded as structurally impossible rather than merely unbuilt. What survives of the original requirement is satisfied — nothing in the codec manufactures a verified value, a decoded envelope is a validated envelope rather than a second editable authority, and any future path that does yield an IR value out of an artifact remains bound by ADR 0071's amended clause. The dispatch record itself is implemented and public, as recorded under "Maturity of the implementation" above.

   **The two follow-on runtime gaps are closed.** A variant's stage execution order is recoverable as of `carry-the-stage-execution-order-in-the-envelope`: the envelope carries the order and the typed dependency edges it discharges, and a decoder refuses an order that contradicts them. Entries still reach a reader in canonical stage-key order, which is identity's order, so a consumer sequences by the order row and not by the entry table. `preflight-every-entry-of-a-multi-stage-route` then made the loader preflight and route every entry in that order and pair the internal slots that share an allocation. A binding addressing only part of a value is packageable as of `carry-the-byte-offset-of-a-partial-binding-view`: the binding row carries the offset its range starts at beside the extent, both derived from the packaged program's own byte window rather than restated by a producer, identity folds them, and a decoder re-proves each at its own use site. `carry-the-binding-offset-through-the-runtime-route` now publishes that evaluated offset beside the extent, and the Metal proof host binds storage at that byte after sizing the allocation through the end of the window. The dispatch record is therefore complete for both facts this item originally listed as missing.
4. **The backend payload descriptor carries its compatibility-contract reference.** It carries the backend key, representation key, payload schema, content digest, the target profile the payload's own bytes were built against, and the execution policy. The reference is folded into the payload's canonical key and therefore into artifact identity, so two payloads that agree on every other field but were built against different profiles are two payloads rather than one.

   The field is the payload's contract, not the plan's. A variant's `TargetProfileRef` and `FeasibilityRuleSetRef` are the *plan's* declared target requirements, and the two coincide only while an artifact carries one payload — which nothing in this model requires, since an entry cross-references one payload per delivery position and each is a separately compiled object. Carrying the contract per payload is what lets an artifact built for several consumer targets state what each of its objects was built for; without it a loader would infer a payload's contract from the variant it happened to route to, which is the inference this layer exists to forbid.

   One sentence here justified the field with a case no artifact can reach, and it was withdrawn rather than made true: it said the field lets "a program share one compiled object across variants declaring different profiles", and `ArtifactProgramBuilder::check_subject` refuses a second variant declaring a different profile as `TargetProfileMismatch`, so no artifact could exercise it. That refusal is unchanged — the `v13` delivery-position step widened what an *entry* may name and never touched the agreement every variant of one artifact owes its siblings, so the shape the withdrawn sentence described is exactly as unreachable now as it was then. What the field does carry its weight for is the case that *is* reachable and is now the ordinary one — several objects in one artifact, one per delivery position, each with its own compatibility contract. Both halves are pinned: `program::tests::refuses_a_second_variant_declaring_a_different_target_profile` for the refusal, `program::tests::packages_one_payload_per_delivery_position` for the reachable case.

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

**Corrected 2026-08-08 by [`reconcile-the-artifact-abis-four-hashing-sites-with-the-fifth-it-names`](../tickets/reconcile-the-artifact-abis-four-hashing-sites-with-the-fifth-it-names.md) — between `96dfe333` and this repair the sentence above read "The framing magic, the seven domain separators", which is the envelope's number carried into the sidecar's section by the sweep that corrected it there.** The retired wording is quoted verbatim so it stays greppable, and a later hit for it lands in this note rather than in a live claim. It was never true of this container: `git show 96dfe333~1:docs/artifact-abi.md` carries "the four domain separators" in this position, `DomainContainer::PROOF_SIDECAR` in `crates/tiler-artifact/src/domains.rs` declares four, and the sidecar's own governed-domain section below and the eighteen-domain census above both already said four while this sentence said seven.

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

The magic differs from the envelope's `TILERART` in the first differing byte, so a sidecar handed to the artifact reader and an envelope handed to the sidecar reader are each refused at the magic rather than misparsed. As in the envelope, the total length is derived from the projected encoding rather than declared after a proportional write, and every declared count is checked against its governed budget before anything proportional to it is reserved.

The header is followed by one canonical manifest and then a stream of length-delimited payloads.

### Canonical manifest and payload stream

**Fact — the manifest opens with the versioned domain tag `tiler.proof-sidecar.manifest.v1\0` and its own `{major, minor}` schema**, then, in this order: the associated artifact's canonical identity bytes; the digest of the exact encoded envelope bytes; the three provenance subjects; the bound input keys and output keys in the artifact's own interface order; the case table; and the sidecar's canonical identity.

Each case row is its stable key and the payload counts it declares in each direction, followed by one descriptor per payload: the payload's canonical ordinal, its exact byte length, and its content digest.

**Fact — payload position is structural rather than referential.** Payloads are framed in one canonical order — cases by stable key, then that case's inputs in interface order, then its expectations in interface order — and a case's descriptors are aligned with that order positionally. There is no payload index a manifest could point at, so the class of forgery in which one payload's descriptor names another payload's bytes does not exist in this format.

**Fact — the three provenance subjects are separately typed because they answer three different staleness questions.** The semantic-graph identity says which mathematical program was evaluated; the numerical-contract identity says under which contract its result is normative; the reference-implementation identity says which implementation computed it. A sidecar can be stale against any one while agreeing with the other two. Following ADR 0072, the frozen registry snapshot is deliberately absent: a provider that was available and never reached does not change what the program computes, so recording it would let an unused provider invalidate a still-correct expectation.

**Fact — the semantic subject is supplied by the producer and compared, not derived.** The risk the check exists to catch is a producer that reference-evaluated a different program from the one it compiled; deriving the subject from the artifact would make the check tautological. A mismatch is a build failure. The other two subjects are opaque bytes this crate compares and encodes and never re-derives, because it is not the authority for either.

**Fact — payloads are bit-preserving and are never interpreted as numbers.** A signalling NaN, a quiet NaN, a negative zero, and a subnormal all survive the container unchanged, which is the only reason a bitwise readback comparison means anything. A container that parsed floats would be free to canonicalize the first into the second and the comparison would then pass against the wrong value.

**Fact — a well-formed but non-canonical encoding is refused rather than normalized.** Named checks reject an out-of-order or repeated case key or interface key, a non-canonical payload ordinal, and a case whose payloads disagree with the container's own bound interface. The reader then re-derives the canonical identity from the decoded content, requires it to equal the carried one, and re-encodes the whole container and requires byte equality. One sidecar therefore has exactly one byte identity, and the identity a reader reports is always the re-derived one rather than the carried one.

### The sidecar's four governed domains

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

These four, the envelope's seven, and the artifact program's seven identity and key domains are the eighteen the union no-prefix obligation under "The governed digest" covers. Two of the four here are framing tags rather than digest arguments, and they are governed for the reason that section states.

### Association is a decision, not a default

**Fact.** A decoded sidecar is fully validated evidence about *nothing* until it is bound. Two checks establish the same association and differ in what they re-prove.

- **`bind_to_envelope(&[u8])`** is for a consumer holding only bytes. It re-derives the envelope digest over the exact bytes supplied, decodes them through the artifact codec, and compares the re-derived artifact identity with the recorded one. Nothing is taken on the producer's word: both values are computed from the caller's own bytes. The digest check runs first, because it is the cheapest and it is the one failure that distinguishes damaged bytes from the wrong artifact entirely.
- **`bind_to_artifact(&VerifiedArtifactProgram)`** is for a consumer holding the program it compiled. It compares the same artifact identity and additionally re-proves every structural obligation locally: that the sidecar binds exactly the artifact's declared inputs and outputs in the artifact's own interface order, that each payload is a whole number of elements of its declared shape, and that all cases agree on each entry's byte length.

**Inference — the second is not a stronger *association*.** Both prove the same artifact identity, which already folds the ordered named interface. The difference is that the second re-proves the obligations rather than inheriting them through an identity comparison, which is what a reader wants when the sidecar was written by an older producer than itself.

**Fact — the obligation has one implementation.** `verify_cases` is called by the builder's terminal and by `bind_to_artifact`. A producer-side copy and a consumer-side copy would agree today, drift later, and each half would still pass its own tests.

**Fact — under the dispatch-record decision, a cold consumer reaches only the weaker check.** A decoded envelope is a dispatch record and never rebuilds a `VerifiedKernelProgram`, so a process that did not compile the artifact cannot obtain a `VerifiedArtifactProgram` and therefore cannot call `bind_to_artifact`. That is not a gap in the sidecar: `bind_to_envelope` establishes the same association from bytes alone, and the obligations `bind_to_artifact` re-proves were already proven by the producer and are folded into the artifact identity both checks compare.

### Governed budgets

**Fact.** Every bound is checked with exact arithmetic before any allocation proportional to it, in both directions. The producer projects the encoded identity, manifest, framed-payload stream, and complete sidecar with checked addition and refuses before cloning a payload, hashing, reserving, or appending. The reader refuses a declared count before reserving for it, and refuses a carried identity length before copying it. A size that is not representable on the host is `Unrepresentable` rather than a wrapped or saturated length. The bounds are 256 MiB per complete encoding, 8 MiB per manifest, 8 MiB per derived identity, 1,024 bytes per received provenance subject, 4 KiB per encoded text run, 256 proof cases, 256 UTF-8 bytes per stable case key, and 4,096 named interface entries per direction. One case payload has no separate size policy: it is admitted when the complete sidecar stays inside the 256 MiB container. The framed payload *count* is *derived* from the case and interface bounds rather than declared, so the framing bound and the structural bounds cannot disagree.

**Fact — the interface bound deliberately equals the artifact model's own.** A sidecar binds one payload per declared entry, so a looser bound here would admit a container no artifact could ever associate with.

**Fact — no storage width is derived, and the omission is deliberate.** A payload is checked to be a whole, nonzero number of the declared shape's elements; the absolute byte width of a governed element type is a backend fact this crate does not own, and inventing one would assert a byte count no verifier examined. Divisibility plus cross-case length agreement catches the same class of producer error without the invention.

### Rejection vocabulary

**Fact.** Failure is typed, non-erasing, and never partially validated. Construction, encoding and decoding, and association each have their own vocabulary, and `ProofCodecError::classification` is a total map from every reader rejection onto five classes a consumer can act on — `Malformed`, `IntegrityFailure`, `Unsupported`, `Invalid`, `Limit` — so an out-of-crate reader never enumerates the `#[non_exhaustive]` variant set. Collapsing the classes would make a version skew look like corruption.

**Fact — a major version step is refused outright rather than read on a best effort**, for the framing format, the canonical encoding profile, and the manifest schema alike; a minor version this build predates is `Unsupported` rather than ignored. This is the same lockstep posture the envelope takes.

### Maturity, stated apart

The container, its canonical form, its integrity validation, its two association checks, and its facade are **implemented and tested**, including against re-sealed forgeries — the adversarial cases build a structurally invalid sidecar and then encode it, which stamps a correct manifest digest, correct payload digests, and a correct identity for whatever it now says, and require the reader to refuse it by name.

**Reserved and not implemented:** case grouping, per-case tolerance, any comparison policy other than bitwise equality, and execution ordering. Bitwise equality is the only comparison the numerical contract admits and the only one a container that never interprets its payloads can honestly support.

**Not a capability of this container at all:** authenticity, as recorded above. Broadening to it would require an external trust anchor and a key-management contract, neither of which exists in this workspace, and would not be a change to this format's digests.

**Measurement — a producer and consumer now exercise the sidecar end to end.** `prototype-metal-aot-slice` generates proof cases from the real reference-evaluated compilation path, and `prototype-metal-runtime-proof` binds the decoded sidecar to the carried artifact and bit-compares all 30 retained cases after exact command-buffer success. That is a delivered evidence path for one Apple M4 Max corpus, not authenticity, production runtime support, or a portable numerical guarantee.

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
byte range, all folded into `tiler.kernel-program.v11` program identity. This
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

- stable plan-value identity, optional semantic component role, and Metal buffer index;
- buffer, metadata block, or scalar role;
- physical storage scalar, complete storage encoding, and kernel access type;
- read, write, or read/write access;
- address space and required alignment;
- alias constraints and separately evaluated accessible byte offset and extent;
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

The conformance evidence that validation produces is **execution-scoped**, and
the split follows the identity boundary above rather than cutting across it. The
validator's own schema key and revision are static and identity-bearing, so a
proof taken under one revision cannot authorize a subject under another. The
bytes it read, the value version, and the coherence epoch under which it read
them are not: they belong to one execution and enter the evidence, never static
artifact identity. Nothing an artifact carries selects a validator today, so no
artifact field records one — a field a producer cannot fill is a placeholder,
and the first artifact-side consumer is the enforcement plan, whose own ticket
owns introducing it and stepping the identity domain that would require.

**Fact — `BindingSpec` deliberately has only the transport `kind`.** The artifact builder derives the target, component role, physical storage scalar, complete storage encoding, kernel access type, access mode, address space, alignment, and accessible byte window from the verified bound program. Reintroducing any of those as caller-declared ABI facts would create two authorities for one executable contract and permit a producer to package a binding that disagrees with the program it names.

Every metadata field states its `AbiExpr` source, byte offset, scalar type,
size, alignment, and encoding. Host packing and MSL declarations are generated
from the same layout; Rust `repr(C)` is not the cross-language contract.
Boolean representation and inline-bytes versus constant-buffer transport are
explicit.

The initial buffer convention is:

- evaluate each binding's accessible byte offset and extent before routing commits and reject overflow in their checked sum;
- require the backing allocation to span through the evaluated end of that window;
- bind the Metal allocation buffer at the evaluated accessible byte offset;
- pass logical `start_element` as typed metadata;
- physical address derivation composes each logical tensor access with the
  selected `BufferView`, adds `start_element` exactly once, and produces an
  offset relative to the bound byte window;
- metadata strides are measured in elements;
- validate the derived range against the binding's accessible extent.

There is no untyped integer “offset”: `accessible_offset` is a byte-count expression and `start_element` is element-count metadata. The loader evaluates and publishes the former on `RoutedBinding`, the backend applies it exactly once at the binding call, and the kernel applies the latter exactly once in logical address derivation. Negative-stride views are initially unsupported.

**Fact — the strict-affine u4 fixture proves this contract structurally without claiming execution.** The interface exposes the logical encoded value as ordered codes, scale, and zero-point components; the code component is stored as `U8` with `PackedU4LsbZeroTail` and accessed as `U8`, the scalar scale is stored and accessed as `F32`, the scalar zero point is stored and accessed as `U8`, and the output is unpacked `F32`. The structured kernel extracts packed nibbles, widens code and zero point through `I32`, subtracts there, converts the difference to `F32`, and multiplies by scale. Component order, role, storage, access type, encoding, and binding target all survive neutral encode/decode and participate in identity.

**Fact — safe execution remains narrower than structural packaging.** Before `RoutingCommit`, a runtime still has to establish physical canonicality plus the support and preparation needed to enforce logical conformance, and the descriptor-only target-neutral payload can perform neither. The four unused tail bits after the final packed nibble, which `PackedTailRule::Zero` requires to be zero, are not part of the logical value and are deliberately unreachable from its scan; they remain the physical representation owner's, and no reachable code discharges them yet. The selected route's unresolved *logical* conformance — codes and zero point inside the inclusive u4 domain and a positive **normal** scale, over the exact ordered roles the type declares — begins after `RoutingCommit` at `EnforcementCommit`, using the contract `tiler_ir::semantic::check_bound_value` derives against the prepared logical view and before the selected plan's first real consumer. Mechanical Metal emission also does not make the artifact executable on the measured Apple profile: `require_declared_realization` refuses it as `MetalEmitError::UnrealizableNumericalObligation { gap: MetalNumericalGap::SubnormalFlushInArithmetic }`, because the strict contract requires subnormal-preserving f32 arithmetic and the measured Metal profile flushes it in every admitted math mode. That typed refusal is correctness, not a fallback license to weaken the contract.

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

**Fact — the implemented device-free loader now exposes this preparation boundary without acquiring device APIs, in two stages ordered by when their facts become true.** `DecodedProgram::preflight` remains the path for routes with neither unanswered deferred predicates nor live-device route requirements, and refuses either; it reports the route-requirement gap first, because a host short of both is short of a bound device before it is short of a prepared pipeline. `DecodedProgram::prepare` instead returns a non-duplicable `LiveDeviceQualification` carrying the routed entries and every `LiveDeviceRequest`. Consuming `resolve_live_device_requirements` passes each row to the host once and yields a `RoutePreparation`; that in turn carries every `TargetPropertyRequest`, each bound to its exact execution-order entry and complete `PreparedEntryTargetRequirement`, and consuming `resolve_target_properties` yields the same single-use `Preflight` only when every directional relation holds. A route declaring no requirement still passes through the first stage, which is what makes the check unskippable rather than conditional. Allocation, command encoding, submission, and every other irreversible program action remain after `Preflight::commit`. This is an implemented artifact/runtime route, not evidence that the retained Metal prototype has exercised it on hardware.

**Fact — the neutral runtime keeps the comparison and refuses what nothing decided.** A host answers a live-device row with a measured `Quantity`, a `Feature` verdict from the owning adapter, or `Unrecognized`; the loader applies the dimension's own relation itself, so an adapter cannot reverse or widen a comparison on the way to an answer, and an answer whose shape disagrees with the row's kind is refused rather than coerced. `Unrecognized` refuses the route: a requirement nothing evaluated has not been met. A row owned by a backend the host did not state is refused earlier still, without consulting any adapter, because the host's own declaration decides it. Every refusal names the exact subject — the dimension, or owner, key, and version — so a host can tell a missing GPU capability from an unmet quantity.

**Fact — a prepared-entry property is the same split, with no numeric sentinel.** `RuntimeAdapter::observe_prepared_entry` returns a `PreparedEntryObservation` of `Quantity(u64)` or `Unrecognized`. An adapter exact-matches provider namespace, name, revision, and property key before reading a pipeline quantity; unknown ownership is `Unrecognized` and the loader classifies it as `UnownedPreparedEntryProperty`, distinct from a measured quantity that misses its relation. The observation type is an accepted public surface as of 2026-08-13 under [`accept-the-prepared-entry-observation-surface`](../tickets/accept-the-prepared-entry-observation-surface.md). There is no compatibility method that maps an unknown property to a number.

`RoutingCommit` occurs only after route-sensitive launch preflight and final
variant selection. Compatibility/capability rejection may route before it;
artifact integrity, schema/ABI inconsistency, dishonest providers, systemic
runtime errors, allocation failure, and all post-commit failures close with an
error.

`EnforcementCommit` occurs when execution of the chosen unresolved semantic
validation begins, including a host scan. No variant or fallback may execute
after it. For a direct encoded input, it precedes result allocation and the
selected alternative's exact first real consumer; success authorizes that
consumer and no other plan's stage. `PublicationCommit` occurs only after a successful witness and makes
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

Those resolved component versions identify the offline compiler and only the offline compiler. Apple's runtime source compiler is a separately versioned build belonging to the execution environment — the [host process and OS](glossary.md#backend-device-and-execution-context-vocabulary) a kernel happens to run in, not the host's stated device-free `ExecutionEnvironment` and not the recorded measurement environment — rather than to the artifact, so no artifact identity can name it, and widening this list would not change that. The [Metal backend](backends/metal.md) contract records the measurement, the bounded cross-path agreement that accompanies it, and why Tiler's ahead-of-time exclusion is what keeps this provenance complete for every kernel Tiler compiles.

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

ADR 0074 convention 2 makes a canonical identity an opaque newtype over its exact canonical byte encoding, with short digests presentation-only, and every layered identity the workspace derives is built that way: the semantic graph, index region, scheduled region, kernel program, and artifact program identities are canonical bytes compared byte for byte, never hashes. Every site that hashes is envelope framing, and each is specified verbatim under "The governed digest" above, over a NUL-terminated `tiler.artifact-envelope.*` constant. How many there are is that section's to state and `tiler_artifact::domains::GovernedDomain`'s to own: it enumerates every governed domain this crate admits, `DomainContainer::ENVELOPE` declares the envelope's share, and `each_container_admits_the_number_of_domains_the_contract_records` fails naming this document when the two disagree. They are not restated here: this block previously carried its own spellings of three of them, they did not match the crate constants, and only the caveat calling every separator in the block illustrative kept that from being an error. One authority for a governed constant is the point.

**Fact — `identity_digest` joined that list at manifest schema 15.0 and does not weaken the sentence above it.** Every one of the five layered identities is still canonical bytes and is still compared byte for byte; `CanonicalArtifactProgramIdentity` is unchanged, and `DecodedArtifact::identity` still returns the decoder's re-derivation of it in full. What became a digest is the manifest's *declaration* of the artifact identity — a producer's statement to its own decoder about the identity it believes it stamped, which the decoder answers by deriving the identity itself and digesting that. So this mints no second identity value for any layer: the digest has no type, no accessor, is never held or compared by anything, and cannot be lifted back out. That is the distinction the decision below turns on, and it is why this is envelope framing like the envelope's other digest arguments rather than a layered digest.

**Corrected 2026-08-08 by [`reconcile-the-artifact-abis-four-hashing-sites-with-the-fifth-it-names`](../tickets/reconcile-the-artifact-abis-four-hashing-sites-with-the-fifth-it-names.md) — the two paragraphs above each counted the envelope's hashing sites, each undercounted it by the carried payload's identity digest, and both are substituted rather than dated beside because neither was true at the commit that wrote it.** They read "Hashing occurs at exactly four sites, all of them envelope framing, and all four are specified verbatim under "The governed digest" above — `manifest_digest`, `section_digest`, `envelope_digest`, and `identity_digest`, each over a NUL-terminated `tiler.artifact-envelope.*` constant" and, of the identity digest, "this is envelope framing like the other three rather than a layered digest". **Both retired strings are quoted verbatim so they stay greppable, and a later hit for either lands in this note rather than in a live claim.** The clause entered as "Hashing occurs at exactly three sites" at `568645b5` on 2026-07-24, where it was exact — `git grep -l 'PAYLOAD_IDENTITY_DOMAIN' 568645b5 -- crates/` returns nothing, so three domains were all the envelope hashed under. The payload identity digest landed seventeen minutes later at `03a86ac3` and the count did not move with it. `09d1666a` then stepped it three → four for `identity_digest` alone, on a tree where `git grep -l 'PAYLOAD_IDENTITY_DOMAIN' 09d1666a -- crates/` already returns `crates/tiler-artifact/src/program/codec/payload.rs`, so "four" was an undercount on the day it was written and so was the "other three" the same commit added beside it. `96dfe333` corrected the governed-digest section above to the envelope's seven governed domains, of which five are digest arguments, and did not reach this block.

**The characterization survived the count, which is why only the number is retired.** `payload_identity` in `crates/tiler-artifact/src/program/codec/payload.rs` hashes at `.digest(PAYLOAD_IDENTITY_DOMAIN, metadata)` over a payload's exact canonical metadata bytes, and that digest is the content address the manifest's payload descriptor carries and `PayloadIdentityMismatch` re-proves — envelope framing exactly as a section digest is, and the identity of no layer. The five layered identities this block names are unchanged and are still canonical bytes compared byte for byte, so the fifth site strengthens the convention-2 claim rather than weakening it, and this contract now states no hashing-site number in either paragraph. The paragraph below returns to the five layered digests the Decision above removed.

**Why specifying them was rejected rather than deferred.** A compact key's only value is being shorter than the canonical bytes, which means trading collision-freedom for width. Applied to a subject that *already* has a canonical-byte identity, that produces a second identity authority over one subject — the shape ADR 0082 names, whose agreement with the real identity could only ever be argued and never checked. Nothing in the tree computes or consumes such a key, no crate exposes one on any of the five types, and the expansion cache keys on a `ComposedSubject` of length-prefixed canonical byte runs rather than on layered digests. Writing a more precise promise that nothing implements would have deepened the divergence this block existed to flag, not closed it.

**What would reopen it.** A consumer that genuinely needs a bounded-width cross-reference to a layer — an external index over layers is the plausible one, and none exists. Such a proposal has to answer the second-authority problem first: which of the two values *is* the identity, what happens when they disagree, and what checks that they do not. Until then the answer is that a consumer wanting a bounded value for presentation uses a presentation label, which is explicitly not an identity.

Section digests are stored only in manifest section descriptors. The manifest
digest is stored only in the framing header and covers the exact manifest bytes,
which contain no `manifest_digest` or `envelope_digest` field. The identity
digest is stored only at the manifest's end and covers the artifact identity,
which is not itself carried anywhere in the envelope — so it too is a digest over
bytes outside the run that holds it. `EnvelopeDigest`
is externally derived and never stored inside the envelope it covers. A layer's identity is
carried as its canonical bytes wherever it is carried at all, so a cross-reference
to one is those bytes and not a digest of them. No field is hashed through a
zeroing convention or recursive definition.

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

**Fact — the implemented reader is exactly that lockstep reader.** It supports only the versions and features this build writes: a major mismatch, a minor beyond what this build implements, an unrecognized digest algorithm, or a required feature it cannot supply is a typed rejection rather than a best-effort read. The optional-field and version-skew rules are consequently still undecided and still unimplemented, which is consistent with the sentence above rather than a gap beneath it. The reader discharges stage 1, the manifest and section half of stage 2, the binding-reference half of stage 6, and the static half of stage 8 — expression typing, availability phase, the guard's predicate type, the exact-entry scope and directional validity of each prepared target requirement, and the launch formula's agreement with the entry's proven resource requirements. Carried Metal payload bytes are implemented, but the neutral codec does not itself provide the backend parser, live device, prepared pipeline, or bound runtime environment needed to discharge the remaining stages; the bounded runtime proof supplies those outside the codec for its measured corpus, while hardware exercise of the exact-entry deferred route remains unclaimed here.

## Traceability

This document owns the neutral artifact envelope and Metal ABI profile. It does
not own backend scheduling or consumer storage. Its governing decisions and
supporting research are declared in frontmatter; unresolved serialization and
compatibility work remains explicit above.
