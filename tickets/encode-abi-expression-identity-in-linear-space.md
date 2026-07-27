---
id: encode-abi-expression-identity-in-linear-space
title: Encode ABI expression identity in linear space
status: review
priority: p1
dependencies: [measure-compiler-and-artifact-hot-paths]
related: [encode-artifact-abi-identity-in-linear-space]
scopes: [implementation/ir, implementation/artifact]
shared_scopes: [project/tickets, contracts/artifacts]
paths: []
tags: [performance, identity, ir]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785180315
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

## Outcome

Done for `tiler-ir`. Kernel-program identity is linear in arena size, injectivity is unchanged and newly tested, and `PROGRAM_DOMAIN` is stepped to `v3`. The `tiler-artifact` half is split into `encode-artifact-abi-identity-in-linear-space` rather than half-done here; the reason it could not be split *smaller* is recorded there.

### What landed

**The ABI arena is encoded once, not once per reference.** `canonical_arena_traversal` (`crates/tiler-ir/src/program/abi.rs`) numbers every node reachable from the use sites, in the canonical order identity already folds them, operands before the nodes naming them — the shape `semantic/identity.rs` uses. `encode_identity` writes that arena as one framed section and names each use site by an eight-byte canonical position.

**The entity keys were the larger term, and got the same treatment.** The diagnosis in the ticket body was right about `expr_key` but incomplete: measured on a two-stage fixture, the ABI was ~300 bytes of a 13,309-byte identity. The rest was `stage_key` / `value_key` / `view_key` / `allocation_key` nesting — a value key embeds a stage key, a view key embeds a value key, and `encode_identity` then embedded those whole keys at every reference, so a ~1 KB bound-kernel identity was restated about ten times. `encode_identity` now writes each entity once, in a count-prefixed section in canonical order, and cross-references by canonical position. The keys themselves are unchanged: they still rank the entities and the verifier still proves them pairwise distinct, but only one comparison sort pays for the nesting instead of every reference.

**Interning is hash-consing now.** `KernelProgramBuilder::push_abi_node` matched a whole-subtree key by linear scan — O(N) comparisons of O(N)-sized keys per push. It now matches the `ExprNode` itself through a `HashMap`. That shallow match decides deep structural equality because the operands index an already-interned arena; the induction is written out at the function. `ExprNode` gains a derived `Hash` for it.

### Domain tag

`PROGRAM_DOMAIN`: `tiler.kernel-program.v2\0` → `tiler.kernel-program.v3\0`, at `crates/tiler-ir/src/program/model.rs`, with the reason stated there — the *subject* is unchanged and only its encoding moved, which is exactly why a `v2` identity must miss rather than match. No external consumer holds one, so this costs a rebuild rather than a migration.

`ARTIFACT_DOMAIN` is **not** stepped, deliberately: `tiler-artifact`'s encoding rules did not change. Its identity bytes moved because the program section it folds is program content, not because it spells anything differently.

### Values that moved

No pinned golden required rebaselining — the whole workspace suite passes unchanged. Every value below is a *measurement*, reproducible on demand; none is a constant in the tree.

| value | before | after | regenerate with |
| --- | --- | --- | --- |
| Kernel-program identity, two-stage fixture, 6-node arena | 13,309 B | **3,118 B** | `cargo nextest run -p tiler-ir -E 'test(abi_identity_size)' --no-capture` |
| — same fixture, 16-level ABI chain | 14,765 B | **3,409 B** | same |
| — same fixture, 16-level shared-DAG ABI | 5,976,994 B | **3,406 B** | same |
| — growth per added arena node, chain | +91 B | **+18 B** | same |
| — growth per added arena node, shared DAG | ×2 per level | **+18 B** | same |
| Artifact envelope, codec hot-path fixture | 26,126 B | **15,030 B** | `cargo nextest run --release -p tiler-artifact -E 'test(hot_path)' --no-capture` |
| Artifact identity, same fixture | 13,320 B | **7,772 B** | same |
| Artifact decode, same fixture, release | 523 µs | **339 µs** | same |
| — the canonicity re-encode within it | 254 µs | **169 µs** | same |

The `before` column is the same instrument run against `d4f82269550be2ff0b5386fc054bd5861dd4e551` in a detached worktree; the `abi_identity_size` rows required copying the new test file there, since the instrument did not exist before this change.

Timing rows are the **minimum of five runs**, each itself a mean of 50 in-process repeats. Minimum rather than mean because every perturbation a host applies makes a decode slower and none makes it faster, so the distribution has a hard floor and an unbounded tail. Byte counts need no such treatment: they are deterministic and identical on every host.

The 3,118-byte floor is now dominated by the two bound kernels' canonical identities, which are irreducible content — identity must name what implementation each stage binds.

### Evidence in the tree

`crates/tiler-ir/src/program/tests.rs` gains two tests:

- `abi_identity_size_grows_linearly_with_the_arena` — builds the same program with an ABI guard grown to 0..17 levels, as a chain and as a shared DAG, prints the curve, and **asserts the per-level increment is constant**. Its failure path is reachable and was checked: run against the `v2` encoding it reports `SharedDag identity size must grow by a constant per level, measured [182, 364, 728, …, 2981888]`.
- `identity_distinguishes_two_arenas_that_differ_only_in_their_wiring` — two programs holding one `true`, one `false`, and two `Or`s, differing only in what those `Or`s name. A reference encoding that lost operand order or lost which node an operand points at would pass everything else and fail this.

### Injectivity

Unchanged, and argued at both sites (`AbiArenaTraversal::encode` and `encode_identity`). Each section is a framed count followed by that many self-delimiting records, so a reader recovers the exact entity or node list from the bytes alone; a canonical position is eight fixed-width bytes into a section that is complete, so it determines its referent exactly as a full copy of that referent's key did. Fixed width is what lets a position replace a length-prefixed key without losing framing. What the encoding stops *restating*, it does not stop *determining*.

### Not done here, and why

The artifact-side nesting: `encode-artifact-abi-identity-in-linear-space`. `expr_key` is unchanged and still quadratic-on-a-chain, exponential-on-a-shared-DAG; `tiler-ir` no longer calls it and `tiler-artifact` still does, from four places that must agree byte for byte, two of them under `codec/`.
