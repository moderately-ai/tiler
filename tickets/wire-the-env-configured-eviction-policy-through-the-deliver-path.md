---
id: wire-the-env-configured-eviction-policy-through-the-deliver-path
title: Wire the env-configured eviction policy through the deliver path
status: review
priority: p2
dependencies: [admit-an-age-bounded-automatic-eviction-into-the-expansion-cache]
related: [decide-the-expansion-cache-collection-schedule]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [cache, eviction, frontend, macro-aot]
claimed_from: todo
assignee: agent-evict-wiring
lease_expires_at: 1785877012
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

## Implemented 2026-08-04 — the deliver path evicts, and every choice has its ground

### The variable, and why it is one

`TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`, read in `crates/tiler-macros/src/eviction.rs` beside the ADR 0089 root resolution, never in `tiler-cache`.

**Spelling.** It follows the surface `cache_root.rs` already established: `TILER_` + the exact subject + the exact property. ADR 0089 rejected `TILER_CACHE_DIR` because "a generic name would silently acquire [later caches] the day they exist, changing what an existing setting means"; the same argument one level down rejects `TILER_EXPANSION_CACHE_MAX_AGE`, because `CollectionBound` carries three ceilings and that spelling is equally the obvious name for a byte or entry ceiling. The cost is length, which ADR 0089 already accepted for this reason.

**One variable, not three.** Only the age ceiling is configurable. The two aggregate ceilings select by *publication* recency, which `CollectionOrder::OldestPublicationFirst` documents as able to evict a hot working set, and Tom's decision was age-based. Deferred with three activation triggers as `configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction` (filed `deferred`, not `todo`, because its body is a deferral).

**Values.** Unset states `MaxEntryAge::DEFAULT` — cited from `tiler-cache`, never restated as a number here, so a frontend copy of the constant cannot go stale. The exact value `off` is the opt-out. Anything else is a whole number and exactly one lowercase unit suffix from `s`, `m`, `h`, `d` (`45s`, `90m`, `12h`, `30d`). No compound form, no sign, no decimal, no separator, no bare count: an unsuffixed `30` is the ambiguity that makes someone delete thirty days of cache believing they wrote thirty days, so it is refused rather than assigned a unit.

**Opt-out spelling.** `off`, and it is `cache_root::DISABLE_VALUE` itself rather than a second constant of the same text — one environment surface, one word for "do not", and a superseding decision that moved ADR 0089's spelling cannot leave two. Matched exactly: `OFF`, `Off`, `off `, ` off` are refusals, which the paired-negative test holds.

### The refusal path

Every unusable value is a typed `EvictionRefusal` (`Empty`, `NotText`, `Malformed`, `NotABound` carrying the cache's own `MaxEntryAgeRefusal`, `TooLarge`) and every one of them means: **do not evict, and carry on**. The expansion compiles, publishes, and embeds exactly as it would have; nothing is removed; no bound is guessed; the build does not fail. `TooLarge` is refused rather than saturated for the same reason — a saturated age would silently mean `off`, and inventing that from an arithmetic accident is the guess this module exists to refuse.

It is attributable: one line on the expanding process's standard error, naming the variable, the offending value, the accepted spellings, the opt-out, the default, and "Nothing was removed" — at most once per process.

**Measurement — macOS 27.0 build 26A5388g, `cargo 1.97.0` and `cargo +nightly-2026-07-19`, 2026-08-04.** A scratch proc-macro crate outside the workspace whose expansion runs `eprintln!` prints the line to the terminal under `cargo build`, once, between the `Compiling` and `Finished` lines, on both toolchains. Boundary: that is Cargo forwarding `rustc`'s standard error, so it holds for a build that *expands* — a fully warm build runs no macro and prints nothing — and where the rust-analyzer server's standard error surfaces was not measured, so the message is best effort there rather than a promise.

### The trigger, and the amortization rule

`aot::deliver` runs a pass when and only when `accept_or_publish_metal_plan` resolves to `Resolution::Published`. A `Hit` runs nothing (that single `matches!` is the whole of "off the hit path"), an `Uncached` resolution has no cache, and a `fallback-only` region never reaches the module. Nothing was needed from `tiler-cache`: the stop condition about the trigger requiring a cache change did not fire, because `Resolution` already distinguishes the three and `collect` is already public and total.

**Amortization: at most one pass per process**, held by a `static EvictionGate` claimed with one `AtomicBool` swap. Stated as a rule a person can say rather than as an interval or a probability, which is what the design record required of any trigger. What it means per driver is read off the measured process patterns in `docs/research/cache/build-tool-exercise.md`: under Cargo the expanding process is `rustc`, one per crate compilation, so a build sweeps at most once per crate that published; under rust-analyzer it is one proc-macro server for the editor session, so an afternoon of thousands of expansions sweeps once. That is the case the rule exists for. No clock decides whether it is time and nothing is persisted — a durable last-collection timestamp was already eliminated twice (durable state the design record refused, and a threshold rather than an act as its explanation).

The scan also rides on a cost already paid: reaching a publication means this expansion just ran `metal` and `metallib` as external processes.

Two flags rather than one, because they fire independently: a host with an unusable statement never sweeps, and a host with a good one never reports. `Relaxed` ordering on both — they order nothing and guard no data, and losing a race costs one extra scan or one extra line.

### The `CollectionReport` disposition — dropped, deliberately

The schedule elimination's two discriminators were *a report must terminate in a reader* and *a bound must arrive with its trigger*, and this ticket's own decision record states that Tom re-weighted exactly those: automation over per-act attribution, the `cargo`/`sccache` shape, where routine hygiene is silent. So there is no reader to reach, and each available channel fails on its own: `compile_error!` fails a build over housekeeping; a build-log line per eviction is noise attached to whichever invocation happened to publish first; a marker file is durable state the design refused. A scan failure (`CacheUnavailable`) is dropped for the same reason — the artifact is already correct and a cache that cannot be walked must not fail the build that filled it.

What replaces it is the policy being readable back: an entry leaves only for the age the consumer's own environment stated, the zero-config value is a documented constant with its ground, and the same public `ExpansionCache::collect` this code calls returns the same per-entry report to anyone who wants it.

**The asymmetry is deliberate and is the one place silence was refused:** a removal is Tiler doing its job, while an unparseable value is the consumer's own input doing nothing, and silence there leaves a setting that looks configured and is not.

### Tests

`crate::eviction::tests` (pure, no filesystem, no process environment): `an_unconfigured_host_evicts_under_the_cache_crates_default_age`, `the_automatic_policy_states_an_age_and_no_aggregate_ceiling`, `the_default_spelling_round_trips_through_the_parser`, `the_opt_out_disables_eviction_exactly`, `a_value_resembling_the_opt_out_is_refused_rather_than_read_as_one`, `every_accepted_unit_states_the_duration_it_names`, `every_unusable_value_refuses_the_eviction_rather_than_guessing_a_bound` (12 enumerated values, each asserted against the exact refusal it earned), `no_floor_sits_above_the_one_refused_age`, `a_value_that_is_not_text_refuses_rather_than_being_read_lossily`, `every_refusal_states_the_variable_both_remedies_and_the_consequence`, `observation_reads_exactly_the_policy_variable`, `a_gate_admits_one_sweep_and_one_report_per_process`, `an_unusable_statement_reports_once_and_evicts_nothing`, `a_usable_statement_and_the_opt_out_report_nothing`.

`crate::aot::tests` (end to end, real Apple toolchain, one scratch root each): `a_publishing_expansion_evicts_an_entry_that_reached_the_stated_age`, `a_cache_hit_evicts_nothing`, `the_opt_out_publishes_and_removes_nothing`, `an_unusable_eviction_policy_delivers_the_region_and_removes_nothing`, `only_the_first_publication_in_a_process_sweeps` (with its control: a third expansion differing from the second in the gate alone, which does remove the aged entry).

No test mutates the ambient process environment. Every one supplies an `EvictionEnvironment` value and its own `EvictionGate`, following the `RootEnvironment` idiom this crate already uses because it forbids the `unsafe` a process-environment mutation would need. Entry ages are set with `File::set_modified` on the published bundle — the same metadata the collector's scan reads and its locked removal re-`stat`s against — which is `tiler-cache`'s own age-test idiom.

### Fault-proof: three perturbations, each watched fail, each restored

| Perturbation | Failed | Message |
| --- | --- | --- |
| trigger widened to `Resolution::Published \| Resolution::Hit` | `a_cache_hit_evicts_nothing` alone | `a hit must leave the cache exactly as it found it — left: [], right: [".../4c1121e9….bundle"]` |
| `EvictionGate::claim_sweep` always returns `true` | `a_gate_admits_one_sweep_and_one_report_per_process` and `only_the_first_publication_in_a_process_sweeps` | `publication 0 after the first must sweep nothing`; `a later publication in one process must run no pass at all` |
| `off` compared with `eq_ignore_ascii_case` | `a_value_resembling_the_opt_out_is_refused_rather_than_read_as_one` alone | `only the exact opt-out disables the eviction: Disabled` |

### What the contract still owes

`docs/integration/frontends.md` is `contracts/integrations`, held by a live sibling ticket, so this branch did not edit it. The complete section — variable, default, opt-out, spellings, refusal behaviour, trigger, amortization, and report disposition — is in this worker's report under **Integrator edits**, ready to paste into the *Compiler cache* section after the ADR 0089 paragraph.

### Found while implementing, not fixed here

- `crates/tiler-cache/src/expansion/collect.rs`'s module documentation says the frontend "invokes this operation off the hit path" as a statement about a caller that did not exist when it was written. It is true as of this branch, and no edit was needed; `implementation/cache` is not this ticket's scope in any case.
- Clippy's `duration_suboptimal_units` and `manual_is_multiple_of` both fired on first-draft code here, matching what `admit-an-age-bounded-automatic-eviction-into-the-expansion-cache` reported for the same lints in `tiler-cache`.
