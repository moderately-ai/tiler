---
id: validate-metal-payload-argument-slots-against-declared-bindings
title: Validate a Metal payload's argument slots against the entry's declared bindings
status: done
priority: p2
dependencies: [prototype-candle-metal-adapter]
related: []
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [candle, runtime, artifacts]
---
## User-visible outcome

A `metallib` whose kernel takes a different set of buffer arguments than the artifact's entry declares is refused before the routing commit, naming the disagreement, rather than reaching the encoder and being bound wrongly.

## Why this is open

ADR 0090 item 8 places three obligations on a backend validating its own payload from bytes: that the bytes decode into something executable, that they name the entry symbol the artifact says they do, and **that the slots they address are the ones the entry declares**. `prototypes/candle-metal-adapter`'s `validate_payload` discharges the first two and not the third.

**Fact — the third needs reflection, which nothing in the stack currently builds.** Reading a Metal function's argument table means `newComputePipelineStateWithFunction:options:reflection:error:` and an `MTLComputePipelineReflection`. Candle's `metal::Device` wrapper (`candle-metal-kernels-0.11.0/src/metal/device.rs`) exposes only `new_compute_pipeline_state_with_function`, which discards reflection, and the adapter builds pipelines through it — so no argument table is available at any point in the route.

**Fact — the consequence today is a wrong binding rather than a refusal.** A slot the object does not address is set and ignored; a slot it addresses that the artifact does not declare is never set, and the kernel reads an unbound argument. Neither produces an error at encode time, so this is one of the few paths in the adapter that does not fail closed. It is bounded by the artifacts this profile routes being produced by Tiler's own emitter from the same ABI, which is a reason it has not bitten rather than a guarantee.

## Closes when

- The adapter obtains the prepared pipeline's reflection — through `objc2-metal` directly if Candle's wrapper still discards it — and compares the buffer arguments it reports against the entry's declared ABI bindings and their transport slots.
- The comparison runs before the routing commit, and its refusal is a typed pre-commit class distinct from an absent symbol.
- The refusal is watched failing against a real object: an artifact whose declared binding count or transport mapping is perturbed away from the object's own.
- If reflection proves unavailable or unreliable on the qualified toolchain row, that is recorded as a measurement with its exact procedure, and the obligation is restated as explicitly undischargeable rather than left implicit.

## Outcome

Discharged. Reflection is available on the qualified row, so the fourth bullet's fallback did not apply.

**Fact — the route to reflection is `objc2-metal` directly, and by the *descriptor* overload.** `candle-metal-kernels` 0.11.0 still discards the out-param, as the ticket recorded. Of the two synchronous reflection-bearing constructors `objc2-metal` 0.3.2 declares, `newComputePipelineStateWithFunction:options:reflection:error:` is an `unsafe fn` (`MTLDevice.rs:1244`) and `newComputePipelineStateWithDescriptor:options:reflection:error:` is safe (`MTLDevice.rs:1302`). The adapter builds an `MTLComputePipelineDescriptor` whose only set property is the compute function and calls the safe one, so the crate keeps the workspace's `unsafe_code = "forbid"` and ADR 0079 is not engaged. `MTLPipelineOption::BindingInfo` is the option; `MTLPipelineOption::ArgumentInfo` is the same bit and is deprecated in this binding.

**Where the comparison runs, and why not in `validate_payload`.** `prepare_entries`. The argument table is a property of the compiled pipeline, Metal publishes it only as the out-param of pipeline creation, and no pipeline exists before that stage; asking earlier would mean building every pipeline during payload validation, moving `PipelineRejected` — a route fact about the device — into the stage that decides artifact invariants. Pre-commit is satisfied by construction: `route_with_adapter` runs `prepare_entries` before `resolve_target_properties`, `plan_dispatch`, and `Preflight::commit`, it returns the adapter's `Refusal` type, and nothing is allocated when it refuses.

The typed class is `RouteRefusal::ArgumentSlotsDisagree`, distinct from `EntrySymbolAbsent`, rendering verbatim as:

```
candle-metal.prepare: entry 0's "tiler_kernel_27169bca52f872ab" addresses buffer argument(s) [0, 1] and the entry declares transport slot(s) [0, 1, 2]
```

A second class, `RouteRefusal::ArgumentTableUnavailable`, fails closed when the device builds a pipeline and returns no reflection — an absent table is never read as an empty one.

**Measurement — reflection agrees with the emitter on this row.** Apple M4 Max, macOS 27.0 build 26A5388g, arm64, family Apple9; toolchain `nightly-2026-07-19`; `objc2-metal` 0.3.2, `candle-metal-kernels` 0.11.0. Procedure: `cargo run -p tiler-prototype-compile -- --out <base>` then `cargo run -p tiler-prototype-candle -- --artifact <base>`. Every routed entry's reflected buffer arguments equalled its declared transport slots — six prepared pipelines across the four routable members, including both two-entry `materialized` members — and the run reported `20 case(s) agreed across 4 of 6 published member(s)`, unchanged by the new stage. The probe prints the real table it read: entry 0's `tiler_kernel_27169bca52f872ab` addresses `[0, 1]` and the artifact declares `[0, 1]`.

Counting is by `MTLBindingType::Buffer` and ignores `isUsed`: the comparison is about the argument table the object addresses, and a declared-but-unread parameter still occupies its index. Threadgroup rows are excluded because `[[threadgroup(N)]]` is a disjoint numbering. The measurement bounds this to the emitter's current signature shape on this row; it is not a claim about objects from another producer.

**The perturbation is on the declaration, and that is forced rather than chosen.** The envelope proves an integrity digest over the object bytes, so an edited transport mapping is refused as a damaged envelope — a different check passing — long before any argument table is read. So the object stays real and the declaration moves. Two independent perturbations were watched failing:

- Through the whole route: `declared_transport_slots` shifted by 7, rebuilt, re-run. The proof failed closed with `adapter.preparation: ... addresses buffer argument(s) [0, 1] and the entry declares transport slot(s) [7, 8]`, exit 1, before any commit.
- In the binary's own probe set, against the real reflected table read from the real published object: one slot the object does not address, one declared slot dropped, and the same count at a renumbered slot each refuse and print.

The unit test was also watched failing: forcing `argument_slots_agree` to never refuse fails `an_argument_table_agrees_with_its_declaration_or_names_the_disagreement`. That perturbation caught a defect in the test itself — an `expect_err` message with format braces in a plain `&str` — which is fixed.

**Cache.** `PipelineCache` now stores a `PreparedPipeline` — pipeline plus reflected table — because reflection is recoverable only during creation. It caches the table as a fact and never the verdict: the declared side comes from the current attempt's routed bindings and is not part of the cache key, so the comparison re-runs on every hit.

Public boundary: none. Everything is inside the non-published `tiler-prototype-candle` prototype; `objc2-metal` gained the `MTLAllocation` and `MTLArgument` features and no new top-level dependency was added.
