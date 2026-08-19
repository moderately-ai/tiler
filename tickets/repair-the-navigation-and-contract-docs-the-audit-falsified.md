---
id: repair-the-navigation-and-contract-docs-the-audit-falsified
title: Repair the navigation and contract docs the audit falsified
status: in-progress
priority: p2
dependencies: []
related: [repair-the-accepted-decision-records-the-splits-and-retirements-falsified, repair-the-research-records-the-key-replacement-and-splits-falsified, repair-the-ticket-population-facts-the-splits-and-retirements-falsified]
scopes: [contracts/navigation, contracts/artifacts, contracts/foundation, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, audit]
claimed_from: todo
assignee: worker-nav
lease_expires_at: 1787162948
---
## User-visible outcome

The entry-point and contract documents agree with the tree and with each other on the validated-overlap count, the retired Metal adapter, the retired contraction key, and the live vocabulary sizes. A reader who arrives at `docs/status.md` or `docs/backends/metal.md` is not given a number three other documents contradict.

## Why this exists

Filed 2026-08-19 from the post-chain multi-lens audit and re-verified site by site by the coordinator at `de18ebdb`. Every Fact below was read at that base.

**Fact — the validated-overlap count disagrees three ways across live documents, and none of the two smaller numbers is right.** The authority is the compile-profile authority ledger, anchor `the validated overlaps, revised at the compilation-selection carrier`, which states four: compiler buffer capacity against the emission limit, the F32 subnormal projection, `declare_metal_bf16_subnormal_behaviour`, and the per-population compilation-selection equality that refuses as `CompilationSelectionMismatch` by population name. Two live documents disagree:

- `docs/backends/metal.md`, anchor `Exactly two overlaps are validated` — says two. This sits in the **same paragraph** that goes on to describe the per-population selection check, so the paragraph contradicts itself within a few sentences.
- `docs/status.md`, anchor `are the three validated overlaps` — says three, and never received the compilation-selection chain at all.

The ledger is out of this ticket's scopes (`research/target-profiles`) and needs no repair; read it as the authority and do not edit it.

**Fact — `docs/backends/metal.md` asserts an Implemented public bound that was retired, and refutes itself in the same file.** Anchor `Implemented bound — caller-vouched F32 subnormal projection` claims a public `tiler-build` adapter with caller-vouched contexts which "does not … populate quantitative or F32-dispatchability rows, bind a plan/artifact/runtime environment, or project F16/BF16". `declare_metal_f32_subnormal_behaviour` is declared `fn`, not `pub fn` (`crates/tiler-build/src/metal_declaration.rs`, anchor `fn declare_metal_f32_subnormal_behaviour`), and is absent from `crates/tiler-build/src/lib.rs`. The same file already records the retirement at anchor `The public caller-vouched declare_metal_f32_subnormal_behaviour adapter was retired`. The bound declaration that replaced it also calls `declare_metal_bf16_subnormal_behaviour`, `declare_measured_dtype_dispatchability`, and `declare_measured_max_threads_per_grid_axis`, so three of the four "does not" clauses are false as well. **Verify each of the four clauses separately against source** — a repair that corrects the visibility claim and leaves the four-clause list standing has fixed the cheaper half.

**Fact — `docs/glossary.md` states two stale counts of the same enum.** `BinaryOp` (`crates/tiler-ir/src/kernel/model.rs`, anchor `pub enum BinaryOp`) has **twelve** variants — `IndexAdd`, `IndexMultiply`, `IndexDivide`, `IndexModulo`, `IndexSubtract`, `F32Add`, `F32Multiply`, `I32Subtract`, `F32Divide`, `F32Maximum`, `Bf16Add`, `Bf16Multiply` — of which **five** carry an `Index` prefix. The glossary says otherwise at two anchors: `four of \`BinaryOp\`'s six variants carry an \`Index\` prefix` (six total, four prefixed) and `Their four sibling variants are spelled` (which then enumerates only `IndexAdd`, `IndexMultiply`, `IndexDivide`, `IndexModulo` as the complete sibling set). Both sentences' *arguments* — that layer-crossing names need qualified spellings, and that this pair is the only both-implemented both-public shared name — are unaffected by the counts; check them separately rather than rewriting them along with the numbers.

**Fact — `docs/roadmap.md`'s contraction row contradicts its own closing Fact.** Anchors `so does the F32 strict tensor contraction` and `reassociation-permitted: false\` withholds` (both resolve, one hit each). The cell ends with a **Fact** dated 2026-08-19 recording that the row's key moved to the permission-indexed successor under ADR 0112, yet earlier in the same cell asserts the numerical signature states "both order permissions" and repeats reasoning built on a declared `reassociation-permitted: false`. That constant no longer exists anywhere in `crates/` — `grep -rn "CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED" crates/` returns nothing — and `reduction_descriptor_record` (`crates/tiler-ir/src/semantic/contraction.rs`) declares that row `"permission-gated"` instead. The headline naming "the F32 strict tensor contraction" as an executed profile names a program the retired key can no longer produce, as `crates/tiler-compiler/tests/retired_contraction_key_never_compiles.rs` pins. **The executed evidence is real; the family name is retired.** Date the claim rather than deleting it.

**Fact — `docs/correctness-and-testing.md` names the retired key inside a gate's admitted-subject bound**, anchor `a strict serial sum, a strict tensor contraction`. That document is one of ADR 0112's own `applies_to` contracts, so this is the acceptance sweep having missed a contract it named.

**Fact — three documents in these scopes cite deleted module paths.** `crates/tiler-artifact/src/program/tests.rs` does not exist (now the directory `program/tests/`): cited three times in `docs/artifact-abi.md` and once in `docs/dtype-support.md`. `crates/tiler-compiler/src/request.rs` **still exists** as the spine beside its new `request/` submodules, so a citation to that path is not automatically stale — but a symbol it names may have moved. `docs/status.md`, `docs/roadmap.md`, `docs/dtype-support.md`, `docs/numerical-semantics.md`, `docs/open-questions.md`, `docs/correctness-and-testing.md`, and `docs/compiler/optimizer.md` all cite it; treat path and symbol separately and repair only the symbols that moved.

## Required work

- Re-audit every Fact above at your actual base and report a per-Fact verdict before editing. Re-derive the counts by reading the enum bodies.
- Repair each site with a dated correction in the file's own convention, preserving the surviving argument. Keep retired wording quoted inside a correction on one source line so sibling anchors resolve into the note.
- Reconcile the overlap count against the ledger as authority and state the four overlaps by name, so the next divergence is visible rather than a bare number.
- Census the deleted-path citations across these scopes with `grep -rlF` and quote the counts. For every path repair, locate the named **symbol** in the split modules; do not repoint at a directory rename alone.
- Where a document's claim is a count of a live vocabulary, say which enum it counts so a later reader can re-derive it.

## Non-goals

`docs/decisions/**`, `docs/research/**`, ticket bodies, and the compile-profile authority ledger — each is another ticket's scope or already correct. No source change.

## Closes when

Every site above is repaired or verified already-correct with evidence, the overlap count reads four in all three documents that state it, the deleted-path census is quoted with counts, `make citations` is green, and no document in these scopes states a vocabulary size or public-surface claim this tree contradicts.
