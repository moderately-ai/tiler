---
id: decide-whether-the-candle-adapter-may-synthesize-zero-extent-storage
title: Decide whether the Candle adapter may synthesize storage for a zero-extent tensor
status: awaiting-decision
priority: p3
dependencies: []
related: [route-a-zero-extent-program-through-candle-metal-storage, prototype-candle-metal-adapter]
scopes: [implementation/candle]
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

## Trigger check log

- 2026-08-05 — **not fired.** Filed at `deferred` by `route-a-zero-extent-program-through-candle-metal-storage`'s typed-refusal close. Tom has not been asked and has not answered; Candle is pinned at 0.11.0, whose Metal allocator still returns `Metal error Failed to create metal resource: Buffer` for a `1 x 0` `f32` tensor, measured on macOS 27.0 (26A5388g) / Apple M4 Max. Recheck: `cargo run -p tiler-prototype-candle -- --artifact <base published by tiler-prototype-compile>` — the two `empty-domain` members print `REFUSED before any Candle storage is asked for` followed by the live allocator error while the trigger has not fired.
