---
id: refresh-the-deferred-triggers-whose-stated-reason-is-now-false
title: Refresh the deferred triggers whose stated reason is now false
status: in-progress
priority: p2
dependencies: []
related: [repair-the-deferred-trigger-checks-that-cannot-report-a-firing, size-the-governed-key-census-from-the-type-across-the-deferred-pool]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [graph-hygiene, deferred-triggers]
claimed_from: todo
assignee: worker-refreshtrig
lease_expires_at: 1787437174
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

**Item 4 needs an idle-M3 timing run and cannot have one yet.** The bench host reported load `2.13 2.17 2.16` with no process above 3.2% CPU when the tile-width protocol lane last measured it — above the 0.5 gate that protocol freezes, with the cause unidentified rather than transient. **Release trigger, corrected 2026-08-22: the gate itself is unsatisfiable and must be re-derived first.** I originally wrote the trigger as *bench-host load below 0.5*. Checking it is what exposed the problem: the bench host's **idle baseline is roughly 2.3**, carried by the Tailscale network extension and the `AppleBCMWLAN` DriverKit extension — nothing CPU-bound, no process in disk-wait, runnable count 2, up 21 days. That figure is a floor the OS configuration imposes, not a queue that drains, so a gate at 0.5 does not delay the run, it **forecloses it permanently**. The real release trigger is therefore [`re-derive-the-quiet-host-gate-the-bench-host-cannot-satisfy`](re-derive-the-quiet-host-gate-the-bench-host-cannot-satisfy.md) landing. The same hold blocks `calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol`; when the gate is re-derived, both run. Recorded rather than worked around: the host is on the pinned build `26A5388g` and is the right machine — only the precondition is wrong.

## Required work

- Per-Fact verdict on all six before editing anything.
- For each, repair the stated reason to one that is true, **preserving the retired wording in a dated correction**. Expect grep counts not to shrink; a shrinking count is a false progress signal.
- Where the verdict itself changes, say so and stop rather than dispatching — whether a ticket becomes dispatchable is the coordinator's call.
- Item 1 needs a reading settled, not a sentence patched. If the ticket text cannot settle what "constructed plan" means, **say so and stop**; that becomes one concrete question rather than a guess.

## Non-goals

Running the idle-M3 measurement, which is held above. Repairing unfireable commands or the hand-maintained key census — both have their own tickets. Any edit outside `tickets/`.

## Closes when

All six state a holding reason that is true at the time of writing, every correction preserves what it replaced, any changed verdict is reported rather than acted on, and item 1's reading is either settled from the text or handed up as one question.

## Four more stale facts, added 2026-08-22 from the trigger-check repair lane

Each was found by *running* a check rather than reading around it, which is the point — a stale fact inside a trigger log is invisible until someone executes the thing.

- **Two tickets assert `declare-cpu-vector-realization-facts-in-the-target-profile` is `awaiting-decision`; it is `blocked`.** Verified by the coordinator at `51145c0a`: its frontmatter reads `status: blocked`, and `accept-adr-0093-cpu-vector-lane-tier.md` and `reconsider-registered-quantitative-capability-axis-schemas.md` both carry the older claim. A cross-ticket status claim that drifts is the shape that makes a sweep believe a decision is pending when it is parked.
- **`StorageScalar` has four variants, not the three a log asserts.** Reported by the repair lane, unverified by the coordinator — re-derive it **from the type** rather than by hand, and prefer `core::mem::variant_count` if the site admits it.
- **Governed keys are 50, not the 47 three `scope-the-*` entries carry.** This is the same transcription defect [`size-the-governed-key-census-from-the-type-across-the-deferred-pool`](size-the-governed-key-census-from-the-type-across-the-deferred-pool.md) owns, appearing at a third stale value; repair it there rather than here, and note it has now drifted 46 → 47 → 50.
- **The host OS build is `26A5416b`.** Already named in item 2 above; recorded here too because a second log carries the superseded `26A5406e`.

**One correction to the coordinator's own census shape.** I recorded that `keep-the-first-macos-apple9-host-row-on-its-measured-os-build` carries no command. It does — `/usr/bin/sw_vers -buildVersion`. The census that produced that claim missed it because its verb list omitted `sw_vers`. **A census over commands is only as complete as its verb list**, and a ticket wrongly recorded as commandless is the same false-absence direction as every other trap in this pool. Do not trust a commandless-entry count you did not derive yourself.

## Per-Fact verdict — 2026-08-22, re-audited at base `c3cf6f9f`

The ticket's first required deliverable. Every Fact was re-read at this base rather than inherited; each repair landed as a dated entry in the named ticket's own trigger check log, preserving the wording it replaced. **No verdict moved: all six remain not fired, and item 4 remains fired-and-parked as it already read.** The audit's headline negative result survives this re-audit.

| # | Ticket | Stated reason | Verdict on the *reason* | Verdict on the *trigger* |
| --- | --- | --- | --- | --- |
| 1 | `derive-the-exact-evaluator-for-a-multi-round-cooperative-fold-order` | "constructed plan" ambiguous under the new verify/score surface | **imprecise, and the ambiguity does not decide it** | not fired, unchanged |
| 2 | `keep-the-first-macos-apple9-host-row-on-its-measured-os-build` | build must equal `26A5388g` **and** Tom must authorize | **false — unsatisfiable as written** | not fired, unchanged |
| 3 | `measure-complete-explain-demand-…-full-physical-provider-activity` | "no consumer or accepted support contract names one" | **false — one is named** | not fired, unchanged |
| 4 | `scope-launch-granularity-optimization-for-the-decode-dominated-regime` | "no timing was run on this coordination host" | **true but not a holding reason** | fired 2026-08-09, unchanged |
| 5 | `remove-the-loop-carried-redundant-staged-fold` | tiled-contraction owner "remains `status: deferred`" | **false — it is `done`** | not fired, unchanged |
| 6 | `reconsider-registered-quantitative-capability-axis-schemas` | "no independently authored target profile exists at all" | **false — one landed 2026-08-22** | not fired, unchanged |

**Item 1's reading is settled from the text and needs no question to Tom.** The compiler-offered/provider-supplied fork does not decide the trigger, because the topology the compiler newly verifies and scores — `CooperativeContraction` — is not ADR 0100's multi-round composition at all: ADR 0100 states its composition over `CooperativeWorkgroup`'s contributor split, and `CooperativeContraction` carries no contributor split. Independently, `crates/tiler-compiler/src/physical.rs` documents the region identifier as one nothing in that crate constructs, existing so a *provider* supplying one is checked rather than refused — the distinction stated in production prose, agreeing with this ticket's own `compiler-constructed plan` Fact.

**Corrections to this ticket's own items, each with evidence in the target ticket's log.** Item 4's framing — unevaluable for closure *solely* for want of a timing run, hence the one dispatchable item — is **false**: the 2026-08-11 entry names four conditions and `## Closes when` clause (b) needs a decoder route and a decided observation model that an idle-M3 run cannot supply. Item 2's ADR 0113 claim is **imprecise**: that record does list `bumping the key for an OS-build move` among forbidden moves, but for the reseat carrier and about public *compile-profile* keys, and it says host applicability is `unchanged today`, handing the policy-shape question to a different deferred ticket.

**Stale facts.** The cross-ticket status claim is repaired in [`accept-adr-0093-cpu-vector-lane-tier`](accept-adr-0093-cpu-vector-lane-tier.md), where re-derivation found **two** further defects beside the one reported: `admit-vector-lane-bindings-into-the-schedule-vocabulary` is `done` rather than `awaiting-decision`, and the third ticket that sentence names was deleted on 2026-08-12 and its link resolves to nothing — invisible to `make citations`, which reads only open tickets. The `StorageScalar` count is **four**, re-derived from the type: four `[StorageScalar; variant_count::<StorageScalar>()]` arrays already exist, so the vocabulary is guarded by a build error rather than a hand count. The governed-key census is left to [`size-the-governed-key-census-from-the-type-across-the-deferred-pool`](size-the-governed-key-census-from-the-type-across-the-deferred-pool.md) as instructed. The host build is `26A5416b`; the claim that **a second log carries the superseded `26A5406e`** is **imprecise** — the only other trigger-log occurrence is a dated 2026-08-18 entry in a `done` ticket recording what the host printed on that date, which is correct history and must not be rewritten. Every other `26A5406e` in `tickets/` names a measured execution environment, not a live host.
