---
id: construct-and-bind-the-first-authoritative-metal-compile-profile
title: Construct and bind the first authoritative Metal compile-time target profile
status: todo
priority: p0
dependencies: [express-metal-honourability-in-the-shared-form, admit-measured-compile-profile-sources-across-fact-families, measure-macos-apple9-f32-under-unified-msl4-profile, source-or-rephase-first-metal-launch-limits, separate-metal-launch-index-from-index-and-address-width, replace-or-justify-the-barrier-count-axis, validate-macos-metal-profile-host-applicability]
related: [admit-a-caller-declared-target-profile, carry-the-honourability-fact-provenance-into-the-artifact-record, decide-per-dtype-dispatchability-as-a-target-capability, record-metal-runtime-compiler-provenance-gap, prototype-metal-runtime-proof, measure-apple-numerics-on-physical-ios-device, spike-bf16-through-the-second-dtype-seams, redesign-the-delivered-realization-record-from-typed-evidence]
scopes: [implementation/build, implementation/compiler, implementation/metal, implementation/metal-aot, implementation/runtime, contracts/foundation, contracts/numerics, contracts/artifacts, contracts/navigation, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, target-profile, numerics, runtime, provenance]
---
## User-visible outcome

A production caller can select one named, versioned macOS Metal compile profile whose every compiler-visible quantitative, F32-dispatchability, and F32 numerical row has a reproducible authority and an exact validity, compiler-build, and execution-environment scope. The compiler and Metal emitter consume one checked bound declaration, and the runtime independently validates that the current host is eligible to offer that exact `TargetProfileRef` before routing. Unknown or unmeasured rows reject. Live-device and prepared-pipeline facts remain later preflight obligations rather than being promoted into the compile profile.

## Why this is a separate p0

**Fact:** `express-metal-honourability-in-the-shared-form` delivers a public, composable F32 projection from caller-stated `MetalTargetFacts` into the compiler's checked per-dimension form. Its module contract explicitly says that the provenance is caller-vouched, the independently supplied context cannot be proved to have produced the fact, the adapter does not populate quantitative or dispatchability facts, and it does not bind a profile, Metal record, plan, artifact, or runtime environment.

**Fact:** the serial-sum producer currently states convenient quantitative values such as grid, workgroup, binding, index, and local-memory bounds. Passing tests show those values are internally usable; they do not establish that each value is an authoritative target-family fact. The zero-synchronization schedule consumes no synchronization fact.

**Fact:** `tiler-runtime::ExecutionEnvironment` is deliberately device-free and compares an independently offered exact profile key and descriptor. The prototype runner currently derives that offer from a compiler `Compilation`, which proves equality with the producer's declaration but not that the live host satisfies it.

**Inference:** moving the bounded F32 projection directly into production would turn unsourced constants and an unvalidated runtime offer into target claims. The projection ticket must close on the ratified seam it actually implements; this ticket owns sourcing, binding, and production migration.

No existing ticket owns the complete outcome. The caller-profile ticket owns the generic vocabulary, the provenance ticket owns the structured source schema, the dispatchability ticket owns the placement of dtype routing facts, and the runtime proof established the one-way routing protocol. None constructs one authoritative Metal profile or independently validates its applicability.

## Work

### 1. Build an exact authority ledger

Enumerate every compile-phase fact the bounded serial-sum path consumes:

- each quantitative axis and its exact semantics;
- F32 dispatchability;
- F32 input- and result-subnormal behaviour;
- the profile key and version;
- each compiler build and execution environment in a measured source; and
- every Metal emission fact that overlaps a compiler fact.

For each row record its owner, source, authority, validity, availability phase, exact environment, and whether it is a **Fact**, **Measurement**, **Inference**, or still a **Proposal**. Prefer primary Apple specifications for normative limits and the retained Apple numerical harness for empirical rows. A row without sufficient evidence remains absent and therefore `Unknown`; do not fill it with a convenient constant, a nearby dtype, a different Apple family, or a live-device value.

The ledger must distinguish a conservative compile guarantee from a reported hardware maximum. Exact device maxima belong to `LiveDevicePreflight`; pipeline properties belong to `PreparedKernelPreflight`. If the serial sum needs a fact that cannot truthfully exist at `CompileProfile`, split a named measurement or contract ticket and leave the production migration blocked.

### 2. Construct one bound compile declaration

Define the smallest named `tiler-build` owner that can see the compiler and both Metal target vocabularies. It constructs:

- the checked compiler `TargetProfile`;
- the exact `MetalTargetFacts` used for emission;
- the total `MetalTargetFacts` to `tiler-metal-aot::MetalTarget` translation; and
- the structured sources backing every projected row.

The boundary must say which fields are projected into the compiler profile and which remain backend-only. Validate every genuine overlap, including buffer/index/launch facts only where both types mean the same thing. Equal compiler profiles may coexist with different nonprojected Metal facts only when the difference is explicitly irrelevant to compiler feasibility and is still carried and validated by its own owner. Never describe an assessment of the F32 subnormal projection as an assessment of the complete `MetalTargetFacts`.

The existing caller-vouched `declare_metal_f32_subnormal_behaviour` remains the low-level composable subnormal seam. Explicit measured declarations now admit the same fixed source across quantitative facts, exact dispatchability, and every non-subnormal numerical dimension without exposing an unrestricted source conversion; production construction must use those checked operations rather than ask a caller to pair arbitrary facts with arbitrary measurement contexts. If a future fact family cannot truthfully express empirical evidence available at `CompileProfile`, refine it through a reviewed type; never relabel bounded empirical evidence as an external portable guarantee or assign later-phase evidence to an earlier phase.

### 3. Source dispatchability and numerical facts without inheritance

State an exact macOS F32 dispatchability row from a real authority. Carry the measured F32 subnormal row with its exact compiler/environment contexts. Unmeasured family/dtype pairs reject. Do not infer F16 or BF16 from F32, do not claim BF16 on either iOS family, and do not claim any numerical result for an unmeasured physical iOS device.

The later BF16 spike owns the first non-F32 use of the mechanism. Its macOS positive row and iOS-Simulator refusal must reuse this profile construction rather than adding another backend dtype list.

### 4. Bind compiler, plan, artifact, cache, and runtime identities

Change `accept_or_publish_metal_plan` to accept the bound declaration and verify the compilation's exact profile key and descriptor before emission. Report an actionable typed mismatch. Retain Metal's backend-local `require_declared_realization` recheck so a direct emitter call or future compiler defect still fails closed.

Producer identity must distinguish every source, context, and fact that can change feasibility. Nonprojected emission facts continue to affect the payload identity they govern rather than being smuggled into the compiler profile. Mutation tests must show that changing a behaviour, compiler build, environment, or projected source changes the profile descriptor and therefore the artifact/cache subject.

The runner must stop deriving its host environment from `Compilation` or from the artifact it is validating. A host adapter independently validates the current platform family, compiler/environment applicability, and any other compile-profile predicates, then offers the same versioned `TargetProfileRef`. Key mismatch and same-key/different-descriptor mismatch both reject. Live device and prepared pipeline properties remain separate and must be checked at their existing preflight phases before the one-way routing commit.

### 5. Migrate the bounded serial sum

Only after every required compile-profile row has an authority:

- replace the producer's governed/hard-coded placeholder profile with the authoritative bound Metal declaration;
- compile and emit through the same bound declaration;
- package the exact profile reference and provenance selected by the checked plan; and
- make the runner offer that profile only after independent applicability validation.

This migration is the production proof. The F32 projection ticket and its synthetic tests are not production-profile evidence.

## Required evidence

- A checked-in per-axis authority ledger with reproducible primary references or exact retained measurements.
- Exact compiler-build and environment contexts for every measured compile-profile row.
- Tests showing omitted facts resolve as `Unknown`, not defaults.
- Tests rejecting duplicate or contradictory overlapping facts before emission.
- Mutations proving behaviour, source, compiler-build, and environment changes move the descriptor and downstream identity.
- A compiler mismatch rejected before Metal emission, plus a direct-emitter test preserving the backend recheck.
- Runtime tests for key mismatch, descriptor mismatch under the same key, and a host outside the profile's applicability.
- Phase tests refusing live-device or prepared-kernel evidence as a compile-profile declaration.
- A bounded serial-sum compile/run proof on an eligible host, with explicit reporting when that environment is unavailable.

## Closes when

1. Every fact consumed by the serial-sum compile phase has a named authority, source, validity, compiler/environment scope, and reproducible evidence; no unsourced quantitative constant is represented as a target fact.
2. One checked boundary binds the exact compiler profile, Metal emission facts, AOT target, and structured sources without claiming nonprojected fields were compiler-assessed.
3. F32 dispatchability and both F32 subnormal dimensions reach compiler feasibility; unknown families and dtypes fail closed; F16, BF16, and physical iOS-device support are not overclaimed.
4. `accept_or_publish_metal_plan` rejects a profile or projection mismatch before emission, while the backend recheck remains effective against a direct bypass.
5. Artifact and cache identity include the exact selected profile and source contexts, and every identity mutation test fails before restoration.
6. The runtime independently validates host applicability before offering the exact profile reference; it does not derive that offer from the compilation or artifact.
7. The serial-sum producer and runner use the authoritative path. If any required row lacks evidence, a named follow-up owns the measurement and this ticket remains open rather than filling the gap.
8. Tom reviews every consequential public boundary, all focused tests and `make full` pass, `tkt lint` and `git diff --check` pass, and the durable target-profile, Metal, numerical, runtime, artifact, status, and navigation contracts agree.

## Graph maintenance

- Follow `express-metal-honourability-in-the-shared-form`; it owns the ratified low-level F32 projection and no production migration.
- Replace the BF16 spike's direct dependency on the projection ticket with this ticket, because BF16 must consume the authoritative construction and runtime-applicability mechanism.
- Make production delivered-realization wiring depend on this ticket before it packages real Metal evidence. The redesign and public-boundary acceptance tickets may continue using checked synthetic evidence.
- Keep `consolidate-private-compiler-target-concerns` directly after the projection ticket; it is a private refactor and does not need production facts.
- Relate workload-definition research without blocking its definition. Any execution ticket derived from it must depend on this profile.

## Authority-ledger research outcome

**Fact:** the compiler consumes six quantitative axes for every scheduled-region proposal. The governed values `65_535`, `1`, `2`, `64`, `true`, and `0` are compiler-owned prototype declarations, not one authoritative Metal profile. Apple primary material supports 31 buffer argument-table entries, MSL `float`, the `device` address space, `uint` launch delivery, and `ulong` storage/arithmetic syntax, but it does not support treating the `uint` maximum as a compute-grid extent or Apple’s theoretical 1,024-thread family limit as a compiled pipeline capacity. The 64-bit integer-math row is Apple-family-specific and is not a generic Mac2 guarantee. The retired barrier-count row had neither a real target capacity nor a nonzero schedule consumer; zero synchronization is vacuous.

**Fact — resolved by the measured-source ticket:** measured compile-profile provenance was constructible only for the two complete F32 subnormal dimensions. `TargetProfileBuilder` now exposes explicit measured operations for all six quantitative axes, exact resolved-type dispatchability, and every non-subnormal numerical dimension, while retaining the complete-table-only subnormal path and preventing conversion to unrestricted `TargetFactSource`. The complete declaration v8 deduplicates provenance across the three fact families so one complete measured F32 profile remains within the bounded identity descriptor.

**Measurement:** the retained Apple numerical record qualifies F32 behavior on an Apple M4 Max under macOS 27.0 build 26A5388g, arm64, Xcode 26.6 build 17F113, macOS SDK 26.5 build 25F70, offline `metalfe-32023.883`, AIR-LLD 32023.883, and runtime/pipeline compiler `metalfe-32023.921`. That row is MSL 3.1 with an emitted macOS 14 triple, so it cannot be silently reused as the requested unified MSL 4.0/macOS 26 production profile. The later MSL 4 measurement ticket owns the exact replacement evidence.

**Measurement:** retained runs on the same named M4 Max report different registry IDs, while macOS and the simulator agree within each run. Registry ID is same-run correlation evidence, not stable hardware identity. `correct-apple-numerical-registry-id-authority` owns the prose correction and is deliberately related rather than a parent dependency.

**Fact — source correction:** current source identity is artifact program v10 and neutral manifest schema 8.0, with resolved value type v3, scheduled region v2, structured kernel v4, and verified kernel program v5. The target feasibility profile's checked descriptor encoding is v7 and its complete declaration is v8. Any implementation or mutation evidence produced here must use those current identities rather than an older artifact/manifest schema.

**Inference:** a truthful first profile must separate launch delivery, arithmetic-index width, and device-address width; consume no synchronization row for its zero-synchronization schedule; source or defer grid extent 4 and workgroup size 1 at their real phases; and independently earn the runtime profile offer from measured host predicates. The cheaper substitutions — 65,535 from `uint`, 1,024 from the feature table, a generic macOS 64-bit claim, a numeric barrier capacity, or a `Compilation`-derived host offer — were eliminated because each can silently certify a fact its source does not establish.

**Proposal:** after the five remaining blockers below close, `tiler-build` can own one bound declaration containing the checked compiler profile, exact F32-only Metal facts, total AOT target projection, structured sources, and host-applicability policy. Compiler buffer capacity must be no greater than the Metal emission limit; F32 subnormal facts must be projected once; backend-only language/platform/deployment facts remain payload identity; and LiveDevice/PreparedKernel facts remain later preflight obligations.

This parent remains open. The barrier-count blocker is resolved by removing the unsupported row; production migration must not begin by filling any other missing row with a convenient constant. It may proceed only after `admit-measured-compile-profile-sources-across-fact-families`, `measure-macos-apple9-f32-under-unified-msl4-profile`, `source-or-rephase-first-metal-launch-limits`, `separate-metal-launch-index-from-index-and-address-width`, and `validate-macos-metal-profile-host-applicability` are complete.
