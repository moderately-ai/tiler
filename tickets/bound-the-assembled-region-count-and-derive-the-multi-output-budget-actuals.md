---
id: bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals
title: Bound the assembled region count and derive the multi-output budget actuals
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-budgets
lease_expires_at: 1786031910
---
## User-visible outcome

The `regions` budget refuses a plan that exceeds it instead of bounding nothing, and the `buffers`/`host-expression-nodes` actuals are derived over every declared output, so a request whose widest plan exceeds its stated budget is refused at the boundary under `BudgetExceeded` rather than dying at assembly as `ProgramError::Storage { rule: "buffer-budget" }` — a caller's request reported as a compiler-output defect, the exact failure the derivation comment says it exists to prevent.

## Why this exists (audited 2026-08-06 at 7bf73713, coordinator-verified by direct read)

**Fact — `regions` bounds nothing.** `check_budget("regions", budgets.regions, 4)` at `crates/tiler-compiler/src/request.rs:3616` compares the caller's budget against a literal; `grep -rn 'budgets\.regions' crates/` shows the only non-test readers are that call and the subject encoder (`request.rs:2241`). `program.rs:1850/:1861` enforce `host_expression_nodes` and `buffers` plan-side; no site enforces `regions` against any plan. The stating sentence — "the widest plan this profile assembles is that chain whatever the submitted program declares" — was true under `output-arity` and is false since multi-output admission: two independent epilogue chains pass every other budget and assemble six stages against a governed value of 4.

**Fact — the `buffers` and `host-expression-nodes` actuals derive from `input_count()` alone** (`request.rs:3625-3641`), and both derivation comments enumerate exactly one output. A program with N declared outputs mints N output values plus per-edge temporaries, so the actuals under-report and the refusal moves to the wrong stage with the wrong error class.

## The work

One sequenced change (both halves touch `check_program_budgets` and the second moves the request-subject pin): (a) derive both input-scoped actuals over `input_count()` and `output_count()`; (b) give `regions` a real actual — the sum over declared outputs of each output's widest producer chain — enforced at the boundary and plan-side beside its siblings; (c) move the governed value to what (b) derives for the widest admitted program, with the derivation in the comment per the budgets idiom; (d) correct both stating sentences. The identity step executes completely: every budget is in `canonical_explain_subject_bytes`, so the explain qualifier moves — recompute from the observed failing value with the ledger, per the two recompute precedents.

## Failure perturbations required

A two-chain program refused by name under the pre-widening budget and compiling after; a two-output program exceeding a stated `buffers` budget refused at `verify_request` with `resource == "buffers"` and shown NOT reaching `verify_host_contract`; the single-output split-plus-epilogue keeping its verdict in both directions.

## Closes when

Both actuals are output-aware, `regions` is enforced against assembled plans, the governed values carry their derivations, the moved pin is enumerated and recomputed, and all three perturbations are observed.

## Outcome

All four parts landed together, plus one correction the work measured.

**(a) Both input-scoped actuals are output-aware.** `check_program_budgets` now derives `host-expression-nodes` as `2 × inputs + 4 × outputs + 3` (three program-scoped nodes — element width, workgroup width, applicability guard — two per declared input, and per declared output its own pair plus its chain's staged partial tensor's pair) and `buffers` as `inputs + 4 × outputs`. Each reduces to today's value at one output, which is the check that the generalization is the same enumeration rather than a new one.

**(b) `regions` has a real actual: `4 × outputs`,** enforced at the boundary and plan-side. Four is the measured widest producer chain one output reaches — prologue, partial, final, epilogue — and the outputs' walks partition the occurrences (`check_output_cover`), so their chains are disjoint region sets and the plan's stage count is their sum. The plan-side site is in `verify_host_contract` beside `host-expression-budget` and `buffer-budget`, under `ProgramError::Structure { rule: "region-budget" }` against `program.core.stages().len()` — the same count as the scheduled regions by `verify_kernel_program_layers`' `cardinality` rule, deliberately not the cover's smaller `region_count()`.

**(c) The governed values moved to that derivation over the decoder layer**, whose measured row is eighteen declared inputs and **three** ordered named outputs (`crates/tiler-reference/tests/decoder_layer.rs`, `assert_eq!(program.output_count(), 3)` at `:1666` and `:1749`; L6 delivery item 2 states the same). `regions` 4 → 12, `host_expression_nodes` 43 → 51, `buffers` 21 → 30. **Moving the latter two was required rather than elective:** the doc's stating sentence is that the program-scoped bounds are sized to that layer, and a one-output derivation over a three-output layer stops admitting it — which would silently un-discharge L6 item 1's discharged budget refusal.

**Correction the work measured — the `buffers` per-output term is four, not three.** The old enumeration stopped at the split's staged partial tensor and missed the fold's staged result an elementwise epilogue reads across. **Measurement:** the widest plan for `sum(x * x, axis 1) * 2.0` over one declared input binds **five** values against a derived four, and two chains over two declared inputs bind **nine** against a derived ten. So the pre-existing one-output derivation already under-reported by one — the same defect class this ticket names, found by measuring rather than by reading the vocabulary. Pinned by `crate::pipeline::tests::the_widest_assembled_plan_binds_four_buffers_per_declared_output`, which states a budget one below each measured count and requires the *boundary* to report.

**(d) Both stating sentences corrected.** `check_program_budgets`' "a region count belongs to a plan, and the widest plan this profile assembles is that chain whatever the submitted program declares" and `DeterministicBudgets::governed`'s "`regions` is `4`, and it is derived from a measurement rather than from the decoder layer" both now record that the ground survives and its conclusion did not: a plan covers every declared output, so a plan-scoped constant is still a function of the declaration, and the two were indistinguishable only while recognition could name one output.

**Moved pin.** `crates/tiler-compiler/src/explain.rs` request qualifier of `deterministic_trace_is_sealed_and_rendered_separately`: `689c3aefc30f48d3` → `8966151e455093ea`, recomputed from the observed failing value on this branch tree (the only failure in a 701-test package run), with the ledger paragraph in the same commit. No encoding version moved — the budget field set, widths, and order in `canonical_explain_subject_bytes` are untouched, so a value change stays injective inside `tiler.compiler.request-subject.v5`. No other pinned identity moved: the full package run was otherwise green, and the plan-side `region-budget` site is in no subject.

## Perturbations observed

1. **Regions actual reverted to the literal `4`.** `the_two_chain_program_is_refused_by_regions_until_the_budget_admits_both` fails with `left: None` against `right: Some(BudgetExceeded { resource: "regions", limit: 4, actual: 8 })` — the boundary *admits* the two-chain program, which is the defect. `each_widened_budget_refuses_the_program_one_step_past_it` fails alongside, reporting `host-expression-nodes` where `regions` should.
2. **`buffers` per-output term reverted to three.** `a_two_output_program_over_its_buffer_budget_is_refused_at_the_request_boundary` fails at its `expect_err` (the narrowed request compiles), and `each_widened_budget_refuses_the_program_one_step_past_it` reports `UnsupportedCapability { phase: "strategy", rule: "output-partition-overlap" }` where `BudgetExceeded { resource: "buffers", limit: 30, actual: 31 }` is required.
3. **Single-output split-plus-epilogue, both directions.** Admitted under the governed budgets with retained stage counts `[3, 4]` under a reassociating contract and `[3]` under a reassociation-forbidding one; refused under a stated `regions` of three with `BudgetExceeded { resource: "regions", limit: 3, actual: 4 }`. `4 × 1` is exactly the literal it replaced, so this output's verdict is unchanged in both directions.
4. **Plan-side `region-budget` proven able to refuse.** It is defence in depth: the boundary's actual dominates any plan this build assembles, so no admitted request reaches it. With the boundary's `regions` check disabled and a stated budget of three, the four-dispatch chain reaches it and reports `InvalidCompilerOutput(Program(Structure { rule: "region-budget" }))`.

## Filed

[`refuse-two-structurally-identical-output-chains-by-name-not-as-compiler-output`](refuse-two-structurally-identical-output-chains-by-name-not-as-compiler-output.md) — **Measurement:** two epilogue chains of identical shape fail `compile` with `InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))`, a caller's valid program reported as a compiler defect. Not reproduced when the chains fold different extents, which is why this ticket's two-output fixture folds `[1, 4]` and `[1, 2]`. Out of scope here and possibly a `tiler-ir` change, so it is filed rather than absorbed.

[`correct-the-l6-budget-item-for-the-output-derived-budgets`](correct-the-l6-budget-item-for-the-output-derived-budgets.md) — L6's discharged-budget item states all five derivations and values as a source-read **Fact**; four of the eight statements were already stale before this ticket (`regions` 3 against a source saying 4) and this ticket moved the rest. `docs/research/program-planning/**` is `research/program-planning`, which this ticket does not hold, so the correction is filed rather than taken here.
