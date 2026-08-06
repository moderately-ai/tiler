---
schema: "tiler-doc/v1"
id: "tiler.spike.runtime.inline-dispatch"
kind: "experiment"
title: "Inline regions dispatched on Metal hardware"
topics: ["runtime", "inline-dx", "metal", "dispatch", "artifacts", "numerics", "multi-entry"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.runtime.execution-contract"]
entrypoints: ["spikes/runtime/inline-dispatch/src/main.rs", "spikes/runtime/inline-dispatch/src/multi_entry.rs", "spikes/runtime/inline-dispatch/src/adapter.rs", "spikes/runtime/inline-dispatch/src/buffer.rs"]
last_verified: "2026-08-04"
verified_at_commit: "2c4d05c"
ticket: "dispatch-a-tiler-region-on-metal-hardware"
---

# Inline regions dispatched on Metal hardware

**Two consumers, one crate, one adapter.** Everything from here to [Unsupported cases and measurement boundary](#unsupported-cases-and-measurement-boundary) is the first: `src/main.rs`, a pointwise region taken to a completed dispatch, reporting `1/1 entry(ies) encoded`. [A two-entry bundle dispatched on Metal hardware](#a-two-entry-bundle-dispatched-on-metal-hardware) at the end is the second: `src/multi_entry.rs`, a region whose *selected* plan needs two executable entries. It was added beside the first rather than replacing it, because the first one's transcript is quoted from several documents. They share `src/adapter.rs` and `src/buffer.rs` deliberately — the adapter's generality over entry count is what the second consumer tests, and a copy would have tested the copy. `ticket:` above names this record's originating ticket; the second consumer arrived with [`dispatch-a-multi-entry-bundle-on-hardware`](../../../tickets/dispatch-a-multi-entry-bundle-on-hardware.md).

An ordinary crate writes

```rust
let d = tiler::tensor! {
    in a: f32[4], b: f32[4], c: f32[4];
    deliver macos;
    contract flush_subnormals_to_zero_f32;
    out (a * b) + c
};
```

and receives what a Metal kernel wrote. The artifact the expansion embedded is decoded, routed, committed, and dispatched on this host's device through the `tiler::value::DispatchAdapter` seam, and the bytes the consumer gets back equal its own `f32` arithmetic bit for bit.

Every link had evidence separately before this. `crates/tiler/tests/facade/pass/inline_region_dispatches.rs` drives the facade to a real routed entry against a real compiled `metallib` and stops there, because a `trybuild` fixture cannot link Metal. `prototypes/candle-metal-adapter` and `crates/tiler-runtime/tests/adapter_route` each reach a completed dispatch through `route_with_adapter`, one on hardware and one against a bit-exact host oracle. **No artifact showed the two composed**, and `RouteOutcome::Dispatched` was reachable from nothing in the repository. This is that artifact.

## Why it is out of tree

Two properties of the workspace, each one line to reproduce, and neither is a limitation to work around:

- `crates/tiler/tests/dependency_direction.rs::no_package_depends_on_the_frontend` forbids any workspace package from depending on `tiler`, so no member crate may be the consumer.
- the root manifest's `[workspace.lints.rust] unsafe_code = "forbid"` cannot be relaxed by an inner attribute at any scope, and `metal` 0.33.0's only storage accessor is `Buffer::contents() -> *mut c_void`, so no member crate may read what a kernel wrote.

Relaxing the workspace `forbid` for one named crate is the other placement the ticket named, and it is not this spike's to choose: AGENTS.md reserves that to Tom. A separate workspace costs nothing already decided, so this crate carries its own `[workspace]` table and sets `unsafe_code = "deny"` — a hard error everywhere except the one site that opts in by name.

## The contract each region states, and why it is that one

**Fact.** Both regions in `src/main.rs` state `contract flush_subnormals_to_zero_f32;`. (The two-entry consumer's region states the other admissible contract, and [why it must](#the-split-is-the-compilers-answer-not-this-files-request) is that section's subject.) Since [`state-the-numerical-contract-in-the-region-grammar`](../../../tickets/state-the-numerical-contract-in-the-region-grammar.md) landed Tom's 2026-08-01 decision that no numerical contract may be assumed, a region stating none is refused at expansion; before that statement was added here, this spike did not build, and the refusal it produced is quoted under "Every check was watched failing" below.

**Fact.** Three of the five statable contracts cannot be delivered for `deliver macos;` at all. `strict_f32`, `reassociate_f32`, and `relaxed_f32` each require preserved input subnormals, and the measured Apple `f32` row flushes them in every math mode; `crates/tiler-macros/src/aot/tests.rs`'s `the_bound_declaration_admits_the_two_flushing_contracts` pins the admitted pair as `FLUSH_SUBNORMALS_TO_ZERO_F32` and `FLUSH_AND_REASSOCIATE_F32`.

**Inference.** Of the two that remain, `flush_and_reassociate_f32` additionally authorizes ordered regrouping of one same-operation operand sequence, and this region contains none — `(a * b) + c` is a pointwise chain with no reduction. Stating it would claim a freedom the program cannot exercise and the transcript does not measure, so `flush_subnormals_to_zero_f32` is the narrowest true statement of what this region computes under: the two dimensions the hardware measurably moves, every other freedom refused. Contraction stays forbidden under **both**, so the oracle's "deliberately not `mul_add`" argument is independent of the choice.

**Measurement — the contract is an input to the kernel identity, and this choice is what preserves the transcript below.** Stating `flush_and_reassociate_f32` on the delivering region instead, on the host in the table, produced `tiler_kernel_f4013709b41a2116` in place of `tiler_kernel_ae031ce7240f7495`; the object length, binding count, launch, and every value were unchanged, and the run still passed its oracle. So the statement above is not cosmetic — a different admissible contract compiles a different artifact.

**Fact — the fallback-only region's statement is inert, and that is not this spike's gap.** With no `deliver`, `tiler-macros`' `expand` resolves the contract and then takes the branch that never calls `aot::deliver`, so nothing compiles under it and no feasibility check sees it. The fallback performs no arithmetic at all, so there is no behaviour for a contract to constrain. It states the same contract as the delivering region because it is the same region — `deliver` being the only difference is the whole claim that function makes. [`check-the-stated-contract-on-the-semantic-fallback-path`](../../../tickets/check-the-stated-contract-on-the-semantic-fallback-path.md) owns the gap and is `deferred` until the fallback evaluates something.

## Running it

By hand, from this directory. **No `make` target reaches a spike**, and none should: `make full` builds the workspace, and this crate is not a member of it.

```sh
cd spikes/runtime/inline-dispatch
cargo run --release
cargo run --release -- --halt-after-commit
cargo run --release --bin multi-entry-dispatch-spike
```

The first is the sound run. The second is the post-commit perturbation described below; it is a flag rather than the default, so the checked-in state is sound. The third is the two-entry consumer, which takes no flags — its perturbation runs first inside the same invocation, for the reason [that section](#the-reordering-runs-first-and-is-not-a-flag) gives.

The manifest sets `default-run = "inline-dispatch-spike"`, so the first two commands are unchanged by the second binary's arrival: a bare `cargo run` would otherwise be ambiguous across two `[[bin]]` targets, and those two invocations are what several documents tell a reader to type.

All three exit `0` on success and `1` on any disagreement, including an oracle mismatch. `cargo build` needs the pinned toolchain (resolved by directory ancestry — this spike deliberately carries no `rust-toolchain.toml`) and the Apple Metal toolchain, because the `deliver macos;` expansion compiles the region ahead of time.

## Host and toolchain

The transcript below was recorded on the left column. The right column is a
distinct environment — a different Metal toolchain build, installed on
2026-08-04 — and is kept as its own column rather than merged, because the two
runs are two measurements and a merged row would claim one.

| | 2026-08-01, re-verified 2026-08-02 | 2026-08-04 re-run |
|---|---|---|
| host | Apple M4 Max, arm64 | Apple M4 Max, arm64 |
| OS | macOS 27.0, build 26A5388g | macOS 27.0, build 26A5388g |
| Rust | `nightly-2026-07-19`, from the repository pin | `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`) |
| Xcode | not recorded | 27.0, build 27A5228h |
| Metal compiler | Apple metal version 32023.883 (`metalfe-32023.883`), target `air64-apple-darwin27.0.0` | Apple metal version 32023.921 (`metalfe-32023.921`), target `air64-apple-darwin27.0.0` |
| `metal` crate | 0.33.0, the version the root `[workspace.dependencies]` pins | 0.33.0 |
| repository commit | `93e253d` plus `state-a-numerical-contract-in-the-inline-dispatch-spike` | `2c4d05c` |

## Transcript, 2026-08-01, re-verified 2026-08-02

Verbatim, `cargo run --release`. Re-recorded when
[`reconcile-the-pre-commit-allocation-seam-with-adr-0051`](../../../tickets/reconcile-the-pre-commit-allocation-seam-with-adr-0051.md)
split the seam's sizing stage from its allocating one, which is why
`allocate-dispatch` appears between `plan-dispatch` and `dispatch`.

**Measurement — adding the `contract` statement moved nothing in this transcript.** Re-run on 2026-08-02 on the host in the table above, at the base commit in it, with both regions stating `flush_subnormals_to_zero_f32`: every line below is reproduced byte for byte, entry symbol `tiler_kernel_ae031ce7240f7495` included. That is the expected result rather than a lucky one — before Tom's decision the frontend compiled every delivering expansion under a constant that was exactly `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` (`crates/tiler-macros/src/aot.rs` at `990290d`, since removed), so restating that contract in the region's own text reproduces the same `CompileRequest` the recorded run was taken under. Under any other admissible contract it would not, as the identity measurement above shows.

**Measurement — the entry symbol drifted once before, and not from that change.** It was
`tiler_kernel_ce0acbceb6c201da` when this record was written at `8366ecd` and is
`tiler_kernel_ae031ce7240f7495` at the base above. The object length, binding
count, launch, and every value are unchanged. The one-line check that places the
drift elsewhere: `git status --porcelain -- crates/tiler-compiler crates/tiler-ir
crates/tiler-macros crates/tiler-metal crates/tiler-metal-aot crates/tiler-build
crates/tiler-artifact` reports nothing on this branch, so the kernel identity's
inputs were untouched by it and the value moved with work landed between the two
commits. Re-running this spike is what detects such drift, which is the trade
[AGENTS.md](../../../AGENTS.md) records for keeping a cited transcript.

**Measurement — 2026-08-02: the entry symbol moved again at the `tiler.schedule.v4` to `v5` step, and it is `tiler_kernel_a0f16709d95528ca`.** Observed by `cargo run --release` from this directory on the host in the table above, under [`implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5`](../../../tickets/implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5.md); the run exited `0` and its oracle passed. The transcript below is **not** rewritten and is not stale in the sense a pinned identity would be: this region carries no cooperative tile, so nothing about the program changed and only the eighteen domain-separator bytes did, through the fold. Every other line of the transcript — object length `3859`, four bindings, launch `4×1`, and all three handover value tables — reproduced byte for byte. This line exists because lines 90-91 above track the symbol *across commits* by hand and no gate checks that tracking, so the pin would otherwise have gone silently stale at `v5`.

**Measurement — 2026-08-04: the governed profile key moved from `tiler.metal.macos-apple9.msl4-0.f32.v1` to `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`, and the transcript below is otherwise reproduced byte for byte.** Observed by `cargo run --release` and `cargo run --release -- --halt-after-commit` from this directory on the 2026-08-04 column of the table above, at `2c4d05c`, with the two-entry consumer beside it; both exited `0`. Four lines moved and they are all the same fact: the `commit:`, `committed route completed`, `DIAGNOSTIC`, and `ADR 0086` lines each render the profile key. `5f1b7b1c` "Declare measured BF16 facts on the Metal profile" added independently sourced BF16 rows to `BoundMetalCompileDeclaration`, and the [authority ledger](../../../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md) records why that produces a *new truthful* key rather than a revision of one whose `.f32` component would have become false. The fifth moved line is the entry symbol, now `tiler_kernel_a0f16709d95528ca`, which the 2026-08-02 note above already tracks. Object length `3859`, four bindings, launch `4×1`, every handover, every stage, the oracle's `[6.5, -5.0, -2.0, 1.0]`, and the quoted `1/1 entry(ies) encoded` are unchanged. The block is not rewritten, for the reason the 2026-08-02 note gives: it is a dated observation, and this line is what keeps the difference between the two readable.

```
device: Apple M4 Max
mode: sound
oracle: the dispatched bytes equal this consumer's own f32 arithmetic bit for bit: [6.5, -5.0, -2.0, 1.0]
commit: committed route completed: 1/1 entry(ies) encoded, terminal status Completed, profile tiler.metal.macos-apple9.msl4-0.f32.v1
result: f32[4], 16 byte(s)
stage: bind
stage: validate-payload
stage: prepare-entries
stage: observe-prepared-entry
stage: plan-dispatch
stage: allocate-dispatch
stage: dispatch
handover: a = [1.5, -2.0, 0.25, 8.0]
handover: b = [4.0, 3.0, -16.0, 0.5]
handover: c = [0.5, 1.0, 2.0, -3.0]
handover: out = 16 byte(s) to write
entry 0: symbol "tiler_kernel_ae031ce7240f7495", 3859 object byte(s), 4 binding(s), launch 4×1
plan: 1 entry(ies), 0 shared allocation(s)
committed route completed: 1/1 entry(ies) encoded, terminal status Completed, profile tiler.metal.macos-apple9.msl4-0.f32.v1
DIAGNOSTIC — producer-declared equality against tiler.metal.macos-apple9.msl4-0.f32.v1, NOT host-earned eligibility
ADR 0086 refuses the host: native `metallib` translation during pipeline creation is a capability fact whose authority is Unknown on every macOS row currently observable, so no host — this one included — earns the right to offer `tiler.metal.macos-apple9.msl4-0.f32.v1`. The route above was settled on producer-declared equality, NOT host-earned eligibility.
fallback-only region: same declared interface (f32[4], 16 byte(s)), and its storage is [0.0, 0.0, 0.0, 0.0] — the facade constructs the declared result and evaluates nothing, so this is not a second value oracle
```

The entry symbol and object length agree with the routed entry `route-an-embedded-artifact-through-a-consumer-storage-seam` recorded from the `trybuild` fixture — `tiler_kernel_ce0acbceb6c201da`, 3859 object bytes, four bindings — which is the same region compiled by the same expansion, now executed rather than refused.

## What the commit evidence actually is

`RouteOutcome::Dispatched` is not returned to a consumer: `bind_route_and_build` yields the region's *value*, because that is what `let d = tiler::tensor! { … }` asked for. So the commit is established structurally instead.

`route_with_adapter` calls `Preflight::commit()` on the line before it calls `RuntimeAdapter::dispatch`, and nothing else calls that method. The `dispatch` stage appearing in the recorded stage list is therefore the routing commit, taken inside the driver that owns it — this spike reimplements no part of it and takes no fallback after it. The `committed route completed: …` note exists only if `dispatch` returned `Ok`, which is exactly the condition that makes the facade's outcome `RouteOutcome::Dispatched`; `main.rs` refuses the run when either is absent.

## The oracle

`main::oracle` is plain Rust `f32`, written the way this consumer would have written the region without Tiler. It is derived from nothing Tiler produced — not a reference kernel, not a sidecar, not the facade's fallback — because an oracle derived from the thing under test agrees with it by construction. The comparison is byte-for-byte over the native-endian `f32` run and runs **before** any other claim the binary makes.

It is deliberately not `mul_add`: the region declares `(a * b) + c`, a multiply and an add with a rounding between them, and a fused multiply-add rounds once and can differ in the last bit. The operands are chosen so every product and sum is exactly representable, which is what makes a bit-for-bit comparison a statement about the dispatch rather than about rounding.

**The facade's fallback is not a second oracle, and the run says so.** `tiler::__private::bind_and_build` checks the region's operands and calls the adapter's `build` for the declared result; nothing in the facade evaluates the expression on the host. The binary runs the same region without `deliver` and reports what that establishes — the same rank, stored scalar, and extents — and reports its storage as `[0.0, 0.0, 0.0, 0.0]` so no reader mistakes it for a computed comparison.

## The post-commit failure, watched failing

`--halt-after-commit` selects `adapter::Perturbation::HaltAfterCommit`. It perturbs the *adapter* and nothing else: the same device, the same region, the same operands, and the same encode. What it withholds is the submission — the command buffer is neither committed nor waited — which leaves it live and non-terminal. That state is reached only after `Preflight::commit`, because `RuntimeAdapter::dispatch` is where it lives.

The terminal `Error` state is deliberately **not** injected, and the boundary is stated rather than left as apparent coverage: forcing a command buffer into `Error` means provoking a GPU fault, which risks a device reset and would not reproduce. The `Error` arm is classified by `adapter::submission_outcome`, whose match over `MTLCommandBufferStatus` is exhaustive and wildcard-free.

Verbatim, `cargo run --release -- --halt-after-commit`, first four lines:

```
device: Apple M4 Max
mode: perturbed — the submission is halted after the routing commit
post-commit failure, as required: adapter.dispatch: the committed route did not complete, and no fallback follows: metal.dispatch: the command buffer is NotEnqueued, which is not a terminal state, so nothing was read back
no value was returned: the halted dispatch's result storage never reached the caller, so nothing could be mistaken for the semantic fallback's answer
```

The rest of the output is identical to the sound run except that the `committed route completed` note is absent, which the binary checks. Three properties hold together and the run refuses if any does not: the route still reached the `dispatch` stage, so this is a *post*-commit case rather than a refusal; the region surfaced `BindError::DispatchFailed`; and no value was returned at all, so neither the halted result storage nor the semantic fallback's value could be mistaken for an answer.

## Every check was watched failing

Each perturbation below was applied to the working tree, run, and reverted — except the fourth, which needed no applying, because it is the state the checked-in spike was already in.

| check | perturbation | observed |
|---|---|---|
| the oracle | `left * right + addend` → `left * right - addend` | `ORACLE DISAGREES: the kernel wrote [6.5, -5.0, -2.0, 1.0] and this consumer's own arithmetic gives [5.5, -7.0, -6.0, 7.0]`, exit `1` |
| the readback delivers the value | `buffer::read_into` removed from `dispatch` | `ORACLE DISAGREES: the kernel wrote [0.0, 0.0, 0.0, 0.0] and this consumer's own arithmetic gives [6.5, -5.0, -2.0, 1.0]`, exit `1` |
| the post-commit refusal | `--halt-after-commit` | `BindError::DispatchFailed`, no value returned, exit `0` for the perturbed mode |
| a region must state a contract | both `contract` statements removed | two expansion errors, one per region, each opening "this region states no numerical contract, so what its arithmetic means is undecided" and naming the five admissible names, reported at the invocation; `cargo run` exit `101` |
| the stated contract reaches the compiler | `flush_subnormals_to_zero_f32` → `strict_f32` on the delivering region | one expansion error, "the compiler recognizes this region and found no feasible plan for the declared Metal target profile under the `strict_f32` numerical contract this region states", reported at the `deliver` keyword; `cargo run` exit `101` |

The readback row is what proves the kernel's answer reaches the consumer through the readback and not from anywhere else: without it the region returns the fallback's zero-filled declared result. There is deliberately **no** separate "the result is not all zeros" check, because the oracle already refuses that state for this region and a check that cannot reach a verdict the first did not reach is a check nothing could watch fail.

The two contract rows are what make the `contract` statement load-bearing rather than decorative. The first of them is not a hypothetical perturbation at all — it is a recording of the state this spike was actually in at `93e253d`, before the statement was added. The second shows that the name a region writes is carried into the compile request and refused there by the target's own measured behaviour, with the consumer's own word quoted back in the diagnostic.

## The one `unsafe` site

`src/buffer.rs::read_into`, and nothing else. ADR 0079's four conditions are each visible in that file:

1. **No safe route through the foreign API.** `metal` 0.33.0 publishes exactly one storage accessor, `Buffer::contents(&self) -> *mut std::ffi::c_void` (`metal-0.33.0/src/buffer.rs:24`) — no slice accessor, no typed view, no copy-out. The *upload* half needs no site at all, because `Device::new_buffer_with_data` (`metal-0.33.0/src/device.rs:1956`) is a safe function.
2. **An `#[allow(unsafe_code, reason = …)]`**, naming why the site exists and what bounds it.
3. **An assertion against the foreign object's own report** — `buffer.length()`, not a length this crate computed twice, which is the disagreement a read past the mapping is made of.
4. **A `SAFETY` comment** naming the pointer's validity extent, the plain-old-data element type, the non-overlap, and the happens-before the observed `Completed` status supplies.

Every other Metal call this spike makes — device creation, library loading, function lookup, pipeline construction, allocation, encoding, dispatch, submission, waiting, and the terminal-status read — is a safe call and lives in `src/adapter.rs`.

## Unsupported cases and measurement boundary

- **Live-device route requirements are refused, not answered, and that is now a deliberate interim rather than an open gap.** The region delivered here declares none — `observe-live-device` is absent from the stage list, which is what zero rows looks like — so nothing in that method ran on this transcript. Both arms answer `LiveDeviceObservation::Unrecognized`, which is fail-closed: the loader refuses the route and the region takes its declared result. The `tiler.metal.route-requirement.minimum-gpu-family` row *is* answerable from `MTLDevice::supportsFamily`, but its payload vocabulary is `tiler_metal::applicability::MetalGpuFamily` and a consumer may not name an internal crate; spelling the family names again here would mint a second authority over a governed vocabulary. [Backend-scoped route-requirement answers](../../../docs/research/runtime/backend-scoped-route-requirement-answers.md) is the design record that derives the channel, and it records fail-closed as the explicit interim while the design is unimplemented — so `Unrecognized` here is the correct answer rather than a placeholder, and it stays correct until a backend publishes an answer surface a consumer may reach. **That surface is a public-boundary question for Tom rather than something to work around locally**; the record's own finding is that the neutral answer channel already exists and works, and what a consumer is missing is the payload decoder, not a channel.
- **One entry, four bindings, no shared allocations — for *this* region.** The multi-entry and shared-allocation paths in `plan_dispatch` and `allocate_dispatch` are written and compiled but not exercised by the pointwise region above. They are exercised on this device by the consumer in [A two-entry bundle dispatched on Metal hardware](#a-two-entry-bundle-dispatched-on-metal-hardware) below, which reaches them through the same adapter; `crates/tiler-runtime/tests/adapter_route` remains where the *failure* classifications on those paths have watched evidence, against a host interpreter.
- **The post-commit allocation failure is unwatched here.** `DispatchFailure::UndersizedStorage` is reachable only from an allocator that returns less than a length it accepted, and Metal's does not on this host — provoking one is not something this spike can do without lying about what it allocated. `crates/tiler-runtime/tests/adapter_route::a_shared_allocation_shorter_than_the_plan_sized_it_fails_after_the_commit` is where that classification has watched evidence, against a host interpreter whose allocator this repository controls. What this run does establish about the split is the *stage order*: `allocate-dispatch` appears after `plan-dispatch` and before `dispatch` in the transcript above, which is the ordering ADR 0051 asks for.
- **No performance claim.** Nothing here is timed, warmed up, or repeated. It is a correctness artifact.
- **One host.** Every statement above is about the machine in the table, at that OS build, with that Metal compiler and that device. `metallib` translation during pipeline creation remains `Unknown` under ADR 0086 on every macOS row currently observable, so a completed dispatch is not eligibility and this spike claims none.

## A two-entry bundle dispatched on Metal hardware

The second consumer, `src/multi_entry.rs`, built by `cargo run --release --bin multi-entry-dispatch-spike`. It writes

```rust
let x: Tensor<Metal> = Tensor::new(HostTensor::f32_dense(&[1, 4], &X), Rc::clone(session));
tiler::tensor! {
    in x: f32[rows: 1, cols: 4];
    deliver macos;
    contract flush_and_reassociate_f32;
    out strict_serial_sum(x * 2.0 + 1.0, [cols])
}
```

and receives what **two** Metal kernels wrote, in the order the artifact declares them.

### What was missing that this supplies

The producer half already had evidence. `tiler-macros`' `a_split_selection_packages_every_entry_in_the_one_embedded_artifact` asserts that the artifact one expansion embeds carries two entries and the edge `(0, 1)` between them, and `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs` states this exact region in an out-of-tree consumer crate. The multi-entry *route* had evidence too, in `crates/tiler-runtime/tests/adapter_route`, against a fixture artifact and a host interpreter.

**Nothing showed a bundle a macro produced actually running on a device.** The in-tree consumer cannot get there and the reason is correct rather than incidental: `crates/tiler/tests/facade/pass/inline_region_dispatches.rs` refuses at `validate_payload`, which `route_with_adapter` calls per entry and stops at the first refusal, so that consumer observes entry 0 and never the count. ADR 0090 item 8 places payload validation on the backend, a `trybuild` fixture cannot declare the `metal` crate, and returning `Ok` there would be claiming the bytes decode into something executable. Only a consumer that really executes a `metallib` reaches `prepare_entries`, `plan_dispatch`, and `dispatch`.

### The split is the compiler's answer, not this file's request

**Fact.** Nothing in the region asks for two kernels. `strict_serial_sum` states a computation and `flush_and_reassociate_f32` states what its arithmetic may do; the selection policy answers with a split. Under `flush_subnormals_to_zero_f32` the same text selects one fused kernel, which is the perturbation the table below uses, and this run reproduces both halves of that pair on hardware.

**Fact.** This consumer therefore never touches a plan portfolio. Handing the artifact builder a non-selected alternative found by searching for an unfused one is the path the 2026-08-01 run rejected, and that rejection stands: a hand-picked plan would be evidence about this file rather than about the compiler.

**Measurement — `[rows: 1, cols: 4]` is the window, not a taste.** Measured on `BoundMetalCompileDeclaration::first_macos_apple9` under this contract at the run's date, `[rows: 1, cols: 8]` and `[rows: 2, cols: 4]` were refused as `NoFeasiblePlan` — a regrouping-permitting contract withheld the whole-program fused plan, so a portfolio with no admissible split had no plan at all — and `[rows: 1, cols: 5]` was refused as `InvalidCompilerOutput`. Those three refusals are dated observations of the declaration this run bound, not the current boundary: the grid-axis row has since widened to a retained measurement and the declined-strategy record was corrected, so the wider windows now plan and are simply unmeasured. The dispatch recorded here happened at `[1, 4]` and stays true. Widening the measured window belongs to `calibrate-and-activate-parallel-reduction-selection` and the reduction-strategy work.

### The reordering runs first, and is not a flag

The one-entry consumer puts its perturbation behind `--halt-after-commit`, so its checked-in state is sound. This one does not, and the difference is deliberate: a completed two-entry dispatch that agrees with the oracle is evidence about *ordering* only if the same binary can watch the ordering fail, and a check that runs only when someone remembers to pass an argument is a check the checked-in state does not make. So one invocation runs the route twice on the same device with the same operands — reversed first, then in the artifact's declared order — and refuses the run if the reversed one did *not* answer wrongly.

`adapter::Perturbation::ReverseEncodeOrder` perturbs the encode order and nothing else, which is exactly the ordering guarantee `RuntimeAdapter::dispatch` documents: Metal orders *encoders* within one command buffer unconditionally, so the order this adapter creates them in is the order the entries execute in. On a one-entry route the reversal is the identity, which is why only a route the compiler actually split can watch it fail — and why that arm is unreachable from the first consumer.

This mirrors `crates/tiler-runtime/tests/adapter_route`'s `dispatching_the_two_entries_out_of_order_returns_a_wrong_answer_rather_than_a_refusal`, which is the same failure against a host interpreter. It is the one place in this stack that fails **open**: nothing refuses, every payload validates, every pipeline builds, both entries reach terminal success, and the bits are wrong.

### The entry count is counted, not assumed

Three independent populations, all reported on one line and all required to be `2`:

- how many payloads the loader handed this adapter to validate, counted from `validate-payload` occurrences in the recorded stage list;
- how many entries the committed route declares, read from the adapter's own `Completion`;
- how many command encoders the adapter created for it.

The state this guards against is a single-entry plan that happened to be selected. Such a run would dispatch, complete, and agree with the oracle while establishing nothing about a bundle — the perturbation table below shows exactly that state, and shows this check refusing it. The count is read from structured journal fields rather than parsed out of the completion sentence, because a consumer asserting a number should read a number.

One further pin sits beside the count: the plan's **one** shared allocation, the scratch the reducing entry reads and the mapping entry writes. It is asserted rather than merely reported because it is what makes the entries ordered at all — two entries touching no common buffer have no order to get wrong, and the reordering above would then prove nothing.

### The oracle, and why bit-for-bit is legitimate under a reassociating contract

`multi_entry::oracle` is plain Rust `f32`, written left to right the way a consumer reading `strict_serial_sum` would write it, and derived from nothing Tiler produced. The comparison is byte-for-byte over the native-endian `f32` run and runs **before** any other claim the sound half makes.

The stated contract permits the compiler to regroup the reduction's operand sequence, and the selected plan exercises that freedom — it maps and partially reduces four contributors in entry 0 and finishes in entry 1, rather than summing left to right. A bit-for-bit comparison is therefore a statement about the dispatch only if **every** grouping produces the same bits, so `X = [0.5, 1.25, -2.0, 3.25]` is chosen to make that true rather than assumed to be harmless.

The argument is finite and does not depend on which split the compiler chose. Each operand is an integer multiple of `0.25`; `x * 2.0` is exact because two is a power of two, and `+ 1.0` leaves every mapped contributor an integer multiple of `0.5` — `[2.0, 3.5, -3.0, 7.5]`. Every sum of a subset of those four is an integer multiple of `0.5` with magnitude at most `13.0`, so it needs at most five significand bits against `f32`'s twenty-four. No partial sum in any association rounds, so no association can disagree, and the answer is `10.0` under all of them. Nothing in the data is subnormal, infinite, or `NaN`, so the flushing half of the contract is inert here too — deliberately, because this run measures entry ordering and not the numerical contract's own boundary.

### Transcript, 2026-08-04

Verbatim, `cargo run --release --bin multi-entry-dispatch-spike`, on the 2026-08-04 column of [Host and toolchain](#host-and-toolchain) above, at `2c4d05c`. Exit `0`.

```
device: Apple M4 Max
region: in x: f32[rows: 1, cols: 4]; deliver macos; contract flush_and_reassociate_f32; out strict_serial_sum(x * 2.0 + 1.0, [cols])
--- perturbed: the committed route's entries are encoded back to front ---
reordering: WRONG ANSWER, not a refusal — the route completed and the kernel wrote [0.0] where this consumer's own f32 gives [10.0]
entries: 2 payload(s) validated, 2 declared by the committed route, 2 encoded; 1 shared allocation(s)
stage: bind
stage: validate-payload
stage: validate-payload
stage: prepare-entries
stage: observe-prepared-entry
stage: observe-prepared-entry
stage: plan-dispatch
stage: allocate-dispatch
stage: dispatch
handover: x = [0.5, 1.25, -2.0, 3.25]
handover: out = 4 byte(s) to write
entry 0: symbol "tiler_kernel_393f5de6952fd574", 7174 object byte(s), 2 binding(s), launch 4×1
entry 1: symbol "tiler_kernel_f635c9c18ef7eb80", 7174 object byte(s), 2 binding(s), launch 1×1
plan: 2 entry(ies), 1 shared allocation(s)
committed route completed: 2/2 entry(ies) encoded, terminal status Completed, profile tiler.metal.macos-apple9.msl4-0.f32-bf16.v1
--- sound: the entries are encoded in the order the artifact declares ---
oracle: the dispatched bytes equal this consumer's own f32 arithmetic bit for bit: [10.0]
commit: committed route completed: 2/2 entry(ies) encoded, terminal status Completed, profile tiler.metal.macos-apple9.msl4-0.f32-bf16.v1
result: f32[1], 4 byte(s)
entries: 2 payload(s) validated, 2 declared by the committed route, 2 encoded; 1 shared allocation(s)
stage: bind
stage: validate-payload
stage: validate-payload
stage: prepare-entries
stage: observe-prepared-entry
stage: observe-prepared-entry
stage: plan-dispatch
stage: allocate-dispatch
stage: dispatch
handover: x = [0.5, 1.25, -2.0, 3.25]
handover: out = 4 byte(s) to write
entry 0: symbol "tiler_kernel_393f5de6952fd574", 7174 object byte(s), 2 binding(s), launch 4×1
entry 1: symbol "tiler_kernel_f635c9c18ef7eb80", 7174 object byte(s), 2 binding(s), launch 1×1
plan: 2 entry(ies), 1 shared allocation(s)
committed route completed: 2/2 entry(ies) encoded, terminal status Completed, profile tiler.metal.macos-apple9.msl4-0.f32-bf16.v1
DIAGNOSTIC — producer-declared equality against tiler.metal.macos-apple9.msl4-0.f32-bf16.v1, NOT host-earned eligibility
ADR 0086 refuses the host: native `metallib` translation during pipeline creation is a capability fact whose authority is Unknown on every macOS row currently observable, so no host — this one included — earns the right to offer `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`. The route above was settled on producer-declared equality, NOT host-earned eligibility.
```

Four facts are worth reading off it directly.

**The two entries are two kernels of one payload.** Both report `7174 object byte(s)` — the same number because it is the *object* the artifact carries, and one payload carries both entries; the entry *symbols* differ, `tiler_kernel_393f5de6952fd574` and `tiler_kernel_f635c9c18ef7eb80`. That is what "one invocation, one macro-local bundle, more than one GPU kernel" looks like from the consumer's side.

**The launches are the two stages.** Entry 0 launches `4×1`, one thread per contributor along `cols`; entry 1 launches `1×1` and finishes the reduction. The stage dependency between them is the one shared allocation.

**The stage list doubles where it should and only there.** `validate-payload` and `observe-prepared-entry` each appear twice, once per entry, while `bind`, `prepare-entries`, `plan-dispatch`, `allocate-dispatch`, and `dispatch` appear once — `prepare_entries` takes the whole slice and builds every pipeline before any deferred property is answered, which is what makes a second entry whose pipeline will not build a refusal rather than a discovery between two dispatches.

**The declared interface did not move.** The result is `f32[1]`, four bytes: `f32[rows]`, one rank below the operand, exactly as the fused plan's would be. A multi-entry bundle changes what is packaged and never what the region promises.

### The two-entry checks, each watched failing

Each perturbation was applied to the working tree, run, and reverted. Exit code `1` in every row; the checked-in state exits `0`.

| check | perturbation | observed |
|---|---|---|
| the oracle | `value * 2.0 + 1.0` → `value * 2.0 - 1.0` in `multi_entry::oracle` | `ORACLE DISAGREES: the kernel wrote [10.0] and this consumer's own arithmetic gives [2.0]`, with the two byte runs printed |
| the readback delivers the value | `buffer::read_into` removed from `adapter::dispatch` | `ORACLE DISAGREES: the kernel wrote [0.0] and this consumer's own arithmetic gives [10.0]` |
| the entry count, **and** the shared pairing, **and** the reordering's observability | `contract flush_and_reassociate_f32;` → `contract flush_subnormals_to_zero_f32;` on the region | `THE REORDERING WAS NOT OBSERVABLE: … still produced [10.0]`, then `THE SELECTED PLAN IS NOT THE SPLIT: … reports EntryCensus { validated: 1, declared: 1, encoded: 1 }`, then `THE ENTRIES DO NOT SHARE STORAGE: … reports Some(0) shared allocation(s)` |
| the reordering is actually applied | `adapter::reverses_encode_order`'s `ReverseEncodeOrder` arm → `false` | `THE REORDERING WAS NOT OBSERVABLE: encoding the entries back to front still produced [10.0]` |

The third row is the one that carries the section. It is not a synthetic perturbation of an expectation — it is the *other* admissible contract, stating a real and narrower meaning for the same text, and the compiler answers it with the fused one-kernel plan. That run dispatches, completes, and **agrees with the oracle**: `1/1 entry(ies) encoded`, `[10.0]`, exit `1` only because the census refused it. So the oracle alone cannot distinguish a bundle from a single kernel, which is precisely why the count is asserted. The three diagnostics are printed together rather than short-circuited, because this is the only perturbation that reaches the second and third of them.

The second row is what places the sound run's `[10.0]` in the readback rather than anywhere else. Without it the region hands back the facade's zero-filled declared result, which matters more here than it did for the pointwise region: the *reordered* run's wrong answer is also `[0.0]`, and this row is what shows those two zeros have different causes.

### The two-entry run's unsupported cases and measurement boundary

- **The reordered run's value is not asserted, only its disagreement.** The reducing entry reads a shared allocation the mapping entry has not written yet, and Metal does not specify the contents of freshly acquired `StorageModePrivate` storage. `[0.0]` is what this host produced; it is a host observation and not a contract, so the check requires only that the bytes differ from the oracle's.
- **A partial-execution failure across two entries is unwatched here.** "One entry reached terminal success and the next did not" is a distinct classification, and provoking it on Metal means provoking a GPU fault. `crates/tiler-runtime/tests/adapter_route::a_halt_in_the_second_entry_is_a_post_commit_failure_naming_that_entry` is where it has watched evidence, against a host interpreter whose execution this repository controls. The `--halt-after-commit` perturbation is not offered by this binary either; it withholds the whole submission and would say nothing new about a second entry.
- **`UndersizedStorage` on the shared allocation is unwatched here**, for the same reason the pointwise consumer records: Metal's allocator does not return short on this host, and `adapter_route`'s `a_shared_allocation_shorter_than_the_plan_sized_it_fails_after_the_commit` is where that classification is watched.
- **One region, one shape, one contract.** `[rows: 1, cols: 4]` under `flush_and_reassociate_f32` was, at the measurement's date, the only window selecting a split on the declaration this run bound, as the measurement above records; the boundary has since widened and only this window has been dispatched. Nothing here is evidence about a three-entry bundle, a plan with more than one shared allocation, or a reduction wide enough to need a different strategy.
- **No performance claim.** Nothing is timed, warmed up, or repeated. Two kernels are not asserted to be faster than one; the compiler chose the split because the fused plan is not admissible under the stated contract, which is a feasibility answer rather than a cost one.
- **One host, and no eligibility.** Every statement here is about the machine in the 2026-08-04 column. `metallib` translation during pipeline creation remains `Unknown` under ADR 0086, so this route was settled on producer-declared equality and the binary prints the facade's own words saying so.
