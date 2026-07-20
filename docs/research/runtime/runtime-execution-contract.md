---
schema: "tiler-doc/v1"
id: "tiler.research.runtime.execution-contract"
kind: "research"
title: "Consumer-neutral runtime execution contract"
topics: ["runtime", "routing", "fallback", "resource-lifetime"]
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.artifact-abi", "tiler.contract.candle-integration"]
adopted_by: ["ADR-0051"]
reproduced_by: ["tiler.spike.runtime"]
ticket: "runtime-execution-contract"
---

# Consumer-neutral runtime execution contract

**Status:** completed research adopted by ADR 0051 and the runtime-facing contracts
**Ticket:** `runtime-execution-contract`

## Outcome

Tiler should expose a prepared-program execution boundary rather than a
backend launch helper. The boundary binds one verified artifact program to one
live device/context, proves all route-sensitive facts, prepares every library
and pipeline required by one complete variant, consumes fallback authority at
`RoutingCommit`, and only then permits program allocation or encoding.

The first execution profile has one live device and one ordered command
stream. It executes a canonical topological order, returns ordered named
outputs, and retains inputs, outputs, temporaries, argument storage, libraries,
and pipelines until their exact final device use completes. A normal execution
may publish dependency-carrying outputs asynchronously. Any validation record
that the host interprets instead requires successful terminal completion of
the exact submission that produced and synchronized that record before
readback.

Fallback is a precommit routing action, not error recovery. Applicability and
typed capability misses may reject a candidate before `RoutingCommit` and try
another semantically equivalent complete program. Artifact defects, semantic
input errors, systemic preparation errors, stale-preparation invariants, and
every failure after commit are returned as typed errors. In particular, there
is no fallback after allocation, partial encoding, submission, device failure,
or semantic validation failure.

This contract is consumer-neutral. Candle is evidence that an adapter must own
framework-specific storage, command-stream, completion, and output hooks; it
is not part of the core abstraction.

## Evidence and classification

### Primary-source facts

- Metal resources and command queues are created from a particular
  [`MTLDevice`](https://developer.apple.com/documentation/metal/mtldevice).
  Device-bound libraries, functions, pipelines, resources, and command objects
  therefore cannot be placed in a device-agnostic runtime cache.
- Apple defines `Completed` and `Error` as distinct terminal
  [`MTLCommandBufferStatus`](https://developer.apple.com/documentation/metal/mtlcommandbufferstatus)
  values. [`waitUntilCompleted`](https://developer.apple.com/documentation/metal/mtlcommandbuffer/waituntilcompleted%28%29)
  waits but returns no success value; execution error detail is obtained from
  [`MTLCommandBuffer.error`](https://developer.apple.com/documentation/metal/mtlcommandbuffer/error).
  A pre-wait non-error status is therefore not evidence of successful
  completion.
- Metal pipeline creation is a distinct fallible step from loading a library.
  Prepared-pipeline properties such as maximum total threads are properties of
  the compiled pipeline, not consequences of metallib load alone. See
  [`MTLDevice.makeComputePipelineState`](https://developer.apple.com/documentation/metal/mtldevice/makecomputepipelinestate%28function:options:reflection:%29)
  and
  [`MTLComputePipelineState`](https://developer.apple.com/documentation/metal/mtlcomputepipelinestate).
- IREE Stream represents resource sizes and lifetimes explicitly and orders
  asynchronous availability with timepoints. This is precedent for retaining
  resources against completion dependencies rather than host encoding scope.
  See the [IREE Stream dialect](https://iree.dev/reference/mlir-dialects/Stream/).
- The accepted Tiler artifact envelope separates integrity, neutral schema,
  backend payload, target compatibility, live preflight, and prepared-entry
  validation. It defines `RoutingCommit` only after route-sensitive launch
  preflight.
- The accepted `KernelProgram` profile requires one ordered stream, explicit
  typed dependencies, complete named outputs, and resource retention through
  device completion. Cross-dispatch scratch is an ordinary typed temporary.
- The accepted semantic-enforcement contract distinguishes `RoutingCommit`,
  `EnforcementCommit`, and `PublicationCommit`; a validation miss is a semantic
  error and never an applicability miss.

### Measured local evidence

At local Candle commit `31f35b147389700ed2a178ee66a91c3cc25cc80d`
(version 0.11.0), source inspection and the accepted transition spike establish
that `Commands::ensure_completed` observes command-buffer status before its
wait but not after it. The modeled committed/scheduled-to-error transition can
therefore return success. Nine transition tests pass and include the negative
case. This is a source/control-flow measurement, not a real-GPU fault
measurement. Details are in
[Candle Metal post-wait error checking](candle-metal-post-wait-error-checking.md).

### Inferences

1. A prepared selection must be scoped by live device/context identity and
   exact input/resource generations. Artifact identity alone cannot make a
   prepared pipeline or binding reusable.
2. Route selection must finish after all required entry pipelines exist and
   their authoritative facts have passed launch preflight. Otherwise a
   pipeline failure after allocation would tempt an unsafe fallback.
3. Fallback authority should be represented by ownership, not a Boolean flag.
   `RoutingCommit` consumes the only fallback token and produces an execution
   token; encoding APIs accept only the latter.
4. Encoding order is not completion. Releasing a temporary after its last
   encoder call, or reading a validation record after an unchecked wait, is
   unsound.
5. A later queue submission reaching a terminal state can establish ordering,
   but does not by itself prove that an earlier submission succeeded. A
   validation readback needs success evidence for the exact execution unit
   that contains the validator and required coherence operation.
6. Pipeline cache failures need typed classification. A stable unsupported
   entry can be a candidate capability miss; an out-of-memory, driver, corrupt
   library, or adapter failure is not evidence that another plan is applicable.

### Proposals

Everything below is the proposed Tiler runtime contract. Type names are
illustrative and do not commit a Rust API or serialization.

## Runtime identities and immutable inputs

```text
RuntimeArtifactKey = EnvelopeDigest + selected BackendPayloadDigest

LiveDeviceKey {
  backend_provider_key,
  runtime_instance_key,
  provider_device_token,
  context_or_queue_family_scope,
}

PreparedEntryKey {
  live_device_key,
  runtime_artifact_key,
  backend_entry_key,
  specialization_values,
  canonical_pipeline_descriptor,
  translation_or_archive_mode,
}

InputBindingFingerprint {
  plan_value_and_component_roles,
  resource_identity_and_generation,
  live_device_key,
  dtype_and_storage_encoding,
  shape_strides_and_logical_start,
  allocation_length_and_accessible_range,
  access_and_alignment,
}
```

`LiveDeviceKey` is runtime-scoped. A bare ordinal, object address, or portable
artifact target is not sufficient identity. The provider decides the stable
token within its declared runtime-instance scope. A new context, device reset,
resource generation, pipeline descriptor, specialization, or translation mode
produces a different relevant key.

The executor accepts only a device-free validated artifact. It does not parse
untrusted framing, normalize a noncanonical manifest, repair cross-references,
or infer backend ABI mappings while launching. Those are earlier monotonic
validation boundaries whose evidence is carried in a `ValidatedArtifact`.

## State machine

```text
ValidatedArtifact
  -> bind live device, semantic roots, inputs
BoundRequest
  -> evaluate semantic constraints and host expressions
  -> enumerate guard-applicable complete variants
CandidateSet
  -> load/cache libraries
  -> prepare/cache every entry of a candidate
  -> query prepared facts and validate launch instances
  -> verify enforcement, allocator, stream, retention, publication capabilities
PreparedSelection + FallbackAuthority
  -> atomically revalidate volatile fingerprints
  -> consume FallbackAuthority
RoutingCommitted
  -> allocate output, temporary, validation, and publication resources
ResourcesAcquired
  -> EnforcementCommit when residual enforcement begins, if any
  -> encode canonical dependency-preserving stage order
Encoding
  -> submit and register resource retention against exact completion receipts
InFlight
  -> optionally observe exact successful completion/coherence/error record
ValidationObserved
  -> publish ordered named outputs or return semantic/runtime error
Published | Failed
```

`Failed` is terminal for this execution. Cleanup may continue until outstanding
device uses complete, but cleanup cannot change the returned failure into a
fallback route.

### Transition contract

| Transition | Work permitted | On failure | Fallback? |
| --- | --- | --- | --- |
| artifact to bound request | device lookup, input inspection, semantic binding | artifact/binding/semantic error | no for defects; outer fallback only after a successfully bound equivalent interface |
| bound request to candidates | pure checked expressions, guards, typed live queries | applicability or capability miss; invariant/system error | yes only for the typed miss |
| candidate to prepared selection | library/function/pipeline preparation, prepared-fact queries, launch and adapter-capability checks | typed candidate capability miss or fatal preparation error | yes only for capability miss |
| prepared to routing committed | exact token/input/device revalidation; ownership transition | stale selection/invariant | no program work has begun, but fail closed rather than silently reroute |
| routing committed to resources | program allocation and enforcement setup | allocation/resource error | never |
| resources to in-flight | ABI packing, ordered encoding, submit, retention registration | encode/submit/adapter error | never, including after zero or partial stages encoded |
| in-flight to validation observed | exact terminal success, coherence, record validation | completion/coherence/record/semantic error | never |
| in-flight or validation to published | construct ordered dependency-carrying outputs; promote/copy private results | publication error | never |

Preparation may allocate backend-internal library or pipeline state. It must
not allocate a program output, program temporary, validation record, private
transaction result, or encode program work. This distinction permits real
pipeline preflight without weakening the no-work-before-commit rule.

## Preflight and routing

The preflight implementation performs these checks monotonically:

1. require device-free artifact validation evidence for framing, schemas,
   complete `KernelProgram`, backend metadata/ABI mappings, and declared target
   compatibility;
2. bind every symbolic affinity to the same admitted `LiveDeviceKey` in the
   initial profile and verify each external input is accessible there with its
   required access, visibility, range, encoding, and alignment;
3. construct the bound semantic environment once, evaluate semantic
   constraints, and evaluate checked output/temporary/launch expressions whose
   facts are available at this phase;
4. evaluate applicability guards and deterministic routing priority over only
   complete semantically and numerically equivalent variants;
5. for one candidate at a time, load its backend payload and prepare every
   referenced entry with exact specialization and descriptor values;
6. query prepared-entry facts and validate all launch dimensions, resource
   limits, dynamic scratch, binding slots, zero-work behavior, and residual
   enforcement requirements;
7. prove adapter support for allocation domains, ordered submission, resource
   retention, completion/coherence, required publication mode, and all opaque
   stage completion/effect contracts; and
8. emit a `PreparedSelection` containing the complete stage/pipeline list,
   evaluated ABI/launch values, named-output plan, device key, bound-environment
   digest, input fingerprints, adapter capability revision, and expiration or
   validity scopes of queried facts.

The executor revalidates every fact that can change between preparation and
commit. A mismatch is `StalePreparedSelection`, not a guard miss. The caller
may explicitly start a new top-level preflight while it still owns the original
Tensor-level operation and fallback, but the launch call never reroutes a stale
selection internally.

Candidate rejection is one of:

```text
NotApplicable(guard, actual_value)
UnsupportedCapability(authority, predicate, validity_scope)
Fatal(error)
```

Only the first two advance to another candidate. `Fatal` includes artifact or
ABI inconsistency, provider dishonesty, corrupt executable data, an untyped
pipeline failure, systemic runtime failure, and violated monotonic evidence.
If every candidate has a typed miss, preflight returns `NoApplicableProgram`
with ordered reasons and the outer consumer may use its equivalent fallback.

## `RoutingCommit` as an ownership boundary

```text
PreparedSelection + FallbackAuthority
  --commit(exact current fingerprints)-->
CommittedExecution
```

`FallbackAuthority` represents the still-unexecuted high-level operation. The
commit consumes it. `CommittedExecution` is the only token accepted by program
allocation and encoding APIs and contains no method to recover fallback. This
makes these invalid traces unrepresentable in a conforming implementation:

```text
allocate candidate A -> allocation fails -> execute candidate B
encode stage A0 -> pipeline/encode fails -> execute fallback
submit validator -> semantic miss -> execute ordinary operation
```

An adapter that cannot keep fallback outside the committed launch boundary
does not implement this runtime profile.

## Allocation, ABI binding, and ordered execution

After commit, checked allocation specifications are passed to the adapter. The
initial profile requires distinct program-owned allocations for every output
and cross-stage temporary. Any allowed reuse is already proven by the
`KernelProgram` verifier and carries a `StorageHandoff`; the runtime does not
invent aliasing from allocator behavior.

For each returned allocation, the executor validates actual device/domain,
capacity, alignment, usage, ownership, and resource generation against the
preflighted allocator contract. Mismatch is a postcommit invariant error. A
zero-byte logical value follows its explicit allocation and ABI policy; it is
not replaced by an implicit null binding.

The encoder walks the canonical topological order. For every stage it:

1. resolves each neutral `EntryBindingId` and component role to the payload's
   backend transport location;
2. validates the current resource generation and accessible range;
3. packs metadata and scalars using the declared width, signedness, byte order,
   Boolean representation, offsets, and alignment;
4. binds the exact prepared entry and declared read/write resources;
5. evaluates no unchecked arithmetic and reconstructs no launch value from
   convention; and
6. encodes the declared launch, materialization, validation, or opaque-call
   operation.

The first profile uses one adapter-provided ordered stream. Canonical order
realizes all dependencies but does not erase their typed reasons. A backend
may batch several stages into one submission or split them across submissions
only when it preserves order, enforcement observation points, opaque-call
completion contracts, and resource retention.

An encoding error after the first stage leaves an unsubmitted or partially
encoded backend object whose disposal is adapter-defined. If any work could
have been submitted or observed, resources remain retained until the adapter
proves it is safe to release them. In every case the result is a typed execution
error, never fallback.

## Completion, validation, and publication

### Ordinary asynchronous execution

With proof-elided or no residual validation, the executor may return ordered
named outputs after all stages are successfully submitted and their resources
are registered against completion. Each output carries a dependency understood
by the consumer runtime. The adapter must prevent reads, mutation, reuse, or
cross-stream consumption that violates that dependency. Complete initialization
is a verified stage property; submission does not permit exposing raw bytes to
an unsynchronized host.

### Host validation

Host validation begins `EnforcementCommit` after routing. It observes the
authoritative logical view only after its producer and coherence dependencies.
A semantic miss returns `SemanticValidationError`; it cannot reroute. No result
work begins before success unless the selected plan is explicitly
transactional.

### Device pre-scan and transactional validation

The adapter returns an opaque `SubmissionReceipt` for each committed execution
unit. Before interpreting an error record, it must:

1. identify the exact receipt containing the validation producer and every
   required copy/synchronization;
2. wait for or otherwise establish that receipt's terminal state;
3. inspect its authoritative terminal status after the wait;
4. require successful completion and propagate execution error detail on
   failure;
5. establish the declared host visibility/coherence of the exact error-record
   range;
6. validate record schema, framing, obligation identity, and bounds; and
7. reduce the record to deterministic success or semantic error.

No error-record byte is interpreted before step 4. A later queue receipt is
acceptable as an ordering wait only if the adapter still reports the exact
validator receipt's successful terminal state. FIFO completion alone does not
turn an earlier execution error into success.

For device pre-scan, successful observation precedes result dispatch. For a
transactional plan, result and dependent effects remain in its declared
private closure; failure discards them only after safe device release, while
success performs the declared promotion or publication copy. That action is
`PublicationCommit`.

### Named outputs

The executor publishes exactly the artifact's ordered `ProgramOutput` list.
Each item contains its stable `ProgramOutputKey`, ABI role, logical type/shape,
concrete allocation/view, and dependency. It neither discovers graph leaves
nor uses diagnostic labels as keys. An adapter whose native custom-op hook can
return only one tensor must provide a higher-level packing/splitting facade or
reject multi-output programs during preflight; silently dropping outputs is an
artifact/adapter invariant failure.

## Resource lifetime and ownership hooks

The adapter exposes explicit hooks equivalent to:

```text
allocate(spec, committed_execution) -> OwnedResource
borrow_input(binding) -> RetainedResourceUse
begin_ordered_encoding(device, stream_scope) -> EncodingScope
encode_stage(scope, prepared_entry, bindings, launch) -> EncodedUseSet
submit(scope) -> SubmissionReceipt
retain_until(receipt, resources_and_backend_objects) -> RetentionLease
observe_terminal(receipt) -> Completed | ExecutionError
make_host_visible(receipt, range, coherence_contract) -> ReadableRange
publish_named(outputs, dependencies) -> ConsumerOutputs
discard_private(resources, after_receipts)
```

Equivalent APIs are allowed, but the following ownership outcomes are not:

- input borrows ending at the last host encoder call;
- temporaries or argument buffers released before their final device use;
- libraries/functions/pipelines evicted while submitted commands depend on
  them where the backend does not independently retain them;
- output allocation returned without its dependency;
- validation or private-result storage freed immediately on an in-flight
  failure path; or
- completion callbacks that lose the error/status for the exact receipt.

The adapter may use reference counts, fences, completion callbacks, queue-owned
retention sets, framework storage ownership, or another mechanism. Its
conformance tests must demonstrate the declared semantics, including error and
early-return paths.

## Pipeline and library caching

The minimum successful-cache keys are:

```text
LibraryCacheKey = LiveDeviceKey + BackendPayloadDigest

PipelineCacheKey = LiveDeviceKey
                 + BackendPayloadDigest
                 + BackendEntryKey
                 + specialization values
                 + canonical pipeline descriptor
                 + translation/archive/runtime mode
```

Cache initialization is fallible and concurrency-safe. Successful objects are
published only after complete construction and validation. Device/context loss
invalidates the relevant scope. A cache may retain a negative result only when
the error is explicitly stable for the full key and validity scope; transient
allocation, driver, cancellation, and systemic errors are not capability
facts. Cache eviction respects in-flight retention leases.

Prepared facts are stored with their authority and validity scope. They refine
artifact facts but never mutate portable artifact identity. Routing explain
records identify cache hit/miss, preparation outcome, actual facts, and the
typed reason a candidate was rejected without exposing unsafe buffer contents.

## Typed failure model

```text
RuntimeFailure {
  stage,
  program_and_variant,
  artifact_and_live_device_keys,
  kind,
  cause?,
}

FailureStage = ArtifactBoundary
             | DeviceBinding
             | SemanticBinding
             | CandidatePreflight
             | LibraryPreparation
             | PipelinePreparation
             | LaunchPreflight
             | RoutingCommit
             | Enforcement
             | Allocation
             | AbiBinding
             | Encoding(stage_id)
             | Submission
             | Completion(receipt_id)
             | Coherence
             | ValidationReadback
             | Publication
             | Retention

FailureKind = InvalidArtifact
            | SemanticInputError
            | NotApplicable
            | UnsupportedCapability
            | NoApplicableProgram
            | StalePreparedSelection
            | InvariantViolation
            | AllocationFailure
            | EncodingFailure
            | SubmissionFailure
            | DeviceExecutionFailure
            | CoherenceFailure
            | InvalidErrorRecord
            | SemanticValidationError
            | PublicationFailure
            | AdapterContractViolation
```

`NotApplicable` and `UnsupportedCapability` are routing dispositions only
before commit. The same words reported by a backend after commit are wrapped as
an invariant or execution failure and cannot drive routing. Diagnostics record
whether `RoutingCommit`, `EnforcementCommit`, and `PublicationCommit` were
crossed and the last successfully encoded/submitted stage.

## Minimum consumer-adapter responsibilities

A conforming consumer adapter must provide:

1. runtime-scoped live-device/context identity and symbolic-affinity binding;
2. complete tensor component/view descriptors with resource identity and
   generation, range, access, encoding, alignment, and visibility facts;
3. governed backend payload loading, entry lookup, specialization, pipeline
   preparation, authoritative prepared facts, and device-scoped caches;
4. checked allocation for every declared domain/specification, with explicit
   ownership and no undeclared aliasing;
5. neutral-to-native ABI mapping and exact scalar/metadata packing;
6. one ordered execution stream for the initial profile and an exact receipt
   for submitted work;
7. retention of all resources and backend objects until exact final device use;
8. terminal success/error observation and declared coherence/readback for
   validation, including post-wait status inspection;
9. private-result discard/promotion or publication-copy semantics for
   transactional enforcement;
10. ordered named-output construction with dependency propagation;
11. a Tensor-level fallback owner that is available only before
    `RoutingCommit` and implements the same bound semantic/numerical contract;
    and
12. typed error mapping that never converts corruption, systemic failure,
    semantic invalidity, or a postcommit failure into a plan miss.

Candle can implement these responsibilities through its Metal storage,
`Layout`, allocator, command encoder, and Tensor wrapper. Its current
single-output custom-op return and unchecked post-wait terminal transition are
adapter capability gaps, not reasons to weaken the consumer-neutral contract.

## Valid and invalid traces

### Valid precommit alternate

```text
vector candidate guards pass
  -> vector pipeline reports typed unsupported resource limit
scalar candidate guards pass
  -> all scalar pipelines and launches preflight
  -> RoutingCommit
  -> allocate, encode, submit, publish scalar outputs
```

### Valid asynchronous two-pass reduction

```text
prepare partial + final pipelines
  -> RoutingCommit
  -> allocate output + typed partials temporary
  -> encode partial before final on ordered stream
  -> submit and retain input/partials/output/pipelines
  -> publish named output carrying completion dependency
```

### Invalid fallback after partial encoding

```text
RoutingCommit -> allocate -> encode K0 -> K1 encode error -> ordinary fallback
```

The required result is `EncodingFailure(K1)`. K0-related resources remain safe
until the adapter proves disposal/completion; no other implementation executes.

### Invalid validation readback

```text
submit validator -> inspect pre-wait status -> wait -> read shared flag
```

The required trace inspects the exact receipt's terminal status after the wait,
requires success, establishes coherence, validates the record, and only then
reads its semantic result.

## Bounded executable model

[`spikes/runtime/runtime_execution_contract.rs`](../../../spikes/runtime/runtime_execution_contract.rs)
models the ownership boundary, typed precommit dispositions, exact validation
receipt, ordered stages, named outputs, pipeline keys, and retention through
completion. Its tests cover precommit capability routing, fatal preparation,
stale selection, allocation and partial-encoding failures, no fallback after
semantic validation, exact post-wait status before readback, dependency order,
device-scoped cache identity, and scratch retention.

The model is deliberately dependency-free and synchronous. It does not model a
real GPU API, allocator concurrency, command-buffer cancellation, actual cache
locking, multi-stream schedules, multi-device transfers, or performance.

## Measurement boundary and follow-on work

No new device latency, allocation, pipeline-cache, or command-submission
measurement is claimed by this contract. The executable spike is a transition
and invariant model. The accepted Candle source audit is the only concrete
runtime observation used here; it is not a GPU fault-injection result.

Before an adapter is production-ready, measure library/pipeline cold and warm
latency, concurrent cache initialization, allocation and retention overhead,
multi-dispatch submission cost, validation synchronization/readback cost, and
publication-copy cost on each supported backend/profile. Those measurements
inform costing and cache policy; they do not change the failure or commit
boundaries above.

Multi-device placement, multiple streams, external storage, in-place mutation,
partial writes, and cancellation remain separate contracts. They require
explicit timepoints, transfer/coherence stages, partial-failure ownership, and
publication rules rather than exceptions to this first profile.

## Traceability

This result is adopted by ADR 0051 and the artifact and Candle contracts. The
[runtime spike](../../../spikes/runtime/README.md) checks the state-machine
invariants; multi-device and multi-stream execution remain separate work.
