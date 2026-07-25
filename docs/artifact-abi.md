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

**Status:** accepted research contract; shared IR ownership established, a bounded neutral envelope codec implemented behind an unaccepted crate-private facade

The private compiler proof constructs provisional program portfolios and artifact-construction inputs. ADRs 0070 and 0071 now assign authoritative target-neutral executable meaning and checked construction to shared `tiler-ir` representations; the proof-specific structs remain private until replaced in dependency order.

**Fact — canonical envelope serialization, canonical form, and integrity validation are implemented; the public artifact API and backend payloads are not.** `crates/tiler-artifact/src/program/codec/` encodes, decodes, and re-validates the envelope this document specifies, in the bounded lockstep profile recorded under "Implemented envelope profile" below. Every item in that module is `pub(crate)` behind a private `mod` under ADR 0074 convention 7, so no crate outside `tiler-artifact` can encode or decode an artifact and no consumer surface has been accepted; promoting it is Tom's decision under ADR 0075. The envelope now carries a backend payload's compilation subject and its object bytes, but nothing yet fills that shape from a real emission and a real compilation; and a decoded envelope still cannot reconstruct the shared-IR programs it names. The accurate statement is that a bounded lockstep codec exists behind an unaccepted facade, not that the artifact format is available.

## Ownership boundary

This document owns envelope framing, wire DTOs and encoding, compatibility,
runtime fact binding, routing commit, digests, failure classification, and
backend payload mappings. The IR contract owns program/portfolio meaning,
canonical identity, ABI-expression semantics, and authoritative verification;
adapters own device-specific loading, binding, and execution. A decoder must
reconstruct shared IR through its checked builders and cannot manufacture a
verified value or retain a second editable authority. The implemented profile
satisfies the second half and not the first, for a structural reason recorded
under "Implemented envelope profile" below.

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

Everything in this section is a fact about `crates/tiler-artifact/src/program/codec/` as of commit `5c51da7`. It records what one build writes and reads. It does not widen the normative contract above, and where the two differ the difference is named rather than resolved by rewriting either side.

### Maturity of the implementation

**Fact — the surface is crate-private and its facade is unaccepted.** `codec` is a private `mod` of `tiler_artifact::program`, every item it exports is `pub(crate)`, and the module carries the `#![allow(dead_code, reason = "…")]` that ADR 0074 convention 7 prescribes for a landed authority whose facade has not been reviewed. Encoding, decoding, `ArtifactEnvelope`, the typed rejection vocabulary, and the governed constants are all unreachable from outside the crate. Promoting the module or any of its types from `pub(crate)` to `pub` is named on ADR 0075's always-ask list and requires Tom's review before merge.

**What a consumer can do today.** Build a `VerifiedArtifactProgram` through `tiler_artifact::program` — itself a reviewed *draft* boundary rather than an accepted facade — and read its canonical identity, which is now derived from the canonical envelope and is therefore exactly the identity a decoder re-derives from bytes.

**What a consumer cannot do today.** Obtain an artifact's bytes, decode bytes into an artifact, name an envelope digest, observe any typed codec rejection, or carry a backend payload — the entry point that carries one is itself `pub(crate)`. There is no serialization API and no exposed file or embedding format.

Four maturity claims stay distinct here. The framing, canonical manifest, section framing, required-feature mechanism, and rejection vocabulary are **implemented**. The section-purpose vocabulary and the carried-payload entry point are **implemented and tested against synthetic content**, and are **reservations** in the narrower sense that no backend fills them from a real emission and a real compilation yet. A `pub` codec facade is an **architectural seam** with no accepted shape. The properties labelled Measurement below are **tested guarantees over the named fixtures**, not universal claims about every artifact.

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

**Fact — every variable-length run carries a fixed-width `u64` length before its content**, so no concatenation of fields is ambiguous, and **every encoded enumeration is written through the one governed tag table its vocabulary owns**, never through a Rust discriminant, so inserting a variant cannot silently renumber a value already on disk. Each table is a forward and inverse pair kept in one place and pinned by an exhaustive round-trip test.

**Fact — a well-formed but non-canonical encoding is refused rather than normalized.** Named checks reject an out-of-order or repeated feature, interface key, provider, payload, expression, deferred predicate, launch precondition, executable entry, or section. Because this reader understands every field, the decoder then re-encodes the validated envelope and requires byte equality, rejecting any residual non-canonical spelling as `NonCanonicalManifest`. One artifact therefore has exactly one byte identity.

### What is meaning and what is canonical

**Fact.** Variant order is routing priority, named-interface order is the semantic interface's, and ABI binding order is the kernel signature's; all three are retained. Provider, payload, deferred-predicate, launch-precondition, executable-entry, expression-arena, and section order are replaced by the canonical content order artifact identity already uses. The arena's canonical order is the unique topological order that always emits the smallest available node by canonical content key.

**Measurement.** Declaring the same payloads and providers in reversed order produces byte-identical envelopes, as does minting the same formulas through two different arena assembly orders.

### Identity is derived from the canonical envelope

**Fact — there is one identity encoder and its subject is the envelope.** `encode_identity` is a function of `ArtifactEnvelope`, and the checked builder's terminal projects the verified draft into its envelope before deriving the identity. A decoder re-derives the identity from decoded content through that same function and compares it with the identity the manifest carries, rejecting a mismatch as `ArtifactIdentityMismatch`. There is therefore no second encoder that a decoder would have to agree with by inspection.

**Inference — equal identity implies equal envelope bytes, and three closure checks are what make that true.** Identity replaces arena positions, payload positions, and section positions with canonical content keys, so it does not by itself fix the tables those positions index. An envelope carrying an expression no use site reaches, a payload no entry realizes, or a section no variant references would keep the same identity while changing the bytes, giving one artifact two byte identities. The decoder rejects all three. This closure is what lets an envelope digest serve as a cache key.

**Fact — a re-proven obligation reports the artifact model's own cause.** A decoded envelope is checked again against the rules the transactional builder discharged at construction, and each rejection carries the model's own typed cause rather than a codec-local restatement: one variant wraps an insertion-time build error, another wraps a whole-artifact diagnostic. A rejection therefore reads the same whether an artifact was refused at construction or at load.

**Fact — two builder obligations are not decidable from an envelope and are pinned by identity instead.** A binding's accessible byte range must equal the exact byte window its stage access addresses, and an entry's bindings must correspond one-to-one with its kernel's buffer parameters. Neither the byte windows nor the kernel signature travel in this profile, so a decoder cannot recompute them. Both are folded into the artifact's canonical identity — through the binding's expression content key and the entry's stage key — and the identity is re-derived and compared, so a forged envelope can restate them only by becoming a different artifact. Carrying the byte windows so the check could run locally was considered and rejected: the window is a value only the program establishes, so a carried copy would prove agreement between two producer-supplied fields rather than agreement with the plan.

### Required features

**Fact — the required-feature set is derived from content and never declared by a producer**, so it cannot understate what a reader must implement; a declared set the content does not imply rejects as `DeclaredFeatureMismatch`. This build derives four governed keys:

| Governed feature key | Derived when | This reader supports it |
| --- | --- | --- |
| `tiler.artifact.feature.multi-variant-routing` | the portfolio carries more than one variant | yes |
| `tiler.artifact.feature.deferred-predicates` | any variant defers a feasibility predicate | yes |
| `tiler.artifact.feature.launch-preconditions` | any entry declares a launch precondition | yes |
| `tiler.artifact.feature.multi-stage-program` | any variant dispatches more than one stage | no |

**Fact — this build emits `tiler.artifact.feature.multi-stage-program` and refuses to read it.** The neutral program section carries a program's canonical identity rather than its dependency graph, so a reader cannot recover the order in which two stages must run; entries are ordered by canonical stage key, which is identity's order and not execution order. Emitting the feature and rejecting it on read as `UnsupportedRequiredFeature` is the fail-closed form of that gap, and treating declaration order as execution order would be the silent one. The gap between the set a producer emits and the set a reader implements is the mechanism working rather than a defect.

### Sections

**Fact — the section vocabulary has three governed purposes in this build**: the canonical kernel-program identity of one packaged variant, and a carried backend payload's compilation subject and its object bytes. Two variants that package the same program share one section, and two payloads carrying the same object share one section, because content is the address — so sharing is a stated property of these purposes rather than an accident of equal bytes. Sections are ordered canonically by content; duplicates, unreferenced sections, a section identifier that is not its canonical position, and an unrecognized purpose tag are each rejected by name.

**Fact — a section carries canonical identity bytes, not a digest of them.** ADR 0074 convention 2 makes a canonical identity an opaque byte encoding and short digests presentation-only, so the governed bound on one section is the shared IR's own identity budget rather than a digest width.

**Proposal — backend metadata and code sections are `prototype-metal-bundle-assembly`'s versioned extension.** The framing, descriptor derivation, digest checking, and cross-reference closure such a section needs already exist and are exercised; the governed purposes it adds are its own. Whether a bundle's identity is content-addressed over its compilation inputs or over the emitted payload bytes is that ticket's decision, and this contract deliberately leaves the seam open rather than pre-empting it.

### The governed digest

**Fact.** The envelope names its digest algorithm by an explicit header tag and a reader never infers one from a digest width. `0x01` is `tiler.digest.sha-256.v1`, the only admitted value in this build. Three domain separators are governed as fixed NUL-terminated crate constants, and a test proves no admitted domain is a prefix of another, so `H(domain || bytes)` genuinely separates its subjects rather than colliding a longer domain with a shorter one plus leading content:

```text
manifest_digest = H("tiler.artifact-envelope.manifest-digest.v1\0"
                    || exact canonical manifest bytes)
section_digest  = H("tiler.artifact-envelope.section-digest.v1\0"
                    || section purpose tag || section content schema
                    || exact section bytes)
envelope_digest = H("tiler.artifact-envelope.envelope-digest.v1\0"
                    || exact complete envelope bytes)
```

A section descriptor is derived from its section's position and exact bytes at encode time and re-derived and compared at decode time, never stored beside the bytes it describes, so the two cannot disagree in memory. The envelope digest is computed and never stored in band; a test asserts its bytes occur nowhere in the envelope it covers.

**Fact — the section digest binds the section's purpose and content schema, so it is a standalone content address.** The pre-image is the domain separator, the purpose tag, the content schema, and then the exact section bytes; the qualifiers are fixed width and precede the variable-length content, so no length prefix between them is needed for the pre-image to be unambiguous. Binding the purpose is what makes the digest usable *outside* a complete envelope, which is what content-addressing a backend code section does. Inside an envelope the purpose was already bound one level up, by the manifest descriptor that names it and the manifest digest that covers that descriptor; lifted out, a digest over bytes alone would give two sections of different purposes one address.

**Fact — the algorithm implementation is in-crate and its selection remains an open bounded decision.** SHA-256 is implemented in the crate rather than taken from a dependency, pinned by the FIPS 180-4 published vectors and by every padding branch, because adding the workspace's first cryptographic dependency would answer that decision by accident. The wire contract commits only to the governed tag, so replacing the implementation with an audited crate leaves every encoded envelope byte identical. `select-the-governed-artifact-digest-implementation` owns the comparison.

### Deliberate exclusions

**Fact — the frozen registry snapshot never enters the envelope.** ADR 0072 keeps the provenance of providers a plan never used out of packaged artifact identity, and carrying the snapshot here would put it back into the envelope's bytes and therefore into its digest, letting an unused provider invalidate a cache entry. Only the three reached subjects travel: the semantic graph identity, the reached definitions, and the admission provenance. **Measurement.** An artifact built with an additional available-but-unreached provider encodes to identical bytes and an equal envelope digest, while changing a *reached* provider's revision changes the digest.

**Fact — presentation-only declaration order never enters it**, under the ordering rules above. This is what makes the envelope's bytes a function of the artifact's identity rather than of the order a producer happened to declare things in.

**Fact — backend payload bytes never enter it.** A payload is named by governed backend and representation keys, its own schema version, its opaque content digest, and its execution policy, and by nothing a backend would spell: no symbol names, binding indices, platform triples, or language versions occur in the neutral manifest.

**Fact — a reconstructable kernel program is not carried, and the blocker is structural rather than an omission.** `tiler_ir::program::KernelProgramBuilder::new` takes a `&SemanticProgram`, which requires a frozen semantic registry holding live inferencer implementations; neither is representable as bytes. A decoded envelope therefore proves *which* program an artifact names and cannot resurrect it, and a consumer that needs a verified kernel program must hold the one it compiled. This is why the decoder half of the ownership boundary above is not yet met, and the consequence is stated rather than approximated. `carry-reconstructable-kernel-programs-in-the-neutral-envelope` owns deciding what a decoded envelope must reconstruct.

### Governed budgets

**Fact.** Every budget is enforced on both sides by one constant. The encoder checks a projected envelope up front, so a legally built artifact that could not be read back fails to encode rather than producing bytes no reader admits; the decoder checks each count the moment it is read and before anything is allocated for it. The envelope-level bounds are 256 MiB per complete encoding, 64 MiB per manifest, 64 MiB per section, 16 MiB per received opaque identity subject, 4 KiB per encoded text run, 64 required features, 4,096 named interface entries, and 4,096 declared shape rank. Every per-collection bound — variants, entries, bindings, expressions, payloads, providers, deferred predicates, launch preconditions — reuses the artifact model's own constant rather than introducing a second authority for the same limit that would drift from it.

### Rejection vocabulary

**Fact.** Failure is typed and non-erasing. A rejection names the boundary that refused and the subject it refused; a framing, schema, canonical-form, structural, or identity failure is never reinterpreted as a plan-applicability miss; and a rejection never yields a partially validated envelope.

| Boundary | Typed causes |
| --- | --- |
| Framing and integrity | `Truncated`, `TrailingBytes`, `TrailingManifestBytes`, `BadMagic`, `BadManifestDomain`, `TotalLengthMismatch`, `ManifestDigestMismatch`, `SectionDigestMismatch`, `SectionLengthMismatch`, `SectionCountMismatch`, `NonCanonicalSectionId` |
| Schema and feature compatibility | `UnsupportedEnvelopeFormat`, `UnsupportedCanonicalEncoding`, `UnsupportedManifestSchema`, `UnsupportedComponentSchema`, `UnsupportedDigestAlgorithm`, `UnsupportedRequiredFeature` |
| Canonical form | `NonCanonicalOrder`, `DuplicateItem`, `NonCanonicalManifest`, `DeclaredFeatureMismatch`, `UnreferencedSection` |
| Structure and closure | `MissingReference`, `ExpressionOperandOrder`, `ExpressionOperandType`, `ExpressionSelectBranchType`, `UnknownTag`, `InvalidText`, `InvalidGovernedKey`, `InvalidInterfaceKey`, `InvalidProviderIdentity`, `InvalidShape` |
| Governed budgets | `Limit` |
| Re-proven model obligations | `ModelRule`, `ModelObligation` |
| Identity | `ArtifactIdentityMismatch`, `IdentityDerivation` |

Each variant carries the structured data a caller reacts to rather than a message: which collection was out of order, which enumeration presented an unimplemented tag with the rejected tag byte, which table a reference missed with the rejected index, which resource was exhausted with its attempted and permitted quantities.

**Measurement — the adversarial cases build a structurally invalid envelope and then encode it**, which stamps a correct manifest digest, correct section digests, and a correct identity for whatever the forgery now says, and require the decoder to reject it anyway by name. Corrupting bytes and watching a digest reject them proves comparatively little, because a forger recomputes digests.

### Where the implemented profile is narrower than this contract

**Fact.** Four normative statements elsewhere in this document once described a format wider than the one this build writes. Items 1 and 4 have since been closed and are retained here as the record of what closed them; items 2 and 3 are open, each with a stated trigger rather than an open-ended gap.

1. **A section descriptor now carries all four declared fields, and one difference remains.** It is an ordered identifier, a purpose tag, the purpose's required/optional disposition, the purpose's content schema, the exact byte length, and the content digest. The one remaining narrowing is deliberate: the *digest algorithm* is named once in the header rather than per descriptor, because one envelope is digested under one governed algorithm and a per-descriptor spelling would admit an envelope whose sections disagreed about it.

   The disposition and the content schema are properties of the *purpose* rather than of the instance, and are written anyway, because the reader that needs them is precisely the one that does not recognize the purpose and so cannot derive them. A reader that does recognize a purpose owns the answer, and this build therefore requires a descriptor to agree with its own table — a disagreement is a descriptor asserting a schema or a skip permission rather than reporting one, and is rejected by name. Every purpose this build writes is `Required`, and an unrecognized purpose is refused outright, so no skip path exists yet; the field is the mechanism item 2 below will need.

   Adding both fields moved the manifest schema to **2.0** — a major step rather than the minor one it might look like. A minor step would have been wrong, because the reader admits `minor <= implemented` and would have gone on accepting a `1.0` manifest whose descriptors it can no longer parse. A field added inside a fixed-width record is not additive. The envelope format and the canonical encoding profile in the header are unchanged at `{1, 0}`: the manifest's layout moved, not the framing around it.
2. **There are no optional sections, so "unknown optional sections may be skipped only when their schema explicitly permits it" describes no implemented behaviour.** Every unrecognized purpose fails closed. This is the deliberate version-1 posture the envelope research records, and "Loading and validation" below already conditions the skip mechanism on exposing the format outside a lockstep release. It is an explicitly deferred question with that trigger rather than an unrecorded gap.
3. **A decoder does not reconstruct shared IR through its checked builders**, for the structural reason under "Deliberate exclusions". It does satisfy the other two halves of that requirement: nothing in the codec manufactures a verified value, and a decoded envelope is a validated envelope rather than a second editable authority.
4. **The backend payload descriptor carries its compatibility-contract reference.** It carries the backend key, representation key, payload schema, content digest, the target profile the payload's own bytes were built against, and the execution policy. The reference is folded into the payload's canonical key and therefore into artifact identity, so two payloads that agree on every other field but were built against different profiles are two payloads rather than one.

   The field is the payload's contract, not the plan's. A variant's `TargetProfileRef` and `FeasibilityRuleSetRef` are the *plan's* declared target requirements, and the two coincide only while a payload is realized by one variant — which nothing in this model requires, since entries cross-reference payloads by index. Carrying it per payload is what lets a program share one compiled object across variants declaring different profiles and still state what that object was built for; without it a loader would infer the payload's contract from whichever variant it happened to route to, which is the inference this layer exists to forbid.

   The narrower alternative — declaring a payload per-variant by construction — was rejected on a concrete cost rather than on taste: it makes a legitimate program inexpressible. Two variants compiling to the same library could not share the payload under the new rule, and could not declare a second identical descriptor either, because the builder already refuses that as a duplicate.

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
program IR in `tiler-ir`. This artifact contract owns their versioned wire
encoding, runtime fact binding and phase checks, compatibility behavior, and
failure classification; it must not recreate a second editable expression
authority.

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

**Two constructs are named alike below and are not the same thing.** ADR 0074 convention 2 makes a canonical identity an opaque newtype over its exact canonical byte encoding, with short digests presentation-only, and every layered identity the workspace derives is built that way: the semantic graph, index region, scheduled region, kernel program, and artifact program identities are canonical bytes compared byte for byte, never hashes. Hashing occurs at exactly three sites, all of them envelope framing. The first five derivations below are therefore a proposal for deriving a compact key from a subject rather than a description of how those subjects are represented today, and the domain-separator spellings in this block are illustrative — each governed constant is owned by the encoder that derives it, and the three envelope-level constants this build writes are recorded verbatim under "The governed digest" above. `decide-whether-layered-subject-digests-exist-as-hashes` owns closing the question.

```text
semantic_digest = H("tiler-semantic-v1" || canonical semantic bytes)
index_digest = H("tiler-index-v1" || canonical index-structure bytes)
schedule_digest = H("tiler-schedule-v1" || index_digest
                    || canonical schedule-structure bytes)
refinement_digest = H("tiler-refinement-v1" || region occurrence/binding
                      || index_digest || reached definitions
                      || selected providers/evidence)
plan_digest = H("tiler-program-v1" || semantic_digest
                || bound refinements/implementations
                || canonical complete-program bytes)
section_digest[i] = H("tiler-section-v1" || section_type/schema
                      || exact section bytes)
manifest_digest = H("tiler-manifest-v1" || exact canonical manifest bytes)
envelope_digest = H("tiler-envelope-v1" || exact complete envelope bytes)
```

Section digests are stored only in manifest section descriptors. The manifest
digest is stored only in the framing header and covers the exact manifest bytes,
which contain no `manifest_digest` or `envelope_digest` field. `EnvelopeDigest`
is externally derived and never stored inside the envelope it covers. Semantic,
index, schedule, refinement, and plan digests may appear as cross-reference values, but their
canonical subject bytes and domain separators are fixed and independently
validated. No field is hashed through a zeroing convention or recursive
definition.

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
