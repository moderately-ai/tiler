---
id: shape-the-conformance-corpus-for-target-multiplication
title: Shape the conformance corpus for target multiplication
status: done
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

**Correction — 2026-08-12.** The deferred work does not name four peer values of one target-profile axis. An iOS profile would be a peer target family, but the CPU-vector and subgroup records describe realization capabilities that a profile may carry, and CUDA is currently only a reserved research scope. All four enlarge the evidence space, but flattening them into one axis would erase the distinction between target identity and a capability declared by that target.

**Fact — retained contraction-cell data is repeated, but the consumers do not make one claim.** `crates/tiler-reference/tests/contraction_profile_cells.rs` carries `PROFILE_CELLS`, `crates/tiler-compiler/src/governed/contraction_conformance.rs` carries the partly overlapping `ADMITTED_CELLS` / `REFUSED_CELLS` partition, and `crates/tiler-conformance/src/envelope.rs` carries `L3_CORRECTNESS_CELLS`. The conformance copy is checked against the retained record by `retained_record::direct_digests`; the reference and compiler copies exercise different work bounds and reachable populations. Shared fixture authority is warranted where their subjects overlap, but the repetition does not prove that every conformance run has one universal case shape.

## The direction the survey reached, and what the current source corrected

**A conformance run should be inspectable data, but not one flat product row.** Current source carries several independently meaningful subjects: a semantic stimulus, a verified target declaration, a host observation, a plan-derived numerical realization, an optional retained measurement, and an execution result. They must compose through one checked lifecycle without becoming fields that can contradict each other. The three original directions survive after narrowing:

1. **The environment row is run context, not case identity or a constant.** The unavailable-measurement report [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) item 1 requires is already a per-row decision. `measurement::MeasurementBoundary` and `Measured<T>` correctly keep the observed row beside the measured result, so one semantic case can be asked under several contexts without copying or contradicting the row inside the case. The six-field retained-record comparison remains a separate empirical-evidence claim: device and GPU-family differences decline that comparison, toolchain differences are announced and compared, and `xcode` is deliberately not observed.
2. **The oracle stays singular and consumes reached authority.** `tiler-reference` answers the semantic question for every profile. The requested numerical contract may be a case input to compilation, but the oracle conformance is derived from the selected packaged plan's exact realization through `ReferenceNumericalConformance::from_realization`; the case may not restate an expected target-specific value or an unchecked delivered realization. The residual `Unstated` population from `strict()` / `new()` remains a separate conformance-bridge boundary.
3. **Refusal is recorded, but it is not conflated with every other disposition.** A profile that does not declare a requirement produces a named feasibility refusal. A missing host/toolchain/device is `Measured::Unavailable`; a stage reached on an available environment and then failing is `Measured::Failed`; a completed execution that disagrees is a mismatch; and a retained empirical comparison may decline while the reference comparison still runs. A structured report must preserve those independent claims rather than reduce them to one catch-all outcome.

**Correction — 2026-08-10.** Direction item 2 previously claimed in present tense that `ReferenceNumericalConformance::from_realization` "currently has no caller anywhere in `crates/`" and pointed at a non-existent `docs/correctness-and-testing.md` heading "What no capability yet checks." That no-caller claim is false after `give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject` and `route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge`; the contract's Semantic authority paragraphs state the retired clauses are now false and that what survives is the narrower `Unstated` population. Item 2 above carries the current contract. The singular-oracle / anti-second-authority design point is unchanged.

**What the survey does not recommend**: a combinatorial generator over the five axes. The population is sparse and non-monotone by design (`docs/dtype-support.md` says so outright), so enumeration would produce mostly refusals and would put the burden of knowing which combinations are meaningful into the harness instead of into the declaration.

## Decision — accepted 2026-08-12

Tom accepted a revised third option after a source-first audit at `de8d0567e708a026e56403443f023233f1e5d885`: **one canonical conformance lifecycle protocol, algebraic family-specific case declarations, separately supplied run context, and structured evidence reports.** The decision rejects both a flat five-axis record and the promise that `tiler-conformance`'s private executor representation will later become the provider-facing API.

The accepted split is:

- **Case definition:** a bounded explicit suite of algebraic family variants. Each variant owns only meaningful fields: semantic recipe, stimulus provenance, requested contract where applicable, and explicit work authorization. Dtype and shape are derived from the verified program and operands rather than restated. `ProofFamily::Contraction` and `ProofFamily::L3CorrectnessCell` are the existing counterexample that requires provenance to remain identity-bearing even when operation and extents agree.
- **Run context:** one verified target declaration/profile and one observed execution environment supplied to a run, not cloned into every case. Applicability, target feasibility, and host availability stay distinct.
- **Oracle:** the singular reference authority under the selected plan's derived realization. No per-target expected-value table, strict/default fallback, or caller-stated delivered realization.
- **Report:** separately typed deterministic construction, feasibility, measurement availability, execution comparison, and retained-record applicability. An invalid declaration is a construction error rather than a case outcome; missing capability, absent environment, reached-stage failure, mismatch, and pass remain distinct.
- **Population:** only explicitly declared cases are visited. There is no Cartesian generator over operations, dtypes, contracts, profiles, and shape classes. Family generation is lazy and carries explicit count/payload/work bounds.
- **Visibility and future provider suite:** the first implementation stays `pub(crate)` / test-only under ADR 0106. `tiler-conformance` remains dependency-top and non-reusable. If a real provider-suite consumer later needs the same neutral specification or receipt subject, move only that shared subject to an accepted public owner. Private device fixtures and executor recipes do not become public merely because both workflows are called conformance.

This is MECE by authority: semantic case, target feasibility, observed environment, reference meaning, and execution/empirical evidence are mutually distinct and collectively cover the lifecycle. A value never occupies two authority roles, and no authority is inferred from a neighbour.

**Rejected alternatives, in order.** A carefully validated public-to-private projection remains possible only when those are genuinely different subjects; status-quo family modules remain correct but duplicate lifecycle rules; publishing the same private case representation prematurely violates the conformance crate's accepted ownership; and a flat product or Cartesian generator creates meaningless combinations and duplicated facts.

**Identity and performance.** The internal protocol is test-only and adds no artifact, cache, request, or canonical-domain version. If a suite or report is persisted later it needs its own explicit versioned identity decision. Ordinary work is proportional to the explicitly declared case population; target/environment context is shared once per run, and large operand generation remains lazy and bounded by the family-owned allowance.

**Acceptance provenance.** Accepted by Tom in the repository coordination conversation on 2026-08-12, relayed and recorded by Codex. The strongest counterargument was the additional type structure relative to one universal record; current source defeats it because flattening already-independent authorities creates optional fields and mismatch states rather than removing complexity.

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
