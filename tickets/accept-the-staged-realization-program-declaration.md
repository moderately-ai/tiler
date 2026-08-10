---
id: accept-the-staged-realization-program-declaration
title: Accept the staged-realization program declaration
status: done
priority: p2
dependencies: []
related: [account-for-a-staged-realization-stage-in-the-kernel-program, admit-a-scheduled-region-for-a-staged-elementary-family, accept-the-fold-with-epilogue-scheduled-region]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, program, identity-domain]
---
## What is being accepted

The third program-scope declaration of `tiler_ir::program`, landed as a labelled draft by [`account-for-a-staged-realization-stage-in-the-kernel-program`](account-for-a-staged-realization-stage-in-the-kernel-program.md). It is implemented and tested; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it. Only Tom closes it.

## The exact surface

New in `tiler_ir::program`:

```rust
pub struct StagedRealization {
    pub producer: StageId,
    pub consumer: StageId,
    pub handed: MaterializedValueId,
    pub occurrence: SemanticOccurrence,
}

impl KernelProgramBuilder {
    pub fn push_staged_realization(
        &mut self,
        realization: StagedRealization,
    ) -> Result<(), KernelProgramBuildError>;
}

impl VerifiedKernelProgram {
    pub fn staged_realizations(&self) -> impl ExactSizeIterator<Item = StagedRealizationRef<'_>>;
}

pub struct StagedRealizationRef<'a> { /* producer(), consumer(), handed(), occurrence() */ }

pub const MAX_PROGRAM_STAGED_REALIZATIONS: usize = 4_096;
```

Insertion returns one new `KernelProgramBuildError` variant (`DuplicateStagedRealization`) and reuses two existing ones (`SelfDependency`, `CoverageOutOfRange`). Four new `KernelProgramDiagnostic` variants — `HandedValueNotInitializedByProducer`, `HandedValueNotReadByConsumer`, `HandedValueNotMaterialized`, `StagedRealizationChainBroken` — and one new `ProgramLimitKind::StagedRealizations`. `KernelProgramDiagnostic::UncoveringStage` gains a third admitting account; its spelling and rule identifier are unchanged.

Nothing else in the public surface moves. `PartialReduction` and `PublishingCopy` keep their shapes, their obligations, and their encodings.

## Why it exists

A registered elementary family whose `IndexRealizationLaw` realizes a region *sequence* is computed by several dispatches. `tiler::rms-norm-f32@1` is the shipped instance: the producing stage folds `x²` over the reduced axis and applies `Rsqrt(a / N + eps)` to the fold's value, and the consuming stage reads both operands and that handed value at its kept coordinates. Whole-program coverage is keyed on `SemanticOccurrence` and refuses one occurrence twice, so only the first dispatch claims it, and the second computes no operation the coverage names. `UncoveringStage` refuses such a dispatch unless a declaration explains it, and the two existing accounts do not fit: it is one cover region of its own, so it is neither pass of a split nor either dispatch of a copy. The arithmetic agrees with the naming — `verify_partial_reductions` requires `partial_elements == result_elements * partitions`, which for the shipped instance is `2 == 4 * partitions` and has no solution.

## The choices worth objecting to

- **A third declaration rather than a widening of either sibling.** A split carries `contributors_per_partition`, which a staged realization would have to invent; a copy requires its two extents to agree, which is exactly what a staged realization must not require — the shipped instance hands a `[2]` fold on to a `[2, 2]` pass, so an extent rule refuses the very case the declaration exists for. **The named consequence, and the thing to object to if it is wrong:** there are now three declaration vocabularies at program scope and a fourth shape would be a fourth. The alternative was one `StageAccount` enum over all three; it was rejected because each carries obligations the others do not, and a shared record would either make every field optional or erase which obligation a program actually owes.
- **The declaration names the occurrence.** Which occurrence a chain continues is recorded nowhere else at program scope, because coverage records only an occurrence's first stage — so without it, two chains through the same stages are indistinguishable and no chain obligation is statable. The cost is that the compiler must project its own `SemanticMemberId` onto the IR's `SemanticOccurrence`, which it does through the same `OccurrenceLowering` that mints the coverage records.
- **The chain obligation is decided over declared edges, not stage ordinals.** `crate::region::chain_realizes_subject` decides the identical rule for the compiler over stage-carrying attribution atoms, and the compiler runs it before deriving these declarations. Program scope has no stage ordinals — `SemanticOccurrence` is a bare ordinal — so the IR walks the declarations of one occurrence from the stage that *covers* it and requires every row to lie on that path. The cost is two spellings of one rule at two layers; the alternative was carrying a stage ordinal into `CoveredOccurrence`, which would put a planner coordinate into program identity.
- **The duplicate key is the (consumer, occurrence) pair, not the consumer.** One fused dispatch may legitimately continue several occurrences' realizations at once, so keying on the stage alone would refuse a legal program. Two rows for one pair have no reading: they name two handed values for one stage boundary.
- **`HandedValueNotMaterialized` is stated and unreachable.** No program can present a handed value that is neither a temporary nor an externally bound input: an input is refused a writer by `ExternalValueWritten` two phases earlier, and `ValueRole::Output` fills only `TensorRole::Output`, which is a write, so no stage can read an output-role value and the read obligation above it always fires first. It is stated for the reason `PartialNotMaterialized` is — the declaration owes the obligation whether or not today's role vocabulary can spell a violation — and the asymmetry with the other three rows is recorded rather than hidden.

## The identity step it rode

`PROGRAM_DOMAIN` moved `tiler.kernel-program.v10` to `v11`, on `v10`'s own recorded reasoning: a new declaration section encoded unconditionally, so every program's bytes move and a cache or artifact holding a `v10` identity must miss rather than match. Nothing below or beside it stepped. `docs/artifact-abi.md` carries the ledger and the manifest schema's non-step derivation.

## Evidence

Landed with five tests in `crates/tiler-ir/src/program/tests.rs` — `an_uncovering_stage_is_admitted_as_a_declared_staged_realizations_consumer`, `a_declared_staged_realization_changes_program_identity`, `the_staged_realization_row_obligations_can_each_say_no`, `a_staged_realization_chain_must_start_where_its_occurrence_is_covered`, and `a_malformed_staged_realization_declaration_is_rejected_at_insertion` — and the end-to-end compile in `crates/tiler-compiler/src/pipeline/tests.rs`, each watched failing under a named deliberate perturbation: the account arm, the chain walk, the identity section, and the compiler's emission were each removed in turn and the naming test failed.

`pipeline::tests::a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit` compiles `rms_norm(value, weight) * value` over `[2, 2]` through `compile()`, dispatches its three kernels through the structured-kernel interpreter in the program's own execution order, and agrees with `tiler-reference`'s evaluation of the same semantic program bit for bit.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the declaration is in use from `tiler-compiler` and labelled a draft at its definition.

## Outcome — accepted

**Accepted by Tom on 2026-08-06, as-is with no exclusion, at the live session's decision round (presented by the orchestrator, explain-then-recommend, relay source this ticket).** The declaration, its builder route, view, limit, and diagnostics are accepted public surface exactly as landed, three-vocabulary consequence included. The in-code draft label rewrites (the `StagedRealization` struct, `staged_realizations()`, and `StagedRealizationRef`) ride with the identity-campaign branch, which holds `implementation/ir`.

## Current-state correction — 2026-08-09

The routed source-label sweep landed: `StagedRealization`,
`push_staged_realization`, `staged_realizations()`, and
`StagedRealizationRef` all state the accepted boundary. The IR contract's live
program-scope paragraph is corrected with this record from proposal/parked to
accepted. No public item, diagnostic, identity domain, or behavior changes in
this documentation reconciliation.
