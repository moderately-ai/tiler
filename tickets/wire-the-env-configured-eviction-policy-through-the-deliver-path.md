---
id: wire-the-env-configured-eviction-policy-through-the-deliver-path
title: Wire the env-configured eviction policy through the deliver path
status: todo
priority: p2
dependencies: [admit-an-age-bounded-automatic-eviction-into-the-expansion-cache]
related: [decide-the-expansion-cache-collection-schedule]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [cache, eviction, frontend, macro-aot]
---
## User-visible outcome

A consumer's cache trims itself: the delivering expansion path (`tiler_macros::aot::deliver`, the route that opens the cache today) invokes the age-bounded eviction under a policy read from environment variables, off the hot path, with a documented default and a documented override, so nothing is required of a consumer who configures nothing.

## Constraints carried from the decision record

- Environment reading belongs here, beside the ADR 0089 root resolution `tiler-macros` already performs — never in `tiler-cache`. The env vars are parsed into the typed policy the cache-side ticket admits; an unparseable value is a typed refusal of eviction (the build proceeds, the cache simply does not evict, and the refusal is attributable), never a guessed bound and never a build failure.
- Trigger placement is after a successful publish on the deliver route, never on the hit path and never inside `get_or_publish` — the placement the design record's performance refusal permits. State in the contract what happens on the rust-analyzer server (long-lived process, many expansions: the trigger must amortize, not walk every shard per expansion).
- Name the variables deliberately and document them in the frontend contract (`docs/integration/frontends.md`) — a name is a public surface. An explicit opt-out (eviction disabled) must exist and be documented.
- The `CollectionReport` from an automatic eviction: decide explicitly what becomes of it and record why; silent discard is a decision to record, not a default to inherit.

## Closes when

A consumer with no configuration gets the default policy applied off the hot path; each variable is exercised by a test including the refusal path; the frontend contract documents names, default, opt-out, and report disposition; and the accepted inline developer experience is untouched (no prepare step, no consumer build.rs).
