---
id: declare-host-dtype-dispatchability-at-the-consumer-boundary
title: Give every consumer a host-earned dtype-dispatchability declaration
status: in-progress
priority: p2
dependencies: [validate-bf16-at-the-runtime-routing-boundary]
related: [decide-per-dtype-dispatchability-as-a-target-capability, read-the-serial-sum-proofs-dtype-rows-from-its-declaration]
scopes: [implementation/frontend, implementation/build, implementation/candle, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, runtime, routing, fail-closed, authority]
claimed_from: todo
assignee: agent-dtype-rows
lease_expires_at: 1786042476
---
## User-visible outcome

A consumer's `ExecutionEnvironment` states which dtypes *this machine's* target family dispatches, derived from something the machine or its build earned, rather than restating what the artifact's producer declared. Until then the runtime's dtype refusal cannot fire on the two paths a real consumer takes.

## Why the restatement is a gap and not a design

**Fact.** `validate-bf16-at-the-runtime-routing-boundary` added `ExecutionEnvironment::dtype_dispatch` and made an undispatchable dtype filter a variant before ADR 0051's routing commit. Every consumer literal it had to fill in restates a producer declaration:

- `crates/tiler/src/route.rs::execution_environment` reads the macro-emitted `RouteFacts`, which carries **no** dtype fact at all, so it states `f32` alone on the reasoning that `tiler-compiler` already refused any request whose dtype the selected profile did not resolve `Dispatchable` — a restatement of the compile gate, not an observation.
- `prototypes/candle-metal-adapter/src/proof.rs::declared_route_environment` transcribes `tiler-build`'s `FIRST_MACOS_APPLE9` ledger rows, and says of itself that it is "producer-declared equality, NOT host-earned eligibility".

**Inference.** A comparison whose two sides come from one authority refuses nothing. This is the same shape as `ExecutionEnvironment::classify` on those paths, and it is why the dtype check's value today is the *named* refusal it can produce rather than a barrier those paths did not already have.

**Fact.** ADR 0086 is directly on point about what may and may not stand in for a host-earned fact: a public execution-environment row is a necessary validity scope and explicitly not sufficient authority, and item 4 excludes a list of substitutes by name. A dtype row asserted at a call site is that failure mode with a different subject.

## Implementation keys

- Emit the selected profile's declared dtype-dispatchability rows into `RouteFacts`, so `execution_environment` restates an emitted fact instead of asserting one. This is what a `bf16` inline region needs before it can route at all — today it would be refused, correctly, for a reason only a source comment records.
- Decide, and record, whether the frontend's inline-region path can offer a *host-earned* row at all, given that it binds no device before `route_with_adapter` returns. If it cannot, say so in the contract rather than leaving the restatement looking like an observation.
- Give `BoundMetalCompileDeclaration` an accessor for its dispatchability rows so the Candle prototype reads them rather than transcribing them; it already holds `bf16_dispatchability`.
- Keep silence fail-closed. A consumer that cannot yet earn a row states nothing and refuses, which is the existing behaviour and must not be relaxed into a permissive default.

## Required evidence

- An inline region whose dtype the emitted rows do not admit is refused, and the refusal is observed failing.
- A perturbation of the emitted row changes the routing outcome, so the fact is load-bearing rather than carried.
- Each remaining restatement, if any survives, is named in the contract with what would make it host-earned.

## Closes when

No consumer asserts a dtype-dispatchability row at a call site without either an emitted fact or a bound-device observation behind it, or the surviving restatements are recorded as such in a durable contract with their authority gap stated.

### Scopes added while working, and why

`contracts/integrations` — one section of `docs/integration/frontends.md`. Implementation key 2 requires the inline-region path's authority answer to be *recorded in the contract*, and that document is the frontend contract; leaving the derivation in a rustdoc comment alone would make it a claim about code rather than a term of the contract. The edit is two added paragraphs in **Direct byte embedding**, beside the paragraph that already describes what an expansion emits, and nothing else. Verified free of any live claim on 2026-08-06: the five other in-progress tickets hold `implementation/ir`, `implementation/compiler`, `implementation/artifact`, `contracts/decisions`, `contracts/navigation`, `contracts/numerics`, `research/numerics`, `research/artifacts`, and `research/apple-targets`.

## Outcome

**The row is now emitted, and emitting it did not make the comparison non-tautological on this path — that part is structural, and the ticket asked for it to be said rather than fixed.** Implementation key 1 is discharged: `RouteFacts` carries the selected profile's declared dtype-dispatchability rows and `execution_environment` restates them. Implementation key 2 is decided and recorded in `docs/integration/frontends.md`: the inline-region path **cannot** offer a host-earned row, because `execution_environment` builds the `ExecutionEnvironment` that is an *input* to `DispatchAdapter::dispatcher`, so the environment exists before the integration's adapter does and there is no point on the path at which a device could be consulted. What emission removed is a call-site literal standing in for a declaration — ADR 0086 item 4's failure mode in a different subject; what it could not remove is that the row a facade publishes is the producer's.

**Fact — what landed, per key.**

1. `crates/tiler/src/route.rs`: `RouteFacts` gains `dtype_dispatch: &'static [(ArithmeticType, DTypeDispatch)]`, and `execution_environment` collects it into the map `ExecutionEnvironment` holds. A repeated arithmetic type is `MalformedRouteFacts` rather than resolved by insertion order, which is what keeps `ExecutionEnvironment`'s own "one arithmetic type cannot carry two verdicts" checkable at the emission boundary. `crates/tiler-macros/src/aot.rs` builds the field from `declaration.dtype_dispatchability_rows()` and renders each pair through two exhaustive path emitters, in the vocabulary a *host* states (`::tiler::runtime::load::DTypeDispatch`) rather than the one a *profile* declares.
2. Recorded in `docs/integration/frontends.md` **Direct byte embedding**, and in `route.rs`'s module and `execution_environment` documentation. Both name the one place a host-earned row can arise instead — the integration's `RuntimeAdapter::bind_execution_context`, which holds the device the facade does not — and both state that an adapter echoing `RegionRequest::declared_environment` back has chosen producer-declared equality for the dtype rows along with everything else.
3. `tiler_build::BoundMetalCompileDeclaration::dtype_dispatchability_rows()` answers from the bound `TargetProfile` at `AvailabilityPhase::CompileProfile` — the same profile and phase `tiler_compiler`'s `require_compile_profile_dispatch` consults — for every `ArithmeticType::ALL` resolvable through `tiler_ir::numerics::registered_arithmetic_value_type`. `prototypes/candle-metal-adapter/src/proof.rs::declared_route_environment` reads it. `tiler-build` re-exports `DTypeDispatchability` so a consumer without a `tiler-compiler` edge can match the verdict rather than have a third spelling minted for it.
4. Silence is omitted at every hop. The accessor returns **no row** for a dtype the profile resolves `Unknown` or `Deferred`, the expansion emits none, and `classify_dtype` resolves an absent key `Unknown`, which refuses. `Deferred` has no runtime spelling on purpose: it is the answer that arrives after the phase a host is stating at.

**Measurement — the refusal watched failing, on the real consumer path.** Four deliberate perturbations, each reverted and each verified reverted by a clean `cargo nextest run -p tiler-build -p tiler-macros -p tiler` (290 passed):

| Perturbation | Tests that failed |
| --- | --- |
| `dtype_dispatchability_rows` returns no row | **4** — both `tiler-build` row tests, `tiler-macros`' emission test, and the facade suite, where *both* `inline_region_dispatches.rs` and the new fixture stopped at stage log `["bind"]` |
| the declared `Dispatchable` verdict rendered as `Unsupported` | **3** — both `tiler-build` row tests and the facade suite, again at `["bind"]` with `published Unsupported` |
| the above **plus** `execution_environment` reverted to the old hard-coded `f32` literal | **3**, all in `crates/tiler` — and the facade suite passed again, which is the point: the old code could not see an emitted row change at all |
| the fixture withholds `bf16` — a dtype the region does not compute in — instead of `f32` | **1** — the fixture, because the route reached `validate-payload`; the refusal is about the region's own dtype and not about withholding something |

**Measurement — the inline-region refusal, end to end.** `crates/tiler/tests/facade/pass/inline_region_refuses_an_undispatchable_dtype.rs` is an out-of-tree consumer crate that routes `in a: f32[4], b: f32[4], c: f32[4]; deliver macos; contract flush_subnormals_to_zero_f32; out (a * b) + c` twice through one adapter over one artifact, differing only in whether the region's own dtype row is stated. Stated: `stages ["bind", "validate-payload"], published Dispatchable`. Withheld: `stages ["bind"], published Unknown` — refused before the payload is looked at, and the region still returns the declared result both times, because both refusals precede the one-way commit. The stage log is asserted rather than the outcome class, for the reason `validate-bf16-at-the-runtime-routing-boundary` recorded: a refusal that moved one phase later still refuses and still looks green.

**Measurement — the Candle path is unchanged on hardware.** `cargo run -p tiler-prototype-compile -- --out <scratch>/serial-sum.tiler && cargo run -p tiler-prototype-candle -- --artifact <scratch>/serial-sum.tiler` on **Apple M4 Max, macOS 27.0 build 26A5388g, registry `0x100000550`, 2026-08-06**: 6 of 6 published members resolved, 4 routed and agreed with the producer's recorded reference evaluation across 20 cases, 2 refused by the zero-extent preflight, ADR 0086 refusal printed before anything routed. The rows now come from the accessor and the run is identical to the transcribed one, which is the whole claim — the read is a source change, not a behaviour change, at the rows the ledger currently declares.

### Surviving restatements, each named with its authority gap

- **`crates/tiler/src/route.rs::execution_environment`** — restates an emitted fact rather than asserting a literal, and the fact is producer-declared. **Gap:** the facade holds no device and the environment is an input to adapter construction, so no observation is reachable here at all. Closing it is not a code change on this path; it is an integration answering `bind_execution_context` from a device it observed. Recorded in `docs/integration/frontends.md` and in the function's own heading.
- **`prototypes/candle-metal-adapter/src/proof.rs::declared_route_environment`** — reads the declaration instead of transcribing it, and the rows stay producer-declared. **Gap:** this binary holds a real `MTLDevice` and asks it nothing about either dtype, so a host-earned row would need a per-dtype observation on that device. ADR 0086 refuses the applicability receipt on every macOS row currently observable, so no observation this binary could take would make the profile this host's to offer — which is why the gap is a decision to record and not a task to schedule. Recorded in the function's own heading.
- **`prototypes/serial-sum-run/src/proof.rs:1097`** — still a transcribed literal, and **out of scope**: `implementation/runtime`, which this ticket does not hold. Filed as `read-the-serial-sum-proofs-dtype-rows-from-its-declaration` with the accessor and the conversion named. **Gap:** the same producer-declared gap as the Candle site, plus the drift risk the accessor removes — a retracted ledger measurement would leave this literal stating a verdict the profile no longer holds.
- **`spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs:257`** — out of scope (`research/target-profiles`) and **not the same shape**. It declares what that vertical's own in-process scalar CPU backend interprets, which is a statement about the thing that will execute the payload rather than a restatement of the producer's profile. No ticket filed, because there is no drift to remove: the row is about the backend compiled into that binary.

### What this ticket did not do

- **It did not admit `bf16` into the region grammar.** `crates/tiler-macros/src/region.rs::ELEMENT_TYPES` admits `f32` alone, so no inline region can state a dtype the emitted rows fail to admit *by writing it*. The required refusal is therefore exhibited by withholding a stated row from the environment the route is settled against, which is the same predicate under the loader's own fail-closed rule and is the only shape this grammar can exhibit today. A `bf16` region reaching this path would be refused at *expansion* by `RequestError::DTypeNotDispatchable` long before routing, which is the compile gate and not this boundary.
- **It moved no support-matrix row.** Nothing here dispatches a `bf16` kernel and nothing here earns a host applicability receipt.

### Public boundaries — draft, pending Tom

- `RouteFacts::dtype_dispatch` is a **new field shape on a macro-emitted surface**, so it lands as a labelled draft. `RouteFacts` reaches consumers only through `tiler::__private`, whose own documentation states that nothing in it is covered by a compatibility claim, but the field is what generated tokens spell and is therefore a real emitted-surface change.
- `BoundMetalCompileDeclaration::dtype_dispatchability_rows` and the `tiler_build::DTypeDispatchability` re-export are additive accessors over a field the declaration already held, which the brief classifies as minor.
