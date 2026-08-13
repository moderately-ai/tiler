---
id: decide-whether-the-candle-adapter-may-synthesize-zero-extent-storage
title: Decide whether the Candle adapter may synthesize storage for a zero-extent tensor
status: done
priority: p3
dependencies: []
related: [route-a-zero-extent-program-through-candle-metal-storage, prototype-candle-metal-adapter]
scopes: [implementation/candle, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [candle, runtime, decision, needs-tom, public-boundary]
---
## The decision

May a Tiler runtime adapter allocate storage that the caller's own tensor does not have?

The concrete case is the `empty-domain` member of the serial-Sum matrix. Its artifact declares a `1 x 0` input; a reduction over zero contributors reads that buffer never and publishes the reduction's identity element. `route-a-zero-extent-program-through-candle-metal-storage` closed by refusing it — `TensorRefusal::ZeroExtentInterface`, decided from the artifact's declared extents before any Candle tensor is asked for — because at Candle 0.11.0 no tensor of that shape exists to bind. The refusal is correct and fail-closed. It is not the only correct answer available.

The alternative is that the adapter synthesizes the missing storage: a one-byte placeholder buffer bound at a **zero-length accessible range**, so the kernel is handed an address it is never permitted to read. That would let the empty-domain member route and agree with the producer's recorded reference evaluation, which the refusal branch cannot do.

## Why this is Tom's and not a worker's

It moves the consumer boundary rather than an implementation detail. Today every buffer the Candle adapter binds is either the caller's own Candle allocation or storage the *route* declares (`Backing::Output`, `Backing::Internal`, a shared intermediate). A placeholder for an absent caller input is a fourth class: storage that stands in for a value the caller supplied no memory for, bound to a slot whose `BindingTarget` is `ProgramInput`. Deciding that an adapter may do this is deciding what a consumer's tensor means when it has no allocation, and it generalizes past this shape — every future zero-extent operand inherits the answer.

Note the asymmetry with `prototypes/serial-sum-run`, whose `proof.rs` allocations use the source-safe anchor `needed.max(1)`. That prototype owns every buffer it binds, so its placeholder is its own storage and no boundary is crossed. The Candle case is a placeholder for a caller's value, which is the part that needs a decision.

## What the answer must preserve either way

- The accessible range bound to the kernel stays **zero**, so a kernel that read the placeholder would be reading outside the range the artifact declares. A placeholder that widens the accessible range is a different and worse proposal.
- The refusal path stays reachable and typed for any case the placeholder does not cover; "extensible" must not become "unclassified".
- Whatever is decided is recorded where the next reader finds it — an accepted ADR if the answer generalizes across adapters, or `docs/integration/candle.md`'s storage-layout contract if it is Candle-specific.

## Activation trigger

Either of these:

- **Tom decides** the placeholder question in either direction. A "no" closes this ticket by recording the refusal as the standing answer; a "yes" opens the implementation with the invariants above as its acceptance conditions.
- **Candle's Metal allocator admits a zero-length allocation** (or rounds one up to a minimum) so a zero-element tensor exists at the pinned revision. The placeholder question then becomes moot for this shape and this ticket closes without needing the decision — `prototypes/candle-metal-adapter`'s proof detects that transition itself and fails rather than reporting a routable member as refused, so the trigger announces itself.

Do not implement the placeholder before the trigger. A half-taken placeholder — storage synthesized without the accessible-range invariant, or a route that stops refusing without a decision behind it — is worse than either endpoint.

## Decision packet — 2026-08-09

The first trigger is this ticket's purpose, not a reason to hide it from the decision queue. The worked `1 x 0` input, current typed refusal, zero-length accessible-range invariant, and consumer-boundary consequence are all present. The node is therefore `awaiting-decision`.

**Recommendation: keep the typed refusal until Candle itself can represent the zero-element tensor.** Synthesizing storage for an absent caller allocation creates a new input-backing class whose lifetime and aliasing contract every adapter would otherwise have to rediscover. **Strongest counterpoint:** the artifact proves the input is unread and declares a zero accessible range, so a one-byte address-only placeholder can preserve memory safety while enabling a semantically valid empty-domain program today.

Tom may accept the refusal as the standing Candle rule, accept the bounded placeholder with the invariants above, or require a cross-adapter ADR before either implementation changes. A “no placeholder” answer closes this ticket; a “yes” answer returns it to `todo` with the accepted exact boundary.

## Source re-audit and corrected decision packet — 2026-08-12

The earlier packet's central premise was false: the pinned Candle release cannot build the empty tensor through `Tensor::from_vec`, but its public API can represent one without a Tiler-private storage kind. In `candle-core` 0.11.0, `MetalDevice::new_buffer` passes a zero logical byte count through `buf_size`, and `0_usize.next_power_of_two()` supplies a one-byte Metal allocation; `MetalStorage::new` separately records the logical element count and dtype; `Tensor::from_storage` and `From<(Storage, Shape)> for Tensor` publicly construct a contiguous tensor from compatible storage and shape. A `MetalStorage` with count zero and dtype `F32`, paired with shape `[1, 0]`, is compatible because both state zero logical elements. `Tensor::from_vec` fails only because its upload path calls `new_buffer_with_data` with a literal zero byte length and bypasses the rounding allocator.

The adapter therefore need not treat absence of caller allocation as a fourth routed [`Backing`](../prototypes/candle-metal-adapter/src/adapter.rs) class. It can expose one explicit construction helper that derives the exact empty shape and dtype from the loaded artifact, allocates the minimum Candle-managed sentinel on the already selected Metal device, constructs a real untracked Candle `Tensor` with logical count zero, and then passes that tensor through the existing `TilerPlan::preflight`, `apply_op1_no_bwd`, `bind_candle_storage`, `Backing::CallerInput`, route fingerprint, and lifetime paths. The helper refuses a nonempty declaration, wrong device, wrong dtype, allocation shorter than the sentinel requirement, or any storage/shape disagreement. It is never an implicit default for a missing ordinary input.

No new semantic enum is warranted: an empty F32 tensor and a nonempty F32 tensor are the same value kind, and the existing `Tensor` already carries their MECE metadata—device, dtype, shape, layout, storage, and autograd state. A `Tensor | DeclaredEmpty` sum at `TilerPlan::apply` would preserve correctness, but would duplicate a representational construction choice in the semantic input API and force every later call site to branch after the helper can instead return the ordinary carrier. A private construction-state enum becomes justified only if implementation discovers a second non-Tensor input representation that cannot first be converted into an honest Candle `Tensor`.

The exact accepted candidate is narrowly input-side. The sentinel allocation exists before the routing attempt as consumer-owned input representation, not as output, scratch, or internal program storage; its allocation cannot be hidden inside `plan_dispatch` or occur after a fallback choice has been spent. Its Metal allocation length is at least one byte, its `MetalStorage` logical count and tensor element count are zero, and every routed input binding's artifact-derived accessible extent remains exactly zero. The helper does not admit an empty output allocation, autograd, a mutable or aliased view, a foreign device, another dtype, or a kernel whose verified access window is nonzero.

The current empty-domain kernel evidence is compatible with this boundary: `a_zero_extent_reduction_commits_its_identity_without_a_loop_or_a_barrier` proves the structured kernel emits no reduction loop or barrier and stores the declared `+0.0` identity; program and artifact builders already require each stage access's evaluated accessible bytes to equal its verified byte window, which is zero for the empty input. The runtime's `RoutedBinding::accessible_bytes` retains that exact zero. A placeholder must never widen it merely because the physical allocation is nonzero.

The corrected ranking is: (1) explicit artifact-derived zero-tensor construction through Candle's public storage APIs, followed by the unchanged ordinary Tensor path—best supported coverage, exact semantics, negligible one-byte-per-device storage, and no parallel binding authority; (2) retain `ZeroExtentInterface` as the standing refusal—equally fail-closed and locally simpler, but unnecessarily excludes a semantic value the pinned consumer can represent; (3) add a public input enum or a fourth route-backing variant—correct if fully validated, but dominated by constructing the standard Tensor before routing; (4) bind a raw placeholder while no Tensor exists, widen the accessible range, use an unrelated anchor tensor, or silently synthesize an omitted nonempty input—rejected.

The strongest counterpoint to option 1 is that Candle documents `Tensor::from_storage` with a caller obligation to ensure shape/storage compatibility rather than checking it. That does not make the route unsound; it makes the Tiler helper the validation owner. The decision reverses if a source or execution probe shows that a zero-count `MetalStorage` over a rounded allocation cannot survive Candle's normal `Tensor`, layout, custom-op, or lifetime paths, or if constructing the sentinel requires bypassing Candle's allocator. Current source shows the opposite, but the landing must prove the full empty-domain route on hardware and retain the old `Tensor::from_vec` failure as the comparison that explains why the helper exists.

This is a consumer-only Rust and storage-policy change. The tensor shape, dtype, accessible range, artifact bytes, plan identity, payload, cache identity, and kernel are unchanged; the sentinel's allocation identity is invocation/device evidence only. No artifact, manifest, semantic, schedule, kernel, or cache domain steps.

## Accepted — 2026-08-12

Tom accepted the corrected option 1 in the live Codex review by replying `okay agreeed, next decision`. The adapter may explicitly construct the artifact's declared zero-element input as a genuine Candle `Tensor` over a minimum Candle-managed sentinel allocation, with logical element count zero and artifact-derived accessible extent zero, and then use the unchanged ordinary Tensor route. The construction is explicit and fallible; it is never a default for an omitted nonempty input.

No public semantic input enum or fourth routed backing class is accepted. Empty and nonempty inputs remain the same Tensor value kind, distinguished by their existing device, dtype, shape, layout, storage, and autograd data. A private construction-state enum may be introduced later only if implementation discovers another non-Tensor representation that cannot first become an honest Candle `Tensor`.

The accepted first pass is input-only and no-autograd. It does not admit empty outputs, raw placeholder binding without a Tensor, unrelated anchor tensors, widened accessible ranges, mutable or aliased views, other dtypes, foreign devices, or any kernel whose verified input window is nonzero. Hardware evidence must route both retained empty-domain members and perturb the count, shape, dtype, device, accessible extent, and lifetime independently before the ticket closes.

## Source re-audit at `b35bb34f` — 2026-08-13

Per-Fact verdicts at this exact base, before any edit. The 2026-08-05 trigger log is historical: status is now `in-progress` implementation of the 2026-08-12 acceptance, not a deferred ticket.

| Claim | Verdict | Evidence |
| --- | --- | --- |
| `empty-domain` artifact declares a `1 x 0` input | **Verified** | `prototypes/serial-sum-compile` `ROWS` is `1`; `REDUCTION_CLASSES` names `("empty-domain", 0)` |
| Empty reduction reads the input never and publishes the identity | **Verified** | `a_zero_extent_reduction_commits_its_identity_without_a_loop_or_a_barrier` stores `F32Bits(0.0_f32.to_bits())` with zero `SerialLoop`s and zero `Barrier`s. Shape in that test is `[2, 0]`, not the published `[1, 0]`; the property is the same |
| `route-a-zero-extent-program-through-candle-metal-storage` closed on `TensorRefusal::ZeroExtentInterface` | **Verified** | That ticket's Outcome; `bind_interface` still called `declared_extents_are_nonzero` on the input |
| Refusal is decided from declared extents before any Candle tensor | **Verified** | `TilerPlan::load` → `bind_interface`; proof's empty-domain arm ran at load |
| `Tensor::from_vec` fails at Candle 0.11.0 for that shape | **Verified** from source | `from_vec` → `storage_owned` → `new_buffer_with_data` with `size_of_val(&[])` = 0. Hardware re-check is owed |
| `prototypes/serial-sum-run` allocates `needed.max(1)` | **Verified** | `device.new_buffer(needed.max(1), …)` and `placed.needed.max(1)` |
| Candle adapter binds only caller storage or route-declared `Output` / `Internal` / shared | **Verified** | `Backing` is `CallerInput`, `Shared`, `Output`, `Internal` |
| `MetalDevice::new_buffer` passes a zero logical byte count through `buf_size`; `0_usize.next_power_of_two()` is one byte | **Verified** | candle-core 0.11.0 `new_buffer` then `buf_size` |
| `MetalStorage::new` records logical count and dtype separately | **Verified** | `MetalStorage::new(buffer, device, count, dtype)` |
| `Tensor::from_storage` and `From<(Storage, Shape)>` publicly construct a contiguous tensor | **Verified** | `Tensor::from_storage` is `pub`; `impl<S: Into<Shape>> From<(Storage, S)> for Tensor` |
| `MetalStorage` count 0 + `F32` + shape `[1, 0]` is compatible | **Verified** | both state zero logical elements |
| `a_zero_extent_reduction_commits_its_identity_without_a_loop_or_a_barrier` is the kernel evidence | **Verified** | `crates/tiler-ir/src/kernel/tests.rs` |
| Program/artifact builders require evaluated accessible bytes to equal the verified byte window | **Verified** | `AccessibleBytesDisagreement` when `computed != window.length` |
| `RoutedBinding::accessible_bytes` retains the artifact-evaluated extent | **Verified** | `place_bindings` writes `unsigned(binding.accessible_bytes(), …)` |
| Candle is pinned at 0.11.0 | **Verified** | `prototypes/candle-metal-adapter/Cargo.toml` |
| 2026-08-05 log still describes a deferred ticket | **Imprecise as current status** | Frontmatter is `in-progress`; Accepted 2026-08-12; 2026-08-12 trigger fired by source audit. The 2026-08-05 entry remains historically accurate |

## Trigger check log

- 2026-08-05 — **not fired.** Filed at `deferred` by `route-a-zero-extent-program-through-candle-metal-storage`'s typed-refusal close. Tom has not been asked and has not answered; Candle is pinned at 0.11.0, whose Metal allocator still returns `Metal error Failed to create metal resource: Buffer` for a `1 x 0` `f32` tensor, measured on macOS 27.0 (26A5388g) / Apple M4 Max. Recheck: `cargo run -p tiler-prototype-candle -- --artifact <base published by tiler-prototype-compile>` — the two `empty-domain` members print `REFUSED before any Candle storage is asked for` followed by the live allocator error while the trigger has not fired.
- 2026-08-12 — **fired by source audit.** The standard `Tensor::from_vec` upload still asks Metal for a literal zero-length buffer and fails, but the same pinned Candle release publicly exposes a rounded typed-buffer allocation, `MetalStorage::new` with an independent logical count, and `Tensor::from_storage`. Their exact composition represents the zero-element tensor without a Tiler-private storage or route-backing class. The implementation still owes a hardware route and subject perturbations before claiming delivery.
- 2026-08-13 — **implementation in progress at `b35bb34f`.** Status is no longer deferred. The 2026-08-05 `REFUSED before any Candle storage is asked for` close path is being replaced by explicit helper construction plus the ordinary Tensor route; `Tensor::from_vec` is retained as the comparison that explains why the helper exists.

## Implementation — 2026-08-13

Worker `worker-candle-zero-extent` on `tkt/decide-whether-the-candle-adapter-may-synthesize-zero-extent-storage`. Exact commit: `959f33376f23ea16a3c9b1158c9589d68dadb9f6` (parent implementation `607b46a641cdff561f192c47a869f947e26baa2f`). Merge and close left to the coordinator.

`TilerPlan::empty_input_tensor` constructs the artifact's declared zero-element input as a genuine Candle `Tensor` over `MetalDevice::new_buffer(0, F32)`, `MetalStorage::new(..., count: 0, F32)`, and `From<(Storage, Shape)>`. `TilerPlan::load` admits empty inputs and still refuses empty outputs with `ZeroExtentInterface`. The ordinary `preflight` / `apply_op1_no_bwd` / `bind_candle_storage` / `Backing::CallerInput` path is unchanged. `bound_accessible_extent` returns the artifact window and ignores allocation length. No public semantic input enum and no fourth `Backing` class.

### Hardware — Measurement, host-bound

Workload: `cargo run -p tiler-prototype-compile --offline -- --out /tmp/serial-sum.tiler` then `cargo run -p tiler-prototype-candle --offline -- --artifact /tmp/serial-sum.tiler`. Target: Metal Apple9 / `tiler.metal` / `metallib`. Toolchain: `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`). Source: `959f33376f23ea16a3c9b1158c9589d68dadb9f6`. Host: `ssh Host m3`, Apple M3 Pro, macOS 27.0 (26A5388g). Observations are host-bound Measurements. Rust/Xcode/SDK were not changed.

```text
Tensor::from_vec still cannot upload [1, 0]: Metal error Failed to create metal resource: Buffer — that is why the helper constructs through MetalDevice::new_buffer
empty input: logical elements 0, allocation 1 byte(s), artifact accessible extent 0 (bound 0)
empty-domain.selected: 1x0 ... 5 case(s) agreed
empty-domain.materialized: 1x0 ... 5 case(s) agreed
candle adapter proof: 6 of 6 published member(s) routed and agreed ... across 30 case(s)
```

A later helper construction on the same device reused a 4-byte pooled buffer; `bound_accessible_extent(0, 4) = 0`.

### Subject perturbations (quoted)

Gate-reachable, subject broken independently (not the assertion):

- count admits 1: `count 1 is not empty: ()`
- shape rank check widened: `left: "candle.preflight.extent: ... carries 0" right: "candle.preflight.rank: ... rank 1"`
- dtype admits f16: `f16 is not f32: ()`
- window admits 4: `4 bytes is a read window: ()`
- `bound_accessible_extent` returns allocation: `left: 1 right: 0`
- lifetime admits tracked: `tracked is refused: ()`

Hardware quotes on m3:

- count: `candle.preflight.empty-input.count: constructed MetalStorage count is 1 and must be 0`
- shape: `candle.preflight.extent: the artifact declares 1 along axis 0 and the tensor carries 2`
- dtype: `candle.preflight.dtype: this profile's artifacts declare f32 and the tensor carries f16`
- device: `candle.preflight.device: this adapter binds Metal storage and the tensor lives on Cpu`; foreign: `the tensor's storage belongs to DeviceId(3) and this adapter bound DeviceId(1)`
- accessible extent: `the artifact-derived accessible extent is 4 byte(s) and an empty input may bind only a zero window`; `bound_accessible_extent(0, 4) = 0`
- lifetime: `the tensor participates in tracked autograd and this fused forward op carries no backward formula`

### Unsupported cases (unchanged first pass)

Empty outputs; omitted nonempty inputs; raw placeholder binding without a Tensor; unrelated anchor tensors; widened accessible ranges; mutable or aliased views; dtypes other than F32; foreign devices; tracked autograd; kernels whose verified input window is nonzero. Candle 0.11.0 `to_dtype` on a zero-element Metal tensor panics (`attempt to divide by zero` in candle-metal-kernels); that path is not used.

### Guard

`tkt guard --base origin/main tkt/decide-whether-the-candle-adapter-may-synthesize-zero-extent-storage`: affected scopes match declared (`contracts/integrations`, `implementation/candle`, `project/tickets`). Verdict WARN for declared-area overlap with open siblings; no under-declaration.
