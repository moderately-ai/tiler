---
id: prototype-candle-metal-adapter
title: Prototype the Candle Metal adapter
status: done
priority: p1
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/candle, implementation/runtime, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: [crates/tiler/tests/workspace_population.rs]
tags: [implementation, integration, candle]
---
## User-visible outcome

A Candle user can run a supported (contiguous, no-autograd) op through Tiler as a custom op — with storage validation, preflight before commit, and wrapper-level fallback — without any Candle type or behaviour leaking into compiler semantics. This is the first external consumer, so it is also the first real test of the consumer-agnostic claim.

Implement the first consumer adapter without contaminating compiler semantics: storage/layout validation, output allocation, device-scoped runtime cache identity, ABI binding, asynchronous lifetimes, preflight before custom-op application, and wrapper-level fallback. Start with the explicit contiguous/no-autograd subset and reject unsupported cases.

## Workspace admission — current facts (verified 2026-07-31)

The owning production crate **is** absent, so this ticket owns its atomic workspace admission and lockfile update. `cargo metadata --no-deps` reports thirteen packages: eleven production crates — `tiler`, `tiler-artifact`, `tiler-build`, `tiler-cache`, `tiler-compiler`, `tiler-ir`, `tiler-macros`, `tiler-metal`, `tiler-metal-aot`, `tiler-reference`, `tiler-runtime` — plus the two prototype members `tiler-prototype-compile` and `tiler-prototype-run`. `ticketsplease.toml:124` maps `implementation/candle` to `crates/tiler-candle/**` and `prototypes/candle-*/**`, neither of which exists. The count moved from the eight recorded here on 2026-07-28 because `tiler-build` and then the frontend pair `tiler`/`tiler-macros` were admitted; none of those is a Candle path, so the admission this ticket owns is unchanged. This ticket already holds the two scopes admission needs: `implementation/workspace` in `scopes` and `implementation/cargo-lock` in `shared_scopes`.

**One thing changed about how that admission is checked.** The Python workspace gate that maintained a member table is gone — `e197176` ("Replace the Python gate with a Makefile of cargo commands") is in `main`, and `make full` is now a list of cargo commands with no separate member inventory. So adding a workspace member is caught by **reading the diff**, not by a gate that knows the expected set. Add the member and the lockfile update in one commit so a reviewer sees both, and note that `make lint` skips `prototypes/` for Clippy while still building and testing it — a prototype adapter is compiled and run by the gate, and only the style pass skips it.

After the real crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.

## Closes when (2026-07-28)

The correctness priorities this adapter sits on are the ones `AGENTS.md` singles out for special scrutiny, so they are the closing criteria rather than a checklist appended to them.

1. **Preflight completes before the custom op is applied.** Every check that can decline — storage contiguity, layout, dtype, device, autograd absence, target availability — runs and commits to a route *before* Candle's custom-op path is entered. A decline discovered after application is not a decline.
2. **Fallback exists only at the wrapper level and only before any program work.** Once the adapter has allocated an output, begun encoding, submitted a command buffer, or failed semantic validation, there is **no fallback** — the failure surfaces as a typed error. A fallback after any of those four points cannot know what state the device is in, and a fallback after semantic validation failure would return a result the compiler refused. The wrapper must also be able to report *which* numerical realization it delivered, or a caller cannot tell a fast path from a fallback.
3. **Command-buffer terminal success is confirmed before any host readback.** No validation, comparison, or returned tensor reads device memory before the command buffer reports terminal success.
4. **The runtime cache is device- and context-scoped in its identity**, not global and not keyed by a name that two devices could share. An entry built for one device must be unusable from another by construction rather than by convention.
5. **Asynchronous resources are retained through their final device use.** Buffers, pipeline states, and encoded arguments outlive the submission that reads them; nothing is dropped on the strength of the host having finished with it.
6. **The contiguous / no-autograd subset is enforced with typed refusals, and everything outside it is refused by name.** An affine-strided layout, a non-contiguous view, an autograd-tracked tensor, or an unsupported dtype produces a typed error naming what was unsupported — never a silent copy, a relayout, or an approximation. `docs/open-questions.md` Q-RUNTIME-002 tracks affine-strided support as explicitly beyond this first profile.
7. **No Candle type reaches `tiler-compiler` or `tiler-ir`.** Reproduce with a dependency check, not by inspection: neither crate's manifest may gain a Candle dependency, direct or transitive through the adapter. This is the guardrail the ticket's first sentence names as "without contaminating compiler semantics", and it is the one that a working prototype most easily violates.
8. **`make full` passes**, with the new member and its lockfile change in the same commit.

## Graph maintenance

- **When the adapter crate first resolves Candle**: `repin-candle-numerical-scope-citation-at-adapter-admission` (p3, depends on this) becomes actionable — do its re-pin as part of your landing or tell the coordinator it is ready. If the resolved revision differs from the Metal provenance contract's citation, that is a separately scoped `contracts/artifacts` correction, not an edit from here.
- **Workspace admission is yours** (see the facts above): member + lockfile in one commit, and remember `make lint` skips `prototypes/` for Clippy only — the gate still builds and tests it.
- **Every unsupported case you reject**: record the rejection class list on this ticket as you go; that list is the seed for the adapter's second iteration and prevents the next worker rediscovering the boundary empirically.

## Outcome

Landed as a **prototype**, `prototypes/candle-metal-adapter` (package `tiler-prototype-candle`), admitted to the workspace with its lockfile change in the same commit. It consumes `tiler-runtime`'s accepted `RuntimeAdapter` seam through `route_with_adapter`, runs real envelopes published by `prototypes/serial-sum-compile`, and compares against the producer's own recorded reference evaluation.

### One declared-path expansion, stated rather than taken silently

The ticket's workspace-admission facts say "adding a workspace member is caught by **reading the diff**, not by a gate that knows the expected set". **That assertion is stale and this landing disproved it.** `crates/tiler/tests/workspace_population.rs` is exactly such a gate: it derives the member set from `cargo metadata --no-deps` and asserts it equals a written-out `EXPECTED_MEMBERS`, and it failed on the first `make full` with `the workspace holds 14 members and this test expects 13`. It was added after the Python gate was removed, so both halves of the ticket's history are true and its conclusion is not.

That file is `implementation/frontend`, which this ticket does not hold. The admission cannot be atomic without it — the alternative is a member landing with a red gate — so the ticket's `paths` now declares that **one exact file** rather than the whole frontend scope, and this note is the record. It is the minimum that makes the admission complete, and a reviewer should confirm nothing else under `crates/tiler/` moved.

### Placement — derived, not defaulted

Both `crates/tiler-candle/**` and `prototypes/candle-*/**` are admitted by `ticketsplease.toml:124`. The prototype wins on three counts and loses on none: a `crates/` member is a public crate boundary, which is one of the three decisions `AGENTS.md` reserves to Tom, and nothing here needed that boundary to be settled; the ticket title says prototype and the artifact it proves is a delivery mechanism rather than an API; and Candle is a heavy external dependency whose long-term place in the workspace is not decided, so admitting it under `crates/` would assert a permanence the evidence does not support. The one argument for `crates/` — that `make lint` skips `prototypes/` — is answered by having run `cargo clippy -p tiler-prototype-candle --all-targets -- -D warnings` clean anyway (evidence below).

**The lint-inheritance question did not arise.** `prototypes/serial-sum-run` diverges from the workspace lints because Metal exposes buffer storage only through a raw pointer. This adapter needs none: `dispatch2::DispatchData::from_bytes` and `MTLDevice::newLibraryWithData:error:` are both safe, Candle's encoder setters are safe wrappers, and read-back goes through Candle's own `to_vec1`. So the member keeps `[lints] workspace = true` with `unsafe_code = "forbid"`, and no second lint-inheritance divergence — which `AGENTS.md` reserves to Tom — was needed.

### Seam consumption, and the one real friction

The adapter implements `RuntimeAdapter` and is driven by `route_with_adapter`; nothing forks around the seam. Every division the seam draws held: the adapter reports and the loader compares, `Unrecognized` is the answer for `RouteResourceDimension::SubgroupThreads` (Metal publishes no device-scoped execution width — `threadExecutionWidth` is a prepared-pipeline fact), payload validation runs from bytes before any device question, and the two error types split exactly where ADR 0051 draws the line.

**Friction — the seam cannot be driven across a foreign framework's callback boundary.** Candle's custom-op contract wants a *prepared selection* held before `apply_op` and only committed inside it, which is what `docs/integration/candle.md`'s `PreparedSelection` token describes. That is not expressible:

- `route_with_adapter` runs stages 1 through 9 in one call and there is no way to stop after stage 7.
- Driving the trait methods by hand instead is impossible, because every one takes `&LiveExecutionContext` and **that type has no public constructor** — `route_with_adapter` mints the only value that ever exists. The property is deliberate, compiler-checked by two `compile_fail` doc-tests, and correct for what it protects; its cost here is that a consumer cannot straddle a callback boundary.

So the device-dependent pre-commit stages necessarily run inside `metal_fwd`. The wrapper answers this by foreclosing the fallback *before* entering the custom op, which is stricter than ADR 0051 and is what criterion 2 actually requires — see below. This is recorded as evidence, not routed around.

**Evidence for the seam's two open sub-questions.** (1) *Context returned to the caller*: yes, and this is the first concrete argument for it. A consumer that could hold a `LiveExecutionContext` across a callback boundary could split stages 1–7 from 8–9 and satisfy criterion 1 literally. A minimal shape would be a `route_prepared(...) -> Result<PreparedRoute<'_, A>, …>` returning a value that owns the context and whose only method is `commit_and_dispatch`; that preserves "no caller mints one" while making the seam usable from a framework that owns the call stack. (2) *Borrowing `Completion`*: no evidence against owning it. `CandleCompletion` carries a `MetalStorage` built from an `Arc<Buffer>` the adapter allocated, which the caller hands straight back to Candle — owned is exactly right, and a borrowed completion would have tied the result tensor's lifetime to the adapter.

### The eight criteria, one by one

1. **Preflight completes before the custom op is applied.** `TilerPlan::preflight` decides autograd, device family, device identity, dtype, aliasing, contiguity, rank, and extents; `TilerPlan::load` decides target availability by classifying every packaged payload's declared profile against the routed environment, and the artifact's declared interface against this wrapper's. All of it runs before `apply_op1_no_bwd`. Watched failing: the run's `probe` lines show each of those refusing its own perturbation, each paired with a `probe baseline` line proving the unperturbed tensor preflights.
   *Limit, stated:* the device-dependent pre-commit stages run inside `metal_fwd`, for the seam reason above. Criterion 1's own enumeration — contiguity, layout, dtype, device, autograd absence, target availability — is entirely on the outside.
2. **Fallback only at the wrapper level, only before program work, with delivered-realization reporting.** The fallback decision is taken in `TilerPlan::apply` before the op is applied, and nowhere else; no `AdapterRouteFailure` is ever converted into a route change. `Delivered` carries the path, the realization, and — obligatorily, per the contract's Diagnostics section — the operations that claim covers. And the fail-closed rule is exercised, not just unit-tested: `fallback_availability` refuses to substitute Candle's kernels for an order-fixing contract, so `plan.apply(&cpu_tensor, …)` returns `NoRealizableFallback` naming both the refusal and the unmet realization. Both arms of that decision are unit-tested device-free.
3. **Terminal success before readback.** The adapter owns its command queue and command buffer, commits, waits, and then reads `MTLCommandBufferStatus`; only `Completed` returns a completion, `Error` and every non-terminal status are typed failures. Watched failing: the run classifies a live, never-committed command buffer as `NotEnqueued, no readback taken`, and a device-free test classifies every named status plus an unnamed one.
   *Why not Candle's stream:* Candle 0.11.0's `Commands::ensure_completed` reads the status **before** the wait and never re-reads it, and `MetalDevice.commands` is `pub(crate)` with no accessor for the in-flight buffer — so a consumer encoding into Candle's stream has no object to ask. `docs/integration/candle.md` permits exactly this ("or the adapter supplies an equivalent checked boundary"). The cost is overlap, and `adopt-candle-command-stream-once-a-terminal-check-is-reachable` carries the trigger.
4. **Device- and context-scoped cache identity.** `DeviceScope` is `(Candle context identity, MTLDevice registryID)` and is a **field of every key**, so an entry built under one scope cannot be found from another — by key inequality, independent of any check running. `PipelineCache::scoped_to` additionally refuses a foreign scope outright, which is the half that can *say no*. Both halves are tested device-free, varying each identifier independently.
   *One thing the ABI decides that a placement cannot:* the encoder declares each slot read-only or write-only from `DecodedBinding::access`, never from what the slot is bound to. A shared intermediate is one allocation two entries address — written by the producer and **read** by the consumer — so deriving the mode from the placement would have declared the consuming slot an output to Candle's hazard tracker. It ran correctly either way, because the explicit inter-encoder fence is what actually orders them, but the declaration would have been a lie the contract forbids.
5. **Asynchronous resources retained through their final device use.** `PlannedRoute` owns an `Arc<Buffer>` per bound slot plus every pipeline, and is alive across the wait. The `Arc` rather than a `Buffer` clone is load-bearing and documented: Candle's allocator recycles a pooled buffer as soon as its `Arc::strong_count` reaches one, so an Objective-C retain alone would let the allocator hand the same live `MTLBuffer` to an unrelated allocation while this route still reads it.
6. **Typed refusals for everything outside contiguous / no-autograd.** The rejection-class list is below; every class is refused by name, and none copies, relayouts, or approximates.
7. **No Candle type reaches `tiler-compiler` or `tiler-ir`.** Proved by `prototypes/candle-metal-adapter/tests/no_candle_reaches_the_compiler.rs`, which walks the **transitive** closure of both crates in `Cargo.lock` and asserts no `candle-*`, `objc2-metal`, or `dispatch2` package appears — with three population checks so it cannot pass vacuously. Both perturbations watched failing (below).
8. **`make full` passes**, with the member and the lockfile in the same commit.

### Rejection-class list (criterion 6)

Tensor-level, decided before the custom op is entered — `crate::refusal::TensorRefusal`, deliberately not `#[non_exhaustive]` because it *is* the profile boundary:

| class | refused case | watched failing |
|---|---|---|
| `NotAMetalDevice` | a tensor on the host device | yes |
| `ForeignMetalDevice` | a tensor on a second Candle Metal device over the same GPU | yes |
| `UnsupportedDtype` | `f16` (and every dtype but `f32`) | yes |
| `AffineStridedLayout` | a view narrowed on the inner axis — Q-RUNTIME-002 | yes |
| `BroadcastView` | a zero-stride aliasing view | yes |
| `UnsupportedRank` | a rank-1 tensor | yes |
| `ExtentMismatch` | a tensor one column wider; also what a transposed 1×N view produces | yes |
| `AutogradTracked` | a `Var` | yes |
| `ForeignInterface` | an artifact declaring other keys, types, arity, or rank | no — no artifact with a differing interface exists to route |
| `IncompatibleTargetProfile` | a host offering another exact profile descriptor | yes |
| `NoRealizableFallback` | any of the above under an order-fixing contract | yes |

**A finding worth carrying forward.** A transposed view of this artifact's declared 1×N input is *not* an affine-stride perturbation: Candle's `Layout::is_contiguous` ignores the stride of any extent-1 axis, so `t()` on a 1×3 yields a 3×1 view Candle still calls contiguous, and it falls through to the extent check. The probe was rewritten to narrow the inner axis of a two-row tensor, which is genuinely non-contiguous. The first version of this probe passed while testing the wrong refusal.

Adapter pre-commit — `RouteRefusal`: `NoExecutionContext`, `PayloadNotALibrary` (watched), `EntrySymbolAbsent` (watched), `PipelineRejected`, `ForeignDeviceScope` (watched, device-free), `WorkgroupTooLarge`, `BindingExceedsBufferLimit`, `UndersizedStorage`, `BindingRangeOverflow`, `UnboundBindingTarget`, `EmptyLaunchNotSkippable`, `NoOutputBinding`, `Allocation`, `PendingCandleWork`. The four pure comparisons were extracted into `binding_fits`, `allocation_holds`, and `workgroup_fits` precisely so the gate can watch them refuse without a device; each is tested at its boundary and one past it.

Post-commit — `DispatchFailure`: `EncoderUnavailable`, `CommandBufferError`, `NonTerminalStatus` (watched). Unlike `prototypes/serial-sum-run`, Metal's own error text is carried: Candle's `CommandBuffer::error` reads the localized description, which the `metal` 0.33 binding does not expose.

### Candle revision resolved, and the re-pin

`Cargo.lock` resolves `candle-core 0.11.0` and `candle-metal-kernels 0.11.0` from crates.io. `huggingface/candle` `31f35b14` is the commit *titled* "Bump candle version to 0.11.0 (#3658)" (verified: `git log --oneline -1 31f35b14` in a local checkout), so the resolved release is the one the contract cites and the citation **agrees**.

A crates.io version pin rather than a `git = … rev = …` pin, derived: the repository's no-vendoring rule is about *forks* ("pin an actively used fork by exact revision"), Candle is upstream and unforked, a registry pin records a content checksum in `Cargo.lock` that is at least as exact as a commit hash, and a git dependency would force network access into `make full` for no additional evidence. If Tiler ever needs a Candle change, that becomes a fork and the rule applies then.

**The three cited line references were re-read at the resolved crate and all hold**: `candle-metal-kernels-0.11.0/src/kernel.rs:109` `load_library`, `:122` `new_library_with_source`, `:182` `get_compile_options` reading `CANDLE_METAL_ENABLE_FAST_MATH` with a default of `true`; `candle-core-0.11.0/src/metal_backend/device.rs:101` `compile`, `:111` `new_library_with_source`. `load_library` still caches by `Source`. None of the three premises the re-pin ticket names as load-bearing has changed.

**`repin-candle-numerical-scope-citation-at-adapter-admission` is now actionable and was NOT done here**: its scope is `contracts/integrations`, which this ticket does not hold, so editing `docs/integration/candle.md` would be a scope escape. It is a one-paragraph restatement — the section's opening "Tiler declares no Candle dependency" paragraph is now false, and the `grep -rn candle --include=Cargo.toml --include=Cargo.lock .` check it cites now matches — with every line reference already verified above.

### Hardware execution evidence

Host: macOS 27.0 build 26A5388g, arm64, Apple M4 Max, GPU family Apple9, registry `0x100000484`. Toolchain: the pinned dated nightly from `rust-toolchain.toml`.

```text
cargo run -p tiler-prototype-compile -- --out /tmp/serial-sum.tiler
cargo run -p tiler-prototype-candle  -- --artifact /tmp/serial-sum.tiler
```

Result: `candle adapter proof: 20 case(s) agreed across 4 of 6 published member(s); 2 member(s) are outside this profile and named above`, exit 0. Both plan roles of both non-empty reduction classes routed: `selected` as one dispatch with no shared allocation, `materialized` as two dispatches through one shared allocation — which is what separates "both agreed" from "both ran the same program twice". ADR 0086's applicability question was asked first and refused, printed as `predicate native-translation-authority, rule metal.host-applicability.unknown-translation-authority`, and the routed environment is labelled `PRODUCER-DECLARED EQUALITY, NOT HOST-EARNED ELIGIBILITY` in those words.

A contiguous view at a **nonzero** start offset is accepted and agrees, which is the positive half the refusal probes cannot supply: the adapter composes Candle's element start offset with the artifact's own accessible offset, and never binds offset zero merely because it holds the buffer.

### Unsupported cases and what was deliberately not done

- **Zero-extent programs.** Both `empty-domain` members are unroutable: `Tensor::from_vec(vec![], (1, 0), &metal)` fails with `Failed to create metal resource: Buffer` because Candle's allocator asks for a zero-length `MTLBuffer`. The failure is upstream of every refusal this adapter owns — there is no tensor to preflight — and the proof names it rather than skipping it. Filed as `route-a-zero-extent-program-through-candle-metal-storage`.
- **ADR 0090 item 8's third obligation** — that the payload's slots are the ones the entry declares — is **not** discharged. It needs `MTLComputePipelineReflection`, which Candle's pipeline wrapper discards. Filed as `validate-metal-payload-argument-slots-against-declared-bindings`; today a slot disagreement reaches the encoder rather than a refusal, which is the one path in the adapter that does not fail closed.
- **Asynchronous encoding into Candle's stream** — deferred with its trigger, as above.
- **Multi-input custom ops.** `CustomOp1` only; the contract's three-input limit is untouched.
- **`ForeignInterface`** is unexercised by probe, for want of an artifact with a different interface.
- **An exhaustive wildcard-free match over `MTLCommandBufferStatus`** is not expressible: `objc2-metal` models it as a `#[repr(transparent)]` newtype with associated constants rather than a Rust enum, so an unnamed status is classified non-terminal by default. `prototypes/serial-sum-run` can write the exhaustive form because `metal` 0.33 gives it a real enum. Stated in the code at the site.

### Public items and workspace changes (provisional-acceptance packet)

No `crates/` boundary moved. Everything below is inside the new prototype and reachable by nothing else, since nothing may depend on a prototype.

- **Workspace**: `Cargo.toml` gains the member `prototypes/candle-metal-adapter`; `crates/tiler/tests/workspace_population.rs` gains `tiler-prototype-candle` and its count moves 13 → 14 (the declared-path expansion above); `Makefile`'s `lint` target gains `--exclude tiler-prototype-candle`, matching the existing prototype-class exclusion; `ticketsplease.toml`'s `[scope_crates]` note records that the Candle scope still carries no crate mapping, deliberately.
- **New external dependencies**, macOS-only: `candle-core 0.11.0` (feature `metal`), `candle-metal-kernels 0.11.0`, `objc2-metal 0.3.1`, `dispatch2 0.3.1`.
- **Crate-internal public surface**: `refusal::{Realization, Delivered, DeliveredPath, FallbackAvailability, fallback_availability, TensorRefusal, RouteRefusal, DispatchFailure}`; `cache::{DeviceScope, LibraryKey, PipelineKey, PipelineCache}`; `adapter::{CandleMetalAdapter, CandleCompletion, DeviceFacts, BoundInput, SubmissionOutcome, submission_outcome, load_library, observed_apple_family, bind_candle_storage, binding_fits, allocation_holds, workgroup_fits, INPUT_KEY, OUTPUT_KEY}`; `wrapper::{TilerPlan, Applied, RouteReport, WrapperError, candle_expression}`.

### Cargo.lock delta

`candle-core`, `candle-metal-kernels`, `candle-ug`, `objc2-metal`, `dispatch2` and their transitive closure (`gemm*`, `half`, `float8`, `safetensors`, `yoke`, `zip`, `tokenizers`, `objc2*`, `block2`, …) enter the graph, reachable only from `tiler-prototype-candle`. Everything resolved from the local registry cache; no network fetch was required.

### Commands and results

- `cargo run -p tiler-prototype-compile -- --out /tmp/serial-sum.tiler` — 6 members published.
- `cargo run -p tiler-prototype-candle -- --artifact /tmp/serial-sum.tiler` — exit 0, 20 cases agreed, output above.
- `cargo clippy -p tiler-prototype-candle --all-targets -- -D warnings` — clean, with the workspace's `pedantic` set, despite the member being excluded from `make lint`.
- `cargo nextest run -p tiler-prototype-candle` — 12 tests, all passing, all device-free.
- **Deliberate failure perturbations, both watched:** adding `tiler-prototype-candle` to the dependency test's neutral list fails with `` `tiler-prototype-candle` must stay consumer-agnostic, and its resolved dependency closure contains: ["candle-core", "candle-metal-kernels", "candle-ug", "dispatch2", "objc2-metal"] ``; replacing the Candle prefix list with one matching nothing fails the population check with `no Candle package is in the parsed lockfile at all, so this test would pass whether or not the compiler depended on one`. Both restored.

**Provisional boundary acceptance (2026-08-01, overnight mode).** The coordinator provisionally accepted the prototype workspace member `tiler-prototype-candle` (prototype placement deliberately avoids the reserved public-crate decision) and recorded the seam-friction report — the `route_prepared` shape answering the adapter seam's first open sub-question — for Tom's morning review.
