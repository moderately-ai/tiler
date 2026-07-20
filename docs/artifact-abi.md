# Proposed Metal artifact and kernel ABI profile

**Status:** proposed

This document describes the proposed first-backend Metal profile of Tiler's
target-neutral artifact concepts. `MetallibBundle`, Metal binding indices, and
direct Rust embedding are profile-specific; the compiler core must also admit
other target payloads and delivery mechanisms.

A metallib alone is not executable safely. The Metal profile pairs compiled code with a
versioned, machine-checkable contract describing executable plans, bindings,
formulas, guards, routing, numerical behavior, and target requirements.

## Artifact hierarchy

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

## Conceptual schema

```rust
struct MetallibBundle {
    format_version: u32,
    ir_version: u32,
    codegen_version: String,
    target: MetalTarget,
    target_profile_id: String,
    target_profile_descriptor_hash: Hash256,
    capability_schema_versions: Vec<SchemaVersion>,
    feasibility_rule_set: RuleSetIdentity,
    artifact_execution_contract: ArtifactExecutionContract,
    cost_model_version: String,
    compiler: CompilerFingerprint,
    manifest_hash: Hash256,
    metallib_hash: Hash256,
    bundle_hash: Hash256,
    metallib: Bytes,
    kernels: Vec<KernelEntry>,
    programs: Vec<ProgramPlan>,
}

struct ProgramPlan {
    semantic_hash: Hash256,
    inputs: Vec<TensorSpec>,
    outputs: Vec<TensorSpec>,
    semantic_root_bindings: Vec<SemanticRootBinding>,
    semantic_constraints: Vec<SemanticConstraint>,
    numeric_contract: NumericContract,
    variants: Vec<PlanVariant>,
    routing: RoutingPolicy,
}

struct PlanVariant {
    plan_hash: Hash256,
    applicability_guards: Vec<RuntimeGuard>,
    target_requirements: Vec<TargetRequirement>,
    deferred_target_checks: Vec<DeferredTargetCheck>,
    buffer_plan: BufferPlan,
    temporaries: Vec<TemporaryTensor>,
    steps: Vec<KernelStep>,
}

struct BufferPlan {
    allocations: Vec<Allocation>,
    value_bindings: Vec<ValueAllocationBinding>,
    lifetime_intervals: Vec<LifetimeInterval>,
}

struct KernelStep {
    kernel_entry: KernelEntryId,
    tensor_bindings: Vec<PlanValueId>,
    dependencies: Vec<StepId>,
    numeric_realizations: Vec<NumericRealizationRef>,
}

struct KernelEntry {
    scheduled_hash: Hash256,
    symbol: String,
    bindings: Vec<Binding>,
    dispatch: DispatchFormula,
    function_constants: Vec<FunctionConstantSpec>,
    static_threadgroup_bytes: u32,
    resource_requirements: ResourceRequirements,
}
```

This is illustrative, not a committed Rust API or serialization format. The
Milestone 2 one-kernel path is a program with one variant, no temporaries, and
one step.

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

## Embedding contract

The proc macro embeds the canonical manifest and metallib as byte-string literal
tokens in its returned Rust expression. Runtime artifact construction borrows
those static byte slices; it does not open source files, compiler-cache paths,
or consumer `OUT_DIR`.

The embedding representation is deterministic and versioned. Artifact identity
is independent of the absolute compiler-cache location. Direct embedding size,
rustc memory, incremental behavior, and repeated-literal binary duplication are
measured and bounded. A later linker-level deduplication mechanism may change
storage without changing bundle semantics or call-site DX.

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

Expansion compilation identity includes canonical scheduled IR and complete
program plans, semantic root-binding declarations, ABIs, guards, routing,
dispatch, numerical contract, translation-unit membership,
schema/helper/codegen versions, target/profile, compiler, flags, and every
selected conformance-evidence record digest and scope.

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
semantic_hash  = H(canonical semantic graph + semantic root-binding interface
                   + operation contract)
scheduled_hash = H(semantic_hash + scheduled IR + target/profile/policy)
plan_hash       = H(semantic_hash + canonical steps/temps/guards)
manifest_hash  = H(canonical embedded manifest payload with hash fields and metallib
                   byte payload excluded)
metallib_hash  = H(raw metallib bytes)
bundle_hash    = H(format tag || manifest_hash || metallib_hash)
```

Stable canonical IR, MSL, manifest, and cache keys are required. Byte-identical
metallibs are promised only within a verified pinned toolchain/environment.

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
