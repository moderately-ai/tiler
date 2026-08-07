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
last_verified: "2026-08-06"
ticket: "measure-executable-coverage-identity-growth-against-the-program-identity-bound"
---

# How kernel-program identity grows against its 64 MiB bound

[`measure-executable-coverage-identity-growth-against-the-program-identity-bound`](../../../tickets/measure-executable-coverage-identity-growth-against-the-program-identity-bound.md) inherited a structural inference with exactly one measured point behind it: because `CanonicalKernelProgramIdentity` embeds one whole reached-only executable-coverage identity per covered occurrence, one record per graph operation, and because each of those records embedded the complete `SemanticGraphIdentity` of the bound graph, program identity should be **Θ(operations × graph-encoding size)** — quadratic in graph size — against a hard `MAX_PROGRAM_IDENTITY_BYTES` of 64 MiB that fails closed with a typed refusal. The one measurement was a five-occurrence stage key at 21,366 bytes. What was unknown was how far from realistic program sizes the refusal sits.

This spike replaced that single point with a curve. On 2026-08-05 the curve was the quadratic the inference predicted. [ADR 0104](../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) then folded the per-record graph restatement to a fixed-width digest, and the run below measures what that did: **the curve is linear, and the quadratic coefficient is exactly zero rather than small.** Both results are retained, each at its own dated path, because each is evidence about the encoding it measured.

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
cargo run --release -- --perturb=program    # a program no plan covers
cargo run --release -- --perturb=coverage   # a corrupted coverage expectation
cargo run --release -- --perturb=fit        # one byte moved in one measured row
cargo run --release -- --perturb=wall       # the wrong class expected at one wall
```

No `make` target reaches here, per [`spikes/README.md`](../../README.md).

## The domain is nine points, and the reason it is not sixty-one is a finding

**Fact.** `DeterministicBudgets::governed` (`crates/tiler-compiler/src/request.rs`) caps `semantic_operations` at **62**, raised from 8 on 2026-08-05 and sized "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit". `DeterministicBudgets` is `pub(crate)` and `CompileRequest` binds `InstalledCapabilities::governed`, so no public caller can state a wider budget.

**Measurement — that budget is not the wall, and the ladder is nine points rather than sixty-one because two other bounds refuse first.** The run compiles at every one of the four points below and requires each stated refusal, so which bound binds is measured rather than read off a constant:

| Operations | Outcome | Class | The bound that produces it |
| --- | --- | --- | --- |
| 2..=10 | verifies | — | — |
| 11 | refuses | `InvalidCompilerOutput` | the explain authority's detail ceiling (`MAX_RECORDS`, 4,096, in `crates/tiler-compiler/src/explain.rs`), exhausted by cover enumeration's per-alternative rejection records — 3,478 in the sealed trace. Region formation itself succeeds here: 66 candidates, no budget stop. |
| 12..=62 | refuses | `NoFeasiblePlan` | `region_expansions` (10,000) stops candidate growth before the whole-program region is formed — `budget-stop:region-expansions:10000:10001`, candidate count falling from 66 at eleven operations to 20 at twelve — so every surviving cover names an unimplemented region and the portfolio is empty. |
| 63 | refuses | `BudgetExhausted` | `semantic_operations = 62`, the one wall here that is about program size. |

**So the reachable domain widened by two operations rather than by fifty-four, and the governed budget of 62 is measured to be unreachable rather than assumed to bound the ladder.** Two of the three walls are search bounds whose exhaustion the compiler reports as a defect or as an infeasible target, and neither is the bound the budget widening moved.

**Inference — two of these are defects rather than properties, and neither is this spike's to fix.** `InvalidCompilerOutput` is documented as "always a defect in Tiler rather than in the caller's program", so an eleven-operation program inside every stated budget refusing under it is a defect by the compiler's own classification; and `DeterministicBudgets::governed`'s doc comment states that "`normalization_rewrites` and every `region_*` bound … bounds a *search*, and exhausting one costs an alternative while the verified input and complete coverage survive", which the twelve-operation row contradicts — exhausting `region_expansions` there costs the only feasible plan. Both are filed: [`refuse-nothing-legal-on-the-explain-detail-ceiling`](../../../tickets/refuse-nothing-legal-on-the-explain-detail-ceiling.md) and [`region-expansion-exhaustion-loses-the-only-feasible-plan`](../../../tickets/region-expansion-exhaustion-loses-the-only-feasible-plan.md).

The previous form of this section asserted a single nine-operation probe and one budget. It **fired on 2026-08-06** — the budget had moved and the probe compiled — which is the refusal it existed to report and the reason the table above replaces it. A single point cannot re-anchor the discipline now that three distinct bounds refuse between the ladder's top and the governed budget, so each is probed *with its class*: a wall that moves in kind fails as loudly as one that disappears.

## Result

**Measurement, 2026-08-06**, retained at [`results/2026-08-06-apple-m4-max-macos27.0-26A5388g/growth.tsv`](results/2026-08-06-apple-m4-max-macos27.0-26A5388g/growth.tsv). Host: macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max, load averages 2.95 / 6.72 / 9.06 at the run — a shared host, which the byte columns are immune to by construction and which the `compile_ms` column is not. Toolchain: `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`), the repository pin resolved by ancestry. Repository base `dd9def76`, plus this branch's harness change. Compile-only: nothing is emitted, linked, or dispatched.

| Operations | Coverage records | Graph identity (bytes) | Program identity (bytes) | Coverage bytes | Mean record (bytes) |
| --- | --- | --- | --- | --- | --- |
| 2 | 2 | 417 | 7,777 | 5,974 | 2,987.0 |
| 3 | 3 | 551 | 11,302 | 9,270 | 3,090.0 |
| 4 | 4 | 685 | 14,827 | 12,566 | 3,141.5 |
| 5 | 5 | 819 | 18,352 | 15,862 | 3,172.4 |
| 6 | 6 | 953 | 21,877 | 19,158 | 3,193.0 |
| 7 | 7 | 1,087 | 25,402 | 22,454 | 3,207.7 |
| 8 | 8 | 1,221 | 28,927 | 25,750 | 3,218.8 |
| 9 | 9 | 1,355 | 32,452 | 29,046 | 3,227.3 |
| 10 | 10 | 1,489 | 35,977 | 32,342 | 3,234.2 |

Coverage records equal the semantic operation count at every point, and the run refuses if they ever do not.

### The curve is exactly linear, and the fit is an equality rather than a resemblance

The first difference of the program-identity column is **3,525 at every step** and the second difference is **0**, so the general quadratic the harness fits comes back with a zero leading coefficient, and it is reported only after reproducing every measured point to the byte:

```
program_bytes(n) = 3525n + 727        residual 0 at all nine points
graph_bytes(n)   =  134n + 149        residual 0 at all nine points
```

| n | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| measured | 7,777 | 11,302 | 14,827 | 18,352 | 21,877 | 25,402 | 28,927 | 32,452 | 35,977 |
| `3525n + 727` | 7,777 | 11,302 | 14,827 | 18,352 | 21,877 | 25,402 | 28,927 | 32,452 | 35,977 |
| residual | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

**The mechanism, stated as the relation a reader can check: the program curve's quadratic coefficient is 0 while the graph curve still grows at 134 bytes per operation.** On 2026-08-05 those two numbers were both 134, and their equality *was* the quadratic — one whole graph identity per coverage record, one record per operation, the product of a linear thing with a linear count. ADR 0104 replaced the per-record restatement with a fixed-width digest, so the per-record reference stopped scaling with the graph and the product is gone. The third column of the harness's structural decomposition shows it directly: the bytes one added operation adds to the whole coverage section are **3,296 at every step**, flat, where under the restatement they climbed with `n`.

The 2026-08-05 result predicted this curve to the unit before the fold landed — ADR 0104 derived `3525n + 719`, and the constant is 727 because the `tiler.kernel-program.v11` staged-realization step added eight unconditional bytes after that arithmetic was written. The measured linear coefficient is the predicted 3,525 exactly.

### Why the observed exponent reads 0.95 and says nothing either way

A log-log least-squares slope over all nine points is **0.9538**. Over the *quadratic* encoding, on the same generator and a narrower ladder, it read **1.0863**. Two encodings whose curves differ in degree produce exponents 0.13 apart, both near one, because the exponent reports where the domain is rather than what the curve is: under the quadratic the linear term dominated everywhere a program could reach, and under the linear one the constant term still shifts the slope below one. Only the exact fit distinguishes them, which is why the harness fits the polynomial and prints the exponent beside it rather than reporting an exponent alone.

### The refusal point

**Extrapolation, labelled.** Solving the fitted curve against the bound, identity first exceeds `MAX_PROGRAM_IDENTITY_BYTES` at **n = 19,038 operations** (67,109,677 bytes at 19,038; 67,106,152 bytes at 19,037). The widest measured point — 10 operations, 35,977 bytes — is **0.054% of the bound**.

**The extrapolation is now weaker than it was on 2026-08-05, in one specific respect that the wall table is what makes visible.** The quadratic result had an independent check outside its own domain: the nine-operation wall probe compiled, and `134·9² + 3650·9 + 719 = 44,423` reproduced its measured identity to the byte. That point is now *inside* the domain — it is the ladder's ninth row — and there is no longer any out-of-domain point to check against, because the ordinary compilation path refuses every program above ten operations for reasons that are not program size. The refusal point is therefore an extrapolation across three orders of magnitude with no confirmation beyond its own ladder.

The fit is exact on its domain, and the domain is 2..=10 operations. Every coefficient is a property of this one program family: the per-operation slope depends on operation-key length, arity, result rank, attribute width, the region identity, the reached definitions, and the admission provenance. A richer family moves both coefficients. **The direction of that error is not neutral**: transformer families carry longer keys, wider attributes, and higher-rank results than a unary `f32` multiply, all of which *raise* the per-operation slope and therefore *lower* the refusal point. 19,038 is an upper-ish estimate of where the bound binds, not a floor.

## Verdict: the margin holds by a wider margin, and the ceiling it holds against has changed

The ticket asked for one of two answers — a margin, or a follow-up decision ticket for a digest form. It got both, in that order: the 2026-08-05 quadratic gave a margin of ×125 and raised the digest question, that question became [ADR 0104](../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md), and the fold it decided is what this run measures.

**Fact — the roadmap's contemplated program sizes are per-layer, not per-model.** [Complete model ingestion and execution](../../../docs/research/program-planning/complete-model-ingestion-and-execution.md#whole-model-composition-three-programs-thirty-executions-three-identities) proposes one forward pass as **three semantic programs executed thirty times**: P1, the embedding gather, at 1 operation; P2, the decoder layer, at **≥ 51 operations**, executed 28 times against one artifact identity; and P3, the final norm and vocabulary projection, at 2 operations.

Evaluating the fitted curve at those sizes, against both consumers:

| Program | Operations | Fitted identity | Share of the 64 MiB program bound | Margin | Fixed content at ×2 | Share of the 1 MiB embedding ceiling |
| --- | --- | --- | --- | --- | --- | --- |
| P1 embedding gather | 1 | 4,252 B | 0.006% | ×15,783 | 8,504 B | 0.81% |
| P3 norm and vocabulary projection | 2 | 7,777 B (measured) | 0.012% | ×8,629 | 15,554 B | 1.48% |
| **P2 decoder layer** | **≥ 51** | **180,502 B** | **0.27%** | **×372** | **361,004 B** | **34.4%** |

P1 sits one operation *below* the ladder's floor — the chain needs a multiply to make its constant output-reachable — so its figure is an extrapolation in the other direction. P3's is the ladder's own two-operation row.

**So the 64 MiB bound is unreachable for the program sizes this roadmap contemplates, with a margin of about ×372 in bytes and ×373 in operation count** (19,038 fitted refusal against P2's 51), where before the fold it was ×125 and ×13.6. The margin is robust to the coefficient being wrong in the unfavourable direction: for the bound to bind at 51 operations the per-operation slope would have to be **1,315,845 bytes rather than the measured 3,525**, a 373× increase that no plausible widening of operation-key length, arity, or attribute width produces.

### What the fold changed about the contingency, and it is the interesting half

**Inference — a whole-model program now fits the program-identity bound, and the argument against fusing across layers has moved to a different ceiling.** [The transformer operation and shape surface derivation](../../../docs/research/shapes/transformer-operation-and-shape-surface.md#occurrence-inventory-for-one-forward-pass) inventories one Qwen3-0.6B forward pass at **≥ 1,068 semantic occurrences**. Compiled as a single semantic program that is **3,765,427 bytes — 3.59 MiB, 5.6% of the 64 MiB bound**, where the pre-fold curve put it at ≈ 149 MiB and a hard typed refusal.

That is a reversal of this spike's own previous verdict and it must not be read as a licence. **The per-layer partition is still load-bearing on size**, and the ceiling that says so is the per-invocation embedding one: at the post-[ADR 0103](../../../docs/decisions/0103-declare-the-manifests-artifact-identity-by-digest.md) envelope multiplicity of two, the same whole-model program's fixed content is **7,530,854 bytes against a 1,048,576-byte ceiling, 7.2× over** — and that ceiling has no typed refusal at the artifact layer at all, so its failure mode is an artifact that compiles and cannot be embedded. The same curve crosses it between **148 and 149 operations** (`2 × (3525·148 + 727) = 1,044,854`; `2 × (3525·149 + 727) = 1,051,904`), where before the fold it crossed between 50 and 51 — that is, *at* the roadmap's own decoder layer.

What survives unchanged from the 2026-08-05 verdict is its shape: the per-layer cut, grounded in that record on artifact-identity reuse and layer-count independence, has a second and independent size ground its own derivation never mentions. What changed is which ceiling supplies it, and how much room is left — P2 at 34.4% of the embedding ceiling rather than the 102% the pre-fold curve gave it at the same multiplicity of two (`2 × (134·51² + 3650·51 + 727) = 1,070,822`).

## Boundary

- **One program family** — a unary `f32` multiply chain over a rank-1 extent-4 tensor — one contract (`FLUSH_SUBNORMALS_TO_ZERO_F32`), one target profile (the authoritative macOS Apple9 declaration), `f32` only. Both fitted coefficients are that family's.
- **Nine points, 2..=10 operations.** Not a sampling choice, and not the governed budget either: the ordinary compilation path refuses at eleven for a defect and at twelve for an exhausted search bound, and the run proves all three walls by compiling at each and requiring its class.
- **The refusal point is an extrapolation across three orders of magnitude**, with no out-of-domain confirmation available — unlike the 2026-08-05 quadratic, whose wall probe checked it one step outside its domain. It is the order of magnitude at which the bound becomes binding, not a number a caller may rely on.
- **The P2 and whole-model comparisons are inferences over a second document's inferences**, not measurements: nothing here compiled a transformer, and nothing can while the compilation path refuses above ten operations. Both source counts are explicit lower bounds (`≥ 51`, `≥ 1,068`), and the per-layer partition they rest on is a `Proposal` with `disposition: pending` rather than an accepted decision. The claim is that the numbers are separated by orders of magnitude, not that either program was observed.
- **The envelope multiplicity of two is imported, not measured here.** It comes from [the manifest-growth attribution](../../../docs/research/artifacts/manifest-fixed-content-growth.md), measured on one fixture for one landing's coverage increment; every embedding-ceiling figure above inherits that bound.
- **Compile-only.** No kernel was emitted, linked, or dispatched, so this spike makes no performance claim. The `compile_ms` column is reachability information — the minimum of the two runs behind each row, on a host running other work — and not a benchmark. It runs 1, 2, 2, 2, 4, 7, 13, 28, 64 ms and roughly doubles per operation above seven, which is the cover enumeration whose rejection records exhaust the explain ceiling two rows later.
- **`CompileFailureClass::BudgetExhausted` carries no resource, limit, or actual value**, and the refusal arrives before a target-qualified explain trace, so the sixty-three-operation wall can report *that* a budget refused but not *which*. Attribution to `semantic_operations` is read from the source cited above, not from the refusal. That gap is [`carry-the-exhausted-resource-through-the-budget-refusal`](../../../tickets/carry-the-exhausted-resource-through-the-budget-refusal.md). The other three walls do carry a trace, and their attributions above are read from it.
