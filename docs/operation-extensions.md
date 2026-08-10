---
schema: "tiler-doc/v1"
id: "tiler.contract.operation-extensions"
kind: "contract"
title: "Operation extension contract"
topics: ["extensions", "operations", "semantics"]
contract_status: "mixed"
implementation_status: "partial"
evidence: ["tiler.research.extensions.operation-extension-surface", "tiler.research.extensions.operation-extension-api", "tiler.research.extensions.proc-macro-extension-visibility", "tiler.research.extensions.semantic-foundation-api-v2"]
---

# Operation extension contract

**Status:** accepted semantic registration boundary and accepted seam classification; index/access lowering capabilities implemented and resolved on the compile path; the scalar-lowering family retired by ADR 0105 and removed from the crate; reference capabilities implemented but reached by no compile-path caller; remaining compiler capabilities proposed

## Ownership boundary

This document owns the public capability surface and trust, identity,
registration, and diagnostic obligations for operation providers. Individual
operation semantics remain in their typed definitions; proc-macro visibility
and backend realization remain separate integration concerns.

ADR 0005 accepts a public experimental vertical extension boundary. This
document proposes the initial safety, determinism, and compilation-phase
contract. The supporting [research](research/extensions/operation-extension-surface.md),
[API sketch](research/extensions/operation-extension-api.md), and
[compile-checking spike](../spikes/extensions/operation-api) validate its
shape; exact Rust names and allocation choices remain experimental.

## Initial trust and linkage model

Extension providers are trusted native compiler code, statically linked into
the process using the ordinary compiler API and supplied explicitly to a
compiler session. They have compiler-process privileges and are not sandboxed.
Native dynamic loading, hot reload, a stable Rust plugin ABI, untrusted plugins,
and cross-process providers are deferred.

A separately compiled function-like proc macro receives tokens and cannot
discover arbitrary provider objects or trait implementations defined later in
the consuming crate. Therefore:

- ordinary compiler-API users may supply external operation providers;
- a proc macro supports providers compiled into its host dependency graph and
  complete canonical semantic declarations visible in invocation tokens;
- Cargo features can select only provider dependencies already declared by the
  macro package;
- consumer-side automatic registration does not cross the proc-macro
  compilation boundary;
- an unavailable provider fails semantic admission rather than becoming an
  opaque operation or runtime compilation request.

This measured limitation is accepted by ADR 0045. It does not make the
compiler-core extension boundary consumer-specific or close the ordinary
compiler API.

**Fact — an accepted record extends this model to backend composition and adds nothing to it.** [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) proposes that backend composition inherits every clause above unchanged, including the proc-macro limitation, and reserves no seam for native dynamic loading, a stable plugin ABI, `dlopen`-style adapter discovery, hot reload, untrusted or sandboxed providers, cross-process callbacks, or runtime source compilation — the one thing crossing a process boundary being artifact bytes, validated from bytes on arrival. That inheritance is a restatement rather than a new position; the record was accepted on 2026-07-31 with the inheritance intact.

**Fact — the first clause of the list above is now reachable rather than intended: an ordinary compiler-API user does supply external operation providers to a session.** This paragraph previously recorded the opposite, that `session::compile_governed` named the governed profile and that `CompilationRequest` and its installed-capability field were crate-private, so composition was reachable out of crate and installation was not. [`prototype-public-compiler-api`](../tickets/prototype-public-compiler-api.md) closed that: `session::CompileRequest` is the consumer-agnostic request, `session::InstalledCapabilities::installed` binds a caller's lowering registry to its exact `FrozenScalarRegistry`, `session::CompileRequest::with_capabilities` installs the checked pair, and `session::compile` compiles through it; the operation provider's same atomic semantic-registration transaction owns its typed realization law, and the session derives rather than accepts that authority. `session::compile_governed` is a single-target convenience spelling of that same path rather than a privileged one. Composing a registry, installing one, and reaching the compiler remain three separate claims and this contract still keeps them apart — all three hold for the index/access family, and it is the only lowering family the three-claim separation applies to. They held only in part for the scalar-lowering family, which composed and installed and which no compile stage resolved; [ADR 0105](decisions/0105-retire-the-scalar-lowering-provider-seam.md) settled that asymmetry by retiring the family rather than by wiring it, and the removal has landed.

## Public extension seams

[ADR 0078](decisions/0078-name-the-intended-public-extension-seams.md) accepts which surfaces Tiler intends as public extension seams. That record owns the classification and the derivation behind it; this contract states which surfaces the classification names and what it obliges a provider surface to keep, and does not restate its reasoning. Its complementary half — which authorities are permanently internal — is stated by [the architecture contract](architecture.md#permanently-internal-authorities), which owns component ownership.

A seam is a propose-then-re-verify boundary, and that is the whole of its trust model. A provider *proposes* work and Tiler *re-derives* every fact the proposal would otherwise assert; a provider is trusted to be deterministic, side-effect-free, and in-process, and it is never believed. Four properties are jointly the admission test, and a surface that cannot hold all four is not a seam however it is spelled:

- the provider's output re-enters the ordinary checked path, so resolution establishes an authority and never the correctness of what that authority emits;
- the provider cannot stamp its own provenance — identity, exact resource requirements, and boundary contracts are derived by the host from verified output, so a proposal can neither forge another provider nor declare what a verifier did not compute;
- its identity is versioned and separated from graph meaning under ADR 0072; and
- every disposition — admission, rejection, ambiguity, absence, and an exhausted proof budget — is a distinct typed outcome that reaches the explain trace.

These are the surfaces intended as public extension seams, with the participation model each is intended to admit at maturity:

| Surface | Intended participation |
| --- | --- |
| `tiler_ir::semantic::SemanticRegistryProvider`, with `OperationInferencer` and `ValueTypeInstanceValidator` | Third-party |
| `tiler_compiler::capability::IndexAccessLoweringProvider` | Third-party |
| `tiler_reference::{ReferenceRegistryProvider, ReferenceOperation, ReferenceValueValidator}` and `tiler_reference::oracle::ScalarReferenceOperation` | Third-party |
| `tiler_ir::index::ScalarOperationInferencer` | Third-party |
| `tiler_compiler::physical_provider::PhysicalImplementationProvider` | Third-party |

One further extension-shaped surface is deliberately unassigned and acquires no intent from standing beside these: the mature per-operation fusion numerical capability that this contract lists among separately versioned optional capabilities below. ADR 0078 records it as an open question. It is not a seam of this contract until that question is answered, and it may not be treated as one because the surrounding table names the surfaces that are.

**Fact — the table carried a fifth row and no longer does.** `tiler_compiler::capability::ScalarLoweringProvider` stood here as a third-party seam until [ADR 0105](decisions/0105-retire-the-scalar-lowering-provider-seam.md), accepted 2026-08-06, retired it. The ground is this contract's own admission test rather than a maturity shortfall: the family's output was a list of scalar value identifiers handed back into a caller-owned region builder, refinement refused a resolved scalar capability by name, and no `refine_scalar_*` authority exists — so the first of the four properties above, that a provider's output re-enters the ordinary checked path, is one that surface could not hold. The registered realization law fixes the whole realization including its per-point scalar applications and refuses any candidate whose canonical identity differs, which left a scalar provider one admissible output decided by an authority it did not own. **A decomposition an index-access provider factors its per-point work through survives the removal**; what does not is the claim that such a factoring is a registered participation boundary. The trait, its family, and its registration are gone from the crate: [`remove-the-scalar-lowering-family-from-the-compiler`](../tickets/remove-the-scalar-lowering-family-from-the-compiler.md) executed the removal, and this contract's status line says so.

**Fact — the physical-implementation provider was the second of those two and is now a row above.** [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), accepted 2026-07-31, decides that target-specific scheduling knowledge is a checked combination split at feasibility, and that the physical-implementation provider becomes an installable public seam through a frozen per-session registry in the same idiom the lowering seam uses. Both halves of ADR 0078 item 5 — the data-versus-code question and the trigger reading — were answered at that acceptance. The implementation landed 2026-08-08 under [`drive-an-external-physical-implementation-provider-through-compilation`](../tickets/drive-an-external-physical-implementation-provider-through-compilation.md), which is what moved the row into the table; **the concrete trait and module surface remain a labelled draft under ADR 0075 until Tom accepts their exact included and excluded sets**, and the row states intended participation rather than an accepted boundary. The fusion-capability question is untouched by that record.

**What the new row admits is narrower than the crate-internal seam, and the narrowing is by refusal rather than omission.** Installation is additive and cannot displace Tiler's own governed provider; a caller may propose a checked scheduled kernel and no other body, because a kernel subprogram states per-stage semantic attribution in a graph-local authoring coordinate this boundary does not export and out-of-crate opaque-call registration stays compiler-owned under ADR 0090 item 14; and every constructible cost estimate carries the one governed cost-model key, so two providers' estimates cannot be incomparable while still being ranked. Installing one identity twice, or an identity equal to the governed provider's, is a typed installation refusal rather than a replacement.

Intended participation is not a maturity claim, and a `pub` keyword is neither necessary nor sufficient for either. ADR 0078 records the rung each row has reached against the four claims `AGENTS.md` keeps apart — a type-system reservation, an architectural seam, implemented support, and a tested guarantee — and those rungs differ across the table and can differ between the two halves of one surface. **Inference — what this contract states in place of a rung, because a rung is a measurement that goes stale and an invariant is not:** a surface has reached a tested guarantee only when a provider written outside the defining crate's own governed set has driven it through the ordinary compile path and the resulting plan names that provider as its authority. Registration and resolution being implemented and tested is the weaker claim, and it is the one the reference row holds: `tiler-reference` is a development dependency of `tiler-compiler`, so it is reachable from tests and proof executables and from no production compiler stage. The retired scalar-lowering row held that same weaker claim, and the invariant retired it rather than scheduling evidence for it — ADR 0105 found the missing evidence to be unproducible in principle rather than merely unwritten, which is the one case where the invariant eliminates a row instead of leaving it below the bar.

**Measurement, 2026-08-08 — the physical-provider row's evidence, with its boundary stated rather than rounded up.** `crates/tiler-compiler/tests/external_physical_provider.rs` defines a provider, installs it through `CompileRequest::with_physical_providers`, compiles, and reads its identity back out of a retained plan through `PlanAlternative::selected_physical_providers`. An integration test is a separate compilation unit that can reach only `pub` items, so what it exercises is exactly the surface an out-of-tree crate would — but it lives in the defining *package*, so it is not the fully out-of-tree fixture the [forkless provider spike](../spikes/extensions/forkless-physical-provider/README.md) is. Read the evidence as: the public surface is sufficient to write and install a provider and to observe its selection, measured; that a published crate outside this workspace can do the same is inference from the same reading. The spike is the artifact that would upgrade it, and re-running it is what would show the two blockers it recorded are gone.

**Corrected 2026-08-10 — the separate-workspace re-run is complete.** The retired sentence "The spike is the artifact that would upgrade it, and re-running it is what would show the two blockers it recorded are gone" is quoted here so a grep hit lands inside this correction rather than proving the re-run remains future work. [`refresh-the-forkless-physical-provider-spike-against-the-landed-seam`](../tickets/refresh-the-forkless-physical-provider-spike-against-the-landed-seam.md) retained [its 2026-08-08 result](../spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json): `cargo nextest run --workspace` from a separate workspace completed with **8 tests run, 8 passed, 0 skipped** against crates subject `cb62784c7d8b63aa2e73c9ac490101b748abc0ec`. This is a bounded out-of-tree Measurement on `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`) on `aarch64-apple-darwin`, macOS `27.0 (26A5388g)`, not a portability claim. The in-package fixture remains separate evidence from a compilation unit inside the defining package, and neither result accepts the exact physical-provider surface: it remains a labelled draft awaiting Tom at [`accept-the-installed-physical-provider-public-surface`](../tickets/accept-the-installed-physical-provider-public-surface.md).

The lowering seam was the case where the two halves differed, and both are now public: a registry composes entirely through `tiler_compiler::capability` and installs through `tiler_compiler::session`, as the fact above records. What the two halves differed in was *family* rather than visibility — the index/access family installed and resolved on the compile path, the scalar-lowering family installable and resolved by nothing — and ADR 0105 closes that difference by retiring the second family rather than reaching it, so one lowering family remains and it is both installed and resolved. Under ADR 0074's staging convention a crate-private authority is a temporary posture rather than a classification, so neither a private surface in this table nor a public one absent from it may be read as an intent.

## Registry lifecycle and coherence

Registration uses an explicit per-compiler/session builder. Before graph
verification or optimization it freezes into an immutable snapshot:

- durable ordering is by operation/provider key, never insertion or link order;
- duplicate semantic `OpKey` ownership is a hard error, never last-wins;
- one semantic authority defines each operation's meaning and schema;
- additional decomposition, lowering, scheduling, or cost providers have
  independently named provider identities and declared compatibility;
- collisions or contradictory provider selections fail deterministically;
- the complete frozen registry participates in compilation-request provenance;
  reached provider-independent definitions, their admission-provider
  provenance, and selected capability providers participate independently in
  selected-plan/artifact identity;
- Rust `TypeId`, vtable addresses, function addresses, registration addresses,
  and hash-map randomization never participate in durable identity.

The frozen registry is immutable and safe for concurrent read-only use.

The current semantic prototype makes registration batches sticky-failing and
transactional. The first rejected type, operation, or marker registration
poisons the batch even if provider code ignores the returned error; no part of
that batch can enter the registry. Freeze validation and public definition
iteration use canonical key order rather than callback or hash-map order.
Registry definition, operation, marker, and aggregate canonical-byte budgets
are checked before the batch is retained.

Semantic keys, normative references, and canonical text/byte payload
constructors inspect borrowed input before making the host-owned copy.
Separately named owned constructors validate first and then move already-owned
storage without copying it. Rejected oversized input is never retained merely
so its length can be checked.

The implementation separates the semantic portion required by `tiler-ir` from
later executable capabilities. Under ADR 0066, `FrozenSemanticRegistry` is a
cheap-clone owned snapshot of nominal definitions, parameterized constructors,
encoded schemes, operation definitions, provider provenance, and optional
local marker bindings. Semantic programs retain it so checked Rust reification
does not require a borrowed context. Registration callbacks are discarded at
freeze; immutable definition objects required for concrete-type validation and
operation inference remain in the snapshot under narrow host-owned contexts.
Under ADR 0065, executable reference capabilities compose in
`tiler-reference`; optimizer capability registration and backend participation
likewise compose later rather than introducing an inward dependency from
semantic IR. **Corrected 2026-08-08:** this read "optimizer and backend
registries", naming a thing [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
eliminated — backend composition is per responsibility and has no registry of
its own, and item 4 states there is "no `BackendProvider`, no provider bundle,
no emitter registration, and no runtime-adapter registration". The registries on
this path are the compiler's installed lowering and physical-provider ones. The
dependency-direction claim ADR 0065 makes here is unchanged, which is why the
sentence is corrected rather than removed.
Provider objects are expected to be `Send + Sync + 'static` unless an explicit
compiler mode serializes a capability.

The semantic registry, value-type registry, and reference capability dispatch
are implemented for the bounded profile. Compiler capability registration for
the index/access family is now implemented in
`tiler_compiler::capability`, merged from
[`prototype-operation-capability-registry`](../tickets/prototype-operation-capability-registry.md).
Registration is transactional per call and rejects colliding capabilities
deterministically, while resolution reports deterministic ambiguity and
missing-capability diagnostics. Its canonical provenance is built from durable
semantic and provider identities, never `TypeId`, vtable, function, or
allocation addresses, and a provider emits only through the canonical
`IndexRegionBuilder` wrapped by a narrow checked context — never constructing
provider-owned IR or finalizing the region. Scheduled-kernel and opaque physical
providers remain owned by their own later tickets.

**Fact — that module registers one lowering family, and every mechanic above is the registry's rather than the family's.** [ADR 0105](decisions/0105-retire-the-scalar-lowering-provider-seam.md) retired the second family and [`remove-the-scalar-lowering-family-from-the-compiler`](../tickets/remove-the-scalar-lowering-family-from-the-compiler.md) removed `LoweringFamily::ScalarLowering`, `register_scalar_lowering`, and `resolve_scalar_lowering` with it. Transactional registration, deterministic collision rejection, deterministic ambiguity, and the missing-capability diagnostic are properties of the registry and not of either family, which is why they survived the removal and why that record made re-expressing their tests against `register_index_access` normative rather than deleting them with the family; ten such tests were ported. The removal is identity-preserving: the capability key encodes the family's stable tag, index access is tag one and stays tag one, and its `key_token` is unchanged, so no frozen registry encodes differently and no governed capability key moved. `LoweringFamily` stays a `#[non_exhaustive]` enum and the stored provider handle stays family-typed; ADR 0105 decision 4 reserves collapsing either to Tom, and a second family would want both back.

This registry resolves available lowering knowledge and provenance but does not
prove an occurrence was lowered correctly. That checked refinement — binding an
exact occurrence, value, access, numerical, and provider selection to a resolved
provider and proving the emitted work refines it — is implemented in
`tiler_compiler::legality`, merged from
[`semantic-to-index refinement`](../tickets/prototype-semantic-index-refinement.md),
and both halves now run on the ordinary compile path: every recognized
occurrence resolves exactly one index/access capability and is then refined
against the region that capability's provider emitted. The registered surface is
a reviewed prototype boundary, not a stabilized compiler-session API. The
ordinary public compiler session exists —
[`prototype-public-compiler-api`](../tickets/prototype-public-compiler-api.md)
landed `tiler_compiler::session` and Tom reviewed it — and that session facade is
accepted but not stabilized. The registered capability surface remains a
reviewed prototype, and no work may treat either surface as published or fixed.

Two consequences belong to this contract rather than to the compiler's own. First, a resolved capability's provenance reaches the artifact: a selected plan records the `{provider identity, capability revision}` pair each occurrence resolved, and the compiler re-derives that set from the installed registry rather than trusting what a plan recorded. Second, resolution for this family fails closed, so the two failing dispositions this contract already distinguishes are load-bearing rather than diagnostic preferences — an absent capability says the installed authority was never extended to the occurrence, a contended one says two extensions contradict each other, and neither is resolved by a priority order or a default provider. [The optimizer contract](compiler/optimizer.md#lowering-capability-resolution-and-index-region-refinement) owns the stage's placement and behaviour.

## Semantic and provider identity

`OpKey { dialect, name, semantic_version }` identifies semantic meaning and
schema compatibility, not one Rust implementation. Every selected provider
declares a stable provider ID and revision/fingerprint covering all
output-affecting behavior it owns, including inference, evaluation,
decomposition, rewriting, lowering, or code generation.

Provider revisions are an explicit author trust contract, not automatic source
attestation. Changing output-affecting behavior without changing the declared
revision is a provider bug.

**Decided — the capability-API and compiler version requirement is retired, because content addressing already discharges what it was for (2026-07-27).** The obligation it expressed is that artifact identity must change whenever a compiler or capability-interface change can change executable meaning or bytes. That obligation stands; a version field is not what meets it here.

**Fact — a capability-API version has nothing it could enforce against.** The trust and linkage model above states that extension providers are "trusted native compiler code, statically linked into the process", and that "native dynamic loading, hot reload, a stable Rust plugin ABI, untrusted plugins, and cross-process providers are deferred". A provider and the capability API it is written against are therefore compiled together into one binary. A provider that has not been rebuilt against a changed API does not produce a mismatched artifact — it fails to compile. There is no reader implementing one version of the API and no producer implementing another, so there is no mismatch a recorded version could detect. **An identity component that cannot be violated is not an identity component**, and the requirement would only become live if one of the deferred linkage models above were admitted, which is where it should be reconsidered.

**Fact — a compiler version is discharged by what artifact identity already folds.** A carried payload's `PayloadDigest` is content-addressed over its compilation subject, and that subject contains *the exact source that was compiled*, the target, the compile and link flags, and the toolchain provenance. Artifact identity folds the descriptor, so it folds all of them. A Tiler build that emits different source for one semantic program therefore yields a different artifact identity by construction; a Tiler build that emits the same source, flags, ABI, and schedule cannot change executable meaning. A backend-toolchain change is covered by the folded provenance rather than by a Tiler version. `crates/tiler-artifact/src/program/codec/payload.rs` states and its decoder re-proves this.

**Inference — recording a compiler version instead would be weaker, not stronger.** It would assert what produced an artifact rather than what the artifact *is*, so two builds of one compiler emitting identical source would be given different identities and lose a legitimate cache hit, while the case it claims to catch — a compiler change that alters emitted meaning — is already caught by the source it alters.

**What is not covered, stated rather than implied.** A provider that changes output-affecting behaviour without bumping its declared revision remains a provider bug, exactly as the paragraph above says, and no capability-API version would have caught it either: the API would be unchanged. Separately, two bundles built from one compilation subject by a non-reproducible linker share an artifact identity and differ in envelope digest; that is a decided property of content addressing over inputs, recorded in the payload module and in the [artifact contract](artifact-abi.md), not a gap this decision opens.

Provider-independent semantic definitions, graph meaning, and provider
provenance are separate identity subjects under ADR 0072. A provider-only
revision does not change an otherwise identical `SemanticGraphIdentity` or its
reached-definition projection. It does change admission provenance, the frozen
registry snapshot, and every selected refinement, plan, or artifact whose
correctness depends on that provider. Unused providers remain request-
environment provenance and do not enter selected artifact identity.

Definition and admission subjects are program-owned results of the complete
transitive authority closure, not caller-selected registry subsets. The closure
includes type references in concrete nested/encoded types, occurrence
attributes, definition facts, operation defaults, operation facts, and
conformance values. Registry freeze runs the same iterative, cycle-safe,
bounded closure over all registered authority and rejects missing referenced
definitions before any program can use the snapshot.
The program exposes graph meaning, reached definitions, admission provenance,
and the complete snapshot only through its non-forgeable `SemanticIdentity`
bundle, preventing extensions or compiler layers from assembling evidence from
different programs.

Value types follow the same durable identity principle. A conceptual
`TypeKey { namespace, name, semantic_version }` identifies a canonical nominal
dtype or parameterized constructor for both built-ins and extensions. ADR 0062
places nominal, parameterized, and encoded-numeric scheme contracts in one
tagged `ResolvedValueType` domain without collapsing `TypeKey` and
`QuantSchemeKey`. Built-ins may expose convenient constants or enum-like
spellings, but durable IR never substitutes Rust discriminants, `TypeId`,
implementation addresses, or unversioned display names for canonical resolved
identity. Registered descriptors define structural and value semantics;
provider identity separately records the implementation that supplies
capabilities.

A canonical type key is not rewritten when its support level changes. If Tiler
later bundles support for `acme::fp8_special@1`, it supports that existing
identity rather than relabeling it as a new `tiler` type. Frontend aliases may
improve spelling, but aliases resolve to a canonical key before semantic
admission and never create identity equivalence implicitly. Namespace
ownership, collision handling, provider compatibility, and durable descriptor
encoding require the same deterministic registry discipline as operations;
ADRs 0060 and 0062 fix the Rust authority boundary: external marker types carry
no key or capability authority merely by implementing a trait. The explicit
registry binds one process-local marker `TypeId` to one complete canonical
registered `ResolvedValueType`, rejects duplicate marker/identity bindings, and
freezes the association before typed construction. The local `TypeId` is lookup
metadata only and never durable identity. Remaining API details concern
ergonomics, not semantic ownership.

Tiler-governed built-in type descriptors contain mandatory normative source
references but Tiler owns their IR-key compatibility. Published descriptors are
immutable. A semantically compatible later standards revision may add
non-semantic provenance/equivalence evidence; a meaning change requires a new
semantic key version. Admission rejects a new built-in key when an existing
external canonical identity already owns that exact format. External
equivalence mappings are explicit, versioned, and conformance-tested rather
than inferred from names or structural fields.

Quantization and other encoded numeric interpretations have a separate
namespaced, versioned scheme identity. A `QuantSchemeKey` is neither the
primitive code/expressed `TypeKey` nor a physical `StorageEncodingKey`.
Providers declare a bounded static scheme schema, ordered typed component
roles, coordinate maps, normative decode and optional encode semantics,
transformation capabilities, operation support, canonical conformance vectors,
and provider revision.

The host owns component operand ordering, canonical encoding, graph dependency
tracking, shape/value constraints, resource bounds, and explanation. Extension
schemes may describe multiple scale levels, codebooks, nested encoded metadata,
or multi-component payloads, but composition is bounded and acyclic. They may
not hide parameter data or mutable calibration state inside callbacks.
Physical encoding providers separately describe packing, buffers, interleaving,
alignment, padding, memory space, and ABI realization without changing the
scheme's numerical meaning.

## Host-owned canonical attributes

Durable attributes use a bounded canonical value model and encoder owned by
Tiler, not arbitrary extension `Serialize` output. The contract defines:

- integer widths and signedness;
- byte order;
- string and Unicode treatment;
- sequence order and canonical map-key order;
- duplicate-key rejection;
- absent-versus-default normalization;
- floating-point bit semantics, including signed zero and NaN payloads;
- schema and unknown-field handling;
- recursion depth, byte, item-count, and shape/rank limits;
- checked size arithmetic.

Providers declare attribute schemas/defaults and validate semantic constraints;
the host canonicalizes, bounds, serializes, and hashes the data.

The accepted v1 model is the discriminated `CanonicalValue` defined in
[the IR contract](ir.md): fixed-width signed/unsigned bits, governed float bits,
bytes, exact UTF-8, type keys, ordered sequences, and records keyed by stable
`AttributeFieldId`. It has one tagged big-endian identity encoding, rejects
unknown/duplicate fields, and resolves schema defaults before hashing. Provider
Rust structs and serializer output are never durable identity.

## Mandatory definition and optional capabilities

Exactly one semantic authority owns an `OpKey`. Its mandatory definition
contains the bounded operand/result/value-kind schema, attribute schema,
initial pure effect signature, deterministic inference and semantic
validation, normative semantic specification identity, conformance vectors,
and stable host-readable names.

When an operation's admitted domain is narrower than its operand types, the mandatory definition also carries bounded typed semantic-precondition declarations. The host derives declaration ordinals, validates every operand selector, instantiates the declarations against exact occurrences, and owns canonical identity. A provider may name a new stable predicate and invalid-input code, but an unknown predicate has no implicit proof or runtime checker: it remains an exactly identified residual and compilation fails closed until separately admitted authorities can assess and enforce it.

Semantic preconditions are not descriptive facts, applicability predicates, representation validators, or inferencer-returned result facts. An extension cannot remove one by returning a convenient result type, cannot ask the caller to restate it as graph data, and cannot turn static disproof into “not applicable.” Providers that declare no narrower input domain need no declarations.

Normative meaning is mandatory, but a particular executable evaluator is not
universally mandatory. Reference evaluation is an optional capability. A phase
that needs executable reference behavior admits the operation only when a
compatible evaluator or exact verified decomposition supplies it. Likewise,
registration alone grants no rewrite, fusion, lowering, costing, or execution
authority.

Decomposition, rewrites, access lowering, fusion participation, typed opaque
physical implementations, structured-kernel lowering, accuracy evidence,
target feasibility/cost evidence, and provider-specific diagnostic detail are
separately versioned optional capabilities. An opaque physical implementation
must expose typed ABI, effect, alias, placement, target, numerical, resource,
and failure-stage boundary contracts; it is not an unrestricted callback in
semantic IR.

"Separately versioned" is two revisions, not one. The implemented access-lowering family realizes it as a capability revision carried beside, and independent of, the admitting provider's own output-affecting revision: one provider may register several capabilities that move at different rates, and both revisions are retained wherever a lowering's provenance is recorded. A provider whose emitted lowering changes must raise the capability revision, because that is the half a compiled artifact's provenance is keyed on.

The bounded physical frontier implements scheduled-kernel and kernel-subprogram
providers, and admits opaque calls through a separate compiler-owned declaration
and registration path that
[`implement-opaque-physical-call-providers`](../tickets/implement-opaque-physical-call-providers.md)
and [`integrate-opaque-calls-into-the-physical-frontier`](../tickets/integrate-opaque-calls-into-the-physical-frontier.md)
delivered. That path is *not* a public seam: `OpaqueCallDeclaration` and
`OpaqueCallRegistry` are crate-private, so no out-of-crate provider registers an
opaque call, and this contract's provider obligations reach it as the shape a
future seam would have to keep rather than as one it already offers.

## Capability coherence

Capability callbacks are immutable and deterministic functions of explicit
inputs. They may not depend on undeclared environment state, time, randomness,
mutable global state, registry order, or call order.

- Inference results are rechecked by host graph verification.
- Decomposition and rewrite output re-enters full semantic verification.
- Lowering declares its numerical, shape, effect, operation-set, and target
  preconditions in machine-checkable form where possible.
- A transcendental definition declares immutable reference semantics,
  supported accuracy envelopes and domains, independent exceptional-value
  behavior, reference-evaluator capability, and scoped conformance evidence.
  Its decompositions and rewrites state exactly which input contract they
  preserve and which subordinate contracts they create.
- Missing optional knowledge is conservative. For the one family the compile
  path resolves today, index/access lowering, "conservative" is a fail-closed
  compile refusal attributed to the occurrence, not a narrowed result.
- Contradictory capability answers are hard diagnostics, not fallback misses.
  A contradiction is a *disproved* predicate rather than a deferred one, and
  the distinction is checked: an absent capability and a contended one reach
  the explain trace as different dispositions, so a reader is never left to
  infer which of the two occurred.

### What a seam is not

The properties below are what a seam must *keep* rather than accidents of the bounded profile, and ADR 0078 makes them normative. They are stated here because this contract owns the provider surface each constrains. Two of them the list above already states in terms — that missing optional knowledge is conservative, and that an absent capability and a contended one are different findings — and they are deliberately not restated.

**Offering nothing is a legitimate local result, not an error and not a licence to approximate.** A provider that recognizes no work for a region and target offers none; enumeration then succeeds with nothing admitted, and a region no provider can implement is reported by complete-plan selection as unimplemented. No offer, a typed rejection, and a compiler fault stay three distinct outcomes. A provider that emits structurally invalid IR fails the whole enumeration closed rather than being dropped into the empty case, which is indistinguishable from outside and is a defect rather than an absence.

**An unenumerated capability fails closed as `Unknown`; it never defaults to supported.** "Conservative" above means a compile refusal for index/access lowering; for fusion legality it means an explicit `Unknown`. An operation family with no registered fusion role classifies as `Unknown` rather than an approximated accept, and normative guarantees, sound proofs, exhaustive finite evidence, empirical evidence, and `Unknown` remain five classes that are never collapsed into one another.

**An exhausted analysis budget is an `Unknown` gap, not a rejection and not an admission.** When a checked proof cannot afford a region, the compiler retains a typed ResourceLimit assessment naming the resource, its original whole-call limit, and the cumulative required amount, and nothing about that obligation was disproved. [ADR 0109](decisions/0109-fail-closed-before-executable-planning-when-index-domain-proof-is-unknown.md) makes the fail-closed boundary and the mixed-assessment rule normative: every produced assessment remains explainable, an assessed `Disproved` claim takes overall precedence over a ResourceLimit Unknown, and otherwise any Unknown refuses before executable planning and coverage.

**A reservation is not a capability.** A reserved body variant exists so an unsupported proposal rejects explicitly under its own name instead of being approximated by the nearest thing the bounded profile can emit; it carries an uninterpreted marker echoed into the rejection and read nowhere else. `View` is the one body variant still reserved on that footing. `KernelSubprogram` and `OpaqueCall` are no longer reservations and must not be cited as ones: the first is admitted as a verified chain of stages, and the second carries the typed ABI, effect, alias, placement, target, numerical, resource, and failure-stage boundary this contract requires above. A reservation becoming a capability is the expected direction of travel and is why the seam is additive; the error to keep avoiding is the reverse reading, treating a variant that only rejects as evidence of the support it names.

**A provider's revision is provenance, not a version negotiation.** A capability key carries its provider identity while resolution selects on family, operation, and signature alone, so no revision supersedes another, no later registration wins, and there is no precedence order and no default provider. **Measured, having been recorded here as an inference until [`test-two-revisions-of-one-provider-as-a-capability-ambiguity`](../tickets/test-two-revisions-of-one-provider-as-a-capability-ambiguity.md) pinned it:** two revisions of one provider register as two distinct keys that one selector matches, producing the same contention two unrelated providers would. `capability::tests::two_revisions_of_one_provider_resolve_to_an_ambiguity` observes both registrations succeeding, resolution returning an ambiguity, and the candidates being both identities in canonical ascending order in either registration order.

**Contention at one seam is not contention at another, and neither rule generalizes.** Two providers claiming one occurrence is unresolvable in the lowering registry, because exactly one authority may define how an occurrence lowers. Two providers proposing an implementation of one region is additive: both are retained, separated by their folded provenance, and chosen between later on cost. A region may legitimately have several correct implementations; an occurrence may not have several meanings.

An extension's semantic-equivalence claim remains trusted. Host verification
can establish structural, typing, shape, memory-safety, and declared numerical
obligations; it cannot generally prove arbitrary replacement mathematics.
Conformance vectors, reference evaluation, differential tests, and negative
precondition tests are therefore mandatory evidence.

### Purity and future effects

Initial extension operations must declare and satisfy the pure operation
contract. In particular, floating-point exception cases may return resolved
values or explicit tensor diagnostics, but may not observe or mutate hidden
status flags, trap state, or another ambient floating-point environment.

This is a capability boundary rather than a permanent exclusion. The durable
operation and value model reserves a separately versioned effect signature and
resource/effect-token value kinds. Adding them requires host-owned ordering,
liveness, verification, lowering, ABI, and partial-execution rules. Existing
pure operation keys keep their meaning; an effectful revision uses a new
compatible identity/schema, and a compiler lacking that capability rejects it.
No extension may smuggle an effect through a `pure` declaration.

## Transactional rewrites and termination

Extension rules do not receive unrestricted mutable graph access. A rule
returns a proposed replacement through a transactional rewriter; the host
validates the replacement before commit.

Rules declare stable rule/provider IDs, generated operation sets, preconditions,
required numerical permissions, and deterministic tie-breaking. Per-rule and
global budgets, cycle detection, and bounded recursive application prevent
nontermination. A proof object may discharge host-checkable obligations but is
not treated as a general proof of semantic equivalence.

## Failure and panic boundaries

Each extension callback requires a diagnostic boundary. A future higher
compiler-session boundary may catch unwinding panics, discard the in-progress
transaction, and report provider/rule identity. This is containment rather
than sandboxing: aborting
panics, hangs, native memory unsafety, and malicious code cannot be recovered
reliably. The host passes only shared provider references and requires
`Send + Sync`, but Rust interior mutability can still change provider-owned
state. Determinism forbids output-affecting hidden mutation; trusted provider
implementations, not the type system, uphold that obligation.

The current semantic inference boundary is synchronous and treats providers as
trusted in-process Rust code. The host supplies an immutable request and a
non-constructible result writer. Every ordered result must pass through
`try_push`; result-count and aggregate canonical result-fact byte budgets are
checked before the host retains the fact, and the first writer failure remains
sticky even when provider code ignores it. The host commits results only after
the callback returns success, the writer remains valid, minimum arity is met,
and every result fact passes semantic-registry validation. A provider error
after writer failure is retained as independent secondary evidence, not
misrepresented as its causal `Error::source`.

Before any type-family validator, attribute-authority callback, or operation
inferencer runs, the host preflights operand arity and attribute field/kind
structure against the immutable schema. Full validation remains in the later
inference path; preflight defines callback ordering and fail-fast behavior, not
a weaker alternate verifier.

Stable diagnostic classes use a validated, cheaply cloned
`ProviderDiagnosticCode`. Dynamic messages are bounded before host copying or
retention. Malformed messages become a typed provider-contract failure wrapped
causally by the operation- or type-validation error; Tiler neither truncates
them nor silently substitutes a semantic rejection. The two rejection roles
remain distinct, and no diagnostic template/argument schema is committed yet.

These budgets constrain host-accepted semantic structure and canonical
identity work. They are deliberately not claims about exact heap consumption:
allocator overhead, trait-object state, and shared `Arc` storage make such a
claim false. Providers can still allocate or loop before calling the writer.
The current semantic prototype also does not catch provider panics itself. With
unwinding enabled, a panic propagates before graph mutation and callers may
catch it; aborting panics remain unrecoverable. Provider attribution at a
higher compiler-session panic boundary remains future work.

## Unknown operations and serialization

The initial verified graph API rejects unknown `OpKey` values. Unknown-operation
round-trip belongs to a future bounded `ParsedGraph` or tooling envelope and
does not imply purity, valid inference, canonical equivalence, evaluability, or
compilability.

Stable public serialized IR is deferred. Any private/version-locked decoder
validates framing, schema, canonical encoding, resource limits, checked
arithmetic, duplicate IDs/fields, use-def structure, and acyclicity before
calling extension code. Deserialization never loads code named by input bytes.

## Rust API evolution

Do not begin with one large downstream-implemented trait. The initial shape is
an explicit per-session `RegistryBuilder` frozen into an immutable canonical
snapshot, one small dyn-compatible semantic definition, and separately
versioned optional capability objects using sealed/opaque host contexts. All
initial provider objects are `Send + Sync + 'static`. This reduces dyn
compatibility, coherence, and semver hazards while allowing capability growth.
Exact names, allocation types, borrowed contexts, and builder ergonomics remain
experimental.

Optional `inventory`- or linker-style adapters may populate the explicit
builder for environments where their ordering and linkage are understood.
They do not replace the builder, define precedence, or solve proc-macro
visibility.

## Required conformance tests

- shuffled, parallel, and repeated registration produces one canonical
  snapshot;
- duplicate semantic ownership and provider conflicts are rejected;
- semantic keys, provider-independent definitions, and provider revisions
  affect only their separately specified identity subjects;
- canonical/noncanonical and oversized attributes are accepted/rejected
  exactly as specified;
- callbacks are checked for deterministic results under repeated/concurrent
  invocation;
- inference/verification and decomposition/lowering contracts cannot disagree
  silently;
- rewrites are transactional, reverified, cycle-bounded, and budgeted;
- callback panics cannot commit partial graph state; a future recovery boundary
  must attribute any caught panic to its provider;
- semantic-precondition declarations reject duplicate meaning/subject pairs and out-of-range selectors; known disproof commits no graph mutation; unknown predicates stay residual; declaration order, predicate revision, invalid-input code, and canonical occurrence/subject coordinates perturb only their governed identity subjects;
- unknown operations never enter `VerifiedSemanticGraph`;
- malformed serialized input cannot trigger extension code before structural
  and resource validation.

## Primary precedents

- [Rust procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)
  establish the separate token-driven compilation boundary and build-script-like
  trust model.
- [Rust trait dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility),
  [trait objects](https://doc.rust-lang.org/reference/types/trait-object.html),
  and [Cargo semver guidance](https://doc.rust-lang.org/cargo/reference/semver.html)
  constrain a public capability API.
- [`TypeId`](https://doc.rust-lang.org/core/any/struct.TypeId.html) and
  [Rust type layout](https://doc.rust-lang.org/reference/type-layout.html) are
  explicitly unsuitable as stable cross-build identities or plugin ABIs.
- [MLIR interfaces](https://mlir.llvm.org/docs/Interfaces/) provide precedent
  for promised capability checks and dialect-wide fallback interfaces.
- [MLIR pattern rewriting](https://mlir.llvm.org/docs/PatternRewriter/)
  provides precedent for transactional mutation, rewrite recursion controls,
  and bounded application.
- [MLIR bytecode format](https://mlir.llvm.org/docs/BytecodeFormat/) illustrates
  why extensible serialized IR needs dialect versioning and upgrade contracts.
