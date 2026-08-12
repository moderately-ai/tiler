---
id: earn-cpu-feature-level-execution-environments-from-host-observation
title: Earn CPU feature-level execution environments from host observation
status: awaiting-decision
priority: p1
dependencies: []
related: [declare-cpu-vector-realization-facts-in-the-target-profile, name-a-host-process-availability-phase, promote-the-bounded-scalar-cpu-vertical-into-a-production-backend]
scopes: [implementation/runtime, implementation/cpu, contracts/foundation, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, runtime, eligibility, preflight, fail-closed]
---
## User-visible outcome

A CPU vector variant is eligible only after the runtime has observed the exact ISA feature level it claims, rather than trusting a caller-restated artifact profile or architecture name.

## Fact — 2026-08-11

Current variant eligibility compares a caller-stated `ExecutionEnvironment`; it performs no CPUID/HWCAP discovery. The scalar prototype's profile can be derived from the compilation itself, which is not independent host evidence.

## Required delivery

- Define a backend-owned, typed CPUID/HWCAP or equivalent observation that earns one exact feature-level execution environment.
- Require callers to choose the CPU approach explicitly; no architecture-only preset, compilation-profile echo, feature superset inference, or silent scalar fallback.
- Keep host eligibility distinct from compile-profile feasibility. Unknown/unowned probes refuse before routing commit with a typed reason.
- Bind observation environment, process/OS assumptions, provider revision, and resulting profile identity canonically where the accepted eligibility contract requires them.
- Perturb each feature bit/row, observation authority, cross-compile host, profile key/descriptor, and missing probe independently.

## Closes when

An artifact cannot make itself eligible by restating its own CPU feature profile and every accepted feature-level environment is earned from the bound host.

## Source-first audit — 2026-08-12

- **Verified:** device-free eligibility compares only the caller-stated `ExecutionEnvironment`. `DecodedProgram::variant_eligibility` has no adapter or host probe, and `ExecutionEnvironment` explicitly calls itself a host declaration rather than discovered truth.
- **Verified:** the scalar spike constructs its `TargetProfileRef` from the same `Compilation` that produced the artifact. Its real architecture, layout, and floating-point probes run later in `prepare_route`, so its profile classification is not independent host evidence.
- **Imprecise:** the host must satisfy every requirement of the explicitly selected CPU approach; it need not have an exactly equal set of feature bits. A newer CPU may execute an older selected approach when every required predicate is observed true. What is forbidden is inferring or silently selecting another approach from a feature superset.
- **Imprecise:** CPUID/HWCAP cannot derive a complete `TargetProfileRef`. The descriptor also commits target-profile facts, numerical authority, resources, and provenance. A CPU adapter must select a governed execution contract independently of the artifact, prove the host satisfies it, and only then report the contract's exact profile reference.
- **False for mutable state:** one observation at adapter binding cannot permanently earn floating-point behaviour. The scalar spike says the process can change floating-point state after its probe and remeasures per run. Thread-local or otherwise mutable predicates require an execution-thread guarantee and final precommit revalidation or scoped establishment; static ISA/layout facts do not.
- **Verified:** `RuntimeAdapter::bind_execution_context` is the only loader-sequenced source of `LiveExecutionContext`. The loader then performs profile/backend/representation comparison, resolves live route requirements, prepares entries, resolves prepared properties, calls the adapter's final reversible `plan_dispatch`, commits once, allocates, and dispatches. This gives a real precommit point for volatile CPU revalidation without changing the device-free loader.
- **Verified:** ADR 0110's accepted real consumer is `tiler-cpu-runtime`, parallel to the Metal runtime role. It depends on shared `tiler-cpu-image`, not on compiler/build/KIR. No mock, fake device, Candle adapter, or test interpreter is a consumer of this decision.
- **Not fired by this design:** `AvailabilityPhase` classifies artifact/target facts. Earning the adapter's complete `ExecutionEnvironment` is an eligibility operation before those fact queries, not a new ABI fact row. Therefore this decision neither adds `HostProcessPreflight` nor broadens `LiveDevicePreflight`; the deferred phase ticket remains related and fires only when a real carried predicate needs a separately observable host-process phase.

## Decision packet — 2026-08-12

### Recommended exact boundary

Use one governed CPU execution contract shared by the accepted producer/runtime split, and keep observation private to the concrete CPU runtime adapter:

1. `tiler-cpu-image` owns a non-forgeable, versioned `CpuExecutionApproach` vocabulary and each approach's immutable `CpuExecutionContract`. The first value is the accepted scalar F32 image approach; future native fixed-vector or scalable-vector approaches are new explicit values, never implicit upgrades. A non-exhaustive public enum is appropriate for caller selection, while all contract derivation and probe dispatch remain exhaustive inside the owning crate.
2. Each contract binds one exact target-profile reference, backend key, representation key, dispatched dtype set, platform/architecture/data-layout requirements, immutable ISA predicates, mutable numerical-environment predicates, and a contract revision. `tiler-cpu` must prove that the target profile it declares yields that exact reference. `tiler-cpu-runtime` consumes the same contract without depending on compiler code. The cross-package profile pin is checked correspondence, not a second caller-authored profile authority.
3. Constructing `CpuRuntimeAdapter` requires an explicit `CpuExecutionApproach` and the already-accepted explicit resource policy. There is no auto-detect constructor, architecture preset, highest-feature search, or default scalar fallback. Unsupported approaches are unconstructible or produce a typed construction refusal before routing.
4. `bind_execution_context` observes all static requirements from the actual executing host. Only after they pass does it return the contract's independently held `ExecutionEnvironment`; it never reads that environment from the artifact or accepts a caller-provided profile reference. Missing, unrecognized, or contradictory probes produce typed context refusals.
5. Mutable numerical predicates are checked on the exact execution thread. The adapter either owns a dedicated thread/session whose floating-point controls it establishes and restores, or revalidates them during `plan_dispatch` and guarantees no external callback or thread migration of the execution context before arithmetic. The first production adapter must choose and test one of those concrete mechanisms; a bind-time-only promise is rejected.
6. The loader still owns every comparison. The CPU adapter reports the environment and live observations; it does not decide whether an artifact matches. Backend-scoped route requirements remain available for genuinely route-specific CPU predicates, but ISA features already defining the selected complete target profile are not duplicated as per-entry rows.

The shared approach vocabulary is a real CPU backend contract, not a generic core feature enum. External CPU implementations remain independently selected runtime adapters under ADR 0090 and own their own governed contracts rather than extending Tiler's built-in enum with arbitrary strings or feature maps.

### Identity and version consequences

The selected target-profile reference, backend key, representation key, and any genuinely route-specific requirement already enter artifact identity through existing fields. The observed host values are ephemeral runtime evidence and must not enter artifact, cache, or semantic identity. Adding a built-in CPU approach adds new profile/artifact values but does not move existing Metal or CPU bytes and does not require a neutral artifact schema/domain step. The shared contract revision is part of the approach/profile correspondence and diagnostic evidence; it is not an adapter identity added to the artifact.

If the exact target-profile descriptor changes, producer and runtime correspondence pins move together and old artifacts fail profile classification. No compatibility alias, descriptor fallback, or artifact-derived reconstruction is retained in this pre-production tree.

### Failure evidence

The implementation must perturb independently: each required ISA predicate; OS, architecture, pointer width, and endianness; target-profile key and descriptor; backend and representation; contract revision/correspondence; missing and unrecognized probes; static-pass/volatile-fail; probe on one thread followed by execution on another; floating-point state changed after bind; extra unrelated host features; and an artifact attempting to restate its own profile. Extra features must leave the explicitly selected lesser approach eligible, while a missing required feature must refuse it.

### Ranked alternatives

1. **Shared governed approach contract plus static bind and volatile execution-thread qualification — recommended.** Correctness and fail-closed behaviour are strongest; producer/runtime correspondence has one owner; ordinary runtime work is one small fixed probe set plus final volatile checks; growth is explicit and backend-local.
2. **Put every host requirement into backend-scoped artifact route rows.** It can be correct only if the rows exhaust every host-sensitive target-profile fact. That duplicates the profile, makes omission a producer defect the neutral artifact layer cannot detect, and still leaves initial profile classification caller-stated. It is useful for route-specific facts, not as the primary eligibility authority.
3. **Let the adapter accept a caller or artifact profile and probe only feature bits.** Locally simple but unsound: the probe does not authenticate the profile's numerical, layout, resource, or provenance claims, and artifact restatement remains tautological.
4. **Auto-select the highest detected ISA or silently retry scalar.** Rejected. It changes approach, representation, identity, and failure timing without caller authority; explicit precommit fallback between complete routes remains the only allowed substitution.
5. **Architecture-name presets, one bind-time numerical probe, environment-variable overrides, or compile-host inference.** Rejected as aliases for unearned or stale execution evidence.

### Strongest counterpoint and reversal evidence

The shared exact profile pin couples runtime releases to target-profile descriptor changes. That is deliberate correspondence, but it carries maintenance cost. Replace it with a more general observed-host qualification protocol only if a bounded implementation proves that the runtime can validate every host-sensitive fact used by feasibility without reimplementing target-profile semantics or permitting omitted requirements. No such complete projection exists today.
