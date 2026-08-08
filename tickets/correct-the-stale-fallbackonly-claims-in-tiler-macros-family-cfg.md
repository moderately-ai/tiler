---
id: correct-the-stale-fallbackonly-claims-in-tiler-macros-family-cfg
title: Correct the stale FallbackOnly claims in tiler-macros family_cfg
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/frontend]
shared_scopes: []
paths: []
tags: []
---
## Two comments in `crates/tiler-macros/src/family_cfg.rs` assert an absence the crate refutes

Found while correcting `docs/roadmap.md`'s Milestone 0B text at base `44a85cfc`. Both sites are comments, so nothing fails; the claim is simply false.

**Fact.** The module doc states: "A region can state a selected family since Tom accepted the `deliver` statement, but no expansion compiles one — the statement is refused before emission — so every expansion delivers `FallbackOnly`, which ADR 0053 defines as invoking no backend compiler." (`crates/tiler-macros/src/family_cfg.rs "so every expansion delivers"`)

**Fact.** The `#[allow(dead_code, reason = ...)]` on `MAP_VERSION` restates it: "Nothing reads it during an expansion yet because every expansion delivers `FallbackOnly` — a stated selected family is refused before emission, since nothing compiles one".

**Fact — both are refuted by the same crate.** `crates/tiler-macros/src/aot.rs "pub(crate) fn deliver"` compiles a stated `macos` selection through `accept_or_publish_metal_plan`, and `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs "The complete inline AOT workflow, in an ordinary consumer crate"` is an out-of-tree consumer crate whose compilation runs it. `consumer_cfg` is reached from `crate::delivery`'s `items_source` for every selected family, so an expansion that delivers does embed a predicate from this table.

**Inference — the narrow claim underneath may still hold and should be checked rather than assumed.** `MAP_VERSION` itself appears to be read by nothing outside this module's tests, so the `dead_code` allowance may be correct while its stated reason is not. Verify before editing: the fix may be to correct the reason rather than to remove the allowance.

## Why this is a separate ticket

The roadmap correction that found it is scoped `contracts/navigation` and may not touch `crates/**`. AGENTS.md treats comments and examples as claims about current behaviour, which is what makes this a defect rather than untidiness.

## Closes when

Both sites state what the crate does at this base, the `dead_code` allowance is either justified by a true reason or removed on evidence, and the package's Clippy, rustdoc, and tests pass.
