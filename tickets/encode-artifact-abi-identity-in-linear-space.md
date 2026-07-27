---
id: encode-artifact-abi-identity-in-linear-space
title: Encode artifact ABI identity in linear space
status: todo
priority: p1
dependencies: []
related: [encode-abi-expression-identity-in-linear-space]
scopes: [implementation/artifact]
shared_scopes: [project/tickets, contracts/artifacts]
paths: []
tags: [performance, identity, artifact]
---
`encode-abi-expression-identity-in-linear-space` fixed the kernel-program half in `tiler-ir` and deliberately left this one, because it cannot be split.

## Fact — the artifact still nests whole subtrees

`expr_key` (`tiler-ir/src/program/abi.rs`) is unchanged and still embeds each operand's whole key inside the node's own. `tiler-ir` no longer calls it; `tiler-artifact` still does, at every use site of an ABI expression in envelope identity, and in `deferred_key` (`tiler-artifact/src/program/model.rs`) and the launch preconditions `push_sorted_keys` folds. So an artifact identity is still quadratic in arena size along a chain and **still doubles per level wherever one node is shared**, bounded only by the identity budget. That doubling is measured: with the same fixture, a 16-level shared-DAG guard took a kernel-program identity to 5,976,994 bytes before the `v3` step and 3,406 bytes after.

## Why it was not done in the same change

The artifact's per-node key vector is derived in **four** places that must agree byte for byte:

- `tiler-artifact/src/program/builder.rs` (interning)
- `tiler-artifact/src/program/model.rs:encode_identity` (envelope identity)
- `tiler-artifact/src/program/codec/model.rs:expression_keys` (projection, and `canonical_expression_order` — the codec **stores the arena in canonical key order**, so the key ordering is part of the wire format)
- `tiler-artifact/src/program/codec/decode.rs` (re-derivation on decode)

Changing `expr_key`'s shape to the flat canonical-ID form requires changing its signature — a flat encoding needs the arena, not the operands' keys — so all four move together or they stop agreeing. Two of them are under `codec/`, which was concurrently owned by other tickets.

## What to do

Apply the shape `tiler-ir/src/program/model.rs:encode_identity` now uses, and which `tiler-ir/src/semantic/identity.rs` used first:

- Encode the reachable arena **once**, in a canonical numbering seeded by the use sites in the order identity already treats as canonical, and name each use site by canonical position.
- Replace the builder's `expr_key` linear scan with hash-consing on `ExprNode` itself (`Hash` is now derived). The induction that makes a shallow match decide deep structural equality is written out at `KernelProgramBuilder::push_abi_node`.
- Resolve `canonical_expression_order`: the codec sorts the stored arena by key. A canonical numbering is itself a canonical order, so the sort may become unnecessary — decide that explicitly rather than keeping both.
- Step `ARTIFACT_DOMAIN` (currently `v4`) and say why at the site, as `PROGRAM_DOMAIN`'s `v3` step does.

**Do not replace the key with a digest.** The flat canonical-ID form is O(N) *and* keeps injectivity exactly; a digest trades collision-freedom for speed and is strictly worse on one axis for no gain on the other.

## Watch for

`expr_key` also has no remaining `tiler-ir` caller. If the artifact stops using it too, delete it rather than leaving a public quadratic encoder nothing calls.

## Closes when

Artifact expression identity is linear in arena size; a deep-chain and a shared-DAG case are measured with a checked-in instrument as `tiler-ir`'s `abi_identity_size_grows_linearly_with_the_arena` is; injectivity is unchanged and tested; `ARTIFACT_DOMAIN` is stepped with its reason recorded; and `make full` passes.
