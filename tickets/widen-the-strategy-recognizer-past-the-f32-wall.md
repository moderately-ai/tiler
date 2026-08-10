---
id: widen-the-strategy-recognizer-past-the-f32-wall
title: Widen the strategy recognizer past the f32 wall
status: done
priority: p1
dependencies: []
related: [conform-the-bf16-vertical-end-to-end, establish-bf16-optimizer-legality, correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate, correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [bf16, dtype, blocker]
---
## The wall, and why it was unowned

**Fact (historical problem statement at filing; retired by Outcome) — `select_supported_strategy` in `crates/tiler-compiler/src/request.rs` refused every non-`f32` program under the rule `dtype-f32`, before a subject was normalized.** Nothing downstream could produce the `PlanAlternative` that `compile()`, the artifact envelope, and the runtime routing commit consume, so those three layers were unreachable for BF16 by any route. Searchable anchors for the retired rule and its replacement: `fn select_supported_strategy(`, `/// **This replaced a `dtype-f32` gate`.

Three existing sites asserted that wall deliberately under names re-founded in Outcome — including `a_flush_accepting_bf16_contract_reaches_the_recognizer_dtype_wall` (now `…_reaches_a_selected_plan`) in `crates/tiler-compiler/tests/bf16_numerical_contract.rs` and the `bf16_scheduled_region` neighbour in `crates/tiler-compiler/src/pipeline/tests.rs` — so it was a stated boundary rather than an oversight.

**It became load-bearing on 2026-08-07.** [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) crossed BF16 from semantic construction to a real GPU dispatch and a bit comparison, and had to assemble its region through `tiler-ir`'s public builders because of this wall. That closed the ticket on everything else its evidence list demanded, and left its **first bullet** — a program carried through compile, artifact, runtime routing, and dispatch — structurally unreachable. It was the fourth block that ticket hit, and the only one with no owner: `establish-bf16-optimizer-legality` holds legality *keying*, not recognition.

## What this owes

- The recognizer admitting a non-`f32` program whose dtype the profile and contract support, so a `PlanAlternative` exists for it to plan.
- **The refusal kept where it belongs.** Widening recognition must not admit a dtype the target cannot honour — that refusal is the target profile's and the numerical contract's, and it must still fire, from its own authority, with its own typed cause. A program that was refused as `dtype-f32` and is now refused as unhonourable is the *correct* outcome for an unsupported row, and the two must be distinguishable.
- The three deliberate wall assertions **re-founded rather than deleted**: whatever still refuses after this lands is what they should assert. Deleting them would remove the evidence that the boundary is where it is meant to be.
- Every downstream leg the wall was hiding checked rather than assumed. A layer that has never seen a non-`f32` `PlanAlternative` may carry its own `f32` assumption; finding one is a result, not a setback.

## Explicit non-goals

Not legality keying — that is `establish-bf16-optimizer-legality`. Not a new dtype, not a new family, and not a widening of what any target declares it can honour. Not the conformance run itself: closing this lets [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md)'s first bullet be met by a **follow-up run**, which is its own ticket rather than this one.

## Required evidence

- A non-`f32` program reaching a selected `PlanAlternative`, named.
- The unhonourable case still refused, from the profile's or contract's authority, watched failing.
- The three wall assertions re-founded on what actually refuses now.
- Whatever identity moves, enumerated and recomputed on the merged tree.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration of the BF16 vertical, from a block its worker found, refused to edit around because the crate was live-claimed, and reported as unowned. It is p1 because it is the single structural obstacle between a BF16 program and the compiled path, and because three other tickets' evidence lists narrow to it.

## Outcome

**The wall is gone, and what replaced it is a derivation rather than a wider constant.** `select_supported_strategy` no longer carries a `dtype-f32` rule. It derives the program's one arithmetic type from its values and refuses two distinct findings by their own names: `dtype-recognized` for a width this build spells no per-point body in, and `dtype-uniform` for a program carrying two recognized widths at once, which no single scheduled region can realize however well each width is supported. The admitted set is stated once, in `request::recognized_arithmetic`, and every other authority asks it.

**Measurement — a non-`f32` program reaching a selected `PlanAlternative`.** `crates/tiler-compiler/tests/bf16_numerical_contract.rs`'s `a_flush_accepting_bf16_contract_reaches_a_selected_plan` compiles the pure-BF16 program `out = x + y` under `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16` on a profile declaring the measured BF16 rows, and asserts the resolved contract key, one enumerated alternative, and a selected one.

**The shape is one occurrence, and the count was the boundary rather than an accident (delivery-time at `0b0b4bed`).** At delivery a region covering two or more occurrences was put to `derive_fusion_legality` before any cover survived, and `FusionNumericalCapabilities::governed` mapped the six `f32` operation keys and nothing else — so a multi-occurrence BF16 region was `Unknown` and every cover placing it was skipped. That was [`establish-bf16-optimizer-legality`](establish-bf16-optimizer-legality.md)'s to widen and was deliberately not absorbed here: reassociation error is bounded by the significand and Finding 28 of the Apple numerical behaviour record measures a target whose contraction behaviour differs between `f16` and `bf16`, so a capability row copied from the `f32` set would have been a legality claim nothing proved. Delivery-time test `a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall` asserted that boundary rather than leaving it to be discovered.

**Correction — 2026-08-10.** That multi-occurrence fusion-legality wall is **no longer live**. [`establish-bf16-optimizer-legality`](establish-bf16-optimizer-legality.md) is `status: done` and widened the governed fusion table across two widths (`/// **The table spans two widths and the entries are not each other's.**`; BF16 role inserts include `constant_bf16_op()`). The delivery-time wall test name does not exist under `crates/` (only historical ticket citations). Live successor: `a_multi_occurrence_bf16_program_derives_its_own_fusion_legality` in `crates/tiler-compiler/tests/bf16_numerical_contract.rs`, which asserts a fused selected plan. A narrower surviving wall is `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall`. This ticket stays closed on its owned work; do not re-open it for the successor.

**Fact — where the refusal went.** A contract's arithmetic is part of its identity (ADR 0076 item 6) and a target's honourability rows are keyed by subject, so a contract about another width is not a question any profile can answer. `RequestError::NoApplicableNumericalContract` is that refusal: program-scoped, checked beside representability and before any target, naming the program's arithmetic and every stated contract's in the caller's order. `resolve_numerical_contract` skips an inapplicable entry rather than rejecting it, so a preference naming this program's width alongside another's still resolves and reports the applicable entries' own causes. Public rule key: `compile.request.numerics.inapplicable`, classed `InvalidRequest`.

**Fact — three downstream `f32` assumptions found, each fixed in `implementation/compiler`.** None was reachable while the recognizer refused first.

1. `crate::program::verify_semantic_output_type` required every declared output to be `f32` exactly, and refused a recognized BF16 program as `semantic-output-type` — a compiler defect reported for a program recognition had just admitted. It now asks `recognized_arithmetic`, which is the one place the admitted set is stated.
2. `crate::physical::region_proposal` built every `NumericalRequirement` with the region's arithmetic beside a hard-coded `tiler::f32@1` resolved type. A requirement matches a declaration only when *both* halves of the subject agree, so a BF16 region matched no BF16 row any profile could declare, every dimension resolved `Unknown`, and the region was refused `target-assessment-unresolved` on a profile whose measured rows answered it exactly. It now derives the subject through `policy::arithmetic_subject`, the same constructor `dimension_requirements` already used.
3. `crate::program`'s `BOUNDED_CARRIER` was the constant `StorageScalar::F32`, sizing and aligning every buffer and every accessible-byte expression, and pairing them with `KernelType::F32`. `KernelProgramBuilder::build` refused the result as `StageElementType { expected: Bf16, actual: F32 }`. It is now `BoundedCarrier`, derived from the contract this target compiles under, carrying storage scalar and kernel element type together because a carrier paired with the wrong element type is exactly the disagreement that refusal names.

**Fact — three governed lowering capabilities added.** `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, and `tiler::add-bf16@1`, each with its own provider identity and its emitted `tiler.scalar` operation, bringing `GOVERNED_INDEX_ACCESS_CAPABILITIES` from 17 to 20. `GovernedConstantF32` and `GovernedPointwiseF32` became `GovernedConstant`/`GovernedPointwise` parameterized by a `GovernedWidth`, so the index work is one derivation and only the scalar authority it reaches differs. The set stops at three because that is the set `tiler_ir::index` registers `bf16` scalars for.

**Fact — the three wall assertions re-founded, none deleted.**

- `a_flush_accepting_bf16_contract_reaches_the_recognizer_dtype_wall` → `a_flush_accepting_bf16_contract_reaches_a_selected_plan`. Before: `CompileFailureClass::UnsupportedCapability { rule: "dtype-f32" }`. After: a planned batch with one alternative and a selected plan under the stated BF16 contract's key.
- `the_accepted_bf16_contract_schedules_and_lowers_a_region_the_request_cannot_reach` → `..._the_request_now_reaches`. Before: the same `dtype-f32` class beside the hand-assembled region. After: the request reaches a selected plan; the region-and-kernel half is unchanged and still derives its realization from the accepted contract.
- `an_f32_contract_is_resolved_against_f32_rows_only` → `an_f32_contract_does_not_answer_for_a_bf16_program`. Before: `dtype-f32` caught the pairing incidentally. After: `InvalidRequest { rule: "compile.request.numerics.inapplicable" }`, with no sealed trace because the refusal precedes every target. The claim is unchanged; the authority that carries it is now the contract's own.

A fourth site the ticket did not name was also re-founded: `crate::pipeline::tests`'s `bf16_scheduled_region` doc claimed no BF16 region was reachable from the request boundary. At this ticket's delivery it was reworded to name the *fusion* boundary rather than `dtype-f32`. **Correction — 2026-08-10.** That fusion wall is gone too after [`establish-bf16-optimizer-legality`](establish-bf16-optimizer-legality.md); the live doc (`/// Assembled through `tiler-ir`'s public builders rather than through`) keeps the fixture hand-assembled for the realization it needs *stated* rather than resolved, and cites `a_multi_occurrence_bf16_program_derives_its_own_fusion_legality` for multi-occurrence fusion success.

`a_pure_bf16_program_is_statable_and_refused_at_the_request_boundary` kept its subject — the governed baseline's own `DTypeNotDispatchable` row — by stating a BF16 contract, because `CompilationRequest::governed`'s `f32` contract now meets the applicability refusal first. `an_f32_contract_stated_for_a_bf16_program_is_refused_before_any_target` is the new neighbour that asserts that pairing.

**Measurement — every new refusal watched failing, then restored.** Each perturbation was applied alone and reverted:

- Applicability check disabled (`if false && …` in `verify_request`): both pairing tests fail; a BF16 program compiles under the strict `f32` contract.
- `region_proposal`'s subject restored to `F32::resolved_type()`: `a_flush_accepting_bf16_contract_reaches_a_selected_plan` fails; the profile's BF16 rows are never consulted.
- `dtype-uniform` arm disabled: the mixed-width program is admitted and refuses `operation-set` instead — `left: Err("operation-set")`, `right: Err("dtype-uniform")`.
- `BoundedCarrier::of(Bf16)` returning the `f32` carrier: the plan fails to assemble.
- `verify_semantic_output_type` restated as `f32` exactly: the plan fails to assemble.

**Measurement — populations counted.** `a_bf16_program_is_recognized_in_its_own_expression_vocabulary` asserts the recognized member count equals `program.operation_count()`, the node count, the two constants as their exact sixteen declared bits, and the one dense read — so an assertion about the expression is an assertion about the whole program rather than a prefix of it. `a_flush_accepting_bf16_contract_reaches_a_selected_plan` asserts the alternative count before asking whether one was selected. `a_mixed_width_program_and_an_unspelled_width_refuse_by_different_names` carries an accepted `f32` neighbour for the second case, so the refusal is attributed to the width rather than to the shape.

**Fact — identity (delivery-time snapshot at `0b0b4bed`; not live pin values).** Exactly one pinned identity moved on this delivery: `crates/tiler-compiler/src/explain.rs`'s `deterministic_trace_is_sealed_and_rendered_separately` request qualifier, `e59cb8aa9b38ef70` → `de9ad4cc087697d8`. The request subject binds `CanonicalLoweringRegistryIdentity`, which encodes every registered capability, so the three added rows moved it for every governed compilation. No encoding version stepped: the `pointwise-bf16.v1` sub-tag is a new arm under the existing per-tag framing, so every `f32` pointwise subject encoded to exactly the bytes it did. At delivery, `crates/tiler-build/src/metal_plan.rs`'s `ARTIFACT_IDENTITY` (`7a2bfe51619c05a13fe86cd973e1dfa85c7353da33e4e75af0531068b774357d`), `CACHE_SUBJECT` (`8bdcde644d7df6d4ca95736f445a011b2d163efdfb3ba93a5c0a954d139b1aa2`), and `FIXED_CONTENT_BYTES` (`65_294`) were unmoved and passed unchanged — executable-coverage identity is reached-only, so growing the registry beneath an `f32` occurrence left its artifact identity byte-identical. **Correction — 2026-08-10.** Later work rebaselined these pins again; live explain pin is `request=7ba3d77a66f04638`, and live `FIXED_CONTENT_BYTES` is `65_313`. Do not treat the hex values above as current.

**Boundary — no public item added or widened.** `git diff` adds no `pub` item. One new *value* reaches the public failure vocabulary: `CompileFailureClass::InvalidRequest { rule: "compile.request.numerics.inapplicable" }`.

**Boundary — stale claims outside this branch's scopes, filed rather than edited.** Two comments in `crates/tiler-conformance` and five documents under `docs/` state the removed `dtype-f32` rule, and two of them cite compiler tests renamed with it. `correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate` and `correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents` carry the exact sites and the corrected statement. The `docs/dtype-support.md` BF16 support-matrix row is named in the second.

**What was not done at delivery.** BF16 optimizer legality, which [`establish-bf16-optimizer-legality`](establish-bf16-optimizer-legality.md) owned and whose absence was, at delivery, the one remaining boundary between this and a multi-occurrence BF16 plan. The conformance run through `compile()`, which is its own ticket. **Correction — 2026-08-10.** That legality ticket is `status: done`; multi-occurrence pure pointwise BF16 now fuses under its own legality (`a_multi_occurrence_bf16_program_derives_its_own_fusion_legality`). Conformance landed under [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md). This ticket's owned work remains closed; the successor was never this ticket's remainder.

## Outcome — delivered 2026-08-07 at `0b0b4bed`

**The wall is down.** `select_supported_strategy` has no `dtype-f32` rule; it derives the program's one arithmetic type from its values through `request::recognized_arithmetic` — the single statement of the admitted set — and mints per-width vocabulary in one walk. Verified by the coordinator: the surviving `dtype-f32` mentions are doc comments *explaining what was removed*, not the rule.

**A non-`f32` program reaches a selected `PlanAlternative`**: pure-BF16 `out = x + y` under `FLUSH_SUBNORMALS_TO_ZERO_BF16` on a profile carrying the measured BF16 rows, with the resolved contract key asserted, one alternative counted, one selected.

**The refusal moved to its proper authority rather than being deleted**, which was the requirement most likely to be got wrong. A BF16 program under an `f32` contract is now refused **by the contract**, program-scoped and before any target is consulted, under a new typed `RequestError::NoApplicableNumericalContract` and the public rule `compile.request.numerics.inapplicable` — resting on ADR 0076 item 6, that arithmetic is part of contract identity and target rows are subject-keyed. The profile's own refusals (`DTypeNotDispatchable`, `NoResolvableNumericalContract`) are unchanged, and the recognizer keeps two rules of its own for a width this build spells no body for and for two widths in one program. Five perturbations, each watched failing alone and restored — including one that let a BF16 program compile under strict `f32`.

**Three downstream `f32` assumptions the wall was hiding, all found and fixed.** This is exactly the class the brief said to look for and called a result rather than a setback:

- `verify_semantic_output_type` required `f32` outputs exactly, so it reported a **compiler defect** for a program recognition had just admitted.
- `region_proposal` paired the region's arithmetic with a hard-coded `tiler::f32@1` resolved type, so a BF16 region matched no row any profile could declare — every dimension `Unknown`, and `target-assessment-unresolved`.
- `BOUNDED_CARRIER` was a constant `F32` pair sizing every buffer and accessible-byte expression at four bytes.

Each is the same shape: a constant standing where a derivation belonged, invisible while only one width could reach it.

**All four wall assertions re-founded, none deleted** — including a fourth site the ticket did not name, whose doc claimed no BF16 region was reachable from the request boundary and at delivery named the *fusion* boundary that then still kept that fixture hand-assembled. **Correction — 2026-08-10.** That fusion wall is also gone; see the mid-Outcome correction and the live `bf16_scheduled_region` doc.

**Identity: exactly one pin moved at delivery (`0b0b4bed`)**, the explain request qualifier `e59cb8aa9b38ef70` → `de9ad4cc087697d8`, and the attribution was **proved rather than argued** — removing only the three new capability rows returned it byte-for-byte, so the whole move was the lowering-registry identity and no `f32` subject byte changed. No encoding version stepped. At delivery the standard Metal pins (`ARTIFACT_IDENTITY`, `CACHE_SUBJECT`, `FIXED_CONTENT_BYTES = 65_294`) were unmoved and their file unedited. **Correction — 2026-08-10.** Those values are delivery-time only; later identity work moved the explain pin and Metal goldens again (live explain `request=7ba3d77a66f04638`, live `FIXED_CONTENT_BYTES = 65_313`).

**No public item added or widened**; one new *value* in the public failure vocabulary.

### The boundary that survived at delivery, asserted rather than left to be discovered

**At delivery (`0b0b4bed`), a multi-occurrence BF16 region still stopped at `fusion_legality`.** `FusionNumericalCapabilities::governed` then mapped the six `f32` op keys, so a two-or-more-member BF16 region was `Unknown` and every cover placing it was skipped. The worker **deliberately did not add rows**, on a ground worth keeping: [`establish-bf16-optimizer-legality`](establish-bf16-optimizer-legality.md) owned it, and Finding 28 measures an Apple row whose contraction behaviour **differs between `f16` and `bf16`** — so copying the `f32` rows would have been a legality claim about another width, made without evidence.

It was asserted by the delivery-time test `a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall`, and the module header said why: "precisely so a reader cannot mistake one planned shape for general support." That was the maturity discipline working — one planned shape is one planned shape.

**Correction — 2026-08-10.** The fusion-legality survival claim above is **historical delivery boundary, not live behaviour**. [`establish-bf16-optimizer-legality`](establish-bf16-optimizer-legality.md) landed and is `status: done`; the governed fusion table spans two widths; the retired wall test is not defined under `crates/` (grep finds only historical ticket prose). Live successor test `a_multi_occurrence_bf16_program_derives_its_own_fusion_legality` asserts a fused selected plan. Narrower surviving wall: `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall`. This ticket is not reopened for that work.

**Released:** [`correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate`](correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate.md) and [`correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents`](correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents.md), the latter naming `docs/dtype-support.md`'s BF16 support-matrix row. In-scope stale claims were corrected directly.

`make full` exit 0 on the branch (2,997 workspace, 1,052 release); re-gated on the merged tree.
