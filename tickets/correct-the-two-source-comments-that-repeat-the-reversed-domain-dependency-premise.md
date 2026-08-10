---
id: correct-the-two-source-comments-that-repeat-the-reversed-domain-dependency-premise
title: Correct the two source comments that repeat the reversed domain dependency premise
status: todo
priority: p2
dependencies: []
related: [repair-the-artifact-abis-stale-cross-crate-no-prefix-argument]
scopes: [implementation/artifact, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Fact — two live source comments retain the retired dependency and namespace argument

`crates/tiler-artifact/src/program/codec/tests.rs`, anchor `since neither depends on the other`, says no crate can hold a cross-crate check and also says the IR population opens only `tiler.ir.`. `crates/tiler-ir/src/index/refinement.rs`, anchor `so neither crate can enumerate the union`, repeats the reversed dependency explanation and first-differing-byte namespace argument. The artifact manifest has a live `tiler-ir.workspace = true` dependency, while the reverse edge is absent; the complete IR pin population is private and test-only rather than unreachable because of dependency direction. `EXPR_DOMAIN` also lives under `tiler.artifact-program.`, so the namespace premise is false.

## Outcome

Replace both live comments with the count-free spelling-and-terminator argument already established by the source authority and accepted artifact ABI contract. Preserve the local no-prefix checks and their ownership limits. Do not change any production domain bytes, public boundary, encoder, schema, or identity.

## Checks

- Re-read both complete source files, the complete domain census modules, both crate manifests, and the accepted cross-crate paragraph before editing.
- Run `cargo test -p tiler-artifact domains::` and `cargo test -p tiler-ir domains::`.
- Run `cargo fmt --all --check`, package Clippy/rustdoc as applicable, `make citations`, `tkt lint`, `git diff --check`, and exact-base `tkt guard`.
- Negative grep must show the two retired live anchors are absent outside dated correction or ticket history.
