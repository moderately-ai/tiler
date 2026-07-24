---
id: scope-tiler-numerical-claims-across-the-candle-kernel-boundary
title: Scope Tiler's numerical claims across the Candle kernel boundary
status: in-progress
priority: p2
dependencies: []
related: [record-metal-runtime-compiler-provenance-gap, prototype-candle-metal-adapter]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, candle, metal, numerics]
claimed_from: todo
assignee: agent-scope-tiler-numerical-claims-across-the-candle-kernel-boundary
lease_expires_at: 1784932575
---
Tiler compiles no MSL at runtime — ADR 0002 and ADR 0043 both forbid it, and `docs/backends/metal.md` now records why that exclusion is what keeps an artifact's toolchain provenance complete. The exclusion covers Tiler's kernels. It does not cover the process they run in.

**Fact — the first consumer compiles its own kernels at runtime.** In the local `huggingface/candle` working checkout at revision `4bb954d`, `Kernels::load_library` in `candle-metal-kernels/src/kernel.rs` compiles each built-in kernel source through `new_library_with_source` (that is, `newLibraryWithSource:options:error:`), caching by `Source`; `MetalDevice::compile` in `candle-core/src/metal_backend/device.rs` does the same for a `ug`-generated kernel. Those libraries are produced by the OS-resident `GPUCompiler.framework`, measured on the recorded Apple row as `metalfe-32023.921` on the macOS host and `metalfe-32023.830.1` inside a booted iOS 26.0 Simulator — neither of which is the `metalfe-32023.883` an artifact's provenance names.

**Inference — a Tiler kernel and a Candle kernel in one command buffer carry different numerical provenance.** Tiler's declared numerical realization, its recorded toolchain provenance, and ADR 0076's delivered-realization record all cover exactly the kernels Tiler emitted and compiled. A neighbouring Candle kernel operating on the same tensors has none of that, and its compiler moves with the OS build under a byte-identical Tiler artifact.

## The work

`docs/integration/candle.md` should state what a consumer may and may not conclude across that boundary: that a Tiler artifact's numerical realization is a claim about Tiler's kernels only; that a mixed Tiler/Candle program has no single numerical provenance; and what a reference comparison over such a program is therefore comparing. Decide whether that is a documented boundary or a checked one — whether anything should reject or warn when a conformance claim is read across a mixed program — and say which.

Re-pin the Candle source claim against whatever revision Tiler actually depends on when the adapter lands; `4bb954d` is a local working checkout on one machine, not a dependency pin. `verify-candle-metal-post-wait-error-checking` set the precedent of citing an exact local Candle commit for a source claim.

## Closes when

`docs/integration/candle.md` states the boundary, names whether it is enforced, and cites an exact inspected Candle revision.
