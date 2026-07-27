---
id: prototype-metal-runtime-execution
title: Implement Metal runtime execution mechanics
status: in-progress
priority: p0
dependencies: [prototype-metal-runtime-preflight, prototype-runtime-routing-commit]
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, metal, execution]
claimed_from: todo
assignee: integrator
lease_expires_at: 1785160795
---
Implement bounded allocation, ABI binding, checked dispatch, asynchronous resource retention through final device use, submission, exact terminal-status validation, and readback. Inject post-commit failures and prove no fallback occurs after commit.

## The 2026-07-25 blocker is resolved — superseded 2026-07-27

The section this replaces said the workspace forbade `unsafe`, that Tiler had no Metal binding and no path to one, and that first dispatch needed a decision from Tom. All three were true when written and none is true now, and a live p0 whose first section says it is blocked stops the next reader.

**Fact — the decision was made.** [ADR 0079](../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md) admits `unsafe` case by case at named sites, in the narrow form that section proposed: the workspace keeps `unsafe_code = "forbid"`, and `prototypes/serial-sum-run` is the one member permitted to diverge, declaring `unsafe_code = "deny"` so a block no attribute admits does not compile.

**Fact — the binding is chosen and the sites are two.** `metal` 0.33.0. The complete extent of `unsafe` in the workspace is `prototypes/serial-sum-run/src/buffer.rs`'s `write_f32` and `read_f32`, each carrying a per-function `#[allow(unsafe_code, reason = …)]`, an assertion against `buffer.length()` before the block, and a `SAFETY` comment. Reproduce with `grep -rn "unsafe_code|unsafe {" --include="*.rs" crates prototypes`.

**Fact — dispatch works.** The proof runs a real `metallib` on an Apple M4 Max through the artifact envelope and agrees bit for bit with the governed reference on both paths.

## Outcome

The ticket's clause list is one property with eight places it could be violated, not eight mechanisms. `docs/research/runtime/runtime-execution-contract.md`'s transition table says **never** for every post-commit transition — routing-committed→resources, resources→in-flight "including after zero or partial stages encoded", and in-flight→validation-observed. Everything enumerated exists to serve that.

Most of it was discharged by `prototype-metal-runtime-preflight`, and that is the right outcome rather than a shortfall: **the strongest guarantee that nothing falls back after the commit is to leave nothing after the commit that could need to.**

| Clause | Where it is discharged |
| --- | --- |
| Bounded allocation | `device_preflight`, before the commit: each binding against `max_buffer_length`, each allocation against the length it was requested at |
| ABI binding | `plan_route` resolves every slot to an owned `PlacedSlot`; `dispatch_prepared` encodes from it |
| Checked dispatch | The declared launch is compared against the pipeline's `max_total_threads_per_threadgroup` before the commit |
| Asynchronous resource retention through final device use | `PreparedRoute::placements` outlives `submit`, which waits for the terminal state |
| Submission | `submit` |
| Exact terminal-status validation | **This ticket** — `submission_outcome` |
| Readback | `crate::buffer::read_f32`, reachable only through the accepted arm |
| Inject post-commit failures, prove no fallback | **This ticket** |

### The one post-commit decision, made total

`submit` checked `status != Completed`. Correct, and it collapsed a distinction the contract keeps: Apple defines `Completed` and `Error` as the two *terminal* states, and the contract records that `waitUntilCompleted` returns no success value, so "a pre-wait non-error status is not evidence of successful completion". A buffer that never left the queue and one the GPU rejected are different things for a caller to do next.

`submission_outcome` is now an exhaustive, wildcard-free match over all six statuses returning `SubmissionOutcome::{Completed, ExecutionError, NotTerminal}`. A status added to the binding is a build error rather than falling into whichever arm a catch-all named — the posture every other vocabulary match in this workspace takes, and this is the one place a wrong answer reads as arithmetic: a readback from a failed dispatch returns whatever the output held before, which compares against the reference as a numerical disagreement.

**`SubmissionOutcome` has no retry or fallback variant, and that absence is the deliverable.** The contract's "never" is stated in the type rather than in a comment, so no status can map to one because there is nothing to map to.

### Proving no fallback follows the commit

- **Compile time.** `Preflight::commit` consumes `self`, and `crates/tiler-runtime/src/load/route.rs` carries doc-tests pinning that a second commit does not compile (`E0382`). Nothing after the commit can re-select a route because the value that would select one is gone.
- **Device-free, over the complete population.** `one_status_permits_a_readback_and_none_permits_a_retry` runs all six statuses through the classifier and asserts exactly one permits a readback. Six is every input that exists, so this is exhaustive rather than sampled. Confirmed able to fail before being trusted: `Scheduled` was pointed at the accepted arm and the case was observed failing, then reverted.
- **On hardware.** `probe_submission_status` observes the check refusing a real, live command buffer that was never committed — the contract's own warning case. Measured on an Apple M4 Max: `a live command buffer that was never committed: NotEnqueued, no readback taken`.

**Measurement boundary, stated rather than papered over.** The terminal `Error` state is *not* injected on hardware. Forcing a command buffer to fail means provoking a GPU fault, which risks a device reset and would not reproduce. That arm is covered device-free over the full vocabulary; no claim is made that a real device error was observed.

### No unsafe site was added, and the derivation is the reason

The contract notes that execution error *detail* comes from `MTLCommandBuffer.error`. `metal` 0.33.0's `CommandBufferRef` exposes `status`, `commit`, `wait_until_completed`, the handler registrations, and the encoder constructors — and no `error` accessor; `MTLCommandBufferError` is declared and returned by nothing. Reading it needs an `unsafe` `msg_send!`.

**ADR 0079's first condition does not admit that site.** It requires the foreign API to leave no safe route *to the same result*, and states that convenience is not a qualifying reason. The success path is already exact without the error object: the only thing it buys is a better failure message. So `ProofError::Dispatch` reports the exact status it stopped in and what that status means, and makes no claim about why the device rejected the work. Admitting a third site for a diagnostic would be a decision to take explicitly, not one to reach by implication.

### Not done here

Multi-entry routes — `preflight-every-entry-of-a-multi-stage-route` owns the runtime half. Per-entry submission and ordering between stages arrive with it.
