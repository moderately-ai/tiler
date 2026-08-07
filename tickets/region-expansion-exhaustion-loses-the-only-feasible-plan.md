---
id: region-expansion-exhaustion-loses-the-only-feasible-plan
title: Region expansion exhaustion loses the only feasible plan
status: in-progress
priority: p2
dependencies: []
related: [widen-the-identity-growth-ladder-to-the-governed-operation-budget, refuse-nothing-legal-on-the-explain-detail-ceiling]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [defect, region-search, budgets, compiler]
claimed_from: todo
assignee: agent-region-expansion
lease_expires_at: 1786077467
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
