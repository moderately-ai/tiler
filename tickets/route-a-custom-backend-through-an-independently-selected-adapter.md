---
id: route-a-custom-backend-through-an-independently-selected-adapter
title: Route a custom backend through an independently selected runtime adapter
status: done
priority: p1
dependencies: [accept-the-public-backend-provider-composition-boundary, declare-a-required-gpu-family-in-the-artifact]
related: [runtime-execution-contract, prototype-metal-runtime-execution, make-runtime-routing-commit-authority-one-shot]
scopes: [implementation/runtime, implementation/artifact, contracts/artifacts, contracts/integrations, contracts/foundation, implementation/cargo-lock]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, runtime]
---
## User-visible outcome

A consumer's statically linked runtime adapter for one backend/representation family binds a validated artifact to a live execution context, prepares it before routing commit, and dispatches it with correct resource lifetimes — selected by the consumer, never resolved from a registry.

## Implementation keys

- Define `LiveExecutionContext` separately from the existing device-free `ExecutionEnvironment`; do not make a caller-stated tuple masquerade as discovered device truth.
- **Corrected 2026-07-31 by accepted [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), which this key predated:** there is no adapter registry — independent selection is the runtime adapter's mechanism, and giving it one is a named eliminated alternative. The consumer selects its adapter the way `prototypes/serial-sum-run` and the CPU vertical already do; what joins producer to adapter is the artifact's governed backend/representation/profile identities, compared by the loader, with producer provenance never matched against adapter identity.
- Match backend family and representation as a pair, classify the target profile, and check payload identity and live applicability before adapter preparation — the loader owns every comparison; the adapter reports facts and never adjudicates them (ADR 0090 item 4).
- Reuse the route-requirement family and exact-entry `PreparedEntryTargetRequirement` authority established by `declare-a-required-gpu-family-in-the-artifact`; do not invent an adapter-specific capability, query, or applicability vocabulary.
- Keep device discovery, library/pipeline preparation, binding, encoding, submission, terminal-success observation, and asynchronous retention in the adapter half, downstream of device-free decoding and validation.
- Preserve preflight before routing commit and forbid fallback after allocation, partial encoding, submission, or semantic validation failure.
- Add one non-Metal adapter fixture and retain the existing Metal proof as an independent consumer.
- Perturb every identity/compatibility field, preparation outcome, binding, and post-commit failure path.
- Present the exact public runtime trait, context, and call-site boundary to Tom (no registry exists to present).

## Closes when

One external adapter executes a carried payload through the ordinary loader/route path; incompatible and post-commit failures are typed and explainable (missing/duplicate registration failures do not exist under the accepted no-registry model); asynchronous resources survive final device use, targeted checks and final gate pass, and `tiler-runtime` remains device-free.

## Graph maintenance

- Release backend-aware variant selection only after a registered adapter can establish eligibility.
- Keep Candle and Metal runtime objects out of the consumer-neutral trait.
- Split any unsafe FFI site into its own ADR-0079-conforming review.

## Outcome

Landed on `tkt/route-a-custom-backend-through-an-independently-selected-adapter` from base `622cf62`. `tiler-runtime` gains `crates/tiler-runtime/src/adapter.rs`: `LiveExecutionContext`, the `RuntimeAdapter` trait, `AdapterRouteFailure`, and the loader-driven `route_with_adapter`. No registry, no discovery, no adapter identity, and no adapter-specific capability vocabulary. The crate still names no device object and still depends on `tiler-artifact` alone.

**The public boundary is presented, not self-accepted.** `adapter` is `pub` and documented as a reviewed draft boundary on the same footing as `load` (ADR 0074 §7); the exact trait, context type, and call-site shape are Tom's under ADR 0075 and are set out in the dispatch report.

**Where the comparisons happen.** Unchanged and entirely the loader's. `route_with_adapter` calls `DecodedProgram::prepare` with the environment the adapter *reported*, so program identity, variant selection, the variant profile classification, the backend/representation pair, the payload profile classification, the execution policy, the launch geometry, the bindings, and the shared-storage derivation all stay in `load.rs`; `resolve_live_device_requirements` and `resolve_target_properties` keep the two device comparisons. The adapter's four reporting methods return an `ExecutionEnvironment`, a `LiveDeviceObservation`, a `u64`, and nothing else.

**Where payload validation runs (ADR 0090 item 8 / D7).** `RuntimeAdapter::validate_payload`, once per routed entry in execution order, immediately after `prepare` publishes the carried objects and before the first live-device question. The schedule is fixed by `route_with_adapter` rather than left to each adapter. `every_payload_defect_is_the_backends_refusal_and_the_artifact_layer_accepts_the_bytes` proves the artifact layer cannot discharge it: eight damaged objects each decode, verify, and re-derive **the same canonical artifact identity** as the sound one, because identity excludes the emitted object.

**The non-Metal fixture** is `crates/tiler-runtime/tests/adapter_route/` — an out-of-crate implementor compiling against the public surface only, with its own governed profile key, backend family, representation, a real domain-separated image format, and a scalar interpreter. It routes a real verified program end to end and agrees with `tiler-reference` bit for bit over a negative zero, the least positive subnormal, and a negative operand. `prototypes/serial-sum-run` is untouched.

**Scope note, declared rather than silent.** `implementation/cargo-lock` was added to this ticket's scopes: the `[dev-dependencies]` on `tiler-ir` and `tiler-reference` that the fixture needs move two lines of `Cargo.lock`, and a manifest change cannot land without them. ADR 0081 item 2 fixes this crate's *dependency* closure at `[tiler-artifact]`; a development edge does not enter it, `tiler-ir` was already linked transitively, and neither edge reaches `tiler-compiler`. The manifest states that derivation at the site. No ADR was edited — `contracts/decisions` is outside this ticket's scopes — so ADR 0090's `implementation_status` stays `not-started`, deliberately: the boundary is unaccepted, and moving it belongs to the acceptance rather than to this branch.

`make full` green.

**Boundary acceptance (2026-07-31).** Tom accepted the `tiler_runtime::adapter` surface as the reviewed draft it is documented as — `RuntimeAdapter`, `LiveExecutionContext`, `route_with_adapter`, `AdapterRouteFailure` — with the two open sub-questions (returning the context to the caller; a borrowing `Completion`) deferred to the first real consumer.
