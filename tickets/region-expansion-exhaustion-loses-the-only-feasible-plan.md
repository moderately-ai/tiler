---
id: region-expansion-exhaustion-loses-the-only-feasible-plan
title: Region expansion exhaustion loses the only feasible plan
status: done
priority: p2
dependencies: []
related: [widen-the-identity-growth-ladder-to-the-governed-operation-budget, refuse-nothing-legal-on-the-explain-detail-ceiling]
scopes: [implementation/compiler, contracts/optimizer, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [defect, region-search, budgets, compiler]
---
## User-visible outcome

Exhausting a `region_*` search budget costs an alternative, as `DeterministicBudgets::governed`'s own doc comment says it does — or that comment stops saying it. Today, at twelve semantic operations, exhausting `region_expansions` costs the *only* feasible plan and the compilation refuses.

## The contradiction, stated against the source

**Fact — the claim.** `DeterministicBudgets::governed` (`crates/tiler-compiler/src/request.rs`) records why the 2026-08-05 widening moved five program-scoped bounds and no search bound:

> `normalization_rewrites` and every `region_*` bound are unchanged because none of them admits or refuses a program: each bounds a *search*, and exhausting one costs an alternative while the verified input and complete coverage survive.

`region_candidates_per_seed`'s own comment makes the same promise more precisely: "Singleton coverage is emitted before growth starts and is never bounded by this budget, so exhausting it loses fused alternatives rather than the unfused plan."

**Measurement, 2026-08-06, `spikes/program-planning/identity-growth`, Apple M4 Max, macOS 27.0 `26A5388g`, toolchain `nightly-2026-07-19`.** A twelve-operation chain — one `tiler::constant-f32@1` and eleven `tiler::multiply-f32@1` over a rank-1 extent-4 `f32` tensor, through `tiler_compiler::session::compile` under `FLUSH_SUBNORMALS_TO_ZERO_F32` against the authoritative macOS Apple9 declaration — refuses:

```
TargetCompileFailure { failure: CompileFailure { class: NoFeasiblePlan, explain: "198 records" }, refusal: None }
```

The trace's second record is the budget stop and its last is the empty portfolio:

```
1 region-formation budget-stopped rule=region.formation.v1@1 … event=budget-stop:region-expansions:10000:10001 causes=0
22 region-formation admitted rule=region.formation.v1@1 … facts=budget-stops:count=1,candidate-count:count=20,operation-count:count=12,rejected-non-convex:count=1917 causes=21
197 selection compiler-failure rule=compile.failure@1 … subject=kernel-program:portfolio event=compiler-failure:portfolio-empty-without-target-rejection causes=22
```

Eleven operations, one step below, has `budget-stops:count=0` and `candidate-count:count=66` and compiles as far as the explain ceiling. Twelve has one stop and twenty candidates.

**Inference — why the unfused plan does not survive here, which is the part the comment misses.** Singleton coverage does survive: twenty candidates remain and twelve of them are the singletons. But at least one singleton region is `region-role:identity=unrecognized`, so every cover built from singletons alone names an unimplemented region and is rejected by `selection.region-coverage.v1@1`. The only implementable cover is the fused whole-program region, and reaching it is exactly what growth was stopped before doing. So the promise holds only for program families in which the unfused plan is itself implementable, and the comment states it unconditionally.

## What this ticket owes

A decision between two readings, taken with the measurement in hand:

- **The comment is the specification and the behaviour is the defect.** Then either growth toward a whole-program candidate is not charged against the same bound as speculative fusion, or exhausting the bound while no implementable cover exists is refused as `BudgetExhausted` rather than `NoFeasiblePlan` — the caller's action differs ("widen a search bound" against "this target cannot run your program"), and `NoFeasiblePlan`'s own documentation says it "is a hard target rejection, never an exhausted analysis budget", which this refusal violates in terms.
- **The behaviour is correct and the comment is the defect.** Then it is corrected to say that a `region_*` bound can refuse a program whose only implementable cover is a fused one, and the 2026-08-05 decision not to widen the search bounds alongside `semantic_operations` is revisited on that basis, because it was taken on the strength of the sentence being true.

Sizing `region_expansions` is not this ticket's to do unilaterally: every budget is written into the canonical request subject and therefore into artifact identity, so a widening moves every governed compilation's qualifier and the one pinned identity that encodes it.

## Why it matters

`semantic_operations` was raised to 62 on 2026-08-05, sized "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit", explicitly on the ground that the search bounds did not need to move with it. This measurement says they did: the largest program the profile admits by size refuses at region formation, and so does every program from twelve operations up.

## Evidence

- `spikes/program-planning/identity-growth/results/2026-08-06-apple-m4-max-macos27.0-26A5388g/growth.tsv`, wall table, rows two and three — the twelve-operation and sixty-two-operation walls, both `NoFeasiblePlan`.
- `cd spikes/program-planning/identity-growth && cargo run --release` reproduces both on every run and fails loudly if either moves.

## Outcome — 2026-08-07

**The first reading is right in both its arms, and the second reading's residue is a narrower correction than it looked. All three landed.** Commit `71ec71bc`.

### The derivation

The sources decide it against themselves, without appeal to preference.

1. **The cover authority already states this stage's contract, and this stage was not honouring it.** `crates/tiler-compiler/src/cover.rs`'s module documentation says: "**Both the fused and the fully-materialized cover are retained.** The fully-materialized (all-singleton) cover is emitted unconditionally, and the fused (whole-program) cover is emitted whenever region formation admitted a whole-program candidate. Neither can be lost to a budget; the budgets bound only the additional partitions the search discovers." `region_covers`'s own field comment in `request.rs` says the same. That guarantee is **vacuous** unless region formation hands it a whole-program candidate, and formation charged that candidate against `region_expansions` and `region_candidates_per_seed` — while growth is breadth-first over set size from the lowest seed, so the whole-program set is the *last* thing enumerated and the *first* thing an exhausted search loses. The asymmetry is not a design choice anyone made: `retain_singleton_coverage` exists and its twin did not.
2. **The class was wrong in the failure's own terms.** `CompileFailureClass::NoFeasiblePlan` is documented as "a hard target rejection, never an exhausted analysis budget", and the refusal's reason code is literally `portfolio-empty-without-target-rejection`. The two cannot both be right.
3. **The `region_*` bounds are two kinds and the comment named one.** `region_candidates_per_seed` and `region_expansions` bound the search *between* the two coverage extremes. `region_members`, `region_boundary_outputs`, and `region_live_values` bound one region's admissible *shape* — they declare the largest region the profile forms, and a program whose only implementable cover needs a bigger one is refused by them however long the search runs. The 2026-08-05 sentence was true of the first pair and false of the second three, so the comment is corrected rather than deleted, and the widening decision it supported is reopened rather than taken (below).

### What changed

- **`region.rs` — `Formation::retain_whole_program_coverage`.** The whole-program node set is formed before growth starts, charged against no search budget, symmetric with `retain_singleton_coverage`. Unlike singleton coverage a rejection here is a legal outcome rather than a defect — the set can be disconnected, hold an impure member, or exceed a shape bound — so it is tallied exactly as growth tallies one. Growth skips re-emitting a set that covers the whole program, which would otherwise collide on its own label.
- **`pipeline.rs` — an empty portfolio names the bound that truncated the analysis.** Order: a disproved target predicate is reported first and keeps `NoFeasiblePlan`, because it is concrete evidence about a region that was actually proposed and a truncated search does not make it less true. With no such rejection, `truncating_budget` consults region formation, cover enumeration, and plan selection in pipeline order and, if any of them stopped, raises `BudgetExhausted` naming the resource, limit, and demand under the reason code `portfolio-empty-after-budget-stop`. A `CoverBudgetResource::Refusals` stop is deliberately not a truncation — it bounds the explanation, not the search. Where nothing was truncated the search was exhaustive and `NoFeasiblePlan` stands.
- **`pipeline.rs` — that refusal is target-local.** `compile_candidate_target` retained only `NoFeasiblePlan` as a per-target outcome. A budget stop raised inside `compile_target_with_explain` truncated *this* target's analysis, and a sibling profile that implements a region this one does not still reaches a plan under the same bound, so it joins it. Program-size budgets are unaffected: `check_program_budgets` refuses before any target compiles and still ends the batch.
- **Comments corrected in place**, per the second reading: `DeterministicBudgets::governed`'s sentence now names which three bounds can refuse a program and says the widening question they raise is not decided there; `region_members` carries the shared note for the shape bounds; `region_candidates_per_seed` and `region_expansions` carry the coverage-extremes guarantee; `session.rs`'s class-reachability ledger records `BudgetExhausted`'s second source; `docs/compiler/optimizer.md` and `docs/compiler/fusion-and-scheduling.md` both stated the unqualified claim as contract and now state the split.

### Before and after at twelve operations

| | before (`d050f10a`) | after |
| --- | --- | --- |
| 12 operations | `NoFeasiblePlan`, 146 explain records, `budget-stop:region-expansions:10000:10001`, terminal `portfolio-empty-without-target-rejection` | compiles; one fused alternative; 43,027-byte kernel-program identity; 12 coverage records |
| 13..=32 | `NoFeasiblePlan` | compile |
| 33..=62 | `NoFeasiblePlan` on `region_expansions` | `BudgetExhausted` on `region_members` (32), raised after planning with the trace sealed |
| 63 | `BudgetExhausted` | unchanged |

### Spike re-derivation

`spikes/program-planning/identity-growth` fired both of its arms on the base tree — twelve compiled where a refusal was required, and sixty-two changed class — which is the outcome its wall table exists to produce. Re-derived in this change per its own discipline:

- `OPERATIONS` widened from 2..=11 to **2..=32**, thirty-one consecutive points.
- `WALLS` re-derived to 33 / 62 / 63, all three `BudgetExhausted`. Because the class no longer separates them, each wall now also declares `reaches_planning` and the probe compares it: 33 and 62 seal a per-target trace, 63 refuses before one exists. `unplannable_program` selects on that flag rather than on the class, which is what its own documentation always meant.
- `--perturb=wall` now names `NoFeasiblePlan` as the wrong class; all four perturbations were run and each exits non-zero for its own arm.
- **Measurement, 2026-08-07**, `results/2026-08-07-post-coverage-extremes-apple-m4-max-macos27.0-26A5388g/growth.tsv`, Apple M4 Max, macOS 27.0 `26A5388g`, `nightly-2026-07-19`, load 4.09/6.06/8.32. `program_bytes(n) = 3525n + 727`, `graph_bytes(n) = 134n + 149`, **residual 0 at all thirty-one points**; no coefficient moved and every downstream figure is confirmed unchanged. Rows 2..=11 reproduce the previous run's structural columns byte for byte, which is the measurement that this fix moved no identity.
- The twenty-one new points were each a prediction the fit made about a program the compiler then refused, and all twenty-one landed to the byte. They are also now inside the domain, so the extrapolation to 19,038 again has no out-of-domain confirmation; the README says so.
- `compile_ms` peaks at 43 ms at eleven operations, **falls to 6 ms at twelve**, and creeps to 10 ms at thirty-two. The doubling below twelve is the connected-set enumeration, which is exponential in this family's fan shape; `region_expansions` is what stops it and first binds at twelve. Retaining the whole-program region unconditionally costs one `form_candidate` call.

### Identity

**Nothing moved.** The request subject encodes budget *values* only (`canonical_explain_subject_bytes` writes the fourteen numbers and nothing about the search), and no value changed. The spike's rows 2..=11 reproducing byte for byte is the direct evidence: a search that reaches further does not change the identity of the plan it selects. No encoding version moved and no pinned identity was recomputed.

### Parked for Tom

[`size-the-region-shape-budgets-to-the-programs-the-profile-admits`](size-the-region-shape-budgets-to-the-programs-the-profile-admits.md), `awaiting-decision`. `region_members` is 32 while `semantic_operations` is 62, so the profile's stated admission envelope and its actual planning envelope disagree for any family whose only implementable cover is its whole program — now measured rather than unknown. Moving it is a request-subject change and therefore an identity move, which is Tom's; the ticket enumerates what moves, states the three answers with their counterpoints, and recommends deriving all three shape bounds from the decoder layer's widest recognized region once that program is plannable, leaving 32 standing until then.

### Carried, not swept

[`carry-the-thirty-two-operation-ladder-into-the-five-records`](carry-the-thirty-two-operation-ladder-into-the-five-records.md), `todo`. Five records state the old ladder and the old wall class and sit in scopes this ticket does not hold — `research/artifacts`, `contracts/decisions`, `contracts/artifacts`, `contracts/foundation`, and one under `research/program-planning` held by a concurrent live claim. Each stale site is named there with what moves.

### Scopes

Added `contracts/optimizer` (the two optimizer contracts state the corrected claim as contract) and `research/program-planning` (the spike re-derivation, required by this ticket). **The `research/program-planning` addition overlaps `refresh-the-l1-operation-family-standing`'s exclusive claim and `tkt why` reports the conflict.** The overlap is declared-scope only: that ticket is documentation-only on `docs/research/program-planning/`, this branch touches only `spikes/program-planning/identity-growth/`, and the two share no file. No branch existed for that claim at the time of writing, so no diff comparison was possible; the coordinator should confirm disjointness at merge.

### Commands

```sh
cargo fmt --all --check
cargo check -p tiler-compiler --all-targets
cargo clippy -p tiler-compiler --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler
cargo nextest run --workspace && cargo test --workspace --doc
cd spikes/program-planning/identity-growth && cargo run --release
cd spikes/program-planning/identity-growth && for m in program coverage fit wall; do cargo run --release -- --perturb=$m; done   # each exits 1
make full
```

### Watched failing

`crates/tiler-compiler/tests/region_search_budget_coverage.rs` was run against `d050f10a` with the source changes stashed: all three tests fail — twelve refuses `NoFeasiblePlan` with 146 records, thirteen the same, and thirty-three reports `NoFeasiblePlan` where `BudgetExhausted` is required. All three pass on the fix. The three in-crate tests that had to be re-derived are recorded in the diff: two used a zero per-seed budget to reach a no-plan state and now reach a plan, which is the fix.
