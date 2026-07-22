---
schema: "tiler-doc/v1"
id: "tiler.contract.operation-extensions"
kind: "contract"
title: "Operation extension contract"
topics: ["extensions", "operations", "semantics"]
contract_status: "mixed"
implementation_status: "spike-only"
evidence: ["tiler.research.extensions.operation-extension-surface", "tiler.research.extensions.operation-extension-api", "tiler.research.extensions.proc-macro-extension-visibility"]
---

# Operation extension contract

**Status:** proposed details under the accepted public extension direction

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

The implementation separates the semantic portion required by `tiler-ir` from
later executable capabilities. Under ADR 0066, `FrozenSemanticRegistry` is a
cheap-clone owned snapshot of nominal definitions, parameterized constructors,
encoded schemes, operation definitions, provider provenance, and optional
local marker bindings. Semantic programs retain it so checked Rust reification
does not require a borrowed context. Registration callbacks are discarded at
freeze; immutable definition objects required for concrete-type validation and
operation inference remain in the snapshot under narrow host-owned contexts.
Under ADR 0065, executable reference capabilities compose in
`tiler-reference`; optimizer and backend registries likewise compose later
rather than introducing an inward dependency from semantic IR.
Provider objects are expected to be `Send + Sync + 'static` unless an explicit
compiler mode serializes a capability.

## Semantic and provider identity

`OpKey { dialect, name, semantic_version }` identifies semantic meaning and
schema compatibility, not one Rust implementation. Every selected provider
declares a stable provider ID and revision/fingerprint covering all
output-affecting behavior it owns, including inference, evaluation,
decomposition, rewriting, lowering, or code generation.

Provider revisions are an explicit author trust contract, not automatic source
attestation. Changing output-affecting behavior without changing the declared
revision is a provider bug. Compiler and capability-API versions also
participate in identity.

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

The accepted v1 model is the discriminated `CanonicalAttrValue` defined in
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
- Missing optional knowledge is conservative.
- Contradictory capability answers are hard diagnostics, not fallback misses.

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
