# Candle Metal post-wait error checking

**Status:** verified source defect; real-GPU fault injection not measured
**Ticket:** `verify-candle-metal-post-wait-error-checking`

## Result

At local Candle commit
`31f35b147389700ed2a178ee66a91c3cc25cc80d` (version 0.11.0),
`candle_metal_kernels::metal::Commands::ensure_completed` can return
`Ok(())` when a command buffer observed as `Committed`, `Scheduled`,
`NotEnqueued`, or `Enqueued` transitions to `Error` during
`wait_until_completed`.

This is verified by a structural audit of the exact source plus an executable
transition test, not by a real GPU fault. The audit extracts the concrete
function and confirms one pre-wait status read, two wait sites, no post-wait
status read, and a success return after the waits. The transition test executes
that control-flow shape with a command buffer whose status changes from
`Committed` or `Scheduled` to `Error` inside the wait. The inspected function
returns success after one pre-wait status observation. A corrected shape reads
status again and returns the stored error.

## Exact local source evidence

The inspected checkout was clean on branch `main` at the commit above.

### Kernel command layer

`candle-metal-kernels/src/metal/commands.rs:317-337` contains:

```text
match cb.status() {
    NotEnqueued | Enqueued => { commit; wait_until_completed; }
    Committed | Scheduled => { wait_until_completed; }
    Completed => {}
    Error => { return CommandBufferError(cb.error()); }
}
Ok(())
```

There is no `status()` or `error()` observation after either wait. The adjacent
callers are affected as follows:

- `Commands::flush_and_wait`, lines 235-264, calls `ensure_completed` for the
  last command buffer. Its later loop explicitly checks only earlier buffers,
  excluding that last buffer.
- `Commands::flush_and_wait_current`, lines 269-284, calls
  `ensure_completed` for the exact buffer containing the caller's work. It then
  removes only buffers whose status is `Completed`; an errored buffer remains
  retained, but the method still returns `Ok(())`.
- `CommandBuffer::wait_until_completed`, in
  `candle-metal-kernels/src/metal/command_buffer.rs:77-79`, wraps Objective-C
  `waitUntilCompleted` and returns `()`. Its `status()` and `error()` accessors
  are separate methods at lines 63-73.

The earlier-buffer loop in `flush_and_wait` does not close the last-buffer gap.
Queue FIFO establishes that earlier buffers are terminal when the last one is
terminal; it does not establish that the last one completed successfully.

### Candle core and synchronous readback

The error can cross into these local Candle core paths:

- `candle-core/src/metal_backend/device.rs:170-187` maps the kernel-layer
  results from both `wait_until_completed` and `flush_and_wait_current`, then
  returns success if those methods do.
- `candle-core/src/metal_backend/mod.rs:2004-2018` encodes a blit to a CPU-visible
  buffer, calls `flush_and_wait_current`, and immediately reads the buffer.
- `candle-core/src/quantized/metal.rs:40-55` and `:441-454` likewise wait and
  then read CPU-visible bytes for dequantization or `data()`.

These are concrete precedents for the kind of synchronous validation-record
readback Tiler needs. A missed final error could cause host code to interpret a
validation record whose producing command buffer failed.

## Apple contract

Apple documents `Completed` and `Error` as distinct terminal command-buffer
states. `Error` means the GPU stopped executing because of a runtime issue;
Apple directs callers to inspect the command buffer's `error` property for the
details. `waitUntilCompleted` blocks until GPU execution and completion
handlers finish, but does not return a success/error value. See Apple's
[`MTLCommandBuffer.status`](https://developer.apple.com/documentation/metal/mtlcommandbuffer/status),
[`MTLCommandBufferStatus.error`](https://developer.apple.com/documentation/metal/mtlcommandbufferstatus/error),
[`MTLCommandBuffer.error`](https://developer.apple.com/documentation/metal/mtlcommandbuffer/error),
and [command-buffer overview](https://developer.apple.com/documentation/metal/mtlcommandbuffer).

Therefore a pre-wait `Committed` or `Scheduled` observation cannot justify a
successful return. The final state is learned only after the wait.

## Required check

Every synchronous completion method used before CPU readback must establish a
successful terminal state after waiting:

```text
inspect initial status
  -> commit if NotEnqueued/Enqueued
  -> wait if not already terminal
inspect final status
  -> Completed: success
  -> Error: return CommandBufferError(error description)
  -> anything else: fail closed as an unexpected post-wait state
```

The post-wait check belongs in `ensure_completed` (or one shared terminal-state
helper called by it), so both `flush_and_wait` and `flush_and_wait_current`
inherit the guarantee. Checking only `error().is_some()` is weaker than checking
status: Apple defines status as the state authority, while error supplies detail
for the `Error` state. Error text should use Candle's existing
`CommandBufferError(String)` and its `"unknown error"` fallback when necessary.

For Tiler's synchronous validation readback, the adapter must not inspect the
shared validation buffer or make a semantic decision until the exact command
buffer containing the validator and any required synchronization/copy has:

1. been committed;
2. reached a terminal state;
3. been observed as `Completed`; and
4. produced no command-buffer error.

An `Error` is a post-`RoutingCommit` execution failure. It must propagate as an
error, never be treated as a validation miss or trigger fallback.

## Executed source test

[`check_candle_post_wait_source.py`](../../../spikes/runtime/check_candle_post_wait_source.py)
audits the concrete checkout rather than relying on a prose transcription:

```sh
python3 spikes/runtime/check_candle_post_wait_source.py \
  /Users/tsanterre/workspace/github.com/huggingface/candle/\
candle-metal-kernels/src/metal/commands.rs
```

The local result reports one status read, two waits, zero post-wait status
reads, one error read in the initial status match, and success after the waits.

[`candle_metal_post_wait.rs`](../../../spikes/runtime/candle_metal_post_wait.rs)
models the exact inspected branch structure and the required terminal check.
Nine tests cover both successful and erroneous transitions, the commit paths,
preexisting errors, error-detail preservation, and fail-closed handling of a
nonterminal status after wait.

Run it with:

```sh
rustc --edition 2021 --test \
  spikes/runtime/candle_metal_post_wait.rs \
  -o /tmp/tiler-candle-post-wait
/tmp/tiler-candle-post-wait
```

Observed result on the local host:

```text
9 passed; 0 failed
```

The decisive negative tests observe `Ok(())` while the simulated concrete
buffer is in `Error` after waiting. The corrected function observes the same
transition and returns the command-buffer error.

## Measurement boundary

No real Metal command-buffer fault was induced. The host has an Apple M4 Max
and Metal 4 support, but `xcrun metal -v` failed because the separately
downloadable Metal Toolchain is not installed. More importantly, no existing
checked-in Candle kernel was identified that deterministically and safely
causes a runtime command-buffer error rather than an encode-time Objective-C
validation exception, undefined memory behavior, a hang, or a device-wide
timeout. Manufacturing one of those failures solely to strengthen an already
decisive source test would be an unsafe experiment.

Accordingly:

- **Measured:** the structural audit confirms the concrete source shape, and
  the transition harness executes it to demonstrate the successful return
  after a wait-to-error transition; all nine transition tests pass.
- **Source-backed:** the exact affected Candle paths and Apple's terminal-state,
  wait, status, and error contracts.
- **Not measured:** a real Apple GPU producing `MTLCommandBufferStatusError`
  during this Candle call and the actual error payload on a supported OS/GPU.

A future upstream Candle regression test should use a test-only injectable
command-buffer trait or a deterministic Metal test fixture supplied by Apple;
it should not rely on GPU hangs, page faults, or out-of-bounds behavior.
