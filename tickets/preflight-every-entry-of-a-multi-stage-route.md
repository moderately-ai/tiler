---
id: preflight-every-entry-of-a-multi-stage-route
title: Preflight a pipeline per entry once a route can carry more than one
status: todo
priority: p2
dependencies: []
related: [prototype-metal-runtime-preflight, carry-the-stage-execution-order-in-the-envelope]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, metal, correctness]
---
Split from `prototype-metal-runtime-preflight`, which built the device-side preflight over **one** entry and, in its first outcome, attributed this remainder to a ticket that does not hold it. That attribution is corrected in the same change that files this.

## What is true today

**Fact — the loader routes exactly one entry.** `accept_entry` in `crates/tiler-runtime/src/load.rs` selects one, `Preflight` carries `entry: DecodedEntry<'a>` and `symbol: &'a str` as singular fields, and `device_preflight` therefore builds one library, resolves one symbol, and creates one pipeline. Writing a loop over a collection that cannot hold two would have claimed coverage that does not exist.

**Fact — the refusal that makes this safe is one layer away.** `LoadRejection::UnroutableEntries` exists at `crates/tiler-runtime/src/load.rs:391` and is unreachable in practice because the *decoder* rejects a multi-stage envelope earlier — `tiler.artifact.feature.multi-stage-program` is derived at the projector and absent from `SUPPORTED_FEATURES`. `route-the-runtime-loader-through-the-dispatch-record` recorded that its loader "genuinely cannot sequence a multi-entry variant", and that ticket is `done`.

**Fact — no live ticket owns the runtime half.** `carry-the-stage-execution-order-in-the-envelope` is the live owner of the *envelope* half: its scopes are `contracts/artifacts` and `implementation/artifact`, and its closing condition is about what a consumer holding encoded bytes can sequence. It does not hold `implementation/runtime` and does not mention preflighting a pipeline per entry. Verified by reading both tickets at `a159dc1`. So the runtime half had no owner until this ticket.

## Why it is not merely bookkeeping

`prototype-metal-runtime-preflight` moved every device-decidable obligation before the routing commit, and the property it bought — the commit is infallible in fact and not only in signature — is stated over one entry. A route with two entries whose second pipeline fails to build would reintroduce exactly the defect that ticket removed, unless the preflight loops. The ordering guarantee is therefore only as general as the route, and this ticket is what keeps the two in step.

## Scope

When a route can carry more than one entry, preflight **every** one before the commit: a pipeline per entry, each entry's launch geometry against its own pipeline's capacity, and every binding of every entry. A refusal must still name the entry it came from, because "some pipeline in this route failed" is not actionable.

Depends in substance on `carry-the-stage-execution-order-in-the-envelope`: until the envelope carries execution order, there is no multi-entry route to preflight. Not linked as a hard dependency because the two could land together.

## Closes when

Every entry of a routed variant has its pipeline, its launch geometry, and its bindings discharged before `Preflight::commit`, a refusal names the entry, and `make full` passes.
