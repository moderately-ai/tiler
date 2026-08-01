---
id: adopt-candle-command-stream-once-a-terminal-check-is-reachable
title: Adopt Candle's own command stream once a terminal check is reachable
status: deferred
priority: p2
dependencies: [prototype-candle-metal-adapter]
related: []
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [candle, runtime, integration]
---
## User-visible outcome

A Tiler custom op overlaps with the Candle work around it instead of serializing against it: the adapter encodes into Candle's active command stream, does not create a private command buffer, does not commit, and does not wait — while still refusing to read device memory before terminal success is observed.

## Why this is deferred rather than done

`docs/integration/candle.md`'s command-stream section requires the first behaviour and its synchronous-readback paragraph permits the second only as an exception, "until the [verified gap](../docs/research/runtime/candle-metal-post-wait-error-checking.md) is fixed **or the adapter supplies an equivalent checked boundary**". `prototypes/candle-metal-adapter` supplies one, and pays for it with the overlap.

**Fact — Candle 0.11.0 performs no post-wait terminal check.** `Commands::ensure_completed` (`candle-metal-kernels-0.11.0/src/metal/commands.rs`) reads the command buffer's status *before* waiting: `NotEnqueued`/`Enqueued` commit then wait, `Committed`/`Scheduled` wait, and both arms return `Ok` without re-reading the status. Only a buffer already in `Error` before the wait is reported. A buffer that transitions to `Error` during the wait returns success.

**Fact — the check is unreachable from outside Candle at that revision.** `MetalDevice`'s `commands` field is `pub(crate)`; `Commands` publishes no accessor for its current or in-flight `CommandBuffer`; and `MetalDevice::wait_until_completed` returns `Result<()>` carrying no status. A consumer that encoded into Candle's stream would have no object to ask, so it could not supply the equivalent boundary *and* keep the overlap.

## Activation trigger

Any one of these makes this actionable, and each needs re-checking at the revision the workspace then resolves rather than assuming this one:

- Candle's `ensure_completed` (or its successor) re-reads the status after the wait and reports `Error`; or
- Candle exposes the in-flight `CommandBuffer`, or a wait that returns a terminal status, on a public API; or
- Tiler pins a fork carrying either change, under the repository's `git = … rev = …` rule.

## Closes when

- The adapter encodes into the guard `MetalDevice::command_encoder` returns and creates no private queue or command buffer.
- No path reads device memory, compares a value, or returns a tensor before terminal success is observed for the exact command buffer the dispatch was encoded into.
- The flush `plan_dispatch` performs against Candle's pending work is removed or re-justified: it exists only because the two streams were separate, and a single stream orders the input's producer against this dispatch without it.
- Overlap is measured rather than asserted — a workload with Candle work on both sides of the custom op, timed before and after, with the procedure and environment recorded.
