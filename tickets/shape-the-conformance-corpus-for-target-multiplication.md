---
id: shape-the-conformance-corpus-for-target-multiplication
title: Shape the conformance corpus for target multiplication
status: awaiting-decision
priority: p2
dependencies: [conform-the-bf16-vertical-end-to-end]
related: [survey-what-belongs-in-the-conformance-crate, admit-the-conformance-crate-to-the-workspace, publish-the-backend-provider-conformance-suite]
scopes: [research/verification, implementation/conformance, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, conformance, target-profiles, architecture, decision, needs-tom, public-boundary, trigger-fired]
---
## Question

The conformance matrix is `operation family x dtype x contract x target profile x shape class`. Hand-written per-combination tests do not survive that multiplication. What shape does?

Filed 2026-08-07 by [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md), which answered the direction and parked the build.

## Why it is not urgent yet, and why it will not stay that way

**Fact — the target-family axis is still singular, but the dtype-family axis is not.** Production Metal compile declaration identities in `crates/tiler-build/src/metal_declaration.rs` all name the same target family, but they are not peer "profile keys." The sole production `TargetProfileKey` is `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` (minted by `BoundMetalCompileDeclaration::first_macos_apple9`). `tiler.metal.first-macos-apple9-msl4.measured.v1` and `...normative.v1` are measured/normative *producer* fact-source identities, not profile keys. Offline strings are producer-defined *role* identities (`tiler.metal.offline-toolchain-distribution`, `tiler.metal.offline-platform-sdk`). The `.v2` form is a test-only rekey of the same row for key/descriptor mutation tests, not a second published declaration. Those are authority variants, roles, and a test rekey of one family — not a second target family. Since this ticket was filed, however, [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) has executed a pure-BF16 multiply/add vertical on the measured Apple9 row alongside the existing F32 evidence. The corpus therefore has two executed dtype families even though it still has one measured target family.

**Fact — the deferred work already names four more axis values.** An iOS profile row (no such row exists; `docs/dtype-support.md` records BF16's measurement as "macOS Apple9 profile rows only" and the retained Apple record's iOS-Simulator row as a separate measurement), a CPU vector tier (`docs/research/target-profiles/cpu-vector-realization-facts.md`), a subgroup execution tier (`docs/research/scheduling/subgroup-execution-tier.md`), and CUDA (`ticketsplease.toml` declares a `research/cuda-transfers` scope whose `docs/research/cuda-transfers/` directory does not yet exist). Each multiplies the matrix rather than adding to it.

**Fact — the existing hand-written shape is already at its limit at one target.** `crates/tiler-reference/tests/contraction_profile_cells.rs` carries a `const PROFILE_CELLS: [Cell; 6]` with a transcribed digest per cell for one host row, and `crates/tiler-compiler/src/governed/contraction_conformance.rs` carries a second, partly overlapping transcription of the same six cells with its own `ADMITTED_CELLS` / `REFUSED_CELLS` partition. Two transcriptions of one record, at one target, is the drift a second target makes structural.

## The direction the survey reached

**A conformance run should be a value, not a function.** The thing that survives multiplication is a declared *case* — operation set, dtype, contract key, shape class, and the environment row it is bounded to — driven by one executor, with the target profile supplied rather than baked in. The three parts, in the order they matter:

1. **The environment row is an operand, not a constant.** The unavailable-measurement report [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) item 1 requires is already a per-row decision. Making the row an input is what turns "this run does not apply here" from a `#[cfg]` into an outcome the run states. The six-field comparison in `publish-an-l3-contraction-cell-through-the-accepted-route` is the existing shape and it is already written against a record rather than against constants; live `crates/tiler-conformance/src/retained_record.rs` still compares six fields (device, gpu-family, architecture, os, offline-compiler, sdk) with `xcode` present in the record and deliberately not compared.
2. **The oracle stays singular.** The matrix multiplies targets, not semantics: `tiler-reference` answers the same question for every profile, and a case's expectation is the oracle's answer under the *declared contract*, never a per-target expectation table. A per-target expected-value table would be a second semantic authority, which is the crate's first anti-goal. What varies per target is the declared numerical realization applied to the oracle before comparison — the machinery `ReferenceNumericalConformance::from_realization` already exposes. That bridge has production callers (`tiler-conformance` `bf16_vertical::conformance_of` and the publication proof route) plus `tiler-reference` unit tests; the residual gap is the `Unstated` subject population from `strict()`/`new()`, not absence of the bridge (see `docs/correctness-and-testing.md` Semantic authority paragraphs).
3. **Refusal is a case outcome, not a missing test.** Most of the matrix is empty and stays empty. A profile that does not declare a dtype must produce a *named* refusal the run records, so an unimplemented cell is reported by the corpus rather than by its absence from it. `crates/tiler-compiler/tests/bf16_numerical_contract.rs` is the precedent one layer down: it asserts the whole refusal — subject, requirement, disposition, declared means, honoured behaviour — rather than its variant.

**Correction — 2026-08-10.** Direction item 2 previously claimed in present tense that `ReferenceNumericalConformance::from_realization` "currently has no caller anywhere in `crates/`" and pointed at a non-existent `docs/correctness-and-testing.md` heading "What no capability yet checks." That no-caller claim is false after `give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject` and `route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge`; the contract's Semantic authority paragraphs state the retired clauses are now false and that what survives is the narrower `Unstated` population. Item 2 above carries the current contract. The singular-oracle / anti-second-authority design point is unchanged.

**What the survey does not recommend**: a combinatorial generator over the five axes. The population is sparse and non-monotone by design (`docs/dtype-support.md` says so outright), so enumeration would produce mostly refusals and would put the burden of knowing which combinations are meaningful into the harness instead of into the declaration.

## Decision Tom needs to make

The case declaration's shape is a consequential conformance boundary under ADR 0075 even if its first implementation stays `pub(crate)`, and [`publish-the-backend-provider-conformance-suite`](publish-the-backend-provider-conformance-suite.md) is the ticket that would consume a validated suite from outside. The trigger has fired, so the first question can no longer be deferred: whether these are one surface or two.

**Option A — one validated case model, internal first (recommended).** Define one target-neutral case declaration and executor boundary inside `tiler-conformance`, keep it non-public while the F32/BF16 population proves validation and refusal semantics, then publish the same validated representation through the provider suite when that ticket fires. This minimizes duplicate authorities and makes the later public boundary evidence-driven, but it commits the internal model to being the candidate external representation.

**Option B — separate internal execution cases and provider-facing declarations.** Keep a narrow private executor input optimized for the current test crate, and later translate provider declarations into it through an explicit validation seam. This avoids prematurely shaping the public representation around two current dtype families, but creates two representations and a translation whose identity, validation, and drift controls must be owned.

**Recommendation.** Accept Option A's one validated model with an explicitly non-public first landing. It preserves one semantic authority and lets the existing F32/BF16 cases expose mistakes before any external surface is accepted. Tom's decision here authorizes only that boundary direction; exact public names and visibility remain separately reviewable when the provider-suite trigger fires.

## Trigger

**Fires when a target profile outside the macOS Apple9 Metal family is admitted with measured rows** — an iOS profile row, a CPU vector tier row, a subgroup tier row, or a CUDA profile — or when a second dtype family reaches an executed cross-layer run. Either makes the axis real; until then the corpus has one family and a parameterization would be a generalization over a population of one.

Reproduce the check:

```sh
grep -rho 'tiler\.metal\.[a-z0-9.-]*' crates/tiler-build/src | sort -u
```

Every line it prints today contains `macos-apple9`, or is one of the two `tiler.metal.offline-*` toolchain sub-keys. A line naming any other family is the trigger firing.

## Trigger check log

- 2026-08-07 — **not fired.** Six keys, all macOS Apple9 Metal or offline-toolchain sub-keys of it. `crates/tiler-conformance/src/lib.rs` holds no items, so there is no executed run at any profile inside the crate.
- 2026-08-09 — **fired by the second-dtype arm.** [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) is `done` and records a pure-BF16 multiply/add program dispatched and compared against the exact-rational oracle on the measured Apple9 row. The target-family axis remains one row, but F32 and BF16 now make the dtype axis real. Moved to `awaiting-decision` because the ticket itself identifies the one-versus-two case-declaration surface as its first consequential choice.
