---
id: correct-the-records-the-derived-region-shape-budgets-falsify
title: Correct the records the derived region-shape budgets falsify
status: in-progress
priority: p2
dependencies: [rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets]
related: [derive-the-region-shape-budgets-from-the-declaration]
scopes: [contracts/foundation, contracts/artifacts, contracts/decisions, research/artifacts, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, budgets, identity]
claimed_from: todo
assignee: w-correct-
lease_expires_at: 1786140924
---
## User-visible outcome

No governing contract, accepted decision, or research record still states that the compilation path refuses this family above thirty-two operations, or that `region_members` is 32, `region_boundary_outputs` 8, or `region_live_values` 64.

## Why this exists

[`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) made the three region-shape budgets derivations over the declaration on Tom's 2026-08-07 decision: `region_members` is `semantic_operations` (62), `region_live_values` is `semantic_values` (80), and `region_boundary_outputs` is the declared output count (3). The whole 33..=62 range now compiles as one whole-program region, and the first refusal for the chain family is `semantic_operations` at sixty-three.

That ticket held `implementation/compiler` and `contracts/optimizer`, and it corrected [`docs/compiler/optimizer.md`](../docs/compiler/optimizer.md)'s budget list and derivation in the same commit. Every site below is in a scope it did not hold.

## The drifted sites, each with its scope and the claim that is now false

Located by `grep -rn "region_members\|region_live_values\|region_boundary_outputs" docs/ spikes/` on the branch that moved them; each was read at its line before being listed.

**`contracts/foundation` — [`docs/ir.md`](../docs/ir.md) line 1130.** "The path refuses this family above thirty-two operations with `BudgetExhausted` on `region_members`, a bound on one region's admissible shape rather than on program size." False: the path compiles the family to sixty-two. The same sentence's "measured on the ordinary compilation path over the widened 2..=32 ladder with residual zero at all thirty-one points" is a measurement claim whose domain moves with the re-run.

**`contracts/artifacts` — [`docs/artifact-abi.md`](../docs/artifact-abi.md) line 247.** The same two claims in the same words, plus "At the governed `semantic_operations` budget of 62 that is 219,277 bytes" — which stops being an extrapolation once 62 is inside the ladder's domain, and should be restated as a measurement or corrected.

**`contracts/decisions` — [ADR 0104](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) line 20.** "The ladder now runs 2..=32; 33..=62 refuse `BudgetExhausted` on `region_members` — a declared bound on one region's admissible shape, whose sizing against `semantic_operations` is a parked decision — so the governed maximum of 62 remains unreachable for this family". Every clause is now false: the sizing was decided, the range compiles, and 62 is reachable. The record's own header measurement paragraph names the domain and the walls.

**`research/artifacts` — [`docs/research/artifacts/manifest-fixed-content-growth.md`](../docs/research/artifacts/manifest-fixed-content-growth.md) lines 165, 169, 211, 241.** Four separate restatements: "62 is not compilable at all", "the compilation path refuses everything above thirty-two operations", "The remaining wall is `BudgetExhausted` on `region_members` at thirty-three, a declared region-shape bound whose sizing is a parked decision", and "**No program above 32 operations compiles at all.** The 62-operation figure is arithmetic over a fit, not an observation." The last is the one that inverts most sharply: the 62-operation figure becomes an observation.

**`research/program-planning` — [`docs/research/program-planning/complete-model-ingestion-and-execution.md`](../docs/research/program-planning/complete-model-ingestion-and-execution.md) line 154.** "The ladder now runs 2..=32, and 33..=62 refuse `BudgetExhausted` on `region_members` (32) … so this record's decoder-layer program at ≥ 51 occurrences is **still not compilable**, now for a declared-bound reason rather than a truncated search." The size reason is gone; the recognizer's refusal — `select_supported_strategy` — is what still blocks the layer, and that is a different statement with a different remedy.

**`research/program-planning` — [`docs/research/program-planning/first-attention-program-vertical.md`](../docs/research/program-planning/first-attention-program-vertical.md) line 186.** "`region_members` bounds a region at 32 semantic occurrences … gives at least forty-four occurrences, and … `EnumerateRegionCandidates` therefore abandons that growth path with a typed budget stop and never decides the block's legality." The arithmetic that made forty-four exceed the bound no longer holds against 62, so this bullet's *conclusion about which wall stops the attention block* has to be re-derived rather than reworded — the block may now reach a different refusal, and naming the wrong one is worse than naming none.

**`research/program-planning` — the spike itself.** Owned separately by [`rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets`](rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets.md), which this ticket depends on: several corrections above quote its ladder extent and its fit, and stating a new extent before the re-run measures one would replace a stale number with an invented one.

## What this ticket owes

Each site above corrected against source rather than against this ticket's summary, with the measurement-bearing sentences carrying the re-run's actual domain, fit, and walls. Where a conclusion inverts rather than shifts — the attention-block wall, the "not compilable at all" paragraph — the record states what the new wall is or that it is unmeasured, and does not paper over the difference.

## Explicit non-goals

Not moving any budget. Not editing `docs/compiler/**`, corrected with the change. Not re-running the identity-growth ladder, which is its own ticket.

## Three of this ticket's own Facts repaired, 2026-08-07, by the worker on a full read of `crates/tiler-compiler/src/request.rs`

Recorded here rather than silently worked around, because two of the corrections would have repeated the error if written from this ticket's summary.

**1. "The three region-shape budgets are derivations over the declaration" is false as stated, and the sentence above stands only as the shorthand it is.** `DeterministicBudgets::governed` is a **nullary `const fn` returning fourteen integer literals** (`request.rs:1046-1063`). `region_members` is the literal `62`, not an expression over `semantic_operations`; `region_boundary_outputs` is the literal `3`, not the declared output count of the program being compiled. The derivation is **authoring-side** — performed once against the C1 decode row of the decoder layer and recorded in that function's prose — and nothing is computed from a request's declaration at run time. The phrasing used in all seven corrections is that the literals are *sized against* the governed profile's declaration. The same false premise was struck from [`decide-whether-a-derived-budget-belongs-in-the-request-subject`](decide-whether-a-derived-budget-belongs-in-the-request-subject.md) on the same day, on the same read.

**2. The `docs/artifact-abi.md` item's proposed remedy does not hold.** It says 219,277 bytes at 62 operations "stops being an extrapolation once 62 is inside the ladder's domain, and should be restated as a measurement". The re-run **falsifies** the figure rather than confirming it: the measured value at 62 is **219,583**, because the fit moved from `3525n + 727` to `3530n + 723` under a `5n − 4` index-refinement encoding step. Restating 219,277 as a measurement would have written down a number nothing measured. The item's alternative — "or corrected" — is what the correction takes.

**3. The site list is under-inclusive, and the grep it was built from is why.** `grep -rn "region_members\|region_live_values\|region_boundary_outputs" docs/ spikes/` cannot see a record that states a superseded value without naming its field. Three such sites exist. One is inside this ticket's scopes and is corrected with the six: [`docs/research/program-planning/first-attention-program-vertical.md`](../docs/research/program-planning/first-attention-program-vertical.md) line 175, whose Q/K/V table row calls three retained outputs "within the 8-output budget" — the budget is now `3`, so the candidate is admitted exactly *at* the bound with no margin and a fourth output would be refused. The other two are outside every scope this ticket holds and are filed as [`correct-the-region-shape-budget-sites-outside-the-corrections-scopes`](correct-the-region-shape-budget-sites-outside-the-corrections-scopes.md): `docs/research/region-search/exhaustive-region-oracle.md` lines 143–144 (`research/region-search`), which spells all three superseded values as prose, and `docs/status.md` line 30 (`contracts/navigation`), which quotes the superseded fit. **The user-visible outcome above is therefore not fully reachable from this ticket's declaration**; it is discharged for the five scopes this ticket holds, and the follow-up carries the remainder.

## Outcome — 2026-08-07

Seven sites across six records corrected, each against source rather than against this ticket's summary, each by a dated correction that preserves the retired text and follows its own file's convention.

**The measurement authority is [`spikes/program-planning/identity-growth`](../spikes/program-planning/identity-growth/README.md)'s re-run**, delivered by [`rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets`](rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets.md) and retained at `results/2026-08-07-post-restored-planning-wall-apple-m4-max-macos27.0-26A5388g/growth.tsv`. The figures every correction carries: domain **2..=62, sixty-one points**; `program_bytes(n) = 3530n + 723`, second difference zero, residual zero at every point; `graph_bytes(n) = 134n + 149` unmoved; `3525n + 727` reproduces **no** point, every value being larger by exactly `5n − 4`, and `(3530n + 723) − (5n − 4) = 3525n + 727` recovers the older ladder by subtraction. Re-solved: the 64 MiB program bound **19,038 → 19,011**; the 1 MiB embedding crossing **unmoved at 148/149** (`1,046,326` and `1,053,386` at ×2); 62 operations **219,583 measured** rather than 219,277 fitted, **41.9%** of the ceiling at ×2 rather than 41.8%; 51 operations **180,753 measured**, **361,506** at ×2, **34.5%** rather than 34.4%, margin **×371**; ≥ 1,068 occurrences **3,770,763 bytes — 3.60 MiB, 5.6%** of the 64 MiB bound, **7,541,526** at ×2 and still 7.2× over the embedding ceiling.

**The out-of-domain claim retires in the harder direction, and every correction says so rather than letting the widened ladder read as reassurance.** The eleventh point and each of 12..=32 confirmed `3525n + 727`; the line moved under them by `5n − 4`, so those twenty-two confirmations expired with the encoding they were about. The thirty new points were not predictions of the fitted line either. The ladder now covers every program size the path admits, so **no out-of-domain check is obtainable along this axis at all** without moving `semantic_operations`. What the doubled domain did check is the *form* — thirty new consecutive points on one line at second difference exactly zero — and that is what ADR 0104's conclusion rests on.

**The one conclusion the new coefficients could plausibly have flipped was checked rather than assumed:** the 148/149 embedding crossing sits on a line whose slope moved by 0.14%, and it did not move.

### The two inversions, each stated as an inversion

- **`docs/research/artifacts/manifest-fixed-content-growth.md` Section 8.** "No program above 32 operations compiles at all" becomes its reverse: the 62-operation figure **stops being arithmetic over a fit and becomes an observation**. The same correction records that the *neighbouring* item moves the opposite way — its "one out-of-domain confirmation" is now **none** — so the section cannot be read as uniformly strengthened.
- **`docs/research/program-planning/complete-model-ingestion-and-execution.md` line 154.** "Still not compilable, now for a declared-bound reason" is false: no bound on program size and no region-shape bound stands between this record's decoder layer and compilation. What still refuses it is `select_supported_strategy`'s named-rule refusal, which the record's **own L6 item 1 already states** ("item 2's recognizer refusal is untouched and is what still refuses P2") — so the correction cites the record against itself rather than importing a claim.

### Where a conclusion was *not* re-derived, and why that is the deliverable

**`docs/research/program-planning/first-attention-program-vertical.md` line 186.** Forty-four occurrences no longer exceeds sixty-two, so the budget stop that bullet attributes to `EnumerateRegionCandidates` does not fire for the reason it gives. The correction **names no replacement wall**: no program of this shape has been compiled through the ordinary path, this record measures none, and the identity-growth ladder sweeps a unary `f32` multiply chain rather than an attention block, so which bound the enumerator reaches for this block is **unmeasured**. Naming a wall on arithmetic alone is what produced the retired bullet. What the correction does state is that `region_live_values = 80` is a candidate the occurrence count does not settle either way, that the second bullet's cross-threadgroup legality ground is untouched and is now the record's only stated ground, and that an unmeasured wall licenses even less than a measured stop does.

### Records changed, with each site's classification

| Record | Scope | Sites | Classification |
| --- | --- | --- | --- |
| [`docs/ir.md`](../docs/ir.md) | `contracts/foundation` | 1 | live false claim, inside a 2026-08-06 dated correction |
| [`docs/artifact-abi.md`](../docs/artifact-abi.md) | `contracts/artifacts` | 1 | live false claim |
| [ADR 0104](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) | `contracts/decisions` | 2 | live false claim ×2 — the header measurement **and** *Bounds on the evidence*, the second not listed by this ticket |
| [`docs/research/artifacts/manifest-fixed-content-growth.md`](../docs/research/artifacts/manifest-fixed-content-growth.md) | `research/artifacts` | 4 paragraphs across Sections 5, 6, 8 | live false claim, incl. one already-dated correction (Section 6's *Landed 2026-08-06*) whose own numbers moved |
| [`docs/research/program-planning/complete-model-ingestion-and-execution.md`](../docs/research/program-planning/complete-model-ingestion-and-execution.md) | `research/program-planning` | 1 | live false claim, inside an already-dated *Superseded twice* correction |
| [`docs/research/program-planning/first-attention-program-vertical.md`](../docs/research/program-planning/first-attention-program-vertical.md) | `research/program-planning` | 2 (lines 175, 186) | live false claim ×2, line 175 not listed by this ticket |

**One ADR touched, and only one**, although `contracts/decisions` globs every numbered record: ADR 0104 is the only accepted decision whose text this change falsifies. Its second site — *Bounds on the evidence*, whose "the widened ladder's whole reachable domain" and "none moved" are both false — was found by reading the record in full rather than from the ticket's list.

**No open decision was resolved or presumed.** Whether `region_members` and `region_live_values` keep their slots in `tiler.compiler.request-subject.v5` is still Tom's under [`decide-whether-a-derived-budget-belongs-in-the-request-subject`](decide-whether-a-derived-budget-belongs-in-the-request-subject.md) (`awaiting-decision`, verified on this branch). Three records previously called the budget *sizing* a parked decision; that one is decided, so each correction separates the settled sizing from the still-open request-subject question rather than retiring both or neither.

**No `crates/` file is touched.** The delta is `docs/` and `tickets/` only.
