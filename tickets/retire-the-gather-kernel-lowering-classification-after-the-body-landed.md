---
id: retire-the-gather-kernel-lowering-classification-after-the-body-landed
title: Retire the gather kernel-lowering classification after the body landed
status: in-progress
priority: p1
dependencies: []
related: [emit-the-indirect-gather-on-metal, lower-the-indirect-gather-read-through-the-structured-kernel-body]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, gather, compiler]
claimed_from: todo
assignee: worker-retireclass
lease_expires_at: 1787480886
---
## User-visible outcome

The compiler stops classifying a kernel-lowering refusal it can no longer take, and the vacuously proved gather fixture asserts the wall it actually reaches instead of one that has been retired.

## Why this exists

Filed 2026-08-23 by `worker-gatherbody` from [`lower-the-indirect-gather-read-through-the-structured-kernel-body`](lower-the-indirect-gather-read-through-the-structured-kernel-body.md), which landed the kernel body and therefore retired the refusal that lane's classifier was written for. The parent ticket asked for this removal in the same landing; it could not be done there, because `implementation/compiler` was held by a live exclusive claim (`re-derive-the-contraction-fusion-role-rationale-after-the-key-replacement`, `worker-fusionrole`) and the parent's brief declared `tiler-compiler` a non-goal. Splitting it is the boundary that keeps the IR landing coherent rather than half-merged.

**Fact — the workspace gate is red on exactly one test until this lands.** Measured on the parent lane's branch: `cargo nextest run --workspace` reports `4064 tests run: 4063 passed, 1 failed`, and the failure is `tiler-compiler request::tests::a_statically_proved_gather_is_declined_for_its_missing_kernel_body`, reporting `left: None  right: Some(("kernel-lowering", "gather-kernel-body"))`. `planning_capability_rule` answers `None` because the refusal is no longer a kernel-lowering one.

**Fact — the wall the fixture now reaches is one layer further down, and it is a different crate's.** Probed on the parent's branch by making the same test print its refusal: `InvalidCompilerOutput(Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 })))`. The kernel declares its address operand at `KernelType::U32`, and `crates/tiler-compiler/src/program.rs`'s `BoundedCarrier::of` materializes every boundary value at the *program's* arithmetic carrier — `ArithmeticType::F32` yields `KernelType::F32` — so the U32 index input is declared as `f32` and `KernelProgramBuilder` refuses the stage. `tiler_ir::program`'s `StorageScalar::U32` already exists and its own documentation names `KernelType::U32` as its natural access type, so the missing half is the compiler's per-input carrier selection, not an IR carrier.

## Required work

- Re-audit every Fact above at your own base before editing.
- Remove `kernel_lowering_failure`'s gather arm and `GATHER_KERNEL_BODY_RULE` from `crates/tiler-compiler/src/pipeline/planning.rs`. The classifier never took a refusal of its own, so nothing changes class except the case that no longer occurs; check whether the function still earns its existence once the arm is gone.
- Decide what `a_statically_proved_gather_is_declined_for_its_missing_kernel_body` should assert now, and say which of the two it is: the fixture either pins the *new* wall by name — which requires deciding whether `InvalidCompilerOutput` is the truthful class for a U32 operand the compiler declines to materialize, or whether that too is a missing capability — or it is deleted because a differently owned ticket pins that wall. Do not leave it asserting a retired rule, and do not rename the assertion to whatever the run prints.
- Route a program input's storage carrier from its own resolved value type rather than from the program's arithmetic type, so a `tiler::u32@1` index operand materializes at `StorageScalar::U32` / `KernelType::U32`. If that turns out to be a public-boundary or identity question rather than a local fix, stop, split it, and say so.
- Perturb each behaviour on its own subject and quote the failure text.

## Non-goals

The kernel body, which landed. The Metal emission, which is [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) and refuses `KernelType::U32` at `msl_type` today.

## Coordinator graph correction — 2026-08-23: this is a co-landing constraint, not a sequencing one

This ticket was filed with `depends_on: [lower-the-indirect-gather-read-through-the-structured-kernel-body]`. **That edge was semantically wrong and has been changed to `related`.**

`depends_on` asserts *that must land first*. These two must land **together**, and neither ordering is buildable alone:

- The body alone turns `main` red — `a_statically_proved_gather_is_declined_for_its_missing_kernel_body` asserts a classification whose arm the body makes unreachable.
- The retirement alone would make `main` stop classifying a gather it still cannot lower, replacing a correct report with an absent one.

The `depends_on` edge also made the work **unreachable**: `tkt claim` refused it with `has unfinished dependencies`, because the body ticket sits in `review` — complete work, held for integration — and `review` is not terminal. The body cannot reach `done` until it merges, and it must not merge until this lands. A dependency edge cannot express that; it expresses the opposite.

**So the constraint is recorded where it belongs — in prose and in the branch layout — rather than as an edge that misstates it.** This lane's worktree is branched from `7d1219ec`, the gather-body branch merged up to `main`, so the body is present in its tree and the classifier's arm is genuinely unreachable there. The combined result merges into `main` **once**.

I did **not** reach for `tkt claim --force`: that flag steals a live lease from another agent and has nothing to do with dependency ordering. Using it here would have been misusing a tool to silence a check that was correctly describing a real problem with my own graph edge.
