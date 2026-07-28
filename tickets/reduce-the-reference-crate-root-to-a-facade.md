---
id: reduce-the-reference-crate-root-to-a-facade
title: Reduce the reference crate root to a public facade
status: done
priority: p2
dependencies: []
related: [reconcile-index-oracle-ownership-prose]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [refactor, reference, progressive-disclosure]
---
Make `tiler-reference` disclose its public contract before its evaluation
mechanisms.

The crate root currently owns tensor representation, semantic registration and
evaluation, scalar dispatch, index evaluation, arithmetic, errors, and a large
test suite.

## Outcome

Keep a small `lib.rs` that introduces and re-exports the reference boundary.
Separate tensor representation, semantic registry/evaluation, scalar
registry/evaluation, index evaluation, arithmetic, errors, and tests into
shallow files.

Do not merge the semantic and scalar registries merely because both dispatch
behavior; they govern different identities and extension obligations. Preserve
the reference implementation's independence from compiler verification.

## Closes when

The crate root is a legible facade, each evaluation authority has one owner,
public paths and reference results are unchanged, and the full gate passes.

## Outcome — 4,100-line root reduced to a 70-line facade (2026-07-27)

`crates/tiler-reference/src/lib.rs` went from **4,100 lines to 70**. It now declares the modules, re-exports the public boundary, states the resource limits, and owns the one rule that spans every module — the arithmetic NaN canonicalization both oracles apply and neither may define separately.

| module | lines | authority |
| --- | --- | --- |
| `lib.rs` | 70 | the facade |
| `tensor` | 319 | what a reference value *is* |
| `registry` | 771 | the semantic capability registry and its dispatch vocabulary |
| `evaluate` | 666 | executing a semantic program against that registry |
| `standard` | 165 | the one provider the crate ships |
| `identity` | 174 | canonical identity encoding for a frozen registry |
| `error` | 571 | every typed failure any of the above reports |
| `tests` | 1,516 | the suite, previously 37% of the crate root |
| `oracle`, `arithmetic` | 2,004 + 425 | unchanged, apart from repointed imports |

**The two registries are not merged, and the root now says why.** `registry` and `oracle` both resolve behaviour by key, which is the resemblance that invites merging them; they govern different identities and different extension obligations, so a shared mechanism would erase a distinction the contracts depend on. That was implicit in the old file's ordering and is now stated at the facade and at `registry`'s own header.

**Public paths are unchanged**, checked by the thing that would break: `cargo check --workspace --all-targets` compiles `tiler-compiler`, `tiler-metal`, `tiler-runtime`, and both prototypes against the same `tiler_reference::*` paths they used before. Every re-export that was `pub` at the root is `pub use`d there now.

**Reference results are unchanged**: 296 tests across `tiler-reference` and `tiler-compiler` pass, including the compiler's conformance comparisons against this oracle.

**What the split cost, stated rather than hidden.** Items the modules share had to widen from private to `pub(crate)` — `ReferenceWork`, `EvaluationRetention`, several registry struct fields, and a handful of helper functions. None crosses the crate boundary and no public type gained a field or method. That is the real price of splitting a module that used one file's privacy as its encapsulation, and it is worth naming because a reader diffing the change will see a lot of `pub(crate)` and should know it is mechanical rather than a widening of the contract.
