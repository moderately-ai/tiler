---
id: keep-the-first-macos-apple9-host-row-on-its-measured-os-build
title: Keep the first macOS Apple9 host row on its measured OS build
status: deferred
priority: p2
dependencies: []
related: [validate-macos-metal-profile-host-applicability]
scopes: [research/target-profiles, implementation/conformance, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, target-profiles, deferred]
---
## User-visible outcome

`MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9` continues to name the OS build the retained measurements actually ran on. A newer coordination-host build is refused as outside the measured row until Tom authorizes a new measurement and a pin change.

## Why this is deferred

**Fact — 2026-08-12, base `5704a1eae08248a7bc7b6ff64f7b4af9b6b037d0`.** `FIRST_MACOS_APPLE9` still requires `os_build: "26A5388g"`. `sw_vers -buildVersion` on the coordination host now reports `26A5406e`. The live offer path refuses `MetalHostPredicate::OsBuild` with `metal.host-applicability.outside-measured-row` and does not reach ADR 0086. Reproduce: `/usr/bin/sw_vers -buildVersion` and `rg 'os_build: "26A5388g"' crates/tiler-metal/src/applicability.rs`.

**Fact.** Widening that pin is a new measurement, not a test update. ADR 0086 and the policy type both say the only value is the transcribed measured row; AGENTS.md forbids changing the evidence environment without Tom's authorization.

**Fact.** The matching-row ADR 0086 authority refusal remains proven on a constructed observation in `applicability::tests::the_composed_observation_answers_every_predicate`. The live-host serial-sum offer test now derives its expected predicate from this host's ambient fields plus the policy's device row, so a drifted build keeps the gate green without claiming this host is still the measured row.

## Work when the trigger fires

- Authorize the evidence-environment change.
- Re-run the retained measurements that `FIRST_MACOS_APPLE9` transcribes, or accept an explicit attestation that the new build is the same row.
- Update the policy id, required `os_build`, and every present-tense host-row claim that still names `26A5388g` as *this* host.
- Recompute any identity or ledger cells that fold the policy id.

## Closes when

The pin and the live host agree again under an authorized measurement, or Tom directs that the coordination host stay outside the measured row and the present-tense host claims are rewritten accordingly.

## Trigger check log

- 2026-08-12 — **not fired.** The host has left the measured OS build (`26A5406e` vs pin `26A5388g`), which is the recording condition, not authorization to widen the row. Reproduce: `/usr/bin/sw_vers -buildVersion` must print `26A5388g` *and* Tom must have authorized a new measurement before this becomes dispatchable. A matching build alone is not enough; an authorized new-build measurement is the firing condition.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **Correction to this repair's own premise:** the entry above *does* name a command, `/usr/bin/sw_vers -buildVersion`; the sweep that filed this repair missed it because its census matched a list of command verbs that did not include `sw_vers`, which is a limitation of the census and not of this ticket. Recorded rather than quietly dropped, because the same census shape is used elsewhere in this repair. Re-run at this base it prints `26A5416b`. The 2026-08-12 entry records the host at `26A5406e`; it has moved again and is still not the `26A5388g` pin, so the recording condition is unchanged while the observed value in that entry is stale. Firing needs the printed build to equal `26A5388g` **and** Tom's authorization; a matching build alone is not enough. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
