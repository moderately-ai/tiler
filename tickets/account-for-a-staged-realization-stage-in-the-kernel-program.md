---
id: account-for-a-staged-realization-stage-in-the-kernel-program
title: Account for a staged realization stage in the kernel program
status: todo
priority: p1
dependencies: []
related: [admit-a-scheduled-region-for-a-staged-elementary-family, admit-the-registered-elementary-families-as-recognizable-program-stages, accept-the-root-mean-square-scale-realization-law]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, program, identity-domain]
---
## User-visible outcome

A program whose middle stage is a registered elementary family assembles into a kernel program and compiles end to end, instead of stopping at `program-assembly/realization-stage-unaccounted`. Every layer below this one already works: [`admit-a-scheduled-region-for-a-staged-elementary-family`](admit-a-scheduled-region-for-a-staged-elementary-family.md) landed the scheduled regions for both stages, they verify through the ordinary checked path, they bind their request subject, a complete plan selects them, and their interpreted result agrees with `tiler-reference` bit for bit. What is missing is one program-scope *declaration*.

## Where the wall is, verified on the landing commit

**Fact — the cover assembles and the extents agree.** `CoverAssembly::from_plan` derives the materialization edge for the handed value, and the check the deriving ticket flagged as the first to test — `materialized-extent-disagreement` — passes rather than refusing: the producing stage's iteration shape *is* the handed value's shape (one element per folded row, `[2]` for a `[2, 2]` occurrence reduced on axis one), and `graph.value_element_count` of the synthetic value is the same 2. Nothing about the extent needed weakening.

**Fact — the refusal is coverage, not extent.** `tiler_ir::program`'s `verify_partial_reductions` refuses a stage whose coverage is empty under `KernelProgramDiagnostic::UncoveringStage` unless a declaration accounts for it, and it admits exactly two accounts: the **combiner of a declared `PartialReduction`** and the **publisher of a declared `PublishingCopy`** (`crates/tiler-ir/src/program/verify.rs:415-427`). Coverage is keyed on `SemanticOccurrence` and refuses one occurrence twice, so `crate::program::covered` projects a stage's atoms down to its *first*-stage ones — a later stage of a multi-region realization therefore claims nothing at program scope.

**Fact — a staged realization's later stage is a third shape and fits neither account.** It is its own cover region, so it is neither pass of a split nor either dispatch of a copy. It cannot be *declared* as a split either, and the arithmetic says so rather than the naming: `verify_partial_reductions` requires `partial_elements == result_elements * partitions`, which for the shipped instance is `2 == 4 * partitions` and has no solution. It is also not one semantically — nothing is partitioned and no partial is combined.

**Fact — the refusal is currently the compiler's, by name and at the right class.** `CoverAssembly::from_plan` detects the unaccounted stage before the IR does and returns `AssemblyRefusal::missing(region, "realization-stage-unaccounted")`, whose `MissingCapability` class reaches a caller as `UnsupportedCapability { phase: "program-assembly", rule: "realization-stage-unaccounted" }` with the region named and the whole explain trace attached. That is the gated boundary this ticket removes, not a defect to repair.

## The surface this touches, and the identity step in it

- **`tiler_ir::program` gains a third program-scope declaration** — a staged-realization contract naming the producing stage, the consuming stage, the handed value, and the occurrence they jointly realize — with its builder entry, its duplicate rule, and its own verification obligations. The obligations are the ones no single stage can see, in the shape the two existing declarations state theirs: the handed value's unique definer is the producing stage, the consuming stage reads it, the value is a `Temporary`, and the chain of claims realizes the occurrence's stages in order without skipping or repeating one. `crate::region::chain_realizes_subject` already states that last obligation for the compiler's side and is the derivation to reuse rather than re-derive.
- **`PROGRAM_DOMAIN` steps from `tiler.kernel-program.v10`**, and the reasoning is `v10`'s own, recorded at `crates/tiler-ir/src/program/model.rs:1643-1660`: a new declaration section is encoded unconditionally, so every program's bytes move and a cache or artifact holding a `v10` identity must miss rather than match. An appended *conditional* section — written only by programs that declare one — was considered and rejected there on grammar-determinacy grounds, and that argument is unchanged. **This is not an appends-only step**, which is why it is a ticket of its own rather than part of the landing above.
- **Everything that folds program identity moves with it**: artifact identity and its pins, cache subjects, and every pinned program identity in the workspace. The artifact codec needs a field for the new declaration if an artifact program carries it, which is a `MANIFEST_SCHEMA` step of its own. Enumerate every moved pin whole in the landing commit with its ledger paragraph.
- **`tiler-compiler`'s `CoverAssembly`** emits the declaration where it currently refuses, and the refusal and its rule are removed rather than left unreachable.
- **The public surface is an accepted-boundary sibling.** `PublishingCopy` is marked *accepted boundary* at its definition; a third declaration beside it is a public IR surface and needs its own acceptance node parked for Tom, on the standing convention.

## What lands with it

`pipeline::tests::a_staged_family_program_spells_both_stages_and_names_the_program_scope_wall` becomes an end-to-end compile: the same program, driven through `compile()`, dispatching three kernels and agreeing with `tiler-reference` bit for bit. `the_staged_regions_compute_the_normalization_bit_for_bit` is the region-level measurement it supersedes and should be kept beside it rather than replaced — the two fail for different reasons.

## Non-goals

The scheduled-region vocabulary (landed). A staged family that *reads* a materialized intermediate ([`admit-a-staged-family-that-reads-a-materialized-intermediate`](admit-a-staged-family-that-reads-a-materialized-intermediate.md)). A parallel split of a fold that carries an epilogue, which is refused by the schedule verifier and is its own widening. The softmax, which has no registered law.

## Closes when

A program with a registered elementary family as a middle stage compiles through the ordinary path and agrees with `tiler-reference` bit for bit, the program-scope declaration is a labelled draft with an acceptance node, the domain step is recorded with its full reasoning, and every moved identity pin is recomputed on the landing tree and enumerated.
