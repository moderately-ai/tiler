---
id: shape-the-conformance-corpus-for-target-multiplication
title: Shape the conformance corpus for target multiplication
status: deferred
priority: p2
dependencies: []
related: [survey-what-belongs-in-the-conformance-crate, admit-the-conformance-crate-to-the-workspace, publish-the-backend-provider-conformance-suite]
scopes: [research/verification]
shared_scopes: [project/tickets]
paths: []
tags: [research, conformance, target-profiles, architecture]
---
## Question

The conformance matrix is `operation family x dtype x contract x target profile x shape class`. Hand-written per-combination tests do not survive that multiplication. What shape does?

Filed 2026-08-07 by [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md), which answered the direction and parked the build.

## Why it is not urgent yet, and why it will not stay that way

**Fact — the corpus is single-family today.** Every profile key `crates/tiler-build/src/metal_declaration.rs` mints names the same target family: `tiler.metal.first-macos-apple9-msl4.measured.v1`, `...normative.v1`, `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` and `.v2`, plus the two offline-toolchain sub-keys. Those are authority variants and versions of one row, not a second family. Every executed comparison in the tree is bounded to one host row — Apple M4 Max, macOS 27.0 build `26A5388g`, Xcode 26.6, SDK 26.5, offline compiler `Apple metal version 32023.883`.

**Fact — the deferred work already names four more axis values.** An iOS profile row (no such row exists; `docs/dtype-support.md` records BF16's measurement as "macOS Apple9 profile rows only" and the retained Apple record's iOS-Simulator row as a separate measurement), a CPU vector tier (`docs/research/target-profiles/cpu-vector-realization-facts.md`), a subgroup execution tier (`docs/research/scheduling/subgroup-execution-tier.md`), and CUDA (`ticketsplease.toml` declares a `research/cuda-transfers` scope whose `docs/research/cuda-transfers/` directory does not yet exist). Each multiplies the matrix rather than adding to it.

**Fact — the existing hand-written shape is already at its limit at one target.** `crates/tiler-reference/tests/contraction_profile_cells.rs` carries a `const PROFILE_CELLS: [Cell; 6]` with a transcribed digest per cell for one host row, and `crates/tiler-compiler/src/governed/contraction_conformance.rs` carries a second, partly overlapping transcription of the same six cells with its own `ADMITTED_CELLS` / `REFUSED_CELLS` partition. Two transcriptions of one record, at one target, is the drift a second target makes structural.

## The direction the survey reached

**A conformance run should be a value, not a function.** The thing that survives multiplication is a declared *case* — operation set, dtype, contract key, shape class, and the environment row it is bounded to — driven by one executor, with the target profile supplied rather than baked in. The three parts, in the order they matter:

1. **The environment row is an operand, not a constant.** The unavailable-measurement report [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) item 1 requires is already a per-row decision. Making the row an input is what turns "this run does not apply here" from a `#[cfg]` into an outcome the run states. The six-field comparison in `publish-an-l3-contraction-cell-through-the-accepted-route` is the existing shape and it is already written against a record rather than against constants.
2. **The oracle stays singular.** The matrix multiplies targets, not semantics: `tiler-reference` answers the same question for every profile, and a case's expectation is the oracle's answer under the *declared contract*, never a per-target expectation table. A per-target expected-value table would be a second semantic authority, which is the crate's first anti-goal. What varies per target is the declared numerical realization applied to the oracle before comparison — the machinery `ReferenceNumericalConformance::from_realization` already exposes and which currently has no caller anywhere in `crates/` (see `docs/correctness-and-testing.md`, "What no capability yet checks").
3. **Refusal is a case outcome, not a missing test.** Most of the matrix is empty and stays empty. A profile that does not declare a dtype must produce a *named* refusal the run records, so an unimplemented cell is reported by the corpus rather than by its absence from it. `crates/tiler-compiler/tests/bf16_numerical_contract.rs` is the precedent one layer down: it asserts the whole refusal — subject, requirement, disposition, declared means, honoured behaviour — rather than its variant.

**What the survey does not recommend**: a combinatorial generator over the five axes. The population is sparse and non-monotone by design (`docs/dtype-support.md` says so outright), so enumeration would produce mostly refusals and would put the burden of knowing which combinations are meaningful into the harness instead of into the declaration.

## What a build would have to decide

The case declaration's shape is a public boundary under ADR 0075 even if it stays `pub(crate)` at first, and [`publish-the-backend-provider-conformance-suite`](publish-the-backend-provider-conformance-suite.md) is the ticket that would consume it from outside. Whether these are one surface or two is the first question, not a detail.

## Trigger

**Fires when a target profile outside the macOS Apple9 Metal family is admitted with measured rows** — an iOS profile row, a CPU vector tier row, a subgroup tier row, or a CUDA profile — or when a second dtype family reaches an executed cross-layer run. Either makes the axis real; until then the corpus has one family and a parameterization would be a generalization over a population of one.

Reproduce the check:

```sh
grep -rho 'tiler\.metal\.[a-z0-9.-]*' crates/tiler-build/src | sort -u
```

Every line it prints today contains `macos-apple9`, or is one of the two `tiler.metal.offline-*` toolchain sub-keys. A line naming any other family is the trigger firing.

## Trigger check log

- 2026-08-07 — **not fired.** Six keys, all macOS Apple9 Metal or offline-toolchain sub-keys of it. `crates/tiler-conformance/src/lib.rs` holds no items, so there is no executed run at any profile inside the crate.
