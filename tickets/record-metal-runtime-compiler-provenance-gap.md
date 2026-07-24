---
id: record-metal-runtime-compiler-provenance-gap
title: Record that Metal artifact toolchain provenance names only the offline compiler
status: in-progress
priority: p2
dependencies: []
related: [probe-metal-runtime-compilation-numerics, declare-metal-numerical-honourability, repoint-adr-0076-evidence-at-the-numerical-record, prototype-metal-bundle-assembly]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, metal, numerics, provenance]
claimed_from: todo
assignee: agent-record-metal-runtime-compiler-provenance-gap
lease_expires_at: 1784929884
---
[probe-metal-runtime-compilation-numerics](probe-metal-runtime-compilation-numerics.md) measured something the Metal contracts do not yet say, and it is about identity rather than about numbers.

**Measurement — a Metal host resolves two compilers, not one.** On the recorded row (macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, Apple M4 Max), `xcrun --sdk macosx metal --version` reports `Apple metal version 32023.883 (metalfe-32023.883)`, resolved from the Xcode MetalToolchain asset. A library compiled in process by `newLibraryWithSource:options:`, serialized through an `MTLBinaryArchive`, embeds `Apple metal version 32023.921 (metalfe-32023.921)`; that compiler is `/System/Library/PrivateFrameworks/MTLCompiler.framework`, `CFBundleVersion` 382.5, which ships with the OS rather than with Xcode. Both are recorded in [`results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv`](../spikes/apple-targets/results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv) as `environment.metal_version` and `environment.runtime_compiler`, and the gate compares against both.

**Inference — an artifact's toolchain provenance is therefore not the provenance of everything that runs.** Tiler's Metal artifact identity records the offline `metal` and `metallib` builds. That identifies the compiler for an AOT-compiled kernel and identifies nothing about a kernel compiled through `newLibraryWithSource:`, whose compiler moves with the OS and can change without the artifact changing at all. The [artifact-compatibility record](../docs/research/apple-targets/artifact-compatibility.md) already establishes that "Xcode 26.6" is an insufficient toolchain identity; this is a second and independent insufficiency on a different axis.

**Fact — the two paths currently agree, which is why this is a contract-wording ticket and not a correctness one.** All 40 measured runtime cases return bit patterns identical to their offline counterparts, so nothing is broken today. The contracts should say that this is a bounded measurement on one host row that happens to hold between two specific compiler builds, not a property of Metal, and that a delivered numerical realization read off an offline build is not thereby true of a runtime-compiled kernel.

## The work

Record in `docs/backends/metal.md` — and in `docs/artifact-abi.md` if that is where toolchain provenance is actually sited, which is worth checking rather than assuming — that:

- the recorded toolchain provenance identifies the offline compiler only;
- a runtime-compiled kernel is compiled by a separate, separately versioned compiler that is not part of that provenance, and on the measured host is a different build;
- the measured agreement between the two is a bounded host-row measurement, with the pointer to `tiler.research.apple-targets.numerical-behaviour` findings 8 and 9.

Decide and state whether Tiler's Metal story admits a runtime-compiled kernel at all. If the AOT boundary is meant to exclude it entirely, that is a stronger and simpler answer than a provenance caveat, and the contract should say so — but say it explicitly rather than leaving the case unaddressed, because Candle's own command stream reaches `newLibraryWithSource:` and `prototype-metal-bundle-assembly` will have to answer the same question.

Do **not** widen ADR 0076's conclusion here. Finding 9 supports it and finding 8 strengthens its central argument. If the ADR gains anything it is one sentence — that a versioned target numerical fact must identify which compiler the realization was measured on — and `repoint-adr-0076-evidence-at-the-numerical-record` holds `contracts/decisions`.

## Scope note

`contracts/artifacts` covers both `docs/backends/**` and `docs/artifact-abi.md`. `declare-metal-numerical-honourability` also holds `contracts/artifacts` and owns the *subnormal* half of the `docs/backends/metal.md` numerics text; this ticket owns the *provenance* half only. Sequence them rather than editing the same paragraph twice.
