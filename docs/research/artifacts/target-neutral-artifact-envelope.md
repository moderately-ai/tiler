---
schema: "tiler-doc/v1"
id: "tiler.research.artifacts.target-neutral-envelope"
kind: "research"
title: "Target-neutral artifact and backend payload envelope"
topics: ["artifacts", "abi", "backends", "validation"]
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.artifact-abi", "tiler.contract.metal-backend"]
reproduced_by: ["tiler.spike.artifacts"]
ticket: "artifact-envelope-model"
---

# Target-neutral artifact and backend payload envelope

**Status:** completed research adopted by the artifact contract; serialization implementation remains future work
**Ticket:** `artifact-envelope-model`

## Outcome

Tiler should package an artifact as one self-verifying envelope with two
strictly separated layers:

1. a target-neutral manifest and neutral program sections that describe
   semantic interfaces, complete program portfolios, routing, guards,
   executable-entry contracts, typed feasibility requirements, ABI expressions,
   selected provider identities, and failure boundaries; and
2. typed backend payloads whose governed backend schemas own executable bytes
   and backend-only metadata.

The neutral layer knows that an entry point is implemented by a payload and an
opaque backend entry key. It does not know that Metal uses metallibs, symbol
strings, buffer indices, function-constant indices, MSL versions, Apple
platform/deployment triples, or particular dispatch APIs. Those fields belong
to the Metal payload schema. A future CUDA payload may use cubin/PTX modules,
CUDA function names, parameter layouts, and driver-JIT policy without changing
the neutral envelope schema.

The envelope is a bounded, canonically encoded manifest followed by
length-delimited sections. Every section is described by stable type/schema,
exact byte length, and a digest over its exact bytes. The header integrity-checks
the exact canonical manifest bytes. An external `EnvelopeDigest` hashes the
complete encoded envelope and has no recursive in-band field. There are no
unhashed executable or required metadata sections in the initial format.

Compatibility is explicit per schema and required feature, never inferred from
the producer crate version, compiler version, backend name, or successful
parsing. Integrity, structural validity, neutral program validity, backend
payload validity, target compatibility, live applicability, and prepared-entry
feasibility are separate validation boundaries.

## Primary-source facts

These facts are precedents, not imported specifications.

- OCI image descriptors bind an opaque blob to a media type, byte size, and
  content digest. Manifests can list multiple typed blobs while the blobs remain
  independently content-addressed. This is useful precedent for typed section
  descriptors, but Tiler needs stronger canonical-manifest and executable
  cross-reference verification. See the OCI
  [descriptor](https://github.com/opencontainers/image-spec/blob/main/descriptor.md)
  and [manifest](https://github.com/opencontainers/image-spec/blob/main/manifest.md)
  specifications.
- LLVM's offload binary and Clang offload bundler package multiple device code
  objects, identify the runtime/target of each entry, and apply target-specific
  compatibility rules when extracting. This validates separating a neutral
  container from backend image selection. Their metadata is not rich enough to
  carry Tiler's tensor program, ABI, numerical, guard, and feasibility
  contracts. See
  [`llvm-offload-binary`](https://llvm.org/docs/CommandGuide/llvm-offload-binary.html)
  and the [Clang Offload Bundler](https://clang.llvm.org/docs/ClangOffloadBundler.html).
- WebAssembly binary sections have explicit IDs and byte lengths; a size
  mismatch makes a known section malformed, while custom sections can be
  skipped. This supports bounded framing and independent required/optional
  semantics. Tiler must additionally digest all sections and reject unknown
  required meanings. See the WebAssembly
  [binary module format](https://webassembly.github.io/spec/core/binary/modules.html).
- RFC 8949 defines deterministic CBOR encodings, requires an application
  protocol to resolve representational choices, and calls out duplicate-map-key
  ambiguity. Merely choosing CBOR would not define Tiler's canonical bytes;
  Tiler would still need a narrowed profile, schema ordering, duplicate
  rejection, and normalization rules. See
  [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html), especially sections
  4.2 and 5.6.
- Protocol Buffers supports schema evolution through stable field numbers and
  unknown fields, but its documentation explicitly says deterministic
  serialization is not canonical across languages, schema changes, libraries,
  or builds and should not be used directly for persistent fingerprints. This
  is direct evidence against hashing ordinary serializer output without a
  Tiler-owned canonicalization contract. See
  [Proto Serialization Is Not Canonical](https://protobuf.dev/programming-guides/serialization-not-canonical/)
  and the [proto3 evolution rules](https://protobuf.dev/programming-guides/proto3/#updating).
- FlatBuffers permits additive schema evolution and supplies a verifier, while
  requiring field-order or explicit-ID discipline and deprecation rather than
  field removal. This supports independently versioned component schemas and
  bounded verification, but does not by itself define semantic compatibility
  or Tiler's canonical artifact identity. See the official
  [FlatBuffers evolution guide](https://flatbuffers.dev/evolution/).
- The Update Framework records target byte lengths and hashes separately from
  opaque target contents and requires clients to verify them before use. Tiler
  is not defining a signature/update framework here, but adopts the simpler
  lesson that length limits are enforced before allocation and digest success
  precedes interpretation. See the
  [TUF specification](https://theupdateframework.github.io/specification/latest/).
- Reproducible Builds documents timestamps and archive metadata as common
  nondeterministic inputs. Tiler should omit wall-clock time, paths, random
  identifiers, host iteration order, and filesystem metadata from canonical
  output. Relevant toolchain identity is represented by normalized content and
  fingerprints instead. See its
  [timestamp guidance](https://reproducible-builds.org/docs/timestamps/).

## Facts, inferences, and proposals

### Facts already accepted by Tiler

- `KernelProgram` is a target-specific, consumer-neutral executable dependency
  DAG with host preflight, materialized values, buffers, named outputs, and a
  `RoutingCommit` boundary.
- A `ProgramPortfolio` routes among complete semantically and numerically
  equivalent programs, not among isolated kernels.
- Target feasibility is typed and monotonically refined through
  `CompileProfile`, `ArtifactEvidence`, `LiveDevicePreflight`,
  `PreparedKernelPreflight`, and `LaunchPreflight`.
- `Unknown` feasibility cannot enter an artifact. Every deferred check has an
  admitted query path before `RoutingCommit` and complete fallback/alternate
  coverage.
- Selected artifact identity includes reached semantic authorities and selected
  output-affecting capability-provider revisions. Unused registry providers do
  not invalidate an artifact.
- Live-device and prepared-entry fact values scope runtime caches and routing
  observations; they are not portable artifact content identity.

### Inferences

1. The neutral envelope must carry complete program semantics and references,
   not just a list of backend symbols. Otherwise it cannot validate routing,
   fallback, ABI, or deferred-feasibility coverage before execution.
2. A backend-independent executable entry can define logical/physical binding
   requirements and checked launch expressions without defining the API slot
   that transports them. Backend slot mappings must remain in the payload.
3. Producer/toolchain provenance and runtime compatibility are different.
   Recording that Xcode produced a section does not prove that a live device
   may load it; the Metal payload must separately declare its compatibility and
   translation policy.
4. Schema parseability is weaker than executable compatibility. A reader may
   understand a newer optional field yet still lack a required capability,
   backend payload parser, feasibility provider, or runtime execution profile.
5. One digest should not be overloaded as semantic equivalence, executable-plan
   equivalence, payload equality, and exact-container equality. Layered digests
   make cache and diagnostic decisions explainable.

## Envelope hierarchy

```text
ArtifactEnvelope
  fixed framing header
  canonical NeutralManifest
  section 0..N
    canonical NeutralProgram section(s)
    canonical BackendMetadata section(s)
    opaque BackendCode section(s)
    optional governed evidence/debug section(s)

NeutralManifest
  schema/feature requirements
  program portfolios
  executable entries
  backend payload descriptors
  exact section descriptors
  selected compilation provenance
```

An envelope may contain several complete programs and several payloads. That
supports one macro-local portfolio and future multi-backend/fat artifacts, but
does not require either. The initial Metal invocation normally emits one Metal
payload containing all entry points reached by its packaged portfolios.

Every collection that has set/map meaning is sorted by its canonical key and
contains no duplicate. Collections whose order is semantic, including program
outputs, operands, ABI fields, priority rules, and canonical topological order,
retain that order. Display labels and source spans never act as keys.

## Proposed target-neutral schema

The records below are conceptual. Stable newtypes and governed keys replace
plain strings in an implementation.

```text
ArtifactEnvelopeHeader {
  magic,
  envelope_format_version,
  canonical_encoding_key_and_version,
  total_byte_length,
  manifest_byte_length,
  section_count,
  manifest_digest_algorithm,
  manifest_digest,
}

NeutralManifest {
  neutral_manifest_schema,
  required_features,
  component_schemas,
  programs: [ArtifactProgram],
  entries: [ExecutableEntry],
  payloads: [BackendPayloadDescriptor],
  sections: [SectionDescriptor],
  selected_compilation_provenance,
}

ArtifactProgram {
  program_id,
  semantic_contract: SemanticContractRef,
  numerical_contract_digest,
  semantic_root_bindings,
  semantic_constraints,
  variants: [ArtifactPlanVariant],
  routing_policy,
  external_fallback_contract?,
  selected_provider_identities,
  neutral_program_section,
  kernel_program_digest,
}

ArtifactPlanVariant {
  plan_id,
  kernel_program_ref,
  applicability_guards,
  target_profile_ref,
  feasibility_rule_set_ref,
  target_requirements,
  deferred_checks_grouped_by_phase,
  execution_profile,
  entry_points,
  routing_commit_contract,
  validation_and_publication_contract,
}

ExecutableEntry {
  entry_id,
  owning_program_and_variants,
  scheduled_region_digest,
  abi_contract,
  launch_contract,
  specialization_contract,
  exact_resource_requirements,
  numerical_realization_refs,
  implementation: BackendEntryRef | TypedOpaqueEntryRef,
}

BackendEntryRef {
  payload_id,
  backend_entry_key,
}
```

The full canonical `KernelProgram` is carried in a neutral program section, not
reconstructed from abbreviated manifest fields. The manifest duplicates only
the fields necessary for bounded discovery and cross-reference roots. Any
duplicated value must match the canonical program section or validation fails.

### ABI contract

The neutral entry ABI describes meaning and bytes without naming an API slot:

```text
EntryAbiContract {
  abi_schema,
  bindings: [EntryBinding],
  metadata_layouts,
  scalar_encoding,
  alias_and_access_contract,
}

EntryBinding {
  entry_binding_id,
  plan_value_id,
  component_role,
  kind: Buffer | MetadataBlock | Scalar | ErrorRecord,
  storage_type_and_encoding,
  access: Read | Write | ReadWrite,
  logical_address_space,
  required_alignment,
  accessible_range_expr,
}
```

The ABI records ordered component roles for composite/quantized values; a
backend never infers component meaning from slot order. Byte offsets, scalar
width/signedness, Boolean representation, alignment, and encoding are explicit
when the generated executable observes them. The backend payload maps each
`EntryBindingId` to its native transport location.

An ABI contract can therefore be shared by two backend payload realizations
without pretending that Metal buffer index 3 and a CUDA parameter offset are
the same concept.

### Launch and specialization

The neutral layer owns the checked expression DAG and the values it computes:

```text
LaunchContract {
  launch_schema,
  host_expressions,
  logical_launch_values,
  zero_work_policy,
  launch_preconditions,
}

SpecializationContract {
  specialization_values: [
    { specialization_id, type, legal_domain, source_expr }
  ],
}
```

The payload maps those stable launch and specialization IDs to backend API
concepts. For example, the Metal payload selects dispatch-by-threadgroups or
dispatch-by-threads and maps specialization IDs to Metal function constants.
The neutral manifest does not contain `MTLSize`, Metal axis limits, or function
constant indices.

### Typed feasibility

Each variant records:

- declared target profile key and descriptor digest;
- artifact execution/translation contract;
- feasibility-rule-set identity;
- exact/proven resource requirements;
- applicability guards;
- canonical deferred predicates with phase, query schema, validity scope, and
  authority; and
- proof/evidence references established during `ArtifactEvidence`.

`Rejected` and `Unknown` candidates cannot be serialized as executable
variants. A `Deferred` variant is valid only when the portfolio verifier has
proved an alternative complete route for every precommit rejection region and
the runtime declares every query provider required before `RoutingCommit`.
Resource estimates and cost values may explain selection but never satisfy a
hard predicate.

## Backend payload descriptor

```text
BackendPayloadDescriptor {
  payload_id,
  required,
  backend_key,
  payload_schema_key_and_version,
  representation_key,
  target_compatibility_contract_ref,
  artifact_execution_contract_ref,
  backend_metadata_section,
  code_sections,
  backend_evidence_sections,
}
```

The neutral reader treats backend metadata and code as typed bytes. It verifies
their descriptors and can determine that no installed backend understands a
required payload. The selected backend plugin parses its metadata schema and
cross-checks every advertised `BackendEntryKey`, binding mapping, launch
mapping, and specialization mapping against the neutral entries.

The initial profile marks every emitted payload required. A future fat-envelope
profile may mark a target family optional only when portfolio validation proves
that dropping that payload and all dependent programs preserves the declared
artifact interface and fallback coverage. "This device will not use it" is not
by itself permission to ignore an unsupported payload.

Payload descriptors and all their referenced section bytes participate in
exact envelope identity. A section may be shared by multiple payload
descriptors only if its purpose/schema explicitly permits sharing; executable
code is not implicitly shared by matching offsets or equal bytes.

### First Metal payload schema

The initial Metal payload metadata owns:

```text
MetalPayloadMetadata {
  metal_payload_schema,
  representation: Metallib,
  apple_platform: MacOS | IOSDevice | IOSSimulator | Catalyst,
  normalized_target_triple,
  sdk_identity,
  deployment_minimum,
  msl_language_standard,
  compile_and_link_options,
  metal_compiler_and_metallib_fingerprints,
  generated_msl_digest,
  helper_library_digests,
  metallib_section,
  runtime_translation_policy,
  entries: [MetalEntry],
}

MetalEntry {
  backend_entry_key,
  metallib_symbol,
  binding_slots: [(EntryBindingId, MetalBufferIndex)],
  function_constants: [(SpecializationId, MetalFunctionConstantIndexAndType)],
  dispatch_mapping: MetalDispatchThreads | MetalDispatchThreadgroups,
  pipeline_descriptor_contract,
  compiler_established_resources,
  optional_reflection_evidence,
}
```

All names containing `Metal`, `MSL`, `metallib`, Xcode, SDK, Apple platform,
deployment minimum, Metal buffer/function-constant index, and Metal dispatch
mode stay in this schema. The neutral profile sees governed keys, referenced
sections, compatibility predicates, and execution policy.

The metallib remains source-level AOT output that may require device-specific
pipeline translation. The Metal payload declares that policy explicitly; it
must not claim `NativeImage` merely because no MSL source is compiled at
runtime.

## Section framing and integrity

```text
SectionDescriptor {
  section_id,
  purpose,
  schema_key_and_version,
  required,
  exact_byte_length,
  digest_algorithm,
  exact_byte_digest,
}
```

Initial sections are stored verbatim without compression or encryption. This
keeps `exact_byte_length` and the interpreted bytes identical. A later storage
codec requires both stored and decoded lengths/digests plus a governed codec
and bounded expansion policy; it is not an optional flag added to version 1.

Parsing order is:

1. read the fixed header without allocation;
2. validate magic, supported envelope/encoding version, total length, manifest
   limit, section-count limit, and checked arithmetic;
3. read and digest exactly `manifest_byte_length` bytes;
4. parse the manifest under depth/count/string/expression budgets;
5. validate canonical wire form; when all fields are understood, re-encode and
   require byte equality; otherwise preserve and canonically validate every
   skippable unknown field record without interpreting or dropping it;
6. validate unique/sorted IDs and all cross-references;
7. stream each length-delimited section under its declared limit and verify
   exact length and digest before interpreting it;
8. require a one-to-one match between descriptors and physical sections, with
   no trailing bytes.

A digest is over raw bytes, not an object reserialized by the reader. This
avoids accepting two byte representations under one content address. A
manifest digest covers the exact canonical manifest bytes. An
`EnvelopeDigest` covers the exact complete envelope bytes and is computed
outside the envelope; embedding it inside the bytes it hashes would create a
recursive definition. Signatures, if later required, sign a domain-separated
root statement and live in a separately defined attestation layer.

All digest uses include an explicit governed algorithm key and domain
separator. The initial algorithm remains a product/version decision; a parser
must never infer one from digest width.

The derived aggregate identities avoid self-reference:

```text
SectionDigest = H(section-domain || exact section bytes)
PayloadDigest = H(payload-domain || canonical payload descriptor
                  || ordered referenced SectionDigest values)
ManifestDigest = H(manifest-domain || exact canonical manifest bytes)
EnvelopeDigest = H(envelope-domain || exact complete envelope bytes)
```

`PayloadDigest` and `EnvelopeDigest` may be computed rather than stored in-band.
An external content-addressed cache key records both its algorithm key and
digest bytes.

## Canonical encoding contract

Tiler owns a small schema-defined tagged binary encoding profile rather than
hashing arbitrary Rust serialization output. Version 1 requires:

- fixed magic and integer byte order;
- shortest/unique integer and length encodings;
- explicit field tags that are never reused;
- definite lengths only;
- duplicate fields and duplicate map/set keys rejected;
- maps/sets sorted by canonical encoded key;
- semantic sequences preserved in declared order;
- enums encoded by governed numeric values, never Rust discriminants;
- UTF-8 normalized only where a field's schema explicitly requires it;
- floats absent from identity fields unless represented by a separately
  specified canonical bit encoding;
- absent versus present-default defined per field, with one canonical form;
- unknown required fields/features rejected and unknown optional fields safely
  skippable only when their exact canonical field records are preserved or
  validated in place; and
- explicit parser budgets for total bytes, depth, fields, collections, strings,
  expressions, programs, entries, payloads, and sections.

Schema implementations may use a library internally only if conformance tests
prove these exact bytes across supported versions and languages. Ordinary
Protobuf deterministic output is explicitly unsuitable. Deterministic CBOR is
a viable implementation basis only after Tiler fixes a specific RFC 8949
profile and application data model; it is not selected merely by naming CBOR.

Canonicalization occurs before hashing and publication. Runtime loading rejects
well-formed but noncanonical encodings rather than normalizing them silently.
That prevents multiple byte identities for one manifest and keeps corruption,
cache, and reproducibility behavior consistent.

Forward compatibility cannot be implemented by decoding into an old struct,
discarding unknown optional fields, and re-encoding: that would change the
manifest bytes and digest. A codec must expose unknown tagged fields to the
canonical validator or preserve their exact canonical field records. The
initial version-1 reader may instead reject all unknown fields until this
mechanism is implemented.

## Versioning and compatibility

Versioning is component-wise:

```text
EnvelopeFormatVersion
CanonicalEncodingVersion
NeutralManifestSchemaVersion
KernelProgramSchemaVersion
AbiExpressionSchemaVersion
GuardAndRoutingSchemaVersion
CapabilityAndFeasibilitySchemaVersions
BackendPayloadSchemaVersion per BackendKey
EvidenceSchemaVersion per evidence kind
```

Every schema has a governed key and `{major, minor}` version. The initial
compatibility rule is:

- a major mismatch is incompatible;
- the reader must support at least the encoded minor version or a declared
  compatible range;
- every required feature and required field meaning must be recognized;
- unknown optional fields may be skipped only where their enclosing schema
  declares that skipping preserves semantics;
- field tags and enum values are never reused;
- adding a field that changes execution when ignored is a required feature or
  major-version change, not an optional minor addition; and
- a backend payload is compatible only when the installed backend provider
  supports its backend key, payload schema, representation, compatibility
  contract, and execution/translation policy.

The producer's crate version, Git revision, compiler fingerprint, SDK version,
and provider revision are provenance/identity, not generic reader-version
constraints. Conversely, a runtime API minimum or target OS minimum is an
explicit compatibility predicate, not inferred from which compiler produced
the bytes.

Unknown optional diagnostic sections may be skipped during interpretation but
their framed bytes remain present and digest-checked. Unknown
required sections, backend metadata, program semantics, ABI expressions,
guards, feasibility predicates, numerical evidence, or routing semantics fail
closed.

## Identity layers

| Layer | Included facts | Excluded facts |
| --- | --- | --- |
| `SemanticDigest` | canonical semantic graph, root interface, resolved operation/numerical contracts | schedules, providers, backend, runtime device |
| `ScheduledRegionDigest` | semantic digest, index region, normalized schedule, numerical realization, target policy needed by the schedule | payload bytes, live device observations |
| `KernelProgramDigest` | scheduled entries, program DAG, buffers/materializations, ABI, expressions, guards, routing, target requirements, selected reached providers | unused registered providers, live/prepared fact values |
| `BackendPayloadDigest` | exact backend metadata/code section bytes, target representation, backend entry mappings | neutral program equality, live runtime state |
| `ManifestDigest` | exact canonical neutral manifest bytes, including section descriptors and selected provenance | section bodies except through descriptors |
| `EnvelopeDigest` | every exact byte of the framed envelope | external cache path, embedding location |

Selected provider identity is represented as
`{ProviderKey, capability_api_version, ProviderRevision}` and appears only for
semantic authorities reached or capabilities selected by a packaged program.
The complete frozen registry may remain compilation-request provenance outside
the runtime artifact; unused registrations must not poison the artifact.

Compiler provenance includes normalized frontend/optimizer/scheduler/codegen
versions, selected providers, generated-source/helper digests, compiler/linker
executables or version fingerprints, target/options, SDK/header identity, and
all output-affecting configuration. Wall-clock timestamps and absolute paths do
not participate unless a backend proves they affect output; useful display-only
provenance is still covered by `ManifestDigest` when embedded.

Live device identity, prepared pipeline/kernel handles and facts, runtime
allocator state, free memory, binary-archive state, and routing observations
are runtime cache/provenance concerns. A runtime pipeline cache key is at least:

```text
live device/context identity
+ EnvelopeDigest or BackendPayloadDigest
+ BackendEntryKey
+ specialization values
+ pipeline descriptor and runtime translation/archive mode
```

## Validation boundaries

Validation is monotonic and fail-closed:

1. **Framing/integrity, device-free:** bounds, canonical manifest, exact section
   lengths/digests, no duplicate/unreferenced/trailing content.
2. **Neutral schema, device-free:** component versions/features, program and
   entry references, semantic/numerical equality across portfolio variants,
   complete outputs, guards/routing, selected providers, and `KernelProgram`
   whole-program verification.
3. **Backend payload, backend-provider but device-free where possible:** parse
   governed backend metadata; verify every backend entry and ABI/launch/
   specialization mapping; inspect executable symbols/reflection when the
   representation permits it.
4. **Declared target compatibility:** match representation, platform/profile,
   data layout, translation policy, and compile guarantees.
5. **Live preflight:** bind semantic roots, evaluate semantic constraints and
   host expressions, query live-device facts, evaluate guards, and route among
   complete variants.
6. **Prepared-entry and launch preflight:** create every required pipeline or
   loaded kernel, query authoritative entry facts, evaluate launch-instance
   predicates, then choose the final route.
7. **`RoutingCommit`:** only after all route-sensitive checks; allocation and
   encoding begin afterward, and later failures return errors rather than
   fallback.

Digest success never establishes semantic, ABI, target, or numerical validity.
Backend load success never establishes that every entry can be prepared or
launched. Conversely, a transient allocation failure after commit is not
retroactively a target-compatibility miss.

## Valid examples

### One Metal payload, two complete variants

```text
program P semantic=S numerical=N
  variant scalar: guard=true -> entries [E0]
  variant vector: guard=aligned(16) -> entries [E1]

E0 -> payload metal0 / backend key "scalar"
E1 -> payload metal0 / backend key "vector"

metal0 metadata section -> exact digest M
metal0 metallib section  -> exact digest L
```

Both variants share `S` and `N`. Their guards and routing are neutral. Metal
symbols, buffer indices, function constants, and dispatch modes exist only in
the metadata section. The payload plugin proves that both backend entry keys
exist and map every neutral binding.

### Multi-dispatch reduction

The neutral program section contains partial and final stages, the partials
temporary, its buffer lifetime, and the dependency edge. Two executable
entries reference one backend payload. The payload owns the two symbols; it
does not own or obscure the program dependency or scratch lifetime.

### Future Metal and CUDA images

One envelope may contain `metal0` and `cuda0` descriptors with independent
schemas and code sections. Each complete program variant references entries in
one compatible target domain. In the initial required-payload profile, a reader
must understand both. A later optional-target-family profile may skip CUDA only
after proving that removal of its dependent programs preserves the envelope's
declared interface and fallback coverage.

## Invalid examples and required diagnostics

| Invalid artifact | Required rejection boundary |
| --- | --- |
| Section truncated, extended, reordered contrary to canonical framing, or digest changed | framing/integrity |
| Fully understood manifest parses but re-encodes differently | `noncanonical-manifest` |
| Duplicate program, entry, payload, field, or section ID | neutral structural validation |
| Unknown required feature or component major version | schema compatibility |
| Program references a missing entry/payload/section | neutral cross-reference validation |
| Unreferenced executable section is appended | exact section closure |
| Two variants differ in semantic or numerical contract | portfolio verification |
| Deferred predicate has no precommit query/provider or complete alternate route | feasibility/portfolio verification |
| Required provider revision is missing or substituted | selected-provider validation |
| Neutral manifest contains a Metal buffer index or MSL version | schema-layer violation |
| Metal metadata omits a neutral binding or maps one binding twice | backend payload validation |
| Metallib loads but a required symbol/pipeline cannot be prepared | prepared-entry preflight; route before commit if covered |
| Launch expression overflows after final selection | invariant failure; fail closed |
| Runtime attempts fallback after allocation/encoding | execution-contract violation |

Diagnostics preserve which boundary failed and do not reinterpret corrupt or
unsupported schemas as plan-applicability misses.

## Bounded serialization/validation spike

`spikes/artifacts/artifact_envelope.rs` implements a dependency-free model of:

- fixed header and length-delimited section framing;
- SHA-256 manifest and exact-section digests;
- deterministic canonical encoding of sorted IDs and provider identities;
- external exact-envelope digest without an in-band recursive field;
- component schema/required-feature and backend-schema compatibility;
- program/entry/payload/section cross-reference closure;
- selected-provider identity;
- rejection of corrupted, truncated, noncanonical, duplicate, missing,
  unsupported, unreferenced, and trailing content; and
- a Metal payload represented only as typed opaque metadata/code sections to
  the neutral validator.

The spike intentionally abbreviates `KernelProgram`, ABI, launch, guards, and
requirements as their already-validated canonical digests. Production loading
must parse and invoke their component verifiers rather than trusting digests
alone. It does not choose the production Rust serialization library or freeze
the final field tags. Its version-1 positional manifest admits no unknown
fields; it therefore does not claim the proposed optional-field evolution
behavior.

Run it with:

```sh
rustc --edition 2021 --test \
  spikes/artifacts/artifact_envelope.rs \
  -o /tmp/tiler-artifact-envelope-spike
/tmp/tiler-artifact-envelope-spike
```

## Remaining bounded decisions and experiments

1. Select the production canonical codec after cross-language byte-vector,
   parser-budget, unknown-field, compile-time, and binary-size measurements.
   Compare a narrow deterministic-CBOR profile, a small owned tagged codec, and
   schema-generated alternatives; ordinary Protobuf output is excluded.
2. Select and govern the initial cryptographic digest algorithm and domain
   separators. Measure hashing during proc-macro expansion and runtime loading.
3. Decide whether an invocation embeds one envelope per target family or one
   fat envelope. This is distribution policy; the neutral model supports both.
4. Define optional/debug-section retention and stripping so that changing
   diagnostics predictably changes `EnvelopeDigest` without changing
   `KernelProgramDigest`.
5. Design signatures/attestations only if authenticity beyond embedded-build
   trust is required. Digests provide integrity and content identity, not
   publisher authenticity.
6. Reconcile the existing Metal-specific `docs/artifact-abi.md` schema with
   this separation in a later contract/ADR ticket: rename the outer bundle to
   the neutral envelope and move all Metal-only fields into
   `MetalPayloadMetadata` without changing accepted inline-AOT behavior.

## Traceability

The result is adopted by the [artifact ABI](../../artifact-abi.md) and exercised
by the [artifact envelope spike](../../../spikes/artifacts/README.md). Production
serialization, authenticity, and version-skew policy remain unimplemented.
