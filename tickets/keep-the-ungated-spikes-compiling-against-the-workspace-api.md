---
id: keep-the-ungated-spikes-compiling-against-the-workspace-api
title: Keep the ungated spikes compiling against the workspace API
status: done
priority: p2
dependencies: []
related: [package-the-admitted-live-schedule-into-a-symbolic-kernel-program, keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud]
scopes: [research/target-profiles, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [spikes, gates, correctness]
---
## User-visible outcome

The retained spikes that consume the workspace's public API still compile, so a spike a document cites as reproducible evidence can actually be run — and a future workspace API change that breaks one is discovered deliberately rather than by someone trying to reproduce a measurement months later.

## Why this exists

Filed 2026-08-19 by the coordinator at `8d2619e5`, immediately on the symbolic-packaging carrier reporting it as an unsupported case. Verified first-hand before filing.

**Fact — the carrier changed a public accessor's name and type.** `crates/tiler-artifact/src/program/codec/view.rs` replaced `pub const fn shape(self) -> &'a Shape` with `pub fn static_shape(self) -> Option<Shape>` on the decoded-input view, at two sites. The change is correct and forced: under the accepted `v13`/`v21` step an interface axis may be symbolic, so a decoded input no longer *has* a static shape unconditionally, and returning one would be the silently-wrong answer the whole step exists to prevent.

**Fact — three spikes call `.shape()` on a decoded input, and `spikes/` is not built by anything.** `spikes/` is absent from `Cargo.toml`'s `members`, and no `Makefile` target names it, so `make full` cannot see the breakage. The three call sites at this base:

| file | note |
| --- | --- |
| `spikes/runtime/backend-provider-portfolio/src/cpu.rs`, anchor `.bind_input_shape(input.key(), input.shape())` | newly broken |
| `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs`, three call sites including `let [rows, columns] = input.shape().extents()` | newly broken |
| `spikes/target-profiles/metal-subgroup-width-route-gate/src/route.rs`, anchor `.bind_input_shape(input.key(), input.shape())` | **already** permanently broken by ADR 0113 and carrying its own `compile_error!` — not this ticket's subject beyond confirming it stays loud for its own stated reason |

**Fact — this is the same defect class the route-gate ticket just closed, one level out.** `keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud` landed a guard for a spike sharing a *file* with a gated test. That guard cannot see this: these spikes break through the ordinary public API, not through a `#[path]` share. AGENTS.md deliberately keeps spikes out of the gate — "run them manually from documented commands so exploratory dependencies do not silently become repository gates" — so the ungated state is a decision, not an oversight. What is missing is any signal at all when a workspace change invalidates one.

## Correction — 2026-08-22, worker at base `3cca5438`

**The two Facts above are verified but the second is materially incomplete, and acting on it alone would have left both spikes broken.** Its table is exactly right: `.shape()` appears at five call sites in the three named files, and `spikes/` is absent from `Cargo.toml`'s `members` with no `Makefile` target naming it. What it does not say is that the accessor change is **one of six API changes, across four independent landings**, that broke these spikes. The table was derived from a grep for `.shape()`, and a grep cannot see a missing argument, a widened enum, a new required trait method, a changed closure signature, or a run-time contract refusal. A clean `cargo check` reports five errors in `scalar-cpu-vertical` and nine in `backend-provider-portfolio`. Landings are attributed with `git log -S` against each changed signature; the three ADR 0013 rows are one commit, not three.

The complete population at this base:

| Landing | Change | Where | Visible to |
| --- | --- | --- | --- |
| `79dc05a1` | `shape` → `static_shape` on the decoded-input view | 5 sites, 3 files (the table above) | `cargo check` |
| `c77aab39` | `push_carried_payload` gained `Option<TargetEnvironmentDeclaration>` | 4 sites across both spikes | `cargo check` |
| `c77aab39` | `assemble_plan_artifact` gained `PlanDeterminismDeclaration` | `backend-provider-portfolio/src/cpu.rs` | `cargo check` |
| `c77aab39` | `RuntimeAdapter` gained `target_environment_support` + `observe_target_environment` | both adapters in `backend-provider-portfolio` | `cargo check` |
| `bc0b7c0e` | `KernelType` gained `U32` | 2 non-exhaustive matches | `cargo check` |
| the strict `f32` contract's dimension set | requires `ReciprocalTransform`, `ApproximateIntrinsics`, `MaterializationRounding` | `scalar-cpu-vertical/src/profile.rs` | **running the spike only** |

The last row is the one that matters for the visibility question this ticket asks, and it was found only because the spike was run rather than checked: `declare_numerics` omitted three dimensions, an omitted dimension is `Unknown`, and the contract check refuses `Unknown` rather than reading it as a denial. The spike compiled cleanly and exited non-zero with `TargetNumericalContractRefusal`.

**Consequence for "Closes when".** "Both newly broken spikes compile" is too weak a closing condition for this ticket's own outcome, which is that a cited spike "can actually be run". Both spikes are now repaired to the point of a completed run against `tiler-reference`, not merely to the point of compiling.

## Required work

- Re-audit the Facts above at your actual base and report a per-Fact verdict; re-derive the call-site list rather than trusting the table, since a sibling lane may have touched these files.
- Repair the two newly broken spikes against the current accessor. `static_shape()` returns `Option`, so each site must state what it does when the shape is not static — **do not `unwrap()` to restore compilation**. A spike that panics where the API now admits a symbolic case is worse than one that fails to compile, because it will be run and believed.
- Decide and record how ungated spike breakage becomes visible in future. Candidates worth costing rather than assuming: a manually-run `make spikes` target that is documented but not gated; a census test in the workspace asserting the retained spikes' call sites still name existing symbols; or an explicit recorded decision that spikes are repaired on demand and their documents say so. **State what each option costs and what it would let through** — AGENTS.md's position that spikes must not become repository gates is a constraint on the answer, not an argument against having one.
- Whatever you choose, perturb it: show the signal firing on a deliberately broken spike, and quote it. If you choose the "repair on demand" option, then the deliverable is the recorded decision plus the documents saying so, and there is no check to perturb — say that explicitly rather than leaving the obligation unmet.

## Non-goals

Adding `spikes/` to the workspace `members` or to `make full` — that is exactly what AGENTS.md forbids, and doing it would turn exploratory dependencies into repository gates. Repairing the route-gate spike's ADR 0113 breakage, which is permanent and deliberate. Any change to `crates/`.

## Closes when

Both newly broken spikes compile and handle the non-static case honestly rather than by unwrapping, the visibility decision is recorded with its cost and what it admits, any check it introduces has been perturbed with its output quoted, and the route-gate spike is confirmed still failing loudly for its own reason.
