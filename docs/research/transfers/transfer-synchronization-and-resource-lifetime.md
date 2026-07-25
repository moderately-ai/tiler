---
schema: "tiler-doc/v1"
id: "tiler.research.transfers.synchronization-lifetime"
kind: "research"
title: "Transfer synchronization and resource-lifetime contract"
topics: ["transfers", "synchronization", "resource-lifetime", "placement"]
catalog_group: "runtime-integration-placement"
research_status: "complete"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.artifact-abi", "tiler.contract.candle-integration", "tiler.contract.cpu-backend", "tiler.contract.metal-backend"]
ticket: "transfer-synchronization-and-resource-lifetime-contract"
---

# Transfer synchronization and resource-lifetime contract

**Status:** completed research; the normalized contract below remains a proposal
that no accepted ADR and no normative contract has yet incorporated
**Ticket:** `transfer-synchronization-and-resource-lifetime-contract`

## Outcome

A placement transfer/enforcer makes one authoritative logical value version
accessible at a destination affinity. The family is not reducible to byte
copies: its separately typed variants include direct movement, same-device
logical materialization, peer copy or peer access, two host-staged legs,
managed migration, and import/alias of shared backing. Encoding-changing
repacking is a separate enforcer of that family. Dtype conversion is a
separately typed stage and is deliberately **not** a member of it: it changes
which values the boundary carries, so no planner may select it to satisfy a
placement obligation. "Enforcer, excluded neighbour, and the asymmetry between
repacking and conversion" below derives that split from the accepted definition
rather than from this memo's own preference.

Recomputation is a member and is the one variant that does not move an existing version — it re-derives one, so it is the only enforcer whose value preservation its mechanism cannot discharge. "The recompute obligation" below states what it must carry instead: bit-identity with the version it stands in for, discharged structurally by four conditions that are decidable from identities and stated facts rather than from a value model, and a typed pre-commit rejection when any of them fails.

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

This proposal supplies the transfer/synchronization layer required by ADR 0047
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
  | Migrate(MigrationStage)
  | Recompute(RecomputeStage)

// Not a member. `ConvertDtype(ConversionStage)` is the realization of a cast
// the semantic graph already contains, named here so a transfer can be checked
// against it and never confused with it.
ExcludedNeighbour =
    ConvertDtype(ConversionStage)

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

// The one enforcer with no `source: TransferEndpoint`. It reads the recomputed
// region's operands and writes a destination; `reference_placement` describes
// the version it must agree with, which a plan that kept the producer's result
// would have delivered and which this plan need not materialize anywhere.
RecomputeStage {
  stage_id,
  logical_value_id,
  authoritative_version,
  destination: TransferEndpoint,
  reference_placement: TransferEndpoint,
  operands: [TransferEndpoint],
  value_preservation: RecomputeValuePreservation,
  dependencies: TransferDependencies,
  hazard_contract,
  retention_obligations,
  failure_contract,
}

// All four are decidable from identities and stated facts. None requires a
// value model, and none may be replaced by an estimate or a cost.
RecomputeValuePreservation {
  implementation_identity: RegionImplementationId,     // equals the reference's
  reference_implementation_identity: RegionImplementationId,
  delivered_realization: DeliveredNumericalRealization, // equals the reference's
  reference_delivered_realization: DeliveredNumericalRealization,
  determinism_level: PlanDeterministic,
  operand_closure: [OperandDelivery],                   // one per operand
}

OperandDelivery {
  operand_index,
  logical_value_id,
  authoritative_version,
  delivered_placement,
  discharging_enforcer,
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
| `RepackEncoding` | new destination | explicitly changes storage encoding; the represented values are unchanged | governed encoding transform and downstream ABI compatibility |
| `Recompute` | new destination; no source version is read, and none need exist | same logical value version and same bits, re-derived rather than moved; encoding and addressing are the recomputing implementation's declared output contract | complete `RecomputeValuePreservation` record: equal `RegionImplementation` identity, equal delivered numerical realization, plan determinism at the executing scope, and an operand-closure entry for every operand |

Every row above is value-preserving, and the last one is the only row whose
mechanism does not discharge that by itself — its fourth column is a required
proof record rather than a capability or a protocol. "The recompute obligation"
below derives it.

The excluded neighbour, listed separately because every row above is
value-preserving and this one is not:

| Excluded neighbour | Allocation result | Value/encoding effect | Required special evidence |
| --- | --- | --- | --- |
| `ConvertDtype` | new destination | explicitly changes represented values/dtype | ADR 0010 conversion family and numerical contract |

`TransferStage` is the encoding-preserving movement variant of the broader
enforcer family. `AliasImport`, `PeerAccess`, `ManagedMigration`,
`MaterializeLayout`, and `Recompute` are not mislabeled as raw byte copies. A backend may
fuse a layout materialization with other computation only if the selected
physical program still discharges the same delivered-placement, dependency,
hazard, and retention obligations.

### Enforcer, excluded neighbour, and the asymmetry between repacking and conversion

This section exists because an earlier version of this memo applied the word
"enforcer" to both `RepackEncoding` and `ConvertDtype`. The intent behind that
was never in dispute — verifier invariant 3 exists precisely to stop a transfer
folding a conversion into a copy, and ADR 0047 already requires that a transfer
not silently convert encoding. Only the label was wrong, and correcting it turns
out to split the two rather than to rename them together.

**Fact — the accepted definition of an enforcer.** [The optimizer
contract](../../compiler/optimizer.md#enforcers) states that "an enforcer
supplies a missing required property at a cost" and "may change only how a
boundary value is stored, addressed, placed, or delivered, never which values
that boundary carries". It derives that from ADR 0001: several physical
schedules must implement one semantic group identically, so a schedule-level
step that altered a value would make one semantic program mean different things
under different plans. It then draws the consequence directly — "a dtype cast is
therefore not an enforcer" — and keeps resolved value dtype off the
boundary-property list by construction rather than by omission.

**Fact — `RepackEncoding` meets that definition and the property it supplies is
now named.** The same contract admits storage encoding to the boundary-property
list, states that "its enforcer is repacking", and cites this memo's own
separation of `MaterializeLayout` from `RepackEncoding` as the reason the
enforcer was accepted before the property was named. Encoding passes the
admission test that dtype fails: a producer can realize one semantic value
either packed or unpacked and the choice is unobservable in the value.

**Inference — so the two do not resolve the same way, and this memo keeps both
words.** `RepackEncoding` stays a `PlacementEnforcer` variant; nothing about it
needed correcting. `ConvertDtype` leaves the family. Its membership was not
merely a mislabel but a structural claim: a sum type a planner selects from,
containing a conversion, says a planner may introduce a conversion to satisfy a
boundary — which ADR 0010 forbids, and which the optimizer contract restates as
"a conversion the graph does not contain may not be introduced by a schedule at
all".

**Inference — no second umbrella term is needed, and adding one would be
harmful.** The alternative was a second family, something like
`ValueProducingStage`, with `ConvertDtype` as its member. It is rejected:
`ConvertDtype` already has an owner. [Numerical
semantics](../../numerical-semantics.md#casts) makes a cast a semantic operation
carrying a resolved typed conversion contract, and its realization is ordinary
lowering of the operation the graph already contains. A second family here would
be a second authority for something ADR 0010 owns, and naming it beside the
enforcer family would reintroduce exactly the reading the split removes — that
the placement layer may pick one. It is listed as an excluded neighbour instead,
because the verifier still has to name it: invariant 3 is what stops a transfer
absorbing a conversion, and it cannot check against a stage the taxonomy does
not name.

**Fact — the other seven rows were checked against the same definition, and one
is not settled by it.** `DirectCopy`, `PeerDirectCopy`, `HostStaged`,
`AliasImport`, `PeerAccess`, and `ManagedMigration` each declare the same value
version, and `MaterializeLayout` declares "same logical value and dtype". All
seven are value-preserving in the required sense. `Recompute` is the one
`PlacementEnforcer` variant this taxonomy table never described, and it is the
one the definition does not settle: it does not move an authoritative version,
it re-derives one, so it is value-preserving only if the recomputation is proved
to produce the same values under the effective numerical contract. ADR 0047
accepted recomputation as an enforcer and that acceptance is preserved here; what
this section originally recorded is that its value-preservation was a proof
obligation this memo had not stated, where every other row's is discharged by the
mechanism itself.

**Fact — that obligation is now stated, and the row exists.** [`qualify-recompute-value-preservation-in-the-transfer-taxonomy`](../../../tickets/qualify-recompute-value-preservation-in-the-transfer-taxonomy.md) closed it. `Recompute` has a taxonomy row above, a `RecomputeStage` shape, a structural item in the portable verifier, a typed pre-commit failure kind, a worked profile, and spike coverage. "The recompute obligation" immediately below is the derivation; nothing about the variant's acceptance changed, and the eight-of-nine reading in this section is preserved as the finding that produced it.

**Fact — the reach of this reconciliation.** This memo is a proposal that no
accepted ADR and no normative contract has incorporated, so this is terminology
brought into line ahead of incorporation and not a correction to an accepted
contract. Nothing outside this file spells `ConvertDtype` except the optimizer
contract's citation of this taxonomy, which says it "keeps both distinct from
`ConvertDtype`" and stays exactly true under the split. The exact check:
`grep -rn ConvertDtype docs spikes crates` returns this file and
`docs/compiler/optimizer.md` and nothing else.

### The recompute obligation

**Fact — `Recompute` is the one enforcer with no source endpoint, and that, not only the missing row, is why the gap survived a table built to make this comparison.** Every other variant names a `source: TransferEndpoint` holding an authoritative version that already exists, and the portable verifier's structural items are written over that pair: one compares the source's and destination's value version and storage encoding, and the next stops a repack or a conversion being absorbed into the movement between them. A recomputation has no source endpoint. It reads the recomputed region's operands and writes a destination, and the version it must agree with need not be materialized anywhere in the plan. Four of the eight structural items still apply to it unchanged — range arithmetic, two-sided dependencies, acyclicity, and retention roles — and the four that carry the value-preservation weight are exactly the four with no subject: the version-and-encoding comparison, the absorbed-repack-or-conversion check, the copy-overlap rejection, and the alias proof. None was false about a recomputation, so none failed; each had nothing to speak about. The row alone would therefore not have closed the gap, and the verifier gains an item of its own below.

**Fact — the accepted decisions admit recomputation without saying what makes one legal.** [ADR 0047](../../decisions/0047-model-placement-as-physical-properties.md) lists "legal recomputation" among the enforcers the optimizer may use. The [device-placement and memory-domain research](../placement/device-placement-and-memory-domains.md) it takes its evidence from states the condition as "recomputation at the destination when the producer is pure and the numerical contract permits the same result". Purity is a checkable property of the producer. The second clause names the obligation and does not say what discharges it; the rest of this section says.

#### The relation is bit-identity, and a requester-declared weaker relation is not available

**Proposal.** The delivered value must be bit-identical, element for element over the delivered logical view, to the version the enforcer stands in for — the version a plan that retained the producer's result and moved it would have delivered. Storage encoding and addressing may differ exactly as they may under any other enforcer; a boundary that requires a different one composes a `RepackEncoding` or `MaterializeLayout` after the recomputation rather than absorbing it, which is the same rule the two movement rows already carry.

**Inference — a weaker relation the boundary's requester declared acceptable is eliminated, and by derivation rather than by preference.** Three checks remove it and each is independently sufficient.

First, the comparison is between two plans of one program, not between a program and a reference. Enforcer legality rests on the [optimizer contract](../../compiler/optimizer.md#enforcers)'s reading of ADR 0001, that "several physical schedules implement one semantic group identically, so a schedule-level step that altered a value would make one semantic program mean different things under different plans". A tolerance attached to the boundary would make two plans deliver different values by construction. That is the conclusion the argument exists to forbid, not a parameter of it.

Second, a per-boundary tolerance is a second numerical authority scoped below the program. [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) states that "no authority may narrow, weaken, or substitute the caller's stated numerical contract in order to make a target feasible", and its stated consequence is that a caller's resolved contract "does not become a per-region choice, and two regions of one program never honour different contracts". A tolerance introduced at a placement boundary is exactly a per-region choice, arriving through the placement layer instead of the request.

Third, and decisively, a tolerance would price meaning. A planner reaches `Recompute` by comparing it against a transfer on cost, so if the recomputation may deliver a value merely within a declared tolerance, the cheaper plan is cheaper partly because it delivers a different value. That is the mistake [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) names as "treating a flush-tolerant plan as a cheaper alternative to a preserving one", and it is the hard-feasibility-versus-cost separation being crossed rather than traded.

The future region/output accuracy layer does not rescue it. [Numerical semantics](../../numerical-semantics.md) requires such a goal to identify an observable output and states that "No region goal silently overrides a local operation contract." A recompute boundary is an internal placement obligation and not an observable output, so even that additive layer would not reach it.

#### The discharge is structural, one identity layer above the obvious candidate

**Fact — `ScheduledRegion` identity excludes the two facts this turns on.** The [IR contract](../../ir.md) layers identity: `IndexRegion` "commits to canonical iteration/scalar/access content and to its declared numerical realization, complete over every dimension"; `ScheduledRegion` "commits to its `IndexRegion` plus normalized schedule"; and `RegionImplementation` commits to its body, boundary contracts, applicability predicates, target requirements, and resource requirements, "including the selected numerical realization/provider". The same contract adds that "The selected realization/provider and every output-affecting helper and flag remain physical-plan and artifact identity", puts "Output-affecting backend/compiler configuration and selected target identity" in artifact identity, and excludes targets from `IndexRegion` identity outright.

**Inference — so "the same `ScheduledRegion` identity with the same numerical realization" is not sufficient, and the case that breaks it is this enforcer's ordinary case rather than an exotic one.** A recomputation exists to run where the retained version is not, so it is selected against the destination's requirements, and a different implementation over the same scheduled region is the expected outcome rather than bad luck. Two implementations refining one declared realization need not agree bit for bit: the [optimizer contract](../../compiler/optimizer.md#physical-implementation) admits that "A stronger implementation may satisfy a weaker requested result set", and a stated accuracy envelope under [ADR 0042](../../decisions/0042-use-typed-transcendental-accuracy-contracts.md) is satisfied by more than one bit pattern. Committing only to the scheduled region admits exactly that pair, silently.

**Proposal — the obligation is discharged structurally by four conditions, and none needs a value model.**

1. **Equal implementation identity.** The recomputing stage's `RegionImplementation` identity equals the identity of the implementation the reference placement was, or would have been, produced by. This subsumes the scheduled region, hence the canonical iteration/scalar/access content and the declared numerical realization complete over every dimension, and adds the selected realization/provider that `ScheduledRegion` identity excludes.
2. **Equal delivered numerical realization.** The realization delivered at the recomputation's execution site equals the one delivered at the reference's, as a stated record rather than an inference from a target name, a flag set, or a module-level declaration. That record reaches the compiler as well as the contract: ADR 0076 item 3 requires a declared behaviour's validity scope to identify "which compiler build and which execution environment" it was measured on, and item 4 makes the artifact record inherit that requirement, so comparing two delivered realizations compares the output-affecting configuration and the compiler build behind each. Within one artifact, one target, and one compiler this is equality by construction and ADR 0076's honesty rule keeps declared and delivered together. Across two of any of those it is a comparison of two records, and the record it needs does not exist yet — see the deferred question below.
3. **Plan determinism at the executing scope.** The selected plan must hold the [plan-deterministic](../../numerical-semantics.md#reductions) guarantee, that "identical input bits and runtime bindings, executed through the same artifact digest and selected plan variant in the same declared target environment, produce identical output bits". That contract already places the burden on the plan rather than on an assumption: "The physical plan must reject timing-dependent atomics or other execution choices that can violate this promise." A recomputation of an implementation that cannot carry that guarantee is not value-preserving even against itself, so this condition is what excludes recomputing a timing-dependent atomic reduction and expecting the bits to match.
4. **Operand closure.** Every operand the recomputation reads is delivered at its affinity as the same authoritative value version, each under its own discharged enforcer obligation. The condition is recursive and terminates, because the operand graph is finite and a program input is a delivered placement with no producer. Without it the first three conditions prove only that the same function was applied, which says nothing about the result when the arguments differ.

**Inference — condition 1 must be identity and not conformance-equivalence, on the fail-closed test.** The weaker rule — admit any implementation whose numerical guarantee fixes the same evaluation exactly, which under a strict contract several schedules do — is the attractive one, because it would let a recomputation choose a cheaper schedule. It is rejected because it cannot fail closed as contracts widen. Its correctness depends on the effective permissions leaving no freedom: [ADR 0011](../../decisions/0011-per-operation-numerical-permissions.md) makes each permission independent, so a contract that moves a reassociation permission from forbidden to permitted converts a discharging recomputation into a non-discharging one with no build error, no gate signal, and no change at the recompute site. That is the hazard [the IR contract](../../ir.md) states as a rule for exactly this shape of reasoning: "a key and a predicate are both projections, and a projection cannot fail closed when its source grows." Identity is not a projection, so it survives the widening that the equivalence test does not.

**Measurement — owned by the [Apple GPU numerical behaviour record](../apple-targets/numerical-behaviour.md), and cited here only for what it establishes about condition 2.** Two of its findings bear directly. Its finding 8 measures the offline and runtime Metal compilers on one host to be separately built — `32023.883` from the Xcode toolchain asset and `32023.921` shipped with the OS — and its finding 9 measures all forty macOS runtime cases agreeing bit for bit with their offline counterparts while stating the boundary of that agreement: it "does not make the offline build's declared realization *transferable* to a runtime-compiled kernel; it makes the two happen to coincide here". Its findings 21 and 22 measure `f16` arithmetic preserving subnormals that `f32` arithmetic flushes, from modules that declare the identical `air.compile_options` set including `air.compile.denorms_disable`, and conclude that "An artifact-side reader that inferred the delivered realization from the module's `air.compile.*` names would be wrong about `f16` on this row". That pair differs in dtype rather than being one operation recompiled, so it is not itself an instance of condition 2 failing; what it establishes is the premise condition 2 rests on — that a compile-side declaration is not a report of delivered arithmetic, so equality of declarations is not equality of deliveries, and condition 2 has to compare delivered records.

**Inference — a numerical proof of the kind `FusionNumericalProof` carries is not the alternative, and no available evidence class supplies one.** The alternative to conditions 1 and 2 would be admitting a recomputation by a *different* implementation and proving the values equal anyway. That is a universal claim over an unbounded floating-point input domain. `FusionNumericalProof` in `crates/tiler-compiler/src/fusion.rs` is not a precedent for it: it binds a rederived candidate, the request subject, and the forbidden-transform permissions, and its four proof components assert that the fusion removed no observable materialization boundary and consumed no forbidden transform. It is a structural proof about one transformation, not a value model comparing two programs, and it has no machinery that would extend to one. The remaining candidate evidence is empirical agreement over sampled inputs, which is a strictly weaker class than the guarantee needed. The honest outcome is therefore that a cross-implementation recomputation is inadmissible rather than provable, which is what makes conditions 1 and 2 requirements rather than a conservative shortcut.

**Fact — retention and hazards follow from having reads without a source version.** A recomputation retains its operand allocations and views through its own final device use, plus the destination, the command object, and the synchronization objects. It retains no staging allocation and no imported backing owner. For this variant the `SourceAllocation` and `SourceView` roles denote the recomputed region's operands rather than a moved version's source. Its destination write must not overlap an operand it still reads, which is the same class of rejection the taxonomy already applies to an overlapping copy and is not weakened by the write being produced rather than copied.

#### A recomputation that cannot discharge the obligation is a typed rejection, never a cost

**Proposal.** Failure of any of the four conditions makes the `Recompute` candidate inapplicable at preflight, with the typed failure `MechanismPreflight`/`RecomputeValuePreservationUnproved` naming which condition failed and against which reference implementation, delivered realization, determinism level, or operand. Because the failure is proved before commit and before any program work, it preserves fallback authority in the same sense `NotApplicable` and `UnsupportedCapability` do: the planner selects another complete enforcer — a transfer, an alias, a materialization — or the placement requirement is rejected with an explainable reason. It is never an infinite or arbitrary cost, never a lower-ranked alternative, and never a delivery of whatever the recomputation happened to produce.

**Inference — this direction is not open, and it is worth saying which rule closes it rather than only that it is closed.** Two do, independently. Hard feasibility is separate from estimated cost, so an infeasible plan is rejected with a reason and never hidden behind a cost; and a plan whose delivered values differ from the boundary's is not a slower or cheaper plan but a wrong one, so it fails the correctness rule that no incorrect tensor may be returned to preserve a fast path. The first alone would still permit ranking; the second is what makes the rejection unconditional.

**Fact — no architectural question survives this section.** Each of the four alternatives tested above was eliminated by a check rather than by a preference: the requester-declared tolerance by three independent contract rules, the scheduled-region discharge by the identity layering, the conformance-equivalence relaxation of condition 1 by the fail-closed test, and the cross-implementation numerical proof by the absence of any evidence class that could carry it. One candidate survives in each case, so there is nothing here to escalate.

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
  | RecomputeValuePreservationUnproved
  | StalePreparedTransfer | InvalidRange | AliasProofFailure
  | AllocationFailure | ImportFailure | EncodingFailure
  | SubmissionFailure | DeviceExecutionFailure
  | CoherenceFailure | CancellationRequested
  | PublicationFailure | AdapterContractViolation
```

Only `NotApplicable`, `UnsupportedCapability`, and
`RecomputeValuePreservationUnproved` before commit preserve fallback. After
commit, failure is terminal for the high-level operation. Cleanup may proceed
asynchronously but cannot authorize another implementation.

`RecomputeValuePreservationUnproved` is reachable only at `MechanismPreflight`,
because every condition it reports is a property of stated identities and stated
target facts rather than of an executed stage. It names which of the four
conditions failed. It is a rejection of one candidate mechanism and never a cost,
a ranking, or a delivery of a differently realized value; the same recomputation
never becomes admissible later in the same plan.

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

### Recomputation instead of a transfer

A pointwise producer `p` runs at affinity `A` and a consumer at affinity `B`
needs its result. One candidate transfers `p`'s retained result from `A` to `B`.
A second recomputes `p` at `B`, which is cheaper whenever `p`'s operands are
already delivered at `B` and its arithmetic costs less than the movement.

The second candidate is admitted only with a complete
`RecomputeValuePreservation` record. Its implementation identity must equal the
one that produced the retained result at `A`, so a cheaper schedule for `B` is a
different candidate and not this one; the delivered numerical realizations at the
two sites must be equal as stated records; the selected plan must carry the
plan-deterministic guarantee at `B`'s execution scope; and every operand of `p`
must be an operand-closure entry naming the same authoritative version and the
enforcer that delivered it. `p`'s own retained result is never read, so the stage
has no source endpoint and no source-side dependency on it — its dependencies are
its operands' deliveries.

If `B` binds a different target, a different compiler build, or a different
output-affecting configuration from `A`, the second condition is what fails, and
it fails as `MechanismPreflight`/`RecomputeValuePreservationUnproved` before any
program work. The planner then selects the transfer. It does not recompute at a
lower cost and deliver whatever `B` produces.

### Managed migration

The backing allocation may retain identity while the provider changes its
resident/authoritative location. The stage declares the migration and
coherence protocol and forbids unordered conflicting accesses. It is not
modeled as allocation-to-allocation byte copy, and a prefetch hint is not
treated as proof that migration or visibility completed.

## Verifier invariants

The portable verifier proves structural items 1–9; runtime preflight re-proves
bound items 10–16 against live capabilities:

1. all endpoint range arithmetic is checked and every touched range lies in
   its view's admitted allocation range;
2. transfer source and destination name the same logical value version and
   storage encoding;
3. an encoding change uses the separately typed `RepackEncoding` enforcer and a
   dtype change uses the separately typed `ConvertDtype` stage, which is not an
   enforcer; neither may be absorbed into a transfer;
4. every source producer reaches the transfer, every internal leg is ordered,
   and every destination consumer depends on destination readiness;
5. the dependency graph is acyclic and every token has one governed meaning;
6. required resource roles exist and each has a final-use release condition;
7. copy overlap is rejected unless explicit overlap-safe semantics are proved;
8. alias elimination carries every required proof component;
9. a `Recompute` carries a complete `RecomputeValuePreservation` record — equal
    implementation identity, equal delivered numerical realization, the
    plan-deterministic guarantee at its executing scope, and one operand-closure
    entry per operand — and its destination write overlaps no operand it reads;
10. concrete endpoints match symbolic affinities, domains, allocations,
    generations, ranges, encodings, access modes, and alignment;
11. the chosen mechanism is admitted for the directed endpoint pair and exact
    ranges, including every staged leg;
12. synchronization objects cover all participating queue/device scopes and
    establish the promised visibility/coherence;
13. overlapping access uses are read/read or ordered by a sufficient hazard
    protocol;
14. staging allocation capacity, alignment, accessibility, and coherence hold;
15. the adapter can retain every resource and exact receipt across all success,
    failure, cancellation, and partial-submission paths; and
16. no allocation/import/encoding begins until commit has consumed fallback
    authority.

Failing an invariant produces a typed, explainable rejection or postcommit
failure. It never becomes infinite cost or an implicit copy.

## Bounded executable spike

[`spikes/transfers/transfer_contract.rs`](../../../spikes/transfers/transfer_contract.rs)
implements checked view/backing ranges, preserved encoding/version,
two-sided dependencies, staged-leg ordering, role-based retention, alias-proof
requirements, the recompute value-preservation record, overlapping-access
hazards, copy-overlap rejection, and the commit/cancellation/release state
machine.

Twenty-one tests pass. The positive examples cover CPU-to-accelerator,
accelerator-to-CPU, same-device materialization, peer direct copy, peer access,
shared-backing alias, managed migration, host-staged transfer, and a
recomputation carrying a complete value-preservation record. Negative
tests cover hidden conversion, invalid ranges,
missing source or consumer dependencies, incomplete alias proof, missing
staging retention/order, a recomputation missing each one of the four
conditions, a recomputation naming no operand to close over, a recomputation
whose destination overwrites an operand it reads, unordered write hazards,
overlapping copy, fallback after commit, cancellation before terminal release,
and failure after staged work has begun.

The recompute model is deliberately narrow in one way worth naming: it takes the
four conditions as stated booleans and checks that all four are present, rather
than deriving any of them. Deriving them needs an implementation-identity
encoding, a delivered-realization record, and a determinism level that live in
`tiler-ir`, `tiler-artifact`, and the physical planner respectively, none of
which this dependency-free file can reach. What it does establish is that the
verifier's structural item is expressible over the same plan shape as the other
mechanisms, that a recomputation is checked against its operands rather than
against a source version, and that a missing condition fails closed as one typed
rejection rather than degrading the delivery.

The spike is dependency-free and synchronous. It does not bind real queues,
events, allocators, devices, pageable/pinned memory, managed residency, or
provider errors. Passing tests demonstrate consistency of the proposed
invariants over the modeled traces, not backend conformance or performance.

## Scope boundary and follow-on evidence

**Deferred question, with its trigger — how condition 2 is checked across two execution sites that do not share a delivered-realization record.** Within one artifact, one target, and one compiler build, the recomputation and its reference share a delivered realization by construction, and ADR 0076's honesty rule keeps declared and delivered together for any artifact that exists. Across two of any of those, condition 2 becomes a comparison of two stated records, and the record it must compare does not exist: [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) item 4 requires a produced artifact to carry a readable record of "the numerical realization actually delivered", and its `implementation_status` is `partial` with that item unstarted. This memo therefore states the condition and does not state its check. The trigger for closing it is whichever comes first: [`record-delivered-numerical-realization`](../../../tickets/record-delivered-numerical-realization.md) landing, which supplies the record to compare; or a second symbolic affinity becoming executable, which is when a recomputation can first reach a site that does not share the reference's compiler. Until then the executable profile's one affinity, one live device, and one ordered stream make the comparison vacuous, and a recomputation to any other site fails condition 2 closed.

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

- **Current disposition:** pending. The [proposed CPU/SIMD target profile](../../backends/cpu.md)
  is the only contract that names this report as evidence, and it cites it for a
  physical-resource boundary rather than reproducing the transfer, dependency,
  hazard, or retention rules above.
- **Normative destinations:** the [artifact ABI](../../artifact-abi.md),
  [Candle integration](../../integration/candle.md), CPU target profile, and
  [Metal AOT backend](../../backends/metal.md) contracts. None of them carries
  this content yet. ADR 0047 requires such a layer but takes its evidence from
  the device/memory-domain research instead, so no accepted decision adopts this
  proposal.
- **Reproduction:** the [transfer spike](../../../spikes/transfers/README.md)
  exercises the proposed invariants. Backend-specific multi-device measurements
  and calibrated transfer costs remain future evidence.
