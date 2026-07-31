---
id: construct-and-bind-the-first-authoritative-metal-compile-profile
title: Construct and bind the first authoritative Metal compile-time target profile
status: in-progress
priority: p0
dependencies: [express-metal-honourability-in-the-shared-form, admit-measured-compile-profile-sources-across-fact-families, measure-macos-apple9-f32-under-unified-msl4-profile, source-or-rephase-first-metal-launch-limits, separate-metal-launch-index-from-index-and-address-width, replace-or-justify-the-barrier-count-axis, validate-macos-metal-profile-host-applicability]
related: [admit-a-caller-declared-target-profile, carry-the-honourability-fact-provenance-into-the-artifact-record, decide-per-dtype-dispatchability-as-a-target-capability, record-metal-runtime-compiler-provenance-gap, prototype-metal-runtime-proof, measure-apple-numerics-on-physical-ios-device, spike-bf16-through-the-second-dtype-seams, redesign-the-delivered-realization-record-from-typed-evidence, close-or-retype-the-operand-permutation-inference, pin-the-serial-sum-producer-runner-shape-interface]
scopes: [implementation/build, implementation/compiler, implementation/metal, implementation/metal-aot, implementation/runtime, implementation/cargo-lock, contracts/foundation, contracts/numerics, contracts/artifacts, contracts/navigation, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, target-profile, numerics, runtime, provenance]
claimed_from: todo
assignee: loop-p0-profile
lease_expires_at: 1785527940
---
## User-visible outcome

A production caller can select one named, versioned macOS Metal compile profile whose every compiler-visible quantitative, operation-complete index-arithmetic, F32-dispatchability, and F32 numerical row has a reproducible authority and an exact validity, offline-compiler, and execution-environment scope. The compiler and Metal emitter consume one checked bound declaration, the Metal translation unit separately carries its selected emission realization, and the runtime independently validates that the current host is eligible to offer that exact `TargetProfileRef` before routing. Unknown or unmeasured rows reject. The current KIR consumes no device-address-width row, so that fact remains absent and `Unknown` until a real consumer and authority exist. Exact native translator/compiler identity remains `Unknown` and the accepted host-applicability authority must account for it; live-device and prepared-pipeline facts remain later preflight obligations rather than being promoted into the compile profile.

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
- operation-complete support for the governed unsigned-64 KIR index family;
- the explicitly absent device-address-width row and the consumer that would trigger reconsideration;
- F32 dispatchability;
- F32 input- and result-subnormal behaviour;
- the profile key and version;
- each compiler build and execution environment in a measured source; and
- every Metal target fact that overlaps a compiler fact, plus each separately selected emission realization.

For each row record its owner, source, authority, validity, availability phase, exact environment, and whether it is a **Fact**, **Measurement**, **Inference**, or still a **Proposal**. Prefer primary Apple specifications for normative limits and the retained Apple numerical harness for empirical rows. A row without sufficient evidence remains absent and therefore `Unknown`; do not fill it with a convenient constant, a nearby dtype, a different Apple family, or a live-device value.

The ledger must distinguish a conservative compile guarantee from a reported hardware maximum. Exact device maxima belong to `LiveDevicePreflight`; pipeline properties belong to `PreparedKernelPreflight`. If the serial sum needs a fact that cannot truthfully exist at `CompileProfile`, split a named measurement or contract ticket and leave the production migration blocked.

### 2. Construct one bound compile declaration

Define the smallest named `tiler-build` owner that can see the compiler and both Metal target vocabularies. It constructs:

- the checked compiler `TargetProfile`;
- the exact `MetalTargetFacts` used for emission;
- the exact `MetalEmissionRealization` selected for the translation unit;
- the total `MetalTargetFacts` to `tiler-metal-aot::MetalTarget` translation; and
- the structured sources backing every projected row.

The boundary must say which fields are projected into the compiler profile, which remain backend-only target facts, and which are selected emission realizations rather than capabilities. Validate every genuine overlap only where both types mean the same thing. In particular, the selected scalar `uint` launch declaration proves neither grid capacity, operation-complete unsigned-64 arithmetic, nor device-address width. Equal compiler profiles may coexist with different nonprojected Metal facts or emission realizations only when the difference is explicitly irrelevant to compiler feasibility and is still carried and validated by its own owner. Never describe an assessment of the F32 subnormal projection as an assessment of the complete `MetalTargetFacts`.

The existing caller-vouched `declare_metal_f32_subnormal_behaviour` remains the low-level composable subnormal seam. Explicit measured declarations now admit the same fixed source across quantitative facts, exact dispatchability, and every non-subnormal numerical dimension without exposing an unrestricted source conversion; production construction must use those checked operations rather than ask a caller to pair arbitrary facts with arbitrary measurement contexts. If a future fact family cannot truthfully express empirical evidence available at `CompileProfile`, refine it through a reviewed type; never relabel bounded empirical evidence as an external portable guarantee or assign later-phase evidence to an earlier phase.

### 3. Source dispatchability and numerical facts without inheritance

State an exact macOS F32 dispatchability row from a real authority. Carry the measured F32 subnormal row with its exact compiler/environment contexts. Unmeasured family/dtype pairs reject. Do not infer F16 or BF16 from F32, do not claim BF16 on either iOS family, and do not claim any numerical result for an unmeasured physical iOS device.

The later BF16 spike owns the first non-F32 use of the mechanism. Its macOS positive row and iOS-Simulator refusal must reuse this profile construction rather than adding another backend dtype list.

### 4. Bind compiler, plan, artifact, cache, and runtime identities

Change `accept_or_publish_metal_plan` to accept the bound declaration and verify the compilation's exact profile key and descriptor before emission. Report an actionable typed mismatch. Retain Metal's backend-local `require_declared_realization` recheck so a direct emitter call or future compiler defect still fails closed.

Producer identity must distinguish every source, context, and fact that can change feasibility. Nonprojected emission facts continue to affect the payload identity they govern rather than being smuggled into the compiler profile. Mutation tests must show that changing a behaviour, compiler build, environment, or projected source changes the profile descriptor and therefore the artifact/cache subject.

The runner must stop deriving its host environment from `Compilation` or from the artifact it is validating. A host adapter independently validates the current platform family, native execution-environment applicability, and any other compile-profile predicates, then offers the same versioned `TargetProfileRef`. It must not substitute the source-JIT compiler measured on a comparison path, OS build, or loaded-image membership for an attributable AOT compiler identity. Key mismatch and same-key/different-descriptor mismatch both reject. Live device and prepared pipeline properties remain separate and must be checked at their existing preflight phases before the one-way routing commit.

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
- Mutations proving behaviour, source, offline-compiler build, and environment changes move the descriptor and downstream identity.
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

**Fact:** the compiler consumes five quantitative axes for every current scheduled-region proposal: grid-axis threads, workgroup threads, buffer bindings, device-memory-space availability, and local-memory bytes. It separately consumes operation-complete support for the governed unsigned-64 KIR index family. The old exact `64` row was not device-address-width evidence, and the current buffer-relative KIR has no device-address-width consumer, so that row is absent and remains `Unknown`. The governed quantitative values remain compiler-owned prototype declarations rather than one authoritative Metal profile. Apple primary material supports 31 buffer argument-table entries, MSL `float`, the `device` address space, and MSL 4.0 `uint64_t` syntax, while 64-bit integer arithmetic support remains operation-complete and Apple-family-scoped rather than a generic Mac2 guarantee. MSL separately permits `ushort` or `uint` for `thread_position_in_grid`; Tiler's scalar `uint` is a selected emission realization and its maximum coordinate value is neither a compute-grid extent nor evidence for either width. Apple’s theoretical 1,024-thread family limit is likewise not a compiled pipeline capacity. The zero-synchronization schedule consumes no synchronization fact.

**Fact — resolved by the measured-source ticket:** measured compile-profile provenance was constructible only for the two complete F32 subnormal dimensions. `TargetProfileBuilder` now exposes explicit measured operations for every quantitative axis, exact resolved-type dispatchability, and every non-subnormal numerical dimension, while retaining the complete-table-only subnormal path and preventing conversion to unrestricted `TargetFactSource`. The corrected complete declaration v9 deduplicates provenance across the fact families so one complete measured F32 profile remains within the bounded identity descriptor.

**Measurement:** the retained Apple numerical record qualifies F32 behavior on an Apple M4 Max under macOS 27.0 build 26A5388g, arm64, Xcode 26.6 build 17F113, macOS SDK 26.5 build 25F70, offline `metalfe-32023.883`, AIR-LLD 32023.883, and the separate source-JIT compiler `metalfe-32023.921`. That row is MSL 3.1 with an emitted macOS 14 triple, so it cannot be silently reused as the requested unified MSL 4.0/macOS 26 production profile. The later MSL 4 measurement ticket owns the exact replacement evidence. The AOT observer subsequently established that native metallib/pipeline preparation cannot attribute its private translator/compiler identity and that the source-JIT build is not evidence for it. Exact native translation identity therefore remains `Unknown`; the applicability decision must state whether the measured OS/architecture/device row is sufficient validity scope.

**Measurement:** retained runs on the same named M4 Max report different registry IDs, while macOS and the simulator agree within each run. Registry ID is same-run correlation evidence, not stable hardware identity. `correct-apple-numerical-registry-id-authority` owns the prose correction and is deliberately related rather than a parent dependency.

**Fact — source correction:** current source identity is artifact program v11 and neutral manifest schema 9.0, with resolved value type v3, scheduled region v2, structured kernel v4, and verified kernel program v6. The target feasibility profile's checked descriptor encoding is v9, its complete declaration is v10, and the governed feasibility vocabulary is `tiler.feasibility.phased-capability-and-numerical-honourability.v4` revision 1. Any implementation or mutation evidence produced here must use those current identities rather than an older artifact/manifest schema.

**Inference:** a truthful first profile must source operation-complete unsigned-64 KIR arithmetic independently, leave device-address width absent while no current consumer exists, and keep the selected scalar `uint` launch declaration on the emission realization rather than the target profile. It must consume no synchronization row for its zero-synchronization schedule, source or defer grid extent 4 and workgroup size 1 at their real phases, and independently earn the runtime profile offer from measured host predicates. The cheaper substitutions — deriving arithmetic support or address width from `uint64_t` syntax, deriving 65,535 or any other grid limit from a launch parameter type, using 1,024 from the feature table as a compiled pipeline capacity, asserting generic macOS 64-bit support, restoring a numeric barrier capacity, or deriving the host offer from `Compilation` — were eliminated because each can silently certify a fact its source does not establish.

**Proposal:** after the remaining blockers below close, `tiler-build` can own one bound declaration containing the checked compiler profile, exact F32-only Metal target facts, selected Metal emission realization, total AOT target projection, structured sources, and host-applicability policy. Compiler buffer capacity must be no greater than the Metal emission limit; operation-complete unsigned-64 arithmetic must have an exact applicable authority; F32 subnormal facts must be projected once; the absent device-address-width row must not be synthesized; backend-only language/platform/deployment facts, offline compiler provenance, and selected emission choices remain payload identity; exact native translation identity remains `Unknown` unless a stronger observer appears; and LiveDevice/PreparedKernel facts remain later preflight obligations.

This parent remains open. The barrier-count blocker is resolved by removing the unsupported row; production migration must not begin by filling any other missing row with a convenient constant. It may proceed only after `admit-measured-compile-profile-sources-across-fact-families`, `measure-macos-apple9-f32-under-unified-msl4-profile`, `source-or-rephase-first-metal-launch-limits`, `separate-metal-launch-index-from-index-and-address-width`, and `validate-macos-metal-profile-host-applicability` are complete.

## Work item 1 delivered — the authority ledger

**Fact:** [`docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md`](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md) enumerates every compile-phase fact the bounded serial-sum path consumes, with owner, source, authority class, validity scope, availability phase, exact environment, and a Fact/Measurement/Inference label per row. It is registered in the `docs/research/README.md` catalog. Work items 2 through 5 remain open and may construct the bound declaration from exactly these rows.

**Fact — every compile-phase row now has an authority.** Grid-axis 4 (macOS 26.5 SDK `dispatchThreads` API contract, conservative); buffer bindings 31 and local memory 32,768 (Apple Metal Feature Set Tables 2025-10-20, `Apple9` column, verified by `pdftotext`); index arithmetic `CompleteU64` (feature tables row `64-bit integer math` = `Metal3 | Apple3 | —`, Apple-family-scoped, which is why the `apple9` applicability predicate is part of the row's validity and not a separate concern); device address space (MSL 4.0 `device`); F32 dispatchability and both F32 subnormal dimensions (the retained MSL 4 run, with execution witnesses). Workgroup threads stays a `PreparedKernelPreflight` query. Device address width stays absent and `Unknown` with a recorded trigger. No synchronization row exists.

**Measurement — the numerical rows are isolated more strongly than the flag names.** The retained record's per-case `float_operations` field holds the fast-math attributes the front end actually emitted: `safe` cases are bare, `relaxed` adds `reassoc nsz arcp afn`, `fast` adds `nnan ninf`, and `+contract` tracks the contraction setting independently. That isolates contraction, reassociation, signed zero, NaN, and infinity as measurements of what was delivered rather than of what was requested. Operand permutation has no isolating case and is labelled `Inference`; the ledger names the two pieces of evidence that would close it.

**Inference — the deployment minimum moves, and this is a real migration consequence.** The retained MSL 4 row compiled `-std=metal4.0` for `air64-apple-macos26.0`. The current prototypes state MSL 3.1 and macOS 14.0. Work item 5 must move both, because reusing the older record for this profile would attribute measurements to a compilation that did not produce them.

**Fact — the ADR 0086 boundary, derived rather than asserted.** [ADR 0086](../docs/decisions/0086-require-attributable-or-attested-native-translation.md) is accepted, and its Consequences section states the split this ticket needs: this profile's "quantitative, dispatchability, and F32 numerical rows are unaffected as measurements; what they lack is the applicability authority that would let a host offer the profile." So work items 2 through 4 are unblocked, and the *runtime offer* in item 5 is not. `tiler_metal::applicability::MetalHostEligibility` holds a `NativeTranslationAuthority` whose one field is a private uninhabited enum, so a positive receipt is impossible to construct anywhere — a runner that offers this profile only from a receipt cannot offer it on any host that exists today. That is closes-when 6 being structurally unsatisfiable on current APIs, not a defect to route around, and the cheaper reading — treating a matching public environment row as sufficient — is the alternative ADR 0086 explicitly rejected. Item 5's producer-side migration and the runner's removal of `host_environment(&Compilation)` remain in scope; the eligible-host offer waits on an ADR 0086 reconsideration trigger.

## Item 5 resolution — decided by Tom, 2026-07-31

**Honest migration with a labelled probe.** The production offer path refuses with ADR 0086's typed refusal, proven on hardware as a refusal; the prototype retains the envelope path as an explicitly-labelled diagnostic — "producer-declared equality, NOT host-earned eligibility" — that keeps exercising the runtime machinery (decode, route, ABI bind, two-stage qualification, dispatch) on hardware without making the authority claim ADR 0086 gates. `host_environment(&Compilation)` is removed from anything that claims authority; the labelled probe states its epistemic status instead of implying one. Items 2 through 5 proceed as one brief under this resolution.

## Work items 2 through 5 delivered

**Fact — the bound declaration.** `tiler_build::BoundMetalCompileDeclaration` (`crates/tiler-build/src/metal_declaration.rs`) is the smallest named owner that can see the compiler's target vocabulary, the Metal emitter's, and the AOT driver's at once. It carries the checked `TargetProfile` keyed `tiler.metal.macos-apple9.msl4-0.f32.v1` (1,741 descriptor bytes), the exact `MetalTargetFacts`, the selected `MetalEmissionRealization` and `NumericalRealization`, and the total `MetalTarget` projection resolved once at declaration time. Its private `LedgerRows` record transcribes the ledger one field per row, which is what makes each row's contribution to the descriptor separately testable. Public construction is closed to `first_macos_apple9`, so no caller can mint a profile for a row nobody measured.

**Fact — the authority classes stay distinct.** Grid-axis 4, buffers 31, index arithmetic `CompleteU64`, device address space, and local memory 32,768 are external normative guarantees under three separately versioned references (the macOS 26.5 SDK dispatch header, the 2025-10-20 feature tables, the MSL 4.0 address-space chapter). F32 `Dispatchable` and every F32 numerical row carry one `TargetCompileProfileMeasurementSource` whose single context pairs four offline components — `metalfe-32023.883`, AIR-LLD 32023.883, Xcode 26.6 17F113, macOS SDK 26.5 25F70 — with the execution environment macOS 27.0 / 26A5388g / arm64 / Apple M4 Max. `metalfe-32023.921` is absent by name. Workgroup threads is a `PreparedKernelPreflight` query; device address width, synchronization, F16, and BF16 have no row and resolve `Unknown`.

**Fact — exactly two overlaps are validated.** Compiler buffer capacity is checked no greater than the emission limit (directionally, and a lower compiler capacity is accepted), and the F32 subnormal projection runs once through `declare_metal_f32_subnormal_behaviour`. Nothing else is compared, and `nonprojected_metal_facts_do_not_reach_the_compiler_descriptor` asserts that changing the language standard moves the AOT target while leaving the compiler descriptor byte-identical — the assessment of the subnormal projection is never described as an assessment of `MetalTargetFacts`.

**Fact — consumption and identity.** `accept_or_publish_metal_plan` takes the declaration and an `OptimizationLevel` (which no ledger row is scoped to), and verifies the compilation's exact profile key and descriptor *before* `emit_translation_unit`, reporting `MetalPlanProfileMismatch::{ProfileKey, ProfileDescriptor}` — the same key/descriptor split `tiler-runtime` classifies by. `MetalPlanBuildPolicy` is removed; its three `compile_fail` doctests pinning the launch-realization negatives moved onto `BoundMetalCompileDeclaration`. Six projected rows and nine measurement-context fields are each separately proved to move the profile descriptor.

**Fact — item 5 under the recorded resolution.** Both prototypes state no target fact of their own and moved from MSL 3.1 / macOS 14.0 to MSL 4.0 / macOS 26.0. `host_environment(&Compilation)` is gone. The runner asks the two authority questions separately: `offer_the_declared_profile` observes the host and refuses, and `declared_route_environment` states the producer's declaration under a label that says so. `prototypes/serial-sum-run` gained a `tiler-build` dependency because the ticket forbids deriving the offer from a `Compilation` or from the artifact under validation, which leaves the declaration as the only honest source; that added `implementation/cargo-lock` to this ticket's scopes.

**Measurement — the bounded proof, Apple M4 Max, macOS 27.0 build 26A5388g.** The producer published six members under the authoritative profile; the runner proved 30 operand cases across them with fused and materialized agreeing bit for bit with the published reference, plus the deep single-member proof over the fail-closed, device-preflight, and post-commit probes. The production offer path printed `metal.host-applicability.unknown-translation-authority: native-translation-authority is unknown for tiler.metal.host-applicability.macos-27.0-26A5388g-arm64-m4max-apple9.v1`, on a host matching the measured row in every public field.

**Fact — a defect the migration exposed, fixed here.** `prove_member` compiled the runner's own `ROWS = 4` against artifacts the producer publishes with one row, so every packaged program was foreign and the entire matrix pass proved nothing. It was introduced by 0b7e59d (2026-07-30) three days after 1f4b7fc added the pass, and no gate reaches it because the matrix runs only against real published members on hardware. It now reads the shape from the artifact, as the deep proof already did. `pin-the-serial-sum-producer-runner-shape-interface` owns making that class of drift gate-visible.

**Fact — what closes-when 6 actually reports.** The runtime validates host applicability independently and *refuses*; it does not offer the profile, and no implementation task can make it. That is ADR 0086 applied, recorded as the ledger's third outcome with the ADR's own reconsideration triggers, and it is why the outcome above says "refuses" rather than "offers".

**Unsupported, and named.** Operand permutation is declared through a measured-source operation while the ledger labels it an `Inference`; `close-or-retype-the-operand-permutation-inference` owns closing or retyping it. Reciprocal transform, approximate intrinsics, and materialization rounding have no row, matching the ledger and the governed profile's own dimension set. Nothing here states an F16, BF16, F64, or iOS-family fact.
