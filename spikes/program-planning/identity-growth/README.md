---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.identity-growth"
kind: "experiment"
title: "How kernel-program identity grows against its 64 MiB bound"
topics: ["program-planning", "identity", "coverage", "index-refinement", "limits"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "executable-model"]
supports: ["tiler.research.program-planning.complete-model-ingestion-and-execution"]
entrypoints: ["spikes/program-planning/identity-growth/src/main.rs"]
last_verified: "2026-08-07"
ticket: "measure-executable-coverage-identity-growth-against-the-program-identity-bound"
---

# How kernel-program identity grows against its 64 MiB bound

[`measure-executable-coverage-identity-growth-against-the-program-identity-bound`](../../../tickets/measure-executable-coverage-identity-growth-against-the-program-identity-bound.md) inherited a structural inference with exactly one measured point behind it: because `CanonicalKernelProgramIdentity` embeds one whole reached-only executable-coverage identity per covered occurrence, one record per graph operation, and because each of those records embedded the complete `SemanticGraphIdentity` of the bound graph, program identity should be **Θ(operations × graph-encoding size)** — quadratic in graph size — against a hard `MAX_PROGRAM_IDENTITY_BYTES` of 64 MiB that fails closed with a typed refusal. The one measurement was a five-occurrence stage key at 21,366 bytes. What was unknown was how far from realistic program sizes the refusal sits.

This spike replaced that single point with a curve. On 2026-08-05 the curve was the quadratic the inference predicted. [ADR 0104](../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) then folded the per-record graph restatement to a fixed-width digest, and the run below measures what that did: **the curve is linear, and the quadratic coefficient is exactly zero rather than small.** Every result is retained at its own path, because each is evidence about the tree it measured, and [`results/README.md`](results/README.md) says which tree that was for each of the five.

**Five runs are retained and only the last describes the compilation path as it stands.** Four bounds have stopped this family in turn — `semantic_operations = 8`, the explain authority's detail ceiling, `region_expansions`, and `region_members = 32` — and each was removed by a ticket the previous run's wall table reported. The fifth and current run is the first whose ladder ends on `semantic_operations` itself, which is what the domain was supposed to be all along. The path qualifiers rather than new dates are because three of the five share a date with a predecessor and differ only in the compiler tree.

**Rows reproduced their predecessor's structural columns byte for byte through the first four runs, and the fifth is the first that does not.** Through 2026-08-07's `post-coverage-extremes` run the columns were identical over every shared domain — which was the measurement that neither the explain-ceiling fix nor the coverage-extremes fix moved any identity, since explain records are diagnostics and widening which regions a search reaches does not change the identity of the plan it selects. The current run's `graph_bytes` still reproduces exactly, but its `program_bytes` and `coverage_bytes` are each larger by exactly **`5n − 4`** at every shared point. That is not the budget derivation — budgets enter the canonical *request subject*, not the kernel-program identity — and [`results/README.md`](results/README.md) records the attribution and its limits.

## What it drives

For each operation count in the reachable domain it builds a semantic program, compiles it through the **ordinary** path — the public `tiler_compiler::session::compile` boundary, whose lowering mints real index-refinement receipts, derives `CoveredOccurrence` records from them, and drives `KernelProgramBuilder` — and reads the byte length of the canonical identity off the verified program the compilation produced. Nothing here constructs an identity, a receipt, or a coverage record; a synthetic one would measure the harness rather than the compiler.

The generator emits one input, one hoisted constant, and a chain of `F32Multiply` steps, so the operation count is exactly `1 + multiplies` and every integer in the domain is reachable. It is a pure multiply chain rather than a mixed multiply/add body because a region holding a multiply adjacent to an add is refused under the one contract that permits arithmetic contraction, and a generator whose admissibility depended on the contract would put a second variable into a one-variable sweep.

## Running it

```sh
cd spikes/program-planning/identity-growth
cargo run --release
```

Four perturbations exist so that the harness's refusals are watched rather than trusted, and each exits non-zero:

```sh
cargo run --release -- --perturb=program    # a program this build reaches no kernel program for
cargo run --release -- --perturb=coverage   # a corrupted coverage expectation
cargo run --release -- --perturb=fit        # one byte moved in one measured row
cargo run --release -- --perturb=wall       # the wrong class expected at one wall
```

No `make` target reaches here, per [`spikes/README.md`](../../README.md).

A run is retained by redirecting it to a new directory rather than over an existing one — `cargo run --release > results/<date>-<host>/growth.tsv` — and [`results/README.md`](results/README.md) is then extended with the tree it measured. Overwriting an existing file destroys the only record of the encoding it read.

## The domain is sixty-one points, and it is finally the governed budget itself

**Fact.** `DeterministicBudgets::governed` (`crates/tiler-compiler/src/request.rs`) caps `semantic_operations` at **62**, raised from 8 on 2026-08-05 and sized "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit". `DeterministicBudgets` is `pub(crate)` and `CompileRequest` binds `InstalledCapabilities::governed`, so no public caller can state a wider budget.

**Measurement, 2026-08-07 — that budget is now the wall, and it is the first time it has been.** The run compiles at the point below and requires the stated refusal with its class *and its phase*, so which bound binds is measured rather than read off a constant:

| Operations | Outcome | Class | The bound that produces it |
| --- | --- | --- | --- |
| 2..=62 | verifies | — | — |
| 63 | refuses | `BudgetExhausted` | `semantic_operations = 62`, raised *before* any target-qualified trace exists. It is now the only wall this family has. |

**So the reachable domain widened from thirty-one points to sixty-one, and the governed budget of 62 stopped being unreachable: it is the ladder's own widest measured point.** The phase is still compared even though one wall is left, because it is an independent property of that wall — a `semantic_operations` refusal that started arriving after planning would mean the program-size gate had moved behind the target loop, which the class alone cannot report.

### What was here yesterday, and why it is not a number swap

**The thirty-three-operation wall this table carried this morning is gone, and it took twenty-nine more with it.** 33..=62 refused `BudgetExhausted` because `region_members` was the bare constant **32**: a pointwise family's recognized partition is its whole program and nothing smaller is implementable, so the whole-program region was the only cover with a plan and above thirty-two operations it was refused as a *region* although every bound on the program's own *size* admitted it. [`derive-the-region-shape-budgets-from-the-declaration`](../../../tickets/derive-the-region-shape-budgets-from-the-declaration.md) made all three region-shape bounds derivations over the declaration rather than constants — `region_members` from `semantic_operations` (**62**), `region_live_values` from `semantic_values` (**80**), and `region_boundary_outputs` from the declared output count (**3**, *narrower* than the 8 it replaced) — on the ground that a region is a subset of the program it covers, so the stated admission envelope and the actual planning envelope became the same formulas over one declaration rather than two disagreeing ceilings. The narrowing does not bind here: this family declares one output.

That means **every ladder and wall table this spike retained before 2026-08-07's second run described a domain truncated by a bound that no longer exists**, and the four earlier `results/` files each stop on a different such bound. [`results/README.md`](results/README.md) names them, so a reader reconciling a retained file against a rerun can tell which regime it belongs to instead of reading a smaller ladder as a smaller domain.

**The twelve-operation wall this table carried on 2026-08-06 went the same way, in a different class.** 12..=62 refused `NoFeasiblePlan` because `region_expansions` (10,000) stopped candidate growth before the whole-program region was formed — candidate count falling from 66 at eleven operations to 20 at twelve — so every surviving cover named an unimplemented region and the portfolio was empty. That contradicted `DeterministicBudgets::governed`'s own claim that every `region_*` bound "bounds a *search*, and exhausting one costs an alternative while the verified input and complete coverage survive": growth reaches the whole-program set *last*, so the one candidate the profile could implement was the first thing an exhausted search lost. [`region-expansion-exhaustion-loses-the-only-feasible-plan`](../../../tickets/region-expansion-exhaustion-loses-the-only-feasible-plan.md) made region formation retain **both** coverage extremes before growth starts — the singletons and the whole-program region, symmetric with the cover authority, which already retains the fully-materialized and fused covers unconditionally and whose guarantee was empty without this one. It also corrected the claim: two of the five `region_*` bounds bound a search and three bound a region's admissible *shape*, and only the second kind can refuse a program. **All three of that second kind are now derivations, and none of them refuses this family.**

**The eleven-operation wall this table carried before that is gone the same way.** It refused `InvalidCompilerOutput` — documented as "always a defect in Tiler rather than in the caller's program" — with 3,478 records in the sealed trace, of which about 2,300 were one `selection.region-coverage.v1` record per *rejected cover* against a single unimplemented singleton region. The rule now emits one record per unimplemented *region*, carrying a `blocked-covers` count, so the record population is bounded by the region count rather than by the cover count. Filed and closed as [`refuse-nothing-legal-on-the-explain-detail-ceiling`](../../../tickets/refuse-nothing-legal-on-the-explain-detail-ceiling.md).

The predecessor of this section asserted a single nine-operation probe and one budget. It **fired on 2026-08-06** — the budget had moved from 8 to 62 and the probe compiled — which is the refusal it existed to report and the reason the table replaced it. The table has now fired three more times: the compiled-where-a-refusal-was-expected arm at eleven, again at twelve, and again this afternoon at thirty-three *and* sixty-two at once; and the class comparison at sixty-two, where one fix moved a refusal from `NoFeasiblePlan` to `BudgetExhausted` without removing it. That is why each wall is probed *with its class and phase*: a wall that moves in kind fails as loudly as one that disappears, and a wall that disappears because the bound behind it dissolved is the outcome this table exists to produce.

**One thing the table lost, stated rather than left to be discovered.** No point in this family now refuses *after* planning, so `--perturb=program` — which takes its program from the wall table precisely so it carries no standing claim of its own — no longer watches the planning-phase abort specifically. It still proves that a refused compilation stops the sweep. Restoring the planning-phase arm needs a point the wall table can also probe, and that is [`restore-a-planning-phase-refusal-to-the-identity-growth-harness`](../../../tickets/restore-a-planning-phase-refusal-to-the-identity-growth-harness.md).

## Result

**Measurement, 2026-08-07 (second run of the day)**, retained at [`results/2026-08-07-post-derived-region-budgets-apple-m4-max-macos27.0-26A5388g/growth.tsv`](results/2026-08-07-post-derived-region-budgets-apple-m4-max-macos27.0-26A5388g/growth.tsv). Host: macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max, load averages 18.37 / 16.16 / 11.92 at the run — a coordination host running other agents' builds, which the byte columns are immune to by construction and which the `compile_ms` column is not. Toolchain: `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`), the repository pin resolved by ancestry. Repository base `cee4fe1a`, plus this branch's harness changes; no `crates/` file was touched. Compile-only: nothing is emitted, linked, or dispatched. **Every byte column reproduced exactly on a second full sweep at the same commit**, which is what the harness's own two-compilations-per-row identity comparison already requires of each row. The predecessor run under [`results/2026-08-07-post-coverage-extremes-apple-m4-max-macos27.0-26A5388g/growth.tsv`](results/2026-08-07-post-coverage-extremes-apple-m4-max-macos27.0-26A5388g/growth.tsv) is this ladder's first thirty-one operation counts; its `graph_bytes` is identical here and its `program_bytes` and `coverage_bytes` are each smaller by exactly `5n − 4`. See [`results/README.md`](results/README.md).

| Operations | Coverage records | Graph identity (bytes) | Program identity (bytes) | Coverage bytes | Mean record (bytes) |
| --- | --- | --- | --- | --- | --- |
| 2 | 2 | 417 | 7,783 | 5,980 | 2,990.0 |
| 3 | 3 | 551 | 11,313 | 9,281 | 3,093.7 |
| 4 | 4 | 685 | 14,843 | 12,582 | 3,145.5 |
| 5 | 5 | 819 | 18,373 | 15,883 | 3,176.6 |
| 6 | 6 | 953 | 21,903 | 19,184 | 3,197.3 |
| 7 | 7 | 1,087 | 25,433 | 22,485 | 3,212.1 |
| 8 | 8 | 1,221 | 28,963 | 25,786 | 3,223.2 |
| 9 | 9 | 1,355 | 32,493 | 29,087 | 3,231.9 |
| 10 | 10 | 1,489 | 36,023 | 32,388 | 3,238.8 |
| 11 | 11 | 1,623 | 39,553 | 35,689 | 3,244.5 |
| 12 | 12 | 1,757 | 43,083 | 38,990 | 3,249.2 |
| 13 | 13 | 1,891 | 46,613 | 42,291 | 3,253.2 |
| 14 | 14 | 2,025 | 50,143 | 45,592 | 3,256.6 |
| 15 | 15 | 2,159 | 53,673 | 48,893 | 3,259.5 |
| 16 | 16 | 2,293 | 57,203 | 52,194 | 3,262.1 |
| 17 | 17 | 2,427 | 60,733 | 55,495 | 3,264.4 |
| 18 | 18 | 2,561 | 64,263 | 58,796 | 3,266.4 |
| 19 | 19 | 2,695 | 67,793 | 62,097 | 3,268.3 |
| 20 | 20 | 2,829 | 71,323 | 65,398 | 3,269.9 |
| 21 | 21 | 2,963 | 74,853 | 68,699 | 3,271.4 |
| 22 | 22 | 3,097 | 78,383 | 72,000 | 3,272.7 |
| 23 | 23 | 3,231 | 81,913 | 75,301 | 3,274.0 |
| 24 | 24 | 3,365 | 85,443 | 78,602 | 3,275.1 |
| 25 | 25 | 3,499 | 88,973 | 81,903 | 3,276.1 |
| 26 | 26 | 3,633 | 92,503 | 85,204 | 3,277.1 |
| 27 | 27 | 3,767 | 96,033 | 88,505 | 3,278.0 |
| 28 | 28 | 3,901 | 99,563 | 91,806 | 3,278.8 |
| 29 | 29 | 4,035 | 103,093 | 95,107 | 3,279.6 |
| 30 | 30 | 4,169 | 106,623 | 98,408 | 3,280.3 |
| 31 | 31 | 4,303 | 110,153 | 101,709 | 3,280.9 |
| 32 | 32 | 4,437 | 113,683 | 105,010 | 3,281.6 |
| 33 | 33 | 4,571 | 117,213 | 108,311 | 3,282.2 |
| 34 | 34 | 4,705 | 120,743 | 111,612 | 3,282.7 |
| 35 | 35 | 4,839 | 124,273 | 114,913 | 3,283.2 |
| 36 | 36 | 4,973 | 127,803 | 118,214 | 3,283.7 |
| 37 | 37 | 5,107 | 131,333 | 121,515 | 3,284.2 |
| 38 | 38 | 5,241 | 134,863 | 124,816 | 3,284.6 |
| 39 | 39 | 5,375 | 138,393 | 128,117 | 3,285.1 |
| 40 | 40 | 5,509 | 141,923 | 131,418 | 3,285.4 |
| 41 | 41 | 5,643 | 145,453 | 134,719 | 3,285.8 |
| 42 | 42 | 5,777 | 148,983 | 138,020 | 3,286.2 |
| 43 | 43 | 5,911 | 152,513 | 141,321 | 3,286.5 |
| 44 | 44 | 6,045 | 156,043 | 144,622 | 3,286.9 |
| 45 | 45 | 6,179 | 159,573 | 147,923 | 3,287.2 |
| 46 | 46 | 6,313 | 163,103 | 151,224 | 3,287.5 |
| 47 | 47 | 6,447 | 166,633 | 154,525 | 3,287.8 |
| 48 | 48 | 6,581 | 170,163 | 157,826 | 3,288.0 |
| 49 | 49 | 6,715 | 173,693 | 161,127 | 3,288.3 |
| 50 | 50 | 6,849 | 177,223 | 164,428 | 3,288.6 |
| 51 | 51 | 6,983 | 180,753 | 167,729 | 3,288.8 |
| 52 | 52 | 7,117 | 184,283 | 171,030 | 3,289.0 |
| 53 | 53 | 7,251 | 187,813 | 174,331 | 3,289.3 |
| 54 | 54 | 7,385 | 191,343 | 177,632 | 3,289.5 |
| 55 | 55 | 7,519 | 194,873 | 180,933 | 3,289.7 |
| 56 | 56 | 7,653 | 198,403 | 184,234 | 3,289.9 |
| 57 | 57 | 7,787 | 201,933 | 187,535 | 3,290.1 |
| 58 | 58 | 7,921 | 205,463 | 190,836 | 3,290.3 |
| 59 | 59 | 8,055 | 208,993 | 194,137 | 3,290.5 |
| 60 | 60 | 8,189 | 212,523 | 197,438 | 3,290.6 |
| 61 | 61 | 8,323 | 216,053 | 200,739 | 3,290.8 |
| 62 | 62 | 8,457 | 219,583 | 204,040 | 3,291.0 |

Coverage records equal the semantic operation count at every point, and the run refuses if they ever do not.

### The curve is exactly linear, and the fit is an equality rather than a resemblance

The first difference of the program-identity column is **3,530 at every step** and the second difference is **0**, so the general quadratic the harness fits comes back with a zero leading coefficient, and it is reported only after reproducing every measured point to the byte:

```
program_bytes(n) = 3530n + 723        residual 0 at all sixty-one points
graph_bytes(n)   =  134n + 149        residual 0 at all sixty-one points
```

| n | 2 | 3 | 4 | 32 | 33 | 51 | 60 | 61 | 62 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| measured | 7,783 | 11,313 | 14,843 | 113,683 | 117,213 | 180,753 | 212,523 | 216,053 | 219,583 |
| `3530n + 723` | 7,783 | 11,313 | 14,843 | 113,683 | 117,213 | 180,753 | 212,523 | 216,053 | 219,583 |
| residual | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

**The linear form survived a doubling of its domain and the coefficients did not, and the two facts have different causes.** Thirty new consecutive operation counts, 33..=62, landed on one straight line with a second difference of exactly zero — that is a check the form could have failed and did not, and it is the only kind of confirmation available now that no wall stands below `semantic_operations`. What did move is the line itself: the previous run's `3525n + 727` no longer reproduces any point, because every `program_bytes` value grew by exactly `5n − 4` under an index-refinement encoding step that landed between the two trees. `(3530n + 723) − (5n − 4) = 3525n + 727` exactly, so the older ladder is recovered from this one by subtraction rather than contradicted by it. [`results/README.md`](results/README.md) carries the attribution.

**That correction retires a claim this section used to make.** It said the fit "has been confirmed twenty-one steps outside the domain it was derived on", meaning `3525n + 727` was fitted on 2..=10 and then met the eleventh point and all of 12..=32 exactly as two defects stopped refusing them. Those confirmations were real on the trees that produced them and they do not carry to this one: the same points now measure `5n − 4` bytes higher. **A fit confirmed out of domain is a claim about one encoding, and it expires when the encoding moves.** What carries across all three trees is the *form* — every run since ADR 0104 has read a quadratic coefficient of exactly zero.

**The mechanism, stated as the relation a reader can check: the program curve's quadratic coefficient is 0 while the graph curve still grows at 134 bytes per operation.** On 2026-08-05 those two numbers were both 134, and their equality *was* the quadratic — one whole graph identity per coverage record, one record per operation, the product of a linear thing with a linear count. ADR 0104 replaced the per-record restatement with a fixed-width digest, so the per-record reference stopped scaling with the graph and the product is gone. The third column of the harness's structural decomposition shows it directly: the bytes one added operation adds to the whole coverage section are **3,301 at every step**, flat over all sixty consecutive steps, where under the restatement they climbed with `n`.

ADR 0104 predicted this curve before the fold landed and the prediction was met exactly on the tree it was written for: it derived `3525n + 719`, the 2026-08-06 run measured `3525n + 727`, and the eight-byte constant gap is the `tiler.kernel-program.v11` staged-realization step added after that arithmetic was written. The **linear coefficient** was predicted to the unit and stayed there through three trees; it reads 3,530 here only because of the later `5n − 4`.

### Why the observed exponent reads 0.98 and says nothing either way

A log-log least-squares slope over all sixty-one points is **0.9824**; over the thirty-one-point ladder it read **0.9745** and over the ten-point one **0.9559**. Over the *quadratic* encoding, on the same generator and a narrower ladder still, it read **1.0863**. Two encodings whose curves differ in degree produce exponents about 0.1 apart, all near one, because the exponent reports where the domain is rather than what the curve is: under the quadratic the linear term dominated everywhere a program could reach, and under the linear one the constant term still shifts the slope below one — **the three linear ladders creep 0.9559 → 0.9745 → 0.9824 toward one as the domain widens, without arriving.** Only the exact fit distinguishes them, which is why the harness fits the polynomial and prints the exponent beside it rather than reporting an exponent alone.

### The refusal point

**Extrapolation, labelled.** Solving the fitted curve against the bound, identity first exceeds `MAX_PROGRAM_IDENTITY_BYTES` at **n = 19,011 operations** (67,109,553 bytes at 19,011; 67,106,023 bytes at 19,010). The widest measured point — 62 operations, 219,583 bytes — is **0.327% of the bound**. It moved from 19,038 only because the slope moved by five bytes per operation; the widening of the ladder itself moved it not at all, because every new point landed on the fitted line rather than beside it.

**The extrapolation now has no out-of-domain confirmation, and this run removed the last one rather than adding any.** Under the quadratic encoding the retained 2026-08-05 file recorded the nine-operation point as a *confirmed wall* — `semantic_operations` was 8 there — so that fit was never checked outside 2..=8 at all. Under the linear encoding the checks were the eleventh point and then all of 12..=32, and both sets are now inside the domain; the thirty points this run added were not predictions of the fitted line, because the line moved under them. **There is no program this compilation path admits that the ladder does not already contain**, so no further confirmation is obtainable without moving `semantic_operations`. The refusal point is an extrapolation across nearly three orders of magnitude with nothing outside the ladder to check it against.

The fit is exact on its domain, and the domain is 2..=62 operations. Every coefficient is a property of this one program family: the per-operation slope depends on operation-key length, arity, result rank, attribute width, the region identity, the reached definitions, and the admission provenance. A richer family moves both coefficients. **The direction of that error is not neutral**: transformer families carry longer keys, wider attributes, and higher-rank results than a unary `f32` multiply, all of which *raise* the per-operation slope and therefore *lower* the refusal point. 19,011 is an upper-ish estimate of where the bound binds, not a floor — and the `5n − 4` step this run measured is a small worked example of the slope moving for a reason nothing about the program family predicts.

## Verdict: the margin holds by a wider margin, and the ceiling it holds against has changed

The ticket asked for one of two answers — a margin, or a follow-up decision ticket for a digest form. It got both, in that order: the 2026-08-05 quadratic gave a margin of ×125 and raised the digest question, that question became [ADR 0104](../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md), and the fold it decided is what this run measures.

**Fact — the roadmap's contemplated program sizes are per-layer, not per-model.** [Complete model ingestion and execution](../../../docs/research/program-planning/complete-model-ingestion-and-execution.md#whole-model-composition-three-programs-thirty-executions-three-identities) proposes one forward pass as **three semantic programs executed thirty times**: P1, the embedding gather, at 1 operation; P2, the decoder layer, at **≥ 51 operations**, executed 28 times against one artifact identity; and P3, the final norm and vocabulary projection, at 2 operations.

Evaluating the curve at those sizes, against both consumers:

| Program | Operations | Identity | Share of the 64 MiB program bound | Margin | Fixed content at ×2 | Share of the 1 MiB embedding ceiling |
| --- | --- | --- | --- | --- | --- | --- |
| P1 embedding gather | 1 | 4,253 B (fitted) | 0.006% | ×15,779 | 8,506 B | 0.81% |
| P3 norm and vocabulary projection | 2 | 7,783 B (**measured**) | 0.012% | ×8,622 | 15,566 B | 1.48% |
| **P2 decoder layer** | **≥ 51** | **180,753 B (measured at 51)** | **0.27%** | **×371** | **361,506 B** | **34.5%** |

P1 sits one operation *below* the ladder's floor — the chain needs a multiply to make its constant output-reachable — so its figure is an extrapolation in the other direction. P3's is the ladder's own two-operation row.

**P2's row stopped being an extrapolation.** Fifty-one operations sat nineteen steps above the ladder's top this morning and its figure was solved from the fit; it is now a measured row, and the fitted and measured values agree because the fit reproduces every point to the byte. What that measures is *this* family at fifty-one occurrences, not the decoder layer: the chain is a unary `f32` multiply chain and the layer is not, so the coefficient caveat below is untouched and the comparison remains an inference about a different program with the same occurrence count. What changed is that the ladder no longer has to reach across a wall to state it.

**So the 64 MiB bound is unreachable for the program sizes this roadmap contemplates, with a margin of about ×371 in bytes and ×373 in operation count** (19,011 fitted refusal against P2's 51), where before the fold it was ×125 and ×13.6. The margin is robust to the coefficient being wrong in the unfavourable direction: for the bound to bind at 51 operations the per-operation slope would have to be **1,315,845 bytes rather than the measured 3,530**, a 373× increase that no plausible widening of operation-key length, arity, or attribute width produces.

### What the fold changed about the contingency, and it is the interesting half

**Inference — a whole-model program now fits the program-identity bound, and the argument against fusing across layers has moved to a different ceiling.** [The transformer operation and shape surface derivation](../../../docs/research/shapes/transformer-operation-and-shape-surface.md#occurrence-inventory-for-one-forward-pass) inventories one Qwen3-0.6B forward pass at **≥ 1,068 semantic occurrences**. Compiled as a single semantic program that is **3,770,763 bytes — 3.60 MiB, 5.6% of the 64 MiB bound**, where the pre-fold curve put it at ≈ 149 MiB and a hard typed refusal. It remains an extrapolation seventeen times the ladder's top, which the widening did not change.

That is a reversal of this spike's own previous verdict and it must not be read as a licence. **The per-layer partition is still load-bearing on size**, and the ceiling that says so is the per-invocation embedding one: at the post-[ADR 0103](../../../docs/decisions/0103-declare-the-manifests-artifact-identity-by-digest.md) envelope multiplicity of two, the same whole-model program's fixed content is **7,541,526 bytes against a 1,048,576-byte ceiling, 7.2× over** — and that ceiling has no typed refusal at the artifact layer at all, so its failure mode is an artifact that compiles and cannot be embedded. The same curve crosses it between **148 and 149 operations** (`2 × (3530·148 + 723) = 1,046,326`; `2 × (3530·149 + 723) = 1,053,386`), where before the fold it crossed between 50 and 51 — that is, *at* the roadmap's own decoder layer. **The five-byte slope move did not shift that crossing**, which is the one place the new coefficients could plausibly have changed a conclusion and did not: the crossing sits 148.3 operations along a line whose slope moved by 0.14%.

What survives unchanged from the 2026-08-05 verdict is its shape: the per-layer cut, grounded in that record on artifact-identity reuse and layer-count independence, has a second and independent size ground its own derivation never mentions. What changed is which ceiling supplies it, and how much room is left — P2 at 34.5% of the embedding ceiling rather than the 102% the pre-fold curve gave it at the same multiplicity of two (`2 × (134·51² + 3650·51 + 727) = 1,070,822`).

## Boundary

- **One program family** — a unary `f32` multiply chain over a rank-1 extent-4 tensor — one contract (`FLUSH_SUBNORMALS_TO_ZERO_F32`), one target profile (the authoritative macOS Apple9 declaration), `f32` only. Both fitted coefficients are that family's.
- **Sixty-one points, 2..=62 operations.** Not a sampling choice, and for the first time it *is* the governed budget: the ordinary compilation path refuses at sixty-three on `semantic_operations`, and the run proves the one remaining wall by compiling at it and requiring its class and its phase. Widening further is a budget decision, not a harness one.
- **The coefficients are pinned to one compiler tree and moved between the last two.** `3530n + 723` describes base `cee4fe1a`; the previous run's `3525n + 727` described `d050f10a` plus a branch, and the difference is exactly `5n − 4`. A retained result is a statement about the tree it names, not about this program family alone — [`results/README.md`](results/README.md) is the index.
- **The refusal point is an extrapolation across nearly three orders of magnitude, and it now has no out-of-domain confirmation and no way to obtain one.** Every wall that once supplied one fell, each confirmation became a ladder row, and the ladder now covers every program the path admits. It is the order of magnitude at which the bound becomes binding, not a number a caller may rely on.
- **The P2 and whole-model comparisons are inferences over a second document's inferences**, not measurements: nothing here compiled a transformer. Fifty-one occurrences is now a measured *row of this family*, which is not the same statement as measuring the decoder layer. Both source counts are explicit lower bounds (`≥ 51`, `≥ 1,068`), the 1,068-occurrence figure remains an extrapolation seventeen times the ladder's top, and the per-layer partition they rest on is a `Proposal` with `disposition: pending` rather than an accepted decision. The claim is that the numbers are separated by orders of magnitude, not that either program was observed.
- **The envelope multiplicity of two is imported, not measured here.** It comes from [the manifest-growth attribution](../../../docs/research/artifacts/manifest-fixed-content-growth.md), measured on one fixture for one landing's coverage increment; every embedding-ceiling figure above inherits that bound.
- **Compile-only.** No kernel was emitted, linked, or dispatched, so this spike makes no performance claim. The `compile_ms` column is reachability information — the minimum of the two runs behind each row, on a coordination host running several other agents' builds at load average 18 — and not a benchmark. It runs 1, 1, 1, 2, 2, 3, 6, 11, 22, 45 ms up to eleven operations, roughly doubling per step, then **falls to 6 ms at twelve and climbs smoothly to 20 ms at sixty-two**. The doubling is the connected-set enumeration, which is exponential in this family's fan shape; `region_expansions` (10,000) is what stops it, and it first binds at twelve. So the discontinuity is the budget engaging, and everything above it is bounded work growing gently with program size.
- **`CompileFailureClass::BudgetExhausted` carries no resource, limit, or actual value.** The sixty-three-operation wall's refusal also arrives before a target-qualified explain trace, so it can report *that* a budget refused but not *which*; attribution to `semantic_operations` is read from the source cited above rather than from the refusal. That gap is [`carry-the-exhausted-resource-through-the-budget-refusal`](../../../tickets/carry-the-exhausted-resource-through-the-budget-refusal.md), and it now covers **every** wall this spike has, because the walls that did seal a trace — and whose `budget-stop:region-members:32:33` supplied their own attribution — are gone.
- **No point in this family refuses after planning any more**, so `--perturb=program` no longer watches the planning-phase abort specifically. See [`restore-a-planning-phase-refusal-to-the-identity-growth-harness`](../../../tickets/restore-a-planning-phase-refusal-to-the-identity-growth-harness.md).
