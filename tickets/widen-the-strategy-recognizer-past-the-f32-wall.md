---
id: widen-the-strategy-recognizer-past-the-f32-wall
title: Widen the strategy recognizer past the f32 wall
status: in-progress
priority: p1
dependencies: []
related: [conform-the-bf16-vertical-end-to-end, establish-bf16-optimizer-legality, correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate, correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [bf16, dtype, blocker]
claimed_from: todo
assignee: agent-recognizer
lease_expires_at: 1786129392
---
## The wall, and why it was unowned

**Fact — `select_supported_strategy` (`crates/tiler-compiler/src/request.rs:4206`) refuses every non-`f32` program under the rule `dtype-f32`, before a subject is normalized.** Nothing downstream can produce the `PlanAlternative` that `compile()`, the artifact envelope, and the runtime routing commit consume, so those three layers are unreachable for BF16 by any route.

Three existing sites assert that wall deliberately — `crates/tiler-compiler/tests/bf16_numerical_contract.rs:399,429,621` and `crates/tiler-compiler/src/pipeline/tests.rs:3922-3927` — so it is a stated boundary rather than an oversight.

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

**The shape is one occurrence, and the count is the boundary rather than an accident.** A region covering two or more occurrences is put to `derive_fusion_legality` before any cover survives, and `FusionNumericalCapabilities::governed` maps the six `f32` operation keys and nothing else — so a multi-occurrence BF16 region is `Unknown` and every cover placing it is skipped. That is `establish-bf16-optimizer-legality`'s to widen and was deliberately not absorbed here: reassociation error is bounded by the significand and Finding 28 of the Apple numerical behaviour record measures a target whose contraction behaviour differs between `f16` and `bf16`, so a capability row copied from the `f32` set would be a legality claim nothing proved. `a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall` asserts that boundary rather than leaving it to be discovered.

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

A fourth site the ticket did not name was also re-founded: `crate::pipeline::tests`'s `bf16_scheduled_region` doc claimed no BF16 region was reachable from the request boundary. It now states the reason that survives — the fixture is a four-occurrence chain, so the *fusion* boundary keeps it hand-assembled, not the dtype.

`a_pure_bf16_program_is_statable_and_refused_at_the_request_boundary` kept its subject — the governed baseline's own `DTypeNotDispatchable` row — by stating a BF16 contract, because `CompilationRequest::governed`'s `f32` contract now meets the applicability refusal first. `an_f32_contract_stated_for_a_bf16_program_is_refused_before_any_target` is the new neighbour that asserts that pairing.

**Measurement — every new refusal watched failing, then restored.** Each perturbation was applied alone and reverted:

- Applicability check disabled (`if false && …` in `verify_request`): both pairing tests fail; a BF16 program compiles under the strict `f32` contract.
- `region_proposal`'s subject restored to `F32::resolved_type()`: `a_flush_accepting_bf16_contract_reaches_a_selected_plan` fails; the profile's BF16 rows are never consulted.
- `dtype-uniform` arm disabled: the mixed-width program is admitted and refuses `operation-set` instead — `left: Err("operation-set")`, `right: Err("dtype-uniform")`.
- `BoundedCarrier::of(Bf16)` returning the `f32` carrier: the plan fails to assemble.
- `verify_semantic_output_type` restated as `f32` exactly: the plan fails to assemble.

**Measurement — populations counted.** `a_bf16_program_is_recognized_in_its_own_expression_vocabulary` asserts the recognized member count equals `program.operation_count()`, the node count, the two constants as their exact sixteen declared bits, and the one dense read — so an assertion about the expression is an assertion about the whole program rather than a prefix of it. `a_flush_accepting_bf16_contract_reaches_a_selected_plan` asserts the alternative count before asking whether one was selected. `a_mixed_width_program_and_an_unspelled_width_refuse_by_different_names` carries an accepted `f32` neighbour for the second case, so the refusal is attributed to the width rather than to the shape.

**Fact — identity.** Exactly one pinned identity moved: `crates/tiler-compiler/src/explain.rs`'s `deterministic_trace_is_sealed_and_rendered_separately` request qualifier, `e59cb8aa9b38ef70` → `de9ad4cc087697d8`. The request subject binds `CanonicalLoweringRegistryIdentity`, which encodes every registered capability, so the three added rows move it for every governed compilation. No encoding version stepped: the `pointwise-bf16.v1` sub-tag is a new arm under the existing per-tag framing, so every `f32` pointwise subject encodes to exactly the bytes it did. `crates/tiler-build/src/metal_plan.rs`'s `ARTIFACT_IDENTITY` (`7a2bfe51619c05a13fe86cd973e1dfa85c7353da33e4e75af0531068b774357d`), `CACHE_SUBJECT` (`8bdcde644d7df6d4ca95736f445a011b2d163efdfb3ba93a5c0a954d139b1aa2`), and `FIXED_CONTENT_BYTES` (`65_294`) are unmoved and pass unchanged — executable-coverage identity is reached-only, so growing the registry beneath an `f32` occurrence leaves its artifact identity byte-identical.

**Boundary — no public item added or widened.** `git diff` adds no `pub` item. One new *value* reaches the public failure vocabulary: `CompileFailureClass::InvalidRequest { rule: "compile.request.numerics.inapplicable" }`.

**Boundary — stale claims outside this branch's scopes, filed rather than edited.** Two comments in `crates/tiler-conformance` and five documents under `docs/` state the removed `dtype-f32` rule, and two of them cite compiler tests renamed with it. `correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate` and `correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents` carry the exact sites and the corrected statement. The `docs/dtype-support.md` BF16 support-matrix row is named in the second.

**What was not done.** BF16 optimizer legality, which `establish-bf16-optimizer-legality` owns and whose absence is the one remaining boundary between this and a multi-occurrence BF16 plan. The conformance run through `compile()`, which is its own ticket.
