---
id: reconcile-illustrative-operation-names-with-governed-keys
title: Reconcile illustrative operation names in the IR contract with governed keys
status: todo
priority: p2
dependencies: []
related: [own-operation-family-support-matrix]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, coherence]
---
The illustrative built-in list in `docs/ir.md` spells operation families as `Constant`, `Cast`, `Reindex`, `Broadcast`, `FloatAdd`, `WrappingAdd`, `CheckedAdd`, `Multiply`, `SaturatingAdd`, `WideningAdd`, `Gelu`, and `Reduce`. None of those spellings is a governed key. The standard semantic registry in `crates/tiler-ir/src/semantic/registry.rs` registers exactly four operations — `tiler::constant-f32@1`, `tiler::multiply-f32@1`, `tiler::add-f32@1`, and `tiler::strict-serial-sum-f32@1` — and the contract text itself later refers to `tiler::add-f32@1` and `tiler::multiply-f32@1` by their real keys when describing the rank-zero scalar admission. So the same document uses two naming systems for the same operations without saying which is which.

`docs/ir.md` does say the list is "illustrative rather than a closed Rust enum", so this is not a false support claim. It is a spelling-coherence hazard: a reader comparing the list against the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix), which records that no `Cast`, `Reindex`, or `Broadcast` key exists, has to infer that `FloatAdd` and `add-f32` are the same family and that `Reduce` is not `strict-serial-sum-f32` but a family that has no key at all.

Decide and record one convention: either give the illustrative list the canonical `namespace::name@version` spelling for families that have a key and a clearly marked hypothetical spelling for families that do not, or state explicitly that illustrative names are prose placeholders that never denote a key. Do not silently rename the governed keys.

Scope note: `docs/ir.md` is `contracts/foundation`. The support matrix in `docs/roadmap.md` is `contracts/navigation` and must be updated in the same change only if this ticket changes a fact the matrix cites.
