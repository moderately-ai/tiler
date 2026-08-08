---
id: correct-the-reversed-requirement-order-in-the-serial-sum-run-doc-comment
title: Correct the reversed requirement order in the serial sum run doc comment
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: []
---

A doc comment states the reverse of what its own function does — and it is the **origin** of a false claim that propagated into two tickets and a coordinator brief before anyone read the code.

## Facts

**Coordinator-verified at `e71b5c0d`.** In `prototypes/serial-sum-run/src/proof.rs`, `fn resolve_prepared_route` calls `qualify_live_device` (`resolve_live_device_requirements`) **first**, then `check_direct_requirements`, then `prepare_pipelines`. Its doc comment says the opposite — "the requirements the verified program itself derived … then the live-device rows the artifact carried".

**Fact — the propagation is the point.** That comment produced a false Fact in `check-synchronization-realization-before-the-routing-commit`, which the coordinator repeated in a brief as "one stage earlier than a live-device resolution", which then entered `discharge-the-derived-requirements-in-the-candle-metal-adapter`. The worker on that last ticket traced it back and repaired the parent with a dated correction. **The true statement is that the discharge stage is one earlier than *pipeline preparation*, not than the live-device rows.**

**Fact — the design is unaffected.** `route_with_adapter` also calls `prepare_entries` after live-device resolution, so the landed order is reproduced exactly. Only what may be *claimed* about it changes.

## What closes this

The comment stating the actual order, cited by **searchable anchor**. Check whether the surrounding sentences depend on the reversed reading — a comment wrong about sequence often has neighbours reasoning from it.

**Treatment:** establish with `git log -S` and `git show <commit>:<file>` whether it was ever true. If the order was reversed at some commit and later changed, date beside; if it was never the actual order, substitute with the retired wording quoted. Repository practice, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim **stays greppable**; say inline that a later hit lands inside your note.

**`prototypes/` is excluded from the style gate but reached by `build`, `test`, and `doc`.** A worker this week had `make full` exit 2 on a broken intra-doc link in this crate. So a warning here is invisible and a broken doc link is not — say which your change could produce, and **read the log tail rather than trusting a reported exit code**: another worker had exit 2 reported as 0 because the exit line went through `tee`.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`** — anchors fail as absence four ways: a line break inside them, an emphasis or backtick marker the source lacks, unescaped brackets read as a character class, and a quoted sentence that never appeared contiguously.

**Check this file's other ordering and stage claims and name the count.** This one misled three documents; assume it is not alone.

## Worker findings, verified at `aae3da24`

**Every stated Fact above is verified**; none is false or imprecise. Each was re-read in full at this base rather than taken from the ticket.

- *Fact 1 (coordinator-verified)* — **verified.** `fn resolve_prepared_route` calls `qualify_live_device`, then `check_direct_requirements`, then `prepare_pipelines`, then `resolve_target_properties`. Read in full in `prototypes/serial-sum-run/src/proof.rs`.
- *Fact 2 (the propagation)* — **verified as reported**, and refined below: the reversal was introduced by `e4041047`, so it was never the order the code ran.
- *Fact 3 (the design is unaffected)* — **verified** in `crates/tiler-runtime/src/adapter.rs`: `route_with_adapter` calls `qualification.resolve_live_device_requirements(...)` and then `adapter.prepare_entries(&context, preparation.entries())`. Nothing in `crates/**` was edited.

**Fact — the comment was never true, so the treatment is substitution rather than a date-beside.** `git log --oneline -S 'Three device stages in the order their facts become true' -- prototypes/serial-sum-run/src/proof.rs` returns exactly `e4041047` (2026-08-08). Its diff replaced a correct two-stage sentence with the three-stage one while inserting the discharge call *below* `qualify_live_device`, so the prose and the code disagreed from the moment the paragraph existed. The retired wording is now quoted verbatim inside a dated correction on the function, and a `grep -F` hit for it lands inside that note.

**Fact — the anchor in Fact 1 above is not greppable, and that is the wrap trap, not an absence.** `git log -S 'then the live-device rows the artifact carried'` returned nothing at this base because the doc comment wrapped after `then the`. Use `grep -F 'live-device rows the artifact carried'` instead.

**Fact — the order is not forced by the loader's types.** `LiveDeviceQualification` publishes `entries()` as well as `RoutePreparation` (`crates/tiler-runtime/src/load/route.rs`), so `check_direct_requirements` could structurally have been called before `qualify_live_device`. No corrected comment claims otherwise.

**Neighbour census: 27 ordering and stage claims read in this file, of which 2 were reversed and 1 misattributed.** The population is every claim asserting a sequence or a before/after relation over the route-preparation path, from decode through the routing commit; the file was read in full rather than grepped, and the count is enumerated rather than estimated. Fixed: the subject; the `run()` comment `Placement first, then the device.` (true at `40c58f32`, falsified by `0b7e59d3` inserting `resolve_prepared_route` beneath it without moving it); and `plan_route`'s doc naming `device_preflight` as the taker of "the library, the pipeline", which `0b7e59d3` moved to `prepare_pipelines`. Verified correct and left alone: `PreflightPhase::DirectRequirement`'s "First" claim, whose domain is the `PreflightRefusal` vocabulary and therefore excludes the live-device stage; `check_direct_requirements`'s synchronization-before-family ordering; `DirectRequirementsDischarged`'s by-value ordering claim; `prepare_pipelines`'s "before any deferred property"; the one-encoder-per-stage guarantees; and `sole_exclusion`'s "a filter applied before any guard is evaluated", which `select_variant` in `crates/tiler-runtime/src/load.rs` bears out — it pushes an ineligible variant onto `filtered` and `continue`s before `boolean(variant.applicability_guard(), subject, facts)` is reached. Reported as loose but not reversed, and deliberately not edited because correcting them means restating the pre-commit boundary narrative this ticket names as a non-goal: the module header's "answered while the `Preflight` is still held" and `prove_contraction`'s list under "The ordering is the contract, not a sequence", both of which name pipeline preparation as inside the `Preflight`'s lifetime when it precedes it.
