---
id: correct-the-debug-gated-retention-clause-in-the-frontends-contract
title: Correct the debug-gated retention clause in the frontends contract
status: in-progress
priority: p2
dependencies: []
related: [repair-the-dangling-ticket-link-in-the-frontends-contract, accept-the-retention-read-back-s-caller-visible-boundary]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [retention, contracts, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786169827
---

`docs/integration/frontends.md` tells a reader that retention is conditional on a debug configuration. It is not. Retention is unconditional and caller-independent, so the contract promises a gate that does not exist.

## Facts, verified 2026-08-08 by the coordinator

**Fact.** The stale clause is anchored by the phrase *"Debug configuration may retain canonical MSL and tool diagnostics under the cache entry."*

**Fact.** `crates/tiler-build/src/metal_cache.rs` states `retained: stage_retention(&outputs)` unconditionally, and its own doc comment names the property in terms this clause contradicts: *"**Always stated, never discovered.**"* There is no configuration, environment variable, `cfg`, or profile in that path.

**Fact.** `crates/tiler-metal-aot/src/driver.rs` captures stderr at **two** sites — one in the `!status.success()` arm and one as the `Ok` value — so the output survives on success, which is the case the clause implies is dropped.

## Why this was split out rather than fixed in place

The link repair that found it was scoped to citations and was explicitly told not to change what the contract *states*. That was the right boundary: this is a contract clause, and correcting it changes a promise to a consumer rather than a reference. It needs its own read of what the delivered behaviour actually guarantees.

**Second site, same sentence.** `tickets/retain-canonical-msl-under-a-debug-expansion-cache-entry.md` carries the identical stale sentence and the same dead link. That ticket is `done`, so `check-citations.sh` skips it as terminal and **no check will ever flag it**. Repair it here or state deliberately that a terminal ticket is a historical record left as written — either is defensible, but the choice should be explicit rather than a consequence of the checker's skip rule.

## What closes this

The clause restated to match delivered behaviour, with the caller-visible boundary that is still Tom's cited rather than pre-empted — `accept-the-retention-read-back-s-caller-visible-boundary` is `awaiting-decision`, which is `parked` and **not** terminal, so a citation to it stays live and keeps being checked. Do not describe the note's ungated-versus-gated question as settled; that is the open decision, not this ticket's to answer.

Before closing, grep the contract for other conditional language about retention. A clause that survived because it reads plausibly is likely to have siblings; name the count either way.
