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
