---
id: refuse-nothing-legal-on-the-explain-detail-ceiling
title: Refuse nothing legal on the explain detail ceiling
status: in-progress
priority: p2
dependencies: []
related: [widen-the-identity-growth-ladder-to-the-governed-operation-budget, carry-the-exhausted-resource-through-the-budget-refusal]
scopes: [implementation/compiler, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [defect, explain, compiler, limits]
claimed_from: todo
assignee: agent-explain-ceiling
lease_expires_at: 1786073446
---
## User-visible outcome

A semantic program that satisfies every governed budget compiles, or is refused with a class that names the caller's problem. It is not refused with `CompileFailureClass::InvalidCompilerOutput`, which the public documentation defines as "always a defect in Tiler rather than in the caller's program".

## The measurement

**Measurement, 2026-08-06, `spikes/program-planning/identity-growth`, Apple M4 Max, macOS 27.0 `26A5388g`, toolchain `nightly-2026-07-19`.** An eleven-operation chain — one `tiler::constant-f32@1` and ten `tiler::multiply-f32@1` over a rank-1 extent-4 `f32` tensor, compiled through `tiler_compiler::session::compile` under `FLUSH_SUBNORMALS_TO_ZERO_F32` against the authoritative macOS Apple9 declaration — refuses:

```
CompileFailure { class: InvalidCompilerOutput, explain: "3478 records" }
```

whose terminal record is `compiler-failure:explain-detail-capacity`. The same generator at ten operations verifies, and at twelve refuses `NoFeasiblePlan` for an unrelated reason (`region-expansion-exhaustion-loses-the-only-feasible-plan`).

**Fact — nothing about the program is out of budget.** Region formation succeeds at eleven operations with `budget-stops:count=0`, `candidate-count:count=66`, `rejected-non-convex:count=1013`. `semantic_operations` is 62, `semantic_values` 80, `regions` 12. The refusal is downstream of all of them.

**Fact — the ceiling.** `MAX_RECORDS = 4_096` and `MAX_CANONICAL_BYTES = 1024 * 1024` (`crates/tiler-compiler/src/explain.rs:114-115`). `ExplainWriter::push_detail` returns `ExplainError::DetailCapacity` when a detail record would exceed either, and `pipeline.rs:2092` maps that to `"detail-capacity"`. The bound is deliberate and its reason is stated in the source: "A trace is complete or it is refused. Exceeding a bound is a typed failure, never a silent drop: a reader who cannot tell which records are missing cannot rely on the ones that remain."

**Inference — what exhausts it is cover enumeration, and it grows with the operation count.** 2,300 of the 3,478 records are `candidate-enumeration rejected-intrinsic rule=selection.region-coverage.v1@1 … event=check:selection.region-implemented:disproved:region-unimplemented`, one per rejected region cover against a single unimplemented singleton region. That population grows combinatorially in the operation count while the ceiling is a constant, so the ceiling is a program-size limit that no budget declares and no caller can read.

## Why the refusal discipline is not the thing to change

The complete-or-refused property is correct and this ticket does not propose weakening it. What it proposes deciding is one of:

- the detail ceiling is sized against the program sizes the governed budgets admit, rather than against a constant chosen before the budgets moved;
- the rejection population is summarized at its source (one record per rejection *class* with a count, rather than one per rejected cover), which is a change to what the trace says and therefore an explainability decision rather than a capacity one; or
- the ceiling refuses under a class that is not `InvalidCompilerOutput`, because a caller who reads that class is told Tiler is broken when what happened is that Tiler declined to explain itself at length.

The third is the smallest and the least satisfying: it renames a refusal without moving the wall. The first and second both move the wall and both touch `docs/compiler/optimizer.md`'s requirement that explain output "never collapses these into 'not fused'".

## Why it matters

The governed `semantic_operations` budget was raised to 62 on 2026-08-05, sized "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit". This ceiling refuses at eleven. Any work that assumes the budget describes the reachable domain is wrong by a factor of about six, and the identity-growth ladder is the first thing to have measured it.

## Evidence

- `spikes/program-planning/identity-growth/results/2026-08-06-apple-m4-max-macos27.0-26A5388g/growth.tsv`, wall table, first row.
- `cd spikes/program-planning/identity-growth && cargo run --release` reproduces it; the wall is probed on every run and fails loudly if it moves.

## Outcome — 2026-08-06: option 2, summarized at its source

**Decision: the rejection population is summarized at its source.** `select_physical_plans` retains one `PlanRejection::RegionUnimplemented` per unimplemented *region*, carrying `covers: u64` — the number of distinct legal covers that region left uncompletable — instead of one rejection per (cover, region) pair. `record_coverage_gaps` emits one `selection.region-coverage.v1` record per rejection, with subjects reduced to the region alone and a new `blocked-covers` count fact beside the existing `region-role`.

### The derivation

**Fact — the ground is the region's, not the cover's.** `entry.admitted.is_empty()` reads the frontier bound to one region occurrence. `pipeline/planning.rs` memoizes frontier enumeration per region subject and states the governing invariant at that site: "One region subject yields one explain subject, so its frontier and any rejection it carries are recorded exactly once however many covers place that same region." The coverage-gap rule was the single site that broke it. Two covers placing the same region were refused on the identical ground, so the multiplicity was a quantity and not a set of distinct grounds.

**Fact — the cover subject named nothing the trace describes.** The second subject was `region-cover:<16 hex>`, produced by `PlanRejection::cover_label`. `grep -rn "region-cover:" crates/` finds it constructed in `cover.rs` and `selection.rs` and consumed in exactly one place — this record. No other trace record names a cover, so a reader could not resolve the digest to a partition. `cover_label` had no other caller and is removed.

**Inference — this satisfies `docs/compiler/optimizer.md`.** Its requirement is that "every rejection records its stage, stable reason code, rule/provider identity, affected operation/value or candidate, failed predicate/evidence, and whether the result is a hard rejection, safe deferral, budget stop, dominance pruning, or cost disadvantage. Explain output never collapses these into 'not fused.'" Every one of those fields still has its own record per distinct value. What is removed is a repetition of one identical tuple under a subject the trace does not otherwise describe, and the `blocked-covers` count preserves the only thing that repetition said.

**The complete-or-refused property survives untouched.** No record is dropped after being decided; `push_detail` still refuses rather than truncating, and `MAX_RECORDS`/`MAX_CANONICAL_BYTES` are unchanged. What changed is the population defined at the source, deterministically and before the writer sees it.

### Correction to this ticket's Fact — it was the byte ceiling, not `MAX_RECORDS`

**Inference, from the ticket's own measurement.** The refused compilation sealed 3,478 records. `finish_failure` appends one terminal record to the details already retained, so 3,477 detail records were retained when the next `push_detail` was refused. `push` refuses on records when `retained_detail_records + 1 > MAX_RECORDS`, and `3,478 ≤ 4,096`, so the record bound did not trip: **`MAX_CANONICAL_BYTES` (1 MiB) is what refused**, at a mean of about 302 bytes per record. This matters for the rejected option 1: sizing the ceiling would have to move the byte bound, and `MAX_TRACE_CANONICAL_BYTES` derives from it.

### Options rejected

- **Option 1, size the ceiling to the admitted program sizes.** The coverage-gap population is bounded only by `region_covers` (1,024) times the regions one cover may hold (at most `semantic_operations`, 62), so 63,488 records for that one rule, plus a `BoundaryDisagreement` population bounded by `physical_plan_combinations`. At the measured ~302 bytes per record that is a **19 MB canonical trace for a single refusal**, and `MAX_TRACE_CANONICAL_BYTES` doubles it. It also buys a reader nothing: 63,488 restatements of one tuple under opaque digests. And it would still be a constant sized against today's budgets, which is precisely how 4,096 became a program-size limit no budget declares.
- **Option 3, reclassify the refusal.** Renames the wall without moving it: the eleven-operation program still would not compile, and a caller told `BudgetExhausted` or a new class would be told a program inside every budget is too large. The ticket names it the least satisfying and it is.

**No public boundary moved and no decision was Tom's.** `PlanRejection`, `SelectedPortfolio`, `ExplainWriter`, and every explain type touched are `pub(crate)`; the trace leaves the crate only as an opaque `VerifiedCompilationExplain` and a rendered string that ADR 0074 and `docs/compiler/optimizer.md` both refuse as a parse target.

**Neither explain version steps.** `EXPLAIN_SCHEMA_VERSION` versions the encoding, and the encoder is untouched: a record with one subject and a record with two were both encodable before, so no previously encodable record's bytes moved or are reinterpreted. `EXPLAIN_RENDERER_VERSION` versions the spelling, and the renderer is untouched: it writes subjects and facts generically, so no existing record's spelling changed. Only which records this build emits changed, which the file's own ledger comment states is exactly what a version does not promise.

### The measurement, before and after

| | before | after |
| --- | --- | --- |
| 11-operation chain, spike profile (`FLUSH_SUBNORMALS_TO_ZERO_F32`, macOS Apple9) | refuses `InvalidCompilerOutput`, terminal record `compiler-failure:explain-detail-capacity`, 3,478 records | **compiles**, kernel-program identity 39,502 bytes |
| 11-operation chain, in-crate governed profile | refuses `InvalidCompilerOutput(Explain(DetailCapacity))` | **compiles**, 65 coverage-gap records accounting for 6,143 (cover, region) pairs |
| governed 5-operation fixture | 38 `selection.region-coverage.v1` records | 14 records, `blocked-covers` summing to 38 |
| 12-operation wall trace | 198 records | 146 records, class unchanged (`NoFeasiblePlan`) |
| 62-operation wall trace | 638 records | 461 records, class unchanged (`NoFeasiblePlan`) |

**No identity moved, measured rather than argued.** Every structural column of the spike's ladder at 2..=10 operations — coverage records, graph identity, kernel-program identity, widest alternative, coverage bytes, stages, alternatives — is byte-identical between the retained pre-fix and post-fix TSVs. Explain records are diagnostics and are not identity content; `encode_portfolio_identity` reads plans only and never rejections. The whole workspace suite passes with no golden, fixture count, or pin rebaselined.

### The spike, re-derived

`spikes/program-planning/identity-growth` was re-derived as the ticket's own discipline demands. `WALLS` loses its eleven-operation entry; `OPERATIONS` becomes 2..=11; the run exits 0 with the remaining three walls confirmed at their unchanged classes, and all four `--perturb` modes still exit non-zero. The retained result is a new path, `results/2026-08-06-post-explain-ceiling-apple-m4-max-macos27.0-26A5388g/growth.tsv`, because it shares its date and host with the run this ticket was filed from and differs only in the compiler tree.

**The fit is unchanged and gained its only out-of-domain confirmation.** `program_bytes(n) = 3525n + 727`, residual 0 at all ten points. `3525·11 + 727 = 39,502` was a prediction about a program the compiler refused; the program now compiles to 39,502 bytes exactly. No coefficient moved, so the 19,038-operation refusal point, the 148/149 embedding crossing, and every P1/P2/P3 figure are confirmed unchanged.

### Scope and follow-up

`research/program-planning` was added to this ticket (`tkt set … --add-scope research/program-planning`) because the fix moves the spike its measurement came from; `tkt why` reports no conflict with either live sibling claim. `docs/research/program-planning/complete-model-ingestion-and-execution.md` is inside that scope and is corrected here. Four records that state the ladder's domain are not — `docs/artifact-abi.md:247`, `docs/ir.md:1138`, `docs/decisions/0104-…`, and `docs/research/artifacts/manifest-fixed-content-growth.md` — and each still asserts nine points over 2..=10, no out-of-domain confirmation, and the explain detail ceiling as an open defect. They are filed with their exact replacement statements as [`carry-the-restored-ladder-point-into-the-four-records`](carry-the-restored-ladder-point-into-the-four-records.md), the same carrier shape the previous ladder change used.

The twelve-operation `NoFeasiblePlan` wall is untouched and remains [`region-expansion-exhaustion-loses-the-only-feasible-plan`](region-expansion-exhaustion-loses-the-only-feasible-plan.md). It is now the only wall between the ladder and the governed budget that is not about program size.

### Checks

New checks watched failing first: `an_eleven_operation_chain_inside_every_budget_compiles` fails on base `09562bab` with `InvalidCompilerOutput(Explain(DetailCapacity))`; the rule census failed 38 against 14; the spike's own wall probe fired (`THE WALL MOVED: 11 operations compiled to a 39502-byte identity`).

```sh
cargo fmt --all --check
cargo clippy -p tiler-compiler --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler
make full
tkt lint
git diff --check
tkt guard tkt/refuse-nothing-legal-on-the-explain-detail-ceiling --format json
cd spikes/program-planning/identity-growth && cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings
cd spikes/program-planning/identity-growth && cargo run --release
cd spikes/program-planning/identity-growth && cargo run --release -- --perturb=program|coverage|fit|wall   # each exits 1
```
