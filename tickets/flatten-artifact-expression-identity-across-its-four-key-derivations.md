---
id: flatten-artifact-expression-identity-across-its-four-key-derivations
title: Flatten artifact expression identity across its four key derivations
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [performance]
---

Split from `encode-artifact-abi-identity-in-linear-space`, whose builder half landed; read that ticket's Outcome section first.

`expr_key` (`tiler-ir/src/program/abi.rs`) embeds full copies of its operands' keys, so a node's key contains its whole subtree: quadratic on a chain and **doubling per level on a shared DAG**. `tiler-ir` no longer uses it — kernel-program identity went from 13,309 bytes to 3,118, and a 16-level shared-DAG case from 5,976,994 bytes to 3,406, by encoding the arena once in a canonical numbering and naming nodes by canonical position. `tiler-artifact` still derives keys the old way.

## The four sites, which move together or not at all

- `program/model.rs:encode_identity` — `push_slice(&keys[node_at(x)])` at every use site: variant guards, binding accessible-bytes, launch grid/workgroup dimensions, deferred predicates, launch preconditions.
- `codec/model.rs` — `expression_keys` and `canonical_expression_order`. **The codec stores the arena in canonical key order, so the key ordering is part of the wire format.**
- `codec/validate.rs` — validation re-derives keys.
- `codec/decode.rs` — re-derivation, plus the check that the stored arena is in canonical order.

A partially converted set does not merely regress. It stops agreeing, and the disagreement surfaces as an artifact that decodes to a different identity than it was published under — which the identity check catches, but only after the artifact exists.

## What to do

Reuse, do not reimplement. `canonical_arena_traversal` / `AbiArenaTraversal` (`tiler-ir/src/program/abi.rs`) numbers every node reachable from an ordered root list, operands before the nodes naming them. It is `pub(crate)`; making it `pub` is the first step, and a second implementation in `tiler-artifact` would create exactly the two-encoders-must-agree defect this work removes.

The root list must be the use sites **in the order `encode_identity` already writes them**, because that order is part of the numbering. A root list that is not canonical yields a numbering that is not canonical — it does not yield an ambiguous one, since the arena is written in that same numbering rather than against an assumed one.

`canonical_expression_order`'s sort becomes unnecessary: a canonical numbering *is* a canonical order. Decide it explicitly, do not keep both.

Step `ARTIFACT_DOMAIN` (currently `v4`) and say why at the site, as `PROGRAM_DOMAIN`'s `v3` step does. Rebaseline the artifact identity and the serial-sum proof goldens, recording old value, new value, and regeneration command for each.

**Do not replace the key with a digest.** The flat canonical-ID form is O(N) *and* keeps injectivity exactly; a digest trades collision-freedom for speed, strictly worse on one axis for no gain on the other.

If `tiler-artifact` stops calling `expr_key`, delete it rather than leaving a public quadratic encoder nothing calls.

## Closes when

Artifact expression identity is linear in arena size; deep-chain and shared-DAG cases are measured by a checked-in instrument as `tiler-ir`'s `abi_identity_size_grows_linearly_with_the_arena` does; injectivity is unchanged and tested; `ARTIFACT_DOMAIN` is stepped with its reason recorded; every moved golden carries its regeneration command; `make full` passes.
