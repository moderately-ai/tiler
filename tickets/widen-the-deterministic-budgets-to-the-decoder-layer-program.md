---
id: widen-the-deterministic-budgets-to-the-decoder-layer-program
title: Widen the deterministic budgets to the decoder-layer program
status: done
priority: p1
dependencies: [assemble-the-decoder-layer-program, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, enumerate-the-split-reduction-on-the-planning-frontier, prototype-public-compiler-api, correct-the-l6-budget-refusal-item]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, budgets, identity, language-model, class-generic-capability]
---
## User-visible outcome

A compilation request carrying a complete decoder layer is admitted rather than refused for its size, so the refusals a transformer meets are about what it computes rather than about how many operations it has.

## Why this exists

**Fact.** `verify_program` at `crates/tiler-compiler/src/request.rs:1886` checks four resources against `DeterministicBudgets::governed()`: `semantic_values` 16 against `program.value_count()`, `semantic_operations` 8 against `program.operation_count()`, `regions` 3, `host_expression_nodes` 32, and `buffers` 4 against `4.max(program.input_count().saturating_add(1))`.

**Inference.** [The L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) derives that the decoder-layer program exceeds three of the four — eighteen inputs against a buffers actual of nineteen, at least twenty-one values, and at least fifty-one operations — while the embedding and vocabulary-projection programs pass every one.

## What makes this a decision rather than a knob

**Fact.** Every budget is written into `VerifiedRequestSubject::canonical_bytes` (`crates/tiler-compiler/src/request.rs:1193`–`1210`) and therefore into artifact identity. The comment on `DeterministicBudgets::governed` states the consequence directly: widening moves "every governed compilation's request subject, and therefore every artifact identity and cache entry derived from it — for programs that never assemble a split as much as for ones that do, because the budget is a property of the *request* rather than of the plan chosen for it."

So the number of widenings is itself a cost. **This is L6's D-18 and it is Tom's:** take one widening sized to the layer program, as the split reduction's widening from two regions and three buffers to three and four was taken once and stated, or let the budgets grow with the profile and move every identity each time.

## Required content

- The new values, each justified by the largest program shape this profile may assemble rather than by the smallest it might — the same rule the current comment states.
- A budget is an upper bound: widening admits program shapes and never requires them, and `verify_program` must still refuse a request whose shape needs more.
- The artifact-identity movement is stated in the change, not discovered by a reader later. No pinned golden encodes these bytes, so nothing in the suite will report it.

## Closes when

The values are chosen from the layer program's measured counts, Tom has answered D-18, `RequestError::BudgetExceeded` still fires against a program one step larger than the new bound, and the identity movement is recorded.

## D-18 — answered

Answered by Tom on 2026-08-06 at the live decision review in the coordination session, witnessed first-hand by the coordinator: **one widening, sized to the layer program** — values justified by the largest program shape this profile may assemble, per the split-reduction precedent, with the identity movement stated in the change. The ticket is dispatchable once `implementation/compiler` frees (held by the concatenate fusion-role claim at the time of the answer).

## Outcome

**The budget refusal the decoder layer met is gone, and it is the only refusal that moved.** `DeterministicBudgets::governed` now admits the layer's measured counts, `check_program_budgets` still refuses the program one step past each new bound, and `select_supported_strategy` refuses the layer exactly as it did — which is the point of scoping the widening to size.

### The values, and why each is what it is

**Fact — the four program-scoped bounds moved and `regions` did not.**

| Budget | Was | Now | Derivation |
| --- | --- | --- | --- |
| `semantic_values` | 16 | **80** | The C1 decode row's eighteen declared inputs plus one result per occurrence, because no occurrence in the layer produces more than one value. The prefill row is `18 + 58 = 76` by the same arithmetic, so eighty bounds both rows. |
| `semantic_operations` | 8 | **62** | The C1 decode row's occurrence count. It exceeds the prefill row's 58 because at `T = 1` six position-axis rank pads duplicate nothing, so `tiler::broadcast-f32@1` refuses a many-to-one relation onto an extent-one result axis and the layer spells those widenings as further occurrences. |
| `host_expression_nodes` | 32 | **43** | `check_program_budgets` derives the actual as `input_count * 2 + 7`, so eighteen inputs reach `2 × 18 + 7`. |
| `buffers` | 6 | **21** | The same function derives the actual as `input_count + 3` — every declared input, the prologue's temporary, a split's staged partial tensor, and the output — so eighteen inputs reach `18 + 3`. |
| `regions` | 3 | **3** | Unchanged, and derived rather than omitted: see below. |

The counts are [`assemble-the-decoder-layer-program`](assemble-the-decoder-layer-program.md)'s measurements, cited rather than re-derived — eighteen inputs at both rows, 58 occurrences over 76 values at C1 prefill, 62 over 80 at C1 decode. Each value is the largest row's own number rather than the smallest that passes, which is the rule the `governed` comment states and the rule the split reduction's `2 → 3` and `3 → 4 → 6` steps followed.

**Inference — `regions` stays 3 because a region count is a property of a plan, and this profile plans no layer.** `check_program_budgets` checks `regions` against the constant 3 — the split program's pointwise, partial, and final stages — not against anything the submitted program declares, so the layer never pressed it. There is no layer region count to derive while `select_supported_strategy` refuses the layer, and a number invented here would be exactly the "smallest that passes" reasoning inverted. **This is a second identity move deferred rather than avoided**: `regions` moves when the layer becomes plannable. Recording it is the honest reading of D-18's "one widening" — one widening sized to what is derivable now, not a promise that no budget moves again.

**Inference — `normalization_rewrites` and every `region_*` bound are unchanged** because none admits or refuses a program: each bounds a *search*, and exhausting one costs an alternative while the verified input and complete coverage survive.

### Two facts in this ticket's own premise were wrong, and both were read from the source

**Fact.** The "Why this exists" section above (inherited from [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md)) states that `buffers` was 4 and is checked against `4.max(input_count + 1)`, and that the layer exceeds *three* of the four resources. `check_program_budgets` refutes both: `buffers` was already **6** on the base commit, it is checked against `input_count + 3` (21 for eighteen inputs, not 19), and `host_expression_nodes` is checked against `input_count * 2 + 7` — **43** against a bound of 32, so it refused the layer too. The layer exceeded **four** resources, and the fourth is one this ticket's premise lists as passing. Reproduce with `sed -n '/fn check_program_budgets/,/^}/p' crates/tiler-compiler/src/request.rs`. Filed as [`correct-the-l6-budget-refusal-item`](correct-the-l6-budget-refusal-item.md); the record is outside this ticket's scopes and nothing was edited there.

### The refusal still bites, observed per resource

**Measurement — `request::tests::each_widened_budget_refuses_the_program_one_step_past_it`, 2026-08-06.** The probe declares `n` inputs over `m` `f32` adds, so `value_count() == n + m`, which is the layer's own identity. Both measured rows are admitted at `check_program_budgets` — `18/62` and `18/58`, the decode row sitting *on* all four bounds — and each one-step-past program is refused through `verify_program`:

| Program | Refusal |
| --- | --- |
| 19 inputs, 62 occurrences (81 values) | `BudgetExceeded { resource: "semantic-values", limit: 80, actual: 81 }` |
| 17 inputs, 63 occurrences | `BudgetExceeded { resource: "semantic-operations", limit: 62, actual: 63 }` |
| 19 inputs, 18 occurrences | `BudgetExceeded { resource: "host-expression-nodes", limit: 43, actual: 45 }` |
| 19 inputs, 18 occurrences, `host_expression_nodes` widened to 45 | `BudgetExceeded { resource: "buffers", limit: 21, actual: 22 }` |

**Fact — `buffers` needs that perturbation, and the reason is structural rather than a test convenience.** Both input-derived bounds are now tight at exactly eighteen declared inputs, so their thresholds coincide: a nineteen-input program exceeds `host-expression-nodes` and `buffers` together and the earlier check reports. The perturbation widens only the bound that shadows `buffers`; the `buffers` value observed refusing is the governed one. The same shape explains the first row: `semantic_values` is exactly `18 + 62`, so one more value is one more input or one more occurrence and no program exceeds it alone — which resource is reported is the check order's guarantee.

**Measurement — every assertion was watched failing.** Five perturbations of `governed()`, each run against the new test alone, each failing at the intended assertion and nowhere else: `semantic_values` `80 → 81` (the semantic-values row then reports `host-expression-nodes`); `semantic_values` `80 → 79` (the admitted `18/62` row is refused); `semantic_operations` `62 → 63` (the refusal becomes `None` — `verify_program` admits the probe outright); `host_expression_nodes` `43 → 45` (the row reports `buffers`); `buffers` `21 → 22` (the refusal becomes `None`). Each was reverted before the next.

### The identity movement, and every moved pin

**Fact.** Every budget is written into `VerifiedRequestSubject::canonical_explain_subject_bytes` — twelve `u32` fields then two `u64` fields, `crates/tiler-compiler/src/request.rs` — so this change moves **every governed compilation's request subject, and therefore every artifact identity and cache entry derived from it**, including for programs nowhere near any of the four bounds. That is the stated consequence of a budget being a property of the *request* rather than of the plan chosen for it.

**Fact — no version constant owns this as a rendering change, determined at its owning site.** The request-subject domain tag is `b"tiler.compiler.request-subject.v5\0"`, written at the head of `canonical_explain_subject_bytes`; the budget field set, their widths, and their order are untouched, so a value change stays injective inside `v5` and the domain does not step. `EXPLAIN_RENDERER_VERSION` stays 7 at `crates/tiler-compiler/src/explain.rs:36` for the same reason the softmax fact correction gave: the rendering is unchanged and only the digest value moves. Neither is a step this change may take.

**Measurement — the pin survey, run before editing, and the two pins that moved.** The population was surveyed with `grep -rEon '\b[0-9a-f]{16}\b' crates/` (44 hits across 14 files) and `grep -rEon '\b[0-9a-f]{64}\b' crates/` (33 hits), then the whole-workspace run was taken as the check that nothing else folds these bytes. Two pins moved, both in `crates/tiler-compiler`:

1. `explain::tests::deterministic_trace_is_sealed_and_rendered_separately`'s request qualifier, **`a95ad77532352d7f` → `8e06e11fdc3a2889`**, rebaselined from the observed failing value with its ledger comment in the same commit. Reproduce with `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` and read the reported `left`. This ticket's premise inherited "no pinned golden encodes these bytes" from the `governed` comment; that sentence was already false when the `buffers` `4 → 6` step moved this same digest, and it is corrected in the rewritten comment rather than repeated.
2. `pipeline::tests::the_widened_budgets_admit_the_split_program_and_still_refuse_a_narrower_request` asserts `request.budgets().buffers`, **`6` → `21`**. Not an identity digest — a direct value assertion — and its companion assertions are what prove the widening did not remove the check: the same test still drives `regions: 2` and `buffers: 3` to a named `BudgetExceeded`, and the one-input split program's derived demand of four is unchanged.

**Fact — nothing else moved.** `cargo nextest run --workspace` is green at 2680 tests, and no golden, digest, or fixture outside those two required an edit. The 64-hex population is accuracy-contract, artifact-codec, and Metal-plan digests; the 16-hex population is Metal kernel/scheduled-region digests in `crates/tiler-metal/goldens/*.metal` and explain-local digests — none reached by the request subject, which the green whole-workspace run is the check for.

### Scope

Two files under `crates/tiler-compiler/**` (`src/request.rs`, `src/explain.rs`, `src/pipeline/tests.rs`) and two under `tickets/**`. `contracts/optimizer` was declared and is unused: `docs/compiler/**` states no program-scoped budget value — its deterministic-budget list names only the five `region_*` fields and the three downstream ones, all unchanged. The check is `grep -rn "semantic_values\|semantic_operations\|host_expression_nodes\|DeterministicBudgets" docs/compiler/`, which returns the `region_*` list and the frontier-retention paragraph and nothing this change touches.

### Measurement boundary

Everything here is the request boundary on one host. Nothing is established about whether the layer *compiles*: the recognizer refuses it under its own named rules, which is the L6 record's refusal 2 with its own owners, and this change deliberately does not touch it. The admitted probes are `f32` add chains with the layer's counts, not the layer; the layer itself has never been submitted to `verify_program`, because [`assemble-the-decoder-layer-program`](assemble-the-decoder-layer-program.md) was forbidden to compile it and nothing since has.

### Commands run

`cargo fmt --all --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-compiler --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps`; `cargo nextest run --workspace` — 2680 passed, 7 skipped; `cargo test --workspace --doc`; `tkt lint`; `git diff --check`; `tkt guard`; `make full`.
