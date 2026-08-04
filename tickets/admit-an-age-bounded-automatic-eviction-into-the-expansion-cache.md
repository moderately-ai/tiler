---
id: admit-an-age-bounded-automatic-eviction-into-the-expansion-cache
title: Admit an age-bounded automatic eviction into the expansion cache
status: review
priority: p2
dependencies: []
related: [decide-the-expansion-cache-collection-schedule, wire-the-env-configured-eviction-policy-through-the-deliver-path, measure-the-expansion-cache-hot-path-efficiency]
scopes: [implementation/cache, research/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, eviction, durability, decision-execution]
claimed_from: todo
assignee: agent-cache-evict
lease_expires_at: 1785875503
---
## User-visible outcome

The expansion cache can evict entries older than a caller-stated age, so a developer machine no longer accumulates entries forever, and the delivering frontend can invoke that eviction automatically under an environment-configured policy without `tiler-cache` ever reading the environment itself.

## The decision this executes, and what it supersedes

Tom decided on 2026-08-04 (recorded in [`decide-the-expansion-cache-collection-schedule`](decide-the-expansion-cache-collection-schedule.md)): no shipped maintenance CLI; automatic age-based eviction of old entries, customizable via environment variables. This explicitly supersedes the "a collection is never automatic" schedule conclusion of the bounded-collection design record (`docs/research/cache/bounded-collection.md`) — the first deliverable is that supersession written into the record, preserving the original rationale (it was correct against the alternatives it considered; the product owner has since weighted background hygiene above per-act attribution).

What the elimination established and this ticket must NOT weaken:

- **The collector's concurrency and crash safety are properties of the collector, not the schedule** (measured at 1/8/32 writer processes; `KeyLock::try_acquire`, re-`stat` before unlink, no journal). Age-based selection must go through the same locked, re-validated removal path.
- **Never on the hot path.** The `get_or_publish`-miss shard walk was refused on performance grounds and no new evidence reopens that. The eviction entry point is a separate call the frontend issues off the hit path (see the successor wiring ticket).
- **`tiler-cache` never reads the environment.** The age bound arrives as an explicit typed value (extend `CollectionBound` or add a sibling age policy type — the exact public shape is a draft for Tom under ADR 0074 §7). Environment-variable names, parsing, and defaults live with the frontend, which already owns environment reading under the ADR 0089 root policy.
- **Explainability survives automation.** `CollectionReport` already names every removal; the automatic caller decides what to do with it, but the mechanism must keep every removal attributable to the stated age policy. Fail closed on an unparseable or contradictory policy: refuse to evict, never guess.

## Implementation keys

- Age is measured from the entry's own filesystem evidence (the modification time the collector already re-`stat`s), not from a durable index — the design record's refusal of a second authority stands.
- A default age must exist for the zero-configuration case (Tom authorized defaults by choosing automation). Choose it explicitly, state its ground in the record (per-entry sizes 32–48 KB and ~10–20 MB per editing afternoon are the measured inputs), and make it overridable; document that the default is a product choice, not a measurement.
- Preserve `CollectionBound::UNBOUNDED` semantics for existing callers; an age bound composes with, not replaces, the size bound.
- Deliberate perturbations: an entry exactly at the age boundary; a clock that moves backwards (mtime in the future — must not panic, must not mass-evict); an eviction racing a publisher re-publishing the same key; a policy stating zero/negative age (typed refusal).

## Closes when

The age-bounded eviction is a tested draft on the public cache boundary (draft for Tom, not self-accepted), the bounded-collection record carries the explicit supersession with rationale preserved, all perturbations pass, and the wiring ticket can consume the typed policy without `tiler-cache` gaining any environment read.

## Implemented 2026-08-04 — a tested draft, moved to `review` for Tom's ruling on the shape

### The supersession, written first

`docs/research/cache/bounded-collection.md` gains a *Superseded — 2026-08-04: the schedule is automatic and age-bounded* section, a one-line pointer above the superseded proposal, and two in-place amendments. The original derivation under *Who runs it, and when* is preserved verbatim — it is the argument a future reader has to defeat to move the weighting back — and the supersession names exactly one moved conclusion (the first clause, "Never automatically") and four that survive: the hot-path refusal, the no-durable-index refusal, the no-default-by-guess rule for the *size* ceilings, and crash/race safety with every accounting rule. It also states the asymmetry the decision turned on: the two discriminators the elimination used (a report terminates in a reader; a bound arrives with its trigger) were product weightings, not correctness findings, and the correctness findings eliminated nothing — so the product owner was the authority over the part that moved.

Two stale assertions in the same record were corrected while there: the status line still claimed the collector was staged crate-private under ADR 0074 convention 7 (the facade was accepted 2026-07-31, which the record's own closing section already said), and the *What this design does not settle* entry still deferred scheduling to a ticket that is now `done`.

### The typed age policy

`CollectionBound` gains a third optional ceiling, `max_entry_age: Option<MaxEntryAge>`, beside the two existing aggregate ceilings.

- **`MaxEntryAge`** is an opaque validated newtype over `Duration` (ADR 0074 convention 6): private field, `new` returning `Result<Self, MaxEntryAgeRefusal>`, `as_duration`. Zero is the one refused value, and the derivation is recorded rather than asserted — `age >= 0` holds for every entry the host can date including one published this instant, so it is "remove everything" said obliquely, with the added failure that it removes an entry a concurrent build is about to hit. A caller that means it has `max_entries: Some(0)` or `purge`. Nothing above zero is refused, because a floor would be exactly the guessed number the record declines to choose. A **negative** age is unrepresentable, not unchecked: `Duration` is unsigned.
- **Composition, not replacement.** The age is a *per-entry* predicate; the byte and entry ceilings are properties of a *total*. Selection runs the age pass first and the aggregate pass over what it left, so an entry is never removed to fit a ceiling that an expiry was about to satisfy. Each ceiling only ever adds removals, so the composition **cannot express a contradiction** — that is why no contradiction check exists, and the absence is derived rather than overlooked.
- **`UNBOUNDED` semantics are byte-for-byte preserved.** `CollectionBound::UNBOUNDED` and `Default` both name all three ceilings absent and still remove nothing; `is_unbounded` now includes the age.
- **Age from the entry's own re-`stat`'d evidence.** Selection reads the `published` modification time the scan already captured, and removal goes through the unchanged `remove_if_unchanged`: the key lock via `try_acquire` (contended → skipped, never waited for), then a re-`stat` that must still agree with the scan on **both length and that same modification time** before the unlink. The age decision and the removal precondition are therefore the same field, so an entry republished after being age-selected is `Superseded` and survives.
- **`collect_at(bound, now)`** is a crate-private seam; the public `collect` pins `now` to `SystemTime::now()`. One clock reading per collection, so an entry's fate never depends on where the walk reached it — and an age boundary becomes a deterministic statement instead of a wall-clock margin.
- **Attribution survives automation.** `RemovedEntry` gains `reason: RemovalReason` (`OlderThanMaxEntryAge` | `OverSizeCeiling`, `#[non_exhaustive]` under convention 5a). Under an automatic eviction nobody is present to remember the policy, and "the cache was over a ceiling" and "this entry was older than the configured age" lead to different corrections. `CollectionOutcome::BoundNotReached` now also fires when an expired entry was selected and not actually removed, because the byte and entry figures it carries are aggregates and cannot say so.
- **No environment read anywhere in `tiler-cache`.** Verified: `grep -rn 'std::env\|env::' crates/tiler-cache/src/` reports matches only inside `#[cfg(test)]` modules (`harness.rs`, `hot_path.rs`, `fault.rs`, `tests.rs`), all pre-existing; `collect.rs`, `store.rs`, `expansion.rs`, and `lib.rs` have none.
- **Off the hot path, structurally.** `grep -rn '\.collect_at(\|\.collect(&' crates/tiler-cache/src/` shows every call site is a test or the test harness. Nothing in `resolve`, `lookup`, `read_entry`, or the publication path reaches a collection.

### The documented default

`MaxEntryAge::DEFAULT` is **thirty days**, spelled `Duration::from_hours(30 * 24)` because `Duration::from_days` is still unstable on the pinned toolchain and a constant does not justify a feature gate. It is a **product choice under Tom's decision, not a measurement**, and the doc comment says so and states its ground so it can be argued with: the collector's own asymmetry (a wrong eviction costs one recompilation; over-long retention costs disk the measured rate bounds) points at the longer end; the measured 32,136–47,803 bytes per envelope and ten-to-twenty megabytes per editing afternoon put the 30-day steady state at roughly 200–400 MB over twenty working days; growth is driven by re-keying rather than editing, since an Apple toolchain update orphans every prior entry at once, and 30 days reclaims each orphaned generation within a month; and a shorter window hands a developer returning from an ordinary absence a completely cold first build. **Nothing in this crate applies it** — it is not `CollectionBound`'s `Default`, `MaxEntryAge` implements no `Default`, and no operation reaches for it; a frontend cites it. The trigger that would replace it with a derived number is a working-set-lifetime measurement, which does not exist today (what exists is per-entry size).

### Every perturbation, as a test

All in `expansion::tests`:

- `an_entry_exactly_at_the_age_boundary_is_removed_and_one_inside_it_is_not` — the boundary is *reached*, not passed. Both entries are dated from one anchor instant and the collection is run at it through `collect_at`, because a wall-clock `now` is necessarily later than the moment a test set the modification time and can therefore never observe equality; a margin would replace the statement with a race.
- `an_entry_dated_in_the_future_is_neither_removed_nor_a_reason_to_remove_others` — no panic, no mass eviction. A future-dated entry has an *unknown* age, not an infinite one, so it is retained, and its presence does not make the rest of the cache collectable. The second half is what makes it a perturbation: a selector computing a cutoff instant, or treating an unrepresentable age as `Duration::MAX`, would empty the cache the moment one clock disagreed.
- `an_age_eviction_racing_a_republisher_removes_nothing_it_did_not_measure` — both observable positions, asserted rather than raced. With the key lock held by the re-publisher the entry is `contended` and the age ceiling reports `BoundNotReached` (the case a caller reading only `bytes` and `entries` would misread as success, since no aggregate ceiling was stated); after the replacement lands, `remove_if_unchanged` on the already-selected stale fact reports `Superseded` and the fresh entry still validates; and a fresh collection under the same policy leaves the replacement alone.
- `a_zero_maximum_entry_age_is_refused_and_no_bound_can_carry_one` — the typed refusal, its `Display` reason, and the deliberate absence of a floor above it (one nanosecond is accepted).
- `the_default_bound_states_no_age_and_the_default_age_is_only_a_constant` — an entry over a year old survives `UNBOUNDED`, `Default == UNBOUNDED`, and `DEFAULT` is thirty days.
- `an_age_ceiling_removes_only_entries_older_than_the_stated_maximum` and `an_age_ceiling_composes_with_an_entry_ceiling` — the plain case and the union, the latter asserting the exact `(key, reason)` sequence so an age pass running *after* the aggregate pass (which would remove one entry too many) fails.

**Both mechanism perturbations were run and watched fail, then restored.** `has_expired`'s `age >= self.0` → `age > self.0` failed only `an_entry_exactly_at_the_age_boundary_is_removed_and_one_inside_it_is_not` (`left: [], right: [CacheKey(…cf0bd610)]`). Replacing `.and_then(|p| now.duration_since(p).ok())` with `.map(|p| now.duration_since(p).unwrap_or(Duration::MAX))` failed only `an_entry_dated_in_the_future_is_neither_removed_nor_a_reason_to_remove_others` (two keys removed where one was expected). Both restored; 112 tests pass.

### What is Tom's, and is not self-accepted

The exact public shape is a **draft** on the accepted maintenance facade, marked as such in `collect.rs`'s *Draft boundary* section, in `expansion.rs`'s module documentation, and in the research record. The atomic question for Tom is drafted in the worker's report. Nothing else on the facade changed.

### Found while implementing, not fixed here

- `RemovedEntry` is a public output record that gained a field and carries no `#[non_exhaustive]`, which ADR 0074 convention 5a would ask for on a record documented as growing. It was accepted without the attribute on 2026-07-31, so adding it is a change to an accepted shape rather than a defect this ticket may fix unilaterally; it is raised in the question to Tom instead.
- `Duration::from_days` is unstable on the pinned toolchain (`duration_constructors`, issue #120301) while `Duration::from_hours` is stable and already used by `limits.rs`. No feature gate was introduced.
- Clippy's `duration_suboptimal_units` fires on `Duration::from_secs(900)` and `from_secs(86_400)` but not on `from_secs(300)` or `from_secs(600)`; the two flagged sites were rewritten as `from_mins`/`from_hours`.
