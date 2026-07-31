---
id: reconcile-the-empty-domain-proof-member-between-the-two-serial-sum-prototypes
title: Reconcile the empty-domain proof member between the two serial-sum prototypes
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, prototype, defect]
---
The two-process serial-sum proof fails on its **first** proof-matrix member. Reproduced at `1e062d9` (this ticket's base) and at `9127b6a`, byte-for-byte the same failure, so it is **not** caused by `state-an-expected-artifact-identity-from-recorded-bytes`, which only found it.

## Reproduction

```text
cargo run -q -p tiler-prototype-compile -- --out <dir>/serial-sum.tiler
cargo run -q -p tiler-prototype-run     -- --artifact <dir>/serial-sum.tiler
```

**Measurement** (macOS, Apple M4 Max, this repository's pinned toolchain, `1e062d9`): the single-member hardware path succeeds end to end — `nontrivial.selected` routes, clears every device-preflight refusal, commits, and reports bit-for-bit agreement on both the direct and envelope paths. The proof matrix then begins and the *first* member fails:

```text
the proof matrix, every published member against every operand case:
artifact: …/serial-sum.tiler.empty-domain.selected (27663 bytes), sidecar … (25765 bytes, 5 case(s))
serial-sum runtime proof failed: the artifact packages a kernel program of 2055 identity bytes
and this process compiled one of 2; the two prototypes have drifted
```

## What is actually wrong

**Fact.** `prove_member` (`prototypes/serial-sum-run/src/proof.rs`) requires the packaged kernel-program identity to equal *some* alternative the runner's own `compile_governed` derives for the same declared shape. For the `empty-domain` class — reduced extent `0`, the first entry of `REDUCTION_CLASSES` in both prototypes — no alternative matches, so `ProofError::ForeignProgram` is raised before any dispatch.

**Fact.** Both prototypes state the class matrix independently and each pins it in a test naming the other side, and those tests pass: the *names and extents* agree. What disagrees is the compiled program for the zero-extent case, which no test compares because nothing links the two crates.

**Inference.** Either the producer and the runner compile the zero-extent reduction differently (a plan-selection or degenerate-shape divergence), or one of them is not compiling the shape the other declares. Which of the two it is has not been established — that is the first thing this ticket should settle, by printing both identities for the failing member rather than only their lengths.

## A second, smaller defect in the same error

**Fact.** `ProofError::ForeignProgram { packaged, compiled }` fills `compiled` with `compilation.alternatives().count()` — a *count of alternatives* — while its `Display` renders it beside `packaged`, a *byte length*: "packages a kernel program of 2055 identity bytes and this process compiled one of 2". A reader reasonably parses `2` as two identity bytes. The message should either name the count as a count or carry the derived identities' lengths; as written it misdirects at exactly the moment somebody is diagnosing a drift.

## Why the gate is green over this

No `make` target runs either prototype binary. `make full` builds and unit-tests both packages, and their unit tests do not execute the matrix — it needs a Metal device and a produced envelope. So this failure is invisible to the gate and is found only by running the two commands above by hand. Deciding whether that stays true is part of this ticket: a proof whose *first* matrix member has been failing is weaker evidence than its retained description implies.

## Outcome

Establish which prototype is wrong for the zero-extent class and correct it, so the full six-member matrix runs; fix the `ForeignProgram` message; and record whether the matrix result should be reachable by something other than a hand run.

## Outcome (2026-07-31)

**The drift was the runner's, and it is already fixed.** `construct-and-bind-the-first-authoritative-metal-compile-profile` established the cause while migrating the prototypes: `prove_member` compiled the runner's own `ROWS = 4` against artifacts the producer publishes with one row — introduced by `0b7e59d` (2026-07-30) — so every packaged program was foreign, the empty-domain member merely being the first the matrix reached. The fix (commit `f81c7f2`) reads the shape from the artifact, as the deep proof already did. Reproduced on current main, Apple M4 Max: the full run passes — `30 case(s) proved across 6 member(s); fused and materialized agree bit for bit with the published reference`, with `empty-domain.selected: 5 case(s) agree` first among them. The producer was never wrong; nothing about the zero-extent compilation diverged.

**The second defect is fixed here.** `ProofError::ForeignProgram` rendered a count of alternatives beside a byte length ("compiled one of 2" reading as two bytes). It is now two variants carrying what they mean: `ForeignProgram { packaged, alternatives }` for the matrix path's none-of-N mismatch, and `ForeignRoutedProgram { routed, derived }` for the routed-identity mismatch, each with a message naming its own quantities.

**Gate reachability is deliberately not absorbed.** Whether the matrix should be reachable by something other than a hand run is exactly the question `pin-the-serial-sum-producer-runner-shape-interface` (filed by the profile ticket, with three candidate mechanisms and their trade-offs) owns; this ticket defers to it rather than duplicating the decision.
