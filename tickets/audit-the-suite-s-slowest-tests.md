---
id: audit-the-suite-s-slowest-tests
title: Audit why five tests dominate the suite's wall clock
status: done
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

## Outcome

Done. All five explained by measurement; one has real improvements available and two follow-ups are filed. No test or production code changed.

**Both hypotheses this ticket offered are true of the dominant test, and they compound.** That was not the expected shape — the ticket read as though one category would apply.

### `single_byte_corruptions_are_rejected` — 13.025s, and it *is* the suite's critical path

**Sweep density.** *(Superseded 2026-07-27 — the sweep is now exhaustive over 15,030 bytes and runs in 132 ms; see `reduce-the-codec-corruption-sweep-to-its-distinct-classes`. The figures below describe the 26,126-byte envelope and the 662 µs decode that preceded the codec and ABI-identity work.)* The test visits 8,451 offsets: 69 header bytes one by one, 295 manifest samples at `.step_by(61)`, and 8,087 post-manifest bytes one by one. Bucketing every offset's refusal by region and variant gives thirteen distinct outcomes — and **8,075 offsets produce a single one of them**, `SectionDigestMismatch`, because the fixture has one section and every content byte is under that one digest. **8,370 of 8,451 offsets (99.0%) reproduce an outcome another offset already produced.**

The density is already inconsistent with itself: the manifest interior is sampled 1-in-61 and is *still* uniform across all 295 samples, while the larger section region is not sampled at all. The header is the opposite and must stay exhaustive — eight distinct classes in 69 bytes.

**Dev-profile optimization.** `Cargo.toml` sets `[profile.dev.package."*"] opt-level = 1`, and Cargo's `"*"` matches dependencies rather than workspace members, so every `tiler-*` crate compiles at `opt-level = 0` for `make check`. The same decode measured in both profiles: 962 µs dev against 182 µs release for a damaged envelope, 2.878 ms against 531 µs for a valid one — **5.3×**. The workload is digest-dominated, which is the shape that suffers most unoptimized, and the multiplier is paid by every digest, encode, and identity derivation in the suite rather than only here.

Filed: `reduce-the-codec-corruption-sweep-to-its-distinct-classes` and `raise-the-dev-opt-level-for-workspace-crates`. They attack the same 13s from opposite sides and compose — the first ticket says to re-measure after the second, because a 5.3× cheaper decode leaves an exhaustive sweep at ~1.5s, which may make keeping exhaustive byte coverage affordable and moot the question of whether sampling it would weaken the property.

### The other four — leave them alone, and here is why

- **`typed_authoring_contract` (2.872s)** and **`shape_evidence_contract` (1.857s)** — inherent compile cost. Seven `trybuild` cases each (one `pass`, six `fail`), so seven `rustc` invocations, at ~0.41s and ~0.27s per case. The cost is compilation, not the code under test, and there is nothing to make faster short of having fewer cases — which would be removing coverage, not waste.
- **`the_integer_nan_predicate_compiles_under_every_realization` (1.256s)** — a genuinely expensive property. Nine real `xcrun metal` compilations, ~0.14s each, which is fast for a real toolchain. The matrix *is* the assertion: it proves the numerical flags reach the compiler, and the test explicitly fails if every combination produced identical bytes.
- **`semantic_graph_identity_handles_a_deep_chain_iteratively` (1.183s)** — a genuinely expensive property. `DEPTH = 50_000` is what the test asserts: that identity derivation over a deep chain is iterative rather than recursive. Reducing the depth would weaken precisely the thing it exists to prove.

Recorded so the next person reading the timing table does not reopen these three.

**Method note.** The per-region refusal bucketing and the dev-versus-release comparison were taken with a temporary instrumented test in `codec/tests.rs`, which was removed; this outcome is the retained record. Reproduce the timing table with `cargo nextest run --workspace --locked --profile timing`, which reports every test over one second.
