---
id: retire-or-re-document-the-now-consumerless-expr-key-in-tiler-ir
title: Retire or re-document the now-consumerless expr_key in tiler-ir
status: awaiting-decision
priority: p2
dependencies: []
related: [replace-the-codec-arena-content-key-with-the-existing-comparator, encode-artifact-abi-identity-in-linear-space]
scopes: [implementation/ir, contracts/decisions, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, cleanup, decision, needs-tom, public-boundary]
---
After `replace-the-codec-arena-content-key-with-the-existing-comparator`, `tiler_ir::program::abi::expr_key` has **no production consumer anywhere in the workspace**. `tiler-ir` stopped calling it at `encode-abi-expression-identity-in-linear-space`; `tiler-artifact` stopped at the artifact identity flattening and, for the codec, at the `14.0` manifest step. The one remaining caller is a `tiler-artifact` codec test, `the_canonical_arena_order_follows_the_comparator_where_the_content_key_disagrees`, which uses it precisely to assert that the key order and `compare_expr_nodes` are *different relations* on a constructed pair.

## Its documentation is now false

`crates/tiler-ir/src/program/abi.rs` still says, under `expr_key`:

> `tiler-artifact` still derives per-node keys this way for envelope identity and for the canonical arena order its codec writes; moving it to the same flat form is `encode-artifact-abi-identity-in-linear-space`, which has to change all four of that crate's key derivations at once or they stop agreeing.

Both clauses are false and the named ticket is `done`. `crates/tiler-ir/src/program/model.rs`, source anchor `` `v2` named each use site's expression ``, carries a second reference in the same spirit.

## The decision, which is not a worker's

Removing a `pub` item from `tiler-ir` is a public crate surface change and belongs to Tom under ADR 0075. The options are not symmetric:

- **Keep and re-document.** The function is the *only* independent statement of what the old canonical arena order was, and the artifact test above is a real consumer of exactly that. Its documentation would be rewritten to say so: retained as the superseded relation, kept public so a cross-crate test can compare the two, with the cost paragraph left intact because it is the derivation the replacement rests on.
- **Retire it.** Nothing in production reaches it. The artifact test would then have to restate the key encoding locally, which reintroduces the second authority the test exists to avoid, or be rewritten to assert the divergence some other way.

The first is the recommendation: a public item with one deliberate test consumer and an accurate docstring is cheaper than a duplicated encoder, and this crate already keeps `expr_key`'s cost paragraph as reference material for why the flat form exists.

## Decision packet — 2026-08-09

The ticket already contains the complete two-option comparison and a recommendation, but was incorrectly left in the implementation queue. Tom must choose whether the public item is retained as the one historical relation used by the cross-crate regression, or removed with an independently truthful replacement for that regression. No worker should remove or re-ratify the public item before that choice.

## Closes when

`expr_key`'s documentation states its current consumer set truthfully, `program/model.rs`'s reference agrees with it, and either the item is retained on Tom's decision or it is removed with the artifact test's divergence assertion preserved by some other means.

## Scope repair — 2026-08-09

`implementation/artifact` is declared because the retire branch must replace the cross-crate codec regression that is the function's sole remaining consumer; IR-only scope could not preserve the ticket's own closing condition.
