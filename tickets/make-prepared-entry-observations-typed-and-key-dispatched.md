---
id: make-prepared-entry-observations-typed-and-key-dispatched
title: Make prepared-entry observations typed and key-dispatched
status: in-progress
priority: p1
dependencies: []
related: [decide-the-prepared-subgroup-width-equality-gate, carry-subgroup-width-through-exact-prepared-entry-equality]
scopes: [implementation/runtime, implementation/candle, implementation/conformance, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, preflight, public-boundary, correctness, fail-closed]
claimed_from: todo
assignee: worker-prepared-entry-observations
lease_expires_at: 1786647017
---
## User-visible outcome

A runtime adapter can report that it does not recognize a prepared-entry property, and cannot accidentally answer one property with an unrelated pipeline quantity.

## Fact — 2026-08-11

`RuntimeAdapter::observe_prepared_entry` returns a bare `u64`. The Candle Metal implementation reads neither the request's property key nor its provider identity and returns `maxTotalThreadsPerThreadgroup` for every request. This is harmless only while exactly one query exists; a second legal key creates a false-admission path when the unrelated quantity equals the required value.

## Required delivery

- Replace the bare number with an exhaustive typed observation carrying `Quantity(u64)` or `Unrecognized`; no numeric sentinel, `Option` default, or catch-all success.
- Make every adapter exact-match provider namespace/name/revision and property key before reading a quantity. Unknown ownership returns `Unrecognized` and the loader classifies it separately from an observed mismatch.
- Preserve loader-owned comparison. An adapter reports an observation and never returns its own satisfaction verdict.
- Include property key, provider, entry, required/observed value, and relation in the typed diagnostic where the repository's error-bound permits them.
- Perturb provider, key, result variant, value, and entry independently. Show that the old wrong-property equality coincidence now refuses.
- Rebaseline the public adapter surface deliberately; do not retain a compatibility method that maps unknown properties to a number.

## Closes when

Every prepared-property consumer is exhaustive over the typed observation, an unknown property cannot be confused with a quantity, and the old wrong-property false admission has a failing-then-green regression.
