---
id: restore-the-metal-toolchain-so-the-workspace-gate-can-run-green
title: Restore the Metal toolchain so the workspace gate can run green
status: blocked
priority: p1
dependencies: []
related: [root-cause-the-intermittent-leaky-test-in-the-workspace-gate]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [environment, gate, metal, toolchain]
---
## What is broken

**Measurement, 2026-08-04, host Darwin 27.0.0.** `make check` on commit `753a06afe643b8a4ccfd07c8089574e09acc4610` fails 18 of 2427 tests (log: `/tmp/tiler-gate-753a06af.log`). All 18 are in the two packages whose tests invoke the Metal compiler at run time — `tiler-macros` `aot::tests` (8) and `tiler-prototype-compile` (10). Twelve print the root cause directly:

```
ToolchainUnavailable { tool: "metallib", phase: Discovery, detail: "xcrun ... unable to find utility \"metallib\"" }
```

The other six assert on the artifact the failed compile never produced ("must produce an artifact, not a retained diagnostic"). Zero failures outside these two packages.

## Root cause

**Fact.** `xcode-select -p` → `/Applications/Xcode-beta.app/Contents/Developer` (Xcode 27.0, build 27A5228h). `xcrun -find metallib` under that developer dir fails: the beta's Metal toolchain component is not installed. The stable `/Applications/Xcode.app` (Xcode 26.6, build 17F113) resolves `metallib` via the downloaded MetalToolchain asset `v17.6.109.0`. Reproduce: `./deps.sh --check` → `[fail] metallib is unavailable; install/select full Xcode and its Metal toolchain`.

**Inference.** The host previously gated green with Metal work landing, so either the developer-dir selection moved to the beta or the beta updated without its Metal toolchain component. Either way this is a host-toolchain regression, not a code defect.

## Why this is blocked, and on whom

Selecting or mutating host Xcode/SDK components is Tom's decision under the toolchain policy in AGENTS.md. The two candidate fixes, either of which Tom may apply:

1. `sudo xcode-select -s /Applications/Xcode.app` — selects Xcode 26.6, whose Metal toolchain is present and matches the SDK generation the declaration ledger cites.
2. `xcodebuild -downloadComponent metalToolchain` under the beta — keeps Xcode 27.0 selected but downloads its Metal toolchain. Note this changes Metal toolchain provenance for any future retained evidence relative to the recorded rows.

## Closes when

`./deps.sh --check` reports the Metal toolchain present, `make check` on the then-current `main` reports the 18 tests passing (or any residual failure re-attributed to a real defect), the exact resulting toolchain component is recorded here, and any Metal-dependent work that was held back on this block is released.

Until then: coordination policy is that docs/ticket integrations proceed on `tkt lint` plus review; code integrations require targeted package gates green plus a full-workspace run whose only failures are these 18, each verified to carry the toolchain attribution above.
