---
id: preflight-every-entry-of-a-multi-stage-route
title: Make an entire multi-stage route ready before routing commit
status: done
priority: p2
dependencies: [carry-the-stage-execution-order-in-the-envelope, make-runtime-routing-commit-authority-one-shot]
related: [prototype-metal-runtime-preflight, carry-the-stage-execution-order-in-the-envelope]
scopes: [implementation/runtime, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, metal, correctness]
---
Split from `prototype-metal-runtime-preflight`, which built the device-side preflight over **one** entry and, in its first outcome, attributed this remainder to a ticket that does not hold it. That attribution is corrected in the same change that files this.

## What is true today

**Fact — the loader routes exactly one entry.** `accept_entry` in `crates/tiler-runtime/src/load.rs` selects one, `Preflight` carries `entry: DecodedEntry<'a>` and `symbol: &'a str` as singular fields, and `device_preflight` therefore builds one library, resolves one symbol, and creates one pipeline. Writing a loop over a collection that cannot hold two would have claimed coverage that does not exist.

**Fact — the envelope side has landed.** The decoded record now carries stage
execution order and dependencies. The runtime still accepts and preflights one
entry, so it cannot yet make the complete route ready.

**Fact — no live ticket owns the runtime half.** `carry-the-stage-execution-order-in-the-envelope` is the live owner of the *envelope* half: its scopes are `contracts/artifacts` and `implementation/artifact`, and its closing condition is about what a consumer holding encoded bytes can sequence. It does not hold `implementation/runtime` and does not mention preflighting a pipeline per entry. Verified by reading both tickets at `a159dc1`. So the runtime half had no owner until this ticket.

## Why it is not merely bookkeeping

`prototype-metal-runtime-preflight` moved every device-decidable obligation before the routing commit, and the property it bought — the commit is infallible in fact and not only in signature — is stated over one entry. A route with two entries whose second pipeline fails to build would reintroduce exactly the defect that ticket removed, unless the preflight loops. The ordering guarantee is therefore only as general as the route, and this ticket is what keeps the two in step.

## Scope

When a route can carry more than one entry, preflight **every** one before the commit: a pipeline per entry, each entry's launch geometry against its own pipeline's capacity, and every binding of every entry. A refusal must still name the entry it came from, because "some pipeline in this route failed" is not actionable.

Preflight every entry in execution order, including pipeline creation,
entry-specific launch limits, bindings, and dependency obligations. Commit
must consume the route-level one-shot authority owned by
`make-runtime-routing-commit-authority-one-shot`, not merely one of several
repeatable preflight values.

## Closes when

Every entry of a routed variant has its pipeline, launch geometry, bindings,
and dependency obligations discharged before one route-level commit; a refusal
names the entry; fallback authority cannot be recovered after commit; and
`make full` passes.

## Outcome

Done. The loader routes every entry of a multi-stage variant and derives the storage those entries share, both before the routing commit.

**What changed.** `accept_entry` became `accept_entries`, returning the variant's entries in `DecodedVariant::execution_order` rather than in the entry table's canonical stage-key order — dispatching in table order would have been treating a sort key as a schedule. `Preflight` and `RoutedDispatch` now hold a `Vec<RoutedEntry>` plus a `Vec<SharedAllocation>` instead of singular `object`/`entry`/`symbol`/`launch`/`bindings` fields. The four per-entry obligations that previously ran once — backend/representation, payload compatibility, execution policy, object-carried — moved inside the per-entry loop, joining `evaluate_launch` and `place_bindings`. Validating one entry's payload and executing another's was the specific defect that shape allowed. Every `PreflightRefusal` and every relevant `AbiSubject` now carries the entry it came from.

**The pairing, which is the part that would have failed open.** For each `Data` edge, `shared_allocations` pairs the predecessor's sole internal *write* binding with the successor's sole internal *read* binding, found by search and asserted unique — not hard-coded to slots 1 and 0, which is what today's two-buffer kernel profile happens to make them. A route whose ends are not determined is refused with `LoadRejection::UnpairableSharedAllocation` before the commit, where fallback is still permitted. Without it a loader allocates per binding, the successor reads uninitialised device memory, and the result is plausible garbage rather than an error — the one place in this stack that would not fail closed.

**Evidence.** `a_multi_stage_route_preflights_every_entry_and_pairs_its_shared_storage` builds from the materialized alternative (2 stages, 1 `Data` edge), asserts both entries route in execution order, and asserts the single derived pairing has an internal *write* at the producing end and an internal *read* at the consuming one — not merely that two entries appeared, which would pass with the data flow silently broken.

**Measurement boundary, stated rather than implied.** Neutering the pairing to never resolve makes that test fail with `UnpairableSharedAllocation`, so the refusal is reachable. Neutering only its *uniqueness* — take the first internal write instead of requiring one — leaves the suite green, because every kernel this profile verifies destructures to `[read_buffer, write_buffer]` and an entry with two internal writes is not constructible through the builder. The uniqueness half is therefore an argued guard, not a tested one; that is recorded at `sole_internal_slot` so it is not mistaken for coverage.

**Also in this change.** `LoadRejection::UnroutableEntries` was removed rather than kept: it lost its only construction site when the cardinality limit was lifted, and a never-constructed variant advertises a refusal the loader does not make. Two module docs that still described the one-entry limit were corrected — one of them the load-bearing "cannot be sequenced from an artifact alone" claim.

**Deliberately not here.** The hardware run still dispatches one entry, because `serial-sum-compile` packages `compilation.selected()` — the fused plan. Executing the materialized plan on device is `prototype-metal-runtime-proof`'s own stated requirement ("execute the retained materialized program in one explicit proof run, then execute the normally selected fused program in a separate proof run"), it needs a producer change plus a runner that does not assume which alternative was packaged, and that ticket depends on this one. Closing here is what releases it.

`carry-the-data-flow-of-a-stage-dependency` was closed as obsolete in the same change, with the derivation that made it unnecessary; its one dependent was unlinked first so the close does not strand it.

Gate: `make full` green (962 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck).
