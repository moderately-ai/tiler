---
id: spike-hermetic-fptaylor-certificate-checking
title: Spike hermetic FPTaylor certificate checking
status: deferred
priority: p2
dependencies: [spike-sound-region-accuracy-analyzer-integration]
related: [research-region-accuracy-contracts-and-analyzable-error-budgets]
scopes: [research/numerics]
shared_scopes: []
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
