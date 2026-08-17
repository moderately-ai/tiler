---
id: refresh-status-and-roadmap-after-landed-cache-reduction-sync-and-l8-work
title: Refresh status and roadmap after landed cache, reduction, synchronization, and L8 work
status: todo
priority: p2
dependencies: []
related: [decide-the-expansion-cache-collection-schedule, wire-the-env-configured-eviction-policy-through-the-deliver-path, calibrate-and-activate-parallel-reduction-selection, check-synchronization-realization-before-the-routing-commit, correct-the-roadmap-s-milestone-0b-inline-composition-claim, land-the-model-level-qualification-record, decide-the-tiler-metal-public-facade-surface]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, status, roadmap, correction]
---
## User-visible outcome

The status portal and roadmap describe the work that is actually landed. They preserve every measurement and maturity limit while no longer assigning current work to terminal tickets or calling a landed record a draft.

## Exact-current Fact audit — 2026-08-17 at `52de1babfe78f4bf3cac2c6e2bb8de50b1d401c5`

1. **False — no automatic expansion-cache collection.** `docs/status.md`, anchor `nothing calls any of them automatically`, is contradicted by `crates/tiler-macros/src/aot.rs`, anchor `The one place an automatic eviction runs`. `aot::deliver` reads the environment-configured policy and runs age eviction only after `Resolution::Published`, at most once per process. Hits, uncached delivery, and fallback-only delivery do not run it. The cache crate itself still reads no environment and schedules nothing.
2. **Stale — measured reduction selection and delivery-time synchronization are remaining work.** `docs/status.md`, anchor `The remaining work is not backend qualification`, points to `calibrate-and-activate-parallel-reduction-selection` and `check-synchronization-realization-before-the-routing-commit`; both are `done`. Selection is active only inside its recorded one-profile, one-contract, one-program-family, F32, one-host measurement boundary. Both retained runtime adapters consume synchronization realization before pipeline construction.
3. **False — the Milestone 0B roadmap still denies inline composition.** `docs/status.md`, anchor `still asserts that inline composition does not exist`, contradicts `docs/roadmap.md`, anchor `Inline composition exists`. The terminal correction ticket itself records this residual. Consumer integration and the second measured Apple family remain absent, and this repair must not declare the milestone exited.
4. **False — the L8 qualification record is drafted rather than landed.** `docs/roadmap.md`, anchor `the record is drafted rather than landed`, contradicts the `done` carrier and `docs/research/program-planning/model-level-qualification.md`, anchor `durable design record for rung L8`. The landed record remains research only: `implementation_status: not-started`, no Tiler-side measurement, no execution, no capability or rung promotion.

## Required work

- Correct the four present-tense claims above after reading `docs/status.md` and `docs/roadmap.md` in full at the implementation base.
- State the cache policy exactly: `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`, the cache-owned thirty-day default, exact `off`, post-publication trigger, at-most-once-per-process gate, and deliberately dropped collection report. Preserve that `tiler-cache` itself owns no environment or schedule.
- State the narrow measured reduction boundary and that synchronization discharge is implemented. Route the remaining whole-`tiler-metal` public-facade maturity question to `decide-the-tiler-metal-public-facade-surface`, not to terminal implementation tickets.
- Remove only the stale Milestone 0B pointer; preserve consumer-integration, second-family, fallback, measurement, and exit-accounting limits.
- Change only the L8 maturity cell needed to say the design record landed. Preserve its `not-started` implementation and no-measurement/no-execution boundary.
- Add a source-reading check whose negative is reachable for each retired phrase, perturb each subject independently, and quote the failure before restoring it.

## Non-goals

No capability, maturity promotion, milestone-exit decision, cache-policy change, new measurement, Metal facade acceptance, or production edit.

## Closes when

All four stale source claims are absent from their live narrative positions, their replacements name the exact current bounded truth, independent subject perturbations fail, and navigation citations and ticket status remain coherent.
