---
id: route-an-embedded-artifact-through-a-consumer-storage-seam
title: Dispatch an embedded artifact through a consumer storage seam
status: in-progress
priority: p1
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/frontend, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, inline-dx, runtime, public-boundary]
claimed_from: todo
assignee: worker-route-an-emb
lease_expires_at: 1785568847
---
## Why this exists

`prototype-inline-aot-integration-proof` landed everything an inline region needs up to the routing commit and stopped one step short of a running kernel. This ticket is that step, and what blocks it is a *missing accepted boundary* rather than unwritten code.

**Fact.** `crates/tiler/src/value.rs` publishes no storage access — "Nothing here yields a pointer, a buffer, a byte slice, or a device object" — and states why: "a storage-access surface would be a public boundary with no caller to review it against". A `tiler`-only consumer therefore has nothing to hand a kernel.

**Fact.** `crates/tiler/tests/dependency_direction.rs::no_package_depends_on_the_frontend` forbids any workspace package from depending on `tiler` or `tiler-macros`, so no in-tree crate can be the consumer that dispatches. The only consumers are the out-of-tree `trybuild` fixtures, which see `tiler`'s dependency list and nothing else.

**Inference.** `tiler_runtime::load::Preflight::commit` is unreachable from any consumer of the facade today, which is why `crates/tiler/src/route.rs` stops at `RouteOutcome::NoDeviceAuthority` and contains no call to `commit` at all.

**Fact.** ADR 0086 refuses on every macOS row, so even with a seam the dispatching path is `prototypes/serial-sum-run`'s producer-declared equality under its labelled diagnostic, never host-earned eligibility.

## User-visible outcome

One inline invocation in an ordinary crate produces a running kernel: the embedded artifact is routed, committed, and dispatched against the consumer's own values, with the fallback still taken before the commit and nowhere after it.

## Closes when

- The storage seam a dispatch needs is designed, put to Tom as a public boundary under ADR 0075, and accepted or refused with a recorded reason. `AdapterCapability::DenseRowMajorStorage` is the reservation it fills.
- `crates/tiler/src/route.rs` gains a committed outcome, and `RouteOutcome::is_fallback` stops reading as a constant.
- The run is recorded with the same labelled producer-declared-equality diagnostic `prototypes/serial-sum-run/src/proof.rs` prints, and says in those words that ADR 0086 refused the host.
- A correctness oracle compares the dispatched result against the semantic fallback's before any performance claim is made.

## Outcome

**The seam is built and the committed outcome exists.** `AdapterCapability::DenseRowMajorStorage` is filled, `RouteOutcome::is_fallback` is a real query, and an out-of-tree consumer naming `tiler` alone drives a compiled Metal artifact through the loader with its own storage. What is *not* delivered is a completed dispatch on hardware; the reason is a boundary rather than an omission and is stated below.

### The seam, and the eliminations behind its shape

**Proposal, provisionally accepted 2026-08-01 under overnight mode; Tom's to ratify or refuse.**

`crates/tiler/src/value.rs` gains a second trait rather than three methods on `TensorAdapter`:

```rust
pub trait DispatchAdapter: TensorAdapter {
    type Refusal;
    type Failure;
    type Dispatch<'region>: RuntimeAdapter<Refusal = Self::Refusal, Failure = Self::Failure>;
    fn storage(value: &Self::Value) -> Result<&[u8], Self::Error>;
    fn storage_mut(value: &mut Self::Value) -> Result<&mut [u8], Self::Error>;
    fn dispatcher<'region>(context: &Self::Context, request: RegionRequest<'region>)
        -> Result<Self::Dispatch<'region>, Self::Error>;
}
```

with `RegionOperand<'a>` (an interface key and a checked byte run), `RegionRequest<'region>` (every operand, the result's storage to write into, and the producer's declared `ExecutionEnvironment`), `BindError::{StorageLengthMismatch, DispatchFailed}`, and `tiler::{runtime, artifact}` re-exporting `tiler_runtime::{adapter, load}` and `tiler_artifact::program` unchanged.

- **A byte-run borrow, not a slot-binding callback.** `DenseRowMajorStorage` is a claim that a value's elements are contiguous, innermost-axis-fastest, with no offset or stride — which is exactly the claim `&[u8]` cashes and a callback does not. A binding callback would let a strided value be bound and would leave the reservation unfilled.
- **A second trait, not more methods on `TensorAdapter`.** The obligation exists only for a region whose `deliver` statement selected a family; generated code calls `bind_and_build` otherwise. Bounding only `bind_route_and_build` by `DispatchAdapter` means the compiler demands storage and a device authority exactly where a kernel would read them, and every fallback-only adapter, doc example, and fixture compiles unchanged.
- **The existing `RuntimeAdapter`, not a new executor seam.** ADR 0090 row 12 already names the consumer's statically linked executor, and `route_with_adapter` already sequences payload validation before the first device question, both device stages unconditionally, and the one-way commit. A facade-local executor would be a second place those obligations live, and the two would drift. It also could not have been added without changing `RuntimeAdapter`, which would break `prototypes/candle-metal-adapter` — outside this ticket's scope.
- **A factory, not a stored adapter.** A `Tensor` is borrowed shared at a call site and every `RuntimeAdapter` method takes `&mut self`, so lending a stored authority would force interior mutability on every integration. Building one per invocation removes the problem and is what makes the region's storage reachable — the adapter is constructed *from* the request. `crates/tiler-runtime/tests/adapter_route` already builds its adapter this way (`ScalarHostAdapter::new(&OPERANDS)`); this makes the idiom the contract.
- **The facade does not call `commit`.** `route_with_adapter` does. `RouteOutcome::is_fallback` delegates to `AdapterRouteFailure::fallback_permitted`, so which side of ADR 0051's commit an outcome landed on is the driver's exhaustive classification and not a second one this crate composes.

### Convergence with the Candle seam-friction report

`prototype-candle-metal-adapter` recorded that `route_with_adapter` "runs stages 1 through 9 in one call and there is no way to stop after stage 7", and that hand-driving is impossible because `LiveExecutionContext` has no public constructor; it asked for `route_prepared(...) -> PreparedRoute<'_, A>`.

**This consumer hit the same wall and resolved it the other way, which is evidence for the report rather than against it.** The friction is identical — a caller needs to interleave its own work with the nine stages — but the interleaving here is *before* stage 1 rather than between 7 and 8, so it is expressible by constructing the adapter around the region's storage. That is a second independent confirmation that the wall is real and that its location is stage 7/8 specifically: a consumer whose work fits before stage 1 can proceed, and one straddling a foreign callback boundary cannot. `route_prepared` remains unanswered and is not needed by this consumer.

### What runs, and the exact transcript

`crates/tiler/tests/facade/pass/inline_region_dispatches.rs` is an out-of-tree crate depending on `tiler` alone. It writes `in a: f32[4], b: f32[4], c: f32[4]; deliver macos; out (a * b) + c`, which compiles a real metallib through `xcrun` during `rustc`, and supplies its own `DispatchAdapter` and `RuntimeAdapter`.

```
$ cd target/tests/trybuild/tiler && cargo run --bin trybuild006
DIAGNOSTIC — producer-declared equality against tiler.metal.macos-apple9.msl4-0.f32.v1, NOT host-earned eligibility
stages: ["bind", "validate-payload"]
handover: [
    "a=[1.5, -2.0, 0.25, 8.0]",
    "b=[4.0, 3.0, -16.0, 0.5]",
    "c=[0.5, 1.0, 2.0, -3.0]",
    "out=16 byte(s) to write",
    "entry symbol \"tiler_kernel_ce0acbceb6c201da\", 3859 object byte(s), 4 binding(s)",
]
```

**Measurement.** macOS 27.0, Apple M4 Max, pinned nightly per `rust-toolchain.toml`, worktree `route-an-embedded-artifact-through-a-consumer-storage-seam/edit` at the commit below. The route decoded the embedded envelope, matched the recorded identity, published the producer's declared environment, selected a variant, routed one entry, and handed this consumer the entry's real symbol, its 3859-byte metallib object, and four bindings. The consumer refused its own payload from the bytes — ADR 0090 item 8's obligation, before the first live-device question — and the region returned its declared result on the fallback.

The labelled diagnostic is `tiler::__private::PRODUCER_DECLARED_EQUALITY`, and `crates/tiler/tests/labelled_diagnostic.rs` asserts it is a substring of `prototypes/serial-sum-run/src/proof.rs`'s printed form after string-continuation reconstruction — so a paraphrase in either place fails the gate.

### Not delivered: a completed dispatch on hardware

**Fact.** No test in this repository reaches `RouteOutcome::Dispatched`, and none can, for a reason that is one line to reproduce in each case:

1. *A `trybuild` fixture cannot link Metal.* `trybuild` generates its manifest from `tiler`'s own `[dependencies]` — inspect `target/tests/trybuild/tiler/Cargo.toml`, which lists `tiler`, `tiler-artifact`, `tiler-ir`, `tiler-macros`, `tiler-runtime` and no `metal` — and `tiler` must never carry a backend.
2. *An integration test in `crates/tiler/tests/` cannot read a `MTLBuffer`.* `metal` 0.33's only storage accessor is `Buffer::contents() -> *mut c_void`, and dereferencing it needs `unsafe`, which `[workspace.lints.rust] unsafe_code = "forbid"` (root `Cargo.toml:97`) makes unrelaxable by any inner attribute. Relaxing it is Tom's decision under AGENTS.md.
3. *No workspace package may depend on `tiler`.* `crates/tiler/tests/dependency_direction.rs::no_package_depends_on_the_frontend`, so `prototypes/serial-sum-run` — which has Metal and the two admitted `unsafe` sites — cannot be the consumer.

**Inference.** The composition needs an out-of-tree consumer crate that links both `tiler` and `metal`, which is a placement decision (a `spikes/` directory with its own workspace, per the existing 12) outside this ticket's declared scopes. Filed as `dispatch-a-tiler-region-on-metal-hardware`.

**What is nonetheless established.** Each link has hardware or compiler evidence separately: the facade reaches a real routed entry against a real metallib (above); `route_with_adapter` reaches a completed dispatch on Metal hardware in `prototypes/candle-metal-adapter` and a completed dispatch with a bit-exact oracle in `crates/tiler-runtime/tests/adapter_route`. What no single artifact yet shows is the two composed.

**The correctness oracle is therefore stated and not yet exercised on a dispatched result.** The fixture computes `(a * b) + c` in its own Rust — deliberately not derived from anything Tiler produced — and the comparison against a dispatched result is what the follow-up ticket adds. `bind_route_and_build` already refuses rather than returning a value when a committed route fails (`BindError::DispatchFailed`), which is the half that could have returned an incorrect tensor.

### Perturbation table

Every new check was watched failing against a case that must fail.

| Perturbation | Caught by |
|---|---|
| drop the operand storage-length check | `route::tests::storage_shorter_than_the_extents_it_reports_is_refused` |
| drop the result storage-length check | `route::tests::a_result_shorter_than_its_declared_shape_is_refused` |
| hand every operand over under the first operand's key | `facade::facade_reexport_contract` (the fixture, on contents) |
| `Dispatched` reports a fallback again | `route::tests::the_fallback_answer_is_no_longer_a_constant` |
| a post-commit failure reports a fallback | `route::tests::the_fallback_answer_is_no_longer_a_constant` |
| paraphrase the label, dropping its negation | `route::tests::the_labelled_diagnostic_names_the_profile_and_keeps_its_negation` |
| make the continuation reconstruction a no-op | `labelled_diagnostic::a_continued_literal_is_reconstructed_before_it_is_compared` |

Two of these were first run against the wrong filter and reported green; both are caught, by a different test than expected. That found a real overclaim: `a_request_looks_its_operands_up_by_interface_key` builds a `RegionRequest` directly and so cannot see a route-level pairing bug. It is renamed and now says so, and names the fixture that does check it.

### One ordering change made because a test said so

The storage-length check was first placed after the decode, on the reasoning that "a damaged artifact should not be reported as a storage problem". Both length tests then failed against `b"not an artifact"`, because the decode refused first. The module's own accepted ordering settles it the other way — "the region's own obligations are checked *first*, so a region whose interface was not honoured refuses with the reason a consumer can act on" — and storage length is a value obligation. The values are now read before the artifact, and only the payload selector precedes them, so a target that embedded nothing still reads no value.

### Commands

- `make full` — green (1910 tests, doc-tests, rustdoc, release numerical suite, `ticketsplease lint`, shellcheck).
- `cargo clippy -p tiler -p tiler-macros -p tiler-runtime --all-targets -- -D warnings` — clean.
- `git diff --check` — clean. `Cargo.lock` unmodified, so `--locked` held and no `implementation/cargo-lock` claim was needed.
- `tkt guard --base d862c2b` — in scope.

### Dispatch note

The worktree was created at `a3d61bd`, one commit *behind* the named base `d862c2b`; the ticket file there still read `status: todo` because the claim was recorded in `d862c2b` itself. The branch had no commits of its own and a clean tree, so it was fast-forwarded to the exact named base before any edit (`git rev-list --left-right --count origin/main...HEAD` → `0 0`).
