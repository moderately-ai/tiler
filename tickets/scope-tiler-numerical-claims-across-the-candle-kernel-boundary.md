---
id: scope-tiler-numerical-claims-across-the-candle-kernel-boundary
title: Scope Tiler's numerical claims across the Candle kernel boundary
status: done
priority: p2
dependencies: []
related: [record-metal-runtime-compiler-provenance-gap, prototype-candle-metal-adapter]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, candle, metal, numerics]
---
Tiler compiles no MSL at runtime — ADR 0002 and ADR 0043 both forbid it, and `docs/backends/metal.md` now records why that exclusion is what keeps an artifact's toolchain provenance complete. The exclusion covers Tiler's kernels. It does not cover the process they run in.

**Fact — the first consumer compiles its own kernels at runtime.** In the local `huggingface/candle` working checkout at revision `4bb954d`, `Kernels::load_library` in `candle-metal-kernels/src/kernel.rs` compiles each built-in kernel source through `new_library_with_source` (that is, `newLibraryWithSource:options:error:`), caching by `Source`; `MetalDevice::compile` in `candle-core/src/metal_backend/device.rs` does the same for a `ug`-generated kernel. Those libraries are produced by the OS-resident `GPUCompiler.framework`, measured on the recorded Apple row as `metalfe-32023.921` on the macOS host and `metalfe-32023.830.1` inside a booted iOS 26.0 Simulator — neither of which is the `metalfe-32023.883` an artifact's provenance names.

**Inference — a Tiler kernel and a Candle kernel in one command buffer carry different numerical provenance.** Tiler's declared numerical realization, its recorded toolchain provenance, and ADR 0076's delivered-realization record all cover exactly the kernels Tiler emitted and compiled. A neighbouring Candle kernel operating on the same tensors has none of that, and its compiler moves with the OS build under a byte-identical Tiler artifact.

## The work

`docs/integration/candle.md` should state what a consumer may and may not conclude across that boundary: that a Tiler artifact's numerical realization is a claim about Tiler's kernels only; that a mixed Tiler/Candle program has no single numerical provenance; and what a reference comparison over such a program is therefore comparing. Decide whether that is a documented boundary or a checked one — whether anything should reject or warn when a conformance claim is read across a mixed program — and say which.

Re-pin the Candle source claim against whatever revision Tiler actually depends on when the adapter lands; `4bb954d` is a local working checkout on one machine, not a dependency pin. `verify-candle-metal-post-wait-error-checking` set the precedent of citing an exact local Candle commit for a source claim.

## Closes when

`docs/integration/candle.md` states the boundary, names whether it is enforced, and cites an exact inspected Candle revision.

## Outcome

`docs/integration/candle.md` gained **Numerical scope across the Candle kernel boundary**, and its Traceability section now claims ownership of what a consumer may conclude across that boundary. The section states that a Tiler numerical claim covers the kernels Tiler emitted and compiled and nothing else; that the two sides differ on three independent axes (compiler build, math mode, and the mechanism and time at which each is fixed); that an end-to-end reference diff over a mixed program measures the composition rather than Tiler's conformance, so attribution requires comparing at the covered operations' boundary; and — explicitly — that none of this is a defect in Candle.

**The enforcement question is answered: documented and reported, never checked at run time.** Two reasons, both recorded. Mixture is the only mode of use, because the adapter is a Candle custom op, so a predicate true on every reachable call carries no information and rejecting on it would reject the product. And the condition is not observable where a check would sit: `load_library` caches by `Source`, so reading `CANDLE_METAL_ENABLE_FAST_MATH` at adapter time establishes nothing about an already-cached library. What is obligatory instead is checkable — the Diagnostics section now requires any reported realization, delivered-realization record, or conformance claim to identify the operations it covers, so the scope cannot be read separately from the statement.

**Citation correction.** The ticket cited `4bb954d`, which is a `tomsanbear/candle` fork-branch commit that `huggingface/candle` does not contain. The section instead cites `31f35b147389700ed2a178ee66a91c3cc25cc80d` (0.11.0, the checkout's `origin/main`, and the revision six other corpus documents already use). Verified at that revision: `Kernels::load_library` at `candle-metal-kernels/src/kernel.rs:109` calling `new_library_with_source` at 122; `get_compile_options` at 182 reading `CANDLE_METAL_ENABLE_FAST_MATH` with default `true`; `MetalDevice::compile` at `candle-core/src/metal_backend/device.rs:101` calling it at 111 with `None` options. The section also records, rather than obscures, that Tiler declares no Candle dependency at all — so this is an inspected upstream revision, not a resolved pin.

Three consequences are ticketed rather than absorbed: [`correct-metal-provenance-candle-revision-citation`](correct-metal-provenance-candle-revision-citation.md) (the same bad `4bb954d` citation in `docs/backends/metal.md`, which is `contracts/artifacts` and out of this ticket's scope), [`repin-candle-numerical-scope-citation-at-adapter-admission`](repin-candle-numerical-scope-citation-at-adapter-admission.md) (re-pin at adapter admission), and [`decide-strict-realization-fallback-availability`](decide-strict-realization-fallback-availability.md) — `awaiting-decision`, because the accepted fallback rule applied to Candle's fast-math default means a strict `f32` contract has no valid Candle fallback and must fail closed, and whether losing availability is the right product behaviour is Tom's call rather than this ticket's.

`uv run --locked python scripts/docs.py render` and `tkt lint` pass.
