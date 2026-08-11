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

**Evidence boundary.** This ledger enumerates every fact the bounded serial-sum compile phase consumes and names, per row, the authority that establishes it. It settles what each row may say; it does not itself construct a profile, and constructing one is work items 2 through 5 of the same ticket. Every quantitative row below except the grid axis was read from a primary Apple document vendored in this repository or from the macOS 26.5 SDK headers; the grid axis, every numerical row, and every dispatchability row come from retained measurement directories. Nothing here is transcribed from a second-hand summary, and no row is filled from a nearby dtype, a nearby Apple family, a live device, or a prepared pipeline.

**The grid-axis row changed authority class on 2026-08-04, and the reason is worth reading before the row itself.** A capacity row consumed as a guarantee needs an authority stating a *floor* — what works — and every normative source about grid extent states a *ceiling* on the space instead. No amount of searching converts one into the other, so that row is now a bounded measurement while its five neighbours stay normative. `establish-an-upper-bound-authority-for-the-metal-grid-axis-row` owns the derivation.

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

Both rows are transcribed from `spikes/apple-targets/results/2026-08-02-numerics-covering-apple9-f32-bf16-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`, keys `environment.*` and `probe.*`, run `environment.date_utc 2026-08-02T16:32:20Z`, harness `probe.harness_sha256 17b8b8ddc7731ba1a11f6e971e17cf3fa874ff4153a52de95c634331693a9bb6` at repository base revision `0fcc952ac8f548f462eff6b204386253e65d2522`.

**Lineage of the F32 control retained beneath the 2026-08-02 row.** The 2026-07-31 F32-only record had already replaced the 2026-07-30 one after `close-or-retype-the-operand-permutation-inference` widened the harness by one kernel pair. Those two older environment tables are byte-identical, and so is every case, comparison, and hazard row the 2026-07-30 record carried: the 2026-07-31 record differs from it in exactly 84 added `permutation_chain*` rows and the four provenance rows — date, harness digest, input-manifest digest, and base revision. Both remain retained controls; neither is the source bound by this ledger now.

**Why the 2026-08-02 record can carry the existing F32 rows as well as the new BF16 rows.** It added BF16 beside F32 under the same indivisible `-std=metal4.0` / `air64-apple-macos26.0` profile. Excluding BF16, all 864 covering and 996 exhaustive `case.*` and `comparison.*` rows are byte-identical to the 2026-07-31 MSL 4.0 record. The older record remains retained evidence; the byte comparison is the control that prevents adding a dtype from silently changing the rows already transcribed here.

**The `macos26` in that directory name is the deployment minimum of the offline request, not the host OS version.** The host ran macOS 27.0. A reader reconciling the two should not "correct" either. The same warning is already carried by `crates/tiler-metal/src/applicability.rs`, and it is repeated here because this is the other document a reader arrives at with both numbers in view.

**`metalfe-32023.921` is not in either table, deliberately.** `record.tsv` records it under `environment.family.macos.runtime_compiler` as the build the host loads for `newLibraryWithSource:options:`. Tiler's AOT route supplies no source, so that build is evidence about a comparison path and about nothing this profile compiles. [ADR 0086](../../decisions/0086-require-attributable-or-attested-native-translation.md) item 4 excludes it by name.

## Quantitative rows

Five quantitative axes reach every current scheduled-region proposal, plus one operation-complete arithmetic row. Each row below states the value the profile may offer and the authority that permits it.

### Grid-axis threads — 268,435,456

- **Owner:** compiler `CapabilityAxis::GridAxisThreads`, meaning dispatched thread extent along one grid axis.
- **Authority:** Measurement, not a normative guarantee. [`spikes/target-profiles/metal-grid-axis-extent`](../../../spikes/target-profiles/metal-grid-axis-extent/README.md), run 2026-08-04, dispatched a ladder of grid extents through this profile's own compilation (`-std=metal4.0`, `air64-apple-macos26.0`, offline), its own launch realization (`uint tid [[thread_position_in_grid]]`), and its own dispatch route (`dispatchThreads:threadsPerThreadgroup:` with an `MTLSize`), verifying **every** slot of a poisoned buffer at three threadgroup widths. All 6,294 dispatched rows reached `Completed` and verified; `2^28` is the widest extent verified at every width.
- **Validity:** `MeasuredEnvironment` — the exact offline compilation environment and the exact execution environment tabulated above, together. The extent ladder ran on precisely those, which is why this row shares the profile's existing `TargetCompileProfileMeasurementSource` rather than adding a second context.
- **Phase:** `CompileProfile`, via `TargetCompileProfileMeasurementSource`, whose phase, authority, and validity are fixed by construction and cannot be widened to a portable claim.

**Why this row is measured when the five beside it are normative, and the reason generalizes beyond this row.** The row is consumed as a *guarantee*: physical feasibility admits a plan when its required extent is no greater than the declared bound. Its authority therefore has to state a **floor on capability** — *extents up to N work*. Every normative source available states a **ceiling on the space** instead, and a ceiling forbids declaring more without licensing anything at all. That asymmetry, not a failure to search, is why no document could fill this row, and it is the general shape to check before hunting for an authority for any capacity row.

**What the superseded row said, and why it was replaced rather than corrected.** The row read 4, sourced as a normative fact from the macOS 26.5 SDK: `MTLComputeCommandEncoder.h` declares `dispatchThreads:threadsPerThreadgroup:` as "Enqueue a compute function dispatch using an arbitrarily-sized grid", and `MTLTypes.h` types every `MTLSize` dimension as `NSUInteger`. That contract is true and remains the reason any extent is *expressible* here — but it proves representability of every extent equally, so it distinguished no number, and four was chosen to cover the bounded serial-sum program. `source-or-rephase-first-metal-launch-limits` established it and said so in its own words: "a deliberately conservative compile guarantee rather than a maximum". The SDK reference is no longer carried as this row's source, because a precondition is not a source for a value; it is recorded here instead.

- **Eliminated — an Apple feature-table row.** Verified rather than assumed: the vendored 2025-10-20 tables carry exactly two grid rows, `Maximum threadgroups per object shader grid` and `Maximum threadgroups per mesh shader grid`. Neither is a compute-grid capacity. The `Apple9` column of the second reads 1,024, which is a mesh-shader figure a reader could easily mistake for a compute limit. The exact command is under "Reproducible checks".
- **Eliminated — an SDK sentence bounding `MTLSize` for `dispatchThreads:`.** There is none. The `@discussion` that follows "arbitrarily-sized grid" scopes the phrase precisely — "threadsPerGrid does not have to be a multiple of the  threadGroup size" — which is a statement about divisibility, not magnitude. **Both installed SDKs agree, by byte comparison rather than by reading:** `MTLComputeCommandEncoder.h` is identical in the macOS 26.5 and 27.0 SDKs (SHA-256 `610bcf8f3e6cb6a7067622f4395d8aa292c56226afde457ac6cb902937872b7b`), and `MTLTypes.h` differs by exactly one added blank line at line 106 with the `MTLSize` definition byte-identical. The 27.0 SDK therefore neither adds nor withdraws anything on this row.
- **Eliminated — the one numeric bound the header does carry.** `MTLDispatchThreadsIndirectArguments` types its grid as `uint32_t threadsPerGrid[3]`, so no extent above `2^32 - 1` is expressible in an *indirect* dispatch argument buffer. Tiler encodes the direct route, whose `MTLSize` is `NSUInteger`, so citing the indirect struct would be citing a bound that does not apply to the route this profile's plans take. It would become relevant the day an indirect dispatch route is added, which is the reconsideration trigger for this elimination.
- **Recorded as a ceiling, and it caps what this row may ever say.** MSL 4.0 §5.2.3.6 Table 5.8 lists the corresponding data types for `thread_position_in_grid` as `ushort, ushort2, ushort3, uint, uint2, or uint3` and offers nothing wider; `threads_per_grid` carries the same list and the notes require the two to match. The 4.1 specification (2026-06-04) is unchanged. **No kernel in this language can distinguish more than `2^32` positions along one axis**, so a declared bound above that would be a guarantee the emission cannot keep — which is a correctness bound, not a preference. This is a genuine upper bound and it is still not this row's authority: it says nothing works, only that nothing above it can. The measured `2^28` sits sixteen times below it.
- **Still eliminated:** 65,535 (no inspected source states it); any Apple9 *hardware* grid maximum (no source states one).
- **What the measured number is not.** `2^28` is the extent ladder's stop condition, set by the four-bytes-per-thread cost of verifying every slot and by covering the widest tensor in the project's conformance corpus. Every rung passed, so **nothing measured a failure and nothing here says where one is.** A later run finding a failing extent would narrow this row; this one cannot widen into "the hardware supports exactly this much". Evidence is exhaustive over the integers below 2,049 and sampled above, so the guarantee between two sampled rungs is an interpolation.

**What the moved row admits, checked by compiling rather than by arithmetic (2026-08-05).** `raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells` owns the consequence this row was moved for: the L3 realization profile's six contraction correctness cells were refused by the superseded four-thread row before any plan composed, so the [retained `result_sha256` values](../../../spikes/scheduling/metal_contraction_vertical/README.md) at those cells could not be used as the cross-check they were retained to be. **Measurement** — all six cells (`w_decode_kv` 1x1024x1024, `w_vocab_slice` 1x8192x1024, `w_prefill_q` 10x2048x1024, `w_prefill_mlp_in` 128x3072x1024, `w_prefill_mlp_out` 128x1024x3072, `w_prefill_o` 128x1024x2048) now reach a *selected* physical plan through the ordinary compiler entry point against this declaration, and so does the `2x3x3` shape whose `required: Threads(6)` refusal the owning ticket recorded. `crates/tiler-build`'s `the_measured_grid_axis_admits_every_l3_contraction_cell` is the gate-reachable check, and it carries the refusal half at the boundary in both directions: `16,384 x 16,384` is exactly 268,435,456 output elements and composes, `16,384 x 16,385` refuses on `grid-axis` by name.

**And what it does not establish.** That is *reachability at the compile phase and nothing further*: it dispatches nothing, so it says nothing about executed bits and does not touch a single retained `result_sha256`. A bound admitting an extent, a plan composing at that extent, and a device returning the expected bytes are three different claims, and only the first two are made here. The device comparison at one cell is `publish-an-l3-contraction-cell-through-the-accepted-route`, which needs the two prototype crates `tiler-build` cannot reach.

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

### Synchronization — the workgroup control barrier over threadgroup memory, `Realized`

- **Owner:** compiler `DeclaredSynchronizationRealization`, keyed by the whole `tiler_ir::schedule::SynchronizationSubject` rather than by an axis. `replace-or-justify-the-barrier-count-axis` removed `CapabilityAxis::Barriers` outright and nothing here restores it: a numeric barrier capacity was the wrong model, and what replaced it is an atomic five-dimension subject a target either realizes or does not.
- **Realization:** `threadgroup_barrier(mem_flags::mem_threadgroup)`, which is exactly what `tiler_metal::emit::barrier_call` emits for this subject and what the `cooperative_workgroup_reduction.metal` golden carries.
- **Authority:** Fact for four of the five dimensions and **Inference** for the fifth, from the vendored [MSL 4.0 specification](../apple-targets/sources/apple-metal-shading-language-specification-v4-2025-10-23.pdf) §6.9.1 "Threadgroup and SIMD-Group Synchronization Functions", Table 6.12 and Table 6.13, pages 195–196. Stating one authority for all five would hide which dimension a later source could move.

  | Dimension | Class | Sentence it rests on |
  | --- | --- | --- |
  | `kind` = `ControlBarrier` | Fact | Table 6.12: "All threads in a threadgroup executing the kernel ... need to execute this function before any thread can continue execution beyond the `threadgroup_barrier`", and §6.9.1: it "acts as an execution and memory barrier". |
  | `execution_scope` = `Workgroup` | Fact | Table 6.12, "All threads in a threadgroup". |
  | `visibility_scope` = `Workgroup` | Fact | Table 6.13, `mem_threadgroup`: "ensures the GPU correctly orders the memory operations to threadgroup memory for threads in a threadgroup". |
  | `fenced_spaces` = workgroup only | Fact | The emitted flag is exactly `mem_threadgroup`; `mem_device` is a separate flag this realization does not pass. Declared `device: false` rather than conservatively true, because a superset fence is a different realization with a different cost and the schedule requires the exact derived one. |
  | `ordering` = `AcquireRelease` | **Inference** | No sentence of the specification assigns this barrier an ordering. See the elimination below. |

- **The ordering dimension, and why it is not a quotation.** MSL declares `enum memory_order { memory_order_relaxed, memory_order_seq_cst }` (§6.15.1) — there is no `memory_order_acquire` or `memory_order_release` in the language at all — and it applies that vocabulary to atomics and to `atomic_thread_fence` (§6.15.3), never to `threadgroup_barrier`. What §6.9.1 does say is that the barrier "can also queue a memory fence (for reads and writes) to ensure the correct ordering of memory operations to threadgroup or device memory". The elimination over Tiler's three-value `MemoryOrdering` is therefore: `Relaxed` is **refuted**, because Tiler defines it as establishing no happens-before edge while the quoted sentence orders memory between threads; `SequentiallyConsistent` is **withheld**, because Tiler defines it as a single total order over every participant's fenced effects and the specification reserves that strength for an explicit `memory_order_seq_cst` fence; `AcquireRelease` is what remains and is exactly the quoted content. A future MSL revision that gave the barrier an explicit ordering would convert this row's fifth dimension from an inference to a fact and change nothing else.
- **Validity:** the language standard, MSL 4.0. Not the Apple family — §6.9.1 is a language guarantee, so unlike the feature-table rows this one is not scoped by the `Apple9` column.
- **Phase:** `CompileProfile`. The fact is true of the language, so nothing about a live device or a prepared pipeline is consulted to know it. This is exactly what separates it from the workgroup-threads row above, which is a `PreparedKernelPreflight` query precisely because its value does not exist until a pipeline does.
- **Convergence is the program's obligation and not this row's.** §6.9.1 additionally requires that if the barrier is inside a conditional and any thread executes it, all threads must, and the same for every loop iteration. That is a property of the emitted body rather than of the target, so it is not a dimension of the subject. The emission discharges it structurally: in the retained golden the barrier statement sits at entry-point top level, outside the `if (v6)` guard bracketing both the staged write and the staged read, so no thread reaches it conditionally. `KernelDiagnostic::SynchronizationConvergence` is the verifier rule that refuses a barrier inside a predicated region or a loop body.
- **Eliminated:** deriving the row from the fact that `crates/tiler-metal/goldens/cooperative_workgroup_reduction.metal` compiles and links (compilation success is not a capability fact, and that golden's measurement establishes only that the toolchain accepts the text); deriving it from `simdgroup_barrier`, which Table 6.12 scopes to a SIMD-group and which no governed memory scope can name; and deriving a device-visibility row from the availability of `mem_device`, which is a different subject this profile does not declare.
- **Unreached today, and by what.** No program on this profile can consume this row yet — see "What remains `Unknown`" below. The row is stated because it is true and sourced, on the same principle as the local-memory row above: stating the sourced value is truthful where omitting it would leave a later reader re-deriving the same authority.

## Cost rows

**A cost row is a different kind of row from every one above, and mixing the two is the failure this section exists to prevent.** Every quantitative row in this ledger is a *hard bound* a feasibility predicate reads: silence about one resolves `Unknown`, and an `Unknown` never reaches an executable frontier. That failure direction is right for a bound and wrong for a preference. [The flash-class capability record](../program-planning/flash-class-capability-set.md) already eliminated putting a bandwidth or clock number on a target profile for exactly this reason, and the argument transfers unchanged: a cost row declared as a `CapabilityAxis` would make silence render a profile **unexecutable for a quantity no feasibility predicate reads**. So a cost row is carried apart, silence about it means *no preference* rather than *no plan*, and a profile declaring none encodes byte for byte as it did before the family existed.

`activate-measured-reduction-selection-from-a-target-cost-row` is the ticket Tom accepted on 2026-08-07 that admits the kind. The exact public spelling of its declaration pair is a **reviewed draft boundary** under ADR 0075 and ADR 0074 convention 7 and is not accepted; the model is.

### Saturated parallel fold steps — 1,056, measured

- **Owner:** compiler `TargetProfileBuilder::declare_measured_saturated_parallel_fold_steps`, encoded into the canonical descriptor behind its own domain separator `tiler.target-profile.cost-row.v1`, written only when the family holds a row.
- **Authority:** **Measurement, 2026-08-07** — [`spikes/program-planning/reduction-dispatch-crossover`](../../../spikes/program-planning/reduction-dispatch-crossover/README.md), retained at `results/2026-08-07-apple-m4-max-macos27.0-26A5388g/`, on a host matching this ledger's offline and execution rows in every field. The sweep timed all three reduction strategies over a 92-cell matrix — 276 dispatched alternatives — and fitted a three-parameter work-span model `sum over stages of ( encoder + max(work / P, depth) * step )` on the perfect-square contributor counts. `P` fits at `1.056e3`, and that is the value this row states: the fold steps the device retires at once when saturated.
- **Validity:** `MeasuredEnvironment`, through the same `TargetCompileProfileMeasurementSource` the grid-axis, dispatchability, and numerical rows carry. The same source and not a second one, because the sweep ran on exactly the offline and execution environments those rows were taken on; a second source would claim a population that does not exist.
- **Phase:** `CompileProfile`, which is the phase physical selection reads it at. A row a profile deferred to a later phase resolves `Deferred` and is treated exactly as silence, because a compile cannot wait for it.
- **What the measurement establishes.** The fitted model reproduces the measured verdict on **24 of the 26 held-out cells whose serial-or-parallel verdict is separated**, worst measured penalty 1.81x, median regret 1.0000. The sweep's own perturbation table shows that **only `P` moves a decision**: scaling `encoder` by twenty or `step` by a tenth leaves every predicted winner unchanged, while scaling `P` by a quarter drops held-out agreement to 20 of 26 and the worst penalty to 3.04x.
- **What it does not.** `P` is determined only to **about a factor of four** — quadrupling it holds fit-set agreement and *improves* the held-out worst penalty to 1.20x — so this number positions a contour rather than pinning a constant. Magnitude accuracy is much weaker than decision accuracy (median relative error 0.16 held out, p90 near 0.76), so the model **is a selector and must not be quoted as a latency estimate**. It is a quantity of this host row alone: another Apple family, OS row, dtype, or device declares its own row or declares none.
- **Eliminated:** declaring it as a `CapabilityAxis`, for the failure direction above; declaring the model's other two fitted parameters, because the sweep measures both inert in the decision and because `encoder` prices dispatch count, which the structural cost model already carries as one of its four exact dimensions and would then be priced under two authorities; and inferring a value for any other target from this one.
- **One boundary the consumer inherits, recorded here because it is not visible in the number.** The retained crossover sweep dispatched the single-workgroup tree at `governed_partition`'s balanced split, because `MEASURED_TREE_PARTICIPANT_CAP` landed after it. The compiler now emits the capped width, so at some shapes the tree it dispatches is not the tree that fitted `P`. [The later tree-width excursion](../../../spikes/program-planning/reduction-partition-calibration/README.md#the-excursion-result) measures current production width against every admissible alternative at six non-power-of-two cells and refutes nearest-to-cap distance as a general width model, but it neither re-fits saturated fold steps nor changes this row: width selection and serial-versus-parallel selection remain distinct consumers. [`calibrate-a-shape-aware-tree-width-cost-row`](../../../tickets/calibrate-a-shape-aware-tree-width-cost-row.md) owns the wider width-selection study.

## Dispatchability

### F32 — `Dispatchable`

- **Owner:** compiler `DTypeDispatchabilityFact` keyed by the exact `ResolvedValueType` of `F32`.
- **Authority:** Measurement, not a normative guarantee. The retained MSL 4 run dispatched F32 compute kernels on the macOS/Apple9 execution environment above and read back results, with `probe.dtypes f32 bf16` and `probe.status validated`. Its arithmetic cases carry an `execution_witness` on a non-subnormal operand reporting `status=executed`, which is what separates "the arithmetic ran" from "the kernel was optimized away"; the arithmetic-free materialization cases deliberately record `execution_witness none` and serve only as the round-trip control.
- **Validity:** `MeasuredEnvironment` — the exact offline compilation environment and the exact execution environment tabulated above, together.
- **Phase:** `CompileProfile`, via `TargetCompileProfileMeasurementSource`, whose phase, authority, and validity are fixed by construction and cannot be widened to a portable claim.
- **Inheritance is refused in every direction.** BF16 receives its own row below; F16 remains `Unknown` and may not inherit either measured answer. The `express-metal-honourability-in-the-shared-form` record establishes that the measured Apple row *disagrees* across dtypes — F32 arithmetic flushes where F16 preserves on the same hardware in the same math modes — so inheritance here is not merely unproven, it is known to be unsound in at least one direction. No iOS family, physical or simulated, gains a row from this one.

### BF16 — `Dispatchable`

- **Owner:** compiler `DTypeDispatchabilityFact` keyed by the exact governed `tiler::bf16@1` resolved type.
- **Authority:** Measurement. The unified MSL 4.0 record states `device_bfloat_support supported`, dispatches every admitted BF16 case on this macOS Apple9 row, and records 91 `executed` witnesses in its covering matrix. The arithmetic-free `materialize_bf16` case also runs and returns all eight BF16 payloads unchanged.
- **Validity and phase:** the same exact offline and execution environments above, at `CompileProfile`. This is not an Apple-wide or iOS claim.
- **No inheritance:** F32 remains independently `Dispatchable`; F16 remains absent and therefore `Unknown`.

## Numerical rows

Every numerical row in this profile is a **Measurement** under the exact offline compiler and flags, not a portable normative guarantee. F32 and BF16 each have complete exclusive input/result subnormal tables; the remaining rows are the F32 honourability dimensions the compiler consults for the current caller contract.

**The flags are part of the row.** These facts describe what the *selected numerical realization* delivers through the *exact offline compiler*, so the bound declaration must carry that realization and the measurement source must carry that compiler build. The retained cases this ledger reads are the `safe` math-mode, `contract-off` ones, which is `NumericalRealization::strict_baseline`. A row read from a `relaxed` or `fast` case would be a different fact about a different compilation.

### F32 input subnormals — flush to zero, preserving sign

- **Measurement.** `case.macos.multiply_two.safe.O2.contract-off.results` returns `00000000` for the subnormal operand `00400000` and `80000000` for `80400000`. The sign row is what makes the zero a measured `PreservesSign` rather than an assumed `+0.0`, and the sibling `multiply_half` case agrees.
- **Execution witness.** `case.macos.multiply_two.safe.O2.contract-off.execution_witness` reads `operand=3f800000,expected=40000000,observed=40000000,status=executed`. Without it, `00000000` and "the arithmetic never ran" would be the same observation.
- **Not materialization.** `case.macos.materialize.safe.O2.contract-off.results` returns all eight operands unchanged — `00000001 00400000 007fffff 00800000 80400000 80000000 3eb97ef9 3f800000` — so the flush is a property of arithmetic and not of a buffer round trip. `case.macos.materialize.safe.O2.contract-off.execution_witness` is `none`, correctly: there is no arithmetic to witness.
- **Declared form:** the complete exclusive three-row table — `Preserve` unsupported, `FlushToZero { PreservesSign }` exact, `FlushToZero { AlwaysPositive }` unsupported.

### F32 result subnormals — flush to zero, preserving sign

- **Measurement.** `case.macos.multiply_half.safe.O2.contract-off.results` returns `00000000` for `00800000`, the least positive *normal*, whose halved result is subnormal. That is the result-side dimension isolated: the operand is normal, so only the result can have been flushed.
- **Declared form:** the same complete exclusive three-row table, on the result dimension.

### BF16 input subnormals — flush to zero, preserving sign

- **Measurement.** `case.macos.multiply_two_bf16.safe.O2.contract-off.results` maps `0040` to `0000` and `8040` to `8000`; its ordinary `3f80` witness returns `4000` with `status=executed`.
- **Not materialization.** `materialize_bf16` returns `0001 0040 007f 0080 8040 8000 3eab 3f80` unchanged and records `float_operations none`.
- **Declared form:** the complete exclusive three-row table for the exact `(ArithmeticType::Bf16, tiler::bf16@1)` subject — preserve unsupported, sign-preserving flush exact, always-positive flush unsupported.

### BF16 result subnormals — flush to zero, preserving sign

- **Measurement.** `case.macos.multiply_half_bf16.safe.O2.contract-off.results` maps the least positive normal `0080` to `0000`, while its ordinary `3f80` witness returns `3f00` with `status=executed`.
- **Declared form:** the same complete exclusive three-row table on the result dimension. No F32 subject supplies either BF16 answer.

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

### Reassociation — **both resolutions declared**, each delivered exactly

- **Measurement.** The `reassociation_chain` kernel separates the modes in the results themselves: `safe` returns lane 8 as `3f800000` while `relaxed` and `fast` return `3f800001`. The `safe` case's `float_operations` carries no `reassoc`, and the relaxed and fast cases carry `fadd+reassoc+…`.
- **One observation, two rows, and the second is not a widening of the first.** That single measurement answers two different questions, and the profile declares both answers. Asked *"can a contract that forbids regrouping be delivered here?"* it says yes, because under the selected `safe`/`contract-off` realization the compiler adds none. Asked *"can a contract that permits regrouping be delivered here?"* it says yes **exactly**, and for the same reason: the permission licenses Tiler to choose a grouping, the chosen grouping is what the emitted source expresses, and the target runs that one rather than substituting another. A permitted contract is honoured by delivering some legal grouping, and this target delivers the one Tiler selected.
- **Why declaring both is not the exclusive-table shape.** The two subnormal rows above are complete exclusive tables because a target flushes or preserves and cannot do both. `Forbidden` and `Permitted` name two *caller contracts*, and one non-reassociating target satisfies both at once. `TargetProfileBuilder::governed`'s own honourability table declares both resolutions for contraction and for reassociation, so this follows the established idiom rather than introducing a shape.
- **What the second row is for.** Every parallel reduction strategy — the multi-pass split and the single-workgroup tree alike — regroups the declared contributor sequence, so both are refused by name on a profile answering only `Forbidden`. This row is a precondition for either reaching this target.
- **Unreached today,** for the reason recorded under "What remains `Unknown`" below: no *contract* a caller can name combines this permission with the subnormal flushing this hardware requires.
- **Validity:** the exact offline compiler and the `safe`/`contract-off` selection, exactly as the neighbouring rows.

### Signed zero — forbidden, delivered exactly

- **Measurement.** `nsz` — LLVM's "no signed zeros" relaxation — appears in the `relaxed` and `fast` attribute strings and in neither `safe` string. The results agree: under `safe`, `scale_one_bias_zero` carries `80400000` and `80000000` through a flush and a `+0.0f` bias to `00000000`, which is IEEE round-to-nearest behaviour for `(-0.0) + (+0.0)` rather than a discarded sign.

### NaN and infinity assumptions — no assumption made, delivered exactly

- **Measurement.** `nnan` and `ninf` appear only in the `fast` attribute strings. The `safe` cases this profile compiles under carry neither, so the compiler is making no finite-math assumption.

### Operand permutation — forbidden, delivered exactly

- **Measurement.** The `permutation_chain` and `permutation_chain_reordered` kernels carry the same three contributors — `2**30`, `2.0`, and `-2**30` — in two orders and differ in nothing else. Under `safe`/`contract-off` each keeps three bare `fadd`s and returns its own left-deep fold: `case.macos.permutation_chain.safe.O2.contract-off.results` is `00000000` on all eight lanes, and `case.macos.permutation_chain_reordered.safe.O2.contract-off.results` is `40000000` on all eight. The cancelling pair absorbs the `2.0` when the order is canonical and cancels first when the `2.0` is moved past the negation.
- **Why the pair and not one expression.** A single kernel's value cannot show that its contributors were *not* reordered; a source-permuted twin whose value differs can. That the result lane moves when — and only when — the source order moves is what makes the canonical kernel's `00000000` a preserved order rather than a shape nothing could disturb.
- **Why this is not a second reading of the reassociation row.** ADR 0014 keeps reassociation and contributor permutation as independent permissions, and this pair separates them. The permuted value `40000000` is unreachable by *any* parenthesization of the canonical leaf order: four leaves admit exactly five full binary trees, and `test_the_permutation_probe_is_unreachable_by_reassociating_the_canonical_order` enumerates all five for every operand in the vector and holds `40000000` to being absent from each. Reassociating the canonical order reaches only `00000000` or the operand itself.
- **Execution witness.** Both kernels witness on `80000000`, reading `operand=80000000,expected=00000000,observed=00000000,status=executed` and `operand=80000000,expected=40000000,observed=40000000,status=executed`. Negative zero is the one non-subnormal operand whose result survives the relaxed licence folding the cancelling pair away, so the witness guards against deletion rather than measuring the licence under test.
- **The relaxed modes are a boundary here, not a second data point.** Under `relaxed` and `fast` the canonical chain's `float_operations` is `none` — the licence folds the cancelling pair to zero and then removes the surviving identity add — so those cases return every operand unchanged, witness `not-executed`, and are inadmissible by the guard's first layer. The reordered twin keeps one `fadd+reassoc+…` and returns `40172fdf` and `40400000` on the two ordinary normals, which is `x + 2.0`. "The relaxed modes did not permute" and "the relaxed modes deleted the question" are different claims and only the second is supported.
- **Validity:** the exact offline compiler and the `safe`/`contract-off` selection, exactly as the four rows above.

### Evaluation-order preservation — **absent, and therefore `Unknown`**

- **Owner:** compiler `TargetProfileBuilder::declare_measured_evaluation_order_preservation`, keyed by the exact scalar subject and by `BackendArithmeticLicence` — whether the backend translation is granted licence to rewrite floating-point arithmetic. `safe` withholds it; `relaxed` and `fast` both grant it.
- **Authority:** none *for this row*, and the reason is not that nobody measured it.
- **Why absent.** [Finding 34](../apple-targets/numerical-behaviour.md) measures the property: an emitted two-by-two split is re-serialized under `relaxed` and `fast` on both compilation paths — the two differently-pinned kernels compile to one identical module, so the pin is erased rather than weakened — and every `safe` cell on both paths, at both optimization levels and every contraction setting, returns the written order. It was taken on a **neighbouring toolchain row**: Xcode 27.0 build `27A5228h`, macOS SDK 27.0 build `26A5388f`, and an offline compiler reporting `Apple metal version 32023.921 (metalfe-32023.921)`. Every row this ledger states was taken under Xcode 26.6 build `17F113`, SDK 26.5, and an offline `metalfe-32023.883`. The property is a property *of the backend compiler build*, and finding 8 is the reason that matters here rather than elsewhere: the offline and runtime compilers are different builds that move independently of the OS and of each other. Attaching build `.921`'s behaviour to a profile whose plans build `.883` compiles is the inheritance every other row of this ledger refuses by name, and it would refuse *less* visibly than an absent row, because it would arrive carrying exact provenance.
- **What the absence costs, and what it does not.** Nothing is admitted or refused by this row today: the [oracle derivation](../reference/permitted-divergence-oracle.md)'s refusal class 3 is the consumer, and its derived oracle has no caller yet. What the row does now is make the gap *statable* — `Unknown` is a declared answer a plan-side check can read, where before there was no field to consult and the pin rested on the flags Tiler happens to pass. **The pin Tiler relies on today holds because Tiler asserts `safe`, not because the compiler preserves order**, and this ledger's numerical rows are all scoped to `NumericalRealization::strict_baseline()` — `safe`, `contract-off` — which is exactly the selection finding 34 measures preserving on its own row.
- **Reconsideration trigger — two closing measurements, and each needs an authorization this record cannot give.** Either re-run [the evaluation-order probe](../../../spikes/apple-targets/evaluation-order-probe/README.md) against Xcode 26.6 / `metalfe-32023.883`, which is this profile's row; or re-take this ledger's whole numerical row against Xcode 27.0 / `metalfe-32023.921`, which is finding 34's. **`xcode-select` on this host now points at Xcode 27.0**, so the first needs the toolchain moved back and the second needs every numerical row above re-measured and re-transcribed. Changing a host toolchain component for a measurement changes the evidence environment, which AGENTS.md reserves to Tom; neither is a step a worker takes on its own initiative. A third route closes nothing: a `relaxed` or `fast` row cannot be declared here at all, because this profile compiles under `safe` and a row read from another selection would be a different fact about a different compilation.
- **What must not be substituted.** Finding 17's reassociation row above is *not* this row. It measures a written serial chain over immediates being regrouped, in the direction this compiler canonicalizes *to*; finding 34's split measures the opposite direction, which is the one a physical plan's declared partition travels. The two agree and neither subsumes the other, and the reassociation rows this profile declares — `Forbidden` and `Permitted`, both `Exact` — remain true of the `safe` selection they were read under. What they do not establish is that the backend would still deliver Tiler's chosen grouping under a selection this profile does not make.

## Metal target facts, and which of them project

`MetalTargetFacts` is the emitter's input record. Only some of its fields have a compiler-profile counterpart, and the difference is load-bearing: a field that does not project must never be described as compiler-assessed.

| Metal field | Value for this profile | Projects into the compiler profile? |
| --- | --- | --- |
| `language` | `MslLanguageVersion::Metal4_0` | No — backend-only; it bounds the *validity* of rows above without being one |
| `platform` | `MetalPlatform::MacOs` | No — backend-only artifact family |
| `deployment_minimum` | `MetalDeploymentMinimum::new(26, 0)` | No — backend-only; recorded in emitted provenance and in the target triple |
| `subnormal_arithmetic` (F32 entry) | `FlushesToZero { PreservesSign }` | **Yes** — into both F32 subnormal dimensions |
| `subnormal_arithmetic` (BF16 entry) | `FlushesToZero { PreservesSign }` | **Yes** — into both BF16 subnormal dimensions |
| `buffer_binding_limit` | 31 | **Yes** — into `BufferBindings` |

**Fact.** The deployment minimum here is 26.0, because `probe.fixed_flags -std=metal4.0` and `environment.family.macos.requested_target air64-apple-macos26.0` are the inputs the retained measurement actually used. Reusing the older MSL 3.1 / macOS 14.0 record for this profile would attribute measurements to a compilation that did not produce them. Both prototypes stated that older record until the migration below; neither states any target fact now.

**Selected, not a capability:** `MetalEmissionRealization { launch_index: LaunchIndexRealization::ThreadPositionInGridUInt }`. MSL 4.0 Table 5.8 permits `[[thread_position_in_grid]]` as either `ushort` or `uint`; Tiler selects `uint` and widens explicitly to the governed `uint64_t` index type. It is carried by the translation unit, it affects payload identity, and it proves nothing about grid capacity, arithmetic support, or address width — three `compile_fail` doctests in `crates/tiler-build/src/metal_plan.rs` already pin each of those three negatives.

## Overlaps, and what validating one means

Exactly three facts are stated in both vocabularies, and each must be validated where — and only where — the two mean the same thing.

1. **Buffer capacity.** `MetalTargetFacts::buffer_binding_limit` and `CapabilityAxis::BufferBindings` mean the same quantity. The compiler's offered capacity must be no greater than the Metal emission limit, or the compiler would admit a signature the emitter must then reject. Both are 31 here, from the same table row.
2. **F32 subnormal behaviour.** The Metal record's F32 entry and the compiler's two subnormal dimensions mean the same thing, and the projection is total in one direction: `MetalSubnormalArithmetic::subnormal_mode` maps every Metal behaviour onto the shared vocabulary. The projection must happen exactly once; declaring it twice would put two rows at one phase and is refused by `declare_measured_*_subnormal_behaviour`'s complete-table conflict check.
3. **BF16 subnormal behaviour.** The same total projection applies to the Metal record's independently measured BF16 entry and the exact governed BF16 arithmetic subject. Its two complete tables are transactional and exclusive; deleting the Metal row refuses construction, while substituting the behaviour moves descriptor identity.

**Everything else is not an overlap and must not be validated as one.** Language, platform, and deployment minimum have no compiler counterpart. Two equal compiler profiles may legitimately coexist with different nonprojected Metal facts or emission realizations — but only where the difference is explicitly irrelevant to compiler feasibility, and each such fact must still be carried and validated by its own owner, where it continues to bind payload identity.

**A specific warning against a sentence that would be easy to write.** An assessment of either subnormal projection is an assessment of two dimensions of one exact dtype. It is not an assessment of `MetalTargetFacts`, which also carries a language standard, an artifact family, a deployment minimum, the unmeasured F16 row, and a binding capacity.

## What remains `Unknown` after this ledger

Four things, and each is `Unknown` in the ADR 0043 sense — neither proved nor disproved — rather than refuted. A fifth entry follows them that is **not** `Unknown` but a measured, exactly located blocker, and it is recorded here because it is what a reader looking for "why can this profile not run a parallel reduction" needs.

1. **Device address width.** No consumer, no authority, no row. Trigger recorded above.
2. **F16 on this profile.** Unmeasured under MSL 4.0, absent, and therefore `Unknown`. BF16 is stated only for this macOS profile; both iOS families remain outside it and gain no row by inheritance.
3. **Exact native translation identity.** [ADR 0086](../../decisions/0086-require-attributable-or-attested-native-translation.md), accepted 2026-07-31, decides that native device translation of a metallib during pipeline creation is a typed capability fact whose authority and provenance are `Unknown` on every macOS row currently observable.
4. **Evaluation-order preservation.** Measured by finding 34 and measured on *another toolchain row*, so absent here and therefore `Unknown`. This is the one entry of the four whose gap is a transcription boundary rather than a missing measurement, and its section above states the two closing runs and why each needs Tom's authorization.

**Resolved, and recorded rather than deleted, because the shape of the gap is what makes the result mean anything.** This paragraph used to read "the contract vocabulary admits no parallel reduction on this hardware", and that was true of the four preset contracts: `tiler.strict-f32.v1`, `tiler.flush-f32.v1`, `tiler.relaxed-f32.v1`, and `tiler.reassociate-f32.v1`, of which **none both flushed subnormals and permitted reassociation**. The two granting regrouping were built on the strict reading and required *preserved* subnormals, which the F32 subnormal rows above measure this hardware refusing in every math mode, while the one this hardware can deliver widened subnormals alone. So every parallel reduction strategy was unreachable — not for want of a target fact, but for want of a contract a caller could name — and the compile-side refusal landed on the `InputSubnormals` dimension without ever reaching reassociation.

`compose-the-numerical-contract-from-its-decided-dimensions` closed it: the contract is composed from its dimensions rather than chosen from a preset list, and `NumericalContract::FLUSH_AND_REASSOCIATE_F32` is the named point resolving both. `crates/tiler-build/src/metal_plan.rs`'s `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` is the positive successor of the old activation trigger and drives all three strategies into one portfolio on this profile. Two things this profile carries are what let it: threadgroup memory at 32,768 bytes with the barrier realization, and *both* resolutions of reassociation.

**What blocked afterwards was the grid-axis row, and that is now closed too.** With the contract available, the measurable domain was still one shape, because the row capped the prologue's one-invocation-per-element launch at four. Against the measured row this ledger now states, the retained sweep reports 24 of its 36 shapes retaining all three strategies and **no grid-axis refusal at all**, where it previously reported one and twenty-three.

The third `Unknown` is not a gap in this ledger. **Every compile-phase row above has its authority.** What is missing is the *runtime* authority that would let a host offer the resulting profile, which is a different question at a different phase, and ADR 0086's own Consequences section states the split precisely: this profile's normative quantitative and synchronization rows remain valid in their exact validity scopes; its independently measured F32 dispatchability and numerical rows, plus BF16 dispatchability and subnormal rows, remain valid in their exact measurement boundary. What they lack is the applicability authority that would let a host offer the profile.

The consequence for work item 5 of the owning ticket is exact and worth stating here so it is not rediscovered: `tiler_metal::applicability::MetalHostEligibility` holds a `NativeTranslationAuthority` whose one field is a private uninhabited enum, so a positive eligibility receipt is impossible to construct anywhere, including inside `tiler-metal`. A runner that offers this profile *only* from a receipt therefore cannot offer it on any host that exists today, and `evaluate_metal_host_applicability` returns `MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority` even for an observation matching the measured row in every public field. That is the accepted decision applied, not a defect to route around, and the cheaper alternative — treating the matching public environment row as sufficient — is the one ADR 0086 explicitly rejected, on the ground that an opaque translator can change while the observed row stays identical.

## What consumed this ledger

**Fact — the rows are constructed, and by one owner.** `tiler_build::BoundMetalCompileDeclaration` (`crates/tiler-build/src/metal_declaration.rs`) assembles the checked compiler `TargetProfile`, the exact `MetalTargetFacts`, the selected `MetalEmissionRealization` and `NumericalRealization`, the total `MetalTarget` projection, and the structured sources, from exactly the rows above. Its private `LedgerRows` record is the transcription, one field per row, so a mutation test can move one row and observe the descriptor move with it. The F32-only endpoint used key `tiler.metal.macos-apple9.msl4-0.f32.v1` and a 1,963-byte descriptor. Adding independently sourced BF16 content creates the truthful key `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` rather than revising a key whose `.f32` component would become false, and the descriptor was then 2,149 bytes. The lengths are pinned by `the_declared_profile_states_one_barrier_realization`; the exact key and descriptor both feed `TargetProfileRef`. At the BF16 landing, the standard Metal artifact identity moved from `3daf11256423c683a75f6aeb6b1e3578b1425d46e0899664ab5df156ca600db6` to `949841c610fef13473e4a4d14ee57a62b39ba09c5ed27a9c7ff16679853827d1`, and its cache subject moved from `0d09c0da9db85c70bb2270cbed3a67859b7718b07c605c45ca5d1a9f6adfa905` to `3bc5f57f3b3e2e07849a3830ec56a89e4332245685fa23c9db4da8a4f71c34d0`. The later canonical-occurrence identity step was recomputed on its merged tree and moved those pins again to artifact identity `124981346c0bd593f19154f7ec3df26588179e0c7b446a995bbe4a7a92ba25bd` and cache subject `94dfde30611c9021da8e4a71f9b6824f3af1ff09ec68daa4c65d05bfc63e6370`; that movement did not revise the BF16 measurement or its historical transition.

**The grid-axis authority step, 2026-08-04, moved the same identities again.** The key is unchanged — the profile still states the same two dtypes over the same macOS Apple9 MSL 4.0 row — but the descriptor is now **1,999 bytes**, and the direction is informative. The bound is a fixed-width `u64` in the encoding, so its value moves no bytes at all; what moved is the *source table*. The macOS SDK dispatch reference was the grid row's only user, so retiring it removed one whole `external_guarantee` record, while the measured source the row joined was already carried by the dispatchability and numerical rows and cost nothing to share. Recomputed on that branch alone, the standard Metal artifact identity was `3f98afa59d9ef46999acc211f2153a7d194444f5be3d0dd946f4128b57674a69` and its cache subject `8bca5e7825cdd1dc37da5135b0ea7d6dbd3e9ce1557097f2ee9e60e79fe23d07`; both were recomputed again when that branch composed with its sibling at integration, which is the rule this ledger's next paragraph exists to keep visible. Those two pins and the descriptor length are the complete set that step moved; `raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells` depends on this ticket precisely so the same row is not stepped twice from two bases.

**What those pins are today, and why they moved without a row above moving.** The values live in `crates/tiler-build`'s `the_standard_metal_path_publishes_its_recorded_identities`, which is their authority; this paragraph mirrors it so a reader comparing this document against the source finds the same numbers. The standard Metal artifact identity is `7a2bfe51619c05a13fe86cd973e1dfa85c7353da33e4e75af0531068b774357d`, its cache subject is `8bdcde644d7df6d4ca95736f445a011b2d163efdfb3ba93a5c0a954d139b1aa2`, the published envelope's fixed content is 65,294 bytes, and the canonical descriptor is **2,099 bytes** — unmoved, because the most recent step touched the index-region encoding rather than this profile.

**The 2026-08-07 `tiler.index-region.v11` step moved the first three of those and no row of this ledger.** `bound-a-symbolic-index-coefficient-interval-from-its-declared-extent` admits a proof reading the region's shape environment — a symbolic index coefficient is now bounded from its declared extent, as a symbolic divisor already was — and retains an `IndexDomainFactSource` on every discharged index-domain assessment naming whether the argument that closed it read that environment. The tag is one byte, appended unconditionally so the slot is fixed-width, so every region carrying a discharged predicate re-encodes. This fixture is one of them **even though it names no symbol at all**: its every new tag reads `Program`, which is precisely the claim the tag exists to make legible. The form of the delta is encoding-predicted and its count is *read off the move rather than derived from the program*, which is one notch weaker than the two steps above and is stated rather than blurred: 65,242 + 52 = 65,294, a whole number of single bytes with no residue, which is the evidence no layout moved. The descriptor is untouched at 2,099 bytes, which is the check that this was an index-region step and not a profile one.

**The most recent movement is the first one a row of this ledger caused since the grid-axis step**, and it is the cost row above. Declaring it lengthens the canonical descriptor by exactly 100 bytes — the cost-row section's length-prefixed 33-byte domain separator (41), a row count (8), the length-prefixed 34-byte row key (42), a fixed-width `u64` value (8), and a one-byte compact source index — with **no source-table growth**, because the row shares the measured source three other row families already carry. The published envelope embeds that descriptor seven times, so its fixed content grew by exactly 700: 64,542 + 700 = 65,242. **That the arithmetic closes is the evidence no layout moved.** A profile declaring no cost row encodes byte for byte what it encoded before the family existed, which `the_declared_profile_states_the_measured_cost_row` drives in both directions.

Every movement between the grid-axis step and the values that step superseded — artifact identity `23c46a19f6bc601d35bf4ca653e890372da3079b1bb60526220dc3b3221dcdd0`, cache subject `e89c4d826149c9d103e2ed8392968c0c519df454e23e7793932bc33bc86b1595`, fixed content 64,542 bytes, descriptor 1,999 bytes — was owned elsewhere and revised no row in this ledger: the `tiler.kernel-program` v9, v10 and v11 folds, the executable-coverage fold, the emitted provenance header widening from `every f32 immediate` to `every floating-point immediate`, which is artifact *content* rather than any domain's grammar and moved the fixed content by exactly the eleven bytes the sentence grew, and then **two steps that landed on the same day and had to be composed on the merged tree rather than taken from either branch**: the `tiler.index-region.v10` step, which makes a linear combination's coefficient a tagged `SourcedIndexInteger` and adds one byte per coefficient this fixture's regions spell, and the compiler's per-locus obligation derivation, which founds each delivered-realization obligation on the operation realizing its occurrence and so emits eleven rows for this fixture's program where it used to emit twenty, taking exactly 180 bytes out at the twenty bytes an obligation row encodes to. The two directions partly cancel and the arithmetic closes exactly — 64,710 + 12 − 180 = 64,542 — which is why neither branch's separately computed values appear here: each was correct against its own base and neither survived the merge. The pin's own doc comment carries that ladder with each superseded row, and is the record to reconcile an older value against.

The authority classes are carried as this ledger states them, not flattened. The five normative quantitative rows and the synchronization row are external guarantees under **three** separately versioned references — the 2025-10-20 feature tables, the MSL 4.0 address-space chapter, and the MSL 4.0 threadgroup-synchronization section — while the grid-axis row, dispatchability, and every numerical row carry one `TargetCompileProfileMeasurementSource` pairing the four offline toolchain components with the execution environment. The macOS SDK dispatch reference was the fourth until the grid row became measured, and it is deliberately not retained as a second source on that row: two sources on one row would let a reader take the normative one as licensing the number. The barrier row has a reference of its own rather than sharing the address-space one, so a reader repairing a stale synchronization row is sent to §6.9.1 rather than to the chapter establishing the `device` address space. Absent rows stay absent: no device-address-width row, a `PreparedKernelPreflight` query rather than a workgroup fact, and no F16 row.

**Fact — exactly three overlaps are validated.** Compiler buffer capacity is checked no greater than the Metal emission limit; the F32 subnormal projection runs once through `declare_metal_f32_subnormal_behaviour`; and the declaration privately projects the independently measured BF16 row through the same shared mode conversion without widening the ratified public F32 boundary. Nothing else is compared: a language standard, artifact family, and deployment minimum have no compiler counterpart, and a test asserts that changing the language standard moves the AOT target while leaving the compiler descriptor byte-identical.

**Fact — the migration landed and the deployment record moved with it.** `accept_or_publish_metal_plan` consumes the declaration and refuses a plan compiled under any other profile before emission, naming the key or the descriptor. Both prototypes now compile, emit, and route under it; neither states a target fact of its own, and both moved from MSL 3.1 / macOS 14.0 to MSL 4.0 / macOS 26.0.

**Measurement — the bounded proof ran on the measured row.** On the Apple M4 Max under macOS 27.0 build 26A5388g, the producer published six members and the runner proved thirty operand cases across them, fused and materialized agreeing bit for bit with the published reference, plus the deep single-member proof over the fail-closed, device-preflight, and post-commit probes.

**Measurement — the production offer path refused, exactly as outcome 3 predicts.** The same run reports `metal.host-applicability.unknown-translation-authority: native-translation-authority is unknown for tiler.metal.host-applicability.macos-27.0-26A5388g-arm64-m4max-apple9.v1`, on a host matching this ledger's execution-environment row in every public field. The envelope route is retained beside it as an explicitly labelled diagnostic — producer-declared equality, not host-earned eligibility — so the runtime machinery keeps being exercised on hardware without making the claim ADR 0086 gates.

## Outcomes

Per the repository's research contract, this record closes with named outcomes rather than open notes.

1. **Contract update, applied.** Every quantitative, index-arithmetic, dispatchability, F32 subnormal, and BF16 subnormal row above has a named authority, an exact validity scope, and a reproducible reference, and the section above names the owner that constructed the bound declaration from exactly these rows and no others.
2. **Explicitly deferred, with a trigger.** The device-address-width row stays absent until a KIR operation consumes it.
3. **Explicitly deferred, with a trigger.** The runtime host offer stays unavailable until one of ADR 0086's three reconsideration triggers supplies the missing authority. No implementation task closes it.
4. **Closed by a bounded experiment, after every normative route was eliminated.** The grid-axis row was a conservative compile guarantee whose cited authority licensed no number. `establish-an-upper-bound-authority-for-the-metal-grid-axis-row` eliminated the feature-table route (no compute-grid row exists), the SDK-sentence route (the header bounds nothing, in either installed SDK), and the indirect-argument route (its `uint32_t` grid binds a dispatch route Tiler does not encode), and identified MSL Table 5.8's `uint` typing as a genuine ceiling at `2^32` that still cannot fill a row consumed as a floor. The row is now a `Measurement` at 268,435,456 from [the retained extent ladder](../../../spikes/target-profiles/metal-grid-axis-extent/README.md), with its exact procedure, its two failure-injection controls, and its stop condition recorded there. **Reconsideration trigger:** a normative Apple statement of a compute-grid maximum, an indirect dispatch route reaching this profile, or any measured failing extent — each would move this row, and the first two would move its class.
5. **Closed by the retained kernel, after the citation route was eliminated.** The operand-permutation row was an `Inference` because no retained case isolated it, and this outcome named two things that would close it. `close-or-retype-the-operand-permutation-inference` attempted the cheaper one — the MSL citation — first, and eliminated it: the vendored MSL 4.0 and 4.1 specifications contain no normative statement about operand order at all, and the sentence that comes closest is refuted on this very row by evidence already in this ledger. The derivation is recorded under "The route this row did not close by" below, so a later reader can refute the elimination rather than only the conclusion. The row is now an isolated `Measurement` beside its four neighbours, established by the retained `permutation_chain`/`permutation_chain_reordered` pair, and the numerical section above states it as one. Every numerical row on this profile is now a measurement under the exact offline compiler; none is an inference.

6. **Explicitly deferred, with a trigger, and the row exists to hold the deferral.** Evaluation-order preservation has a compiler vocabulary as of `declare-evaluation-order-preservation-in-the-target-profile`, and this profile declares no row in it, so it answers `Unknown` for every subject and licence at every phase. `crates/tiler-build`'s `the_declared_profile_answers_unknown_on_evaluation_order_preservation` is the gate-reachable check, and it asserts the negative at `LaunchPreflight` so a row declared at any phase would break it. **Reconsideration trigger:** either closing measurement named in the row's own section — the evaluation-order probe re-run against `metalfe-32023.883`, or this ledger's numerical row re-taken against `metalfe-32023.921` — each of which moves a host toolchain component and therefore needs Tom's authorization.

## The route this row did not close by

Recording the eliminated route, because the next reader tempted by it needs the refutation and not only the conclusion.

**Fact — the vendored specifications say nothing about operand order.** Neither `apple-metal-shading-language-specification-v4-2025-10-23.pdf` nor `apple-metal-shading-language-specification-v4.1-2026-06-04.pdf` contains the string `operand order`, any occurrence of `commut`, or any occurrence of `evaluation order` or `order of evaluation`, case-insensitively, in their extracted text. The exact command is in "Reproducible checks" below; it prints `0` for each document, and printing nothing is what this claim consists of.

**Fact — what the MSL 4.0 specification does say.** §1.6.3 (page 15) enumerates six fast-math relaxations — no NaNs, no INFs, no signed zeroes, allow reciprocal, allow reassociation, allow contract — and describes the strictest mode as: "If you set the option to safe, it disables unsafe floating-point optimizations by preventing the compiler from making any transformations that might affect the results. This sets the FP contract to on."

**Inference — that sentence cannot be quoted as this row's normative authority, and the refutation is in this ledger.** Read strongly, "any transformations that might affect the results" is a universal claim, and it is false of the compilation this profile actually consumes: under `safe` the emitted module declares `air.compile.denorms_disable`, and the F32 subnormal rows above measure that declaration changing results. A sentence whose universal quantifier the retained evidence already contradicts on this exact toolchain is not evidence of the strength `external_guarantee` asserts. Read narrowly — as scoped to the six enumerated relaxations — it establishes only that the enumerated relaxations are off, and absence from a list of things a *relaxed* mode enables is not a statement about what the strict mode *guarantees*. Neither reading yields a normative operand-order guarantee, so the row is sourced as a measurement.

## Reproducible checks

Each command is one line and either reproduces or refutes a claim above.

**The three commands that read an Apple SDK header resolve against whatever SDK `xcrun` selects, which is not the SDK this ledger was recorded against.** The offline compilation environment above names `macosx` 26.5, build `25F70`; that row is a dated measurement this record keeps rather than re-bases. Measured on this host 2026-08-07: `xcrun --sdk macosx --show-sdk-path` resolves to `/Applications/Xcode-beta.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX27.0.sdk`, version 27.0, build `26A5388f` — the same SDK build the evaluation-order row above records finding 34 being taken on, and for the same reason it gives: `xcode-select` on this host now points at Xcode 27.0. So a reader running the block below reads the **27.0** headers, not the 26.5 ones. What that costs is measured rather than assumed, and it is not the same for the two headers involved:

- **`MTLComputeCommandEncoder.h` is byte-identical across the two SDKs**, so the two commands reading it print at 27.0 exactly what they printed at 26.5 — the `arbitrarily-sized grid` abstract and its divisibility `@discussion` at lines 240–241, and the indirect-argument struct at line 34, on both. This is the one already carried as SHA-256 `610bcf8f3e6cb6a7067622f4395d8aa292c56226afde457ac6cb902937872b7b` on the grid-axis elimination above, and the digest check in the block runs it on the reader's host rather than asking for it on trust.
- **`MTLComputePipeline.h` is not identical**: 305 lines at 26.5 against 297 at 27.0, digests `8f194e26c3df43a8787edc1aa6898f7156f065fb21bf043629d7e5227865c9aa` and `1b30d5dbf85c6ae007fb5b5c2a5194fce225d0afcf01fd02d5600d8660f9e3b5`. **Measured by `diff` rather than inferred from the digests: the entire difference is documentation-comment prose about reflection information, plus two blank lines.** No declaration is added and none is withdrawn, and `maxTotalThreadsPerThreadgroup` is declared identically in both. What moves is only where the command reports it — lines 52, 53, 55, 227 and 230 at 26.5 against 52, 53, 55, 217 and 220 at 27.0, the ten-line shift the deleted comment block accounts for.

**So the divergence costs this block nothing, and that is a measured result rather than the absence of a check.** It is stated because the two facts a reader needs to know it — that one header is covered by the digest check and the other is not, and that the uncovered one is the one that actually differs — are the two facts the block did not carry. The workgroup-threads row's evidence in particular survives the selection: it is evidence that the value lives on a prepared pipeline, and both SDKs declare the property on `MTLComputePipelineState`.

```sh
# The quantitative rows, from the vendored feature tables.
pdftotext -layout docs/research/apple-targets/sources/apple-metal-feature-set-tables-2025-10-20.pdf - \
  | rg -n '64-bit integer math|Maximum number of entries in the buffer argument|Maximum total threadgroup memory allocation|Maximum threads per threadgroup'

# The synchronization row, from the vendored MSL 4.0 specification. The first
# prints Table 6.12's barrier contract and Table 6.13's `mem_threadgroup` flag;
# the second prints the two-value `memory_order` enumeration that is the whole of
# the ordering elimination, and printing exactly `memory_order_relaxed` and
# `memory_order_seq_cst` is what shows no acquire/release spelling exists.
pdftotext -layout docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4-2025-10-23.pdf - \
  | rg -n -A6 'Table 6.12. Synchronization compute function|mem_threadgroup  *The flag ensures'
pdftotext -layout docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4-2025-10-23.pdf - \
  | rg -n -A4 'enum memory_order'

# What the SDK does and does not say about grid extent. The first prints the
# "arbitrarily-sized grid" abstract and the divisibility discussion that scopes
# it; the second prints the indirect-dispatch struct, whose `uint32_t` grid is
# the only numeric bound in the header and binds a route Tiler does not encode.
#
# These three read whatever SDK `xcrun` selects, which on this host is 27.0
# (build 26A5388f) and not the 26.5 / 25F70 row these were recorded against.
# The paragraph above this block states what the difference costs; the digest
# and `diff` checks below are how it was measured, on both headers.
rg -n 'arbitrarily-sized grid|threadsPerGrid does not have' \
  "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLComputeCommandEncoder.h"
rg -n -A3 'MTLDispatchThreadsIndirectArguments' \
  "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLComputeCommandEncoder.h"
rg -n 'maxTotalThreadsPerThreadgroup' \
  "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLComputePipeline.h"

# What the two installed SDKs do and do not agree on, over both headers the
# commands above read, by byte comparison rather than by reading. Prints four
# digests, dispatch then pipeline for each SDK in turn: lines 1 and 3 match and
# lines 2 and 4 do *not*. That second pair is why this block states its SDK
# provenance above rather than leaving the selection latent.
for sdk in /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk \
           /Applications/Xcode-beta.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX27.0.sdk; do
  shasum -a 256 "$sdk/System/Library/Frameworks/Metal.framework/Headers/MTLComputeCommandEncoder.h"
  shasum -a 256 "$sdk/System/Library/Frameworks/Metal.framework/Headers/MTLComputePipeline.h"
done

# What that second difference consists of, which is what decides it costs the
# block nothing: reflection-related documentation comments and two blank lines,
# declaring nothing new and withdrawing no declaration. Prints only `///` and
# blank-line hunks -- no line beginning `@property` or `NSUInteger` appears.
diff /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk/System/Library/Frameworks/Metal.framework/Headers/MTLComputePipeline.h \
     /Applications/Xcode-beta.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX27.0.sdk/System/Library/Frameworks/Metal.framework/Headers/MTLComputePipeline.h

# The MSL language ceiling on the addressable grid, and the counted absence of
# anything wider than `uint`. The second command prints `0` per specification.
pdftotext -layout docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4-2025-10-23.pdf - \
  | rg -n -A6 'thread_position_in_grid  *ushort,'
for pdf in docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4*.pdf; do
  pdftotext -layout "$pdf" - \
    | rg -c -i 'ulong.*thread_position_in_grid|thread_position_in_grid.*ulong|uint64.*thread_position_in_grid' \
    || echo "0 $pdf"
done

# The grid rows the feature tables actually carry: object- and mesh-shader
# grids, and no compute grid.
pdftotext -layout docs/research/apple-targets/sources/apple-metal-feature-set-tables-2025-10-20.pdf - \
  | rg -n -i 'grid'

# The measured grid-axis row itself.
cd spikes/target-profiles/metal-grid-axis-extent
DEVELOPER_DIR=/Applications/Xcode.app cargo run --release

# What that row admits: the six L3 contraction cells reach a selected plan, and
# one output element past the bound still refuses on `grid-axis` by name.
cargo nextest run -p tiler-build \
  -E 'test(the_measured_grid_axis_admits_every_l3_contraction_cell)'

# The two environments and the measured numerical rows.
cd spikes/apple-targets/results/2026-08-02-numerics-covering-apple9-f32-bf16-unified-msl4-macos26-xcode26.6-metal32023.883
rg -n '^(probe|environment)\.' record.tsv
rg -n 'case\.macos\.(multiply_two|multiply_half|materialize)\.safe\.O2\.contract-off\.(results|execution_witness)' record.tsv

# The attribute strings that isolate contraction, reassociation, signed zero, NaN, and infinity.
# Every `safe` row is bare; `relaxed` adds reassoc/nsz/arcp/afn; `fast` adds nnan/ninf.
rg -n 'float_operations' record.tsv | rg -o '(safe|relaxed|fast)\.O2\.contract-off\.float_operations\t.*' | sort -u

# The permutation pair: three bare adds each, `00000000` for the canonical order
# and `40000000` for the source-permuted twin, both with an executed witness.
rg -n 'case\.macos\.permutation_chain(_reordered)?\.safe\.O2\.contract-off\.' record.tsv
```

The eliminated normative route, from the repository root. The first command prints
`0` once per vendored specification — a counted absence rather than a silent one,
so "no match" and "the command did not run" are distinguishable. The second prints
the sentence that comes closest, and that the F32 subnormal rows above refute as a
universal claim.

```sh
for pdf in docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4*.pdf; do
  pdftotext -layout "$pdf" - | rg -i -c 'operand order|commut|evaluation order|order of evaluation' || echo "0 $pdf"
done

pdftotext -layout docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4-2025-10-23.pdf - \
  | rg -n -A2 'If you set the option to safe'
```

The feature-table check is a positive check on four rows. The `maxTotalThreadsPerThreadgroup` check is deliberately *not* a source for the workgroup row: it is the evidence that the value lives on a prepared pipeline, which is why the row is a query rather than a fact. It is also the one command in the block whose header is not byte-identical across the two installed SDKs, and the paragraph introducing the block states what that does and does not move.
