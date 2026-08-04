---
schema: "tiler-doc/v1"
id: "tiler.contract.runtime-state"
kind: "contract"
title: "Runtime state boundary"
topics: ["runtime", "state", "kv-cache", "identity", "placement", "lifetime"]
contract_status: "proposed"
implementation_status: "not-started"
evidence: ["tiler.research.runtime.autoregressive-state-and-kv-cache", "tiler.research.runtime.execution-contract", "tiler.research.transfers.synchronization-lifetime"]
ticket: "define-the-runtime-kv-state-boundary"
---

# Runtime state boundary

**Status:** proposed public boundary. The concrete surface below is a reviewable draft, not an accepted API and not an implementation claim. Tom must accept the consequential public boundary before production code adopts it.

## Scope and evidence

This contract specializes the consumer-neutral [runtime execution contract](../research/runtime/runtime-execution-contract.md) for the first retained inference state: the dense-decoder [autoregressive KV cache](../research/runtime/autoregressive-state-and-kv-cache.md). It is governed by the accepted placement, routing-commit, validation, and lifetime decisions cited by those records.

**Fact.** The semantic program can state `S = C + T` with `ExtentRelation::AdditiveEquality`, but no runtime consumer yet evaluates that retained relation against live invocation bindings. The initial placement profile is one symbolic affinity, one live device, and one ordered stream. Concrete device handles, contexts, queues, allocations, and completion objects stay below the compiler/runtime boundary. `RoutingCommit` is one-way before program allocation, and initial transactions are out of place.

**Inference.** A reusable state boundary must therefore carry enough runtime-instance identity to reject stale or foreign state without putting any mutable fact in semantic, artifact, expansion-cache, library-cache, or pipeline-cache identity. It must also make publication conditional on the exact execution's observed terminal success, because allocation bytes and host belief can disagree even when the old allocation is physically intact.

**Proposal.** The types and transitions below are the complete first public boundary. Names are exact for acceptance review; field layout and private helper organization remain implementation details.

**Measurement.** None. This document makes no latency, allocation-cost, or device-support claim.

## Decision elimination: where device scoping lives

Three candidates were tested against correctness, performance, and long-term maintainability.

| Candidate | Correctness | Performance | Maintainability and support | Disposition |
| --- | --- | --- | --- | --- |
| Put a platform device handle or platform API type in the consumer-neutral runtime surface | Can compare the concrete object, but violates ADR 0047's dependency direction and makes a portable runtime crate depend on one backend | No useful advantage over comparing an already-bound token | Prevents consumer-neutral adapters and makes every new platform widen the core type | Eliminated |
| Keep the entire state scope and comparison private and ad hoc inside each adapter | Can be correct if every adapter independently implements every identity and refusal rule | The comparison can be constant-time | Duplicates the same fail-closed rule, gives generic state transitions no typed scope, and makes cross-adapter conformance depend on convention | Eliminated |
| Define a governed opaque `LiveStateScope` in the runtime surface, minted by the adapter from its private live device/context and compared by the generic state boundary | Rejects a foreign device or context without exposing either object | Fixed-size equality; no device query, allocation, or synchronization | One public refusal and conformance rule; adapters retain platform ownership and may choose their stable runtime-scoped token | **Survives** |

The surviving candidate is not a product-priority trade. The other two fail an architectural invariant or leave one silent-wrongness check conventional, so device scoping is fixed autonomously as the governed opaque-token design. `LiveStateScope` is not a portable device identity, not a target profile, and never enters an artifact or cache key.

## Concrete public draft

The public surface is parameterized over adapter-owned storage and retention types; it owns no device API.

```text
StateInterfaceKey            // opaque subject: validated semantic-graph identity,
                             // ordered logical interface, and input ordinal
LayerOrdinal(u32)            // bounded layer position within that interface
StateGeneration(u64)         // checked, monotonically advancing publication generation
StateCursor(u64)             // C, the sole valid-length authority
StateCapacity(u64)           // fixed at construction

LiveStateScope               // opaque, Eq + Hash, no public fields
  minting authority: RuntimeAdapter
  semantic components: backend family, representation, runtime instance,
                       adapter-private device token, execution-context token
  forbidden contents: platform handle, portable ordinal, artifact identity

StateInterfaceKey::from_decoded_program(&DecodedProgram, input_ordinal)
  -> Result<StateInterfaceKey, StateInterfaceKeyError>
                             // resolves the ordinal through DecodedProgram::inputs;
                             // no caller-byte or detached-input constructor

LiveStateScopeFactory        // non-Clone capability; no public constructor
  mint(device_token_bytes, context_token_bytes)
    -> Result<CurrentLiveStateScope<'context>, LiveStateScopeBuildError>

CurrentLiveStateScope        // non-Clone capability tied to the current route's
                             // private LiveExecutionContext; no raw constructor

RuntimeAdapter::bind_live_state_scope(
  &mut self,
  &LiveExecutionContext,
  LiveStateScopeFactory,
) -> Result<CurrentLiveStateScope<'_>, Self::Refusal>
                             // the route supplies the factory only to the adapter

KvStateIdentity {
  program_interface: StateInterfaceKey,
  layer: LayerOrdinal,
  live_scope: LiveStateScope,
  generation: StateGeneration,
}

KvExecutionIdentity          // opaque runtime-instance identity of one attempt;
                             // binds state identity before the attempt, T, and
                             // consumer execution ordinal

KvStateStatus {
  Ready,
  Poisoned { failed_execution: KvExecutionIdentity },
}

KvState<Storage, Retention>  // singular named member; private fields and no
                             // independent constructor or mutation path
  identity()
  interface_key()
  layer()
  scope_identity()            // presentation-safe identity, not current authority
  generation()
  capacity()
  cursor()
  status()
KvStateSet<Storage, Retention> // bounded ordered complete transaction set
  members() -> ExactSizeIterator<&KvState>
  retire(self)                // retires every member; final use still governs release

PreparedKvStateSet           // non-Clone; carries every ordered member snapshot,
                             // C, T, checked S, generations, fingerprints, and scope
  cursor() -> StateCursor
  step() -> u64
  total() -> u64

DecodedProgram::state_interface()
  -> DecodedStateInterface   // validated artifact-owned complete ordered manifest;
                             // rows name cache input/layer/axis, step input/axis,
                             // retained output/axis, and S = C + T relation

KvArtifactStateBindingSet    // non-Clone exact ordered bijection derived from the
                             // authoritative decoded manifest, never a caller list
  from_decoded_program(&DecodedProgram)
    -> Result<KvArtifactStateBindingSet, KvArtifactBindingError>

KvRouteSession<'adapter, A> // non-Clone; owns one &mut A, one LiveExecutionContext,
                            // and its CurrentLiveStateScope from one context bind
  bind(&'adapter mut A) -> Result<KvRouteSession<'adapter, A>, A::Refusal>
  construct_set(self, StateCapacity,
                ExactSizeIterator<KvStateMemberSpec<Storage, Retention>>)
    -> Result<KvStateSet, KvStateSetBuildError>
  prepare(self, &KvStateSet, expected_cursor, T,
          KvArtifactStateBindingSet, &mut DecodedProgram,
          expected_artifact, AbiFactBinder)
  -> Result<PreparedKvRoute, KvRoutePreflightError>
                            // preflights every set member with this session's scope,
                            // binds every C/T/S relation, and carries this same
                            // adapter/context through plan_dispatch

PreparedKvRoute             // non-Clone; carries PreparedKvStateSet, session,
                            // and exact loader Preflight minted from those facts
  commit(&mut KvStateSet, consumer_execution_ordinal)
    -> Result<BoundKvTransaction<'state, 'program, 'adapter, A>, KvStateSetRefusal>
                            // revalidates all members, mints KvExecutionIdentity,
                            // then consumes Preflight::commit

BoundKvTransaction          // non-Clone; owns exclusive set borrow, adapter/context,
                            // identity, and RoutedDispatch; poisons every member
                            // if dropped unfinished
  routed_dispatch()
  execute(self)
    -> Result<ExactTerminalSuccess<A::Completion>,
              KvPostCommitFailure<A::Failure>>
                            // drives the carried adapter/dispatch; adapter returns
                            // identity-bound replacements to the transaction,
                            // which validates all then publishes atomically

RuntimeAdapter {
  type StateStorage;
  type StateRetention;
  observe_state_storage(
    &mut self, &LiveExecutionContext, &Self::StateStorage,
    StateObservationFactory,
  ) -> Result<StateStorageObservation, Self::Refusal>
  execute_state_transaction(
    &mut self, &LiveExecutionContext, &RoutedDispatch,
    StateTransactionReporter<Self::StateStorage, Self::StateRetention,
                             Self::Completion, Self::Failure>,
  ) -> StateTransactionReport<Self::StateStorage, Self::StateRetention,
                              Self::Completion, Self::Failure>
}

StateObservationFactory     // non-Clone, no public constructor; session-bound
  resource_key(adapter_resource_token_bytes, resource_generation)
    -> Result<StateResourceKey, StateObservationError>
  receipt_key(adapter_receipt_token_bytes)
    -> Result<SubmissionReceiptKey, StateObservationError>
                             // factories attach current scope or execution identity

StateStorageObservation {   // private fields, runtime-readable
  resource: StateResourceKey,
  memory_domain: MemoryDomainKey,
  byte_range: CheckedByteRange,
}

StateMemberEvidence {
  expected_member: KvStateIdentity,
  storage: StateStorage,
  storage_observation: StateStorageObservation,
  coherence_receipt: SubmissionReceiptKey,
  validation_receipt: SubmissionReceiptKey,
  retention: StateRetention,
  retained_through: SubmissionReceiptKey,
}

SubmissionObservation {
  receipt: SubmissionReceiptKey,
  terminal: ExactTerminalStatus,
}

StateTransactionReporter    // non-Clone, no public constructor; bound transaction
  failure(KvFailureStage, F) -> StateTransactionReport
  success(C, SubmissionObservation,
          ExactSizeIterator<StateMemberEvidence<Storage, Retention>>)
    -> StateTransactionReport

StateTransactionReport      // opaque; reporter attaches execution identity and
                            // expected complete ordered member population

ExactTerminalSuccess<C>      // opaque completion; minted by the bound transaction
                             // only after exact receipt/coherence/validation/publication

KvPostCommitFailure<F>       // opaque, no public constructor; transaction attaches
  execution()                // its own identity to adapter-supplied stage/cause
  stage()
  cause()

KvPostCommitFailureCause<F> = Adapter(F) | InvalidReport(KvStateReportError)

KvFailureStage = Allocation | StorageBinding | Encoding | Submission
               | Completion | Coherence | ValidationReadback
               | Retention | Publication
```

`StateInterfaceKey` is minted only while holding the whole `DecodedProgram`. Its subject is the decoded envelope's already-validated semantic-graph identity, the complete input/output logical interface in semantic order (stable keys, logical shapes, and resolved logical types), and the selected input's ordinal and stable key. The constructor resolves that ordinal through `DecodedProgram::inputs()` and refuses an absent slot. It is not a second caller-authored grammar, cannot be constructed from arbitrary bytes or a detached `DecodedInput`, and does not include physical components, artifact canonical identity, payloads, variants, delivery position, provenance, or any live fact. Two unrelated programs that both call an input `k_cache` therefore do not share state identity; K and V in one program remain distinct by ordinal and key.

The artifact's governed state-interface manifest is the sole population authority. Its encoder derives rows from the verified semantic retained-state declarations and additive extent relations, encodes every row in semantic input order, and the decoder refuses duplicate, missing-target, ill-typed, unrelated, or non-injective rows before exposing `DecodedProgram`. The manifest is artifact content and therefore covered by artifact identity, but a *live state instance* is not: rows contain only semantic/interface slots, axes, layer ordinals, and relation references. `KvArtifactStateBindingSet::from_decoded_program` consumes that complete decoded view without accepting caller specs, so omitting V cannot redefine a K-only set as complete. The dependent artifact/runtime binding ticket owns adding and validating this manifest and its schema/identity step; this proposed state boundary fixes the required consumer surface and does not claim it exists today.

`KvStateIdentity` includes the generation because a binding prepared against generation `g` must not bind after another execution publishes generation `g + 1`. It deliberately excludes the cursor and capacity as separate identity fields: the generation is the version of the immutable published snapshot that contains them. Both values remain checked state metadata, never specialization values or cache-key material.

`LiveStateScope` has no public constructor from caller bytes. A runtime adapter authenticates it only after binding its private live device and execution context; two values compare equal only within the declared runtime-instance scope. A reset, context replacement, provider token change, or runtime-instance replacement mints a different scope.

The external adapter can authenticate the current scope only through `LiveStateScopeFactory`, a non-Clone capability constructed inside the runtime route after `LiveExecutionContext` exists and passed only to `RuntimeAdapter::bind_live_state_scope`. The adapter supplies bounded stable token bytes for its private device and context; the factory binds them to the observed backend family, representation, runtime-instance nonce, and the private current-context lifetime before constructing `CurrentLiveStateScope`. That capability contains the comparable `LiveStateScope` but does not expose a raw-parts constructor. The tokens are identities, not serialized platform handles.

`KvState::scope_identity()` is diagnostic/comparison output only. No constructor or preflight accepts it or an arbitrary `LiveStateScope`. `KvRouteSession::bind` performs `RuntimeAdapter::bind_execution_context` exactly once, promotes that observation into its private `LiveExecutionContext`, supplies the scope factory to that same adapter, and retains the adapter borrow, context, and resulting `CurrentLiveStateScope` together. Its consuming `construct_set` uses that authority for initial members; a later session's consuming `prepare` uses newly authenticated authority for every existing member and carries that same adapter/context into the prepared and bound route. A caller who reads an old scope therefore cannot present it as current authority, and context A cannot authenticate state while context B dispatches it.

One `KvState` remains the singular named allocation and identity required by this contract, but it is constructed and mutated only as a member of `KvStateSet`. The set is bounded by the decoded artifact interface limit (`4,096` members), sorted in semantic input order, and rejects empty, oversized, duplicate, missing, extra, or out-of-order membership. All members share one semantic-graph/logical-interface subject, live scope, capacity, cursor, generation, and readiness status; each retains its own input slot/layer identity, storage fingerprint, allocation, and retention. This generic set says nothing about a model's layer count. The model boundary chooses the exact 28-layer composition and token policy.

`KvExecutionIdentity` is not an artifact subject and has no caller constructor. `PreparedKvRoute::commit` mints it from the complete prepared member snapshots, `T`, the exact `RoutedDispatch`, and the caller's diagnostic execution ordinal. The bound transaction carries it into every adapter operation, exact-success observation, failure report, and poison transition; neither success nor failure accepts a restated identity. Its presentation may include the consumer ordinal, but equality is over the complete opaque runtime identity.

### Meaning and concrete spelling

The boundary's **meaning** is that one adapter-authenticated runtime-instance scope covers one live device and execution context; authority tied to the currently bound route, not equality supplied by the consumer, is required at every state bind, and a reset or context replacement invalidates it. The runtime interprets no adapter token and exposes no platform object. `StateInterfaceKey` similarly means one exact input slot in one decoded semantic graph and ordered interface, not caller text or a locally stable input key that happens to match. These validation obligations survive any later Rust renaming.

The **concrete spelling proposed for acceptance** is the type and method inventory above: `LiveStateScope`; non-Clone, runtime-minted `LiveStateScopeFactory` and `CurrentLiveStateScope`; `RuntimeAdapter::bind_live_state_scope`; `StateInterfaceKey::from_decoded_program` and `StateInterfaceKeyError`; singular read-only `KvState`; constructing/mutating `KvStateSet`; non-Clone context-bound `KvRouteSession`; artifact-owned `DecodedStateInterface` and validated `KvArtifactStateBindingSet`; `PreparedKvRoute::commit` consuming the exact existing loader `Preflight` it carries; `BoundKvTransaction` carrying its identity, adapter/context, set guard, and `RoutedDispatch`; adapter `StateStorage`/`StateRetention` and `execute_state_transaction` reporter protocol; opaque `StateTransactionReport`, `ExactTerminalSuccess`, and `KvPostCommitFailure`; and `KvFailureStage`. No public raw-parts constructor, caller-authored state-manifest population, detached-input constructor, independent state constructor, mutating cursor/generation/scope/status setter, execution-identity constructor, success/failure identity argument, or public way to join arbitrary state preparation to an arbitrary `Preflight` exists.

## Construction and representation invariants

A state set is created from a current adapter-authenticated scope, ready at cursor `0`, member generation `0`, and a positive fixed capacity selected from the consumer's declared workload bound. The constructor consumes `CurrentLiveStateScope`; a consumer cannot stamp storage with a replayed comparable identity. Each initial dense member has F32 storage with physical capacity `[8, capacity, 128]`, row-major and unpacked. Its logical program view is `[8, C, 128]`; bytes outside sequence range `[0, C)` are unreachable and meaningless.

The constructor validates checked allocation arithmetic for `8 × capacity × 128 × sizeof(F32)`, the adapter-declared memory domain and alignment, and that the backing allocation reaches the required physical capacity. Capacity never grows. Replacing a state with a larger allocation is construction of a different state, not growth of this one.

The identity's `program_interface` and `layer` prevent two same-shaped cache tensors from different interfaces or layers from aliasing by convention. K and V remain distinct named program-interface states; the interface key includes which named slot this object realizes rather than relying on a label supplied at bind time.

No live state value, identity, scope, generation, cursor, capacity, allocation, or failure status appears in:

- semantic graph identity;
- artifact canonical identity or descriptor bytes;
- expansion-cache identity;
- library or pipeline cache identity; or
- specialization values.

The artifact carries formulas over bound `C`, `T`, and `S`, never their invocation values. The runtime binds those values at preflight from the state and request.

## Preflight and typed refusals

`KvRouteSession::prepare` preflights the complete ordered set before artifact routing, allocation, encoding, or other program work. It rejects missing, extra, duplicate, or out-of-order requested members first, then checks each member in semantic input order so one invocation has a deterministic primary refusal:

1. status is `Ready`;
2. the member interface key agrees;
3. member layer ordinal agrees;
4. the route-authenticated `CurrentLiveStateScope` contains the same identity as the state's scope;
5. caller-stated `expected_cursor` equals the state's cursor;
6. checked addition computes `S = C + T` without overflow;
7. the generation can advance without overflow;
8. `S <= capacity`; and
9. the logical storage view agrees with `[0, C)` and the preparation-time storage fingerprint.

The same consuming session then joins those snapshots to `KvArtifactStateBindingSet` derived from that decoded program's authoritative manifest. It requires an exact ordered bijection between set members and all manifest rows, binds every member's exact `C` and request `T` through the supplied `AbiFactBinder`, rejects a contradictory fact, evaluates every retained additive relation and requires each output to equal checked `S`, then passes the frozen `AbiFacts` through the existing adapter qualification/preparation/plan stages. The returned `PreparedKvRoute` is the only public join between state preparation, one live context, and loader `Preflight`; none can be substituted after the join.

The public refusal inventory is exhaustive and has no catch-all variant:

```text
KvStateRefusal {
  PoisonedState { failed_execution },
  StateInterfaceMismatch { expected, observed },
  LayerMismatch { expected, observed },
  ForeignLiveStateScope { expected, observed },
  StaleGeneration { expected, observed },
  StaleCursor { expected, observed },
  CursorOverflow { cursor, step },
  GenerationExhausted { generation },
  CapacityExceeded { cursor, step, required, capacity },
  InvalidStorageView { required_range, observed_range },
  StaleStorage { expected_fingerprint, observed_fingerprint },
}

KvStateSetRefusal {
  MissingMember { expected },
  ExtraMember { observed },
  DuplicateMember { member },
  MemberOutOfOrder { previous, observed },
  MixedProgramInterface { member, expected, observed },
  MixedCursor { member, expected, observed },
  MixedGeneration { member, expected, observed },
  MixedLiveStateScope { member, expected, observed },
  Member { member, refusal: KvStateRefusal },
}
```

`StateStorageFingerprint` is an opaque, presentation-safe runtime identity over the adapter resource identity and generation, memory domain, physical allocation range, and logical valid range observed during preparation. It is not content identity and never enters artifact or cache identity. `StaleStorage` catches replacement or mutation that preserves the same logical range; `InvalidStorageView` catches a range disagreement.

The decoded-artifact join has its own exhaustive pre-commit errors: an absent or foreign interface slot, a cache/step/output axis mismatch, no retained additive relation connecting those exact slots, a duplicate or contradictory ABI fact, an evaluated total unequal to the prepared `S`, or the existing typed loader/adapter refusal. None is a post-commit failure and all leave the state ready and unchanged.

```text
StateInterfaceKeyError {
  UnknownInputOrdinal { requested, inputs },
}

KvArtifactBindingError {
  NoStateInterface,
  ForeignStateInterface,
  MissingMember { expected },
  ExtraMember { observed },
  DuplicateMember { member },
  MemberOutOfOrder { previous, observed },
  UnknownInputOrdinal { requested, inputs },
  UnknownOutputOrdinal { requested, outputs },
  InvalidSequenceAxis { slot, axis, rank },
  MissingAdditiveExtentRelation,
  AmbiguousAdditiveExtentRelation { matches },
}

KvRoutePreflightError<R> {
  StateSet(KvStateSetRefusal),
  ArtifactBinding(KvArtifactBindingError),
  AbiBinding(AbiBindingError),
  ExtentRelationMismatch { cursor, step, expected_total, observed_total },
  Route(R),
}

KvStateReportError {
  MissingReplacement { expected },
  ExtraReplacement { observed },
  DuplicateReplacement { member },
  ReplacementOutOfOrder { previous, observed },
  MemberIdentityMismatch { expected, observed },
  SubmissionReceiptMismatch,
  NonterminalObservation,
  IncompleteCoherence { member },
  IncompleteValidationReadback { member },
  InvalidReplacementStorage { member },
  InvalidRetention { member },
}

StateObservationError {
  EmptyResourceToken,
  ResourceTokenTooLong { length, maximum },
  EmptyReceiptToken,
  ReceiptTokenTooLong { length, maximum },
  InvalidByteRange { offset, length },
}
```

`R` is the existing typed pre-commit loader/adapter refusal rather than an erased string. These enums are exhaustive at this boundary; the existing adapter route wrapper may remain `#[non_exhaustive]` under its own accepted convention.

Construction has a separate exhaustive error boundary because malformed initial storage is not an attempted execution:

```text
LiveStateScopeBuildError {
  EmptyDeviceToken,
  DeviceTokenTooLong { length, maximum },
  EmptyContextToken,
  ContextTokenTooLong { length, maximum },
}

KvStateBuildError {
  ZeroCapacity,
  AllocationSizeOverflow { capacity },
  InvalidStorageCapacity { required_bytes, observed_bytes },
  InvalidMemoryDomain { required, observed },
  InvalidAlignment { required, observed },
}

KvStateSetBuildError {
  EmptyStateSet,
  TooManyMembers { actual, maximum: 4096 },
  DuplicateMember { member },
  MemberOutOfOrder { previous, observed },
  MixedProgramInterface { member, expected, observed },
  Member { member, error: KvStateBuildError },
}
```

The adapter may wrap these in its consumer-specific error, but it may not erase the class or turn one into a route miss. `PoisonedState` names the execution that poisoned the state. `ForeignLiveStateScope` names presentation-safe identities for the stored and currently authenticated scopes and never formats a platform handle.

## Update, publication, and failure

The first update is always out of place. The old allocation is bound read-only over `[8, C, 128]`; the new allocation is distinct and has the same fixed physical `[8, capacity, 128]` capacity while the program writes only its declared `[8, S, 128]` logical output. Input and output sharing one allocation is refused before commit by the existing `ForbiddenAlias` rule.

After `RoutingCommit`, every allocation, binding, encoding, submission, completion, coherence, validation-readback, retention, and publication failure is terminal for the attempt and poisons the complete state set. There is no fallback and no reuse of any still-intact old member: the consumer did not receive the failed step's token, so continuing from the old cursor would execute a different sequence from the one the consumer believes it owns.

The existing adapter's combined `allocate_dispatch`/`dispatch` result is insufficient for state publication, so acceptance includes `observe_state_storage`, `execute_state_transaction`, and associated storage/retention types. `StateObservationFactory` attaches the session's current scope to bounded adapter resource tokens and the bound execution identity to bounded receipt tokens; the adapter reports domain and checked range but cannot mint a key for another scope or execution. Initial construction and every replacement use the same governed `StateStorageObservation`, from which the runtime derives and later compares the storage fingerprint. The bound transaction creates `StateTransactionReporter` from its own execution identity and authoritative manifest population and gives that capability to the adapter with the exact context and routed dispatch. The adapter cannot construct `StateTransactionReport` directly. It returns either `reporter.failure(stage, cause)` or `reporter.success(...)` with one terminal receipt and an exact ordered `StateMemberEvidence` population. The reporter binds the report to this execution; it does not trust `A::Completion` to contain evidence by convention.

`BoundKvTransaction::execute` validates the returned report before publication. The generic runtime—not the adapter—compares each member identity, resource key/generation, attached live scope, memory domain, checked byte range, coherence receipt, validation receipt, and retained-through receipt against the prepared member and single terminal submission receipt. An adapter-reported failure becomes `KvPostCommitFailureCause::Adapter`; an absent/extra/duplicate/unordered member, mismatched observation, foreign/reused storage, wrong receipt, incomplete coherence/readback coverage, or retention defect becomes `InvalidReport(KvStateReportError)` at the exact stage the transaction was validating. Both attach the transaction's identity and poison all members. The adapter is authoritative only for its observations; the runtime owns every comparison and verdict.

Publication requires a complete ordered replacement set and `TerminalSuccess` tied to the bound transaction's own `KvExecutionIdentity` and submission receipt. Before mutating anything, `execute` verifies that replacements are neither missing, extra, duplicate, nor out of order; each replacement names its member identity, has distinct old/new allocation, exact capacity/domain/range, retention lease, coherence, and validation record; and the receipt covers the whole route. Only then does one infallible critical section replace every member and the shared cursor/status:

```text
(all old member allocations, C, member generations, Ready)
  -> (all new member allocations, C + T,
      every member generation + 1, Ready)
```

The checked cursor addition and generation increment cannot wrap. `GenerationExhausted` is a pre-commit refusal for the next attempt; it is never repaired by resetting the generation inside the same state identity.

Every non-success transition after commit instead performs one set-wide transition:

```text
(all old member allocations, C, member generations, Ready)
  -> (all old member allocations, C, member generations,
      Poisoned { failed_execution })
```

The cursor therefore advances by exactly `T` once, on exact terminal success, and by zero on every refusal, device error, nonterminal observation, cancellation, retention error, or publication failure. Publication cannot expose a subset of new allocations, `(new members, old cursor)`, or `(old members, new cursor)`. If validation of even the last replacement fails, no member moves and the already-committed set becomes poisoned.

`PreparedKvStateSet` carries every preflighted identity and storage fingerprint but no mutable borrow. `PreparedKvRoute::commit` takes the complete set, revalidates its membership and every identity, generation, cursor, scope, status, and fingerprint, and returns `KvStateSetRefusal` while its loader authority remains uncommitted. Only after all revalidation succeeds does it mint `KvExecutionIdentity`, call consuming infallible `Preflight::commit`, and construct `BoundKvTransaction` with the set guard already attached. A change to one member reaches a member-indexed refusal before `RoutingCommit`. Allocation and dispatch are reachable only through that transaction. Its consuming `execute` derives success or failure identity internally. Dropping it through early return or panic poisons every member; there is no interval after commit with no poison guard and no path that restores only part of the set to `Ready`.

## Placement, aliasing, retention, and destruction

Every state-set member occupies the one symbolic affinity's admitted memory domain under the initial ADR 0047 profile. The set creates no new domain and no transfer edge. The adapter validates that all old and new allocations belong to the session's bound live scope and the plan's required domain; one foreign member refuses the whole set rather than requesting a transfer.

Old and new allocations are distinct. Both, plus every view, pipeline, command object, argument resource, and synchronization object that can reach either, are retained through exact final device use. A successful publication may release the old allocation only after the extending execution's completion receipt proves its last read complete. The new allocation remains retained by the published state and by every later in-flight use. A poisoned state retains whatever resources still have device use until their receipts permit release.

The runtime instance owns the live allocation, metadata, status, and retention leases. The consumer owns the state handle/session lifetime and explicitly destroys it; destruction asks the runtime instance to retire the object and defers resource release until every exact final-use receipt permits it. Dropping a host reference is not completion evidence.

## Ownership by layer

| Layer | Owns | Must not own |
| --- | --- | --- |
| Semantic program | Named K/V state inputs and retained outputs; concatenation meaning; `S = C + T` | Capacity, cursor value, generation, allocation, device/context, poison status |
| Physical plan | Row-major layout, accessible domains, distinct input/output storage, realization and guard choice | Live allocation identity, consumer token position, artifact-independent mutable state |
| Artifact | Ordered named slots; formulas for logical ranges and launch geometry over `C`, `T`, `S`; fixed variants and canonical artifact identity | State identity, scope, capacity, generation, cursor value, allocation, poison status, specialization on any cursor-derived value |
| Runtime instance | Singular `KvState` members inside a bounded `KvStateSet`; fixed capacity, cursor/status, member generations, atomic set replacement, retention leases | Platform device API objects in its consumer-neutral types; semantic meaning; model layer count; sampling policy |
| Runtime adapter | Minting `LiveStateScope`; private device/context and storage objects; domain/alias checks; allocation; submission receipts; terminal observation; exact final-use retention | Portable artifact identity changes; semantic graph changes; silent recovery after commit |
| Consumer | State-handle lifetime, expected execution ordinal, one cursor authority for cache extent/rotary rows/mask, sampling, termination, explicit destruction | Direct cursor mutation, publication before success, reconstructing state identity from labels |

## Negative examples

### Stale state

State generation `g` has cursor `13`, but the consumer tries to bind `expected_cursor = 14` because it retained a handle from another step. Capacity is `18`, so every byte range would fit. Preflight returns `StaleCursor { expected: 14, observed: 13 }` before artifact routing. Supplying an old prepared binding after publication instead returns `StaleGeneration`.

### Capacity overflow

State cursor is `17`, step length is `2`, and capacity is `18`. Checked addition obtains required context `19`; preflight returns `CapacityExceeded { cursor: 17, step: 2, required: 19, capacity: 18 }`. No route is selected and no allocation occurs. `C = u64::MAX, T = 1` returns `CursorOverflow` rather than wrapping.

### Cross-device or cross-context reuse

An adapter presents scope B for a state minted under scope A. Even if both devices advertise the same target profile, backend family, and representation, preflight returns `ForeignLiveStateScope`. A different context on the same device also mints a different scope and refuses. No transfer or implicit rebind is attempted.

Reading state A's `scope_identity()` and passing that value back cannot defeat this refusal: neither `KvRouteSession::construct_set` nor route preparation accepts it. Both use the non-Clone authority held by the same kind of context-bound session that constructs or routes the set.

### Same local key in another program

Programs P and Q both name input ordinal 2 `k_cache`, with the same type and shape. Their semantic graph identities or complete ordered interfaces differ. `StateInterfaceKey::from_decoded_program(P, 2)` and `StateInterfaceKey::from_decoded_program(Q, 2)` are unequal, so binding P's state while routing Q returns `StateInterfaceMismatch`; the repeated local `InputKey` cannot alias them.

### Contradictory artifact facts

A prepared state set carries `C = 14`, `T = 1`, and checked `S = 15`. Each decoded binding names the exact cache input, step input, extended output, sequence axes, and retained additive relation. If the supplied ABI binder already says one cache axis is `13`, the join returns its typed duplicate/contradictory binding error. If one decoded relation evaluates to any value other than `15`, it returns that member's `ExtentRelationMismatch`. Neither case can produce a loader `Preflight` or reach routing commit.

### Storage replacement between preparation and commit

A set was prepared with one member at allocation generation A and logical range `[0, 14)`. Before commit, storage generation B is installed for that member with the same range. Range-only checking would accept it; member-indexed fingerprint revalidation returns `StaleStorage` before consuming the loader preflight.

### Complete K/V transaction

One decoded route declares K and V state pairs in semantic order. Omitting V, repeating K, swapping their order, or adding a state not named by the route refuses before loader preflight. After commit, K replacement validates but V replacement fails validation readback. No publication has started: both remain at the old cursor and generations and the complete set becomes poisoned. When all replacements and the exact receipt validate, one infallible critical section publishes both; no observer can see new K with old V.

### Poisoned state

Execution `e5` crossed `RoutingCommit` and then its submission reported a device error. The state remains at its previous cursor and generation but becomes `Poisoned { failed_execution: e5 }`. Every later preflight returns `PoisonedState { failed_execution: e5 }`; the consumer must construct a fresh state from a known prefix.

### Cursor advancement after non-success

Starting from `(C = 14, generation = 5)`, a step with `T = 1` reaches a waiting state whose receipt is nonterminal, errors, or belongs to another execution. None is `TerminalSuccess(e6)`. The only legal result is `(C = 14, generation = 5, Poisoned(e6))`; `(15, 6, Ready)` is constructible only from the exact successful receipt. A pre-commit capacity refusal leaves `(14, 5, Ready)` instead, because no committed execution existed to poison it.

An early return immediately after `PreparedKvRoute::commit` still drops a fully constructed `BoundKvTransaction`, whose drop poisons every member. There is no public sequence that consumes `Preflight::commit` first and attaches the set guard later. The adapter reports one governed `KvFailureStage` with its cause, while the transaction wraps both with its own execution identity; malformed success-report validation uses the stage the transaction itself is checking. An adapter therefore cannot poison the set under a different attempt's label or erase the failure class into one opaque completion error.

## Unsupported cases and extension seams

The first boundary does not support batched or ragged state, prefix sharing, speculative rollback, growing capacity, windowed/in-place append, partial publication, cross-device transfer, multi-stream state use, recurrent or convolutional state, or per-layer cursor drift. None is reserved by an enum catch-all. Each requires a separately accepted state or execution contract.

The fixed dense shape `[8, capacity, 128]`, F32 storage, and one-state-per-cached-tensor profile are the first supported specialization, not a claim that all retained runtime state has this form. A later generic state family may reuse `LiveStateScope`, generation, poisoning, and publication receipts only after proving those invariants fit its semantics.

## Acceptance inventory

Tom's acceptance is required for this exact consequential public draft:

- `LiveStateScope`, an opaque adapter-authenticated runtime-scoped device/context identity with no platform handle, and non-Clone `LiveStateScopeFactory`/`CurrentLiveStateScope` as its only construction and current-route authority capabilities;
- `RuntimeAdapter::bind_live_state_scope`, which receives that factory only after a live context is bound and returns the lifetime-bound current authority directly to the generic route, plus the exact `LiveStateScopeBuildError` inventory;
- opaque `StateInterfaceKey::from_decoded_program` over semantic-graph identity, the complete ordered logical interface, and input ordinal, plus exhaustive `StateInterfaceKeyError`, `LayerOrdinal`, `StateGeneration`, `StateCursor`, `StateCapacity`, `KvStateIdentity`, and opaque `KvExecutionIdentity` in the state surface;
- `KvStateStatus::{Ready, Poisoned { failed_execution }}` and opaque `KvExecutionIdentity` minted only by commit from the complete snapshots, step, routed dispatch, and diagnostic ordinal;
- singular read-only `KvState<Storage, Retention>` members, plus `KvStateSet` as their only construction/mutation owner, bounded at 4,096 and rejecting empty, duplicate, missing, extra, unordered, mixed-scope, and mixed-cursor sets;
- non-Clone `KvRouteSession` holding one adapter borrow, one exact `LiveExecutionContext`, and its current scope from binding through commit; non-Clone `PreparedKvStateSet`; artifact-owned complete `DecodedStateInterface`; validated exact-bijection `KvArtifactStateBindingSet` with no caller population; and `PreparedKvRoute::commit` producing `BoundKvTransaction` with its set guard, context, adapter, identity, and `RoutedDispatch` already attached;
- adapter associated `StateStorage`/`StateRetention`, governed `StateObservationFactory`/storage/member/submission observation records, `observe_state_storage`, `execute_state_transaction`, non-Clone transaction-bound `StateTransactionReporter`, runtime-compared complete replacement/evidence reporting, opaque transaction-minted `StateTransactionReport`, `ExactTerminalSuccess`, and `KvPostCommitFailure`, and exhaustive `KvFailureStage` including coherence and validation readback;
- exact `KvStateBuildError`, `KvStateSetBuildError`, `KvStateRefusal`, `KvStateSetRefusal`, artifact-binding, route-preflight, and invalid-report inventories above, including same-range stale-storage detection, without wildcard classes;
- fixed-capacity, out-of-place publication that validates all identity-bound replacements before one infallible publish-all-or-none allocation/cursor/generation replacement;
- poisoning of every member after every post-commit non-success or unfinished drop, and cursor advancement only on the exact transaction-minted terminal-success receipt;
- runtime-instance ownership with adapter-owned storage/device objects and consumer-owned state-handle lifetime; and
- removal of adapter-only ad hoc device scoping as an alternative.

Acceptance would approve the boundary for implementation; it would not claim the types already exist, accept any backend-specific storage API, or authorize the unsupported cases above.

### Rollback and removal consequences

Nothing is released by this draft. If Tom rejects or materially changes it before implementation, rollback removes this proposed contract, its glossary/navigation/catalog links, and L5's D-15 disposition, then restores D-15 as unresolved; no artifact schema, identity domain, generated catalog, crate dependency, Rust call site, or compatibility path moves. The dependent artifact/runtime binding ticket remains blocked until a replacement public boundary is accepted.

After acceptance and implementation, the superseded adapter-only ad hoc scope paths are removed rather than retained as compatibility shims. A later semantic replacement must migrate every state constructor, bind, refusal mapping, and completion/publication path together; changing only the token spelling while accepting the old validation authority would not be a replacement.
