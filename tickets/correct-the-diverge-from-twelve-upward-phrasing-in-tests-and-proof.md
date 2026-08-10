---
id: correct-the-diverge-from-twelve-upward-phrasing-in-tests-and-proof
title: Correct the "diverge from twelve upward" phrasing in tests and proof
status: in-progress
priority: p3
dependencies: []
related: [restate-the-tree-width-rule-outside-the-compiler-crate]
scopes: [implementation/conformance, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, documentation]
claimed_from: todo
assignee: terra-diverge-twelve-prose
lease_expires_at: 1786406457
---
## The claim, and why it is wrong

**Fact — two live comments still read universal divergence past twelve.** Both phrases are present at the 2026-08-10 audit base:

- `crates/tiler-conformance/src/serial_sum/tests.rs`, four-contributor portfolio assertion comment, anchor `` they diverge from twelve contributors upward ``.
- `prototypes/serial-sum-run/src/proof.rs`, `declared_partition` doc, anchor `` the two agree at four and diverge from twelve contributors upward ``.

"Diverge from twelve contributors upward" reads as *every* admitting count from twelve on differs. That is false. Under the landed `capped_tree_partition` rule, **1,180 of the 3,530 admitting counts below 4,096 still agree** with `governed_partition`. Twelve is only the *first* differing count (tree `6 x 2`, split `4 x 3`).

**Fact — the same defect class was already repaired once under this parent.** [`restate-the-tree-width-rule-outside-the-compiler-crate`](restate-the-tree-width-rule-outside-the-compiler-crate.md) tightened `crates/tiler-conformance/src/serial_sum.rs`'s `declared_partition` doc from "diverging … upward" to "first diverge at [`SEPARATING_COLUMNS`]" and named the 1,180 agreeing population. The tests.rs portfolio comment and the prototype `proof.rs` doc were left behind — the former inside a held `implementation/conformance` scope, the latter outside claim under `implementation/runtime`.

**Fact — four-contributor agreement itself is still true.** Both sites correctly state that at four contributors the two parallel rules return the same split. Only the universal-upward gloss is wrong.

## What this owes

- Prefer language that matches the repaired conformance doc: "first diverge at twelve" (or at `SEPARATING_COLUMNS` where that constant is in scope), and/or name that agreement is not universal past twelve (1,180 of 3,530 admitting counts below 4,096 still agree).
- Comment- and doc-comment-only edits at the two sites above. No figure, constant, rule, test assertion, or identity change.
- No new cost claim at non-power-of-two contributor counts.

## Closes when

Both residual "diverge from twelve contributors upward" phrases are gone or rewritten so they no longer claim universal divergence past twelve; a reader of either site cannot infer that every count from twelve on differs.

## How it was found

Filed 2026-08-10 by the Phase B ticket-audit repair on [`restate-the-tree-width-rule-outside-the-compiler-crate`](restate-the-tree-width-rule-outside-the-compiler-crate.md). That ticket's own Fact audit already marked the `proof.rs` claim imprecise and never split a remainder; the audit report also found the same phrasing still live in the held-scope conformance test comment.
