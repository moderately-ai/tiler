---
id: flatten-artifact-expression-identity-across-its-four-key-derivations
title: Flatten artifact expression identity across its four key derivations
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
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

## Finding 2026-07-27 — the `tiler-ir` shape does not transfer directly, and why

**This ticket says "reuse, do not reimplement" and it is right about the traversal. It assumes the `tiler-ir` call shape transfers with it, and that part is false.** The obstacle is small to state and decides the design, so it is recorded before any code moves.

**Fact — `tiler-ir` has no sorted expression set; `tiler-artifact` has two.** `crates/tiler-ir/src/program/model.rs`'s `abi_use_sites` builds its root list purely positionally: the applicability guard, then per stage the grid, workgroup, and access sites, in stage order. Nothing there is sorted by an expression-derived key. `tiler-artifact`'s `encode_identity` sorts two expression sets by their key bytes — the deferred feasibility predicates (`push_sorted_keys(... deferred_key(keys, predicate) ...)`) and each entry's launch preconditions — because which obligation the producer happened to enumerate first is not meaning.

**Inference — replacing keys with canonical IDs is circular at exactly those two sites.** The sort needs a per-expression key; the canonical ID is that key; the ID comes from the numbering; the numbering comes from the root order; and the root order is the sorted order. `tiler-ir` never met this because its roots are positional.

**Three ways out, and the elimination:**

1. **Root the numbering in declaration order and sort the encoded set by canonical ID.** Breaks the cycle and breaks canonicity with it: an expression reachable *only* from a deferred predicate is numbered by the producer's enumeration order, so two artifacts differing only in that order get different identities. That is the regression the sort exists to prevent. **Rejected.**
2. **Keep a structural key solely to order the two sets, and use canonical IDs everywhere else.** Retains something `expr_key`-shaped, which is the quadratic encoder this ticket exists to delete — unless the ordering key is cheap. **Survives only in the form below.**
3. **Order the two sets by a digest of the subtree, used for ordering only.** This is the option worth stating explicitly, because it looks like the thing the ticket forbids and is not. The ticket's "do not replace the key with a digest" is about *identity*: a digest there would trade injectivity for speed. An ordering digest costs no injectivity at all, because the arena is still encoded in full and in canonical order — a collision would make the *order* of two set members ambiguous, not the identity ambiguous, and it can be broken deterministically by comparing the subtrees. **Survives.**

**So the shape of the work is: option 3 or option 2-with-a-cheap-key, decided explicitly, and the decision belongs in the same change** — it is what `canonical_expression_order`'s removal turns into. The ticket already says "decide it explicitly, do not keep both"; this is the decision it was pointing at, now with the reason it cannot be skipped.

**Nothing was changed.** The four sites move together or not at all, and starting the conversion before this is settled would produce exactly the partially-converted state the ticket warns about. The traversal in `tiler-ir` is still the right thing to reuse and still needs promoting from `pub(crate)` to `pub` as step one.

**Also confirmed while reading, and it enlarges the change beyond the four named sites:** `tiler-artifact`'s builder uses `expr_key` for *deduplication* (`crates/tiler-artifact/src/program/builder.rs` inserts `node -> expression_keys.len()` keyed by the node's key), not only for identity. A canonical numbering is not available at build time — it is a function of the finished use-site set — so the dedup needs its own answer, and "delete `expr_key` if `tiler-artifact` stops calling it" cannot be satisfied until that answer exists.

### Correction to the finding above — a structural comparator beats the ordering digest

**Verified first, because the whole cycle depends on it:** neither set is canonicalized at build time. `check_deferred` and the launch-precondition loop in `crates/tiler-artifact/src/program/builder.rs` both preserve the caller's declaration order and only reject duplicates. So the sort in `encode_identity` is the *only* thing making those orders content-derived, and rooting the numbering in declaration order would genuinely lose canonicity, as stated.

**But option 3 is not the best resolution, and the better one needs no digest.** Sort the two sets with a **structural comparator over the expression DAG** — compare two nodes by constructor tag, then by their operands recursively — rather than by any materialized key.

Why it is better on every axis that matters here:

- **It breaks the cycle completely.** The comparator needs no numbering, so there is no root order to depend on. Sort first, then number from the sorted roots, then encode. No circularity to design around.
- **It is exactly injective**, so the digest-collision question never arises and no tie-break rule is needed. That removes the one place where the ordering digest asked a reader to accept "a collision would only make an order ambiguous".
- **It keeps the complexity win.** The materialization this ticket exists to delete is quadratic on a chain and doubling per level on a shared DAG *because every node's key embeds its whole subtree*. A comparator walks two subtrees and stops at the first difference; it never materializes one. Sorting `n` members costs `n log n` comparisons bounded by subtree size, against the current cost of building `n` keys each of which may be exponential in depth.
- **It is the smaller change.** Nothing new is governed: no ordering domain, no digest constant, no algorithm tag.

**So the shape is:** sort the deferred predicates and launch preconditions by structural comparison; take the root list in the order `encode_identity` writes them, which is now well-defined because the sorted order no longer depends on the numbering; number once with `canonical_arena_traversal`; encode the arena once; replace every `keys[node_at(x)]` with the fixed-width canonical ID. `canonical_expression_order`'s sort then goes, because the numbering *is* the order — which is what the ticket asked to decide explicitly.

**The builder's dedup still needs its own answer** and is unaffected by this correction: it keys `node -> position` at build time, where no canonical numbering exists yet. Structural equality is what it actually wants, and it is available at build time — but confirming that is work this ticket has not done.

## Progress 2026-07-27 — step one landed: the shared primitives exist

**Landed** (`tiler-ir`, no behaviour change, 274 tests pass):

- `AbiArenaTraversal`, `canonical_arena_traversal`, and its four methods are now `pub`, which this ticket names as the first step. `tiler-artifact` can reuse them instead of growing the second implementation that would recreate the two-encoders-must-agree defect.
- **`compare_expr_nodes` is new**, and it is what makes the reuse possible at all — see the correction above for why the `tiler-ir` call shape does not transfer without it.
- `the_structural_comparator_is_a_total_order` checks reflexivity, antisymmetry, transitivity, and that no two structurally distinct nodes tie, exhaustively over every ordered pair and triple of an arena carrying all four constructors with sharing. A merely *consistent* comparator would not do: an intransitive one makes `sort_by` produce an order that depends on the input permutation, which is exactly the canonicity the sort exists to provide.
- Verified to bite: ignoring operands in the `Binary` arm makes `a + b` and `b + a` tie, and the test fails naming those two nodes.

**Not started, and it is the atomic part.** The four derivation sites still use `expr_key`. They move together or not at all, so nothing above touches them.

### What the next session does, in order

1. In `encode_identity`, sort `variant.deferred` and each entry's `launch.preconditions` with `compare_expr_nodes`. Neither is canonicalized at build time — verified: `check_deferred` and the launch-precondition loop preserve caller order and only reject duplicates — so this sort is where their order becomes content-derived.
2. Build the root list in the order `encode_identity` writes it, which is now well-defined: variant program section, guard, deferred (sorted), then per entry the bindings' `accessible_bytes`, `launch.grid_threads`, `launch.threads_per_workgroup`, preconditions (sorted).
3. `canonical_arena_traversal` over that list; `arena.encode` once; replace every `push_slice(&keys[node_at(x)])` with the fixed-width `arena.canonical_id(x).to_be_bytes()`.
4. The same numbering in `codec/model.rs`, `codec/validate.rs`, and `codec/decode.rs`; delete `canonical_expression_order`, because the numbering *is* the order.
5. Step `ARTIFACT_DOMAIN` `v4` → `v5` with the reason at the site, and rebaseline the artifact identity and serial-sum proof goldens, recording old value, new value, and regeneration command for each.
6. Add the linearity instrument, mirroring `abi_identity_size_grows_linearly_with_the_arena` in `tiler-ir`'s `program/tests.rs`, over chain and shared-DAG growth.

**The builder's `expr_key` dedup is still unanswered** and blocks step 6's "delete `expr_key`". It keys `node -> position` at build time where no canonical numbering exists. Structural equality is what it wants and `compare_expr_nodes` now supplies it, but converting the dedup is its own change and was not attempted.

## Outcome — flattened, `v5`, measured against the `v4` it replaces (2026-07-27)

**Measurement, and it is the whole point.** The instrument is `artifact_identity_size_grows_linearly_with_the_abi_arena` in `crates/tiler-artifact/src/program/tests.rs`, mirroring `tiler-ir`'s. The `v4` column was produced by grafting the same instrument onto a clean detached worktree at `origin/main`, so both columns come from one measurement rather than from a remembered figure.

| ABI arena | `v4` identity | `v5` identity |
| --- | --- | --- |
| 16-level shared DAG (24 nodes) | **5,971,457 bytes** | **7,669 bytes** |
| 16-level chain (25 nodes) | 9,228 bytes | 7,687 bytes |
| growth per level | doubling on a shared DAG | **constant, 18 bytes** |

The `v4` shared-DAG column doubles exactly as the ticket said: 100,865 → 194,049 → 380,417 → 753,153 → 1,498,625 → 2,989,569 → 5,971,457. `v5` adds 18 bytes per level on both shapes, and the test asserts the increment is constant rather than merely smaller — a quadratic curve would still shrink the number while failing the property.

**All four sites moved together**, plus two the ticket did not name:

- `program/model.rs::encode_identity` — the arena is written once by `arena.encode`, and every use site is a fixed-width canonical position via `push_abi_reference`.
- `codec/model.rs` — stores the deferred set in the shared canonical order.
- `codec/validate.rs` — checks the stored order *against that same shared definition* rather than re-deriving a key sort, so the two cannot drift into two definitions of "canonical" that only happen to agree.
- `codec/decode.rs` — unchanged in the end: it re-derives through the same helpers.
- **Not named by the ticket:** `deferred_key` now takes the traversal instead of a key table, and `a_non_canonical_deferred_predicate_order_is_rejected` was rewritten to forge against the canonical order rather than against a key comparison.

**The sorted-set cycle was resolved with the structural comparator**, per the correction above: the two expression sets are ordered by `compare_expr_nodes` before the arena is numbered, so nothing circular arises, and the order is exactly injective with no tie-break rule.

**One thing the `tiler-ir` model got wrong for this crate, found by a failing test rather than by reading.** `tiler-ir` asserts every arena node is reached by a use site, because verification precedes identity there. In `tiler-artifact` it does not: `an_expression_no_use_site_reaches_is_rejected` and `an_empty_portfolio_is_rejected` both derive an identity for an envelope that is *about to be refused*, and a `debug_assert` copied from `tiler-ir` fired on both. The root list therefore ends with every remaining arena position, so the numbering is total and a malformed envelope gets its typed rejection instead of a panic inside the traversal. For a valid artifact the tail adds nothing and the numbering is exactly the use-site one — which is why the measurements above are unaffected by it.

**`ARTIFACT_DOMAIN` is `v5`**, with both reasons recorded at the constant: the encoding change, and the fact that the two sets' canonical *order* changed as well as their spelling. No golden moved, because nothing in the repository pins an artifact identity byte string or restates the domain — verified by searching for `artifact-program.v4` across `docs/`, `tickets/`, `crates/`, and `prototypes/`, which returns nothing. The serial-sum proof derives its identities rather than pinning them, so it re-derives under `v5` and the 30-case hardware matrix passes unchanged.

`make full` green: 1,023 workspace tests and 297 release-profile tests.

**Left open, as flagged:** `expr_key` still exists because the builder uses it for *deduplication*, not identity, and a canonical numbering is not available at build time. `compare_expr_nodes` now supplies the structural equality that dedup actually wants, so converting it is a contained follow-up — but it is a separate change and was not attempted here.
