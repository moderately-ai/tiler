---
id: admit-the-sub-tensor-selection-family
title: Admit the sub-tensor selection operation family
status: done
priority: p2
dependencies: []
related: [admit-a-position-selecting-slice-for-the-rotary-table, project-only-the-final-position-logits, admit-live-extent-operands-to-payload-indexing, admit-the-reindex-and-broadcast-operation-families, own-operation-family-support-matrix, reclassify-language-model-work-as-a-conformance-track, decide-the-source-bearing-slice-offset-boundary]
scopes: [contracts/foundation, implementation/ir, implementation/reference, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, operation-families, slice, breadth, class-generic-capability]
---
## User-visible outcome

A program can read a rectangular sub-region of a tensor — an injective, non-surjective coordinate map — as one governed operation family with its own refusals and its own reference evaluator, instead of each workload that needs a selection inventing a narrower one.

## Why this is its own ticket

**Fact.** [The support matrix](../docs/roadmap.md#operation-family-support-matrix) holds sub-tensor selection at R1: no contract defines a slice and no key exists. `tiler::reindex-f32@1` admits bijective permutation, split, merge, unit-axis insertion or removal, and within-axis reversal, and its registered `reindex.split.not-surjective` refusal names a non-surjective map as outside that family; `tiler::broadcast-f32@1` does not admit one either.

**Fact — the family had two consumers and no owner.** Two tickets each need a selection and neither is the family: [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md) carries a decode step's position-identity argument, and [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md) carries a prefill residency argument. Until 2026-08-04 the first also carried the family's design in a "Required design" section and depended on [`integrate-the-autoregressive-decode-loop`](integrate-the-autoregressive-decode-loop.md), so a generic operation family was scheduled behind a complete consumer decode loop. Filed under [`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md), whose classification is that the capability is generic even though both of its triggers are workload observations.

**Fact — a constant-offset form is reachable today and a symbolic-offset form is not.** `IndexNode` at `crates/tiler-ir/src/index/model.rs:94`–`109` has five variants: `Constant`, `Dimension`, `LinearCombination` whose constant and per-term coefficients are `IndexInteger`, and `FloorDiv` and `Modulo` whose `divisor` is a `SourcedExtent`. `SourcedExtent` is the only carrier of a possibly-symbolic extent and it appears in no other position, so `t + k` for a literal `k` is expressible and `t + C` for a bound symbol `C` is not. Reproduce with `grep -n 'enum IndexNode' -A 16 crates/tiler-ir/src/index/model.rs`.

## Required delivery

- **The family's form, decided before it is registered.** A general `Slice`, a bounded offset-and-extent selection along one or more axes, or a strided form — argued against the rule [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) already applied to contraction: one keyed family carrying a canonical structure attribute, or a key per shape class, with a frontend never choosing among keys.
- **Semantic identity and validation.** A governed `OpKey`, a canonical attribute carrying the per-axis selection, and construction-time refusals for a selection that leaves the source's declared extent, an empty selection, a duplicated axis, and an axis out of range — each a typed provider diagnostic in the shape `StrictSerialSumF32::infer` already sets in `crates/tiler-ir/src/semantic/registry.rs`.
- **A normative reference and a reference evaluator** for the exact signature.
- **The symbolic-offset boundary, stated rather than assumed.** Either the constant-offset form is admitted and the symbolic-offset form refuses by name with its own reconsideration trigger, or the `IndexNode` gap above is closed as part of this work. Those are different sizes and this ticket records which it took. [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) owns the physical half of a live extent and is not a substitute for the index-layer gap.
- **A support-matrix row movement**, to the rung the delivered layers actually support, with both existing triggers retained and their evidence attached.

## Non-goals

Any lowering capability, physical schedule, or backend emission; those follow and are separately scheduled. Gather and scatter, which are a different access class under [Q-SHAPE-007](../docs/open-questions.md#q-shape-007--indirect-gatherscatter-relations) and owned by [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md). Deciding which of the two triggers is delivered first — this ticket owes them the family, not an ordering.

## Closes when

The family's form is decided with its ground, a governed key verifies and reference-evaluates a constant-offset selection, every listed refusal is watched firing, the symbolic-offset boundary is explicit, and the support-matrix row moves off R1 with both triggers preserved.

## Outcome — 2026-08-04

**The delivered half is the literal offset.** The symbolic-offset form is refused by name and its `IndexNode` gap is not closed here: closing it is an index-vocabulary change, which this ticket's stop conditions name as the other half rather than as this one's remainder.

**Form: one keyed family carrying a canonical selection structure**, on the rule [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) applied to the contraction, whose three deciding costs transfer intact — a frontend never chooses among keys, a per-class key set grows without bound with each key owing a full registry/reference/conformance/ABI/identity vertical, and generalizing a fixed key later migrates every identity that named it. The structure is a **total per-axis selection**: exactly one entry per operand axis, in axis order, over two relations — `whole-axis` and `window`, a contiguous run from a literal offset. The sparse `(axis, offset, extent)` alternative the ticket's refusal list implied was eliminated on canonical identity: a sparse list has one spelling per *ordering*, so it either gives one program two identities or normalizes a caller's list where this corpus refuses to; it also makes a duplicated axis and an out-of-range axis representable, where the total form makes both unstatable and replaces them with one rule decided against the operand's own rank. Rank is preserved; removing the extent-one axis a single-position selection leaves is a `remove-unit-axis` reindex written after it.

**Key: `tiler::slice-f32@1`**, derived rather than chosen. The corpus already names this family "slice" in *registered normative text* — `tiler::reindex-f32@1`'s definition says a non-surjective mapping "is a slice, a different family", and its `reindex.split.not-surjective` message says "a slice rather than a reindex" — so any other spelling would leave the registry describing one family under two names. `select-f32` was eliminated on collision: the [glossary](../docs/glossary.md) separates four unrelated `Select`s and the support matrix carries `Select` and bit-selecting operations as a *different* R1 row. `sub-tensor-selection-f32` was eliminated as the taxonomy's group label rather than the family's name. One survivor, so this was decided rather than escalated; the diagnostic prefix `slice.` follows the key.

**Nine refusals, each its own code, each watched firing** in `crates/tiler-ir/src/semantic/slice/tests.rs`: `no-restricted-axis`, `empty-window`, `entry-count`, `out-of-bounds` (including a `u64::MAX` offset whose sum saturates rather than wraps), `window-is-whole-axis`, `unadmitted-relation`, `strided-window-unsupported`, `symbolic-offset-unsupported`, `too-many-axes`, `canonical-bound`, and `malformed-attribute` across six subjects. The last two of the ticket's listed four — a duplicated axis and an axis out of range — are unrepresentable under the total form and are reported as an entry-count disagreement instead; that substitution is argued in the module's own documentation rather than assumed. `no-restricted-axis` and `window-is-whole-axis` together make the family's non-surjectivity a *proved* property of every admitted occurrence rather than a claim about it.

**Out-of-bounds is a refusal, never a clamp or a wrap**, adopting the posture the taxonomy's F-24 proposed after recording that the primary authorities diverge and that the two conventions return a different tensor for one program rather than a different diagnostic.

**Reference:** `SliceF32Reference` in `crates/tiler-reference/src/structural.rs`, registered over the same unary signature the two other coordinate-mapping families use, cloning elements so exceptional payloads cross unchanged. `crates/tiler-reference/tests/slice_conformance.rs` covers ranks one through four, outermost/interior/innermost axes, several restricted axes at once, two composed selections, the reindex that removes the left-behind axis, and the exceptional payloads; perturbing the evaluator's offset to zero fails eight of its nine tests.

**Support matrix:** the row moved **R1 → R4** for the F32 literal-offset semantics and reference, with R5 awaiting a fusion role and the strided and symbolic relations explicitly left at R1 with their own triggers. Both existing triggers are preserved with their evidence; the rotary-table trigger now owes the index vocabulary rather than the family.

**Compiler seating is an integrator edit, not part of this branch** — `crates/tiler-compiler/**` was held by a live sibling. Two edits are required in the same landing or the workspace gate goes red, both exactly the shape `tiler::concatenate-f32@1` needed: add `"tiler::slice-f32@1"` to `UNPLANNED_OPERATIONS` in `crates/tiler-compiler/src/policy.rs` (the family performs no arithmetic, so a capability row would claim a target dimension it never asks for), and recompute the pinned explain request digest in `crates/tiler-compiler/src/explain.rs` **on the merged tree** from an observed run, because the request subject folds the registry snapshot and every admitted family moves it.

## Current correction — 2026-08-09

The outcome above is historical where it says the symbolic-offset remainder is an `IndexNode` vocabulary gap. The index module now carries `SourcedIndexInteger` coefficients, and the slice module records the fired trigger under the source anchor `the index-vocabulary one has since closed`: `t + C` is expressible as a sourced coefficient.

The delivered literal family and every identity, refusal, reference result, and support-matrix movement above remain unchanged. The live refusal is now at the semantic selection boundary: `SliceAxisSelection::Window` still carries `offset: u64`, and `decode_axis` refuses `symbolic-window` before parsing relation fields. [`decide-the-source-bearing-slice-offset-boundary`](decide-the-source-bearing-slice-offset-boundary.md) owns whether that source arrives as an attribute or an operand and the resulting inference, bounds, reference, and identity consequences. The rotary-table consumer therefore waits on that decision, not on more index-expression vocabulary.
