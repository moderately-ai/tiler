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
- **Reason repaired — 2026-08-22; verdict unchanged, still not fired.** The 2026-08-12 entry states a firing condition that **cannot ever be satisfied as written**, and the two halves of the contradiction are quoted here rather than deleted so the retired wording stays searchable. It says the command `must print 26A5388g *and* Tom must have authorized a new measurement`, which makes a matching build **necessary**; the very next sentence says `an authorized new-build measurement is the firing condition`, and a *new-build* measurement is by definition one taken at a build that is not the pin. Conjoined, the two demand that the host simultaneously be on `26A5388g` and not be, so the entry as written forecloses its own trigger.

  **The defect is a sufficient condition read as a necessary one.** They are not rival requirements; they are two independent routes, and this ticket's own body already names both. `## Closes when` reads `The pin and the live host agree again under an authorized measurement, or Tom directs that the coordination host stay outside the measured row`, and `## Work when the trigger fires` opens with `Authorize the evidence-environment change` and then offers `or accept an explicit attestation that the new build is the same row` — a bullet that is meaningless unless the new build differs from the pin. So: **route A**, the host returns to `26A5388g` on its own and the pin and host agree with nothing to authorize; **route B**, Tom authorizes the evidence-environment change and the retained measurements are re-run or attested on whatever build the host is on, moving the pin. Tom's authorization is necessary for route B alone. The printed build value is necessary for neither route on its own, and it is what the 2026-08-12 entry wrongly welded to the authorization.

  **Not fired, on the repaired condition rather than the impossible one.** Route A: `/usr/bin/sw_vers -buildVersion` prints **`26A5416b`**, not `26A5388g` — it has moved twice now, past the `26A5406e` the 2026-08-12 entry recorded, and the entry above already carries that correction. Route B: no authorization exists. The standing measurement authorization covers measurement *sessions* and excludes environment changes, so it does not reach this. The pin is intact: `grep -n 'os_build: "26A5388g"' crates/tiler-metal/src/applicability.rs` returns one line. Reproduce both halves with `/usr/bin/sw_vers -buildVersion` and that grep; the changed answer is a printed `26A5388g`, or a recorded authorization from Tom.

  **One governing-authority move, which narrows what route B may do.** [ADR 0113](../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md) was accepted on 2026-08-19, after every entry above. Its decision 1 makes OS builds and hardware model names provenance vocabulary that `may not appear in a minted public key`, and its reseat-carrier section lists `bumping the key for an OS-build move` among forbidden moves. `MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9` carries `id: "tiler.metal.host-applicability.macos-27.0-26A5388g-arm64-m4max-apple9.v1"`, which embeds both the build and the model, so this ticket's `Update the policy id` bullet is no longer obviously permitted work. **Stated with its exact limit, because overstating it would be the same error this entry repairs:** ADR 0113's forbidden list is written for the reseat carrier and about public *compile-profile* keys, and a host-applicability policy id is a different identifier in a different namespace. ADR 0113 addresses this surface directly and says it is `unchanged today — no host earns a receipt`, handing the policy-shape question to [`define-host-applicability-for-profiles-whose-rows-span-environments`](define-host-applicability-for-profiles-whose-rows-span-environments.md) (`deferred`). So the authority did not change this ticket's verdict; it changed who owns the id question when route B runs, and that ticket is where it is owned.
