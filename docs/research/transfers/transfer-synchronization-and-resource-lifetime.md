---
schema: "tiler-doc/v1"
id: "tiler.research.transfers.synchronization-lifetime"
kind: "research"
title: "Transfer synchronization and resource-lifetime contract"
topics: ["transfers", "synchronization", "resource-lifetime", "placement"]
catalog_group: "runtime-integration-placement"
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.artifact-abi", "tiler.contract.candle-integration", "tiler.contract.cpu-backend", "tiler.contract.metal-backend"]
ticket: "transfer-synchronization-and-resource-lifetime-contract"
---

# Transfer synchronization and resource-lifetime contract

**Status:** completed research incorporated into the physical and runtime contracts
**Ticket:** `transfer-synchronization-and-resource-lifetime-contract`

## Outcome

A placement transfer/enforcer makes one authoritative logical value version
accessible at a destination affinity. The family is not reducible to byte
copies: its separately typed variants include direct movement, same-device
logical materialization, peer copy or peer access, two host-staged legs,
managed migration, and import/alias of shared backing. Dtype conversion and
encoding-changing repacking remain separate enforcers.

Every executable transfer names both endpoint placements and allocation
regions, the chosen mechanism, source-producer and destination-consumer
dependencies, completion and coherence obligations, possible hazards, and the
resources retained through exact final device use. Runtime queues, events,
fences, command buffers, and device handles bind this portable contract but do
not enter portable artifact identity.

Dependency completion and successful execution are different evidence. A
dependency token may order a consumer and establish declared visibility. Host
readback, validation, failure reporting, or safe early release that depends on
success additionally requires an exact submission receipt to reach a checked
successful terminal state. Cancellation is a request, not completion: after
`RoutingCommit` it never restores fallback authority or permits resources to be
released while device use may remain.

This contract supplies the transfer/synchronization layer required by ADR 0047
without enabling distributed scheduling. It preserves symbolic multi-device
and multiple-queue extension points, while an executable profile may still
restrict all affinities to one bound device and one ordered stream.

## Evidence and classification

### Primary-source facts

- [IREE Stream](https://iree.dev/reference/mlir-dialects/Stream/) represents
  affinities, resource lifetimes and sizes, asynchronous transfers, and
  timepoints above concrete devices. [IREE HAL](https://iree.dev/reference/mlir-dialects/HAL/)
  lowers to device queue allocation, copy, execution, deallocation, and
  wait/signal fences. This is direct precedent for separating a portable
  transfer stage from runtime execution and completion objects.
- MLIR [`gpu.memcpy`](https://mlir.llvm.org/docs/Dialects/GPU/#gpumemcpy-gpumemcpyop)
  consumes async dependency tokens and may return an async token. Its source and
  destination are memrefs rather than a single untyped byte-pointer pair. This
  supports explicit dependencies and views, but does not by itself define
  Tiler's cross-device failure or lifetime contract.
- CUDA devices have separate default streams, and an event from one device can
  be waited on by a stream from another. Peer memory access and peer copies are
  separately enabled and directional; cross-device memory consistency still
  requires synchronization. See the CUDA Programming Guide on
  [multi-GPU systems](https://docs.nvidia.com/cuda/cuda-programming-guide/03-advanced/multi-gpu-systems.html).
- CUDA exposes distinct synchronous and asynchronous copy, peer-copy,
  prefetch/migration, and memory-advice APIs. The
  [runtime memory API](https://docs.nvidia.com/cuda/cuda-runtime-api/group__CUDART__MEMORY.html)
  also constrains source/destination kinds and stream association. One generic
  `copy(bytes)` contract would erase relevant feasibility and ordering facts.
- Metal command queues, command buffers, and resources are associated with an
  `MTLDevice`; Apple therefore directs applications to select and retain the
  appropriate device objects. See
  [Selecting device objects for compute processing](https://developer.apple.com/documentation/metal/selecting-device-objects-for-compute-processing).
- Metal shared events carry monotonically signaled values and may synchronize
  devices or processes, but the storage resource and the synchronization event
  remain different objects. See
  [Synchronizing events across multiple devices or processes](https://developer.apple.com/documentation/metal/synchronizing-events-across-multiple-devices-or-processes).
  [`MTLCommandBufferDescriptor.retainedReferences`](https://developer.apple.com/documentation/metal/mtlcommandbufferdescriptor/retainedreferences)
  separately controls whether the command buffer holds strong references to
  objects needed for execution; this makes resource retention an adapter
  contract rather than an inference from encoding scope.
- PyTorch's
  [`Tensor.record_stream`](https://docs.pytorch.org/docs/stable/generated/torch.Tensor.record_stream.html)
  records non-creation-stream uses so the caching allocator does not reuse
  storage before queued work completes. Its documented alternative is explicit
  event/stream lifetime management. Host reference lifetime alone is therefore
  insufficient in an asynchronous consumer runtime.
- At Candle commit
  [`31f35b147389700ed2a178ee66a91c3cc25cc80d`](https://github.com/huggingface/candle/blob/31f35b147389700ed2a178ee66a91c3cc25cc80d/candle-metal-kernels/src/metal/commands.rs),
  Metal work is accumulated in command buffers and `ensure_completed` branches
  on command-buffer status before deciding whether to wait, without a second
  error-status check after waiting from the committed or scheduled states. This
  is concrete evidence that a Candle adapter owns command-buffer completion and
  error propagation; it does not make Candle types part of the compiler
  contract.

### Local decisions and proposals

- **Fact — accepted decision:** ADR 0046 separates logical tensor access from allocation-relative physical
  views. A transfer must therefore name the logical view being delivered and
  the backing byte ranges its realization reads or writes.
- **Fact — accepted decision:** ADR 0047 models placement as directed affinity/domain capabilities and
  requires transfer, import, materialization, or recomputation to be explicit
  `KernelProgram` enforcers. Accessibility, visibility, and coherence are
  distinct, and portable plans contain no live runtime handles.
- **Proposal — supporting contract:** The runtime execution contract consumes fallback authority at
  `RoutingCommit`, before program allocation or encoding. It retains resources
  through exact final device use and requires checked terminal success before
  validation readback.

### Inferences

1. Reachability is not synchronization. Same device, same backend, peer access,
   shared virtual addressing, or shared physical backing does not prove that a
   destination sees the authoritative value version.
2. A portable completion token must describe an ordering/visibility obligation,
   not embed an event or fence handle. Runtime binding chooses queue-local,
   cross-queue, cross-device, or host-mediated synchronization.
3. Ordering completion is not sufficient success evidence. A later operation
   may be ordered after an earlier failed operation, so exact terminal status
   remains necessary wherever correctness depends on successful execution.
4. Copy, logical materialization, import/alias, peer copy, managed migration,
   and host staging have different allocation, hazard, failure, and retention
   behavior. They require distinct mechanism records even when a backend lowers
   several of them to one API family.
5. No-copy elimination is a proof obligation. Equal pointers, equal domain
   names, equal device ordinals, or read-only use alone do not prove semantic
   view equivalence, destination accessibility, visibility, ownership, or
   lifetime.
6. A transfer can fail after one leg or one submission has begun. Fallback at
   that point could duplicate effects or race retained resources, so only a
   typed failure and safe deferred cleanup are valid.
7. Allocator retention and execution ordering are related but not identical.
   Preventing byte reuse does not establish producer-to-consumer visibility;
   ordering a consumer does not by itself keep every referenced host/backend
   object alive.

### Measurements

No GPU transfer bandwidth, latency, overlap, cancellation, peer topology, or
fault-injection measurement is claimed here. The local Candle source revision
above was inspected, but no real Metal failure was induced. The executable
spike is a synchronous verifier and state-machine measurement only; its test
count and boundary are recorded below.

## Proposed normalized contract

The spelling is illustrative. Stable newtypes are required in an implementation.

```text
PlacementEnforcer =
    Transfer(TransferStage)
  | ImportAlias(ImportAliasStage)
  | PeerAccess(PeerAccessStage)
  | MaterializeLayout(MaterializeStage)
  | RepackEncoding(RepackStage)
  | ConvertDtype(ConversionStage)
  | Migrate(MigrationStage)
  | Recompute(RecomputeStage)

TransferStage {
  stage_id,
  logical_value_id,
  authoritative_version,
  source: TransferEndpoint,
  destination: TransferEndpoint,
  semantics: PreserveStorageEncoding,
  mechanism: TransferMechanism,
  execution_legs: [ExecutionLeg],
  dependencies: TransferDependencies,
  hazard_contract,
  retention_obligations,
  failure_contract,
}

ExecutionLeg {
  leg_id,
  symbolic_affinity,
  queue_role,
  reads,
  writes,
  waits,
  signals,
  receipt_role,
}

TransferEndpoint {
  symbolic_affinity,
  placement_role,
  allocation_role,
  memory_domain_class,
  logical_view,
  allocation_extent_bytes,
  accessible_allocation_range,
  touched_backing_range,
  storage_encoding,
  access_mode,
  alignment,
}
```

`logical_view` contains shape, logical element start, strides/layout, and the
relation to the semantic value. `accessible_allocation_range` is the range the
bound view is permitted to address. `touched_backing_range` is the exact range
when proved, otherwise a conservative containing range used for retention and
hazard checks. For packed encodings, sharing one containing byte defeats a
byte-disjointness proof unless a later bit-level contract proves compatible
access.

Source and destination allocation roles are stable plan identities, not
addresses. Runtime binding supplies allocation identity, generation, concrete
range, device/context, allocator/pool, and imported ownership evidence.

### Enforcer and mechanism taxonomy

| Enforcer/mechanism | Allocation result | Value/encoding effect | Required special evidence |
| --- | --- | --- | --- |
| `DirectCopy` | distinct destination | same value version and encoding | copy capability, two-sided ordering, nonoverlap or explicit overlap semantics |
| `PeerDirectCopy` | distinct destination owned/imported for destination | same version and encoding | directional pair capability, source read and destination write access, cross-device synchronization |
| `HostStaged` | destination plus staging allocation | same version and encoding over two ordered legs | both leg capabilities, intermediate completion, staging capacity/alignment/coherence |
| `AliasImport` | no new backing; new admitted view/ownership binding | same authoritative backing/version | complete no-copy proof and imported-owner retention |
| `PeerAccess` | no destination copy; source backing is remotely addressable | same authoritative backing/version | directional peer enablement plus complete alias, hazard, synchronization, and retention proofs |
| `ManagedMigration` | backing identity may remain while residence/authority changes | same version and encoding | provider migration/coherence protocol; forbidden concurrent accesses |
| `MaterializeLayout` | new destination | same logical value and dtype; addressing/layout may change | verified logical access relation and kernel/copy schedule |
| `RepackEncoding` | new destination | explicitly changes storage encoding | governed encoding transform and downstream ABI compatibility |
| `ConvertDtype` | new destination | explicitly changes represented values/dtype | ADR 0010 conversion family and numerical contract |

`TransferStage` is the encoding-preserving movement variant of the broader
enforcer family. `AliasImport`, `PeerAccess`, `ManagedMigration`, and
`MaterializeLayout` are not mislabeled as raw byte copies. A backend may
fuse a layout materialization with other computation only if the selected
physical program still discharges the same delivered-placement, dependency,
hazard, and retention obligations.

### Dependencies, synchronization, and completion

```text
TransferDependencies {
  source_ready: [AvailabilityToken],
  mechanism_internal: [DependencyEdge],
  destination_ready: AvailabilityToken,
}

DependencyEdge {
  producer,
  consumer,
  reason: ProducerData | StagingLeg | Visibility | OwnershipHandoff,
  required_scope,
}

RuntimeCompletionBinding {
  token,
  execution_scope,
  wait_objects,
  signal_objects,
  submission_receipts,
  coherence_actions,
}
```

The verifier requires all of the following:

1. the transfer cannot read the source before every source producer token;
2. each internal leg waits for the leg that produces its input;
3. no destination consumer begins before `destination_ready`;
4. the bound synchronization scope covers the producer and consumer affinities,
   queues, and touched ranges;
5. the completion path establishes the destination visibility/coherence
   promised by the delivered placement; and
6. every exact submission receipt remains available for error propagation and
   any host success observation.

An ordered stream may lower some edges without an explicit event, but it does
not erase the typed dependency from the artifact or explain record. Cross-queue
or cross-device lowering must bind an event/fence protocol admitted by both
execution scopes, or use an explicit host-mediated wait-and-submit stage.
`queue_role` is stable, such as source-copy, destination-copy, or compute; the
runtime binds it to a live queue only after device/context preflight.

`destination_ready` means a conforming dependent operation may consume the
destination while carrying asynchronous error propagation. It does not mean a
host has observed terminal success. Host reads, semantic validation, resource
reclamation based on success, and synchronous publication require:

```text
SuccessfulCompletionEvidence {
  exact_receipts,
  checked_terminal_success,
  destination_visibility,
  range,
}
```

A fence/event reaching its value cannot be silently reclassified as exact
success unless the backend contract proves that relationship and preserves
the associated error.

### Hazards, aliasing, and no-copy elimination

The transfer verifier builds an access ledger over concrete allocation roles,
generations, and backing ranges after runtime binding:

```text
AccessUse {
  allocation_role,
  generation,
  backing_range,
  mode: Read | Write | Atomic,
  begins_after,
  completes_before,
}
```

Overlapping read/read uses are compatible. Any overlap with a write or atomic
requires a dependency/hazard protocol strong enough for both execution scopes.
Unknown overlap fails closed when either use writes. A copy whose source and
destination overlap is rejected unless its mechanism explicitly specifies and
the backend proves overlap-safe semantics; ordinary device-copy APIs are not
assumed to behave like `memmove`.

An `AliasImport` or eliminated copy requires one proof record establishing:

1. the destination view refines the semantic coordinate relation and remains
   inside the imported accessible allocation range;
2. source and destination refer to the same bound backing allocation and
   generation, not merely equal addresses or domain labels;
3. logical value version and storage encoding are identical;
4. the destination affinity has the required read/write mode for that range;
5. producer visibility/coherence is forwarded or explicitly enforced before
   every destination consumer;
6. no incompatible access overlaps the alias lifetime;
7. ownership/import rules keep the backing, view metadata, and external owner
   alive through all final uses; and
8. consumer-facing aliasing and mutation rules admit the returned view.

Failure of any item selects a real materialization before commit or rejects the
candidate. It cannot turn into a best-effort alias after commit.

### Retention and ownership

```text
RetentionObligation {
  resource_role,
  uses: [StageOrReceiptId],
  release_after: CompletionCondition,
  on_failure: RetainUntilNoPossibleDeviceUse,
}

RetainedResourceRole =
    SourceAllocation | DestinationAllocation | StagingAllocation
  | SourceView | DestinationView | ArgumentStorage
  | CommandObject | QueueSubmissionObject
  | EventOrFence | ImportedBackingOwner
  | Library | Function | Pipeline
```

Every resource has an exact last-use condition, not merely a lexical owner.
For a two-leg staged transfer, the source may become releasable after the first
leg's exact safe-use condition, but staging remains through the destination
leg, and the destination becomes owned by the delivered placement rather than
being freed at transfer completion. Synchronization objects remain live until
all encoded waits/signals and status/error observations that reference them are
safe.

The adapter may implement obligations with command-buffer retained references,
completion handlers, queue retention sets, allocator stream recording,
reference counts, or explicit fences. Its conformance surface must demonstrate
equivalent behavior on success, error, cancellation, early return, and partial
submission. Destruction of a host wrapper is never itself final-device-use
evidence.

### Commit, cancellation, and failures

Preflight proves the complete mechanism before `RoutingCommit`: endpoint
bindings, all allocation/import capabilities, queue/event scope, copy or
materialization support, staging specifications, range arithmetic, hazards,
coherence, retention hooks, and publication mode. A typed applicability or
capability miss may select another complete enforcer only before commit.

```text
PreparedTransfer + FallbackAuthority
  -> RoutingCommit
CommittedTransfer
  -> acquire/import all program resources
  -> encode zero or more legs
  -> submit zero or more execution units
  -> observe dependency completion or exact terminal failure
  -> deliver placement or fail
```

Failures are typed by the exact stage and leg:

```text
TransferFailureStage =
    EndpointBinding | MechanismPreflight | HazardPreflight
  | RoutingCommit | SourceAllocation | DestinationAllocation
  | StagingAllocation | Import | Encoding(leg_id)
  | Submission(leg_id) | Completion(receipt_id)
  | Coherence(leg_id) | Publication | Retention

TransferFailureKind =
    NotApplicable | UnsupportedCapability
  | StalePreparedTransfer | InvalidRange | AliasProofFailure
  | AllocationFailure | ImportFailure | EncodingFailure
  | SubmissionFailure | DeviceExecutionFailure
  | CoherenceFailure | CancellationRequested
  | PublicationFailure | AdapterContractViolation
```

Only `NotApplicable` and `UnsupportedCapability` before commit preserve
fallback. After commit, failure is terminal for the high-level operation.
Cleanup may proceed asynchronously but cannot authorize another implementation.

Cancellation before commit abandons a candidate without program work.
Cancellation after commit is best effort: it records intent, stops encoding or
submission only where the backend proves that safe, and otherwise waits for an
exact terminal or provider-defined safe-release condition. An event not yet
signaled, a dropped future, a destroyed wrapper, or a backend cancellation
return does not by itself prove that no command can touch retained resources.

### Artifact identity, runtime binding, and explain

Portable artifact identity includes:

- stable enforcer/stage and logical value-version IDs;
- symbolic source/destination affinities and relational constraints;
- endpoint placement/allocation roles, view descriptors, byte-range
  expressions, storage encoding, access, and alignment;
- governed mechanism key and schema revision;
- dependency graph, synchronization scopes, hazard policy, and retention roles;
- staging specifications, completion semantics, and typed failure policy; and
- all static predicates that affect legality or lowering.

It excludes live devices, ordinals, contexts, queues, event/fence objects,
allocations, addresses, command buffers, submission receipts, and current peer
topology. Runtime routing/cache fingerprints bind those live facts and their
capability revisions to the immutable artifact.

Explain output reports the requested delivered placement, every considered
mechanism, accepted/rejected capability and alias proofs, source/internal/
destination dependencies, conservative versus exact hazard ranges, retention
last uses, commit state, bound mechanism provider, and typed failure stage. It
uses stable diagnostic IDs; live object descriptions may be redacted or
runtime-scoped and never become artifact keys.

## Worked profiles

### CPU to accelerator

A host producer finishes writes to an input view. A transfer stage waits on
that producer evidence, reads the exact host backing range, writes a distinct
accelerator allocation with the same encoding, and signals
`destination_ready`. Pinned direct DMA, shared backing import, and pageable
host staging are different candidate mechanisms. The source owner, both views,
destination, command object, and synchronization objects remain retained
through their final device uses.

### Accelerator to CPU

The stage waits on the accelerator producer, copies or synchronizes the
authoritative range into a host-accessible destination, and produces an async
dependency. The host must not inspect bytes until the exact transfer receipt is
successfully terminal and the declared host-visibility action has completed.
Waiting without checking terminal status is insufficient.

### Same-device materialization

A consumer requires a contiguous view while the source is strided. A
`MaterializeLayout` stage reads source elements through the verified logical
access relation and writes a distinct contiguous allocation on the same
affinity. Same device eliminates neither the producer dependency nor hazards.
If source and destination backing overlap, the plan needs an explicit
overlap-safe schedule proof; otherwise it is rejected.

### Peer/direct transfer

`gpu0` produces a value and `gpu1` consumes it. Runtime preflight proves the
directed pair capability and binds a peer-copy queue plus a cross-device event
or host-mediated ordering protocol. The peer copy waits on `gpu0`'s producer
and `gpu1` waits on the copy completion. Peer accessibility alone satisfies
neither dependency.

A distinct `PeerAccess` candidate may let `gpu1` read `gpu0` backing directly.
It still needs directional access enablement, the full no-copy proof, explicit
producer-to-consumer synchronization, hazard exclusion, and source-owner
retention. It is not interchangeable with `PeerDirectCopy`, which produces a
separate destination allocation.

### Shared backing/import

CPU and accelerator bindings refer to one imported backing allocation. No copy
stage is emitted only after the full alias proof: same generation/range,
semantic view equivalence, destination access, visibility, nonconflicting
hazards, ownership, and consumer alias admission. The delivered dependency is
the forwarded producer/coherence dependency, not a fabricated completed token.

### Host-staged peer transfer

When direct peer copy is unavailable, leg 1 copies source device to an admitted
host staging allocation; leg 2 waits on leg 1 and copies staging to the
destination device. The destination consumer waits on leg 2. Failure after leg
1 does not permit fallback, and source/staging/destination resources follow
their individual exact safe-release conditions.

### Managed migration

The backing allocation may retain identity while the provider changes its
resident/authoritative location. The stage declares the migration and
coherence protocol and forbids unordered conflicting accesses. It is not
modeled as allocation-to-allocation byte copy, and a prefetch hint is not
treated as proof that migration or visibility completed.

## Verifier invariants

The portable verifier proves structural items 1–8; runtime preflight re-proves
bound items 9–15 against live capabilities:

1. all endpoint range arithmetic is checked and every touched range lies in
   its view's admitted allocation range;
2. transfer source and destination name the same logical value version and
   storage encoding;
3. encoding or dtype changes use their separately typed enforcers;
4. every source producer reaches the transfer, every internal leg is ordered,
   and every destination consumer depends on destination readiness;
5. the dependency graph is acyclic and every token has one governed meaning;
6. required resource roles exist and each has a final-use release condition;
7. copy overlap is rejected unless explicit overlap-safe semantics are proved;
8. alias elimination carries every required proof component;
9. concrete endpoints match symbolic affinities, domains, allocations,
   generations, ranges, encodings, access modes, and alignment;
10. the chosen mechanism is admitted for the directed endpoint pair and exact
    ranges, including every staged leg;
11. synchronization objects cover all participating queue/device scopes and
    establish the promised visibility/coherence;
12. overlapping access uses are read/read or ordered by a sufficient hazard
    protocol;
13. staging allocation capacity, alignment, accessibility, and coherence hold;
14. the adapter can retain every resource and exact receipt across all success,
    failure, cancellation, and partial-submission paths; and
15. no allocation/import/encoding begins until commit has consumed fallback
    authority.

Failing an invariant produces a typed, explainable rejection or postcommit
failure. It never becomes infinite cost or an implicit copy.

## Bounded executable spike

[`spikes/transfers/transfer_contract.rs`](../../../spikes/transfers/transfer_contract.rs)
implements checked view/backing ranges, preserved encoding/version,
two-sided dependencies, staged-leg ordering, role-based retention, alias-proof
requirements, overlapping-access hazards, copy-overlap rejection, and the
commit/cancellation/release state machine.

Seventeen tests pass. The positive examples cover CPU-to-accelerator,
accelerator-to-CPU, same-device materialization, peer direct copy, peer access,
shared-backing alias, managed migration, and host-staged transfer. Negative
tests cover hidden conversion, invalid ranges,
missing source or consumer dependencies, incomplete alias proof, missing
staging retention/order, unordered write hazards, overlapping copy, fallback
after commit, cancellation before terminal release, and failure after staged
work has begun.

The spike is dependency-free and synchronous. It does not bind real queues,
events, allocators, devices, pageable/pinned memory, managed residency, or
provider errors. Passing tests demonstrate consistency of the proposed
invariants over the modeled traces, not backend conformance or performance.

## Scope boundary and follow-on evidence

This contract intentionally does not choose devices, schedule distributed
graphs, define sharding/collectives, or optimize communication topology. A
future multi-device physical profile may instantiate several symbolic
affinities and queues using these endpoint, dependency, completion, hazard, and
retention records. Distributed failure recovery and collective semantics remain
separate layers.

Before enabling a backend transfer profile, add adapter conformance tests for
direct and staged paths, peer directionality, exact terminal errors, host
visibility, allocator reuse, alias import ownership, cancellation, and failures
at every leg. Measure latency, bandwidth, overlap, staging thresholds, and
retention overhead per exact device/runtime profile. Those measurements inform
feasibility predicates and cost models; they do not weaken the correctness or
failure boundaries above.

## Traceability

This contract refines the placement and runtime boundaries and is exercised by
the [transfer spike](../../../spikes/transfers/README.md). Backend-specific
multi-device measurements and calibrated transfer costs remain future evidence.
