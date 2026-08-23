---
id: close-the-refutable-bindings-that-survive-the-physical-planner-build-trap
title: Close the refutable bindings that survive the physical planner build trap
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, exhaustiveness, maintainability]
claimed_from: todo
assignee: worker-refutable
lease_expires_at: 1787491945
---
## User-visible outcome

The refutable bindings over `RegionProgram` and `ScalarProgram` in the physical planner either name what a new variant means or record why their implicit arm is correct, so the build trap those enums document is not defeated by a second construct after the wildcards were closed.

## Why this exists

Filed 2026-08-23 by the coordinator from the residual `worker-buildtrap` reported at the close of [`restore-the-build-trap-the-physical-planner-wildcards-defeat`](restore-the-build-trap-the-physical-planner-wildcards-defeat.md), which landed as `354f4086`. That lane closed every `_ =>` arm and every `matches!` over those two enums — a span-aware scan of `physical.rs` now finds **0** of either. It then reported this residual and **deliberately left it**, on the ground that a refutable binding is not a wildcard and the ticket's Closes-when did not reach it. That scope judgement was correct, which is why this is a separate ticket rather than a gap.

**Fact — reported by that lane, NOT verified by the coordinator.** Eight refutable `let … else` / `if let` bindings over these enums survive in `crates/tiler-compiler/src/physical.rs`. Three are production and five are `#[cfg(test)]` fixture setup. **Re-derive the whole population yourself before acting.**

**Use a span-aware scan, not `grep`.** This is not optional advice — the coordinator's own line-oriented census of the sibling construct returned **0** where a span-aware scan returned **6**, because every one of those `matches!` wrapped. `let … else` and `if let` wrap the same way. A line-scoped count here is a floor, not a population, and its silence is indistinguishable from absence.

**The three production sites, as reported.** At `4384`, a `RegionProgram::Numerical` binding whose `else` refuses under `request-binding` — the gate every subject binding passes through. At `4666` and `4710`, `ScalarProgram::PointwiseF32` bindings whose `else` **silently delegates to the producer's subject**. That last disposition is the one to look at hardest: for a new variant it is a decision nobody made, and it is not obviously fail-closed the way a refusal is.

## Required work

- Re-derive the population with a span-aware scan and report it, stating the method. Give a per-site verdict distinguishing production from `#[cfg(test)]`.
- **Decide by reading, per site, between three outcomes**: convert the binding so a new variant is a build error; record why the implicit arm is correct there, with a reconsideration trigger tied to either enum gaining a variant; or — where the `else` silently delegates rather than refuses — establish what a new variant would actually do and whether that is fail-closed. **An `else` that delegates is not the same as an `else` that refuses**, and the two should not be given the same treatment by default.
- The lane notes `4384` is two arms and "would be nearly free". Confirm that before relying on it.
- **Preserve every current disposition.** This is an exhaustiveness restoration, not a behaviour change; if making a site total would change what any existing variant does, stop and report.
- Perturb by adding a probe variant to each enum and quote the failures, with the base-tree negative control showing which sites were silent before. `tiler-ir`'s in-crate total maps must be patched for the consumer to be reached at all — the sibling lane needed 8 patches for `RegionProgram` and 16 for `ScalarProgram`. Revert fully and confirm with `git grep` that no probe name survives.

## Non-goals

The `_ =>` arms and `matches!` invocations already closed by `354f4086`. Adding `#[non_exhaustive]` to either enum — their own docs forbid it, and that is the point of the trap. Changing any refusal's class or behaviour. `crates/tiler-ir/`.

## Closes when

Every refutable binding over `RegionProgram` or `ScalarProgram` in the physical planner either fails the build on a new variant or carries a recorded reason with a reconsideration trigger, the delegating `else` sites have a stated answer for what a new variant does, no disposition has changed, and the perturbation is watched failing with its base-tree control quoted.
