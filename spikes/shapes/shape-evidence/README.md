---
schema: "tiler-doc/v1"
id: "tiler.spike.shapes.shape-evidence"
kind: "experiment"
title: "Stable-Rust shape-evidence feasibility spike"
topics: ["shapes", "rust", "semantics", "diagnostics"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.shapes.stable-rust-shape-evidence"]
entrypoints: ["spikes/shapes/shape-evidence/src/lib.rs", "spikes/shapes/shape-evidence/measure.sh"]
last_verified: "2026-07-20"
ticket: "prototype-shape-evidence-spike"
---

# Stable-Rust shape-evidence feasibility spike

This isolated Rust 1.89 crate tests optional shape refinements over graph-owned
typed values. The graph remains authoritative: only it constructs refined
handles after checking canonical metadata, and all evidence-preserving
operations delegate to the unrefined admission path before rechecking results.

The model covers `Rank<R>`, `Exact<S>`, explicit weakening, pointwise
propagation, statically checked reduction axes, graph-owned same-shape
witnesses, foreign-proof rejection, evidence-neutral canonical identity, and
downstream static-shape descriptions which grant no authority by themselves.
Trybuild cases retain the Rust 1.89 diagnostics for rank/exact mismatches,
invalid or duplicate axes, sealed-evidence implementation, and handle forgery.

Run the correctness and diagnostic suite:

```sh
cargo +1.89.0 test --manifest-path spikes/shapes/shape-evidence/Cargo.toml
cargo +1.89.0 clippy --manifest-path spikes/shapes/shape-evidence/Cargo.toml --all-targets -- -D warnings
```

Regenerate the 1/10/100/1,000-shape workloads and repeat the bounded host
measurement:

```sh
spikes/shapes/shape-evidence/measure.sh
```

Raw run products are ignored. The compact checked-in result is
[`measurements/summary.json`](measurements/summary.json). One sample per case
is sufficient to reject catastrophic scaling, not to estimate a production
compile-time cost distribution.
