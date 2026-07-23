---
id: prototype-metal-runtime-proof
title: Execute and validate the serial Sum Metal proof end to end
status: todo
priority: p0
dependencies: [prototype-metal-runtime-execution]
related: []
scopes: [implementation/runtime, research/runtime]
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
one intermediate, and every failed preflight exits before device work. Admitted
applicability/capability misses preserve precommit fallback authority; corrupt
artifacts, inconsistent proof data, and systemic preparation failures fail
closed rather than masquerading as route misses.
The prototype need not implement a semantic fallback evaluator, but it must
demonstrate that fallback authority still exists before commit and is
unrecoverable afterward. No Candle integration, fallback after
partial submission, reusable Metal runtime crate, or production runtime API
belongs in this ticket.

Use an injectable prototype runtime adapter to exercise deterministic negative
library, function, pipeline, guard, and routing-preflight cases, alongside at
least one successful execution on a compatible live Metal device. Simulated
failures do not satisfy the live success gate, and absence of a compatible
device is an unmet evidence condition rather than success.
