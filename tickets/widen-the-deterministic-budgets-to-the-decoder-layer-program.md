---
id: widen-the-deterministic-budgets-to-the-decoder-layer-program
title: Widen the deterministic budgets to the decoder-layer program
status: done
priority: p1
dependencies: [assemble-the-decoder-layer-program, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, enumerate-the-split-reduction-on-the-planning-frontier, prototype-public-compiler-api, correct-the-l6-budget-refusal-item, bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals, correct-the-l6-budget-item-for-the-output-derived-budgets]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, budgets, identity, language-model, class-generic-capability]
---
## User-visible outcome

A compilation request carrying a complete decoder layer is admitted rather than refused for its size, so the refusals a transformer meets are about what it computes rather than about how many operations it has.

## Why this exists

**~~Fact~~ — historical filing premise; false as a live claim.** At filing this section said `verify_program` at `crates/tiler-compiler/src/request.rs:1886` checked four resources against `DeterministicBudgets::governed()`: `semantic_values` 16, `semantic_operations` 8, `regions` 3, `host_expression_nodes` 32, and `buffers` 4 against `4.max(program.input_count().saturating_add(1))`. That line pin, the buffers bound of 4, and the `4.max` actual are all wrong even for the pre-widening tree this ticket closed against (see Outcome premise correction below); treat the sentence as the L6-inherited problem statement only.

**Correction — 2026-08-10.** Live admission is still `verify_program` → `check_program_budgets` against `DeterministicBudgets::governed()`, but the function site is not `:1886`, and live governed bounds for the five program-scoped resources are `semantic_values` 80, `semantic_operations` 62, `regions` 12, `host_expression_nodes` 51, `buffers` 30 — this ticket landed 80/62 and the host/buffer widenings to 43/21 with regions left at 3; later [`bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals`](bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals.md) owns 51/30/12 and the output-aware actuals. Reproduce: `rg -n 'fn verify_program|fn check_program_budgets|pub(crate) const fn governed' crates/tiler-compiler/src/request.rs` and read the `Self { semantic_values:` block.

**Inference (historical L6 sizing).** [The L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) derived that the decoder-layer program exceeded three of the four — eighteen inputs against a buffers actual of nineteen, at least twenty-one values, and at least fifty-one operations — while the embedding and vocabulary-projection programs pass every one. The Outcome below shows the layer actually exceeded **four** resources under the true pre-widening actuals.

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

**Fact — the four program-scoped bounds moved and `regions` did not (at landing).**

**Correction — 2026-08-10.** The table's "Landing" column is the governed surface on commit `62c63061` (2026-08-06), not live `DeterministicBudgets::governed`. Landing was `semantic_values` 80, `semantic_operations` 62, `regions` 3, `host_expression_nodes` 43, `buffers` 21 under input-only host/buffer actuals (`input_count * 2 + 7`, `input_count + 3`) and constant regions `3`. Live values and actual formulas are owned by [`bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals`](bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals.md) (commit `8a78f079` and its predecessors): 80/62 remain from this ticket; live `regions` **12**, `host_expression_nodes` **51**, `buffers` **30**, with actuals `output_count * 4`, `input_count * 2 + output_count * 4 + 3`, and `input_count + output_count * 4`. Reproduce: `rg -n 'semantic_values: 80|semantic_operations: 62|regions: 12|host_expression_nodes: 51|buffers: 30' crates/tiler-compiler/src/request.rs`; `rg -n 'output_count\(\)\.saturating_mul\(4\)' crates/tiler-compiler/src/request.rs` inside `check_program_budgets`.

| Budget | Was | Landing (`62c63061`) | Derivation at landing |
| --- | --- | --- | --- |
| `semantic_values` | 16 | **80** | The C1 decode row's eighteen declared inputs plus one result per occurrence, because no occurrence in the layer produces more than one value. The prefill row is `18 + 58 = 76` by the same arithmetic, so eighty bounds both rows. Still live. |
| `semantic_operations` | 8 | **62** | The C1 decode row's occurrence count. It exceeds the prefill row's 58 because at `T = 1` six position-axis rank pads duplicate nothing, so `tiler::broadcast-f32@1` refuses a many-to-one relation onto an extent-one result axis and the layer spells those widenings as further occurrences. Still live. |
| `host_expression_nodes` | 32 | **43** | At landing, `check_program_budgets` derived the actual as `input_count * 2 + 7`, so eighteen inputs reach `2 × 18 + 7`. Superseded live: bound 51, actual `input * 2 + output * 4 + 3`. |
| `buffers` | 6 | **21** | At landing, the same function derived the actual as `input_count + 3` — every declared input, the prologue's temporary, a split's staged partial tensor, and the output — so eighteen inputs reach `18 + 3`. Superseded live: bound 30, actual `input + output * 4`. |
| `regions` | 3 | **3** | Unchanged at landing; see inference below. Superseded live: bound 12 (`3 × 4` for the layer's three outputs), actual `output_count * 4`. |

The counts are [`assemble-the-decoder-layer-program`](assemble-the-decoder-layer-program.md)'s measurements, cited rather than re-derived — eighteen inputs at both rows, 58 occurrences over 76 values at C1 prefill, 62 over 80 at C1 decode. Each value is the largest row's own number rather than the smallest that passes, which is the rule the `governed` comment states and the rule the split reduction's `2 → 3` and `3 → 4 → 6` steps followed.

**Inference — `regions` stayed 3 at landing because a region count is a property of a plan, and this profile planned no layer.** At landing, `check_program_budgets` checked `regions` against the constant 3 — the split program's pointwise, partial, and final stages — not against anything the submitted program declares, so the layer never pressed it. There was no layer region count to derive while `select_supported_strategy` refused the layer, and a number invented here would have been exactly the "smallest that passes" reasoning inverted. **This was a second identity move deferred rather than avoided** under D-18's "one widening" — sized to what was derivable then, not a promise that no budget would move again. **Correction — 2026-08-10.** Multi-output admission later made a bare constant refuse nothing useful for multi-output programs; [`bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals`](bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals.md) derived regions as `4 × outputs` and set governed to 12 for three outputs — an identity move for plan-shape reasons, not because the real layer became plannable. The measurement boundary that the layer itself is still not compiled remains true.

**Inference — `normalization_rewrites` and every `region_*` bound are unchanged** because none admits or refuses a program: each bounds a *search*, and exhausting one costs an alternative while the verified input and complete coverage survive.

### Two facts in this ticket's own premise were wrong, and both were read from the source

**Fact.** The "Why this exists" section above (inherited from [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md)) states that `buffers` was 4 and is checked against `4.max(input_count + 1)`, and that the layer exceeds *three* of the four resources. `check_program_budgets` refutes both: `buffers` was already **6** on the base commit, it is checked against `input_count + 3` (21 for eighteen inputs, not 19), and `host_expression_nodes` is checked against `input_count * 2 + 7` — **43** against a bound of 32, so it refused the layer too. The layer exceeded **four** resources, and the fourth is one this ticket's premise lists as passing. Reproduce with `sed -n '/fn check_program_budgets/,/^}/p' crates/tiler-compiler/src/request.rs`. Filed as [`correct-the-l6-budget-refusal-item`](correct-the-l6-budget-refusal-item.md); the record is outside this ticket's scopes and nothing was edited there.

### The refusal still bites, observed per resource

**Measurement — `request::tests::each_widened_budget_refuses_the_program_one_step_past_it`, 2026-08-06 (landing matrix).** The probe declares `n` inputs over `m` `f32` adds, so `value_count() == n + m`, which is the layer's own identity. Both measured rows were admitted at `check_program_budgets` — `18/62` and `18/58`, the decode row sitting *on* all four bounds under the landing actuals — and each one-step-past program was refused through `verify_program`:

| Program | Refusal at landing |
| --- | --- |
| 19 inputs, 62 occurrences (81 values) | `BudgetExceeded { resource: "semantic-values", limit: 80, actual: 81 }` |
| 17 inputs, 63 occurrences | `BudgetExceeded { resource: "semantic-operations", limit: 62, actual: 63 }` |
| 19 inputs, 18 occurrences | `BudgetExceeded { resource: "host-expression-nodes", limit: 43, actual: 45 }` |
| 19 inputs, 18 occurrences, `host_expression_nodes` widened to 45 | `BudgetExceeded { resource: "buffers", limit: 21, actual: 22 }` |

**Correction — 2026-08-10.** The live test still admits both measured rows (with three outputs) and still refuses semantic-values at 81 and semantic-operations at 63, but the host/buffer matrix is the multi-output one: host limit 51 / actual 53, buffers limit 30 / actual 31 with host perturbed to 53, plus a four-output regions probe (limit 12 / actual 16). Read `each_widened_budget_refuses_the_program_one_step_past_it` in `crates/tiler-compiler/src/request.rs`.

**Fact — `buffers` needed that perturbation at landing, and the reason was structural rather than a test convenience.** Both input-derived bounds were tight at exactly eighteen declared inputs under the landing formulas, so their thresholds coincided: a nineteen-input program exceeded `host-expression-nodes` and `buffers` together and the earlier check reported. The perturbation widened only the bound that shadowed `buffers`; the `buffers` value observed refusing was the governed one. The same shape explains the first row: `semantic_values` is exactly `18 + 62`, so one more value is one more input or one more occurrence and no program exceeds it alone — which resource is reported is the check order's guarantee.

**Measurement — every assertion was watched failing (landing).** Five perturbations of `governed()`, each run against the new test alone, each failing at the intended assertion and nowhere else: `semantic_values` `80 → 81` (the semantic-values row then reports `host-expression-nodes`); `semantic_values` `80 → 79` (the admitted `18/62` row is refused); `semantic_operations` `62 → 63` (the refusal becomes `None` — `verify_program` admits the probe outright); `host_expression_nodes` `43 → 45` (the row reports `buffers`); `buffers` `21 → 22` (the refusal becomes `None`). Each was reverted before the next.

### The identity movement, and every moved pin

**Fact.** Every budget is written into `VerifiedRequestSubject::canonical_explain_subject_bytes` — twelve `u32` fields then two `u64` fields, `crates/tiler-compiler/src/request.rs` — so this change moves **every governed compilation's request subject, and therefore every artifact identity and cache entry derived from it**, including for programs nowhere near any of the four bounds. That is the stated consequence of a budget being a property of the *request* rather than of the plan chosen for it.

**Fact — no version constant owned this as a rendering change, determined at its owning site (landing).** At landing the request-subject domain tag was `b"tiler.compiler.request-subject.v5\0"`, written at the head of `canonical_explain_subject_bytes`; the budget field set, their widths, and their order were untouched, so a value change stayed injective inside `v5` and the domain did not step. `EXPLAIN_RENDERER_VERSION` stayed 7 for the same reason the softmax fact correction gave: the rendering is unchanged and only the digest value moves. Neither is a step this change may take. **Correction — 2026-08-10.** The live domain tag is `tiler.compiler.request-subject.v6` for a later shape-environment subject step, not this ticket; `EXPLAIN_RENDERER_VERSION` remains 7. Reproduce: `rg -n 'request-subject\.v6|EXPLAIN_RENDERER_VERSION' crates/tiler-compiler/src/`.

**Measurement — the pin survey, run before editing, and the two pins that moved (landing).** The population was surveyed with `grep -rEon '\b[0-9a-f]{16}\b' crates/` (44 hits across 14 files) and `grep -rEon '\b[0-9a-f]{64}\b' crates/` (33 hits), then the whole-workspace run was taken as the check that nothing else folds these bytes. Two pins moved on this ticket's landing, both in `crates/tiler-compiler`:

1. `explain::tests::deterministic_trace_is_sealed_and_rendered_separately`'s request qualifier, **`a95ad77532352d7f` → `8e06e11fdc3a2889`**, rebaselined from the observed failing value with its ledger comment in the same commit. Reproduce with `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` and read the reported `left`. This ticket's premise inherited "no pinned golden encodes these bytes" from the `governed` comment; that sentence was already false when the `buffers` `4 → 6` step moved this same digest, and it is corrected in the rewritten comment rather than repeated. **Correction — 2026-08-10.** Later identity/budget moves rebaselined the same pin past landing; live sealed-trace pin is `request=7ba3d77a66f04638`.
2. `pipeline::tests::the_widened_budgets_admit_the_split_program_and_still_refuse_a_narrower_request` asserted `request.budgets().buffers`, **`6` → `21`**. Not an identity digest — a direct value assertion — and its companion assertions are what prove the widening did not remove the check: the same test still drives narrow `regions`/`buffers` to a named `BudgetExceeded`. **Correction — 2026-08-10.** Live assertions track the later multi-output governed surface (`buffers` 30, `regions` 12).

**Fact — nothing else moved at landing.** `cargo nextest run --workspace` was green at 2680 tests, and no golden, digest, or fixture outside those two required an edit on this commit. The 64-hex population is accuracy-contract, artifact-codec, and Metal-plan digests; the 16-hex population is Metal kernel/scheduled-region digests in `crates/tiler-metal/goldens/*.metal` and explain-local digests — none reached by the request subject, which the green whole-workspace run is the check for.

### Scope

Three compiler sources under `crates/tiler-compiler/**` (`src/request.rs`, `src/explain.rs`, `src/pipeline/tests.rs`) and two under `tickets/**`. **Correction — 2026-08-10.** The original sentence said "Two files" while enumerating three paths; the landing touch set is those three sources plus tickets. `contracts/optimizer` was declared and unused at close: at that time the ticket treated `docs/compiler/**` as carrying no program-scoped budget value. That rationalization is stale for live docs — `docs/compiler/optimizer.md` now discusses `DeterministicBudgets` and program-scoped bounds extensively — but it is not a live scope violation on a done ticket.

### Measurement boundary

Everything here is the request boundary on one host. Nothing is established about whether the layer *compiles*: the recognizer refuses it under its own named rules, which is the L6 record's refusal 2 with its own owners, and this change deliberately does not touch it. The admitted probes are `f32` add chains with the layer's counts, not the layer; the layer itself has never been submitted to `verify_program`, because [`assemble-the-decoder-layer-program`](assemble-the-decoder-layer-program.md) was forbidden to compile it and nothing since has.

### Commands run

`cargo fmt --all --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-compiler --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps`; `cargo nextest run --workspace` — 2680 passed, 7 skipped; `cargo test --workspace --doc`; `tkt lint`; `git diff --check`; `tkt guard`; `make full`.

## Fact audit — 2026-08-10

- **Landing vs live governed surface.** This ticket landed `80/62/3/43/21` under input-only host/buffer actuals and constant regions. Live `DeterministicBudgets::governed` is `80/62/12/51/30`; host/buffer/regions actuals and the further widenings are owned by [`bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals`](bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals.md) (and related multi-output work). Semantic values/operations from this ticket remain.
- **Why premise.** The inherited L6 Fact (line `:1886`, buffers `4`, `4.max(input+1)`) is struck above; pre-widening tree already had buffers 6 and `input+3` actuals, and `host_expression_nodes` also refused the layer.
- **Request-subject domain.** Landing kept `v5`; live domain is `v6` for a later shape-environment subject step, not this ticket's budget value change.
- **Explain pin.** Landing moved `a95ad77532352d7f` → `8e06e11fdc3a2889`; live sealed-trace pin is past that (`request=7ba3d77a66f04638` at audit). `EXPLAIN_RENDERER_VERSION` remains 7.
- **Scope count.** Landing touched three compiler sources, not two.
- **Graph.** `related` now includes the later multi-output budget owners for supersession navigation; they are not reverse dependencies of this ticket.
- **Closure.** Status `done` stands; user-visible size admission for the four resources this ticket moved remains achieved; later identity moves do not reopen this ticket.
