---
schema: "tiler-doc/v1"
id: "tiler.portal.status"
kind: "portal"
title: "Project status"
topics: ["status", "orientation"]
related: ["tiler.questions.open", "tiler.roadmap"]
---

# Project status

Tiler is an alpha-stage, bounded architectural prototype. It has an executable compiler-to-Metal value proof and reviewed experimental public boundaries, not a production tensor compiler, a general Metal runtime, or a stabilized API.

## Delivered bounded vertical

- **Fact — semantic and reference:** `tiler-ir` exposes a typed, multi-result semantic graph with checked construction, governed operation and type authority, canonical identity, shaped-value evidence, and a downstream reference-evaluation path. The implemented standard profile remains deliberately narrow; the [operation-family matrix](roadmap.md#operation-family-support-matrix) and [dtype maturity ledger](dtype-support.md) own its exact breadth.
- **Fact — compiler:** the ordinary `tiler-compiler` path reaches checked normalization, capability resolution, semantic-to-index refinement, fusion legality and covers, target feasibility, scheduling, physical-plan selection, structured KIR, verified kernel-program construction, and artifact construction for the bounded governed profile. The reviewed-draft `tiler_compiler::session::{compile, compile_governed}` facade and the concrete, still-reserved-for-review `tiler_compiler::target` draft let a caller install index-access lowering capabilities, select an ordered numerical-contract preference, and supply an ordered nonempty set of immutable validated target profiles. The target draft admits structurally attributed external guarantees and measurements, sparse per-axis quantitative facts whose omissions remain unknown, dimension-specific F32 numerical declarations, and exact full-resolved-type dispatch facts; it does not let a caller assert compiler-governed proof, exact emulation, or ABI layout. This is a usable alpha boundary, not a stable or workload-general API; shape-environment choice and planning budgets remain internal.
- **Fact — identity and schema:** the current source-derived artifact ledger is resolved value type v3, scheduled region v2, structured kernel v3, verified kernel program v5, artifact program v9, and neutral manifest schema 7.0. The envelope format and canonical encoding remain 1.0, and each of the four component schemas remains 1.0. These are separate subjects rather than one global version; [Artifact ABI](artifact-abi.md) owns the complete ledger and evolution rationale.
- **Fact — Metal artifact-to-proof path:** `tiler-metal` deterministically lowers the bounded structured KIR to MSL, `tiler-metal-aot` invokes the selected Apple offline toolchain and records provenance, and `tiler-build` assembles the carried Metal payload into the neutral artifact. The device-free `tiler-runtime` validates, preflights, and commits routing once; the retained `serial-sum-run` prototype then binds resources, dispatches, waits for exact command-buffer success, and validates readback.
- **Measurement — offline AOT:** the retained [Metal AOT proof](../tickets/prototype-metal-aot-slice.md) produced the selected fused and materialized programs on one Apple M4 Max with macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, Metal/AIR-LLD 32023.883, and macOS SDK 26.5 build 25F70; two independent links produced byte-identical 3,843-byte metallibs. This is deterministic construction evidence for that measured row, not runtime compatibility evidence.
- **Measurement — device execution:** the retained [runtime proof](../tickets/prototype-metal-runtime-proof.md) bit-compared 30 cases on one Apple M4 Max under `FlushSubnormalsToZeroF32`, covering three reduction classes, both selected-fused and materialized roles, and five cases per class/role. The selected route used one dispatch and no shared allocation; the materialized route used two dispatches and one shared allocation. This establishes that exact host, toolchain, program shape, numerical realization, and corpus only; it is neither a portable Apple-family guarantee nor a production runtime qualification.
- **Fact — artifact and cache infrastructure:** `tiler-artifact` exposes the reviewed public neutral codec/capability boundary, including carried compilation-subject and object bytes, and `tiler-cache` implements the immutable self-validating cross-process expansion cache. The complete inline orchestration that composes those pieces remains open.

## Authorized prototype

Tom authorized the bounded serial `f32` `Sum` Metal value proof in [ADR 0055](decisions/0055-use-a-serial-sum-for-the-first-metal-value-proof.md). The proof passes through the documented semantic, reference, optimizer, schedule, structured-kernel, artifact, cacheable-build, and guarded-runtime boundaries; a handwritten standalone kernel would not satisfy it. ADR 0067 selects the exact `nightly-2026-07-19` toolchain and dependent-array `StaticShape<RANK, EXTENTS>` family, with a retained [shape conformance harness](../spikes/shapes/nightly-dependent-static-shapes/README.md).

The delivered path recognizes two one-input/one-output bounded F32 shapes: a four-operation pointwise add or multiply over one input plus constants, and a four- or five-operation scale-bias-strict-serial-sum program with a deterministic fused candidate and a deliberately materialized comparison candidate. Its generic authorities are real and reusable, but their current admitted workload is not representative of broad tensor compilation. Quantized U4 construction has later-layer structural fixtures but still fails closed before executable support; the dtype ledger records that non-monotone evidence without promoting it to a vertical guarantee.

## Not yet delivered

- **Fact — inline developer experience:** the inline proc-macro frontend remains awaiting decision, and the complete cold/warm inline AOT and embedding workflow remains open. Implemented cache and AOT components do not by themselves satisfy the Milestone 0B exit; no default cache-root chooser or accepted cache-maintenance boundary has landed.
- **Fact — consumer integration:** no Candle adapter, einops-derived workload, or other production consumer path exists.
- **Fact — runtime product:** the device-execution code is retained in `prototypes/serial-sum-run`; there is no reusable live-device runtime, general pipeline cache, product fallback integration, broad buffer/shape handling, or production compatibility matrix.
- **Fact — breadth:** the compiler request recognizer, semantic operation set, dtype support, schedules, Metal lowering, and execution corpus are narrow. General backend support, wider dtypes and operations, dynamic workloads, parallel reductions, contractions, and optimized model inference remain separately tracked work.
- **Fact — stability:** reviewed public draft boundaries may still change during the alpha phase. Implemented canonical identities and lockstep schemas prevent accidental subject confusion; they do not promise long-term backward compatibility.

The workspace-member absence claims above are reproducible from the repository root:

```sh
test ! -d crates/tiler-macros
test ! -d crates/tiler-candle
! rg -n 'proc-macro\s*=\s*true' crates --glob Cargo.toml
! rg -n -i 'metal|objc|MTL' crates/tiler-runtime/Cargo.toml
```

## Evidence boundary

- Executable spikes and prototypes validate only their recorded representations, protocols, numerical cases, hosts, and toolchains.
- Apple compatibility remains bounded to measured toolchains and hosts; old-OS and device-family runtime coverage is not universal.
- Sound region-accuracy analysis exists for a narrow trusted-analyzer profile; independent certificate checking remains deferred.
- Multi-device, external-storage, and richer transfer work is deliberately deferred, not silently ready.

## Live work state

Ticketsplease is authoritative for current workflow state:

```sh
tkt rollup
tkt ready
tkt tracks
tkt reconcile
```

See the [work-tracking guide](work-tracking.md) before claiming work. The [roadmap](roadmap.md) describes proposed progression, while its operation matrix and the dtype ledger record bounded delivered support.
