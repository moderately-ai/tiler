# Operation extension contract

**Status:** proposed details under the accepted public extension direction

ADR 0005 accepts a public experimental vertical extension boundary. This
document proposes the initial safety, determinism, and compilation-phase
contract; it does not yet commit exact Rust trait signatures.

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
- a proc macro initially supports providers compiled into its own dependency
  graph or described through a future explicit declarative mechanism;
- consumer-side automatic registration is not assumed to cross the proc-macro
  compilation boundary.

This limitation is an integration feasibility gate, not a reason to make the
compiler-core extension boundary consumer-specific.

## Registry lifecycle and coherence

Registration uses an explicit per-compiler/session builder. Before graph
verification or optimization it freezes into an immutable snapshot:

- durable ordering is by operation/provider key, never insertion or link order;
- duplicate semantic `OpKey` ownership is a hard error, never last-wins;
- one semantic authority defines each operation's meaning and schema;
- additional decomposition, lowering, scheduling, or cost providers have
  independently named provider identities and declared compatibility;
- collisions or contradictory provider selections fail deterministically;
- registry contents and selected provider revisions participate in compilation
  provenance and output identity;
- Rust `TypeId`, vtable addresses, function addresses, registration addresses,
  and hash-map randomization never participate in durable identity.

The frozen registry is immutable and safe for concurrent read-only use.
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

Element types follow the same durable identity principle. A conceptual
`TypeKey { namespace, name, semantic_version }` identifies a canonical dtype
for both built-ins and extensions. Built-ins may expose convenient constants or
enum-like spellings, but durable IR never substitutes Rust discriminants,
`TypeId`, implementation addresses, or unversioned display names for the type
key. The descriptor attached to a key defines structural and value semantics;
provider identity separately records the implementation that supplies
capabilities for it.

A canonical type key is not rewritten when its support level changes. If Tiler
later bundles support for `acme::fp8_special@1`, it supports that existing
identity rather than relabeling it as a new `tiler` type. Frontend aliases may
improve spelling, but aliases resolve to a canonical key before semantic
admission and never create identity equivalence implicitly. Namespace
ownership, collision handling, provider compatibility, and durable descriptor
encoding require the same deterministic registry discipline as operations;
their exact Rust API remains open.

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

## Capability coherence

Capability callbacks are immutable and deterministic functions of explicit
inputs. They may not depend on undeclared environment state, time, randomness,
mutable global state, registry order, or call order.

- Inference results are rechecked by host graph verification.
- Decomposition and rewrite output re-enters full semantic verification.
- Lowering declares its numerical, shape, effect, operation-set, and target
  preconditions in machine-checkable form where possible.
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

Each extension callback has a diagnostic boundary. Where unwinding is enabled,
a panic may be caught to discard the in-progress transaction and report the
provider/rule identity. This is containment rather than sandboxing: aborting
panics, hangs, native memory unsafety, and malicious code cannot be recovered
reliably. Provider state is immutable; partially mutated provider state is not
reused.

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

Do not begin with one large downstream-implemented trait. The likely shape is a
small dyn-compatible root descriptor plus separately versioned optional
capability objects and sealed/opaque host contexts. This reduces object-safety,
coherence, and semver hazards while allowing capability growth. Exact trait
splitting, allocation types, registration containers, and builder ergonomics
remain implementation decisions to prototype before stabilization.

## Required conformance tests

- shuffled, parallel, and repeated registration produces one canonical
  snapshot;
- duplicate semantic ownership and provider conflicts are rejected;
- semantic keys and provider revisions affect the intended identities;
- canonical/noncanonical and oversized attributes are accepted/rejected
  exactly as specified;
- callbacks are checked for deterministic results under repeated/concurrent
  invocation;
- inference/verification and decomposition/lowering contracts cannot disagree
  silently;
- rewrites are transactional, reverified, cycle-bounded, and budgeted;
- callback panics produce provider-attributed diagnostics where recoverable;
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
