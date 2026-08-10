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
- **False — the Unknown conflated initial recognition with the ordinary compile path and missed an existing normalization regression.** Read `crates/tiler-compiler/src/pipeline.rs`, `crates/tiler-compiler/src/normalize.rs`, and `crates/tiler-compiler/src/pipeline/tests.rs` in full. At `fn compile_contract_group`, ordinary compilation calls `normalize_semantics` and chooses `normalization.normalized_program().unwrap_or(semantic)` before candidate formation and `readmit_candidate`. At `fn detect_shared_values`, the common-subexpression rule keys every `OperationEffect::Pure` invocation by operation key, exact canonical attributes, congruent operands, and result types. Equal standard constant invocations therefore share one signature and are rebuilt as one semantic value. The existing test `normalization_converges_duplicated_and_shared_constants_on_one_portfolio` is the authoritative positive for the f32 compile path: the duplicated program reports one `normalize.common-subexpression.v1` rewrite and converges on the shared program's portfolio. The original source audit stopped too early and its conclusion that no normalization rule exists was false. This repair narrows the answer without changing the ticket's research purpose: occurrence identity survives the initial recognizer/mint, then ordinary compilation removes the equal pure constant duplication before scheduled-program planning.

## Outcome — 2026-08-10

**Fact — exact initial-seam source rule.** `F32Constant::apply` and `Bf16Constant::apply` each enter `SemanticProgramBuilder::push_operation`, which allocates a fresh result `ValueId`. `plan_elementwise` records each completed value in `minted: Vec<ValueId>`: revisiting the same exact value is skipped when `minted.contains(&value)`, while two distinct constant values with equal attribute bytes each produce an `ElementwiseMint::Constant` step. `mint_into` likewise reuses only an earlier entry with the same `ValueId` and otherwise calls the width-specific sink once per constant step. This rule describes direct recognition/minting of an unnormalized semantic program, not the graph ordinary compilation later plans.

**Fact — ordinary compilation normalizes before physical planning.** `compile_contract_group` calls `normalize_semantics`, takes its normalized graph when one is returned, forms candidates from that graph, and passes those candidates through `VerifiedTargetRequest::readmit_candidate` before physical planning. `detect_shared_values` merges equal pure constant invocations because their operation keys, exact canonical attribute bytes, empty operand lists, and result types are equal. Thus the ordinary f32 compile path does not retain the initial four-versus-five-node spelling: `normalization_converges_duplicated_and_shared_constants_on_one_portfolio` proves that the duplicated and shared programs converge on one portfolio before scheduled-program identity is minted.

**Measurement — direct f32 and BF16 recognition preserves the spelling.** The regression `equal_constant_occurrences_remain_distinct_through_initial_recognition` builds paired programs for `x * 2 + 2`. Direct recognition of the shared f32 and BF16 programs mints four-node expressions; direct recognition of the duplicated programs mints five-node expressions whose two constant nodes carry equal payloads. This is bounded evidence about the initial internal recognizer/mint seam only.

**Measurement — BF16's governed boundary precedes recognition, while its normalization rule is width-generic.** The built-in `tiler.prototype-target-neutral-baseline.v1` profile resolves BF16 dispatch as `Unknown`, so `verify_request` returns `RequestError::DTypeNotDispatchable` before `select_supported_strategy` invokes recognition. Direct internal recognition can therefore demonstrate the four-versus-five-node initial spelling, but the new pair does not reach a governed `ScalarProgram::PointwiseBf16`. A caller-supplied BF16 profile is known to admit BF16 compilation, while the common-subexpression signature is dtype-agnostic; the normalization regression drives the equal BF16 pair directly and proves that it converges before any reachable physical planning would consume it. This does not claim that the new pair was compiled through a caller profile.

**Fact — exact exceptional boundary.** If `normalization_rewrites` is smaller than the rewrite demand, `run_rewrite_engine` returns `BudgetStopped` and `normalize_semantics` returns no partial graph. `compile_contract_group` then retains the original semantic graph through `unwrap_or(semantic)`. Rewrite-budget exhaustion is therefore the explicit path on which duplicate equal constants can survive normalization into candidate readmission; it is not the behavior of an ordinary governed request and is not an identity policy.

**Perturbation evidence.** Excluding `constant_f32_op` from `detect_shared_values` left the duplicated f32 graph unchanged and failed `f32_equal_pointwise_constants_converge_at_normalization` with `equal pure f32 constants are commoned`. After restoring f32 handling, independently excluding `constant_bf16_op` left the duplicated BF16 graph unchanged and failed `bf16_equal_pointwise_constants_converge_at_normalization` with `equal pure bf16 constants are commoned`. Both production mutations were restored before rerunning the passing tests. These perturbations alter the CSE machinery whose reachability is under test, not a fixture census or assertion.

**Boundary and unsupported cases.** This evidence covers exact equal-payload constants in the f32 and BF16 constant/multiply/add families, one static-shape input, direct internal recognition, ordinary normalization, and the ordinary governed f32 compile path. Governed BF16 is refused on dtype dispatch before recognition, and this ticket does not claim a compiled `ScalarProgram::PointwiseBf16` for the new pair. Rewrite-budget exhaustion is the sole demonstrated exception to whole-run normalization, not evidence that occurrence spelling normally reaches scheduled identity. This ticket does not decide whether equal constants should be canonicalized, whether occurrence spelling belongs in any schedule/request/artifact/cache identity, or what a consumer should author; those are policy consequences for the dependent decision ticket. It does not generalize the measurement to other payload widths, other operation families, multi-output partitions, staged regions, or constants synthesized below this semantic lowering seam.

Focused commands: `cargo test -p tiler-compiler request::tests::equal_constant_occurrences_remain_distinct_through_initial_recognition -- --exact --nocapture`; `cargo test -p tiler-compiler normalize::tests::f32_equal_pointwise_constants_converge_at_normalization -- --exact --nocapture`; `cargo test -p tiler-compiler normalize::tests::bf16_equal_pointwise_constants_converge_at_normalization -- --exact --nocapture`; and `cargo test -p tiler-compiler pipeline::tests::normalization_converges_duplicated_and_shared_constants_on_one_portfolio -- --exact --nocapture`.
