---
id: own-the-numerical-realization-profile-key
title: Give NumericalRealization an owned profile key
status: todo
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [ir, numerics, serialization]
---
**Fact.** `tiler_ir::schedule::NumericalRealization::profile_key` is `&'static str` (`crates/tiler-ir/src/schedule/numerics.rs`). It therefore names a compile-time constant of the producing build and cannot represent a key read from bytes.

**Consequence, measured by `prototype-neutral-artifact-codec`.** The artifact envelope cannot decode into a `NumericalRealization`. The codec carries a `NumericalFacts` record instead — the same two enum vocabularies, an owned `String` key — solely because of the lifetime. The identity bytes are unchanged (the encoder writes the key's bytes either way), so this is an ergonomics and layering cost, not a correctness one, but it means one numerical vocabulary now has two record spellings in the workspace.

**What closes this.** Either give `NumericalRealization` an owned or interned key so a decoder can rebuild it directly and `NumericalFacts` is deleted, or record that the split is intended and say which record each layer uses. The first changes a public signature in `tiler-ir` and is Tom's under ADR 0075.

**Scope note.** Deliberately left unscoped: the fix touches `implementation/ir`, and the ticket that takes it should claim that scope.
