---
id: apply-the-accepted-host-evidence-composition-model
title: Apply the accepted host-evidence composition model
status: todo
priority: p2
dependencies: [decide-the-host-evidence-to-profile-composition-model]
related: [reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records, define-host-applicability-for-profiles-whose-rows-span-environments, declare-metal-subgroup-realization-facts-in-the-target-profile]
scopes: [contracts/decisions, contracts/navigation, implementation/build, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, target-profiles, provenance, adr-carrier]
---
## User-visible outcome

The accepted composition model is a citable accepted ADR, and every document whose premise it supersedes says so: the standard-profile subgroup deferral loses its falsely narrow two-branch premise, the demoted spike's README/catalogue pair records the permanent rerun-from-`586c508a` disposition, and the `metal_subgroup_declaration.rs` module doc's "What would license a public host-scoped profile" paragraph states the accepted rule.

## Why this exists — filed 2026-08-19 at acceptance of item 25

Tom accepted model 2b (components 1–6) as packeted in [`decide-the-host-evidence-to-profile-composition-model`](decide-the-host-evidence-to-profile-composition-model.md) (packet `6236feef`, review `427b2080`; acceptance recorded there). The packet names this one sweep carrier as the acceptance's complete follow-up; per the no-fork rule the ADR body lifts components 1–6 verbatim from the packet rather than re-drafting them.

## Required work

- Author the accepted ADR from the packet's six components verbatim (dated acceptance provenance: Tom, 2026-08-19, live session, relayed first-hand by the coordinator), with the packet's consequence set and the reviewer's two in-place repairs carried. Index it in `docs/decisions/README.md` (both index positions, per that file's convention).
- Repair `measure-thread-execution-width-on-the-standard-metal-profiles-own-host` (deferred): its two-branch premise (restore `26A5388g`, or re-row the whole profile) is superseded — under the model the path is a new frozen protocol pre-naming `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` as beneficiary, run on the current host under the standing measurement authorization; update its trigger accordingly with a dated entry.
- Record the permanent disposition on the route-gate spike: README plus the `experiment_status`/catalogue pair (already `blocked` since `1b4c79c3`) restated as the permanent rerun-from-`586c508a` exception; this also releases the held ticket `keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud`, whose fixture-ownership question should be answered in the same sweep or explicitly re-filed.
- Add the module-doc paragraph in `crates/tiler-build/src/metal_subgroup_declaration.rs` beginning from its existing "What would license a public host-scoped profile" hook, stating the accepted licensing rule (production-code doc edit; no behaviour change).
- Every anchor grep-verified; `make citations`, `tkt lint`, `git diff --check`, `tkt guard`; package gates for the tiler-build doc edit.

## Coordination

The `implementation/build` doc edit must not run beside the selection carrier's integration if that merge is still in flight; the coordinator sequences this after that landing. Read the composition ticket's review section for the two repairs that must carry into the ADR text.

## Closes when

The ADR is accepted-indexed with verbatim components and provenance; the deferral premise, spike pair, and module doc are aligned; and the held route-gate ticket is released or re-filed with its release recorded.
