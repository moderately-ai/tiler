---
id: drive-the-build-orchestrator-from-a-checked-compiler-plan
title: Drive the build orchestrator from a checked compiler plan
status: done
priority: p2
dependencies: [accept-and-publish-validated-artifacts-through-the-expansion-cache]
related: []
scopes: [implementation/compiler, implementation/metal, implementation/artifact, implementation/workspace, implementation/build, contracts/decisions, contracts/foundation, implementation/cargo-lock, implementation/metal-aot, project/tickets]
shared_scopes: []
paths: []
tags: [build, compiler, metal]
---
## User-visible outcome

A checked compiler product reaches the admitted `tiler-build` path without prototype code manually reconstructing backend, artifact, or cache inputs. The reusable build-time flow owns compiler plan consumption through artifact acceptance while keeping frontend syntax and runtime device objects outside it.

## Implementation keys

The first missing call site is the handoff from the checked selected physical plan and structured lowering authorities into `tiler-metal` emission. Add `tiler-compiler` and `tiler-metal` edges to `tiler-build` only with a concrete orchestration type that consumes their existing verified products.

Preserve the direction accepted by ADR 0085: the orchestrator calls compiler, backend, artifact, AOT, and cache authorities; none calls back into it. Do not turn the initial serial-sum support profile into the public compilation model, do not let Candle types cross the boundary, and keep unsupported lowering or emission cases typed and fail-closed.

Move the prototype's reconstructive glue behind this path only after identity fixtures are recomputed on the merged tree and the reusable flow proves the same value result. Exact public crate, type, and call-site boundaries require Tom's review before acceptance.

## Graph maintenance

When the checked plan drives backend emission and the resulting artifact reaches the already-landed cache path, update the architecture's implemented dependency list, rebaseline every pinned identity on the final tree, close this ticket, and inspect `tkt ready` for the next dependency-satisfied build/integration work. Split frontend proc-macro consumption or reusable runtime execution into separately reviewed tickets rather than expanding this crate across those boundaries.

## Delivered

`PlanAlternative` now retains its owning `Compilation`, and that compilation publishes the complete compiler-offered provider environment. The accepted `tiler-build` facade consumes that owner-linked plan, emits its kernels, prepares and executes the Metal AOT request only on a cache miss, assembles the neutral artifact without replaying ABI expressions, and returns only after the producer-side verified artifact agrees with the accepted decoded envelope.

The serial-sum producer now supplies policy, target facts, output names, and sidecars to that facade rather than reconstructing compiler, backend, artifact, or cache inputs. Its duplicate artifact assembler and duplicate low-level cache protocol tests were deleted; the reusable crate retains the hit-without-recompile, selected-versus-materialized identity, fail-before-toolchain, and decoded multi-stage dependency-order proofs.

Targeted compiler, build, and producer suites and Clippy pass. Each new assertion was perturbed and observed failing. The final-tree producer generated all six proof members with the system Metal toolchain, and the Apple M4 Max runner returned bit-identical governed results for all 30 fused and materialized cases. Existing pinned identities and Metal source goldens remained current, so no rebaseline was required. The architecture dependency list and accepted ADR 0085 now describe the implemented boundary.
