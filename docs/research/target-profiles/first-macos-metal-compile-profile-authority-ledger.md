---
schema: "tiler-doc/v1"
id: "tiler.research.target-profiles.first-macos-metal-compile-profile-authority-ledger"
kind: "research"
title: "Authority ledger for the first macOS Metal compile profile"
topics: ["targets", "feasibility", "metal", "apple-targets", "numerics", "provenance"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "adopted"
implementation_status: "implemented"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.metal-backend", "tiler.contract.artifact-abi", "tiler.contract.numerical-semantics"]
evidence: ["tiler.research.apple-targets.numerical-behaviour", "tiler.research.apple-targets.compatibility", "tiler.research.target-profiles.physical-feasibility-model"]
ticket: "construct-and-bind-the-first-authoritative-metal-compile-profile"
---

# Authority ledger for the first macOS Metal compile profile

**Status:** research record for `construct-and-bind-the-first-authoritative-metal-compile-profile`, work item 1.
**Ticket:** `construct-and-bind-the-first-authoritative-metal-compile-profile`

**Evidence boundary.** This ledger enumerates every fact the bounded serial-sum compile phase consumes and names, per row, the authority that establishes it. It settles what each row may say; it does not itself construct a profile, and constructing one is work items 2 through 5 of the same ticket. Every quantitative row below was read from a primary Apple document vendored in this repository or from the macOS 26.5 SDK headers, and every numerical and dispatchability row from one retained measurement directory. Nothing here is transcribed from a second-hand summary, and no row is filled from a nearby dtype, a nearby Apple family, a live device, or a prepared pipeline.

Rows are labelled **Fact** (primary documentation or inspected source), **Measurement** (an observation tied to an exact environment and procedure), **Inference** (derived from stated facts), and **Proposal** (a design not yet accepted). A reader acts differently on each, and three rows below are `Unknown` — which is a fourth thing, and is neither a fact nor a refuted claim.

## The two environments this ledger keeps apart

A compile-profile row is scoped by *where the evidence came from*, and two different environments produce evidence in this profile. Collapsing them is the mistake this section exists to prevent.

**The offline compilation environment** produced the bytes. It is the exact toolchain the retained measurement invoked, and it is what a numerical row measured under those flags is valid for:

| Component | Exact value |
| --- | --- |
| Offline compiler | `Apple metal version 32023.883 (metalfe-32023.883)` |
| Offline linker | `AIR-LLD 32023.883 (metalfe-32023.883) (compatible with legacy metallib linker)` |
| Xcode | `Xcode 26.6 Build version 17F113` |
| macOS SDK | `macosx` 26.5, build `25F70` |
| Requested target | `air64-apple-macos26.0` |
| Emitted triple | `air64_v28-apple-macosx26.0.0` |
| Language standard | `-std=metal4.0` (MSL 4.0) |

**The execution environment** ran the kernels that produced the numerical observations:

| Component | Exact value |
| --- | --- |
| OS | macOS 27.0, build `26A5388g` |
| Architecture | `arm64` |
| Device | `Apple M4 Max` |
| Apple GPU family | `apple9` (`device_apple9_support supported`) |

Both rows are transcribed from `spikes/apple-targets/results/2026-07-30-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`, keys `environment.*` and `probe.*`, run `environment.date_utc 2026-07-30T21:15:27Z`, harness `probe.harness_sha256 ef224faf467be9321e4f2086d47543916ddae78c5bcca67eb42ebedcf2fc91e1` at repository base revision `0cd85ce5f01470fe8410c61d3ff4128a0633b207`.

**The `macos26` in that directory name is the deployment minimum of the offline request, not the host OS version.** The host ran macOS 27.0. A reader reconciling the two should not "correct" either. The same warning is already carried by `crates/tiler-metal/src/applicability.rs`, and it is repeated here because this is the other document a reader arrives at with both numbers in view.

**`metalfe-32023.921` is not in either table, deliberately.** `record.tsv` records it under `environment.family.macos.runtime_compiler` as the build the host loads for `newLibraryWithSource:options:`. Tiler's AOT route supplies no source, so that build is evidence about a comparison path and about nothing this profile compiles. [ADR 0086](../../decisions/0086-require-attributable-or-attested-native-translation.md) item 4 excludes it by name.

## Quantitative rows

Five quantitative axes reach every current scheduled-region proposal, plus one operation-complete arithmetic row. Each row below states the value the profile may offer and the authority that permits it.

### Grid-axis threads — 4

- **Owner:** compiler `CapabilityAxis::GridAxisThreads`, meaning dispatched thread extent along one grid axis.
- **Authority:** Fact — normative, from the macOS 26.5 SDK. `MTLComputeCommandEncoder.h` declares `dispatchThreads:threadsPerThreadgroup:` as dispatching an arbitrarily sized grid and states explicitly that the grid need not be a multiple of the threadgroup; `MTLTypes.h` types every `MTLSize` dimension as `NSUInteger`. The API is available from macOS 10.13.
- **Validity:** the exact macOS 26 profile, as an API contract.
- **Phase:** `CompileProfile`.
- **Why 4 and not a maximum.** The API contract proves that extent 4 is *representable*; it establishes no upper bound at all. Four is therefore a deliberately conservative compile guarantee chosen to cover the bounded serial-sum program, not an Apple9 hardware maximum. `source-or-rephase-first-metal-launch-limits` established this row and its reasoning; this ledger transcribes rather than re-derives it.
- **Eliminated:** 65,535 (no inspected source states it); the maximum value of the `uint` launch builtin (a coordinate ceiling is not a thread count, and the builtin's declared type is a selected emission realization); any Apple9 hardware grid maximum (the feature tables name only object- and mesh-shader grid rows, and neither is a compute-grid capacity).

### Workgroup threads — absent as a fact, declared as a prepared-kernel query

- **Owner:** compiler `CapabilityAxis::WorkgroupThreads`.
- **Authority:** Fact — the row that *would* fill it is refused by its own source. Apple's [Metal Feature Set Tables](../apple-targets/sources/apple-metal-feature-set-tables-2025-10-20.pdf) (2025-10-20) report `Maximum threads per threadgroup` as 1,024 for Apple9, and footnote 4 directs the reader to `MTLComputePipelineState.maxTotalThreadsPerThreadgroup` for the actual compiled-function maximum. A theoretical family limit is not a compiled pipeline's capacity.
- **Phase:** `PreparedKernelPreflight`, as a `QuantitativeCapabilityQueryDeclaration` naming property key `tiler.target.prepared-entry.max-threads-per-workgroup.v1` and provider `tiler`/`prepared-entry-properties` revision 1.
- **Inference:** the serial sum requires workgroup size 1. That requirement is the *left* side of `1 <= prepared.maxTotalThreadsPerThreadgroup`, and the right side is bound only once the exact pipeline exists. No inspected primary source states a portable minimum of one thread for every compiled pipeline, so there is nothing to promote to `CompileProfile`.
- **Measurement — corroboration only, not a source.** On the Apple M4 Max under Xcode 26.6 (17F113) and macOS SDK 26.5, a minimal prepared pipeline reported `maxTotalThreadsPerThreadgroup = 1024`, and a kernel constrained and required at `(1, 1, 1)` prepared successfully and reported 1. That the two differ is the point; neither value may source a compile profile.

### Buffer bindings per entry — 31

- **Owner:** compiler `CapabilityAxis::BufferBindings`.
- **Authority:** Fact — normative. Apple's Metal Feature Set Tables (2025-10-20), "GPU implementation limits by family", row `Maximum number of entries in the buffer argument table, per graphics or kernel function`, column `Apple9`: **31**. The same table's column header row is `Metal3 | Metal4 | Apple2 … Apple10 | Mac2`, and every column on this row reads 31.
- **Validity:** the Apple9 family named by the column.
- **Phase:** `CompileProfile`.
- **Overlap:** this is the one quantitative row that also exists as a Metal target fact, `MetalTargetFacts::buffer_binding_limit`. See "Overlaps" below; the two must be validated equal, and the compiler's offered capacity must never exceed the emission limit.

### Index arithmetic — `CompleteU64`

- **Owner:** compiler `CapabilityAxis::IndexArithmeticU64`, meaning complete support for the governed unsigned-64 operation family `tiler_ir::kernel::KernelType::Index` may emit.
- **Authority:** Fact — normative, Apple-family-scoped. Apple's Metal Feature Set Tables (2025-10-20), "GPU family 1" table, row `64-bit integer math`: `Metal3 | Apple3 | —`. The three columns are the minimum Metal version, the minimum Apple family, and the minimum Mac family. MSL 4.0 ≥ Metal3 and Apple9 ≥ Apple3, so the row applies to this profile.
- **Validity:** the Apple family, not the artifact family. **The Mac column is `—`.** This is exactly why `separate-metal-launch-index-from-index-and-address-width` records that a Mac artifact family alone does not imply the support 64-bit arithmetic needs: a macOS artifact running on a Mac2-only GPU would be outside this row. The applicability policy's `apple9` predicate is what bounds the claim, and it is a *necessary* part of this row's validity scope rather than a separate concern.
- **Phase:** `CompileProfile`.
- **Inference, and it is labelled as one.** The table names the feature "64-bit integer math" and enumerates no operation subset. Reading it as *operation-complete* over the governed index family is an inference from a source that does not itself decompose the feature. It is the reading the row's own name supports and the only reading under which the row is usable; a future source that enumerates operations could narrow it. What the inference does **not** rest on is MSL 4.0's `uint64_t` syntax: a spellable type is a language fact, and `separate-metal-launch-index-from-index-and-address-width` eliminated deriving arithmetic support from it.
- **Eliminated:** deriving support from the existence of `uint64_t` or `ulong` (syntax is not arithmetic); deriving it from a 64-bit storage slot; asserting generic macOS 64-bit support (the Mac column refutes it).

### Device address space — available

- **Owner:** compiler `CapabilityAxis::DeviceAddressSpace`, meaning an explicitly addressable device memory space exists.
- **Authority:** Fact — normative. MSL 4.0 defines the `device` address space, and every buffer this profile binds is emitted into it.
- **Validity:** the language standard, MSL 4.0.
- **Phase:** `CompileProfile`.
- **Distinct from device-address *width*.** Availability of an address space and the width of the address model are two facts; the second is absent below.

### Local memory bytes — 32,768

- **Owner:** compiler `CapabilityAxis::LocalMemoryBytes`, meaning maximum explicitly staged local memory.
- **Authority:** Fact — normative. Apple's Metal Feature Set Tables (2025-10-20), "GPU implementation limits by family", row `Maximum total threadgroup memory allocation`, column `Apple9`: **32 KB**, which is 32,768 bytes.
- **Validity:** the Apple9 family named by the column.
- **Phase:** `CompileProfile`.
- **Note on the consumer.** The bounded serial-sum schedule stages no local memory, so it requires 0 and this row is not what admits it. The row is stated because the axis is one the compiler consults for every proposal, and stating the sourced family value is truthful where stating a convenient 0 would be a number with no authority behind it. A reader should not read this row as evidence that any Tiler schedule has used threadgroup memory.

### Device address width — **absent, and therefore `Unknown`**

- **Owner:** compiler `CapabilityAxis::DeviceAddressWidthBits`.
- **Authority:** none, and none is sought.
- **Why absent.** The current structured kernel IR performs buffer-relative integer offsets and has no pointer-integer operation, so no proposal consumes a device-address-width requirement. A row with no consumer and no exact authority is a row that can only be wrong.
- **Reconsideration trigger:** the first KIR operation that converts between a device pointer and an integer, or that requires a flat device address. That operation's ticket owns finding the authority; it may not reuse `uint64_t`, `size_t`, the `air64` triple spelling, or the launch builtin's declared type, each of which `separate-metal-launch-index-from-index-and-address-width` eliminated by name.

### Synchronization — **no row exists**

- `replace-or-justify-the-barrier-count-axis` removed `CapabilityAxis::Barriers` outright. The bounded serial-sum schedule contains no synchronization point, so it emits no feasibility requirement, and a vacuous requirement over an invented numeric capacity was the wrong model rather than a missing measurement. There is no row here to source, and no replacement synchronization fact is introduced.
- The first genuine nonzero synchronization path is owned by `admit-the-first-typed-synchronization-point-and-atomic-target-authority` and is deliberately not a blocker for this zero-synchronization profile.

## Dispatchability

### F32 — `Dispatchable`

- **Owner:** compiler `DTypeDispatchabilityFact` keyed by the exact `ResolvedValueType` of `F32`.
- **Authority:** Measurement, not a normative guarantee. The retained MSL 4 run dispatched F32 compute kernels on the macOS/Apple9 execution environment above and read back results, with `probe.dtypes f32` and `probe.status validated`. Every case carries an `execution_witness` on a non-subnormal operand reporting `status=executed`, which is what separates "the arithmetic ran" from "the kernel was optimized away".
- **Validity:** `MeasuredEnvironment` — the exact offline compilation environment and the exact execution environment tabulated above, together.
- **Phase:** `CompileProfile`, via `TargetCompileProfileMeasurementSource`, whose phase, authority, and validity are fixed by construction and cannot be widened to a portable claim.
- **Inheritance is refused in every direction.** F16 and BF16 are `Unknown` on this profile: they were not measured under MSL 4.0, and the F32 row may not answer for them. The `express-metal-honourability-in-the-shared-form` record establishes that the measured Apple row *disagrees* across dtypes — F32 arithmetic flushes where F16 preserves on the same hardware in the same math modes — so inheritance here is not merely unproven, it is known to be unsound in at least one direction. No iOS family, physical or simulated, gains a row from this one.

## Numerical rows

Every numerical row in this profile is a **Measurement** under the exact offline compiler and flags, not a portable normative guarantee. Two of them are the complete exclusive subnormal tables; the rest are the honourability dimensions the compiler consults for the declared contract.

**The flags are part of the row.** These facts describe what the *selected numerical realization* delivers through the *exact offline compiler*, so the bound declaration must carry that realization and the measurement source must carry that compiler build. The retained cases this ledger reads are the `safe` math-mode, `contract-off` ones, which is `NumericalRealization::strict_baseline`. A row read from a `relaxed` or `fast` case would be a different fact about a different compilation.

### F32 input subnormals — flush to zero, preserving sign

- **Measurement.** `case.macos.multiply_two.safe.O2.contract-off.results` returns `00000000` for the subnormal operand `00400000` and `80000000` for `80400000`. The sign row is what makes the zero a measured `PreservesSign` rather than an assumed `+0.0`, and the sibling `multiply_half` case agrees.
- **Execution witness.** `case.macos.multiply_two.safe.O2.contract-off.execution_witness` reads `operand=3f800000,expected=40000000,observed=40000000,status=executed`. Without it, `00000000` and "the arithmetic never ran" would be the same observation.
- **Not materialization.** `case.macos.materialize.safe.O2.contract-off.results` returns all eight operands unchanged — `00000001 00400000 007fffff 00800000 80400000 80000000 3eb97ef9 3f800000` — so the flush is a property of arithmetic and not of a buffer round trip. `case.macos.materialize.safe.O2.contract-off.execution_witness` is `none`, correctly: there is no arithmetic to witness.
- **Declared form:** the complete exclusive three-row table — `Preserve` unsupported, `FlushToZero { PreservesSign }` exact, `FlushToZero { AlwaysPositive }` unsupported.

### F32 result subnormals — flush to zero, preserving sign

- **Measurement.** `case.macos.multiply_half.safe.O2.contract-off.results` returns `00000000` for `00800000`, the least positive *normal*, whose halved result is subnormal. That is the result-side dimension isolated: the operand is normal, so only the result can have been flushed.
- **Declared form:** the same complete exclusive three-row table, on the result dimension.

### The emitted float-operation attributes, which isolate four of the five remaining dimensions

The record retains a `float_operations` field per offline case, holding the LLVM fast-math attributes the front end actually emitted. Across all 688 retained cases the attribute strings fall into a small set, and the discriminator is exactly the math mode:

- `safe` cases emit bare operations — `fmul`, `fadd`, `fdiv`, `fmul fadd`, `air.fma.f32` — carrying **no** relaxation attribute.

  **`air.fma.f32` under `safe` is not contraction, and a reader will be tempted to read it as such.** It is the `fused_pair` kernel's *explicit* `fma` call, which the front end lowers to the intrinsic whatever the contraction setting is. Compiler contraction appears as the separate `+contract` attribute, and the retained set contains both `air.fma.f32` and `air.fma.f32+contract` for exactly that reason. The contraction row below is sourced from `contraction_pair`, which calls no intrinsic, and never from this one.

- `relaxed` cases add `reassoc nsz arcp afn`.
- `fast` cases add `nnan ninf` on top of those.
- `contract-fast` and `contract-on` cases add `+contract`, and `contract-off` cases do not.

That is a direct, isolated observation of what the selected realization permits, and it is stronger than reading the flag names off the command line: it is what the compiler emitted, not what it was asked for. Each row below cites it.

### Contraction — forbidden, delivered exactly

- **Measurement.** The `contraction_pair` and `contraction_pair_canonicalized` kernels are retained at `contract-off`, `contract-on`, and `contract-fast`. Lane 7 of the result vector is the discriminator: `3fc58f9d` where the pair contracted into an FMA and `3fc58f9e` where it did not. The selected `safe`/`contract-off` realization produces `3fc58f9e`, and its `float_operations` carries no `+contract`.
- **Why the pair and not one expression.** A single expression cannot show that contraction did *not* happen; two spellings that must agree can.
- **Validity:** the exact offline compiler and the `-ffp-contract=off` flag, which is why this row travels with the selected numerical realization rather than with the hardware.

### Reassociation — forbidden, delivered exactly

- **Measurement.** The `reassociation_chain` kernel separates the modes in the results themselves: `safe` returns lane 8 as `3f800000` while `relaxed` and `fast` return `3f800001`. The `safe` case's `float_operations` carries no `reassoc`, and the relaxed and fast cases carry `fadd+reassoc+…`.

### Signed zero — forbidden, delivered exactly

- **Measurement.** `nsz` — LLVM's "no signed zeros" relaxation — appears in the `relaxed` and `fast` attribute strings and in neither `safe` string. The results agree: under `safe`, `scale_one_bias_zero` carries `80400000` and `80000000` through a flush and a `+0.0f` bias to `00000000`, which is IEEE round-to-nearest behaviour for `(-0.0) + (+0.0)` rather than a discarded sign.

### NaN and infinity assumptions — no assumption made, delivered exactly

- **Measurement.** `nnan` and `ninf` appear only in the `fast` attribute strings. The `safe` cases this profile compiles under carry neither, so the compiler is making no finite-math assumption.

### Operand permutation — forbidden, and this row is an `Inference`

- **No retained case isolates it.** The four rows above each have an attribute or a result lane that separates the modes; operand permutation has neither. It is delivered by the same `safe` compilation, whose attribute strings carry no relaxation at all, so a permutation relaxation would have to be one the front end applied without recording it.
- **Inference, labelled as one.** Sound enough to construct a profile with, and not the same class of evidence as the five rows above. A reader must not quote this row as an isolated measurement.
- **What would close it.** One retained kernel whose result distinguishes an operand order, compiled under the exact offline compiler, or a citation to MSL 4.0's normative statement of what `-fmetal-math-mode=safe` guarantees about operand order.

## Metal target facts, and which of them project

`MetalTargetFacts` is the emitter's input record. Only some of its fields have a compiler-profile counterpart, and the difference is load-bearing: a field that does not project must never be described as compiler-assessed.

| Metal field | Value for this profile | Projects into the compiler profile? |
| --- | --- | --- |
| `language` | `MslLanguageVersion::Metal4_0` | No — backend-only; it bounds the *validity* of rows above without being one |
| `platform` | `MetalPlatform::MacOs` | No — backend-only artifact family |
| `deployment_minimum` | `MetalDeploymentMinimum::new(26, 0)` | No — backend-only; recorded in emitted provenance and in the target triple |
| `subnormal_arithmetic` (F32 entry) | `FlushesToZero { PreservesSign }` | **Yes** — into both F32 subnormal dimensions |
| `buffer_binding_limit` | 31 | **Yes** — into `BufferBindings` |

**Fact.** The deployment minimum here is 26.0, because `probe.fixed_flags -std=metal4.0` and `environment.family.macos.requested_target air64-apple-macos26.0` are the inputs the retained measurement actually used. Reusing the older MSL 3.1 / macOS 14.0 record for this profile would attribute measurements to a compilation that did not produce them. Both prototypes stated that older record until the migration below; neither states any target fact now.

**Selected, not a capability:** `MetalEmissionRealization { launch_index: LaunchIndexRealization::ThreadPositionInGridUInt }`. MSL 4.0 Table 5.8 permits `[[thread_position_in_grid]]` as either `ushort` or `uint`; Tiler selects `uint` and widens explicitly to the governed `uint64_t` index type. It is carried by the translation unit, it affects payload identity, and it proves nothing about grid capacity, arithmetic support, or address width — three `compile_fail` doctests in `crates/tiler-build/src/metal_plan.rs` already pin each of those three negatives.

## Overlaps, and what validating one means

Exactly two facts are stated in both vocabularies, and each must be validated where — and only where — the two mean the same thing.

1. **Buffer capacity.** `MetalTargetFacts::buffer_binding_limit` and `CapabilityAxis::BufferBindings` mean the same quantity. The compiler's offered capacity must be no greater than the Metal emission limit, or the compiler would admit a signature the emitter must then reject. Both are 31 here, from the same table row.
2. **F32 subnormal behaviour.** The Metal record's F32 entry and the compiler's two subnormal dimensions mean the same thing, and the projection is total in one direction: `MetalSubnormalArithmetic::subnormal_mode` maps every Metal behaviour onto the shared vocabulary. The projection must happen exactly once; declaring it twice would put two rows at one phase and is refused by `declare_measured_*_subnormal_behaviour`'s complete-table conflict check.

**Everything else is not an overlap and must not be validated as one.** Language, platform, and deployment minimum have no compiler counterpart. Two equal compiler profiles may legitimately coexist with different nonprojected Metal facts or emission realizations — but only where the difference is explicitly irrelevant to compiler feasibility, and each such fact must still be carried and validated by its own owner, where it continues to bind payload identity.

**A specific warning against a sentence that would be easy to write.** An assessment of the F32 subnormal projection is an assessment of two dimensions of one dtype. It is not an assessment of `MetalTargetFacts`, which also carries a language standard, an artifact family, a deployment minimum, two unmeasured dtype rows, and a binding capacity.

## What remains `Unknown` after this ledger

Three things, and each is `Unknown` in the ADR 0043 sense — neither proved nor disproved — rather than refuted:

1. **Device address width.** No consumer, no authority, no row. Trigger recorded above.
2. **F16 and BF16 on this profile.** Unmeasured under MSL 4.0. BF16 additionally has a *macOS-only* measurement on the older MSL 3.1 row and an iOS-Simulator pipeline-creation refusal; neither reaches this profile, and `spike-bf16-through-the-second-dtype-seams` owns the first non-F32 use of the mechanism and must consume this construction rather than adding a second backend dtype list.
3. **Exact native translation identity.** [ADR 0086](../../decisions/0086-require-attributable-or-attested-native-translation.md), accepted 2026-07-31, decides that native device translation of a metallib during pipeline creation is a typed capability fact whose authority and provenance are `Unknown` on every macOS row currently observable.

The third is not a gap in this ledger. **Every compile-phase row above has its authority.** What is missing is the *runtime* authority that would let a host offer the resulting profile, which is a different question at a different phase, and ADR 0086's own Consequences section states the split precisely: this profile's "quantitative, dispatchability, and F32 numerical rows are unaffected as measurements; what they lack is the applicability authority that would let a host offer the profile."

The consequence for work item 5 of the owning ticket is exact and worth stating here so it is not rediscovered: `tiler_metal::applicability::MetalHostEligibility` holds a `NativeTranslationAuthority` whose one field is a private uninhabited enum, so a positive eligibility receipt is impossible to construct anywhere, including inside `tiler-metal`. A runner that offers this profile *only* from a receipt therefore cannot offer it on any host that exists today, and `evaluate_metal_host_applicability` returns `MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority` even for an observation matching the measured row in every public field. That is the accepted decision applied, not a defect to route around, and the cheaper alternative — treating the matching public environment row as sufficient — is the one ADR 0086 explicitly rejected, on the ground that an opaque translator can change while the observed row stays identical.

## What consumed this ledger

**Fact — the rows are constructed, and by one owner.** `tiler_build::BoundMetalCompileDeclaration` (`crates/tiler-build/src/metal_declaration.rs`) assembles the checked compiler `TargetProfile`, the exact `MetalTargetFacts`, the selected `MetalEmissionRealization` and `NumericalRealization`, the total `MetalTarget` projection, and the structured sources, from exactly the rows above. Its private `LedgerRows` record is the transcription, one field per row, so a mutation test can move one row and observe the descriptor move with it. The profile key is `tiler.metal.macos-apple9.msl4-0.f32.v1` and its canonical descriptor is 1,741 bytes.

The authority classes are carried as this ledger states them, not flattened. The quantitative rows are external normative guarantees under three separately versioned references — the macOS 26.5 SDK dispatch header, the 2025-10-20 feature tables, and the MSL 4.0 address-space chapter — while dispatchability and every numerical row carry one `TargetCompileProfileMeasurementSource` pairing the four offline toolchain components with the execution environment. Absent rows stay absent: no device-address-width row, a `PreparedKernelPreflight` query rather than a workgroup fact, no synchronization row, and no F16 or BF16 row at all.

**Fact — exactly two overlaps are validated.** Compiler buffer capacity is checked no greater than the Metal emission limit, and the F32 subnormal projection runs once through `declare_metal_f32_subnormal_behaviour`. Nothing else is compared: a language standard, artifact family, and deployment minimum have no compiler counterpart, and a test asserts that changing the language standard moves the AOT target while leaving the compiler descriptor byte-identical.

**Fact — the migration landed and the deployment record moved with it.** `accept_or_publish_metal_plan` consumes the declaration and refuses a plan compiled under any other profile before emission, naming the key or the descriptor. Both prototypes now compile, emit, and route under it; neither states a target fact of its own, and both moved from MSL 3.1 / macOS 14.0 to MSL 4.0 / macOS 26.0.

**Measurement — the bounded proof ran on the measured row.** On the Apple M4 Max under macOS 27.0 build 26A5388g, the producer published six members and the runner proved thirty operand cases across them, fused and materialized agreeing bit for bit with the published reference, plus the deep single-member proof over the fail-closed, device-preflight, and post-commit probes.

**Measurement — the production offer path refused, exactly as outcome 3 predicts.** The same run reports `metal.host-applicability.unknown-translation-authority: native-translation-authority is unknown for tiler.metal.host-applicability.macos-27.0-26A5388g-arm64-m4max-apple9.v1`, on a host matching this ledger's execution-environment row in every public field. The envelope route is retained beside it as an explicitly labelled diagnostic — producer-declared equality, not host-earned eligibility — so the runtime machinery keeps being exercised on hardware without making the claim ADR 0086 gates.

## Outcomes

Per the repository's research contract, this record closes with named outcomes rather than open notes.

1. **Contract update, applied.** Every quantitative, index-arithmetic, dispatchability, and F32 subnormal row above has a named authority, an exact validity scope, and a reproducible reference, and the section above names the owner that constructed the bound declaration from exactly these rows and no others.
2. **Explicitly deferred, with a trigger.** The device-address-width row stays absent until a KIR operation consumes it.
3. **Explicitly deferred, with a trigger.** The runtime host offer stays unavailable until one of ADR 0086's three reconsideration triggers supplies the missing authority. No implementation task closes it.
4. **Open question, with the evidence that would close it.** One row — operand permutation — is an `Inference` rather than a `Measurement`, because no retained case isolates it and the emitted attribute strings that isolate its four neighbours have nothing to say about operand order. It closes either by a retained kernel whose result distinguishes an operand order under the exact offline compiler, or by a citation to MSL 4.0's normative statement of what `-fmetal-math-mode=safe` guarantees about it. Until then the row is constructible as stated, carrying the `Inference` label, and must not be quoted as an isolated measurement.

## Reproducible checks

Each command is one line and either reproduces or refutes a claim above.

```sh
# The quantitative rows, from the vendored feature tables.
pdftotext -layout docs/research/apple-targets/sources/apple-metal-feature-set-tables-2025-10-20.pdf - \
  | rg -n '64-bit integer math|Maximum number of entries in the buffer argument|Maximum total threadgroup memory allocation|Maximum threads per threadgroup'

# The grid-axis authority, from the installed SDK.
rg -n 'arbitrarily-sized grid|threadsPerGrid does not have' \
  "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLComputeCommandEncoder.h"
rg -n 'maxTotalThreadsPerThreadgroup' \
  "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLComputePipeline.h"

# The two environments and the measured numerical rows.
cd spikes/apple-targets/results/2026-07-30-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883
rg -n '^(probe|environment)\.' record.tsv
rg -n 'case\.macos\.(multiply_two|multiply_half|materialize)\.safe\.O2\.contract-off\.(results|execution_witness)' record.tsv

# The attribute strings that isolate contraction, reassociation, signed zero, NaN, and infinity.
# Every `safe` row is bare; `relaxed` adds reassoc/nsz/arcp/afn; `fast` adds nnan/ninf.
rg -n 'float_operations' record.tsv | rg -o '(safe|relaxed|fast)\.O2\.contract-off\.float_operations\t.*' | sort -u
```

The feature-table check is a positive check on four rows. The `maxTotalThreadsPerThreadgroup` check is deliberately *not* a source for the workgroup row: it is the evidence that the value lives on a prepared pipeline, which is why the row is a query rather than a fact.
