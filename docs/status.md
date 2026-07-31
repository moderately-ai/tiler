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
- **Fact — compiler:** the ordinary `tiler-compiler` path reaches checked normalization, capability resolution, semantic-to-index refinement, fusion legality and covers, target feasibility, scheduling, physical-plan selection, structured KIR, verified kernel-program construction, and artifact construction for the bounded governed profile. The `tiler_compiler::session` facade remains a reviewed experimental draft as a whole: its public `CompileRequest`, `InstalledCapabilities`, `compile`, `compile_governed`, compilation/result readers, and typed failure surface may still be reshaped during alpha. It already lets an out-of-crate caller install index-access lowering capabilities and state an ordered numerical-contract preference. Separately, Tom accepted the experimental `tiler_compiler::target::{TargetProfileBuilder, TargetProfile, TargetRequest}` boundary on 2026-07-30; it admits structurally attributed external guarantees and measurements, sparse per-axis quantitative facts whose omissions remain unknown, dimension-specific scalar numerical declarations, and exact full-`ResolvedValueType` dispatch facts. It does not let a caller assert compiler-governed proof, exact emulation, or ABI layout. These are usable alpha boundaries, not stable or workload-general APIs; shape-environment choice and planning budgets remain internal.
- **Fact — bounded Metal profile projection:** `tiler-build` now projects a caller-stated Metal F32 subnormal fact into the target-neutral compiler profile because it is the first crate that can depend on both sibling authorities. The public path requires a structurally valid caller-vouched measurement source, converts every represented Metal F32 mode including flush zero sign, rejects conflicts transactionally, never finalizes the profile, and retains Metal's independent emitted-operation recheck. It does not prove that the supplied contexts produced the independently supplied fact, construct an authoritative production profile, source quantitative limits or F32 dispatchability, bind a plan or artifact to the evidence, validate runtime applicability, or extend the declaration to F16, BF16, F64, or iOS devices. Those obligations are the P0 follow-up [`construct-and-bind-the-first-authoritative-metal-compile-profile`](../tickets/construct-and-bind-the-first-authoritative-metal-compile-profile.md).
- **Fact — identity and schema:** the current source-derived artifact ledger is resolved value type v3, scheduled region v2, structured kernel v4, verified kernel program v5, artifact program v11, and neutral manifest schema 9.0. The target feasibility profile's checked descriptor encoding is v9, its complete declaration is v10, and the governed feasibility vocabulary is `tiler.feasibility.phased-capability-and-numerical-honourability.v4` revision 1. The envelope format and canonical encoding remain 1.0; the program, ABI-expression, and guard-and-routing component schemas remain 1.0, while the target-requirement component schema is 2.0. These are separate subjects rather than one global version; [Artifact ABI](artifact-abi.md) owns the complete ledger and evolution rationale.
- **Fact — Metal artifact-to-proof path:** `tiler-metal` deterministically lowers the bounded structured KIR to MSL, `tiler-metal-aot` invokes the selected Apple offline toolchain and records provenance, and `tiler-build` assembles the carried Metal payload into the neutral artifact, including compiler-minted target requirements bound to exact prepared entries. The device-free `tiler-runtime` keeps its refusal-only preflight for routes with unanswered deferred predicates and also exposes a staged `RoutePreparation` whose exact-entry answers must satisfy the carried directional requirements before the same one-way commit. The retained `serial-sum-run` prototype binds resources, dispatches, waits for exact command-buffer success, and validates readback; the measured corpus below predates and does not itself establish hardware execution of the exact-entry deferred route.
- **Measurement — offline AOT:** the retained [Metal AOT proof](../tickets/prototype-metal-aot-slice.md) produced the selected fused and materialized programs on one Apple M4 Max with macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, Metal/AIR-LLD 32023.883, and macOS SDK 26.5 build 25F70; two independent links produced byte-identical 3,843-byte metallibs. This is deterministic construction evidence for that measured row, not runtime compatibility evidence.
- **Measurement — device execution:** the retained [runtime proof](../tickets/prototype-metal-runtime-proof.md) bit-compared 30 cases on one Apple M4 Max under `FlushSubnormalsToZeroF32`, covering three reduction classes, both selected-fused and materialized roles, and five cases per class/role. The selected route used one dispatch and no shared allocation; the materialized route used two dispatches and one shared allocation. This establishes that exact host, toolchain, program shape, numerical realization, and corpus only; it is neither a portable Apple-family guarantee nor a production runtime qualification.
- **Fact — artifact and cache infrastructure:** `tiler-artifact` exposes the reviewed public neutral codec/capability boundary, including carried compilation-subject and object bytes, and `tiler-cache` implements the immutable self-validating cross-process expansion cache. The complete inline orchestration that composes those pieces remains open.

## Authorized prototype

Tom authorized the bounded serial `f32` `Sum` Metal value proof in [ADR 0055](decisions/0055-use-a-serial-sum-for-the-first-metal-value-proof.md). The proof passes through the documented semantic, reference, optimizer, schedule, structured-kernel, artifact, cacheable-build, and guarded-runtime boundaries; a handwritten standalone kernel would not satisfy it. ADR 0067 selects the exact `nightly-2026-07-19` toolchain and dependent-array `StaticShape<RANK, EXTENTS>` family, with a retained [shape conformance harness](../spikes/shapes/nightly-dependent-static-shapes/README.md).

The delivered path recognizes two one-input/one-output bounded F32 shapes: a four-operation pointwise add or multiply over one input plus constants, and a four- or five-operation scale-bias-strict-serial-sum program with a deterministic fused candidate and a deliberately materialized comparison candidate. Its generic authorities are real and reusable, but their current admitted workload is not representative of broad tensor compilation. Quantized U4 construction has later-layer structural fixtures but still fails closed before executable support; the dtype ledger records that non-monotone evidence without promoting it to a vertical guarantee.

## Not yet delivered

- **Fact — inline developer experience:** the frontend crate boundary is settled and the workflow behind it is not. Tom ratified the two-crate topology and the public `tiler::tensor!` path on 2026-07-30 and accepted the exact facade surface and the artifact-family-selection placement on 2026-07-31, so `tiler` and `tiler-macros` are admitted workspace members recorded by [ADR 0088](decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md). What they carry is the `tensor!` re-export, the generated-path anchor the expansion names, the expansion's stated canonical artifact-family delivery policy, and — since 2026-07-31 — its stated expansion-cache root policy. What they do not carry is a grammar: empty input expands to an inert anchor and any non-empty input is a spanned `compile_error!`, so region syntax, expansion, symbol binding, runtime value adaptation, and the complete cold/warm inline AOT and embedding workflow all remain open. Implemented cache and AOT components do not by themselves satisfy the Milestone 0B exit.

  **Fact — the cache-root chooser exists and its consumer-visible spelling is unaccepted.** [The root policy note](research/cache/root-policy.md) records the derivation — `TILER_EXPANSION_CACHE_DIR` when set, otherwise `$HOME/Library/Caches/ai.moderately.tiler/expansion`, with `off` as an explicit disable and a typed refusal for every unusable or non-private root — and `crates/tiler-macros/src/cache_root.rs` implements it as a crate-private draft. Two things are deliberately *not* done: nothing calls the resolver, because `tensor!` opens no cache and `tiler-macros` holds no edge to `tiler-cache` (`grep -n 'tiler-cache' crates/tiler-macros/Cargo.toml` reports no match); and the variable name, the disable value, the derived path, and the refusal text are Tom's under [ADR 0075](decisions/0075-scope-public-boundary-approval-by-change-category.md) and undecided. [`record-the-expansion-cache-root-policy-decision`](../tickets/record-the-expansion-cache-root-policy-decision.md) carries the ADR and the contract propagation.

  **Correction — 2026-07-31.** This bullet previously ended "and no accepted cache-maintenance boundary has landed". That has been false since 2026-07-31: `crates/tiler-cache/src/expansion.rs` records that Tom accepted the maintenance boundary under [`accept-the-expansion-cache-maintenance-boundary`](../tickets/accept-the-expansion-cache-maintenance-boundary.md), which is `done`, and `account`, `collect`, and `purge` sit on `ExpansionCache` with their report vocabulary re-exported. What remains open is the *schedule* rather than the boundary — nothing calls any of them automatically, by design — and [`decide-the-expansion-cache-collection-schedule`](../tickets/decide-the-expansion-cache-collection-schedule.md) owns it.
- **Fact — consumer integration:** no Candle adapter, einops-derived workload, or other production consumer path exists.
- **Fact — runtime product:** the device-execution code is retained in `prototypes/serial-sum-run`; there is no reusable live-device runtime, general pipeline cache, product fallback integration, broad buffer/shape handling, or production compatibility matrix.
- **Fact — breadth:** the compiler request recognizer, semantic operation set, dtype support, schedules, Metal lowering, and execution corpus are narrow. General backend support, wider dtypes and operations, dynamic workloads, parallel reductions, contractions, and optimized model inference remain separately tracked work.
- **Fact — stability:** reviewed public draft boundaries may still change during the alpha phase. Implemented canonical identities and lockstep schemas prevent accidental subject confusion; they do not promise long-term backward compatibility.

The workspace-member claims above are reproducible from the repository root. Two are presence claims and two are absence claims, and the proc-macro check is written as an equality against the whole `crates` tree rather than a bare match, so it says no in both directions — a second proc-macro crate fails it just as a missing one does:

```sh
test -d crates/tiler && test -d crates/tiler-macros
test "$(rg -l 'proc-macro\s*=\s*true' crates --glob Cargo.toml)" = crates/tiler-macros/Cargo.toml
test -f crates/tiler/tests/facade/fail/undefined_grammar.stderr
test ! -d crates/tiler-candle
! rg -n -i 'metal|objc|MTL' crates/tiler-runtime/Cargo.toml
```

The third line is the checked-in compile-fail golden behind "`tensor!` has no grammar": it is the evidence that rejecting undefined input is a tested behaviour rather than a description of one.

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
