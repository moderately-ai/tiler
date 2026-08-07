# Which tree each retained result measured

Each `growth.tsv` here is evidence about the tree it was produced on and **none of them is regenerated when a later one lands**. That is deliberate — a rerun on today's compiler would measure today's encoding, so overwriting an older file would destroy the only record of the encoding it read — but it means a reader comparing two files is comparing two trees, and the columns move for reasons that are not the curve. This file says which regime each belongs to so the comparison is possible at all. The spike's own [`README.md`](../README.md) carries the current reading; the newest file below is the one it reports.

Only the newest result describes the compilation path as it stands. **The three oldest state a reachable domain of 2..=32 that no longer holds, and byte columns that no longer reproduce.** Read them as history, not as current figures.

| Result | Ladder | What stopped it | Program-identity fit | What moved into it |
| --- | --- | --- | --- | --- |
| [`2026-08-05-…`](2026-08-05-apple-m4-max-macos27.0-26A5388g/growth.tsv) | 2..=8, 7 points | `semantic_operations = 8` | `134n² + 3650n + 710` — **quadratic** | the first sweep; the pre-[ADR 0104](../../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) encoding, one whole graph identity per coverage record |
| [`2026-08-06-…`](2026-08-06-apple-m4-max-macos27.0-26A5388g/growth.tsv) | 2..=10, 9 points | the explain authority's detail ceiling, `InvalidCompilerOutput` at 11 | `3525n + 727` — linear | `semantic_operations` moved 8 → 62; ADR 0104's fold made the per-record graph identity a fixed-width digest |
| [`2026-08-06-post-explain-ceiling-…`](2026-08-06-post-explain-ceiling-apple-m4-max-macos27.0-26A5388g/growth.tsv) | 2..=11, 10 points | `region_expansions`, `NoFeasiblePlan` at 12 | `3525n + 727` — linear | `refuse-nothing-legal-on-the-explain-detail-ceiling`; the eleventh point stopped refusing |
| [`2026-08-07-post-coverage-extremes-…`](2026-08-07-post-coverage-extremes-apple-m4-max-macos27.0-26A5388g/growth.tsv) | 2..=32, 31 points | `region_members = 32`, `BudgetExhausted` at 33 | `3525n + 727` — linear | `region-expansion-exhaustion-loses-the-only-feasible-plan`; 12..=32 stopped refusing |
| [`2026-08-07-post-derived-region-budgets-…`](2026-08-07-post-derived-region-budgets-apple-m4-max-macos27.0-26A5388g/growth.tsv) | 2..=62, 61 points | `semantic_operations = 62`, `BudgetExhausted` at 63 | `3530n + 723` — linear | `derive-the-region-shape-budgets-from-the-declaration`, which dissolved the 33..=62 wall; and the index-refinement encoding step described below |
| [`2026-08-07-post-restored-planning-wall-…`](2026-08-07-post-restored-planning-wall-apple-m4-max-macos27.0-26A5388g/growth.tsv) | **2..=62, 61 points** | **`semantic_operations = 62`**, `BudgetExhausted` at 63 | **`3530n + 723`** — linear, **every byte column identical to its predecessor** | nothing in the encoding; `restore-a-planning-phase-refusal-to-the-identity-growth-harness` added a second wall entry and a control, and this file is the first whose wall section carries a refusal raised *after* planning |

## The two regimes, and what separates them

**Fact — the domain.** Every file above the last was produced while `DeterministicBudgets::governed`'s `region_members` was the bare constant `32`. In the last of them that constant was what stopped the ladder: a shared-constant `f32` multiply chain's recognized partition is its whole program and nothing smaller is implementable, so 33..=62 occurrences refused `BudgetExhausted` on region size although every bound on the program's own *size* admitted them. `derive-the-region-shape-budgets-from-the-declaration` replaced the constant with `semantic_operations` on 2026-08-07 and the wall dissolved. **The ladder that stops at thirty-two stops on that constant, not on anything the identity encoding does there** — the three ladders above it stop on three earlier bounds, each named in the table and each since removed.

**Measurement — the byte columns moved too, and the shift is exact.** Comparing the last two files over the thirty-one operation counts they share:

- `graph_bytes` is **identical at every shared point**.
- `program_bytes` and `coverage_bytes` are each larger by exactly **`5n − 4`** bytes at every shared point, and the two deltas are equal — so the whole program-identity move lives in the coverage section.
- The fits differ by the same amount: `(3530n + 723) − (3525n + 727) = 5n − 4`. **The older ladder is recovered from the newer one by that subtraction**, which is what makes the two files comparable rather than merely different.

**Inference — where those bytes came from, and it is not the budget change.** Budgets are written into the canonical *request subject* and not into the kernel-program identity, so the derivation cannot have moved these columns; `DeterministicBudgets::governed`'s own note records the one pinned identity that did move with it, a request qualifier. The candidate that fits is `d39cee59`, "Admit an environment read in a proof and name it in the evidence", which stepped the canonical region domain to `tiler.index-region.v11` and appends one fixed-width `IndexDomainFactSource` tag per **discharged** index-domain assessment. `5n − 4` over a chain of one hoisted constant and `n − 1` multiplies reads as five discharged assessments per multiply and one for the constant. This is an inference from the encoder plus the arithmetic above, not a bisection: several commits touching identity-bearing code landed between the two trees and no per-commit measurement was taken.

**So the older files are stale in two independent ways** — a domain that was truncated by a bound since dissolved, and byte columns from an earlier index-region encoding. Neither is a defect in them.

## The last two files are one regime, and that is itself a measurement

**Measurement, 2026-08-07.** The newest file was produced on base `25e76d5d`, six commits past the `cee4fe1a` its predecessor names — commits touching `tiler-compiler`, `tiler-ir` and `tiler-build`, including a widened elementary recognizer, BF16 fusion legality, and a staged family admitted over a materialized intermediate. **All nine structural columns are identical at all sixty-one points**, checked column by column rather than by fit: `requested`, `operations`, `coverage_records`, `stages`, `alternatives`, `graph_bytes`, `program_bytes`, `widest_alternative_bytes`, `coverage_bytes`. `compile_ms` is not compared and is not evidence.

So `3530n + 723` is a statement about two trees rather than one, and the spike README's whole verdict — the refusal point, the margins, the embedding-ceiling crossing — carries to this base unchanged. A separate file is retained anyway because its *wall section* differs: it is the first that probes a refusal raised after planning, and the first that compiles a control.

## Why none of them is regenerated

Regenerating one means checking out the tree it names and rebuilding the compiler there. That would not restore the file: it would produce a *fifth* regime's numbers under an older path name, and the record of what the encoding did on 2026-08-05 and 2026-08-06 would be gone. The retained files are the only surviving statement of those encodings, and the reconciliations above — `graph_bytes` identical, `program_bytes` differing by exactly `5n − 4` — are checks that they were read correctly, which a regeneration would remove rather than strengthen.

## Host

Every file above was produced on the same host: macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max. The byte columns are deterministic by construction and the harness refuses if two compilations of one program disagree, so they are immune to host load. The `compile_ms` column is not, and it is reachability information rather than a benchmark — see the spike README's boundary section. Load averages at each run are recorded in the spike README's `Result` section.
