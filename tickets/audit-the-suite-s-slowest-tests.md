---
id: audit-the-suite-s-slowest-tests
title: Audit why five tests dominate the suite's wall clock
status: todo
priority: p2
dependencies: []
related: []
scopes: [research/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [testing, performance, research]
---
Audit only. This ticket does **not** change any test or production code; its deliverable is an explanation plus the follow-up tickets that would act on it. Filing the follow-ups is what closes it.

## Measurement

**Measurement — Apple M4 Max, 14 cores, `cargo nextest run --workspace --locked`, dev profile, warm build, 962 tests.** Reproduce the per-test timings with the `timing` profile kept in `.config/nextest.toml`, which lowers `slow-timeout` to 1s and reports every test that crosses it:

```sh
cargo nextest run --workspace --locked --profile timing
```

Five tests exceeded one second. Wall clock for the whole run was 13.09s.

| Test | Time |
| --- | --- |
| `tiler-artifact` `program::codec::tests::single_byte_corruptions_are_rejected` | 13.025s |
| `tiler-ir::typed_handles` `typed_authoring_contract` | 2.872s |
| `tiler-ir::shape_evidence_ui` `shape_evidence_contract` | 1.857s |
| `tiler-metal-aot` `driver::tests::the_integer_nan_predicate_compiles_under_every_realization` | 1.256s |
| `tiler-ir` `semantic::program::tests::semantic_graph_identity_handles_a_deep_chain_iteratively` | 1.183s |

**Inference — one test is the entire critical path.** The slowest test takes 13.025s and the suite takes 13.09s, so every other test in the workspace completes underneath it and the remaining 961 contribute essentially nothing to wall clock. The suite is not broadly slow; it is one test long. Any change to the other four is invisible until that one moves.

## What is already known, and what is not

**Fact — the shape of the dominant test.** `crates/tiler-artifact/src/program/codec/tests.rs:508` sweeps byte offsets across an encoded artifact, and for each one flips a byte, calls `decode`, asserts a rejection, and restores. It already samples rather than sweeping exhaustively: the manifest interior is visited with `.step_by(61)` while the header and the post-manifest remainder are visited byte by byte.

**Inference — the cost is a full validating decode per offset, not the flipping.** `decode_artifact` verifies framing, manifest and section digests, component schemas, canonical order, arena closure, and re-derives the artifact identity. That work is repeated once per swept offset over the whole artifact.

**Unknown, and the point of the audit.** Which of those decode phases dominates; whether the offsets outside the manifest are being visited at a density that buys anything the sampled interior does not; whether the fixture is larger than the property needs; and whether an equivalent guarantee is reachable at a fraction of the cost. None of that is measured yet, and the answer decides whether the follow-up is "shrink the fixture", "sample differently", "make the digest cheaper in dev", or "leave it alone".

**Fact — three of the five are `trybuild` compile tests.** `typed_authoring_contract`, `shape_evidence_contract`, and `index_region_ui` each construct a `trybuild::TestCases`, which invokes `rustc` once per case. Their cost is compilation, not the code under test, so they are a different problem from the codec sweep and may well have no worthwhile fix. Say so explicitly if that is the conclusion.

## Scope

For each of the five, determine where the time actually goes — by measurement, not by reading and reasoning about the code. Then decide, per test, which of these it is:

- a fixture or sweep density larger than the property requires;
- a genuinely expensive property whose cost is the price of the guarantee;
- an accident of the dev profile that a per-package `opt-level` override would fix; or
- inherent compile cost, as `trybuild` cases are.

**Do not weaken a correctness property to make a test faster.** `single_byte_corruptions_are_rejected` and its siblings are the evidence that the codec fails closed on damage; a cheaper test that checks less is a regression wearing a speedup's clothes. If the only way to cut the time is to check less, the finding is "leave it alone" and that is a valid outcome to record.

## Closes when

Each of the five tests has a measured explanation of its cost, and every one that admits a worthwhile improvement has a follow-up ticket filed naming the specific change and the expected saving. Tests that should stay as they are are recorded as such, with the reason, so the next person reading the timing table does not re-open the question. No test or production code changes on this ticket.
