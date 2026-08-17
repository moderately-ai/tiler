---
id: refresh-status-and-roadmap-after-landed-cache-reduction-sync-and-l8-work
title: Refresh status and roadmap after landed cache, reduction, synchronization, and L8 work
status: in-progress
priority: p2
dependencies: []
related: [decide-the-expansion-cache-collection-schedule, wire-the-env-configured-eviction-policy-through-the-deliver-path, calibrate-and-activate-parallel-reduction-selection, check-synchronization-realization-before-the-routing-commit, correct-the-roadmap-s-milestone-0b-inline-composition-claim, land-the-model-level-qualification-record, decide-the-tiler-metal-public-facade-surface]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, status, roadmap, correction]
claimed_from: todo
assignee: worker-status-roadmap
lease_expires_at: 1786976056
---
## User-visible outcome

The status portal and roadmap describe the work that is actually landed. They preserve every measurement and maturity limit while no longer assigning current work to terminal tickets or calling a landed record a draft.

## Exact-current Fact audit — 2026-08-17 at `404cacd21ee9a1ae91c10cc1d86b77f6752f2439`

1. **Imprecise — no automatic expansion-cache collection.** The audit base named above supersedes the prior `52de1babfe78f4bf3cac2c6e2bb8de50b1d401c5`. `docs/status.md`, anchor `nothing calls any of them automatically`, is false as a live claim: `crates/tiler-macros/src/aot.rs`, anchor `The one place an automatic eviction runs`, shows one frontend-owned pass after `Resolution::Published`. `crates/tiler-macros/src/eviction.rs`, anchor `MAX_ENTRY_AGE_VARIABLE`, owns `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`, whose unset policy cites the cache's thirty-day `MaxEntryAge::DEFAULT` and whose exact `off` disables removal; the sweep is at most once per process and deliberately drops its report. `crates/tiler-cache/src/expansion.rs`, anchor `no environment is read here`, remains the boundary: that crate owns neither environment parsing nor a schedule.
2. **Imprecise — measured reduction selection and delivery-time synchronization are remaining work.** `docs/status.md`, anchor `The remaining work is not backend qualification`, points to `calibrate-and-activate-parallel-reduction-selection` and `check-synchronization-realization-before-the-routing-commit`; both are `done`. The former measured a crossover only for one profile, `FLUSH_AND_REASSOCIATE_F32`, one multiply-add-prologue trailing-axis-sum family, `f32`, and one host row, but its anchor `Selection is unchanged and still takes the serial fold everywhere` means it did **not** activate a cost-based winner; the separately blocked `activate-measured-reduction-selection-from-a-target-cost-row` owns that public/identity decision. The latter's anchor `before any pipeline is prepared` proves delivery-time synchronization discharge is implemented. The unresolved whole-`tiler-metal` facade belongs to `decide-the-tiler-metal-public-facade-surface`, not either terminal implementation ticket.
3. **Verified — the Milestone 0B roadmap no longer denies inline composition.** `docs/status.md`, anchor `still asserts that inline composition does not exist`, is false as a live claim: `docs/roadmap.md`, anchor `Inline composition exists`, records the corrected position. `correct-the-roadmap-s-milestone-0b-inline-composition-claim`, anchor `Residual navigation`, identifies the status-page drift. Consumer integration and the second measured Apple family remain absent, and this repair must not declare the milestone exited.
4. **Verified — the L8 qualification record is landed.** `docs/roadmap.md`, anchor `the record is drafted rather than landed`, is false as a live claim: the carrier `land-the-model-level-qualification-record` is `done`, and `docs/research/program-planning/model-level-qualification.md`, anchor `durable design record for rung L8`, is its destination. The record remains research only: `implementation_status: not-started`, no Tiler-side measurement, no execution, no capability or rung promotion.

## Required work

- Correct the four present-tense claims above after reading `docs/status.md` and `docs/roadmap.md` in full at the implementation base.
- State the cache policy exactly: `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`, the cache-owned thirty-day default, exact `off`, post-publication trigger, at-most-once-per-process gate, and deliberately dropped collection report. Preserve that `tiler-cache` itself owns no environment or schedule.
- State the narrow measured reduction boundary and that synchronization discharge is implemented. Route the remaining whole-`tiler-metal` public-facade maturity question to `decide-the-tiler-metal-public-facade-surface`, not to terminal implementation tickets.
- Remove only the stale Milestone 0B pointer; preserve consumer-integration, second-family, fallback, measurement, and exit-accounting limits.
- Change only the L8 maturity cell needed to say the design record landed. Preserve its `not-started` implementation and no-measurement/no-execution boundary.
- Add a source-reading check whose negative is reachable for each retired phrase, perturb each subject independently, and quote the failure before restoring it.

## Source-reading check — 2026-08-17

Run from the repository root. Each subject is an exact phrase in its dated correction; the `case` makes a missing subject fail loudly rather than letting an empty grep read as a pass.

```sh
for check in \
  'docs/status.md|frontend-owned `aot::deliver` reads `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`' \
  'docs/status.md|the serial fold remains selected everywhere' \
  'docs/status.md|says **"Inline composition exists"**' \
  'docs/roadmap.md|is landed at its intended destination'; do
  file=${check%%|*}
  subject=${check#*|}
  rg -q -F "$subject" "$file" || {
    printf 'source-claim check: missing current subject %s in %s\n' "$subject" "$file"
    exit 1
  }
done
```

The negative controls independently restored cache-owned collection, an active cost-based winner, absent inline composition, and a drafted L8 record. Each then failed with the printed `source-claim check: missing current subject …` message before the current subject was restored.

## Non-goals

No capability, maturity promotion, milestone-exit decision, cache-policy change, new measurement, Metal facade acceptance, or production edit.

## Closes when

All four stale source claims are absent from their live narrative positions, their replacements name the exact current bounded truth, independent subject perturbations fail, and navigation citations and ticket status remain coherent.
