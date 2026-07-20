---
schema: "tiler-doc/v1"
id: "tiler.contract.artifact-abi"
kind: "contract"
title: "Artifact envelope and Metal kernel ABI profile"
topics: ["artifacts", "abi", "metal", "runtime"]
contract_status: "accepted"
implementation_status: "not-started"
evidence: ["tiler.research.artifacts.target-neutral-envelope", "tiler.research.cache.crash-race-protocol", "tiler.research.runtime.execution-contract"]
ticket: "synthesize-artifact-contracts"
---

# Artifact envelope and Metal kernel ABI profile

**Status:** accepted research contract; concrete serialization remains internal

## Ownership boundary

This document owns the target-neutral envelope, serialized program portfolio,
ABI roles, routing commit, guards, digests, and backend payload boundaries. The
IR contract owns compiler-model meaning and schedule legality; adapters own
device-specific loading, binding, and execution.

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

Integrity, structural validity, neutral-program validity, backend-payload
validity, declared target compatibility, live applicability, prepared-entry
feasibility, and launch feasibility are distinct monotonic validation stages.
Parse success never implies executable compatibility. See the
[target-neutral envelope research](research/artifacts/target-neutral-artifact-envelope.md).

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

```text
semantic_digest = H("tiler-semantic-v1" || canonical semantic bytes)
scheduled_digest = H("tiler-schedule-v1" || semantic_digest
                     || canonical scheduled bytes)
plan_digest = H("tiler-program-v1" || semantic_digest
                || canonical program bytes)
section_digest[i] = H("tiler-section-v1" || section_type/schema
                      || exact section bytes)
manifest_digest = H("tiler-manifest-v1" || exact canonical manifest bytes)
envelope_digest = H("tiler-envelope-v1" || exact complete envelope bytes)
```

Section digests are stored only in manifest section descriptors. The manifest
digest is stored only in the framing header and covers the exact manifest bytes,
which contain no `manifest_digest` or `envelope_digest` field. `EnvelopeDigest`
is externally derived and never stored inside the envelope it covers. Semantic,
scheduled, and plan digests may appear as cross-reference values, but their
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

## Traceability

This document owns the neutral artifact envelope and Metal ABI profile. It does
not own backend scheduling or consumer storage. Its governing decisions and
supporting research are declared in frontmatter; unresolved serialization and
compatibility work remains explicit above.
