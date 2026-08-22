---
id: measure-thread-execution-width-on-the-standard-metal-profiles-own-host
title: Measure threadExecutionWidth on the standard Metal profile's own host
status: in-progress
priority: p3
dependencies: []
related: [declare-metal-subgroup-realization-facts-in-the-target-profile, measure-metal-thread-execution-width-across-prepared-pipelines, correct-the-metal-profile-authority-ledgers-stale-identity-pins]
scopes: [research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, metal, subgroup, target-profiles, evidence, needs-tom]
claimed_from: todo
assignee: worker-width
lease_expires_at: 1787424719
---
## User-visible outcome

The standard macOS Apple9 profile (`tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`) can state a subgroup realization backed by a width measured on its own ledger host, instead of staying subgroup-silent forever or inheriting the M3 Pro row.

## Why deferred rather than ready

**Fact — the standard profile is subgroup-silent by its evidence, not by an oversight.** The retained width measurement (`spikes/target-profiles/metal-thread-execution-width`) is M3 Pro evidence whose frozen protocol pre-scoped it away from this profile by name; `declare-metal-subgroup-realization-facts-in-the-target-profile` therefore landed the evidence-backed row on a new M3 Pro-scoped declaration and asserted the standard profile's silence by test.

**Fact — no host currently matches the standard profile's ledger execution row.** The ledger row is `Apple M4 Max`, macOS 27.0 build `26A5388g`; the coordination M4 Max observed on 2026-08-18 reports build `26A5406e`. A width measured there could not source the existing row either — it would be a different execution environment, exactly the inheritance the ledger refuses.

**The path, under the accepted composition model.** A new width measurement runs on the **current** M4 Max (`Apple M4 Max`, macOS 27.0 build `26A5406e`, `arm64`) under a **new frozen protocol that pre-names `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` as beneficiary before the run**, and lands as a subgroup row carrying its own population source and its own execution environment beside the existing rows — not as a source for the `26A5388g` row, which it may never be. That is component 3 of the model ([ADR 0113](../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md), `docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md "pre-registered beneficiary, stated value, measured validity"`), and component 2 is what makes a profile's rows legally span two environments, each scoped exactly. The measurement session itself needs no new permission: the standing measurement authorization covers it (`tickets/resolve-the-retained-metal-profile-measurement-invocation-authority.md "never ask for permission again"`), and no host or toolchain **change** is proposed — the run happens on the host as it is. What stays Tom's is the last step: adding the subgroup fact family to the flagship public profile moves that profile's stated content, its descriptor, and every dependent pin, so it reaches him as a decision packet under the existing rules.

*(Corrected 2026-08-19 by [`apply-the-accepted-host-evidence-composition-model`](apply-the-accepted-host-evidence-composition-model.md) at acceptance of the composition model. This paragraph previously offered two branches and called the choice between them Tom's — an M4 Max host restored to or found at the ledger's exact row plus an authorized quiet device window, or a profile revision that re-rows the whole standard declaration to a currently observable M4 Max environment. Both branches presupposed the one-execution-environment-per-profile convention the accepted model replaces: restoring the exact build is unnecessary because the new row carries its own population source, and re-rowing the whole profile is forbidden, since environments are never folded and the key may not be bumped for an OS-build move. Neither branch is the path any more.)*

## What a run would do

Freeze a **new** protocol first — same matrix, flags, repetitions, custody, and stop conditions as `measure-metal-thread-execution-width-across-prepared-pipelines`, but naming `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` as beneficiary in the protocol text before a single width is read, because a record cannot be scoped to a profile after the fact. Then run it on the current host, retain the record beside the existing ones, and declare (through the Metal-owned factory pattern `BoundMetalSubgroupDeclaration` established) only what the record evidences: whole-subject equality, nothing extrapolated, silence for every unobserved subject.

## Closes when

The standard profile either carries a `Realized` subgroup row backed by a retained measurement on its own execution row, or a recorded decision retires the question.

## Released from deferred — 2026-08-22, acting on this ticket's own fired trigger

The trigger log below already records `2026-08-19 — **fired, on the decision rather than on the host**`, and states the ticket stayed `deferred` only because "ticket state changes belong to the coordinator, not to the sweep that repaired the premise." **That coordinator action is this note.** Verified at `fd7706e0`: ADR 0113 carries `decision_status: "accepted"` and its `pre-registered beneficiary, stated value, measured validity` clause resolves, superseding both branches this ticket waited between.

**HOST DRIFT — correct the protocol before pre-registering anything.** This ticket records the host build as `26A5406e`. `sw_vers -buildVersion` now prints **`26A5416b`** — verified by the coordinator. The frozen protocol must pre-name `26A5416b`, not the recorded build, and the measurement is valid for that row only. This is host drift, **not** an authorized environment change: do not install, downgrade, or alter any toolchain or OS component — AGENTS.md reserves that to Tom.

## Trigger check log

- 2026-08-18 — **not fired.** `ssh m3` is an M3 Pro; the coordination M4 Max reports `sw_vers -buildVersion` → `26A5406e`, not the ledger's `26A5388g`. Reproduce: `sw_vers -buildVersion` and `sysctl -n machdep.cpu.brand_string` on the candidate host, compared against the ledger's execution table. *(That comparison was the trigger under the two-branch premise corrected above; it is no longer the condition this ticket waits on — see the 2026-08-19 entry.)*
- 2026-08-19 — **fired, on the decision rather than on the host.** Tom accepted the host-evidence composition model ([ADR 0113](../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md)), which supersedes both branches this ticket was waiting between: the measurement no longer needs a host at the ledger's exact build, and the profile is not re-rowed. The measurable half is now unblocked on the current host under the standing measurement authorization. This ticket stays `deferred` rather than moving itself: its close condition still needs the Tom-facing fact-family packet that adds a subgroup row to the flagship public profile, and ticket state changes belong to the coordinator, not to the sweep that repaired the premise. Reproduce: `grep -c "pre-registered beneficiary, stated value, measured validity" docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md` (nonzero — component 3 is recorded accepted), and `sw_vers -buildVersion` plus `sysctl -n machdep.cpu.brand_string` on the intended host to fix the environment the new protocol will pre-name (this host, 2026-08-19: `26A5406e`, `Apple M4 Max`).
