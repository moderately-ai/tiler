---
id: restore-the-spikes-against-the-composed-numerical-contract
title: Restore the contract-naming spikes against the composed numerical contract
status: in-progress
priority: p3
dependencies: [compose-the-numerical-contract-from-its-decided-dimensions]
related: [restore-the-scalar-cpu-vertical-spike-against-the-current-crates]
scopes: [research/cache, research/extensions, research/target-profiles, research/scheduling, research/numerics, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [spikes, maintenance, numerics]
claimed_from: todo
assignee: agent-spike-restore
lease_expires_at: 1785960717
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

**Fact — observed 2026-08-01 at base `29a9680`, and the cache spike needs a second edit this enumeration does not name.** `retire-the-device-translation-policy-from-the-cache-spike-and-its-citing-records` ran `CARGO_TARGET_DIR=./target cargo check` from `spikes/cache/build-tool-exercise` and `CARGO_TARGET_DIR=./target cargo run -- results/2026-07-31-macos-arm64.json` from `spikes/target-profiles/scalar-cpu-vertical`, and the compiler's own lines are what these facts come from rather than a re-reading of the source:

- `spikes/cache/build-tool-exercise` reports **two** errors, not one. The enumerated `E0599` at `envelope/src/lib.rs:59` is the first; the second is `E0560: struct BackendEntryRef has no field named payload` at `envelope/src/lib.rs:160`, which is the `payload` → `payloads` step `restore-the-scalar-cpu-vertical-spike-against-the-current-crates` repaired for the other spike and nothing repaired for this one. The single-object spelling is `payloads: vec![payload]`, and the delivery-position decision that ticket had to take does **not** arise here: `grep -rn "DecodedProgram\|decode_artifact\|payloads" expansion-macro/src/lib.rs envelope/src/lib.rs consumer/src/lib.rs` finds no call site, because this spike assembles and encodes an envelope and never decodes one.
- `spikes/target-profiles/scalar-cpu-vertical`'s site has drifted to `src/vertical.rs:779` from the `:759` recorded above — the 2119b20 restoration moved it — and it is that spike's only remaining error. The run therefore stops at compilation and never reaches the fixture, so `results/2026-07-31-macos-arm64.json` is unchanged (sha256 `7c774b159d06f489c6c8d8ab44d29ae09d277b5fbd5eb0da9e4530da05877196`) and re-running it is still this ticket's work rather than something a later reader can assume happened.

*Inference.* The cache spike's own `Closes when` obligation — build **and run** — was already broader than the site list, and this is the gap that made the difference visible. Whoever claims this ticket should expect the enumeration to be a floor rather than a census, and re-derive it from a clean `cargo check` per spike.

**Fact — one of the three has a live owner.** `restore-the-scalar-cpu-vertical-spike-against-the-current-crates` was in progress against `spikes/target-profiles/scalar-cpu-vertical` when this drift was introduced, which is why that spike was left alone rather than edited underneath it. Whoever claims this ticket checks that ticket's state first and either coordinates or narrows this one to the other two scopes.

## The research records naming the same removed spellings

**Fact — four retained research documents name a Rust identifier that no longer exists**, found by `grep -rn 'StrictF32\b\|FlushSubnormalsToZeroF32\|RelaxedF32\|ReassociateF32' docs/research` on the landing commit: `docs/research/scheduling/first-metal-contraction-realizations.md:85`, `docs/research/numerics/first-quantized-lm-profile.md:134` and `:211`, `docs/research/program-planning/first-metal-lm-workload.md:221`, and `docs/research/extensions/backend-provider-composition.md:201` and `:292` — the last two being compilable example code.

They were left alone deliberately rather than swept. Each sits inside a recorded measurement or derivation, so the edit is a *spelling* correction inside a retained record and has to preserve what the record claims; `docs/research/program-planning/**` was additionally held by a live ticket at the time. The two `backend-provider-composition.md` sites are the sharper ones: they are example code a reader is meant to be able to run.

## Boundaries

- A spike runs by hand from its own directory using the invocation its README records; a build that is only *checked* is not the evidence. Re-run each spike and confirm the fixture or golden it retains still matches, rather than only making it compile.
- Do not repoint a `trybuild` golden by copying the new output without reading it: the fail case's message is the claim, and a diagnostic that moved for a different reason would be laundered by a blind rebaseline.

## Closes when

All three spikes build and run from their own directories, every retained fixture or golden they cite still matches or has been rebaselined with the reason recorded, the six research-record sites name the current spelling without changing what each record claims, and `grep -rn 'NumericalContract::[A-Z][a-z]' spikes docs` reports no match.
