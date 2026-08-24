---
id: decide-the-host-gather-snapshot-receipt-and-preflight-surface
title: Decide the host-gather snapshot, receipt, and preflight surface
status: todo
priority: p1
dependencies: [decide-the-invocation-gather-artifact-obligation-and-old-reader-fence, admit-a-storage-carrier-for-integer-program-inputs]
related: [admit-an-invocation-scoped-gather-index-validation-receipt, accept-the-invocation-scoped-gather-validation-public-surface, carry-semantic-enforcement-plans-through-program-and-artifact, implement-first-runtime-semantic-value-precondition-enforcement, reconcile-direct-input-conformance-order-with-adr-0033, validate-device-resident-gather-indices-before-dispatch, admit-a-zero-copy-exclusive-lease-for-validated-gather-indices]
scopes: [contracts/decisions, contracts/integrations, contracts/foundation, research/runtime]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [decision, needs-tom, runtime, frontend, gather, validation, fail-closed, public-boundary]
---
## User-visible outcome

The narrow host-visible U32 lane has one exact labelled-draft runtime and facade surface: preflight copies the exact bound bytes once, validates only that owned snapshot, seals an occurrence- and attempt-bound receipt, substitutes only that immutable snapshot into dispatch, consumes the authority once, and returns typed semantic or evidence refusals before routing commit without trying another route.

## Exact-base Facts — `6e713e12`

- **Fact — current preflight receives no input storage.** `DecodedProgram::preflight` and `DecodedProgram::prepare` in `crates/tiler-runtime/src/load.rs` receive the target environment, expected identity and `AbiFacts`; neither sees program-input payload bytes or an invocation attempt.
- **Fact — current dispatch retains the caller's bytes.** `dispatch_embedded_route` in `crates/tiler/src/route.rs` obtains each `DispatchAdapter::storage` borrow, builds `RegionOperand` over it, and later dispatches through that original binding. No owner can replace a validated index with a receipt-owned immutable snapshot.
- **Fact — the one-way authority is narrower.** `pub struct Preflight` and consuming `Preflight::commit` in `crates/tiler-runtime/src/load/route.rs` make existing routing commit infallible after device/property checks, but carry no semantic input receipts. `RuntimeAdapter`, `DispatchAdapter`, `LoadRejection`, and `BindError` have no gather-validation requests or typed failures.
- **Fact — committed dispatch is reusable today.** `RoutedDispatch` in `crates/tiler-runtime/src/load/route.rs` derives `Clone`, while `RuntimeAdapter::allocate_dispatch` and `RuntimeAdapter::dispatch` in `crates/tiler-runtime/src/adapter.rs` both accept `&RoutedDispatch`; the same committed value can therefore be cloned or passed to either method repeatedly. Existing one-way `Preflight::commit` alone cannot make a new validation receipt single-use.
- **Fact — nearby semantic evidence does not already implement this lane.** `ValueConformanceEvidence` and `SemanticPreconditionsDischarged` in `crates/tiler-ir/src/semantic/conformance.rs` bind resolved-value conformance and operation-occurrence preconditions, respectively, but no compiler, artifact, runtime or facade path consumes them for gather bounds. `Unresolved logical value conformance` in `docs/artifact-abi.md` places direct-input conformance after `RoutingCommit` at `EnforcementCommit`, while ADR 0108 requires this gather validation before routing commit; the decision must reconcile the lifecycles rather than silently creating a second authority or moving a commit boundary.
- **Fact — accepted authority fixes semantics but no spelling.** ADR 0108 fixes host-visible U32, `decide_gather_index`, exact extent, immutable receipt-owned copy, occurrence/type/binding/snapshot/attempt binding, one-way consumption and exclusions. It assigns no attempt/generation owner, receipt identity/domain, public methods, input-storage handoff, error variants or precedence.

## Decision packet

Starting from the exact decoded artifact obligation accepted by the dependency, compare a runtime-owned validation stage before existing preflight; extending the existing staged route with a typed non-`Clone` intermediate; late specialization after validation; and typed deferral. Eliminate any route that re-reads caller storage after validation, lets an adapter assert success, treats a receipt as artifact/cache identity, allows receipt reuse/crossing, falls through selection after a semantic input error, or requires inline/device validation in the first pass.

Specify exactly:

- invocation-attempt/generation ownership and lifetime;
- private receipt fields, opaque identity if any, constructors and borrowed views;
- how artifact occurrence/type/extent/program binding is matched to facade storage;
- U32 byte decoding, element order, alignment/endian rules and an atomic same-read operation that copies caller storage exactly once into owned bytes and validates only those owned bytes with `decide_gather_index`; validation must never scan the caller and then copy it or read the caller again;
- exact deduplication keys and checked resource limits before any allocation, copy or scan: per obligation, per unique snapshot and whole invocation attempt element counts, byte counts, predicate-evaluation counts and peak owned allocation, including checked-multiply/add overflow and limits on the obligation and dedup populations themselves;
- the binding/storage representation by which the receipt-owned bytes become the only bytes dispatch can reach across every stage and variant;
- the total precedence from facade binding and artifact decoding through conditional-route selection, terminal semantic validation, pipeline/device preflight, one-way commit, allocation and dispatch, including the exact relation to ADR `RoutingCommit`/`EnforcementCommit` and the existing direct-input conformance contract;
- whether this occurrence/access-bounds authority composes with or remains distinct from `ValueConformanceEvidence` and residual `SemanticPrecondition` discharge, with one owner for each fact and no duplicate witness, validator, or error precedence;
- the consuming transition into the existing non-`Clone` preflight/commit authority, and the replacement or restriction required for current `Clone` `RoutedDispatch` plus repeated `allocate_dispatch(&RoutedDispatch)` / `dispatch(&RoutedDispatch)` calls so one receipt cannot authorize two allocations, submissions or attempts;
- exact public runtime/facade/adapter signatures and visibility; and
- exhaustive typed errors and precedence for unsupported carrier, missing/duplicate/crossed/stale/consumed evidence, length/type/extent/binding/snapshot/generation mismatch and semantic out-of-range.

State explicitly why out-of-range is terminal semantic input failure before variant/backend fallback and why every unsupported mutable, device-resident/produced, callback, assertion, inline-check and general-indirect case refuses. Apply the Pareto-complete decision gate and obtain Tom's decision. Final public-surface acceptance remains downstream after implementation.

## Closing checks and negative controls

- Perturb occurrence, logical type, extent, source/index/result program binding, snapshot byte content and invocation generation independently. Each unchanged assertion must name the mismatched subject before commit.
- Mutate the original adapter storage after validation and independently attempt mutation of the receipt snapshot. The first must not change dispatched bytes; the second must be impossible through the accepted surface. Prove the exact validated byte sequence is the one supplied to every consuming entry.
- Exercise missing receipt, duplicate receipt, duplicate consumption, reuse on a second attempt, crossed receipts between two otherwise shape-equal occurrences, and one index binding shared by several obligations independently.
- Independently call allocation twice, dispatch twice, dispatch without the matching allocation, clone every still-cloneable routed authority, and replay after terminal success or failure. Each unchanged control must demonstrate that the receipt authorizes at most one allocation/dispatch attempt and that repeated calls cannot re-read, rebind or resubmit its snapshot.
- Exercise checked overflow and every per-obligation, per-unique-snapshot and whole-attempt element/byte/evaluation/peak-allocation/dedup limit one at a time; each must refuse before allocation, copy or scan, and sharing one binding across obligations must copy one snapshot while applying only the explicitly deduplicated validation work.
- Make one element out of range and pin position/value/extent. Independently enable a second variant, reference path and backend; none may run after that semantic error. Repeat for unsupported carrier and unanswerable storage without converting either into a plan miss.
- Census every receipt/state/error variant from its type and make each exhaustive runtime, facade and display match fail on widening. Pin the complete facade/artifact/selection/validation/preflight/commit/allocation/dispatch precedence, including its reconciliation with direct `ValueConformanceEvidence`, residual semantic preconditions and `EnforcementCommit`.
- Record the exact accepted public signatures/errors, unsupported population, host-memory bound, negative-control failure text, acceptance provenance and landed contract hashes in the implementation and downstream acceptance tickets.

## Non-goals

Implementing the surface, final public acceptance, mutable zero-copy, exclusive leases, device-resident or device-produced validation, callbacks, caller assertions, inline kernel checks, fallback, general indirect validation, or Metal emission.

## Closes when

Tom has accepted one exact host validation, snapshot, receipt, substitution, refusal and one-way consumption surface; every no-check/use-gap and forbidden-fallback control is specified with a reachable failure; and the implementation ticket cites the exact landed record rather than deriving runtime authority itself.
