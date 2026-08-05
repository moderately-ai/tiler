---
id: cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock
title: Cut the decoder-layer reference evaluation's suite wall clock
status: in-progress
priority: p2
dependencies: []
related: [assemble-the-decoder-layer-program, audit-the-suite-s-slowest-tests]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [testing, performance]
claimed_from: todo
assignee: agent-suite-clock
lease_expires_at: 1785950113
---
## The measurement, and why it is a ticket rather than a footnote

**Measurement — Apple M4 Max, `cargo nextest run -p tiler-reference --test decoder_layer --profile timing`, dev profile, warm build, 2026-08-05.** Eighteen tests, 71.0s wall clock. Six crossed one second:

| Test | Time |
| --- | --- |
| `the_layer_evaluates_end_to_end_at_the_c1_prefill_row` | 71.015s |
| `the_layer_evaluates_end_to_end_at_the_c1_decode_row` | 25.512s |
| `the_grouped_query_head_reading_is_semantic` | 22.555s |
| `the_mlp_gating_is_semantic` | 22.384s |
| `the_c1_decode_row_evaluates_under_the_default_work_bound` | 14.360s |
| `the_reference_work_bound_refuses_the_c1_prefill_row` | 6.008s |

**Inference — this is the workspace's new critical path, by an order of magnitude.** [`audit-the-suite-s-slowest-tests`](audit-the-suite-s-slowest-tests.md) left the previous dominant test, `single_byte_corruptions_are_rejected`, at 132 ms, and recorded that the suite "is not broadly slow; it is one test long". It is one test long again, and the test is now 71s.

**Fact — what one prefill evaluation costs.** The C1 prefill row walks 157,286,400 multiply-accumulate steps through the reference evaluator's certified element arithmetic and 30,720 certified `SiLU` evaluations, each of which builds an exact-rational enclosure of `exp`. The independent recomputation in the same test walks the same 157 M steps natively and repeats the 30,720 certified activations, because `tiler_reference::silu_f32` is the crate's authority and a second hand-rolled copy would only agree with itself.

**Fact — a constant that is paid six times.** `LayerFixture::new` materializes about 12.6 M `ReferenceElement` payloads per construction; `the_reference_work_bound_refuses_the_c1_prefill_row` spends 6.0s almost entirely there, since the evaluator refuses at the query projection before any MLP weight is read. Six tests construct a fixture, two of them twice.

## What is not on the table

**Do not reduce the C1 prefill row.** Its extents are the ticket's own outcome — the layer reference-evaluates at the pinned conformance row with nothing reduced — and a cheaper test at a narrower model dimension checks a different program. This is the shape `audit-the-suite-s-slowest-tests` recorded as "a cheaper test that checks less is a regression wearing a speedup's clothes".

## Candidates, unmeasured

1. **Measure first.** Attribute the 71s among evaluator contraction steps, certified `SiLU`, fixture construction, and the recomputation, by instrumentation rather than by the arithmetic above. Every candidate below is idle until that split exists; the arithmetic here is consistent with several different splits.
2. **Share one fixture per row across tests.** A `OnceLock`-held fixture per extents row would pay the ~6s construction once instead of eight times — but only if the tests can share immutable state without a mutable path between concurrently running processes, which nextest's process-per-test model already gives for free within one binary only if the tests are in one process. Check before assuming.
3. **A memoized certified exponential.** The SiLU reference's cost is per distinct argument; the fixture's operands are drawn from a generator with a bounded mantissa, so the distinct-argument count may be far below 30,720. Measure the ratio before building anything.
4. **Leave it.** If the cost is the evaluator's certified arithmetic walking the row's real step count, that is the price of the guarantee and the finding is to record it in the timing table so the next reader does not reopen it.

## Closes when

The 71s has a measured attribution, and each component is either improved with the saving stated or recorded as inherent with its reason — on the same terms `audit-the-suite-s-slowest-tests` set. No correctness property of `decoder_layer.rs` is weakened and the C1 prefill row's extents do not move.

## Outcome

**Measurement — the attribution, by instrumentation, Apple M4 Max, dev profile.** One `the_layer_evaluates_end_to_end_at_the_c1_prefill_row` splits as: program build 1.3 ms, `LayerFixture::new` 175 ms, `evaluate_layer` 25.5–29.9 s, `recompute_layer` 26.1 s. Inside the recomputation, `silu_f32` over the 30,720 gate elements is 66.3 s of a 66.6 s call in the instrumented build — 99.6 % — while all seven `project` folds together, the whole 157,286,400 multiply-accumulate steps the ticket's arithmetic is about, are 140 ms.

**Fact — the ticket's own two attributions above are both refuted.** `LayerFixture::new` is 175 ms, not seconds: the "constant paid six times" is a fifth of a second, and `the_reference_work_bound_refuses_the_c1_prefill_row` does not spend its time there. And the 157 M multiply-accumulate steps are not the cost of the recomputation; the 30,720 certified exponentials are. The step count was the wrong number to reason from.

**Measurement — where the certified exponential spends itself.** A `/usr/bin/sample` stack profile of the prefill test puts about 80 % of non-idle samples in `num_bigint`'s Stein binary greatest-common-divisor (`biguint_shr`, `gcd`, `sub_assign`, `trailing_zeros`), reached from `ExactRational::normalize`, which every exact-rational operation calls. Timing the phases of `exp_enclosure` in the same process: series loop 79 % (its `add` 33 %, its `multiply`-then-`divide` 46 %), tail bound 4 %, squaring chain 17 %. The series' terms carry `2^(m*i) * i!` denominators, so the magnitudes — and with them a quadratic gcd — grow with every term.

**Fact — candidate 3 is eliminated by measurement rather than by argument.** Of the 30,720 SiLU arguments at the C1 prefill row, 30,710 are distinct. They are contraction results over a 1,024-wide fold, not fixture draws, so a memo would hit ten times in thirty thousand. Candidate 2 is moot: the fixture is 175 ms.

**Fact — one attempt was measured and discarded.** Rounding the series' partial sum and term outward onto a working grid each iteration — which the module's own prose claims it already does — produced byte-identical enclosures at all 1,200 probed arguments and ran 1.5× *slower* (2,157 µs against 1,445 µs per call, interleaved): it bounds the magnitudes but quadruples the operation count, and the per-operation saving does not cover it.

**The change, in `crates/tiler-reference/src/accuracy.rs`.** Three parts. The argument reduction drives the reduced argument to `2^-8` rather than `2^-1` (`REDUCED_ARGUMENT_BITS`), trading series terms — whose cost grows superlinearly — for squarings, whose operands are already grid-bounded; the depth is the measured optimum of a 1/4/8/12/16/20 sweep and `MAX_ARGUMENT_HALVINGS` is restated as `23 + REDUCED_ARGUMENT_BITS` so the admitted argument domain does not move with it. `CertifiedEnclosure::multiply` takes the two ordered corners directly when both brackets are non-negative, which is every squaring, instead of computing four products and six cross-multiplied comparisons to rediscover an order it has. The squaring chain rounds onto a grid `REDUCED_ARGUMENT_BITS + 2` finer than the caller's and rounds to the caller's grid once at the end, because squaring amplifies each intermediate rounding by every squaring that follows it and the deeper reduction would otherwise have degraded a coarse grid geometrically worse than before.

**Measurement — before and after, Apple M3 Pro, load average 2.0, `cargo nextest run -p tiler-reference --profile timing`, warm build, three interleaved rounds per test binary so drift reaches both alike.**

| Test | Before | After | Ratio |
| --- | --- | --- | --- |
| `the_layer_evaluates_end_to_end_at_the_c1_prefill_row` | 42.90 / 41.13 / 42.82 s | 24.46 / 26.26 / 25.68 s | 1.67× |
| `the_layer_evaluates_end_to_end_at_the_c1_decode_row` | 7.42 / 7.38 / 7.38 s | 4.65 / 4.68 / 4.62 s | 1.59× |
| `the_mlp_gating_is_semantic` | 5.71 / 5.59 / 5.55 s | 3.77 / 3.78 / 3.74 s | 1.49× |
| `the_grouped_query_head_reading_is_semantic` | 5.72 / 5.60 / 5.58 s | 3.79 / 3.78 / 3.85 s | 1.48× |
| `the_c1_decode_row_evaluates_under_the_default_work_bound` | 3.12 / 2.89 / 2.90 s | 2.00 / 1.99 / 1.98 s | 1.47× |
| `the_reference_work_bound_refuses_the_c1_prefill_row` | 1.71 / 1.51 / 1.50 s | 1.52 / 1.52 / 1.51 s | 1.00× |
| `decoder_layer` binary wall clock | 42.93 / 41.15 / 42.84 s | 24.49 / 26.28 / 25.71 s | 1.65× |

The whole package moves with it, two interleaved rounds on the same host: 42.64 / 42.07 s to 26.23 / 26.03 s, 272 tests passing in every run. `silu::tests::the_band_produces_no_subnormal_result`, the other certified-exponential test, goes 6.28 / 6.20 s to 3.23 / 3.17 s — 1.95×. `the_reference_work_bound_refuses_the_c1_prefill_row` is unmoved because it refuses at the query projection and evaluates no activation, which is the control the table needed.

**Fact — the oracle is byte-identical.** `certified_exp_f32` and `silu_f32` over 8,000 arguments spanning `[-104, 89]` digest to `0x2f1bd9460d7a73a1` before and after; the prefill row's three outputs digest to `0xa848fe08a6ff0d3c` and the decode row's to `0xf702a6e848b25f4e`, both unchanged. The enclosure *endpoints* do move — they must, since the squaring count changed — and at all 1,200 interleaved probe arguments the new enclosure contains the old one, which is the safe direction.

**Two existing perturbation tests moved with the reduction depth, and neither was loosened.** `a_precision_the_series_cannot_reach_is_refused` asked for a 5,000-bit grid because 512 terms of a `y <= 1/2` series could not reach it; at `y <= 2^-8` they can, so the test now states the grid with its derivation — `T_512 <= 2^-(512k + log2(512!))`, about `2^-8483` — asks for 12,000 bits, and asserts the reachable neighbour at 4,000 so the refusal stays attributable to the grid. `a_degraded_enclosure_yields_undecided_rather_than_a_silent_pass` was what caught the squaring-grid defect: at two fraction bits the amplified upper end left binary32's range and the decision became `Undecided { MetricUndefined }` instead of `EnclosureTooWide`. That is the failure the third part of the change repairs, and the test is unedited.

`an_over_large_argument_is_refused` gains the boundary it was missing: `2^23` is the first refused binade and `2^10` an admitted neighbour, so a change that narrowed the domain fails there rather than passing by refusing more. Watched failing under a perturbed `MAX_ARGUMENT_HALVINGS`.

**Recorded as inherent, with the reason.** The prefill test is still 25.7 s and still the package's critical path. What remains is the same certified exponential — 61,440 evaluations per run, 30,720 in the evaluator and 30,720 in the independent recomputation, which is not redundancy but the crate's authority being the only admissible arithmetic for both — plus the evaluator's 157 M certified multiply-accumulate steps, which the sample put at about 10 % of the test. Reducing the exponential further means attacking `ExactRational::normalize`'s greatest-common-divisor on every operation, which lives in `crates/tiler-ir` and is outside this ticket's scope; filed as [`bound-the-exact-rational-gcd-cost-in-certified-enclosures`](bound-the-exact-rational-gcd-cost-in-certified-enclosures.md).

**One finding out of scope, filed rather than absorbed.** Probing whether the admitted-argument domain had moved measured `exp_enclosure` at `2^16` costing 157 ms, `2^20` costing 37.5 s and `2^22` not completing in nine minutes: `MAX_ARGUMENT_HALVINGS` bounds the halving count and not the magnitude the halvings then square up to, so the module admits arguments it cannot cost, which is the one bound in this crate that does not fail closed. Pre-existing, unreachable through `certified_exp_f32`'s own `+89`/`-104` guards, and filed as [`bound-the-certified-exponential-s-cost-in-its-admitted-argument-region`](bound-the-certified-exponential-s-cost-in-its-admitted-argument-region.md).

**Measurement boundary.** All ratios are dev profile, warm build, on the two named hosts; the M4 Max development host carried load averages between 13 and 61 from concurrent agents throughout, so its absolute numbers are not comparable to the M3 Pro table and only the interleaved in-process ratios from it are quoted. Nothing here measures release profile. The M3 Pro figures were taken in a scratch copy of this branch's tree under `/tmp` on that host, toggled between the two states with the same patch, so they measure this diff and not a rebuild difference.
