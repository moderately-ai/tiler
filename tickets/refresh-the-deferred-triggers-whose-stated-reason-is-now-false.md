---
id: refresh-the-deferred-triggers-whose-stated-reason-is-now-false
title: Refresh the deferred triggers whose stated reason is now false
status: todo
priority: p2
dependencies: []
related: [repair-the-deferred-trigger-checks-that-cannot-report-a-firing, size-the-governed-key-census-from-the-type-across-the-deferred-pool]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [graph-hygiene, deferred-triggers]
---
## User-visible outcome

Six deferred tickets state a holding reason that is true, so a later sweep can act on what it reads instead of re-deriving why each one is still parked.

## Why this exists

Filed 2026-08-22 from the deferred-pool trigger audit. **Its headline is a negative result worth stating plainly: across all 119 logs, no trigger was found to have unambiguously fired while logged `not fired`.** That is the good outcome, and it differs from what prompted the sweep. What it did find is six whose *stated reason* is now false while the verdict survives on grounds the entry never gives — which produces the same end state as a mislogged firing, a ticket that reports "not fired" forever.

All six are the audit's, and **the coordinator has verified two**. Re-audit each at your base and report a per-Fact verdict; do not inherit any of them.

1. **`derive-the-exact-evaluator-for-a-multi-round-cooperative-fold-order` — genuinely ambiguous, and the ambiguity decides it.** Its trigger fires when a topology realizing multi-round composition is "reachable from a constructed plan". The compiler has since gained `verify_cooperative_contraction_subject_binding`, `COOPERATIVE_CONTRACTION_REGION`, and a `CooperativeContraction` cost arm — so it now **verifies and scores** a multi-round topology it does not itself construct. Under "compiler-offered" it has not fired; under "provider-supplied" a caller-installed provider could supply one that verifies and scores. **The audit corrected its own subagent here**, which had cited `blocked_operand_tile` uses as production when every one sits inside `#[cfg(test)] mod tests`; the conclusion survived, the evidence did not. **Settling what "constructed plan" means is the work.**
2. **`keep-the-first-macos-apple9-host-row-on-its-measured-os-build` — self-contradictory, and fired under one reading.** One sentence makes the host printing `26A5388g` jointly necessary, which requires the machine to *revert*; the next names an authorized new-build measurement as sufficient on its own. Read literally it can never fire. Two stale facts alongside: the ticket says the host "now reports `26A5406e`" while `/usr/bin/sw_vers -buildVersion` returns **`26A5416b`** — verified by the coordinator — and its governing authority moved, ADR 0113 listing "bumping the key for an OS-build move" among forbidden moves. The pin itself is intact.
3. **`measure-complete-explain-demand-and-lossless-compaction-for-full-physical-provider-activity` (p1) — premise refuted by Tom.** Its last entry rests on "no consumer or accepted support contract names one". On **2026-08-18 Tom accepted two active specialists as the supported ordinary population**. The verdict survives only on an unstated qualifier, and the ticket's own Dispatch condition says to repair its subject when a trigger names a smaller population — it still measures 31.
4. **`scope-launch-granularity-optimization-for-the-decode-dominated-regime` — the one genuinely dispatchable item.** Logged **fired** on 2026-08-09, then partially fired and unevaluable for closure on 2026-08-11 solely because "no timing was run on this coordination host". Unrevisited for 11 days. The harness exists; it needs an idle-M3 measurement, which is pre-authorized. **See the hold below.**
5. **`remove-the-loop-carried-redundant-staged-fold` — stated reason inverted.** Its 2026-08-09 basis is that the tiled-contraction owner "remains `status: deferred`"; that ticket is now `done`. Still not fired on the merits — the contraction vertical rode the `direct` realization, not a multi-round cooperative kernel — so this is a log refresh, with limb 1 moving from unevaluable to evaluable-not-measured.
6. **`reconsider-registered-quantitative-capability-axis-schemas` — premise dead.** Its basis was "no independently authored target profile exists at all"; one landed on 2026-08-22 in the new conformance fixture. Not fired on the merits: that fixture shows no quantitative-axis blockage.

## The hold, recorded with its exact release trigger

**Item 4 needs an idle-M3 timing run and cannot have one yet.** The bench host reported load `2.13 2.17 2.16` with no process above 3.2% CPU when the tile-width protocol lane last measured it — above the 0.5 gate that protocol freezes, with the cause unidentified rather than transient. **Release trigger: bench-host load below 0.5.** The same hold blocks `calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol`; when it clears, both run.

## Required work

- Per-Fact verdict on all six before editing anything.
- For each, repair the stated reason to one that is true, **preserving the retired wording in a dated correction**. Expect grep counts not to shrink; a shrinking count is a false progress signal.
- Where the verdict itself changes, say so and stop rather than dispatching — whether a ticket becomes dispatchable is the coordinator's call.
- Item 1 needs a reading settled, not a sentence patched. If the ticket text cannot settle what "constructed plan" means, **say so and stop**; that becomes one concrete question rather than a guess.

## Non-goals

Running the idle-M3 measurement, which is held above. Repairing unfireable commands or the hand-maintained key census — both have their own tickets. Any edit outside `tickets/`.

## Closes when

All six state a holding reason that is true at the time of writing, every correction preserves what it replaced, any changed verdict is reported rather than acted on, and item 1's reading is either settled from the text or handed up as one question.
