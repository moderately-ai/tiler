---
id: prototype-runtime-routing-commit
title: Implement one-way runtime routing commit
status: done
priority: p0
dependencies: [prototype-runtime-artifact-validation]
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, routing, correctness]
---
Implement a state boundary preserving fallback authority only before one-way commit and consuming it before allocation, encoding or submission. Demonstrate fallback is uncallable afterward; semantic invalidity and corrupt artifacts fail closed rather than becoming route misses.

## Outcome

The state boundary already existed as three types with an infallible `commit` consuming its input, landed under `prototype-runtime-artifact-validation`. That shape is preserved and was not rebuilt. What this ticket adds is the two things the boundary was missing: the demonstration it was never given, and a consumer that actually respects it.

### The commit is one-way, and the compiler now says so

`Preflight::commit` carries three examples that `cargo test --doc` compiles. Committing once compiles; committing twice does not (`compile_fail,E0382`); and duplicating a `Preflight` to keep a spare does not (`compile_fail,E0277`, because the type is deliberately not `Clone`). This is the ticket's "demonstrate fallback is uncallable afterward" and it is now gate-run evidence rather than a documented intention.

**Measurement.** The error-code pins are enforced, checked by deliberately mis-pinning one: replacing `E0382` with `E0499` fails the doctest with `Some expected error codes were not found: ["E0499"]` while reporting the real `error[E0382]: use of moved value: preflight ... value used here after move`. So a change that made either example compile, *or* that made it fail for an unrelated reason, fails the gate. The pin was restored.

No dependency was added to reach this. `trybuild` would have been the other route and was eliminated: it needs a dev-dependency edit to `scripts/check_workspace.py`'s `EXPECTED_DEPENDENCIES` (`implementation/workspace`) and a `Cargo.lock` change (`implementation/cargo-lock`), neither scope held here, in exchange for pinning full stderr text rather than an error code. The positive companion example rules out the incidental-failure hazard that is the usual reason to prefer `trybuild`.

### One defect fixed: the proof committed before it had finished judging

**Fact.** `prototypes/serial-sum-run` called `preflight.commit()` and only then, inside `dispatch_routed`, decided whether it binds storage for each routed ABI slot (`UnboundBinding`) and whether the launch covers any threads (`EmptyLaunch`). Both are pure functions of the `Preflight`: `Preflight::bindings()` and `Preflight::launch()` publish exactly them, and `load/route.rs` documents that they are published pre-commit *because* "those decide whether to commit at all".

**Why it is a defect and not a style point.** ADR 0051 permits a fallback only before the commit. A host that commits and then discovers it cannot bind a slot has destroyed its own fallback authority for a reason that was decidable while it still held it. The runner takes no fallback, so nothing was silently wrong — but it is the reference consumer of this boundary, and it was demonstrating the opposite of the boundary.

**Fix.** `plan_route(&Preflight) -> Result<Vec<PlacedSlot>, ProofError>` resolves every routed slot to an owned placement decision and refuses a zero-thread launch, and is the last call before `commit`. `dispatch_routed` now consumes that plan and has no unbound case left to discover.

What deliberately stays after the commit is what needs a device: the pipeline's `max_total_threads_per_threadgroup` and the length of an allocation actually made. Neither can move earlier — `RoutedDispatch` is the only publisher of the object bytes and entry symbol, by design, so building a pipeline *is* program work — and a refusal there is a failure reported, never a fallback taken.

### Fail-closed, measured on the real envelope

`probe_fail_closed` runs against the exact bytes the producer wrote, before the positive route is claimed, and asserts the *class* of each refusal. The class is the whole obligation: a damaged file reported as `NoApplicableVariant` reads as "this artifact does not apply to your host" and sends a reader to rebuild a plan when the repair is to re-fetch the bytes.

**Measurement**, Apple M4 Max, this branch, artifact from `tiler-prototype-compile` (32,449 bytes, identity 12,711 bytes):

```text
a flipped byte at offset 16224: runtime.artifact: artifact.integrity: ManifestDigestMismatch
truncated to 16224 byte(s): runtime.artifact: artifact.malformed: TotalLengthMismatch { declared: 32449, actual: 16224 }
an expected identity that is not this artifact's: runtime.program-mismatch: ...
a host offering another profile descriptor: runtime.incompatible-target: the selected variant ... DescriptorMismatch
a host stating another backend family: runtime.unexecutable-payload: ... tiler.metal/metallib payload ... states tiler.some-other-backend/metallib
```

Five perturbations, five distinct classes, and none of them a route miss. The run then completes: `bit-for-bit agreement: direct on 4 element(s), envelope on 4 element(s)`, direct at `4x3` and envelope at `4x1` — the `MAX_OPAQUE_IDENTITY_BYTES` limit owned by `bound-the-backend-entry-key-by-the-identity-it-carries`, not worked around here.

### Two doc claims corrected, both found by reading the code beside them

- `preflight`'s stated refusal order omitted `accept_entry`'s two refusals entirely and merged the variant and payload profile checks into one step. It now states the actual nine-step order.
- `select_variant` documented that the walk stops at the first guard evaluating true, and said nothing about a guard that cannot be *evaluated*. That case aborts rather than skipping, and the distinction is load-bearing: skipping would silently route to a variant the producer ranked lower because the caller under-bound a fact, and report it as a successful route. Now documented.

### The two defects the dispatch asked about are both fully fixed at the base

Verified by reading, not by grep. Deferred feasibility predicates: `accept_entry` (`load.rs`) refuses on `variant.deferred_predicates().len() > 0` as `UnansweredDeferredPredicates`, and it sits *outside* the selection loop so a deferring variant is refused rather than falling through to a lower-ranked one. The variant's own declared target profile: `preflight` classifies `variant.target_profile()` and reports `TargetDeclaration::Variant`, separately from the payload's `compatibility` reported as `TargetDeclaration::Payload`. Both are now also covered by a probe or by the run above.

### One claim checked and *not* retracted

`preflight`'s three-way `let-else` treats "no carried object", "no entry symbol" and "no transport mapping" as one condition. That looked like three conditions collapsed into one classification, which would have reported a carried object as `ObjectNotCarried`. Read in full and it holds: `codec/view.rs:148-160` derives `payload_metadata` from `payload_content`, so the two are `Some` together, and `codec/validate.rs:check_entry_mappings` refuses a carried payload whose metadata lacks a mapping for a dispatched entry key (`UnmappedBackendEntry`). The comment is accurate.

### What is implemented and what is tested — three different claims

- **Implemented and tested by the compiler, in the gate:** the one-way commit, no second commit, no duplicated `Preflight`.
- **Implemented and measured, but only under the proof invocation:** every fail-closed class above, and the pre-commit ordering. These need a valid artifact and a device; the gate has neither, and a checked-in fixture would go stale against the encoder it exists to exercise. They are a measurement on a named host, not a gate-enforced property. Split into [`gate-the-runtime-fail-closed-probes`](gate-the-runtime-fail-closed-probes.md), which carries the two candidate closures and why choosing between them is a boundary question rather than test plumbing.
- **Implemented and not tested, asserted as nothing more:** retention of asynchronous resources through their final device use. `placements` outlives the `submit` call that waits for the command buffer's terminal state, so every buffer is alive through its last device use by construction — but no test provokes a premature release, and none is claimed.
- **Not claimed at all:** no crash, race, or submission-failure property is tested. Nothing here forces a command buffer into `Error`, so "exact terminal success before host readback" is implemented (`status()` compared against exactly `Completed` before any readback) and exercised only on the success path.

### A limitation recorded rather than worked around

A failed dispatch is reported as its terminal `MTLCommandBufferStatus` and not as Metal's own error, because `metal` 0.33.0's `CommandBufferRef` exposes no accessor for the buffer's `NSError` — checked by reading `commandbuffer.rs` in full; the `MTLCommandBufferError` enum it declares is returned by nothing. Reaching it means an `unsafe` `msg_send!`, which is a new admitted site under ADR 0079 and not this ticket's to take.
