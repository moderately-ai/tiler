---
id: close-the-serial-sum-run-gpu-family-probe-table
title: Close the serial-sum-run GPU family probe table
status: done
priority: p2
dependencies: []
related: [close-the-metal-gpu-family-out-of-crate-total-map, widen-the-metal-gpu-family-vocabulary-to-apple10]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, api-conventions, metal, adr-0074]
---
## User-visible outcome

`prototypes/serial-sum-run` probes a device for exactly the families `tiler_metal::applicability::MetalGpuFamily::ALL` names, so a family added to that vocabulary cannot leave the proof runner's applicability observation silently one family short.

## Why this is a separate ticket

**Fact.** `close-the-metal-gpu-family-out-of-crate-total-map` closed the same defect at `prototypes/candle-metal-adapter/src/adapter.rs` and named that site as the only one. It was not: `highest_apple_family` in `prototypes/serial-sum-run/src/proof.rs` (lines 703-716 at `cb5d86a`) carries the identical five-element pair table:

```rust
[
    (MTLGPUFamily::Apple9, MetalGpuFamily::Apple9),
    (MTLGPUFamily::Apple8, MetalGpuFamily::Apple8),
    (MTLGPUFamily::Apple7, MetalGpuFamily::Apple7),
    (MTLGPUFamily::Apple6, MetalGpuFamily::Apple6),
    (MTLGPUFamily::Apple5, MetalGpuFamily::Apple5),
]
```

**Fact — it is a different scope.** `prototypes/serial-sum-run/**` maps to `implementation/runtime` in `ticketsplease.toml`, and the closing ticket declared `implementation/metal` and `implementation/candle` only.

**Fact — and it is not the same fix, which is the substantive reason it was not absorbed.** The candle adapter binds Metal through `objc2-metal` 0.3.2, where `MTLGPUFamily(pub NSInteger)` is a public tuple newtype, so `tiler_metal::applicability::AppleGpuFamilyConstant::value()` crosses into it directly and the whole correspondence disappears. This runner binds Metal through `metal` 0.33.0, where `MTLGPUFamily` is a `#[repr(i64)]` **Rust enum** with no `TryFrom` and no safe constructor from a raw value (`metal-0.33.0/src/device.rs:70-89`, `supports_family` at `:1629`). A raw enumerator therefore has to be named back into that enum by hand here, and the crate forbids the transmute that would avoid it (`unsafe_code = "deny"`, one admitted site, and ADR 0079's first condition is not met because a safe route exists).

## Implementation keys

- Drive the probe from `tiler_metal::applicability::observe_highest_gpu_family` so the *population* is the vocabulary's, exactly as `prototypes/candle-metal-adapter/src/adapter.rs::observed_apple_family` now does.
- Decide what the residual `isize -> metal::MTLGPUFamily` step does with an enumerator this binding does not name, and make it loud. `metal` 0.33.0 stops at `Apple9` while the macOS 26.5 SDK declares `MTLGPUFamilyApple10 = 1010`, so the case is reachable rather than theoretical the moment `widen-the-metal-gpu-family-vocabulary-to-apple10` lands. Returning `false` is the defect being closed wearing different clothes: it answers "this device does not support that family" to a question that was never asked.
- Two shapes are worth weighing, and the choice is a public-boundary consequence to state rather than assume. Either the runner refuses locally — it stops calling `observing_gpu_family`, so the policy reports `MetalHostApplicabilityRefusal::Unobserved { predicate: GpuFamily }`, which is precisely the typed outcome that exists for "an adapter did not ask" — or `tiler-metal` grows a fallible probe (`observe_highest_gpu_family` returning `Result`, or a closure returning `Option<bool>`) so every binding of this shape gets the channel rather than each inventing one. The first needs no new public surface; the second is the general answer and is Tom's to accept.
- Whatever is chosen, add the check that can say no and watch it fail: the population this runner probes must be compared against `MetalGpuFamily::ALL` with a **literal** count, and verified by adding a sixth family locally and watching the count assertion reject it.

## Explicit non-goals

- **Do not widen `MetalGpuFamily`.** `widen-the-metal-gpu-family-vocabulary-to-apple10` owns that, and it is a measurement question rather than a transcription.
- **Do not implement the answer surface** proposed in `docs/research/runtime/backend-scoped-route-requirement-answers.md`.

## Closes when

No workspace consumer pairs a `MetalGpuFamily` variant with an Apple enumerator by hand, an enumerator this runner's binding cannot name is refused rather than answered `false`, the counted-population check has been watched failing, and `make full` is green.

## Outcome

**Fact — the pair table is gone and the population is the vocabulary's.** `prototypes/serial-sum-run/src/proof.rs::highest_apple_family` is replaced by `probe_apple_families`, which drives the walk from `tiler_metal::applicability::observe_highest_gpu_family` exactly as `prototypes/candle-metal-adapter/src/adapter.rs::observed_apple_family` does. Nothing in the workspace now pairs a `MetalGpuFamily` variant with an Apple enumerator by hand.

**Fact — the residual `isize -> metal::MTLGPUFamily` step is a join on Apple's own number, not a second pairing.** `BINDING_APPLE_FAMILIES` lists the nine Apple enumerators `metal` 0.33.0 names (`Apple1`–`Apple9`), and `binding_apple_enumerator` finds one by comparing `enumerator as isize` against `AppleGpuFamilyConstant::value()`. Both sides transcribe the same `MTLDevice.h` number, so there is no correspondence to keep in step — and the list is not the population, which remains `MetalGpuFamily::ALL`. The list is hand-written because it has to be: `metal`'s enum is `#[non_exhaustive]`, publishes no iteration, and offers no `TryFrom`.

### The shape chosen, with the elimination

**The first shape — local refusal via `Unobserved` — is implemented; the general channel in `tiler-metal` is filed for Tom as `decide-the-unnameable-gpu-enumerator-channel`.** The sequencing is deliberate and was directed by the dispatch: the second shape is an `implementation/metal` edit *and* a public-boundary addition to an ADR 0074 §7 draft surface, the metal scope was held by a live sibling, and the general answer is Tom's either way. Implementing the local shape now leaves this runner fail-closed regardless of how that question is answered.

Eliminated on correctness, not on cost:

- **Answering `false` on an unnameable enumerator.** It reports a question nobody asked as a device that answered no. Watched producing `GpuFamilyMismatch { required: Apple9, observed: NoneNamed }` — the defect this ticket closes, wearing different clothes.
- **Converting the raw value inside `tiler-metal`.** The crate names no Metal runtime type; doing so would bind the compiler crate to one consumer's choice of binding.
- **A transmute.** ADR 0079's first condition is unmet — a safe route exists — and a value that is not a valid enumerator would be UB rather than a lint question. The crate's `unsafe_code = "deny"` is untouched and its one admitted site is unchanged.

`ProbedGpuFamily` carries the two outcomes, because "the device named no family this vocabulary knows" and "this binding could not ask" have different repairs — a host to change versus a Metal binding to upgrade.

### The loud refusal, at both consumers

- **Host applicability.** `stating_probed_family` leaves the predicate *unstated* on `Unnameable`, so `evaluate_metal_host_applicability` answers `MetalHostApplicabilityRefusal::Unobserved { predicate: GpuFamily }` — the typed outcome that already exists for an adapter that did not ask. Calling `observing_gpu_family` with anything at all would be the adapter claiming it asked.
- **Route requirements.** `decide_live_device_requirement` returns `LiveDeviceObservation::Unrecognized`, which refuses the route. `Feature(false)` would refuse it too, and would refuse it as a device that answered no.
- **Both prints** now go through `impl Display for ProbedGpuFamily`, which names the exact enumerator: `unobserved: the governed vocabulary names MTLGPUFamily 1010, which this binding cannot name, so this device was never asked`. This also removed a second spelling — the preflight line said "no Apple family reported" where the applicability line said "no named Apple family" for the same fact.

One unnameable enumerator discards the **whole** walk, not only its own query. `observe_highest_gpu_family` walks highest first and stops at the first supported family, so a family above the one that answered would leave `Highest(lower)` an understatement wearing the shape of a most-specific claim.

### The counted-population check, watched failing

`const _: ()` beside `binding_apple_enumerator` asserts `MetalGpuFamily::COUNT == 5` (the literal) and then sweeps `MetalGpuFamily::ALL`, asserting each family is nameable by this binding.

**Measurement — the perturbation, in this worktree, reverted.** Adding `Apple10` to `MetalGpuFamily` (variant, `ALL`, `as_str`, `apple_constant() = 1010`) and running `cargo check -p tiler-prototype-run --all-targets --locked`:

```
error[E0080]: evaluation panicked: this runner expects the governed vocabulary to name five Apple
families; `metal` 0.33.0 stops at Apple9, so a widened vocabulary needs a newer binding here before
the count is raised
   --> prototypes/serial-sum-run/src/proof.rs:772:5
```

Bumping the literal to `6` — the naive repair — does **not** make it pass, which is the point of the second half:

```
error[E0080]: evaluation panicked: `metal` 0.33.0 cannot name an Apple enumerator
MetalGpuFamily::ALL declares, so this runner would leave the GPU-family predicate unobserved on
every host
   --> prototypes/serial-sum-run/src/proof.rs:780:9
```

`crates/tiler-metal/src/applicability.rs` was restored and verified byte-identical against a pre-edit copy; `git status` showed `prototypes/serial-sum-run/src/proof.rs` as the only modified file.

### The runtime refusals, each watched failing

Three perturbations, run together, each failing for its own reason and no other test failing (30 passed, 3 failed of 33):

- `stating_probed_family`'s `Unnameable` arm changed to state `NoneNamed` → `an_unnameable_enumerator_leaves_the_family_predicate_unobserved` failed with `left: GpuFamilyMismatch { required: Apple9, observed: NoneNamed }`, `right: Unobserved { predicate: GpuFamily }`.
- `decide_live_device_requirement`'s `Unnameable` arm changed to `false` → `a_family_row_is_unrecognized_when_the_binding_could_not_ask` failed with `left: [Feature(false)]`, `right: [Unrecognized]`.
- The Apple10 reachability pin's `1010` changed to `1009` → `this_binding_cannot_name_the_family_apple_declares_above_its_last` failed as written.

### Measurement boundary and unsupported cases

- **No device was involved.** Every new test is device-free; `make full` reaches no Metal device, and the hardware run was not performed by this ticket. What is proved is about the *adapter* — which families it asks about, and what it answers when it cannot ask — not about any host's actual family support.
- **The `Unnameable` runtime path is not reachable in this build**, and the docs say so rather than implying otherwise: the compile-time assertion rejects the vocabulary state that would produce it. The path exists because an assertion is a claim about one build and can be relaxed in one line, while what the probe *answers* must stay fail-closed on its own. It is exercised by constructing `ProbedGpuFamily::Unnameable` directly in tests.
- **`ProbedGpuFamily` is private to the prototype.** No public boundary was added or changed by this ticket.

### Non-goals honoured

`MetalGpuFamily` was not widened (the perturbation was reverted and verified byte-identical), and no answer surface from `docs/research/runtime/backend-scoped-route-requirement-answers.md` was implemented.

### Verification

`cargo fmt --all`; `cargo check -p tiler-prototype-run --all-targets --locked`; `cargo nextest run -p tiler-prototype-run -p tiler --locked` (66 passed — includes `crates/tiler/tests/labelled_diagnostic.rs`, whose pinned producer-declared-equality substring was not touched); `make full`; `tkt lint`; `git diff --check`; `tkt guard --base a5e9886`.
