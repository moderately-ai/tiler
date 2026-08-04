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

KvState<Storage, Retention>  // private fields, no unchecked constructor
  new(KvStateSpec, CurrentLiveStateScope, Storage, Retention)
    -> Result<KvState, KvStateBuildError>
  identity()
  interface_key()
  layer()
  scope_identity()            // presentation-safe identity, not current authority
  generation()
  capacity()
  cursor()
  status()
  retire(self)                  // consumer ends the handle lifetime; the
                                // runtime instance retains pending device uses
  preflight(&self, expected_interface, layer, CurrentLiveStateScope,
            expected_cursor, T)
    -> Result<PreparedKvStep, KvStateRefusal>

PreparedKvStep               // non-Clone; carries C, T, checked S, expected
                             // generation, storage fingerprint, and scope
  cursor() -> StateCursor
  step() -> u64
  total() -> u64

KvArtifactStateBinding       // non-Clone validated decoded-artifact view naming
                             // cache input, step input, extended output, their
                             // sequence axes, and the retained S = C + T relation
  from_decoded_program(&DecodedProgram, StateInterfaceKey, ...)
    -> Result<KvArtifactStateBinding, KvArtifactBindingError>

prepare_kv_route(PreparedKvStep, KvArtifactStateBinding, &mut DecodedProgram,
                 &mut RuntimeAdapter, expected_artifact, AbiFactBinder)
  -> Result<PreparedKvRoute, KvRoutePreflightError>
                             // binds C/T through AbiFactBinder, proves the decoded
                             // relation yields S, and performs every existing
                             // adapter route stage through plan_dispatch

PreparedKvRoute             // non-Clone; carries PreparedKvStep plus the exact
                             // loader Preflight minted from those bound facts
  commit(&mut KvState, KvExecutionIdentity)
    -> Result<BoundKvStep<'state, 'program>, KvStateRefusal>
                             // revalidates first, then consumes Preflight::commit

BoundKvStep                  // non-Clone; owns the exclusive state borrow after
                             // RoutingCommit plus the RoutedDispatch, and poisons
                             // the state if dropped unfinished
  routed_dispatch()
  succeed(ExactTerminalSuccess, NewStorage, RetentionLease)
  fail(KvPostCommitFailure<AdapterFailure>)

ExactTerminalSuccess         // opaque; exact execution + receipt already checked

KvPostCommitFailure<F> {
  execution: KvExecutionIdentity,
  stage: KvFailureStage,
  cause: F,
}

KvFailureStage = Allocation | StorageBinding | Encoding | Submission
               | Completion | Coherence | ValidationReadback
               | Retention | Publication
```

`StateInterfaceKey` is minted only while holding the whole `DecodedProgram`. Its subject is the decoded envelope's already-validated semantic-graph identity, the complete input/output logical interface in semantic order (stable keys, logical shapes, and resolved logical types), and the selected input's ordinal and stable key. The constructor resolves that ordinal through `DecodedProgram::inputs()` and refuses an absent slot. It is not a second caller-authored grammar, cannot be constructed from arbitrary bytes or a detached `DecodedInput`, and does not include physical components, artifact canonical identity, payloads, variants, delivery position, provenance, or any live fact. Two unrelated programs that both call an input `k_cache` therefore do not share state identity; K and V in one program remain distinct by ordinal and key.

`KvStateIdentity` includes the generation because a binding prepared against generation `g` must not bind after another execution publishes generation `g + 1`. It deliberately excludes the cursor and capacity as separate identity fields: the generation is the version of the immutable published snapshot that contains them. Both values remain checked state metadata, never specialization values or cache-key material.

`LiveStateScope` has no public constructor from caller bytes. A runtime adapter authenticates it only after binding its private live device and execution context; two values compare equal only within the declared runtime-instance scope. A reset, context replacement, provider token change, or runtime-instance replacement mints a different scope.

The external adapter can authenticate the current scope only through `LiveStateScopeFactory`, a non-Clone capability constructed inside the runtime route after `LiveExecutionContext` exists and passed only to `RuntimeAdapter::bind_live_state_scope`. The adapter supplies bounded stable token bytes for its private device and context; the factory binds them to the observed backend family, representation, runtime-instance nonce, and the private current-context lifetime before constructing `CurrentLiveStateScope`. That capability contains the comparable `LiveStateScope` but does not expose a raw-parts constructor. The tokens are identities, not serialized platform handles.

`KvState::scope_identity()` is diagnostic/comparison output only. `KvState::preflight` does **not** accept it or an arbitrary `LiveStateScope`; it consumes the non-Clone `CurrentLiveStateScope` delivered directly by the current stateful route. A caller who reads a state's old identity therefore cannot present it as current authority. The capability cannot outlive the private `LiveExecutionContext` whose route minted it, and the generic route passes it from `bind_live_state_scope` to state preflight without returning it to the consumer.

`KvExecutionIdentity` is not an artifact subject. It exists so every post-commit failure and poisoned-state refusal names the exact attempt whose token was not produced. Its presentation may include a consumer ordinal, but equality is over its opaque runtime identity rather than a diagnostic label.

### Meaning and concrete spelling

The boundary's **meaning** is that one adapter-authenticated runtime-instance scope covers one live device and execution context; authority tied to the currently bound route, not equality supplied by the consumer, is required at every state bind, and a reset or context replacement invalidates it. The runtime interprets no adapter token and exposes no platform object. `StateInterfaceKey` similarly means one exact input slot in one decoded semantic graph and ordered interface, not caller text or a locally stable input key that happens to match. These validation obligations survive any later Rust renaming.

The **concrete spelling proposed for acceptance** is the type and method inventory above: `LiveStateScope`; non-Clone, runtime-minted `LiveStateScopeFactory` and `CurrentLiveStateScope`; `RuntimeAdapter::bind_live_state_scope`; `StateInterfaceKey::from_decoded_program` and `StateInterfaceKeyError`; `KvState::new`; the listed readers; `retire`; current-authority-consuming `preflight`; validated `KvArtifactStateBinding`; `prepare_kv_route`; `PreparedKvRoute::commit` consuming the exact existing loader `Preflight` it carries; `BoundKvStep` carrying its `RoutedDispatch`; `ExactTerminalSuccess`; `KvPostCommitFailure`; and `KvFailureStage`. No public raw-parts constructor, detached-input constructor, mutating cursor setter, generation setter, scope setter, status setter, unchecked state constructor, or public way to join an arbitrary `PreparedKvStep` to an arbitrary `Preflight` exists.

## Construction and representation invariants

A state is created from a current adapter-authenticated scope, ready at cursor `0`, generation `0`, and a positive fixed capacity selected from the consumer's declared workload bound. The constructor consumes `CurrentLiveStateScope`; a consumer cannot stamp storage with a replayed comparable identity. The initial dense profile has F32 storage with physical capacity `[8, capacity, 128]`, row-major and unpacked. The logical program view is `[8, C, 128]`; bytes outside sequence range `[0, C)` are unreachable and meaningless.

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

`KvState::preflight` is completed before artifact routing, allocation, encoding, or other program work. Checks occur in this order so one invocation has a deterministic primary refusal:

1. status is `Ready`;
2. the state interface key agrees;
3. layer ordinal agrees;
4. the route-authenticated `CurrentLiveStateScope` contains the same identity as the state's scope;
5. caller-stated `expected_cursor` equals the state's cursor;
6. checked addition computes `S = C + T` without overflow;
7. the generation can advance without overflow;
8. `S <= capacity`; and
9. the logical storage view agrees with `[0, C)` and the preparation-time storage fingerprint.

The artifact check is a separate, explicit continuation rather than a claim this signature cannot fulfill. `prepare_kv_route` consumes the resulting `PreparedKvStep` and a `KvArtifactStateBinding` derived from that same decoded program. It binds the token's exact `C` and `T` through the supplied `AbiFactBinder`, rejects a pre-existing contradictory fact, evaluates the retained additive relation and requires its output to equal the token's checked `S`, then passes the frozen `AbiFacts` through the existing `DecodedProgram::prepare` and adapter qualification/preparation/plan stages. The returned `PreparedKvRoute` is the only public join between state preparation and loader `Preflight`; neither constituent can be substituted after the join.

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
```

`StateStorageFingerprint` is an opaque, presentation-safe runtime identity over the adapter resource identity and generation, memory domain, physical allocation range, and logical valid range observed during preparation. It is not content identity and never enters artifact or cache identity. `StaleStorage` catches replacement or mutation that preserves the same logical range; `InvalidStorageView` catches a range disagreement.

The decoded-artifact join has its own exhaustive pre-commit errors: an absent or foreign interface slot, a cache/step/output axis mismatch, no retained additive relation connecting those exact slots, a duplicate or contradictory ABI fact, an evaluated total unequal to the prepared `S`, or the existing typed loader/adapter refusal. None is a post-commit failure and all leave the state ready and unchanged.

```text
StateInterfaceKeyError {
  UnknownInputOrdinal { requested, inputs },
}

KvArtifactBindingError {
  ForeignStateInterface,
  UnknownInputOrdinal { requested, inputs },
  UnknownOutputOrdinal { requested, outputs },
  InvalidSequenceAxis { slot, axis, rank },
  MissingAdditiveExtentRelation,
  AmbiguousAdditiveExtentRelation { matches },
}

KvRoutePreflightError<R> {
  State(KvStateRefusal),
  ArtifactBinding(KvArtifactBindingError),
  AbiBinding(AbiBindingError),
  ExtentRelationMismatch { cursor, step, expected_total, observed_total },
  Route(R),
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
```

The adapter may wrap these in its consumer-specific error, but it may not erase the class or turn one into a route miss. `PoisonedState` names the execution that poisoned the state. `ForeignLiveStateScope` names presentation-safe identities for the stored and currently authenticated scopes and never formats a platform handle.

## Update, publication, and failure

The first update is always out of place. The old allocation is bound read-only over `[8, C, 128]`; the new allocation is distinct and has the same fixed physical `[8, capacity, 128]` capacity while the program writes only its declared `[8, S, 128]` logical output. Input and output sharing one allocation is refused before commit by the existing `ForbiddenAlias` rule.

After `RoutingCommit`, every allocation, binding, encoding, submission, completion, retention, and publication failure is terminal for the attempt and poisons the state. There is no fallback and no reuse of the still-intact old bytes: the consumer did not receive the failed step's token, so continuing from the old cursor would execute a different sequence from the one the consumer believes it owns.

Publication requires a `TerminalSuccess` tied to the exact `KvExecutionIdentity` and submission receipt that produced the new storage. The adapter must already have observed terminal success, post-wait status, required coherence, and all validation records. Publication then performs one atomic runtime-instance replacement:

```text
(old allocation, C, generation g, Ready)
  -> (new allocation, C + T, generation g + 1, Ready)
```

The checked cursor addition and generation increment cannot wrap. `GenerationExhausted` is a pre-commit refusal for the next attempt; it is never repaired by resetting the generation inside the same state identity.

Every non-success transition after commit instead performs:

```text
(old allocation, C, generation g, Ready)
  -> (old allocation, C, generation g,
      Poisoned { failed_execution })
```

The cursor therefore advances by exactly `T` once, on exact terminal success, and by zero on every refusal, device error, nonterminal observation, cancellation, retention error, or publication failure. Publication itself cannot partially expose `(new allocation, old cursor)` or `(old allocation, new cursor)`.

`PreparedKvStep` carries the complete preflighted state identity and storage fingerprint but no mutable borrow, and exposes only its checked `C`, `T`, and `S` values to the governed join. `prepare_kv_route` consumes it while producing the loader preflight from those same facts. `PreparedKvRoute::commit` takes a mutable state, revalidates the current identity, generation, cursor, scope, status, and storage fingerprint, and returns any `KvStateRefusal` while its carried loader authority remains uncommitted. Only after every revalidation succeeds does it call the consuming, infallible `Preflight::commit`; `BoundKvStep` owns the resulting `RoutedDispatch` together with the exclusive state borrow. A publication between preparation and commit therefore reaches `StaleGeneration` before `RoutingCommit`, while allocation and dispatch can be reached only through `BoundKvStep::routed_dispatch`. Its success and failure methods consume it. Dropping it through an early return or panic poisons the state with its bound `KvExecutionIdentity`; there is no path that abandons a committed step while restoring `Ready` or recovers fallback from the carried route.

## Placement, aliasing, retention, and destruction

The state storage occupies the one symbolic affinity's admitted memory domain under the initial ADR 0047 profile. It creates no new domain and no transfer edge. The adapter validates that both old and new allocations belong to the bound live scope and the plan's required domain; a second device or context is a `ForeignLiveStateScope` refusal, not a transfer request.

Old and new allocations are distinct. Both, plus every view, pipeline, command object, argument resource, and synchronization object that can reach either, are retained through exact final device use. A successful publication may release the old allocation only after the extending execution's completion receipt proves its last read complete. The new allocation remains retained by the published state and by every later in-flight use. A poisoned state retains whatever resources still have device use until their receipts permit release.

The runtime instance owns the live allocation, metadata, status, and retention leases. The consumer owns the state handle/session lifetime and explicitly destroys it; destruction asks the runtime instance to retire the object and defers resource release until every exact final-use receipt permits it. Dropping a host reference is not completion evidence.

## Ownership by layer

| Layer | Owns | Must not own |
| --- | --- | --- |
| Semantic program | Named K/V state inputs and retained outputs; concatenation meaning; `S = C + T` | Capacity, cursor value, generation, allocation, device/context, poison status |
| Physical plan | Row-major layout, accessible domains, distinct input/output storage, realization and guard choice | Live allocation identity, consumer token position, artifact-independent mutable state |
| Artifact | Ordered named slots; formulas for logical ranges and launch geometry over `C`, `T`, `S`; fixed variants and canonical artifact identity | State identity, scope, capacity, generation, cursor value, allocation, poison status, specialization on any cursor-derived value |
| Runtime instance | `KvState`, fixed capacity, published cursor and generation, status, atomic replacement, retention leases | Platform device API objects in its consumer-neutral types; semantic meaning; sampling policy |
| Runtime adapter | Minting `LiveStateScope`; private device/context and storage objects; domain/alias checks; allocation; submission receipts; terminal observation; exact final-use retention | Portable artifact identity changes; semantic graph changes; silent recovery after commit |
| Consumer | State-handle lifetime, expected execution ordinal, one cursor authority for cache extent/rotary rows/mask, sampling, termination, explicit destruction | Direct cursor mutation, publication before success, reconstructing state identity from labels |

## Negative examples

### Stale state

State generation `g` has cursor `13`, but the consumer tries to bind `expected_cursor = 14` because it retained a handle from another step. Capacity is `18`, so every byte range would fit. Preflight returns `StaleCursor { expected: 14, observed: 13 }` before artifact routing. Supplying an old prepared binding after publication instead returns `StaleGeneration`.

### Capacity overflow

State cursor is `17`, step length is `2`, and capacity is `18`. Checked addition obtains required context `19`; preflight returns `CapacityExceeded { cursor: 17, step: 2, required: 19, capacity: 18 }`. No route is selected and no allocation occurs. `C = u64::MAX, T = 1` returns `CursorOverflow` rather than wrapping.

### Cross-device or cross-context reuse

An adapter presents scope B for a state minted under scope A. Even if both devices advertise the same target profile, backend family, and representation, preflight returns `ForeignLiveStateScope`. A different context on the same device also mints a different scope and refuses. No transfer or implicit rebind is attempted.

Reading state A's `scope_identity()` and passing that value back cannot defeat this refusal: neither `KvState::new` nor `KvState::preflight` accepts it. Both require the non-Clone authority minted inside the currently bound adapter route.

### Same local key in another program

Programs P and Q both name input ordinal 2 `k_cache`, with the same type and shape. Their semantic graph identities or complete ordered interfaces differ. `StateInterfaceKey::from_decoded_program(P, 2)` and `StateInterfaceKey::from_decoded_program(Q, 2)` are unequal, so binding P's state while routing Q returns `StateInterfaceMismatch`; the repeated local `InputKey` cannot alias them.

### Contradictory artifact facts

A prepared state step carries `C = 14`, `T = 1`, and checked `S = 15`. Its decoded binding names the exact cache input, step input, extended output, sequence axes, and retained additive relation. If the supplied ABI binder already says that cache axis is `13`, the join returns its typed duplicate/contradictory binding error. If the decoded relation evaluates to any value other than `15`, it returns `ExtentRelationMismatch`. Neither case can produce a loader `Preflight` or reach routing commit.

### Storage replacement between preparation and commit

A step was prepared over allocation generation A and logical range `[0, 14)`. Before commit, storage generation B is installed with the same range. Range-only checking would accept it; fingerprint revalidation returns `StaleStorage` before consuming the loader preflight.

### Poisoned state

Execution `e5` crossed `RoutingCommit` and then its submission reported a device error. The state remains at its previous cursor and generation but becomes `Poisoned { failed_execution: e5 }`. Every later preflight returns `PoisonedState { failed_execution: e5 }`; the consumer must construct a fresh state from a known prefix.

### Cursor advancement after non-success

Starting from `(C = 14, generation = 5)`, a step with `T = 1` reaches a waiting state whose receipt is nonterminal, errors, or belongs to another execution. None is `TerminalSuccess(e6)`. The only legal result is `(C = 14, generation = 5, Poisoned(e6))`; `(15, 6, Ready)` is constructible only from the exact successful receipt. A pre-commit capacity refusal leaves `(14, 5, Ready)` instead, because no committed execution existed to poison it.

An early return immediately after `PreparedKvRoute::commit` still drops a fully constructed `BoundKvStep`, whose drop poisons the state. There is no public sequence that consumes `Preflight::commit` first and attaches the state guard later.

## Unsupported cases and extension seams

The first boundary does not support batched or ragged state, prefix sharing, speculative rollback, growing capacity, windowed/in-place append, partial publication, cross-device transfer, multi-stream state use, recurrent or convolutional state, or per-layer cursor drift. None is reserved by an enum catch-all. Each requires a separately accepted state or execution contract.

The fixed dense shape `[8, capacity, 128]`, F32 storage, and one-state-per-cached-tensor profile are the first supported specialization, not a claim that all retained runtime state has this form. A later generic state family may reuse `LiveStateScope`, generation, poisoning, and publication receipts only after proving those invariants fit its semantics.

## Acceptance inventory

Tom's acceptance is required for this exact consequential public draft:

- `LiveStateScope`, an opaque adapter-authenticated runtime-scoped device/context identity with no platform handle, and non-Clone `LiveStateScopeFactory`/`CurrentLiveStateScope` as its only construction and current-route authority capabilities;
- `RuntimeAdapter::bind_live_state_scope`, which receives that factory only after a live context is bound and returns the lifetime-bound current authority directly to the generic route, plus the exact `LiveStateScopeBuildError` inventory;
- opaque `StateInterfaceKey::from_decoded_program` over semantic-graph identity, the complete ordered logical interface, and input ordinal, plus exhaustive `StateInterfaceKeyError`, `LayerOrdinal`, `StateGeneration`, `StateCursor`, `StateCapacity`, `KvStateIdentity`, and opaque `KvExecutionIdentity` in the state surface;
- `KvStateStatus::{Ready, Poisoned { failed_execution }}`;
- private-field `KvState<Storage, Retention>`; `KvState::new` consuming current route authority; readers for identity, interface key, layer, presentation-safe scope identity, generation, capacity, cursor, and status; preflight consuming current route authority rather than caller-supplied identity; and consuming `retire`;
- non-Clone `PreparedKvStep` exposing checked `C`, `T`, and `S`; validated `KvArtifactStateBinding`; `prepare_kv_route` as the only join that binds those facts and carries the resulting loader `Preflight`; exhaustive artifact-binding and route-preflight errors; and non-Clone `PreparedKvRoute::commit` producing `BoundKvStep` with its `RoutedDispatch` and poison-on-unfinished-drop guard already attached;
- opaque `ExactTerminalSuccess`, generic `KvPostCommitFailure`, and exhaustive `KvFailureStage`;
- exact `KvStateBuildError` and `KvStateRefusal` inventories above, including same-range stale-storage detection, without wildcard classes;
- fixed-capacity, out-of-place publication with simultaneous allocation/cursor/generation replacement;
- poisoning after every post-commit non-success and cursor advancement only on the exact terminal-success receipt;
- runtime-instance ownership with adapter-owned storage/device objects and consumer-owned state-handle lifetime; and
- removal of adapter-only ad hoc device scoping as an alternative.

Acceptance would approve the boundary for implementation; it would not claim the types already exist, accept any backend-specific storage API, or authorize the unsupported cases above.

### Rollback and removal consequences

Nothing is released by this draft. If Tom rejects or materially changes it before implementation, rollback removes this proposed contract, its glossary/navigation/catalog links, and L5's D-15 disposition, then restores D-15 as unresolved; no artifact schema, identity domain, generated catalog, crate dependency, Rust call site, or compatibility path moves. The dependent artifact/runtime binding ticket remains blocked until a replacement public boundary is accepted.

After acceptance and implementation, the superseded adapter-only ad hoc scope paths are removed rather than retained as compatibility shims. A later semantic replacement must migrate every state constructor, bind, refusal mapping, and completion/publication path together; changing only the token spelling while accepting the old validation authority would not be a replacement.
