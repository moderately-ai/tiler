---
id: encode-abi-expression-identity-in-linear-space
title: Encode ABI expression identity in linear space
status: todo
priority: p1
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/ir, implementation/artifact]
shared_scopes: [project/tickets, contracts/artifacts]
paths: []
tags: [performance, identity, ir]
---
**Measured: a five-operation program has a 13,623-byte kernel-program identity.**

## Fact — the encoding embeds whole subtrees

`expr_key` (`tiler-ir/src/program/abi.rs:570`) embeds full copies of its operands' keys:

```rust
ExprNode::Binary { op, left, right } => {
    push_slice(&mut bytes, &keys[position(*left)]);
    push_slice(&mut bytes, &keys[position(*right)]);
}
```

So a node's key contains a serialization of its entire subtree. On a chain that is O(N^2) total bytes; **on a shared DAG where a node references its predecessor twice, key size doubles per level**, bounded only by the 64 MiB identity budget. The same recurrence appears in `stage_key`, `view_key`, and `deferred_key` (`tiler-ir/src/program/model.rs:1105-1195`, `tiler-artifact/src/program/model.rs:1244`, `:1649`).

It compounds: `builder.rs:597` linear-scans comparing full keys on every node push, so O(N^2) comparisons of O(N)-sized keys.

## The fix is already in this codebase, and it is not hashing

`compute_graph_identity` (`tiler-ir/src/semantic/identity.rs:85`) solves this exact problem correctly — a flat encoding with **canonical integer IDs**, a precomputed exact length (`graph_identity_encoded_len`), `Vec::with_capacity`, and an explicit iterative worklist. It handles a 50,000-node chain in 1.18 s. That is an existence proof that nested keys are a choice rather than a requirement.

**Do not replace the key with a digest.** That trades collision-freedom for speed. The flat-canonical-ID form is O(N) *and* keeps injectivity exactly — strictly better on both axes, with no guarantee given up.

## Related

The artifact identity embeds whole section bodies (`tiler-artifact/src/program/model.rs:1604`) and is then embedded in the manifest and hashed, so payload bytes are hashed twice inside a single encode. That is why the manifest measures ~18 KB for an 8 KB payload. Consider it in the same change or split it explicitly.

## Consequence to accept knowingly

This changes the canonical bytes of every encoded program, invalidating every artifact identity and cache entry, and **requires a domain-tag bump**. Acceptable while there are no external consumers, but it must be a stated decision at the site with the version stepped — never a silent rebaseline. The producer's two-process determinism test and the serial-sum artifact identity are the pins that will detect it.

## Closes when

Expression and stage identity encoding is linear in arena size; a deep-chain and a shared-DAG case are both measured; the injectivity property is unchanged and tested; the identity domain tag is stepped with its reason recorded; and `make full` passes.
