---
id: determine-whether-compiler-lowering-mints-duplicate-pointwise-constants
title: Determine whether compiler lowering mints duplicate pointwise constants
status: in-progress
priority: p2
dependencies: []
related: [share-identical-constants-in-the-pointwise-expression-canonical-form]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, identity, research]
claimed_from: todo
assignee: sol-pointwise-constants
lease_expires_at: 1786391757
---
## Why this evidence is separate

**Fact — current builder spelling.** `PointwiseF32ExpressionBuilder::constant` and `PointwiseBf16ExpressionBuilder::constant` each append a draft constant unconditionally. Their `canonicalize_nodes` walks preserve every reachable draft constant rather than hash-consing equal payloads. The schedule witness test `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` therefore proves that the public builders can spell two distinct canonical expressions for one binary32 function.

**Fact — current compiler minting seam.** Compiler lowering does not call those builders directly from semantic operations. `mint_into` replays one `ElementwisePlan`, keeps a `Vec<(ValueId, S::Value)>`, and calls `sink.constant` once for every `ElementwiseMint::Constant` step. Repeated operands naming one semantic `ValueId` reuse the earlier minted value; two distinct semantic constant occurrences with equal bits would reach two constant steps if the semantic and planning layers preserve both occurrences.

**Unknown.** No current test proves whether two semantic programs computing the same function — one reusing a single constant occurrence and one carrying two equal constant occurrences — can both reach `mint_elementwise` and produce distinct `ScalarProgram::PointwiseF32` or `ScalarProgram::PointwiseBf16` payloads. That is a source-and-experiment question, not an identity-policy decision.

## Work

1. Read the semantic constant construction, elementwise planning, `mint_into`, both pointwise builders, and the schedule-witness test in full.
2. Build two semantic programs computing the same function that differ only in whether one exact constant occurrence is reused or two distinct equal-payload occurrences are used.
3. Drive both through the compiler recognition/minting path and compare the resulting scalar programs. Cover `f32`; cover `bf16` if the compiler currently admits an equivalent reachable pair, otherwise record the exact refusal that makes that width unreachable.
4. Perturb the subject so the check demonstrates that it reaches constant-occurrence identity rather than merely comparing two fixtures. Quote the failure.
5. Record the exact source rule if either spelling is rejected or normalized before minting.

Do not change either builder, schedule identity, an identity domain, request/artifact/cache subjects, or Metal goldens here. Those consequences belong to the dependent decision ticket.

## Closes when

The compiler-reachability question is answered by a source-backed compiled pair or by a proved rejection/normalization rule, with a regression test that reaches the decisive seam and a ticket Outcome that states the exact `f32` and `bf16` boundary.

## Fact audit — 2026-08-10 at `93c69b14731902c9dfb167a899af3f8b4d905fdb`

- **Verified — current builder spelling.** Read `crates/tiler-ir/src/schedule/pointwise.rs` and `crates/tiler-ir/src/schedule/pointwise_bf16.rs` in full. The source-safe anchors are `pub fn constant` and `fn canonicalize_nodes`: each application appends one draft node, and canonicalization retains every reachable draft node by old ordinal rather than looking up equal payloads. Read `crates/tiler-ir/src/schedule/witness/tests.rs` in full; `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` still asserts distinct binary32 expressions. The Fact is current.
- **Verified — current compiler minting seam.** Read the elementwise recognition/planning and minting sections of `crates/tiler-compiler/src/request.rs` in full, including the anchors `struct ElementwisePlan`, `fn plan_elementwise`, `trait PointwiseMintSink`, `fn mint_into`, and `fn mint_elementwise`. Planning and minting are keyed by exact semantic `ValueId`; they contain no payload-equality lookup. Read the complete standard constant definitions and handles in `crates/tiler-ir/src/semantic/standard_operations.rs` and `crates/tiler-ir/src/semantic/handles.rs`, plus semantic construction and output compaction at the anchors `SemanticProgramBuilder::push_operation` and `compact_to_outputs` in `crates/tiler-ir/src/semantic/program.rs`. A second constant application creates a second reachable occurrence; compaction remaps reachable values but does not coalesce equal constants. The Fact is current.
- **Verified — the Unknown was an evidence gap.** The compiler tests at this base cover shared subexpressions, repeated reads, and ordinary constants, including a BF16 constant/multiply/add fixture, but no paired programs differ only by reuse versus repetition of one equal-payload constant occurrence. Searches for the builder and minting anchors located no such paired compiler regression. This audit found no rejection or normalization rule that would make the pair unreachable. No Fact repair or purpose change was required.

## Outcome — 2026-08-10

**Fact — exact source rule.** `F32Constant::apply` and `Bf16Constant::apply` each enter `SemanticProgramBuilder::push_operation`, which allocates a fresh result `ValueId`. `plan_elementwise` records walk state by that exact `ValueId`: reuse of one value reaches the existing `Done` state, while two distinct constant values with equal attribute bytes each produce an `ElementwiseMint::Constant` step. `mint_into` likewise reuses only an earlier entry with the same `ValueId` and otherwise calls the width-specific sink once per constant step. No semantic, planning, or minting layer compares constant payloads for equality.

**Measurement — f32 is reachable through the decisive scheduled-program seam.** The regression `equal_constant_occurrences_remain_distinct_through_compiler_minting` builds two programs for `x * 2 + 2`. Reusing one constant yields three semantic occurrences and `ScalarProgram::PointwiseF32` with four nodes (one input, one constant, multiply, add). Applying the same constant twice yields four semantic occurrences and `ScalarProgram::PointwiseF32` with five nodes; its two constant nodes both carry `2.0_f32.to_bits()`. The scalar programs compare unequal.

**Measurement — bf16 reaches recognition and minting, while the built-in target boundary is exact.** The equivalent BF16 pair is currently statable and recognized. It mints `RecognizedPointwise::Bf16` expressions with four versus five nodes; the repeated spelling contains two constant nodes both carrying `0x4000`, and the expressions compare unequal. `crates/tiler-compiler/src/physical.rs` exhaustively maps that recognized variant to `ScalarProgram::PointwiseBf16` at the anchor `RecognizedPointwise::Bf16(expression) => ScalarProgram::PointwiseBf16(expression)`. A caller-supplied BF16 profile already proves multi-occurrence BF16 planning in `a_multi_occurrence_bf16_program_derives_its_own_fusion_legality`. The built-in `tiler.prototype-target-neutral-baseline.v1` profile does not declare BF16 dispatch, so the same repeated program through the governed request stops before planning at `RequestError::DTypeNotDispatchable` for `Bf16::resolved_type()` with disposition `Unknown`; the regression asserts every field of that refusal.

**Perturbation evidence.** Replacing the repeated `F32Constant::apply` with reuse of `two` made the regression fail at the semantic occurrence census with `assertion left == right failed`, `left: 3`, `right: 4`. After restoring f32, the same subject perturbation to the BF16 arm failed independently with the same values at its BF16 census. Both changes were restored, and the passing test was rerun. The check therefore reaches the occurrence distinction it claims; a fixture that accidentally reuses the value fails before its scalar-node comparison.

**Boundary and unsupported cases.** This evidence covers exact equal-payload constants in the governed f32 and BF16 constant/multiply/add families, one static-shape input, and reuse within one whole-program pointwise expression. It does not decide whether equal constants should be canonicalized, whether occurrence spelling belongs in any schedule/request/artifact/cache identity, or what a consumer should author; those are policy consequences for the dependent decision ticket. It does not generalize the measurement to other payload widths, other operation families, multi-output partitions, staged regions, or constants synthesized below this semantic lowering seam.

Focused command: `cargo test -p tiler-compiler request::tests::equal_constant_occurrences_remain_distinct_through_compiler_minting -- --exact --nocapture`.
