---
id: validate-bf16-at-the-runtime-routing-boundary
title: Validate BF16 at the runtime routing boundary before the one-way commit
status: todo
priority: p1
dependencies: [carry-bf16-through-the-artifact-encoding-and-identity, lower-bf16-to-metal]
related: [spike-bf16-through-the-second-dtype-seams, decide-per-dtype-dispatchability-as-a-target-capability]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, runtime, routing, fail-closed]
---
## User-visible outcome

The runtime refuses a BF16 program whose bound values are not BF16, and refuses a BF16 program on a host whose target family cannot dispatch the dtype — both **before** the one-way routing commit. A caller binding an `f32` buffer to a BF16 input gets a typed refusal instead of a plausible tensor computed from misread bytes.

## Why the phase ordering is the whole ticket

**Measurement.** Finding 26 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) records that the iOS Simulator compiles and links every `bfloat` module and *then* fails pipeline creation. That failure occurs at `PreparedKernelPreflight` — one phase **after** the one-way routing commit of ADR 0051. `decide-per-dtype-dispatchability-as-a-target-capability` eliminated device preflight for exactly this reason and made dispatchability a profile-owned, family-keyed fact resolvable at the compile profile.

**Inference.** So the refusal has to come from the profile, before routing commits. [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) showed the resolution mechanism produces `Dispatchable`, `Unsupported`, and `Unknown` at `AvailabilityPhase::CompileProfile`; this ticket is the runtime consuming it.

**Fact.** The bytes-per-element difference makes the binding check load-bearing rather than pedantic: BF16 is two bytes and `f32` is four, so a wrongly typed binding of the same buffer misreads element count and every value, and produces a full-looking result.

## Implementation keys

- Value binding validates the bound value's storage scalar against the program's declared one, refusing a mismatch by name. The existing `StorageScalarMismatch` refusal is the shape; it must reach BF16.
- Element count derives from the declared element width, so a two-byte element is not counted as four.
- A BF16 program offered a host whose profile resolves `Unsupported` or `Unknown` for `tiler::bf16@1` is refused at preflight, before the commit. `Unknown` refuses exactly as `Unsupported` does — the caller learns which, but both fail closed.
- No fallback after the commit. If a BF16 refusal is ever reachable after allocation, partial encoding, or submission, that is a defect in this ticket and not a case to handle.
- The refusal names the dtype and the family, so an explain record can say why.

## Required evidence

- An `f32` buffer bound to a BF16 input is refused, with the element-count consequence shown rather than asserted — a test that demonstrates the misread would have occurred.
- A BF16 program on a profile resolving `Unsupported` is refused at preflight; the refusal is shown to occur before the routing commit, by a check on the phase and not only on the message.
- The same on a profile resolving `Unknown`, and the two refusals are distinguishable.
- A BF16 program on the macOS profile routes and executes, so the refusals are not a blanket refusal.
- An `f32` program is unaffected on all three profiles.

## Closes when

Both refusals happen before the routing commit and are observed failing, the `Unknown` and `Unsupported` cases are distinguishable, a correctly bound BF16 program still routes on the dispatchable profile, and the `Runtime semantic validation` cell for BF16 moves.

## Graph maintenance

- Depends on the artifact carrying the dtype (there is nothing to validate against otherwise) and on lowering (there is nothing to route otherwise).
- `docs/dtype-support.md` records `Runtime semantic validation` as `absent/unsupported` for `f32` as well. This ticket does **not** discharge it for `f32`; if the mechanism it adds is dtype-neutral, say so and file the `f32` row separately rather than claiming a cell this ticket did not test.
- The pre-commit ordering is the property most likely to regress silently, because a refusal that moves one phase later still refuses and still looks green. Assert the phase, not just the outcome.
