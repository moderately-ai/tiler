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
last_verified: "2026-08-05"
ticket: "measure-executable-coverage-identity-growth-against-the-program-identity-bound"
---

# How kernel-program identity grows against its 64 MiB bound

[`measure-executable-coverage-identity-growth-against-the-program-identity-bound`](../../../tickets/measure-executable-coverage-identity-growth-against-the-program-identity-bound.md) inherited a structural inference with exactly one measured point behind it: because `CanonicalKernelProgramIdentity` embeds one whole reached-only executable-coverage identity per covered occurrence, one record per graph operation, and because each of those records embeds the complete `SemanticGraphIdentity` of the bound graph, program identity should be **Θ(operations × graph-encoding size)** — quadratic in graph size — against a hard `MAX_PROGRAM_IDENTITY_BYTES` of 64 MiB that fails closed with a typed refusal. The one measurement was a five-occurrence stage key at 21,366 bytes. What was unknown was how far from realistic program sizes the refusal sits.

This spike replaces that single point with a curve, and the answer is not the reassuring one.

## What it drives

For each operation count in the reachable domain it builds a semantic program, compiles it through the **ordinary** path — the public `tiler_compiler::session::compile` boundary, whose lowering mints real index-refinement receipts, derives `CoveredOccurrence` records from them, and drives `KernelProgramBuilder` — and reads the byte length of the canonical identity off the verified program the compilation produced. Nothing here constructs an identity, a receipt, or a coverage record; a synthetic one would measure the harness rather than the compiler.

The generator emits one input, one hoisted constant, and a chain of `F32Multiply` steps, so the operation count is exactly `1 + multiplies` and every integer in the domain is reachable. It is a pure multiply chain rather than a mixed multiply/add body because a region holding a multiply adjacent to an add is refused under the one contract that permits arithmetic contraction, and a generator whose admissibility depended on the contract would put a second variable into a one-variable sweep.

## Running it

```sh
cd spikes/program-planning/identity-growth
cargo run --release
```

Three perturbations exist so that the harness's refusals are watched rather than trusted, and each exits non-zero:

```sh
cargo run --release -- --perturb=program    # a program that cannot lower
cargo run --release -- --perturb=coverage   # a corrupted coverage expectation
cargo run --release -- --perturb=fit        # one byte moved in one measured row
```

No `make` target reaches here, per [`spikes/README.md`](../../README.md).

## The domain is seven points, and that is the whole of it

**Fact.** `DeterministicBudgets::governed` (`crates/tiler-compiler/src/request.rs:818-835`) caps `semantic_operations` at **8**. `DeterministicBudgets` is `pub(crate)`, and `CompileRequest` binds `InstalledCapabilities::governed`, so no public caller can state a wider budget. The ladder 2..=8 is therefore not a sample of the reachable domain; it *is* the reachable domain, and a nine-operation program is refused with `CompileFailureClass::BudgetExhausted` before any kernel program exists.

The run demonstrates that wall rather than reading it off the constant, because a budget named in source and a budget that actually refuses are different facts. If the probe ever *succeeds*, the run fails and says the recorded result is stale — a moved budget widens the domain and invalidates the ladder, which is a finding rather than a pass.

## Result

**Measurement, 2026-08-05**, retained at [`results/2026-08-05-apple-m4-max-macos27.0-26A5388g/growth.tsv`](results/2026-08-05-apple-m4-max-macos27.0-26A5388g/growth.tsv). Host: macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max. Toolchain: `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`), the repository pin resolved by ancestry. Repository base `5f14cd116a7da52184b05ff81e5c930c136e32df`. Compile-only: nothing is emitted, linked, or dispatched.

| Operations | Coverage records | Graph identity (bytes) | Program identity (bytes) | Coverage bytes | Mean record (bytes) |
| --- | --- | --- | --- | --- | --- |
| 2 | 2 | 417 | 8,546 | 6,760 | 3,380.0 |
| 3 | 3 | 551 | 12,866 | 10,851 | 3,617.0 |
| 4 | 4 | 685 | 17,454 | 15,210 | 3,802.5 |
| 5 | 5 | 819 | 22,310 | 19,837 | 3,967.4 |
| 6 | 6 | 953 | 27,434 | 24,732 | 4,122.0 |
| 7 | 7 | 1,087 | 32,826 | 29,895 | 4,270.7 |
| 8 | 8 | 1,221 | 38,486 | 35,326 | 4,415.8 |

Coverage records equal the semantic operation count at every point, and the run refuses if they ever do not.

### The curve is exactly quadratic, and the fit is an equality rather than a resemblance

The second difference of the program-identity column is **268 at every step**, so an exact quadratic exists and the harness reports it only after reproducing every measured point to the byte:

```
program_bytes(n) = 134n² + 3650n + 710
graph_bytes(n)   =  134n  +  149
```

**The quadratic coefficient of the program curve (134) is the per-operation slope of the graph curve (134).** That equality is the mechanism rather than a coincidence of this ladder: one whole graph identity is embedded per coverage record, and there is one record per operation, so the product of a linear thing with a linear count is what makes the total quadratic. The structural inference the ticket carried is confirmed, and confirmed by the identity that generates it rather than by a fitted exponent.

### Why the observed exponent reads 1.09 and does not refute that

A log-log least-squares slope over all seven points is **1.0863**, and read alone it would look like a refutation. It is not. The linear term dominates until `n = 3650/134 ≈ 27` operations, and the governed budget stops the domain at 8 — so every point that can be measured sits in the region where the curve is still essentially linear. The exponent reports where the domain is, not what the curve is. This is why the harness fits the polynomial and reports the exponent beside it, rather than reporting an exponent alone.

### The refusal point

**Extrapolation, labelled.** Solving the fitted curve against the bound, identity first exceeds `MAX_PROGRAM_IDENTITY_BYTES` at **n = 695 operations** (67,262,810 bytes at 695; 67,073,034 bytes at 694, just inside). The widest measured point — 8 operations, 38,486 bytes — is **0.057% of the bound**.

The fit is exact on its domain, and the domain is 2..=8 operations. Every coefficient is a property of this one program family: the graph-identity slope depends on operation-key length, arity, result rank, and attribute width, and the per-record remainder depends on the region identity, the reached definitions, and the admission provenance. A richer family moves all three. **The direction of that error is not neutral**: transformer families carry longer keys, wider attributes, and higher-rank results than a unary `f32` multiply, all of which *raise* the per-operation graph slope and therefore *lower* the refusal point. 695 is an upper-ish estimate of where the bound binds, not a floor.

## Verdict: the margin holds, and it holds because of the per-layer partition

The ticket asked for one of two answers — a margin, or a follow-up decision ticket for a digest form. **It is the margin**, and the number depends entirely on a composition decision that was taken for unrelated reasons.

**Fact — the roadmap's contemplated program sizes are per-layer, not per-model.** [Complete model ingestion and execution](../../../docs/research/program-planning/complete-model-ingestion-and-execution.md#whole-model-composition-three-programs-thirty-executions-three-identities) proposes one forward pass as **three semantic programs executed thirty times**: P1, the embedding gather, at 1 operation; P2, the decoder layer, at **≥ 51 operations**, executed 28 times against one artifact identity; and P3, the final norm and vocabulary projection, at 2 operations. The per-layer boundary was chosen for artifact reuse and layer-count independence, and the record is explicit that it buys "a compiled program one twenty-eighth the size".

Evaluating the fitted curve at those sizes:

| Program | Operations | Fitted identity | Share of the 64 MiB bound | Margin |
| --- | --- | --- | --- | --- |
| P1 embedding gather | 1 | 4,494 B | 0.007% | ×14,900 |
| P3 norm and vocabulary projection | 2 | 8,546 B | 0.013% | ×7,850 |
| **P2 decoder layer** | **≥ 51** | **535,394 B (0.51 MiB)** | **0.80%** | **×125** |

**So the bound is unreachable for the program sizes this roadmap contemplates, with a margin of about 125× in bytes and 13.6× in operation count** (695 fitted refusal against P2's 51). The margin is robust to the coefficients being wrong in the unfavourable direction: for the bound to bind at 51 operations the per-operation graph slope would have to be **25,801 bytes rather than the measured 134**, a 193× increase that no plausible widening of operation-key length, arity, or attribute width produces.

### The margin is contingent, and this is the part worth carrying forward

**Inference — a whole-model program would not fit, and the gap is not marginal.** [The transformer operation and shape surface derivation](../../../docs/research/shapes/transformer-operation-and-shape-surface.md#occurrence-inventory-for-one-forward-pass) inventories one Qwen3-0.6B forward pass at **≥ 1,068 semantic occurrences**. Compiled as a single semantic program that evaluates to **≈ 149 MiB, about 2.3× the bound** — a hard typed refusal, past the fitted refusal point of 695 by roughly 1.5×.

That is not the current design, and this spike does not argue it should be. What it establishes is that **the per-layer partition is load-bearing for a reason its own derivation never mentions.** The record grounds the cut in artifact-identity reuse and says so explicitly — "three programs rather than one, and the ground is artifact-identity reuse rather than size" — and the size consequence measured here is a second, independent reason the same cut is correct. Anything that later reconsiders the boundary toward whole-model fusion, for cost or scheduling reasons, is trading against a 64 MiB ceiling it would have no reason to know about.

The digest question is therefore **deferred rather than open**: [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](../../../tickets/decide-whether-executable-coverage-evidence-folds-as-a-digest.md) is filed at `deferred` and carries the activation triggers — a program boundary that admits more than ~350 operations, or a measured per-operation graph slope materially above 134. It is a decision ticket and not a fix, because replacing the embedded graph identity changes the accepted `tiler.ir.index-refinement-executable-coverage.v1` projection, which is an identity-domain decision this ticket explicitly does not take. The redundancy such a proposal would rest on is already provable and is recorded in the encoding itself: `encode_identity` writes the program's one `SemanticGraphIdentity` above the stage section, and the builder proves every coverage record names that same graph, so the per-record copy determines nothing the encoding has not already fixed.

## Boundary

- **One program family** — a unary `f32` multiply chain over a rank-1 extent-4 tensor — one contract (`FLUSH_SUBNORMALS_TO_ZERO_F32`), one target profile (the authoritative macOS Apple9 declaration), `f32` only. Every fitted coefficient is that family's.
- **Seven points, 2..=8 operations.** Not a sampling choice; the governed `semantic_operations` budget admits no more, and the run proves it by watching the ninth refuse.
- **The refusal point is an extrapolation across roughly two orders of magnitude** from a curve fitted below the crossover where its own quadratic term starts to dominate. It is the order of magnitude at which the bound becomes binding, not a number a caller may rely on.
- **The P2 and whole-model comparisons are inferences over a second document's inferences**, not measurements: nothing here compiled a transformer, and nothing can while the budget stands at 8. Both source counts are explicit lower bounds (`≥ 51`, `≥ 1,068`), and the per-layer partition they rest on is a `Proposal` with `disposition: pending` rather than an accepted decision. The claim is that the numbers are separated by two orders of magnitude, not that either program was observed.
- **Compile-only.** No kernel was emitted, linked, or dispatched, so this spike makes no performance claim. The `compile_ms` column is reachability information — the minimum of the two runs behind each row — and not a benchmark.
- **`CompileFailureClass::BudgetExhausted` carries no resource, limit, or actual value**, and the refusal arrives before a target-qualified explain trace, so the wall probe can report *that* a budget refused but not *which*. Attribution to `semantic_operations` is read from the source cited above, not from the refusal. That gap is [`carry-the-exhausted-resource-through-the-budget-refusal`](../../../tickets/carry-the-exhausted-resource-through-the-budget-refusal.md).

## Why neither catalog lists this record yet

Stated so the absence reads as a deferral rather than an oversight. Both hand-maintained catalogs — [`spikes/README.md`](../../README.md) and [the research catalog](../../../docs/research/README.md) — are in `contracts/navigation`, a scope the ticket that produced this spike does not hold and that a live ticket does. The research catalog was refuted outright by that worker's actual branch diff; the experiment catalog was file-level disjoint and therefore admissible, but declaring the scope to write one appended row made `tkt why` report a batch conflict against a live p1 ticket. Both rows are owned by [`add-the-identity-growth-experiment-rows-to-the-two-catalogs`](../../../tickets/add-the-identity-growth-experiment-rows-to-the-two-catalogs.md), which carries the exact text of each and the reconciliation count that closes it.
