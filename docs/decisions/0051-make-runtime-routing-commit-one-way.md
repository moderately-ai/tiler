---
schema: "tiler-doc/v1"
id: "ADR-0051"
kind: "decision"
title: "Make runtime routing commit one-way before program work"
topics: ["runtime", "fallback", "routing", "errors"]
catalog_group: "runtime-integration-placement"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.artifact-abi", "tiler.contract.candle-integration"]
evidence: ["tiler.research.runtime.execution-contract", "tiler.research.runtime.candle-post-wait", "tiler.research.runtime.semantic-validation"]
ticket: "runtime-execution-contract"
---

# 0051: Make runtime routing commit one-way before program work

**Status:** accepted

## Context

Fallback is safe only while the consumer still owns an unexecuted semantic
operation. Pipeline preparation may fail before device work, but allocation,
partial encoding, submission, validation, and publication can have observable
resource or execution effects. Retrying an ordinary fallback after those stages
can duplicate work, hide device errors, or publish inconsistent results.

## Decision

Runtime preflight binds inputs and the live device, evaluates guards, prepares
every entry of one complete variant, validates launch/resource capabilities,
and then consumes `FallbackAuthority` at `RoutingCommit`.

Only the resulting committed execution authority may allocate program resources
or encode work. After commit, every allocation, encoding, submission,
completion, validation, and publication failure is a typed terminal execution
error. The launcher cannot recover fallback or silently route another variant.

A synchronous validation record is interpreted only after the exact submission
that produced and synchronized it reaches terminal success. A post-wait error
propagates and never becomes a validation miss or fallback condition.

## Consequences

- Typed applicability/capability misses may try another complete equivalent
  route only during precommit preparation.
- Corrupt artifacts, ABI inconsistencies, systemic preparation failures, stale
  prepared selections, and dishonest providers fail closed.
- Program allocations and partial encodings never precede a fallback decision.
- Runtime adapters must retain all resources through their exact final device
  use and expose trustworthy completion/error observation.
- Consumer integrations unable to preserve this ownership boundary do not
  implement the initial runtime profile.

## Alternatives considered

Fallback after an arbitrary runtime error is ergonomically tempting but cannot
distinguish no-work failures from partial effects. A Boolean `can_fallback`
flag is weaker than a consumed ownership token. Preloading only the library is
insufficient because function lookup and pipeline creation are separately
fallible.

## Implementation boundary

Added 2026-08-01 by [`re-audit-adr-implementation-status-after-the-runtime-and-metal-landings`](../../tickets/re-audit-adr-implementation-status-after-the-runtime-and-metal-landings.md), which moved `implementation_status` from `not-started` to `partial`. This section states which clauses that value rests on and which it does not, read at `2aa0824`. It is a status record and adds no decision.

Amended 2026-08-01 by [`reconcile-the-pre-commit-allocation-seam-with-adr-0051`](../../tickets/reconcile-the-pre-commit-allocation-seam-with-adr-0051.md), read at `29a9680`, which closed the single `Divergent` entry by moving the code rather than the decision. `implementation_status` stays `partial`: the three `Unrealized` entries below are untouched, and one clause becoming true does not make the rest so.

**Realized — the commit is one-way by construction rather than by rule.** `Preflight` at `crates/tiler-runtime/src/load/route.rs:579` is declared neither `Clone` nor `Copy`, because "a route that could be duplicated could be committed twice, and 'committed once' is the property ADR 0051 asks for", and `Preflight::commit` at `:740` consumes `self` and is infallible — there is no `Result`, so an obligation discharged in the wrong stage would be a compile-time absence rather than a runtime surprise. The module header at `:7` states the invariant, and six compile-fail doc-tests pin it: committing twice (`:668`, `E0382`), keeping a spare by cloning (`:683`, `E0277`), minting a second authority from a still-borrowed program (`:707`, `E0499`), duplicating the program to escape that borrow (`:726`, `E0277`), and committing from either unresolved device stage (`:379`, `:496`). A caller takes the fallback the decision permits by dropping the `Preflight` rather than by calling anything.

**Realized — one authority per attempt, not merely one use per authority.** `DecodedProgram` is not `Clone` and `DecodedProgram::preflight` takes `&mut self` (`crates/tiler-runtime/src/load.rs:399`), and the committed `RoutedDispatch` carries that exclusive borrow forward, so a caller cannot hold a committed route and an uncommitted authority for the same attempt. Preflighting again after abandoning stays legal, which is the fallback the decision preserves.

**Realized — preflight discharges every decidable obligation before the commit, in phase order.** `crates/tiler-runtime/src/adapter.rs:496`'s `route_with_adapter` sequences the ten stages the decision names: bind the live context, compare program identity and select the variant its guards admit, validate each carried payload from its bytes, resolve every live-device requirement, prepare every entry of the one selected variant, resolve every prepared-entry property, size the dispatch, commit, allocate, dispatch. Each device stage is a distinct consuming type (`LiveDeviceQualification`, `RoutePreparation`, `Preflight`), so a route with an empty requirement list still passes through the stage rather than skipping it.

**Realized — after the commit, failure is typed, terminal, and forecloses fallback.** The adapter seam declares two error types for the two sides of the boundary: `RuntimeAdapter::Refusal` at `crates/tiler-runtime/src/adapter.rs:261` for every pre-commit outcome and `RuntimeAdapter::Failure` at `:267` for the two committed stages. `AdapterRouteFailure::fallback_permitted` at `:615` is an exhaustive match with no wildcard arm, answering `false` for `Allocation` and `Dispatch` and for nothing else, so a stage added to the route must be classified deliberately instead of defaulting to recoverable. The split follows the associated types exactly: a variant carrying the refusal type is pre-commit and a variant carrying the failure type is not.

**Realized — the fail-closed classes the Consequences name.** `LoadRejection` (`crates/tiler-runtime/src/load.rs:1211`) refuses corrupt bytes with the codec's own classification, an artifact that is not the expected one, a host that can execute no packaged variant, an artifact whose own guards exclude the bound facts, an execution policy this build cannot deliver, an unowned or misanswered or unsatisfied route requirement, and a data dependency whose shared storage is not determined — each before anything is prepared or committed.

**Unrealized — the synchronous validation record and its post-wait interpretation.** Nothing in `crates/` carries a validation record, a submission receipt, an `EnforcementCommit`, or a terminal-status observation. `RuntimeAdapter::dispatch` states the obligation in its documentation and the implementing crate defines none of it, so the decision's third paragraph is an adapter contract rather than implemented support.

**Unrealized — revalidation of volatile facts at the commit.** The adopted research record's `PreparedSelection` revalidates exact fingerprints before consuming fallback authority and reports `StalePreparedSelection`. `Preflight::commit` re-checks nothing, and `LoadRejection` has no stale-selection class, so the state is unreachable rather than refused.

**Unrealized — resource retention through final device use.** The Consequence binds runtime adapters, and no adapter lives in `crates/`; every implementation is under `prototypes/` or `spikes/`.

**Realized — allocation moved to the committed side of the seam, on directed work rather than by superseding this decision.** The divergence this section recorded on 2026-08-01 — `RuntimeAdapter::plan_dispatch` taking a `&Preflight`, allocating program storage, and refusing recoverably — was resolved *toward* this decision. Tom chose that direction at the live session on 2026-08-01 (relayed by the coordinator who witnessed it, and recorded in full on [`reconcile-the-pre-commit-allocation-seam-with-adr-0051`](../../tickets/reconcile-the-pre-commit-allocation-seam-with-adr-0051.md)), and no clause here was superseded: the seam moved to match the text.

`RuntimeAdapter::plan_dispatch` is now a pre-commit sizing and capacity stage that acquires nothing — byte ranges derived from the route, compared against the limits the bound context declares, with each slot's storage *decided* rather than taken — and `RuntimeAdapter::allocate_dispatch` is a second stage reached only from a `RoutedDispatch`, the type `Preflight::commit` is the only source of. That is the Decision's "only the resulting committed execution authority may allocate program resources" carried by an argument type instead of by a rule: an adapter cannot be handed the authority to allocate program storage before the route commits. Its failure is `Self::Failure`, and `AdapterRouteFailure::fallback_permitted` answers `false` for the new `Allocation` variant, so the Consequence "program allocations and partial encodings never precede a fallback decision" now holds of the code. `spikes/runtime/inline-dispatch/src/adapter.rs` and `prototypes/candle-metal-adapter/src/adapter.rs` were both split the same way, and their observed-length assertions moved with the allocations they inspect.

**The priced cost, accepted explicitly rather than discovered.** A device that cannot hold the plan is terminal at the allocating stage instead of recoverable at the sizing one. Pre-commit sizing against declared limits — a single-buffer bound, a caller's supplied operand length, an addressable range — catches every case an adapter can state without storage; what is left is the allocator's own residue, and this decision's position is that such a residue failing loudly is a defect signal rather than a routing input. What the pre-commit stage may still refuse is unchanged in kind: a range that does not fit one allocation, a launch wider than the prepared pipeline admits, an input the caller did not supply, a target the consumer does not place.

## Traceability

Applies to the [artifact execution boundary](../artifact-abi.md) and
[Candle adapter](../integration/candle.md). The
[runtime execution](../research/runtime/runtime-execution-contract.md),
[semantic validation](../research/runtime/semantic-validation-enforcement.md),
and [Candle post-wait](../research/runtime/candle-metal-post-wait-error-checking.md)
reports define the accepted state machine and failure boundary.
