---
id: widen-the-identity-growth-ladder-to-the-governed-operation-budget
title: Widen the identity-growth ladder to the governed operation budget
status: done
priority: p2
dependencies: []
related: [measure-executable-coverage-identity-growth-against-the-program-identity-bound, decide-whether-executable-coverage-evidence-folds-as-a-digest, attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget, add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, program-planning, identity, measurement]
---
## User-visible outcome

[`spikes/program-planning/identity-growth`](../spikes/program-planning/identity-growth/README.md) runs again and its ladder covers the domain the governed budget actually admits, so the record's headline claim — "the bound is unreachable for the program sizes this roadmap contemplates, with a margin of about 125×" — rests on measurement over the current domain rather than on an extrapolation from a domain that has since grown roughly eightfold.

## Why this exists — the harness says so itself, and it exits non-zero

**Measurement, 2026-08-06, at `f38813da`.** `cd spikes/program-planning/identity-growth && cargo run --release` **fails**, exit 1, on its own wall probe:

```
THE WALL MOVED: 9 operations compiled to a 44423-byte identity, so the governed
semantic-operations budget is no longer 8 and this ladder is no longer the whole
reachable domain. Widen OPERATIONS and rerun; the recorded result and its verdict
are stale.
```

That is the harness's designed refusal working exactly as its README says it should: "If the probe ever *succeeds*, the run fails and says the recorded result is stale — a moved budget widens the domain and invalidates the ladder, which is a finding rather than a pass."

**Fact — what moved it.** `36d05128` (*Integrate the budgets widening D-18 decided*, 2026-08-05) raised `DeterministicBudgets::governed`'s `semantic_operations` from **8 to 62**, sizing the five program-scoped bounds "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit". The spike's `OPERATIONS` ladder and its `BEYOND_THE_WALL` probe still name the old wall.

**Measurement — two things the re-run establishes before the refusal, and both are worth keeping.** The whole ladder is **+9 bytes at every point** against the retained result — 8,546 → 8,555, 12,866 → 12,875, 38,486 → 38,495 — and the fitted constant term is **719 rather than 710**. Those nine bytes are exactly the publishing-copy step `f8dfa8f6` landed, attributed independently at the envelope layer by [Where the artifact envelope's fixed content came from](../docs/research/artifacts/manifest-fixed-content-growth.md). And the wall probe's own number is a **free validation of the fit one step outside its domain**: `134·9² + 3650·9 + 719 = 44,423`, which is the measured identity length to the byte.

## What this ticket owes

- `OPERATIONS` and `BEYOND_THE_WALL` moved to the current governed budget, with the ladder's own doc comment restating the derivation rather than a new constant appearing without one.
- A re-run and a newly retained result beside the existing one, which is evidence at its own commit and is not overwritten.
- The verdict re-derived: the fitted curve, the refusal point, and the ×125 margin all move, and the record's *Verdict* section says which of its numbers reproduced and which moved.
- **A stated feasibility boundary, because the ladder may not reach 62.** The retained `compile_ms` column runs 1, 1, 2, 2, 3, 7, 14 over 2..=8 — roughly doubling per operation at the top of the range — so a contiguous 2..=62 ladder may not be affordable. If it is not, the honest form is a stated sub-range with the wall probe at the first point beyond it and the reason recorded, not a silently truncated sweep. How far the ladder actually reaches is itself a measurement this ticket owes.

## Why it matters beyond tidiness

The refusal point, the margin, and the deferral triggers on [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md) were all derived against a domain of 2..=8 operations and a budget of 8. The budget is now 62 — that is, the governed profile already admits, by size, the decoder-layer program the deferral's third trigger treats as a future contingency. Nothing here says the 64 MiB bound is in danger; what it says is that the numbers guarding it were computed against a wall that has moved.

## Explicit non-goals

Not moving `semantic_operations`. Not deciding the digest question, which is [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md)'s. Not editing that deferral's triggers, which is [`add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral`](add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md)'s.

## Closes when

The harness runs to completion at the current budget, a new result is retained beside the old one, and the research record states which figures reproduced, which moved, and how far the ladder reached with the reason it stopped there.

## Outcome — 2026-08-06

**The harness runs to completion, exit 0.** `cd spikes/program-planning/identity-growth && cargo run --release`. Retained at [`results/2026-08-06-apple-m4-max-macos27.0-26A5388g/growth.tsv`](../spikes/program-planning/identity-growth/results/2026-08-06-apple-m4-max-macos27.0-26A5388g/growth.tsv), beside the 2026-08-05 result, which is untouched. Host: Apple M4 Max, macOS 27.0 `26A5388g`, `arm64`, toolchain `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`), load averages 2.95 / 6.72 / 9.06 — a host running other agents' builds, which the byte columns are immune to by construction and the `compile_ms` column is not.

### The ladder is nine points, not sixty-one, and that is the headline

This ticket asked for a ladder spanning 2..=62. **The governed budget of 62 is not reachable**, and the run now proves that by compiling at each wall and requiring its class rather than asserting one:

| Operations | Outcome | Class | Bound |
| --- | --- | --- | --- |
| 2..=10 | verifies | — | — |
| 11 | refuses | `InvalidCompilerOutput` | the explain authority's detail ceiling (`MAX_RECORDS` = 4,096), exhausted by 2,300 cover-enumeration rejection records; region formation succeeds here with 66 candidates and `budget-stops:count=0` |
| 12..=62 | refuses | `NoFeasiblePlan` | `budget-stop:region-expansions:10000:10001` stops growth before the whole-program candidate; candidates fall 66 → 20, every surviving cover names an unimplemented region |
| 63 | refuses | `BudgetExhausted` | `semantic_operations = 62` — the only wall about program size |

62 is probed explicitly, so the largest program the profile admits *by size* is measured to refuse for a reason that has nothing to do with size. The ticket's anticipated feasibility limit was compile time; the actual limit is two search bounds, and compile time (1, 2, 2, 2, 4, 7, 13, 28, 64 ms) was never the binding constraint.

**Two of those three walls are defects and neither is this ticket's.** Filed: [`refuse-nothing-legal-on-the-explain-detail-ceiling`](refuse-nothing-legal-on-the-explain-detail-ceiling.md) — `InvalidCompilerOutput` is documented as "always a defect in Tiler rather than in the caller's program", and it refuses an eleven-operation program inside every governed budget. [`region-expansion-exhaustion-loses-the-only-feasible-plan`](region-expansion-exhaustion-loses-the-only-feasible-plan.md) — `DeterministicBudgets::governed`'s doc comment states that every `region_*` bound "bounds a *search*, and exhausting one costs an alternative while the verified input and complete coverage survive", which the twelve-operation row contradicts in terms, and that sentence is the stated ground on which the 2026-08-05 widening moved `semantic_operations` without moving the search bounds.

### The fit, with residuals

```
program_bytes(n) = 3525n + 727      residual 0 at all nine points
graph_bytes(n)   =  134n + 149      residual 0 at all nine points
```

| n | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| measured | 7,777 | 11,302 | 14,827 | 18,352 | 21,877 | 25,402 | 28,927 | 32,452 | 35,977 |
| fitted | 7,777 | 11,302 | 14,827 | 18,352 | 21,877 | 25,402 | 28,927 | 32,452 | 35,977 |
| residual | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

**The linear prediction holds across the widened domain**, including the two points ADR 0104 never had: 9 and 10 reproduce to the byte, and the second difference is 0 at every step. Coverage bytes add a flat **3,296 per operation** where under the pre-fold restatement that step climbed with `n` — the mechanism read off the curve rather than off the encoder. The log-log exponent is 0.9538 against 1.0863 for the quadratic encoding, which is why the harness reports the exact fit and prints the exponent only beside it.

### Crossings: every quoted number reproduced; none moved

- **64 MiB program bound at 19,038 operations** (67,109,677 B at 19,038; 67,106,152 B at 19,037) — ADR 0104's figure, **confirmed**.
- **219,277 bytes at the governed budget of 62**, whose doubling is **41.8%** of the 1 MiB per-invocation ceiling — ADR 0104's figures, **confirmed** (extrapolated; 62 is now known to be uncompilable).
- **Embedding-ceiling crossing between 148 and 149 operations** — `2 × (3525·148 + 727) = 1,044,854`, `2 × (3525·149 + 727) = 1,051,904` — ADR 0104's figure, **confirmed**.
- **Linear coefficient 3,525 to the unit and constant 727** — ADR 0104's figures, **confirmed** over a domain two points wider.

The pre-fold ×125 margin and 695-operation refusal point are superseded by the fold, not by this run; the retained 2026-08-05 result remains evidence about the encoding it measured.

### Records updated (in scope)

- [`spikes/program-planning/identity-growth/README.md`](../spikes/program-planning/identity-growth/README.md) — result, fit, mechanism, refusal point, verdict, boundary, and the wall table all rewritten; `last_verified` to 2026-08-06. The stale "Why neither catalog lists this record yet" section is deleted: both catalogs list it and [`add-the-identity-growth-experiment-rows-to-the-two-catalogs`](add-the-identity-growth-experiment-rows-to-the-two-catalogs.md) is `done`.
- [`docs/research/program-planning/complete-model-ingestion-and-execution.md`](../docs/research/program-planning/complete-model-ingestion-and-execution.md) — its two identity-growth paragraphs rewritten to the measured curve (P1 4,252 B, P3 7,777 B measured, P2 180,502 B at 0.27% of the bound, margin ×372 against the prior ×125), plus a third paragraph recording that 12..=62 does not compile at all.

**The verdict reversal that matters.** A whole-model program at ≥ 1,068 occurrences is now **3,765,427 B — 3.59 MiB, 5.6% of the 64 MiB bound**, where the pre-fold curve put it at ≈ 149 MiB and a typed refusal. **Program identity no longer forbids whole-model fusion.** The per-layer partition is still load-bearing on size, but the ceiling that says so is now the per-invocation embedding one: the same program's fixed content at the post-ADR-0103 multiplicity of two is 7,530,854 B against 1,048,576 — 7.2× over, and with no typed refusal at the artifact layer. P2 sits at 34.4% of that ceiling against the 102% the pre-fold curve gave it.

### Owed corrections — records this ticket does not hold

**[`docs/research/artifacts/manifest-fixed-content-growth.md`](../docs/research/artifacts/manifest-fixed-content-growth.md) Section 5** (`research/artifacts`). Every figure in its last four paragraphs is pre-fold and pre-ADR-0103 and is now wrong rather than merely stale. Verbatim replacements, at the post-ADR-0103 multiplicity of two:

- "kernel-program identity is exactly `134n² + 3650n + 710` bytes" → **`3525n + 727` bytes, linear, measured 2026-08-06 over 2..=10 operations with a residual of zero at all nine points**.
- "passes the **1,048,576-byte per-invocation ceiling between 32 and 33 semantic operations** — `4 × (134·32² + 3650·32 + 719) = 1,018,940` and `4 × (134·33² + 3650·33 + 719) = 1,068,380`" → **between 148 and 149 semantic operations — `2 × (3525·148 + 727) = 1,044,854` and `2 × (3525·149 + 727) = 1,051,904`**.
- "puts the 64 MiB `MAX_PROGRAM_IDENTITY_BYTES` refusal at **695 operations**. The embedding ceiling therefore binds **about 21× earlier in operation count**" → **at 19,038 operations. The embedding ceiling therefore binds about 128× earlier in operation count**.
- "At the governed maximum of 62 operations the fitted curve gives `134·62² + 3650·62 + 719 = 742,115` bytes … four times that is **2,968,460 bytes — 2.83× the … ceiling**. The roadmap's decoder layer at ≥ 51 operations is `4 × 535,403 = 2,141,612`, roughly 2.0×." → **`3525·62 + 727 = 219,277` bytes, and twice that is 438,554 bytes — 41.8% of the ceiling. The roadmap's decoder layer at ≥ 51 operations is `2 × 180,502 = 361,004`, 34.4%.**
- The whole "**Inference — so the ceiling is not a future risk, it is a present one**" paragraph inverts: at the governed maximum the envelope is now well inside the ceiling. Its "Why that conclusion survives the fit being wrong" paragraph is moot — there is no quadratic term left to delete.
- Its "The bound on that inference" paragraph says "the re-run's 9-operation wall probe reproduces the curve to the byte". That confirmation is **no longer available**: 9 is inside the widened domain, and the compilation path refuses every program above 10, so the extrapolation to 62 / 148 / 19,038 has no out-of-domain check at all.

**[`docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md`](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md)** (`contracts/decisions`). **No number moves — all four of its quoted figures reproduced.** Three statements are superseded:

- Header measurement paragraph: "2..=8 operations with a nine-operation probe outside the fitted domain" → **2..=10 operations, with four class-checked walls at 11, 12, 62, and 63**.
- Header measurement paragraph: "**The ladder harness that produced these numbers still refuses on its own wall probe** … its re-run must fit a linear curve rather than a quadratic one and carry the 727 constant, and its retained 2026-08-05 result is superseded in both respects." → the re-run is done, it fit `3525n + 727`, and it confirmed every crossing this record quotes.
- "Bounds on the evidence": "the fitted curve reproduces a 9-operation probe outside its domain to the byte" is no longer an out-of-domain confirmation, per the point above — this **weakens** the stated bound and should be recorded as such rather than dropped. The same section's closing sentence ("no figure here was re-measured on a widened ladder") is discharged.

**[`docs/artifact-abi.md`](../docs/artifact-abi.md) line 247** (`contracts/artifacts`) and **[`docs/ir.md`](../docs/ir.md) line 1138** (`contracts/foundation`) both carry "measured … over 2..=8 operations, with a nine-operation probe outside the fitted domain reproducing to the byte". Same correction: **2..=10 operations, no out-of-domain probe available.** Their numeric claims (`3525n + 727`, 19,038, 148/149, 219,277, 41.8%) are all confirmed.

**[`docs/status.md`](../docs/status.md) line 30** (`contracts/navigation`): its figures are confirmed; no correction owed.

**[`tickets/decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md)** is `done`, so no update is owed, but for the record its trigger 2 is now evaluable rather than unevaluable and **does not fire**: the measured `graph_bytes(n) = 134n + 149` slope is unchanged at 134. Not edited here, per this ticket's explicit non-goals.

### Harness changes, and one that was not asked for

`OPERATIONS` is 2..=10 with its derivation restated. `BEYOND_THE_WALL` is replaced by a `WALLS` table of four `(operations, class, why)` entries, each compiled and required to refuse **with its class** — so a wall that moves *in kind* fails as loudly as one that disappears. `compile_once` now returns a typed `Refusal` carrying `CompileFailureClass` read through the public accessor rather than scraped from the rendered trace, which `ExplainReport` documents as not a parse target. `--perturb=wall` watches the class comparison fail (the "it compiled" arm already fired for real on 2026-08-06). `first_refusing_operation_count` solves the linear case as a line instead of dividing zero by zero. The structural-decomposition table's third column is now the coverage-bytes step, which is what discriminates the two encodings. The `bytes_per_op_squared` column is `bytes_per_op`.

**`--perturb=program` had silently stopped perturbing.** Its program was a reverse-axis `tiler::reindex-f32@1`, justified by an access relation the region vocabulary could not spell. All six `ReindexFormKind` arms are recognized on this tree, so the perturbed program compiled, the sweep measured nine copies of a one-operation graph, and the run still exited 1 — from the fit check refusing a degenerate ladder rather than from the arm the mode exists to watch. It now takes its program from the `NoFeasiblePlan` wall, so the same run that uses it also requires its refusal and the mode cannot go stale without a wall failing first.

All four perturbations were watched failing: `program` (exit 1, `NoFeasiblePlan` at the first ladder point), `coverage` (exit 1, completeness assertion), `fit` (exit 1, "NO EXACT QUADRATIC FITS"), `wall` (exit 1, "THE WALL CHANGED KIND at 11 operations").

### Checks

`cargo clippy --release --all-targets -- -D warnings` clean in the spike workspace; `cargo build --release` clean; the documented run exits 0; all four perturbations exit 1; `tkt lint`, `git diff --check`, and `tkt guard` clean. Nothing outside `spikes/program-planning/`, `docs/research/program-planning/`, and `tickets/` moved. No crate under `crates/` was edited, so the workspace gate is untouched by this branch.
