---
schema: "tiler-doc/v1"
id: "tiler.portal.spike.indexing-guide"
kind: "portal"
title: "Index/access contract spike"
topics: ["indexing", "access-maps", "rust"]
---

# Index/access contract spike

This dependency-free Rust crate mechanically checks the key ADR 0046
boundaries. It is intentionally not production scaffolding.

Run:

```sh
cargo test --manifest-path spikes/indexing/index-access-model/Cargo.toml
```

The tests cover deterministic canonicalization, permutation, broadcast read
aliasing, unique write ownership, static and symbolic reshape expressions,
positive divisors, noncontiguous allocation-relative views, negative-stride
rejection, data-dependent/nonlinear rejection, tail masking, and a `u32`
counterexample whose wide element-offset path remains correct.
