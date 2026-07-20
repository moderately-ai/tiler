---
schema: "tiler-doc/v1"
id: "tiler.research.extensions.operation-extension-surface"
kind: "research"
title: "Operation-extension surface research"
topics: ["extensions", "operations", "api"]
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.operation-extensions"]
adopted_by: ["ADR-0005", "ADR-0044", "ADR-0052"]
reproduced_by: ["tiler.spike.extensions"]
ticket: "operation-extension-surface"
---

# Operation-extension surface research

**Question:** What public extension boundary lets built-in and third-party
tensor operations participate in one semantic graph without making meaning,
identity, or optimization depend on arbitrary Rust implementation details?

## Findings from primary precedents

### MLIR

[MLIR interfaces](https://mlir.llvm.org/docs/Interfaces/) separate IR entities
from optional operation- and dialect-wide behaviors. Transformations query the
interfaces they require and must be conservative when an interface is absent.
External interface models can attach behavior without changing the operation
class, while promised interfaces make a missing required registration fail
rather than silently changing behavior.

[MLIR pattern rewriting](https://mlir.llvm.org/docs/PatternRewriter/) also keeps
mutation behind a rewriter that records replacements and controls recursion.
The useful lesson is capability-based participation with host-owned mutation;
MLIR's process-global context and C++ type identity are not suitable durable
identities for Tiler artifacts.

### StableHLO

The [StableHLO specification](https://openxla.org/stablehlo/spec) distinguishes
two important extension cases:

- `composite` has named, versioned semantics and a decomposition that can
  replace it without changing meaning;
- `custom_call` delegates to implementation-defined behavior and carries
  backend metadata, side-effect, version, and alias information.

This supports a strict Tiler distinction between semantic operations with
normative meaning and typed physical implementations. An arbitrary opaque
callback is not a semantic definition and must not acquire fusion or rewrite
privileges merely by being registered.

### TVM

TVM's [operator registry](https://tvm.apache.org/docs/reference/api/doxygen/ir_2op_8h.html)
associates named primitive operations with separately registered attributes.
This validates independently extensible capabilities, but process-global
registration, mutable attribute attachment, and explicit override behavior are
poor defaults for reproducible compilation. Tiler needs session-local freeze,
collision rejection, and canonical ordering.

### Rust boundaries

The [Rust procedural macro reference](https://doc.rust-lang.org/reference/procedural-macros.html)
defines a separately compiled token-to-token component. It cannot receive a
trait object constructed later in the consumer crate merely because that type
implements a public trait. The core compiler API can therefore accept ordinary
Rust providers, but visibility through an inline macro requires a separate
feasibility mechanism.

The [dyn compatibility rules](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility)
and [Cargo semver guidance](https://doc.rust-lang.org/cargo/reference/semver.html)
argue against one large downstream-implemented trait. Adding required methods
to such a trait would break implementers. A small root plus versioned optional
capabilities leaves room to grow.

## Alternatives

| Model | Advantage | Failure mode |
|---|---|---|
| Closed built-in enum | Exhaustive matching and simple implementation | Cannot prove the accepted public vertical extension path; each new official operation edits central code |
| One monolithic public trait | Familiar Rust surface | Couples semantics, optimization, targets, and diagnostics; required-method additions are breaking; unsupported phases become ambiguous stubs |
| Automatic linker/global registration | Convenient call sites | Hidden global state, collision/order ambiguity, weak session isolation, and no solution to the proc-macro crate boundary |
| Explicit registry builder and capability set | Deterministic, testable, session-scoped, supports multiple frontends | More explicit setup; macro visibility still needs a separate mechanism |

The explicit registry is the only model compatible with deterministic
compilation and the accepted extension direction. Optional discovery adapters
may populate the builder, but they cannot define registry semantics.

## Recommended contract

### Durable data versus executable behavior

Durable semantic IR contains only:

```text
OpKey
canonical attributes
ordered operand ValueIds
ordered result types/ValueIds
```

It never contains a trait object, callback, Rust `TypeId`, vtable/function
address, registry slot, or insertion order. A frozen registry resolves the key
to trusted compiler-process behavior.

### Mandatory semantic definition

Exactly one semantic authority owns an `OpKey`. It supplies immutable,
host-readable data and deterministic validation:

- durable key and bounded operand/result/value-kind schema;
- host-owned canonical attribute schema, defaults, and semantic validation;
- pure effect signature for the initial graph;
- deterministic shape, dtype, axis, and constraint inference/validation;
- normative semantic specification identity and conformance vectors;
- stable names and documentation used by host-owned explain output.

Normative meaning is mandatory; one particular executable evaluator is not.
An operation may become executable or transformable only if it additionally
has either an exact verified decomposition, a compatible reference evaluator
where the phase requires one, or a typed implementation capability satisfying
that phase's obligations. Thus a semantically valid operation can remain an
explicit compilation boundary without being ambiguous.

### Optional capabilities

Capabilities are independently versioned provider objects rather than methods
on one universal trait:

- reference evaluation;
- exact decomposition;
- canonicalization and semantic rewrite rules;
- iteration-domain/access lowering and fusion participation;
- typed opaque physical implementation;
- structured-kernel lowering;
- target feasibility and cost evidence;
- numerical-accuracy realization and conformance evidence;
- provider-specific diagnostic detail.

Differentiation, effects/resources, dynamic loading, public serialized IR, and
automatic proc-macro discovery are later capability families, not placeholder
methods in the initial root trait.

### Authority boundaries

The provider declares schemas, semantics, preconditions, and proposed
transformations. Tiler owns:

- canonical attribute representation and encoding;
- registry ordering, collision handling, and freeze;
- graph mutation transactions and reverification;
- resource budgets and recursion limits;
- stable diagnostic codes, structural context, and explain formatting;
- identity projection and artifact/cache hashing;
- target-profile and numerical-contract verification.

Provider prose and pretty diagnostic fragments may enrich output but cannot
replace host-owned machine-readable reasons.

### Registry and concurrency

The initial registry is built explicitly per compiler/session, then frozen into
an immutable `Send + Sync + 'static` snapshot. It has:

- one semantic registration per `OpKey`;
- zero or more named capability providers per operation and capability kind;
- hard rejection of duplicate semantic ownership and duplicate capability
  slots;
- explicit deterministic provider selection rather than last-wins precedence;
- canonical ordering independent of insertion, link, or map iteration order;
- no provider calls before structural bounds and canonical data validation.

Providers are trusted native code with compiler-process privileges. Panic
catching can preserve a diagnostic boundary under unwind builds, but does not
sandbox aborts, hangs, data races, or unsafe memory access.

## Identity layers

Three identities must not be collapsed:

1. **Semantic graph identity** contains operation semantic keys and canonical
   graph data. Provider implementation revisions do not change graph meaning.
2. **Compilation request provenance** records the complete frozen registry and
   options so an invocation is explainable and reproducible.
3. **Selected plan/artifact identity** contains semantic authorities actually
   reached and capability providers actually selected, plus their revisions
   and emitted output. An unused registered provider must not invalidate an
   otherwise identical artifact.

`semantic_version` changes when operation meaning or schema compatibility
changes. `provider_revision` changes when output-affecting implementation
behavior changes without changing the operation's meaning. Capability API
versions independently identify the host/provider calling contract.

## Failure matrix

| Case | Required result |
|---|---|
| Unknown `OpKey` in verified graph | Reject before invoking extension code |
| Duplicate semantic authority | Deterministic hard error |
| Duplicate provider slot or ambiguous selection | Deterministic hard error |
| Opaque/noncanonical/oversized attribute | Reject in host validation |
| Provider key disagrees with registered key | Reject registry freeze |
| Empty, malformed, or stale declared revision | Reject or fail conformance policy; never silently accept |
| Missing optional capability | Preserve semantic graph; conservatively block that phase |
| Callback panic under unwind | Discard transaction and return provider-attributed diagnostic |
| Repeated callback produces different result | Provider contract violation and hard diagnostic |
| Rewrite emits invalid graph | Reject proposal without mutating the accepted graph |
| Capability refers to absent semantic authority | Reject registry freeze |

Static type signatures cannot prove semantic determinism. Debug/conformance
modes should repeat and permute calls, while release correctness relies on the
trusted-provider contract, revisions, conformance vectors, and differential
tests.

## Spike result

[`spikes/extensions/operation-api`](../../../spikes/extensions/operation-api)
compile-checks the proposed split using only the standard library. It proves
that:

- small dyn-compatible capabilities can coexist with host-owned canonical IR;
- an explicit builder can freeze into canonically ordered immutable state;
- duplicate semantic ownership and malformed revisions are rejected;
- request provenance and selected artifact provenance can be projected
  separately;
- panic attribution and repeated-call determinism checks fit outside provider
  traits.

The spike intentionally does not settle allocator choices, exact borrowed
context types, hashing/serialization formats, or proc-macro discovery. Those
are implementation or separate feasibility questions.
