---
id: prototype-metal-runtime-proof
title: Execute and validate the serial Sum Metal proof end to end
status: done
priority: p0
dependencies: [prototype-metal-runtime-execution, carry-the-stage-execution-order-in-the-envelope, preflight-every-entry-of-a-multi-stage-route, make-runtime-routing-commit-authority-one-shot]
related: []
scopes: [implementation/runtime, research/runtime, implementation/metal-aot]
shared_scopes: [project/tickets, contracts/integrations, contracts/navigation, contracts/artifacts, contracts/numerics, implementation/cargo-lock]
paths: []
tags: [implementation, prototype, metal, runtime, vertical-slice]
---
Integration gate only: execute the produced bundle through the non-published
`serial-sum-run` consumer without importing `tiler-ir`, the compiler, or
backend internals. This ticket builds no runtime capability itself —
device-free artifact validation is owned by
`prototype-runtime-artifact-validation`, live device/library/function/pipeline/
resource/launch preflight by `prototype-metal-runtime-preflight`, the one-way
routing/fallback commit by `prototype-runtime-routing-commit`, and
allocation/dispatch/terminal-status/resource-retention by
`prototype-metal-runtime-execution`. If integration exposes a gap in a
component, reopen or follow up that ticket rather than implementing the
capability here. The compile producer supplies a separate bounded proof-case
sidecar containing input and normative expected bytes; the runner treats those
bytes as test data, not as artifact semantics or an independent reference
implementation.

The integration must:

- validate the sidecar schema, section digests, unique case keys, and exact
  association with the selected envelope before using any case;
- compose the component capabilities in contract order for each independent
  proof execution: device-free validation, live preflight, one-way routing
  commit consumed before any allocation or encoding, then execution through
  terminal command status with resource lifetimes retained through final
  device use;
- execute the retained materialized program in one explicit proof run, then
  execute the normally selected fused program in a separate proof run and
  compare both readbacks with the producer's normative expected bytes for
  canonical NaN, infinity, signed-zero, subnormal, contraction-sensitive,
  empty-domain, singleton, and nontrivial reduction cases; and
- record the observed dispatch count, eliminated intermediate, pre-commit
  routing boundary, terminal status, and post-commit failure behavior.

The proof succeeds only when both device programs agree bitwise with the
normative reference, normal routing selects the fused program, its observed
execution uses one dispatch and no intermediate instead of two dispatches and
one intermediate, and every admitted precommit refusal exits before output or
scratch allocation, command encoding, or submission. Library, symbol, and
pipeline preparation may interact with the device during preflight. Admitted
applicability/capability misses preserve precommit fallback authority; corrupt
artifacts, inconsistent proof data, and systemic preparation failures fail
closed rather than masquerading as route misses.
The prototype need not implement a semantic fallback evaluator, but it must
demonstrate that all routes are prepared before one route-level fallback
authority is consumed and that authority is unrecoverable afterward. No Candle
integration, fallback after
partial submission, reusable Metal runtime crate, or production runtime API
belongs in this ticket.

Use an injectable prototype runtime adapter to exercise deterministic negative
library, function, pipeline, guard, and routing-preflight cases, alongside at
least one successful execution on a compatible live Metal device. Simulated
failures do not satisfy the live success gate, and absence of a compatible
device is an unmet evidence condition rather than success.

## Two dependencies were missing — added 2026-07-27

This ticket requires executing the retained **materialized** program as well as the fused one and comparing both readbacks. The materialized plan is multi-stage — `materialized.kernels().len() > 1`, pinned by a passing test in `prototypes/serial-sum-compile/src/bundle.rs` — and until `carry-the-stage-execution-order-in-the-envelope` a multi-stage envelope was refused outright, so the program could not travel in a bundle this reader accepts.

Three of the twelve requirements were therefore unreachable, and they are the three the success clause leads with: the materialized proof run, the comparison of both readbacks, and the one-dispatch-versus-two-dispatches observation. The graph did not show it, so the ticket kept appearing ready.

`carry-the-stage-execution-order-in-the-envelope` supplies the envelope half and has landed. `preflight-every-entry-of-a-multi-stage-route` supplies the runtime half — a loader still dispatches exactly one entry — and has not.

## Outcome

Done, with one class of the matrix explicitly split out. The multi-stage path executed on device for the first time.

**The proof matrix.** The producer now publishes four members — two reduction classes (singleton, nontrivial) times two plan roles — as `<base>.<class>.<role>` with a `.proof` sidecar each. Each sidecar carries five operand cases named for the numerical class they exist to exercise: ordinary, signed-zero-and-subnormal, non-canonical-NaN, infinity, and contraction-sensitive. The runner proves every member against every case: **20 cases across 4 members, all agreeing bit for bit with the published reference.**

**Measurement — Apple M4 Max, this host.** `nontrivial.selected` routes 1 dispatch over 0 shared allocations; `nontrivial.materialized` routes **2 dispatches over 1 shared allocation** and returns the same bits. Same for the singleton pair. That contrast is the deliverable: the fused plan the optimizer selects and the materialized plan that computes the same function through an intermediate are genuinely different programs on the device, and they agree.

**Why the shapes are asserted.** If both roles routed the same way, their agreement would be one program agreeing with itself — true and worthless. `UnexpectedRouteShape` refuses that per case, not once per member, because the shape is derived from the artifact on every route. Confirmed able to fail: swapping the expected shapes fails the run at the first member with `singleton.selected routed 1 dispatch(es) over 0 shared allocation(s), and its role means 2 over 1`.

**One routing authority per case.** `DecodedProgram` is not `Clone` and `preflight` takes `&mut self`, so each case decodes the envelope afresh. Reusing one decode across cases does not compile, which is ADR 0051 holding structurally rather than by recollection.

**A check that had to be corrected.** The runner compared the packaged kernel-program identity against `compilation.selected()`. That was only ever right because the producer packaged the selected plan; it would refuse the materialized member for being exactly what it is meant to be. It now matches against *any* alternative this build derives for the shape the artifact declares — still this build's own governed compilation, so a narrower claim than "some program" by a wide margin, and honest about the producer legitimately packaging a plan the portfolio did not rank first.

## The empty domain is absent, and that is a measured gap

**Superseded 2026-07-28 — the gap is closed and the qualification below no longer applies.** `emit-an-empty-domain-reduction-to-metal` (`done`, commit `97ab545`) removed the `unreferenced-buffer-parameter` rule rather than weakening it — what it guarded against became unrepresentable once the argument table was derived from `VerifiedKernel::declared_buffers` in declaration order — so the matrix is now six members and **30 cases, all bit-identical to the published reference on Apple M4 Max**, and `REDUCTION_CLASSES` carries `("empty-domain", 0)` in both prototypes rather than a reserved comment. The section below is retained as the measurement that identified the gap; its `emit.rs:410` citation refers to the pre-`declared_buffers` emitter.

**Fact.** A reduction over extent 0 compiles and retains both alternatives, but `emit_translation_unit` refuses **both** with `MalformedKernel { rule: "unreferenced-buffer-parameter" }`. Extents 1, 2, and 3 emit and pass `require_declared_realization`.

**Fact — the cause, read at `crates/tiler-metal/src/emit.rs:410`.** The emitter derives its binding table from what the kernel body reads and refuses when that count differs from the declared parameters, because emitting a signature that silently dropped one would change the ABI. An empty reduction reads its input never.

**Inference.** The refusal is correct at that boundary; the gap is that the emitter cannot express an empty-domain reduction at all. This ticket is an integration gate and its own text says a gap exposed in a component is followed up rather than implemented here, so it is: `emit-an-empty-domain-reduction-to-metal` owns it, and its closing condition includes restoring the third entry to the matrix. Both `REDUCTION_CLASSES` arrays reserve the slot in a comment naming that ticket.

The proof's stated success condition names an empty-domain case, so this is a qualification of the result rather than a detail: **the proof holds for the singleton and nontrivial classes and is unproven for the empty domain.**

Gate: `make full` green (965 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck). Hardware run green on Apple M4 Max.
