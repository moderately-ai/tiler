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

## Fact audit at base `14dc18c950f9eacd99bffaac7e110cbf263e3f57` — worker-width, 2026-08-22

Every Fact re-read at this base before any measurement. Two are stale.

- **Fact — "the standard profile is subgroup-silent by its evidence, not by an oversight."** **Verified.** `crates/tiler-build/src/metal_subgroup_declaration.rs` states the M3 Pro record's protocol pre-scoped it — `width claim over the frozen population only` — and that the standard profile `stays subgroup-silent`, asserted by a test in that module. (The neighbouring phrase "does not edit" is **not** usable as an anchor: it wraps across a `//!` line break in the source and greps to 0, which is the failure-as-absence shape AGENTS.md warns about. Both anchors above were run against the file this citation names.) The first protocol's own text confirms it: `spikes/target-profiles/metal-thread-execution-width/README.md "It does not source the M4 Max qualified numerical, grid-axis, or dispatchability rows"`.
- **Fact — "The ledger row is `Apple M4 Max`, macOS 27.0 build `26A5388g`."** **False as written, at this base.** The ledger no longer has *a* row: the reseat under ADR 0113 already landed, and it now keeps **two** execution environments per population — **A** (`26A5406e`, sourcing the grid-axis and cost rows) and **B** (`26A5388g`, sourcing the tree-width policy and the dispatchability/numerical rows). Anchor: `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md "four retained measurement populations across two execution environments on one device"`. The singular "the ledger row" is the pre-reseat framing the ticket's own correction note says was superseded; it survived in this paragraph.
- **Fact — "the coordination M4 Max observed on 2026-08-18 reports build `26A5406e`."** **Stale.** `sw_vers -buildVersion` → `26A5416b`. The coordinator flagged this and was right. Device is `Apple M4 Max`, `Mac16,6`.
- **Fact — "no host currently matches the standard profile's ledger execution row."** **Verified, and now doubly so.** `26A5416b` matches neither A nor B. Under ADR 0113 component 2 this no longer blocks anything: the new record carries its own population source and its own environment.
- **Fact — ADR 0113 is accepted and component 3 resolves.** **Verified.** `decision_status: "accepted"`; `grep -c "pre-registered beneficiary, stated value, measured validity"` on the ADR returns 1.

**Coordinator briefing claim corrected.** The brief stated that `metal` on this host resolves to a toolchain reporting `32023.921` while the profile's rows are measured under `32023.883`. True only of the **default** selection: `xcode-select -p` is `/Applications/Xcode-beta.app/Contents/Developer`, so bare `xcrun --sdk macosx metal --version` prints `32023.921`. Under `DEVELOPER_DIR=/Applications/Xcode.app` — which is what the frozen protocol pins, and what this run used — the host resolves `Apple metal version 32023.883 (metalfe-32023.883)`, `AIR-LLD 32023.883`, `Xcode 26.6 Build version 17F113`, SDK `26.5`/`25F70`: the authority ledger's offline table, field for field. So the offline row was **not** a difference in this measurement, and the only axis varied from environments A and B is the execution build. `crates/tiler-conformance/src/retained_record.rs` is consistent with this — its `this repository's host resolves` sentence describes the default resolution its own crate sees, not the `DEVELOPER_DIR`-pinned invocation.

## Outcome — measurement complete, declaration not attempted

**Pre-registration.** [`spikes/target-profiles/metal-thread-execution-width/PROTOCOL-2026-08-22-standard-profile.md`](../spikes/target-profiles/metal-thread-execution-width/PROTOCOL-2026-08-22-standard-profile.md) names `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` as beneficiary and pre-names the `Apple M4 Max` / macOS 27.0 `26A5416b` / `arm64` / `apple9` execution row. It was committed at `f5a274bafe938b3c3a8df6143db0183b4405d135` **before the harness was run**, so the ordering ADR 0113 component 3 requires is provable from history rather than asserted.

**Measurement.** Retained at `spikes/target-profiles/metal-thread-execution-width/results/2026-08-22-apple-m4-max-macos27.0-26A5416b/widths.json`, SHA-256 `12fe14ebecb64c013d26d680817803fe32c4e4e1c47252307550882e586ba4bf`. **31 of 34 frozen identities prepared; 93 retained widths; every one 32**; `all_prepared_widths_equal` true; zero variance across kernel, arithmetic, control flow, threadgroup shape, memory, register pressure, and compiler selection. The three optional compile failures are the language's, not the host's, and match the first record exactly.

**The harness was not edited, and that is checked rather than claimed.** `validate` recomputes `harness_source_sha256` from the tree, so any edit to `src/*.rs`, `kernels/*.metal`, or `Cargo.lock` would break the retained 2026-08-13 record. Both records carry `a918c8e423ccb85f89334ed2f397efc926d89f0622d4ea676cdb44d48bb8ba38` and both report `validation passed` at this base.

**Extent of the claim.** A fact about the `26A5416b` row only — not `26A5406e`, not `26A5388g`, not any other M4 Max, not the Apple9 family. Agreement with the M3 Pro record is a comparison, not a widening; what it does establish is that ADR 0113 component 5's in-family contradiction path is **not** triggered.

**Not done, deliberately, and why.**

- **No `crates/` change.** Declaring the row through `BoundMetalSubgroupDeclaration` is `implementation/build`, outside this ticket's scopes, and it moves the flagship profile's stated content, descriptor, and every dependent pin — a Tom-facing packet under ADR 0113 and the readiness gate. This ticket's own text scopes only the measurable half here.
- **No ledger row or third environment table.** `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` is in scope and was deliberately left unchanged: its per-population environment tables exist to say which record sources which row, and this record sources none yet. Adding a table now would either imply a row that has not been accepted or mirror the spike's statement into a second drift-prone authority over one fact — the failure mode ADR 0113 eliminated by name. The ledger gains its third population when the declaration packet lands.

**What remains for the close condition.** The Tom-facing decision packet adding the subgroup fact family to `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`, carrying: the declaring-module change and its `UnevidencedWidth`-pattern refusals, the descriptor and pin recomputation, and the ledger's third population. The evidence half is now complete and blocks nothing.

## Trigger check log

- 2026-08-18 — **not fired.** `ssh m3` is an M3 Pro; the coordination M4 Max reports `sw_vers -buildVersion` → `26A5406e`, not the ledger's `26A5388g`. Reproduce: `sw_vers -buildVersion` and `sysctl -n machdep.cpu.brand_string` on the candidate host, compared against the ledger's execution table. *(That comparison was the trigger under the two-branch premise corrected above; it is no longer the condition this ticket waits on — see the 2026-08-19 entry.)*
- 2026-08-19 — **fired, on the decision rather than on the host.** Tom accepted the host-evidence composition model ([ADR 0113](../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md)), which supersedes both branches this ticket was waiting between: the measurement no longer needs a host at the ledger's exact build, and the profile is not re-rowed. The measurable half is now unblocked on the current host under the standing measurement authorization. This ticket stays `deferred` rather than moving itself: its close condition still needs the Tom-facing fact-family packet that adds a subgroup row to the flagship public profile, and ticket state changes belong to the coordinator, not to the sweep that repaired the premise. Reproduce: `grep -c "pre-registered beneficiary, stated value, measured validity" docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md` (nonzero — component 3 is recorded accepted), and `sw_vers -buildVersion` plus `sysctl -n machdep.cpu.brand_string` on the intended host to fix the environment the new protocol will pre-name (this host, 2026-08-19: `26A5406e`, `Apple M4 Max`).
