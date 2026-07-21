---
schema: "tiler-doc/v1"
id: "tiler.research.workspace.prototype-crate-layout-and-msrv"
kind: "research"
title: "Prototype crate layout and Rust MSRV"
topics: ["rust", "workspace", "msrv", "architecture"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "partially-adopted"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.architecture", "tiler.contract.frontend-integration"]
adopted_by: ["ADR-0056", "ADR-0057", "ADR-0065"]
ticket: "prototype-foundation-contract"
---

# Prototype crate layout and Rust MSRV

## Question

What is the smallest Rust workspace that mechanically preserves Tiler's
compiler/artifact/runtime boundaries for the authorized value proof, and which
Rust version supports its cache protocol without an additional lock adapter?

## Crate-layout findings

**Fact:** The architecture requires runtime execution not to link the optimizer,
backend emission not to own runtime objects, and semantic/index/schedule/kernel
representations to remain separable while initially permitting them to be
modules in one crate.

**Inference:** Four reusable libraries are the minimum useful enforcement
boundary:

```text
tiler-ir       -> []
tiler-artifact -> [tiler-ir]
tiler-compiler -> [tiler-ir, tiler-artifact]
tiler-metal    -> [tiler-ir, tiler-artifact]
```

Two non-published proof executables keep compiler and runtime dependency graphs
honest:

```text
prototype-compile -> [tiler-ir, tiler-compiler, tiler-metal, tiler-artifact]
prototype-run     -> [tiler-artifact, platform Metal bindings]
```

`prototype-compile` constructs the fixed graph, reference-evaluates it, selects
the serial schedule, lowers to MSL, invokes Apple's offline compiler, and writes
the bounded artifact. `prototype-run` validates and loads that artifact,
preflights the live device, commits routing, dispatches, and compares readback.
Separating them prevents the runtime proof from importing optimizer internals.

Semantic, index/access, schedule, program, and structured-kernel IRs remain
modules in `tiler-ir`. Fusion, scheduling, costing, and explainability remain
modules in `tiler-compiler`. MSL emission and offline invocation remain separate
modules in `tiler-metal`. Frontend, proc-macro, Candle, generalized cache, and
reusable Metal-runtime crates wait until the vertical proof establishes their
need.

Subsequent evaluator implementation supplied new evidence: reference values,
execution traversal, and executable operation capabilities form a genuine
downstream consumer boundary. ADR 0065 therefore adds `tiler-reference ->
tiler-ir` while preserving the remaining dependency conclusions below.

The counterpoint is ceremony compared with one core crate and executable. That
smaller layout cannot mechanically test the accepted runtime/optimizer
separation. Conversely, scaffolding every proposed future component would
prematurely stabilize packaging. The four-plus-two layout enforces only the
boundaries under test and preserves later source-compatible splits behind
unstable re-exports.

## MSRV findings

**Fact:** Rust 1.89 stabilized `std::fs::File::{lock, lock_shared, try_lock,
try_lock_shared, unlock}` and `TryLockError`. The standard-library documentation
states that the implementation currently maps to `flock` on Unix and
`LockFileEx` on Windows and releases on close or explicit unlock. This directly
supports Tiler's accepted stable per-key advisory-lock protocol. See the
[Rust 1.89 release announcement](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/)
and [`std::fs::File`](https://doc.rust-lang.org/stable/std/fs/struct.File.html).

Rust 2024 requires Rust 1.85. `proc_macro::Literal::byte_string` and the atomic
filesystem primitives needed by the prototype predate 1.89. Experimental
`proc_macro::tracked` remains nightly-only and therefore cannot make external
Xcode state a stable tracked input.

**Proposal:** Set every prototype workspace package to edition 2024 and
`rust-version = "1.89"`. Keep file locking behind an internal cache-lock adapter
so a later compatibility requirement can substitute an audited implementation
without changing semantic IR, public APIs, artifact identity, or cache format.

The counterpoint is excluding consumers pinned to 1.88 or older. Supporting
them now would add a third-party or platform lock layer before the experimental
prototype has users, while every macro consumer ultimately depends on the
host-side AOT/cache path. Per-crate lower MSRV promises are therefore deferred.

## Superseding toolchain evidence

The Rust 1.89 finding remains correct for advisory locking and explains the
implemented stable workspace floor. It no longer governs the complete
prototype toolchain. Follow-up [shape-parameter
research](../shapes/nightly-const-shape-parameters.md) established that one
arbitrary-rank exact-evidence family requires dependent array const parameters;
ADR 0067 accepts an exact dated nightly for that capability and supersedes ADR
0057. Cargo's `rust-version` cannot express this channel requirement, so the
workspace pin moves to `rust-toolchain.toml` when the conformance harness lands.

## Traceability

Adopted historically by [ADR 0056](../../decisions/0056-use-four-libraries-and-two-proof-executables.md)
and [ADR 0057](../../decisions/0057-set-the-prototype-msrv-to-rust-1-89.md),
whose toolchain portion is superseded by [ADR
0067](../../decisions/0067-use-pinned-nightly-dependent-static-shapes.md).
The [architecture contract](../../architecture.md) owns dependency direction.
