---
id: preflight-every-entry-of-a-multi-stage-route
title: Make an entire multi-stage route ready before routing commit
status: todo
priority: p2
dependencies: [carry-the-stage-execution-order-in-the-envelope, make-runtime-routing-commit-authority-one-shot]
related: [prototype-metal-runtime-preflight, carry-the-stage-execution-order-in-the-envelope]
scopes: [implementation/runtime]
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
