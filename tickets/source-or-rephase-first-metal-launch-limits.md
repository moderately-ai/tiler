---
id: source-or-rephase-first-metal-launch-limits
title: Source or rephase the first Metal launch limits
status: in-progress
priority: p0
dependencies: []
related: [restore-replayable-apple-compatibility-evidence, prototype-metal-runtime-proof]
scopes: [research/apple-targets, implementation/compiler, implementation/artifact, implementation/runtime, contracts/foundation, contracts/artifacts, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [metal, launch, feasibility, target-profiles]
claimed_from: todo
assignee: codex-root
lease_expires_at: 1785457900
---
## User-visible outcome

The first Metal profile proves the bounded serial-sum grid extent 4 and workgroup size 1 from authority available at the phase that consumes each value, or correctly defers the predicate to live-device and prepared-kernel preflight without inventing a compile-time limit.

## Facts and measurement boundary

**Fact:** `GridAxisThreads` means dispatched thread extent along one grid axis and `WorkgroupThreads` means threads per workgroup. The governed placeholder offers 65,535 and 1; those are internally usable constants, not sourced Metal family facts.

**Fact — positive grid authority:** the macOS 26.5 SDK's `MTLComputeCommandEncoder.h` declares `dispatchThreads:threadsPerThreadgroup:` to dispatch an arbitrarily sized grid and explicitly says the grid need not be a multiple of the threadgroup. `MTLTypes.h` represents every `MTLSize` dimension as `NSUInteger`, so extent 4 is representable on the exact macOS 26 profile. The API is available from macOS 10.13. This supports a deliberately conservative compile-profile offer of 4 from the API contract; it is not an Apple9 hardware maximum and is not derived from the kernel builtin's source type.

**Fact — workgroup authority:** Apple Metal Feature Set Tables (2025-10-20), page 6, report 1,024 as the theoretical Apple9 maximum threads per threadgroup. Footnote 4 on page 8 explicitly directs readers to `MTLComputePipelineState.maxTotalThreadsPerThreadgroup` for the actual compiled-function maximum. The macOS 26.5 SDK's `MTLComputePipeline.h` exposes that value on the prepared pipeline state. `MTLDevice.maxThreadsPerThreadgroup` is a separate live-device per-dimension limit. No inspected primary source states a portable minimum of one thread for every compiled pipeline.

**Fact — declarations are not observations:** MSL 4.0 §1.6.6 and §5.1.3 define `-fmax-total-threads-per-threadgroup`, `[[max_total_threads_per_threadgroup]]`, and `[[required_threads_per_threadgroup]]`. They constrain compilation and launch agreement; they do not observe a device-specific pipeline capability, and pipeline creation can still fail. MSL 4.0 §5.2.3.6 and Table 5.8 define the `ushort`/`uint` spellings of `thread_position_in_grid` and `threads_per_grid` without declaring a general compute-grid capacity.

**Fact — implemented representation:** a compiler `CapabilityFact` remains an observed value, while `QuantitativeCapabilityQueryDeclaration` declares a future exact-entry query with no fabricated bound. The selected plan exposes `PreparedEntryTargetRequirement` values; `tiler-build` carries each whole requirement and exact execution-order entry to the artifact builder; and the builder alone mints the executable directional predicate. Device-free `preflight` still refuses unanswered predicates, while `prepare` yields a non-committable `RoutePreparation` whose consuming resolver can produce `Preflight` only after every exact-entry answer satisfies its requirement.

**Inference:** promoting 1,024 into `CompileProfile`, deriving 65,535 from a `uint`, treating a prepared-pipeline observation as a portable profile fact, or inserting a dummy later-phase numeric bound as a query declaration can admit an infeasible launch. Grid extent 4 survives as a compile guarantee. Workgroup size 1 must remain the requirement side of `1 <= prepared.maxTotalThreadsPerThreadgroup`, with the observed right-hand side bound only after the exact pipeline is prepared.

**Measurement boundary:** a conservative normative minimum may be a compile-profile guarantee only when the primary source says it applies to the exact profile. Device-reported maxima are `LiveDevicePreflight`; `maxTotalThreadsPerThreadgroup` is `PreparedKernelPreflight`; concrete launch validation is later still.

**Measurement — corroboration only:** on the Apple M4 Max host under Xcode 26.6 (17F113), macOS SDK 26.5, and MSL 4.0, a minimal prepared pipeline reported `maxTotalThreadsPerThreadgroup = 1024`; a kernel constrained and required at `(1, 1, 1)` prepared successfully and reported 1. This demonstrates why the theoretical family row and prepared value differ. It is not a portable minimum and must not source `CompileProfile`.

## Reproducible primary-source checks

Run:

```sh
pdftotext -layout docs/research/apple-targets/sources/apple-metal-feature-set-tables-2025-10-20.pdf - | rg -n 'Maximum threads per threadgroup|theoretical maximum number of threads per threadgroup|maxTotalThreadsPerThreadgroup'
pdftotext -layout docs/research/apple-targets/sources/apple-metal-feature-set-tables-2025-10-20.pdf - | rg -ni 'maximum.*grid'
pdftotext -layout docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4-2025-10-23.pdf - | rg -n 'Maximum Total Threadgroup Size Option|max_total_threads_per_threadgroup|required_threads_per_threadgroup|thread_position_in_grid|threads_per_grid'
rg -n 'arbitrarily-sized grid|threadsPerGrid does not have' "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLComputeCommandEncoder.h"
rg -n 'maxTotalThreadsPerThreadgroup' "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLComputePipeline.h"
rg -n 'maxThreadsPerThreadgroup' "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h"
```

The second PDF check is intentionally a negative check and is not the grid proof: it names only object- and mesh-shader grid rows. The positive proof is the `dispatchThreads` API contract above.

## Implementation keys

1. Replace the governed grid placeholder with the conservative API-backed bound 4 and cite the exact authority beside its structured profile source. Do not encode 65,535, `uint` maximum, or an Apple9 hardware maximum.
2. Introduce a typed quantitative query declaration distinct from an observed `CapabilityFact`. It names the capability axis, the earliest truthful phase, a governed target-property key, and the versioned provider that answers it; it carries no fabricated available value. Canonical profile identity must distinguish query declarations from facts and cover every field.
3. Make compile feasibility admit a candidate only when every unresolved requirement has such a query path before `RoutingCommit`. Preserve `Unknown` for an axis with neither an available fact nor a query declaration. Retain the canonical deferred set on the admitted implementation, selected plan, and public borrowed plan view; do not collapse it into `ProvenEvidence`.
4. Have `tiler-build` translate the compiler-minted deferred requirement into the artifact's existing expression vocabulary: `required <= TargetProperty(query_key, phase)`, paired with the exact selected authority. The assembler must receive the compiler's typed view and must not reconstruct the axis, required quantity, phase, key, or authority from strings.
5. In the Metal runtime proof, bind `MTLComputePipelineState.maxTotalThreadsPerThreadgroup` to that exact prepared-kernel property, evaluate every prepared-phase predicate, and refuse before `RoutingCommit` when capacity is below the required workgroup size. Keep `tiler-runtime` device-free; its existing refusal for unanswered deferred predicates remains correct.
6. Update the target descriptor and any public/compiler/artifact identity domains whose encoded subjects change. Recompute every pinned descriptor or identity from the merged tree; do not select an old fixture value.

## Required evidence

Provide one positive bounded serial-sum compile/package/route and negative cases for grid extent 5 and for a prepared-pipeline capacity below required workgroup size. A profile with neither a compile fact nor a query declaration must remain `Unknown`; a query declaration must be `Deferred`, never `Proven`, at compile phase. Phase tests must reject `DeviceRuntime` or `PreparedKernel` evidence inserted at `CompileProfile`, and must reject a prepared-kernel query declared at an earlier phase. Mutation tests must prove that replacing extent 4 with 65,535, replacing the prepared query with theoretical 1,024, omitting the predicate from the artifact, or advancing routing commit before the predicate is answered is detected.

## Evidence obtained

**Measurement:** `cargo nextest run -p tiler-ir`, `cargo nextest run -p tiler-compiler`, `cargo nextest run -p tiler-artifact`, `cargo nextest run -p tiler-build`, `cargo nextest run -p tiler-runtime`, `cargo nextest run -p tiler-prototype-compile`, and `cargo nextest run -p tiler-prototype-run` pass on the ticket worktree. The two-stage runtime prototype proves two equal-key requests remain distinct by exact entry, refuses the second entry when its prepared capacity is zero, and admits the same artifact when both prepared pipelines report sufficient capacity. The offline producer retains both fused and materialized programs for one-row, three-contributor input; its five sidecar operand classes preserve exceptional-value and contraction-sensitive coverage without exceeding the truthful four-thread grid guarantee.

**Measurement:** replacing the governed grid bound 4 with 65,535 makes `the_governed_grid_authority_admits_four_and_refuses_five` fail; replacing the prepared query with a compile-profile fact of 1,024 makes `an_alternative_names_its_capabilities_and_exposes_its_abi_inputs` fail; omitting the compiler-minted deferred set in `tiler-build` makes `a_checked_plan_publishes_then_hits_without_recompiling` fail; and temporarily exposing `RoutePreparation::commit` makes its compile-fail doc-test fail. An off-by-one mutation of the explain emitter's entry also makes the exact subject-entry conformance fixture fail. Restoring the offline producer's former four rows makes the determinism test fail specifically because the materialized nontrivial pointwise stage requires twelve grid threads while the governed profile proves only four; the one-row fixture makes the same test pass without weakening feasibility.

**Measurement boundary:** the Metal host path constructs the exact `MTLComputePipelineState` objects before allocation or routing commit and answers from each pipeline's `maxTotalThreadsPerThreadgroup`. The ordinary package tests exercise the device-free route and a deterministic prepared-capacity fixture. They do not claim that this ticket's new path was executed on Metal hardware; the retained host corroboration above remains the bounded hardware measurement.

## Public boundary ratification

Tom ratified the consuming resolver boundary on 2026-07-30: `DecodedProgram::prepare` returns a non-clonable `RoutePreparation`; the host inspects its routed entries and exact `TargetPropertyRequest` values; `resolve_target_properties` passes each exact request to a callback once and yields the existing `Preflight` only after every answer satisfies its typed requirement; and only `Preflight` exposes the one-way commit. A loose property-key map was eliminated because equal keys can name different prepared entries, a Metal provider registry was eliminated because `tiler-runtime` is device-independent, and direct commit from preparation was eliminated because it permits program work before feasibility is established. A typed out-of-order answer batch remains a compatible future extension if a real asynchronous consumer justifies its identity and matching machinery.

## Closes when

Both launch quantities have exact typed authorities at their real phases; a query declaration and a measured fact are different types; the compiler retains rather than refuses the prepared predicate; the artifact and runtime consume the compiler-minted predicate without re-derivation; the profile and artifact contain no 65,535/1,024 conflation; later facts are checked before routing commit; and the bounded compile/run proof passes or reports the unavailable required environment explicitly.

## Graph maintenance

This ticket blocks `construct-and-bind-the-first-authoritative-metal-compile-profile`. Keep `prototype-metal-runtime-proof` related because it owns the existing pre-commit pipeline check, and keep `restore-replayable-apple-compatibility-evidence` related because compiler acceptance is not a device or pipeline capacity guarantee.
