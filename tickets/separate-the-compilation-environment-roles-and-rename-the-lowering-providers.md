---
id: separate-the-compilation-environment-roles-and-rename-the-lowering-providers
title: Separate the compilation-environment roles and rename the lowering providers
status: todo
priority: p1
dependencies: []
related: [package-selected-physical-implementation-provenance-in-artifact-identity, decide-the-artifact-physical-selection-provenance-surface]
scopes: [implementation/artifact, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, refactor, public-boundary, provenance]
---
## User-visible outcome

`CompilationEnvironment` carries an explicit lowering role and an explicit physical role rather than one undifferentiated set, and every existing selected-provider name says *lowering* — so the physical-selection surface can be added next without a rename tangled into an identity step.

## Why this exists

Split out 2026-08-22 by the coordinator when `package-selected-physical-implementation-provenance-in-artifact-identity` **stopped correctly** rather than leave a half-applied identity migration. That worker mapped the full scope at base `e1ada851`: **216 rename occurrences across 56 files**, 72 `CompilationEnvironment::new` sites, 15 coupled `VariantSpec` literals, roughly 1,500 lines of new surface across 11 core files, plus the `tiler-build` bridge, `docs/artifact-abi.md`, and pin/golden/cache-subject recomputation. It got as far as writing new capacity constants and **reverted them**, because landing a role rename half-applied is exactly the incoherent state an identity migration must never be left in. That was the right call.

**This lane is the half that gates on its own.** It performs no identity step, adds no new surface, and steps no domain. The parent keeps the physical-selection run, the `v21` to `v22` step, its pins, and its perturbations, as one coherent migration.

**Coordinator deviation from the delivering worker's proposal, stated so it is not mistaken for an oversight.** That worker proposed three lanes: rename, then physical-row surface, then pins-and-perturbations. I split it into **two**. Pin and golden recomputation cannot be a lane separate from the domain step that invalidates them — AGENTS.md requires identity-domain changes to stay coherent across owning version, ledgers, and pins, and recomputing them in a later lane would publish a tree whose pins disagree with its bytes. The rename genuinely does gate alone; the pins genuinely do not.

## Facts, verified by the coordinator at `2cc3aefa`

**Fact — a bulk rename would silently corrupt two unrelated APIs, and both are out of scope.** `spikes/program-planning/physical-frontier-budget-calibration/src/program.rs` declares `pub selected_providers: usize` at three sites — a spike-local summary counter with no relation to the artifact surface. `spikes/artifacts/artifact_envelope.rs` carries **8** occurrences of the same name in its own standalone model; it is a `rustc --test` file that does not link `tiler-artifact` at all. Both match the rename pattern. **Neither may be touched.**

**Fact — one packet instruction has no subject.** The accepted packet's section 1 requires that "built **and decoded** `selected_providers()` accessors" be renamed. There is no decoded accessor: `crates/tiler-artifact/src/program/codec/view.rs` contains **zero** occurrences of the string `provider`, and no `pub fn …provider` exists anywhere under the codec directory. Only the model-side accessor exists. Rename the one that exists and **record that the packet's population was wrong** rather than inventing the missing one.

**Fact — the superstring ordering is what makes the rename safe.** `SelectedProviders` is the sole extension of `SelectedProvider`, so renaming the longer symbol first makes the remainder unambiguous. Verify this at your base before relying on it; a second extension appearing would invalidate the ordering.

## Required work

- Re-audit all three Facts at your base and report a per-Fact verdict; re-derive the 216/56 census rather than trusting it, and **say which unit you report** — `grep -c` counts lines, not occurrences.
- Apply the rename with a script that **reads each file and writes it back whole**. This environment blocks stream-editor mutation in place, and that block has already prevented one corruption here, so treat it as a guard rather than an obstacle. Exclude both spike populations above by path, explicitly, and print the excluded set so a reviewer can check it.
- Separate `CompilationEnvironment` into explicit lowering and physical roles: required sets, **no union, no default, and no inference from payload, backend, or profile**. A missing member is a typed artifact-build refusal; never substitute the governed provider.
- Validate existing selected *lowering* rows against the lowering set only. Do not add any physical row, accessor, tag, or byte — that is the parent's work.
- **No identity, schema, domain, pin, golden, or cache subject may move in this lane.** Rederive and state that; if one moves, **stop and report** — that would mean the split boundary is wrong.

## Evidence

- Perturb the subject separately for each new refusal and quote the failure text: an absent lowering member, and a cross-role member supplied to the wrong set.
- One negative control that a correct environment still builds and its artifact bytes are **byte-identical** to the pre-rename tree. That control is the whole justification for calling this lane identity-neutral.
- Before trusting any new check, state what it would take for it to say *no*, and confirm that case is reachable.

## Non-goals

The physical-selection run, its encoding, the `v21` to `v22` step, pins, goldens, and cache subjects — all belong to [`package-selected-physical-implementation-provenance-in-artifact-identity`](package-selected-physical-implementation-provenance-in-artifact-identity.md). Touching either spike population. Inventing the decoded accessor the packet names but the tree lacks.

## Closes when

Roles are separated with cross-role and absent-member refusals watched firing, every surviving selected-provider name says *lowering*, both spike populations are demonstrably untouched, artifact bytes are unchanged against the pre-rename tree, and the full repository gate is green.
