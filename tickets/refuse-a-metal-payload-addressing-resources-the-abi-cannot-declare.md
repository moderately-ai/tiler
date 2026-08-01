---
id: refuse-a-metal-payload-addressing-resources-the-abi-cannot-declare
title: Refuse a Metal payload addressing resources the artifact ABI cannot declare
status: todo
priority: p3
dependencies: [validate-metal-payload-argument-slots-against-declared-bindings]
related: []
scopes: [implementation/candle]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [candle, runtime, artifacts]
---
ADR 0090 item 8's third obligation is discharged for buffer arguments by `validate-metal-payload-argument-slots-against-declared-bindings`. This is the remainder it named rather than absorbed.

## User-visible outcome

A `metallib` whose kernel addresses a texture, sampler, or threadgroup resource — none of which the artifact ABI can declare — is refused before the routing commit, rather than being prepared and dispatched with that resource left unbound.

## Why this is open

**Fact — the comparison that landed counts buffers only.** `prepare_pipeline_with_reflection` filters the reflection to `MTLBindingType::Buffer`, and `declared_transport_slots` reads `RoutedBinding::transport_slot`, which is a `[[buffer(N)]]` index. That filter is correct as far as it goes: threadgroup rows are numbered in the disjoint `[[threadgroup(N)]]` namespace, so counting them against buffer slots would refuse a correct object.

**Fact — the artifact ABI models no other resource kind.** `tiler-metal`'s emitter produces `[[buffer(N)]]` parameters plus launch builtins (`emit.rs` `parameter_declaration` / `builtin_declaration`), and `RoutedBinding` carries a transport slot and nothing else. So a reflected texture, sampler, or threadgroup row has no declared counterpart to disagree with.

**Fact — that gap is inside the threat model, not outside it.** The objects this check exists to catch are exactly the ones Tiler's emitter did not produce. An object whose buffer arguments happen to match the declaration but which additionally addresses a texture passes today, and the encoder never binds it — a kernel reading an unbound resource rather than a refusal.

It has not bitten because every object this profile routes comes from Tiler's own emitter, which cannot emit one. That is a reason rather than a guarantee, and it is the same reason the buffer half was open.

## Closes when

- The adapter refuses, before the routing commit, a prepared entry whose reflection reports any binding the artifact ABI cannot declare, under a typed class distinct from a buffer-slot disagreement.
- The refusal names the resource kind and index it found, so a reader can tell a texture from a threadgroup allocation.
- It is watched failing against a real object that addresses such a resource — which needs a hand-written MSL kernel compiled to a `metallib` outside the emitter, since the emitter cannot produce one. If building that object on the qualified row proves impractical, that is recorded as a measurement with its exact procedure and the check is landed with its evidence boundary stated.
- Threadgroup rows are decided explicitly rather than by omission: either they are refused with the rest, or the reason a compiled kernel may legitimately carry one is recorded.

## Outcome

Every closing condition is met. `prepare_pipeline_with_reflection` now classifies every reflected row before it derives the buffer table, and refuses under `RouteRefusal::UndeclarableBindings` — a class distinct from `ArgumentSlotsDisagree` — naming each offending row's resource class *and* index.

**Fact — the classes are enumerated, with no wildcard acceptance.** `reflected_binding_class` maps all twelve constants `objc2-metal` 0.3.2 declares on `MTLBindingType` (`Buffer` 0, `ThreadgroupMemory` 1, `Texture` 2, `Sampler` 3, `ImageblockData` 16, `Imageblock` 17, `VisibleFunctionTable` 24, `PrimitiveAccelerationStructure` 25, `InstanceAccelerationStructure` 26, `IntersectionFunctionTable` 27, `ObjectPayload` 34, `Tensor` 37) onto a typed `ReflectedBindingClass`, and binds the raw code of anything else as `Unnamed(code)`. The binding models `MTLBindingType` as a `#[repr(transparent)]` newtype over `NSInteger` with associated constants rather than as a Rust enum, so an exhaustive wildcard-free match is not expressible — the same shape `submission_outcome` records for `MTLCommandBufferStatus` — and the final arm is therefore fail-closed rather than accepting. `ReflectedBindingClass::is_declarable` is the single authority that only `Buffer` is declarable, and both derivations from a reflection read it.

**Fact — threadgroup rows are refused with the rest, and the reason a kernel may carry workgroup memory is recorded.** `tiler-metal`'s `address_space_declaration` refuses `AddressSpace::Workgroup` outright, so the emitter cannot produce a `[[threadgroup(N)]]` parameter; a reflected one would need `setThreadgroupMemoryLength:atIndex:`, which the artifact ABI cannot state and this adapter never calls, so the kernel would address a zero-length allocation. **Measurement** (Apple M4 Max, macOS 27.0 build 26A5388g, `air64-apple-macos26.0` / `metal4.0`, Xcode `metal`/`metallib` as resolved by `xcrun --sdk macosx`): a `threadgroup float scratch[4]` declared *inside* a kernel body produces **no** binding row — the pipeline prepared addressing buffer argument(s) `[0]` — so refusing threadgroup rows refuses the dynamically sized argument form only, not workgroup memory as such.

**Measurement — watched failing against real objects.** `probe_undeclarable_resources` compiles three hand-written MSL kernels through the same `tiler-metal-aot` driver and the same authoritative target the producer compiles with, then loads and prepares each through the exact `load_library` / `prepare_pipeline_with_reflection` a route takes. On the row above:

- `an object declaring a threadgroup allocation inside the kernel body`: prepared, addressing buffer argument(s) `[0]` — the accepted neighbour, without which the two refusals would be indistinguishable from a check that refuses every compiled object.
- `an object addressing a texture and a sampler`: `candle-metal.prepare: entry 0's "tiler_probe_kernel" addresses texture at index 0, sampler at index 0, and the artifact ABI declares buffer arguments and nothing else, so this consumer would bind nothing for it`. Its buffer half (`[[buffer(0)]]`) agrees exactly, which is the shape of the gap this ticket names.
- `an object taking a threadgroup memory argument`: refused likewise, naming `threadgroup memory at index 0`.

A refusal must arrive under `UndeclarableBindings` specifically; anything else is `ProofError::ProbeMisclassified` rather than counted as evidence. A toolchain that will not compile a probe prints `NOT MEASURED` with the exact `xcrun` procedure and flags rather than failing the run.

**Perturbation.** With `ReflectedBindingClass::is_declarable` forced to `true`, the proof failed at the texture object (`the probe accepted an object addressing a texture and a sampler, and it must be refused; the check that was supposed to say no did not`, exit 1); reverted, the run is green. The hardware-free half is `only_a_buffer_binding_is_a_class_the_abi_declares` (every named constant plus an unnamed one, distinct renderings, only `Buffer` declarable) and `a_reflected_table_is_declarable_or_names_every_row_that_is_not` (empty and buffer-only tables admitted; a texture, sampler, threadgroup, and unnamed row each named beside buffer rows that are not).

**Hardware re-run**: 20 case(s) agreed across 4 of 6 published members, unchanged.

`implementation/cargo-lock` was added to this ticket's shared scopes: the probe compiles through `tiler-metal-aot`, and placing that dependency edge on `prototypes/candle-metal-adapter` necessarily edits `Cargo.lock`. The edge is used by `src/proof.rs` alone — nothing on the route compiles MSL, and a probe object is never carried in an artifact or dispatched.

**Boundary.** The refusal is a fact about what `MTLComputePipelineReflection` reports on the measured row; no other class than texture, sampler, and threadgroup memory has been exhibited by a real object, and the remaining nine are covered by the classification test alone.
