---
id: restore-the-spikes-against-the-composed-numerical-contract
title: Restore the contract-naming spikes against the composed numerical contract
status: todo
priority: p3
dependencies: [compose-the-numerical-contract-from-its-decided-dimensions]
related: [restore-the-scalar-cpu-vertical-spike-against-the-current-crates]
scopes: [research/cache, research/extensions, research/target-profiles, research/scheduling, research/numerics, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [spikes, maintenance, numerics]
---
## User-visible outcome

Every retained spike that names a numerical contract builds against the composed `NumericalContract`, so re-running one still produces the evidence it claims to.

## Why this exists

**Fact.** `compose-the-numerical-contract-from-its-decided-dimensions` replaced the public `NumericalContract` enum with a composed type whose named points are associated constants, and it deliberately did not edit `spikes/`. No `make` target reaches a spike, so the workspace gate stayed green while three spikes were left naming variants that no longer exist. Each is a one-token change per site.

**Fact — the exact sites, from `grep -rn --include="*.rs" 'NumericalContract::' spikes` on the landing commit.**

- `spikes/cache/build-tool-exercise/envelope/src/lib.rs:59` — `NumericalContract::FlushSubnormalsToZeroF32`.
- `spikes/extensions/forkless-physical-provider/probe/tests/composition.rs:54` and `:89` — the same variant. Note this spike also carries `trybuild` UI fixtures naming it: `tests/ui/pass/lowering_installation_seam_exists.rs:27` and `tests/ui/fail/no_physical_provider_installation_seam.rs:35`, and the `fail` case's `.stderr` golden may move with the type.
- `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs:759` — `NumericalContract::StrictF32`.

The replacement in every case is the corresponding associated constant: `FLUSH_SUBNORMALS_TO_ZERO_F32`, `STRICT_F32`.

**Fact — one of the three has a live owner.** `restore-the-scalar-cpu-vertical-spike-against-the-current-crates` was in progress against `spikes/target-profiles/scalar-cpu-vertical` when this drift was introduced, which is why that spike was left alone rather than edited underneath it. Whoever claims this ticket checks that ticket's state first and either coordinates or narrows this one to the other two scopes.

## The research records naming the same removed spellings

**Fact — four retained research documents name a Rust identifier that no longer exists**, found by `grep -rn 'StrictF32\b\|FlushSubnormalsToZeroF32\|RelaxedF32\|ReassociateF32' docs/research` on the landing commit: `docs/research/scheduling/first-metal-contraction-realizations.md:85`, `docs/research/numerics/first-quantized-lm-profile.md:134` and `:211`, `docs/research/program-planning/first-metal-lm-workload.md:221`, and `docs/research/extensions/backend-provider-composition.md:201` and `:292` — the last two being compilable example code.

They were left alone deliberately rather than swept. Each sits inside a recorded measurement or derivation, so the edit is a *spelling* correction inside a retained record and has to preserve what the record claims; `docs/research/program-planning/**` was additionally held by a live ticket at the time. The two `backend-provider-composition.md` sites are the sharper ones: they are example code a reader is meant to be able to run.

## Boundaries

- A spike runs by hand from its own directory using the invocation its README records; a build that is only *checked* is not the evidence. Re-run each spike and confirm the fixture or golden it retains still matches, rather than only making it compile.
- Do not repoint a `trybuild` golden by copying the new output without reading it: the fail case's message is the claim, and a diagnostic that moved for a different reason would be laundered by a blind rebaseline.

## Closes when

All three spikes build and run from their own directories, every retained fixture or golden they cite still matches or has been rebaselined with the reason recorded, the six research-record sites name the current spelling without changing what each record claims, and `grep -rn 'NumericalContract::[A-Z][a-z]' spikes docs` reports no match.
