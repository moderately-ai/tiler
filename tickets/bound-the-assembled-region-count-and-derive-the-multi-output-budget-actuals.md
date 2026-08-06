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
