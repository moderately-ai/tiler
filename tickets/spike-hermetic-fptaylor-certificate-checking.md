---
id: spike-hermetic-fptaylor-certificate-checking
title: Spike hermetic FPTaylor certificate checking
status: deferred
priority: p2
dependencies: [spike-sound-region-accuracy-analyzer-integration]
related: [research-region-accuracy-contracts-and-analyzable-error-budgets]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy, proof, spike]
---
Build a hermetic FPTaylor plus HOL Light certificate experiment for the same
bounded corpus used by `spike-sound-region-accuracy-analyzer-integration`.
Do not install OCaml/opam or other global toolchains without explicit approval;
prefer a pinned, reproducible environment whose complete identity is recorded.

Measure certificate generation, certificate size, independent checker startup
and checking latency, total trusted computing base, and unsupported cases.
Verify that the selected formal path actually covers explicit f16/f32 casts,
required FMA, round-to-nearest-ties-to-even, gradual subnormals, exact reduction
topology, and the admitted assumption language. FPTaylor's deprecated `fma`
spelling and advanced power-of-two rounding exclusions must not be papered over.

The gate is whether independently checked evidence materially reduces the
trusted base at acceptable compile-time cost. Missing formal coverage returns
`Unknown`; empirical agreement with Daisy or high-precision samples is not a
substitute for certificate validation.

## Activation gate

Keep this ticket deferred until the bounded analyzer integration is complete
and a milestone requires independently checkable accuracy evidence rather than
the accepted trusted-analyzer result. Activate it only with a pinned hermetic
toolchain plan and explicit approval for any host-level installation.

## Exit criteria

Produce the pinned environment, exact corpus, generated certificates,
independent-check commands, unsupported-case inventory, trusted-base analysis,
and timing/size measurements. Mark the experiment done with a positive adoption
recommendation only if the independent checker covers every admitted numerical
construct; otherwise mark it done with a documented `Unknown`/negative result
and leave empirical agreement explicitly non-authoritative.

## Trigger check log

- 2026-08-04 — **not fired**, and the gate is a conjunction whose halves now disagree. [`spike-sound-region-accuracy-analyzer-integration`](spike-sound-region-accuracy-analyzer-integration.md) is `done`, so the first half — the bounded analyzer integration being complete — **is** met. The second half is not: no milestone requires independently checkable accuracy evidence rather than the accepted trusted-analyzer result, and no pinned hermetic toolchain plan or host-installation approval exists. A future sweep evaluates the second half only. Reproduce with `grep -m1 '^status:' tickets/spike-sound-region-accuracy-analyzer-integration.md`.
- 2026-08-05 — **not fired, and the second clause moved further away rather than closer.** [`connect-certified-rounding-error-bounds-to-rewrite-permissions`](connect-certified-rounding-error-bounds-to-rewrite-permissions.md) was the sweep that could have fired this, being the ticket that asks what admits a rewrite on a certificate. Its outcome eliminates the analyzer from the online path: [the record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) derives a **per-rule parametric bound** whose trusted base is a reviewed in-tree derivation plus an exact-rational instantiation check, routing through no analyzer at all — so it needs neither a trusted-analyzer result nor an independent certificate to replace one. The elimination rests on measured cost (the sibling spike's 1.0–1.5 s per invocation, against a stage-3 search that retains every alternative and may prune none on an estimate) and on coverage (`exp` is outside the adapter profile that spike admitted). It narrows what would fire this deferral to the **offline cross-check** role — certifying a rule's derivation once, on the bench, rather than certifying a candidate inside the compiler — which is a smaller and more plausible milestone than the one this ticket was written against, and which has not arrived. The third clause is unmet regardless: no pinned hermetic toolchain plan exists and no host installation has been authorized. **Two facts for the exit criteria were upgraded from the tool's README to its TOPLAS paper in the same pass**, now preserved as `fptaylor-toplas-2018` in [the source manifest](../docs/research/numerics/sources/README.md): "The general improved rounding model is not formally verified yet" confirms this ticket's excluded-rounding-model note at the primary source, and "The formalization of FPTaylor helped us to find a critical bug in our implementation" is evidence *for* the certificate route that this ticket's framing did not carry and that a future adoption argument should. Reproduce with `grep -c 'routes through no analyzer' docs/research/numerics/certified-bounds-as-rewrite-permissions.md`.
- 2026-08-09 — **not fired.** The bounded analyzer dependency remains `done`, but no milestone now requires an independently checked offline certificate, no pinned hermetic FPTaylor/HOL Light environment has been proposed, and no host-level installation has been authorized. The in-tree exact-rational rewrite-price derivation remains the active path, so the second and third activation clauses are still absent.
