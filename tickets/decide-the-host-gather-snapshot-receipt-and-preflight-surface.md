---
id: decide-the-host-gather-snapshot-receipt-and-preflight-surface
title: Decide the host-gather snapshot, receipt, and preflight surface
status: todo
priority: p1
dependencies: [decide-the-invocation-gather-artifact-obligation-and-old-reader-fence, admit-a-storage-carrier-for-integer-program-inputs]
related: [admit-an-invocation-scoped-gather-index-validation-receipt, accept-the-invocation-scoped-gather-validation-public-surface, validate-device-resident-gather-indices-before-dispatch, admit-a-zero-copy-exclusive-lease-for-validated-gather-indices]
scopes: [contracts/decisions, contracts/integrations, contracts/foundation, research/runtime]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [decision, needs-tom, runtime, frontend, gather, validation, fail-closed, public-boundary]
---
## User-visible outcome

The narrow host-visible U32 lane has one exact labelled-draft runtime and facade surface: preflight validates and copies the exact bound bytes, seals an occurrence- and attempt-bound receipt, substitutes only that immutable snapshot into dispatch, consumes the authority once, and returns typed semantic or evidence refusals before routing commit without trying another route.

## Exact-base Facts — `6e713e12`

- **Fact — current preflight receives no input storage.** `DecodedProgram::preflight` and `DecodedProgram::prepare` in `crates/tiler-runtime/src/load.rs` receive the target environment, expected identity and `AbiFacts`; neither sees program-input payload bytes or an invocation attempt.
- **Fact — current dispatch retains the caller's bytes.** `dispatch_embedded_route` in `crates/tiler/src/route.rs` obtains each `DispatchAdapter::storage` borrow, builds `RegionOperand` over it, and later dispatches through that original binding. No owner can replace a validated index with a receipt-owned immutable snapshot.
- **Fact — the one-way authority is narrower.** `pub struct Preflight` and consuming `Preflight::commit` in `crates/tiler-runtime/src/load/route.rs` make existing routing commit infallible after device/property checks, but carry no semantic input receipts. `RuntimeAdapter`, `DispatchAdapter`, `LoadRejection`, and `BindError` have no gather-validation requests or typed failures.
- **Fact — accepted authority fixes semantics but no spelling.** ADR 0108 fixes host-visible U32, `decide_gather_index`, exact extent, immutable receipt-owned copy, occurrence/type/binding/snapshot/attempt binding, one-way consumption and exclusions. It assigns no attempt/generation owner, receipt identity/domain, public methods, input-storage handoff, error variants or precedence.

## Decision packet

Starting from the exact decoded artifact obligation accepted by the dependency, compare a runtime-owned validation stage before existing preflight; extending the existing staged route with a typed non-`Clone` intermediate; late specialization after validation; and typed deferral. Eliminate any route that re-reads caller storage after validation, lets an adapter assert success, treats a receipt as artifact/cache identity, allows receipt reuse/crossing, falls through selection after a semantic input error, or requires inline/device validation in the first pass.

Specify exactly:

- invocation-attempt/generation ownership and lifetime;
- private receipt fields, opaque identity if any, constructors and borrowed views;
- how artifact occurrence/type/extent/program binding is matched to facade storage;
- U32 byte decoding, element order, alignment/endian rules, exact `decide_gather_index` calls, immutable allocation and multiple-obligation deduplication or non-deduplication;
- the binding/storage representation by which the receipt-owned bytes become the only bytes dispatch can reach across every stage and variant;
- the consuming transition into the existing non-`Clone` preflight/commit authority;
- exact public runtime/facade/adapter signatures and visibility; and
- exhaustive typed errors and precedence for unsupported carrier, missing/duplicate/crossed/stale/consumed evidence, length/type/extent/binding/snapshot/generation mismatch and semantic out-of-range.

State explicitly why out-of-range is terminal semantic input failure before variant/backend fallback and why every unsupported mutable, device-resident/produced, callback, assertion, inline-check and general-indirect case refuses. Apply the Pareto-complete decision gate and obtain Tom's decision. Final public-surface acceptance remains downstream after implementation.

## Closing checks and negative controls

- Perturb occurrence, logical type, extent, source/index/result program binding, snapshot byte content and invocation generation independently. Each unchanged assertion must name the mismatched subject before commit.
- Mutate the original adapter storage after validation and independently attempt mutation of the receipt snapshot. The first must not change dispatched bytes; the second must be impossible through the accepted surface. Prove the exact validated byte sequence is the one supplied to every consuming entry.
- Exercise missing receipt, duplicate receipt, duplicate consumption, reuse on a second attempt, crossed receipts between two otherwise shape-equal occurrences, and one index binding shared by several obligations independently.
- Make one element out of range and pin position/value/extent. Independently enable a second variant, reference path and backend; none may run after that semantic error. Repeat for unsupported carrier and unanswerable storage without converting either into a plan miss.
- Census every receipt/state/error variant from its type and make each exhaustive runtime, facade and display match fail on widening. Pin error precedence before allocation, pipeline preparation and commit.
- Record the exact accepted public signatures/errors, unsupported population, host-memory bound, negative-control failure text, acceptance provenance and landed contract hashes in the implementation and downstream acceptance tickets.

## Non-goals

Implementing the surface, final public acceptance, mutable zero-copy, exclusive leases, device-resident or device-produced validation, callbacks, caller assertions, inline kernel checks, fallback, general indirect validation, or Metal emission.

## Closes when

Tom has accepted one exact host validation, snapshot, receipt, substitution, refusal and one-way consumption surface; every no-check/use-gap and forbidden-fallback control is specified with a reachable failure; and the implementation ticket cites the exact landed record rather than deriving runtime authority itself.
